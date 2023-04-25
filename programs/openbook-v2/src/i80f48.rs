use fixed::types::I80F48;

pub trait ClampToInt {
    fn clamp_to_i64(&self) -> i64;
    fn clamp_to_u64(&self) -> u64;
}

impl ClampToInt for I80F48 {
    fn clamp_to_i64(&self) -> i64 {
        if *self <= i64::MIN {
            i64::MIN
        } else if *self >= i64::MAX {
            i64::MAX
        } else {
            self.to_num::<i64>()
        }
    }

    fn clamp_to_u64(&self) -> u64 {
        if *self <= 0 {
            0
        } else if *self >= u64::MAX {
            u64::MAX
        } else {
            self.to_num::<u64>()
        }
    }
}

impl ClampToInt for f64 {
    fn clamp_to_i64(&self) -> i64 {
        if *self <= i64::MIN as f64 {
            i64::MIN
        } else if *self >= i64::MAX as f64 {
            i64::MAX
        } else {
            *self as i64
        }
    }

    fn clamp_to_u64(&self) -> u64 {
        if *self <= 0.0 {
            0
        } else if *self >= u64::MAX as f64 {
            u64::MAX
        } else {
            *self as u64
        }
    }
}

impl ClampToInt for u64 {
    fn clamp_to_i64(&self) -> i64 {
        if *self >= i64::MAX as u64 {
            i64::MAX
        } else {
            *self as i64
        }
    }

    fn clamp_to_u64(&self) -> u64 {
        *self
    }
}

pub trait LowPrecisionDivision {
    fn checked_div_30bit_precision(&self, rhs: I80F48) -> Option<I80F48>;
    fn checked_div_f64_precision(&self, rhs: I80F48) -> Option<I80F48>;
}

impl LowPrecisionDivision for I80F48 {
    /// Divide by taking the top 64 bits of self, and top 32 bits of rhs. Then divide
    /// those as u64 and shift everything back. Leads to a division result that has the
    /// first 30 bits correct.
    fn checked_div_30bit_precision(&self, rhs: I80F48) -> Option<I80F48> {
        let a = self;
        let b = rhs;

        if b.is_zero() {
            return None;
        }

        let a_neg = a.is_negative();
        let b_neg = b.is_negative();
        let r_neg = a_neg ^ b_neg;

        let an_bits = if a_neg { -a.to_bits() } else { a.to_bits() } as u128;
        let bn_bits = if b_neg { -b.to_bits() } else { b.to_bits() } as u128;

        let an_zeros = an_bits.leading_zeros();
        let bn_zeros = bn_bits.leading_zeros();

        // shift (positive means to the left) and
        // ar has the high bit be 1
        let (an_shift, ar) = if an_zeros >= 64 {
            let s = an_zeros - 64;
            (s as i32, (an_bits << s) as u64)
        } else {
            let s = 64 - an_zeros;
            (-(s as i32), (an_bits >> s) as u64)
        };

        // br has the first u32 be zero
        let (bn_shift, br) = if bn_zeros >= 96 {
            (0, bn_bits as u64)
        } else {
            let s = 96 - bn_zeros;
            (-(s as i32), (bn_bits >> s) as u64)
        };

        let rr = ar / br;
        let s = 48 + bn_shift - an_shift;
        let rr_zeros = rr.leading_zeros() as i32;
        let r = if 63 + rr_zeros < s {
            return None; // overflow
        } else if s >= 0 {
            (rr as u128) << s
        } else {
            (rr as u128) >> (-s)
        };

        Some(I80F48::from_bits(if r_neg {
            -(r as i128)
        } else {
            r as i128
        }))
    }

    /// Convert to f64 and divide those.
    fn checked_div_f64_precision(&self, rhs: I80F48) -> Option<I80F48> {
        I80F48::checked_from_num(self.to_num::<f64>() / rhs.to_num::<f64>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_i80f48_mul_rounding() {
        // It's not desired, but I80F48 seems to round to -inf
        let price = I80F48::from_num(0.04);
        let x = I80F48::from_bits(96590783907000000);
        assert_eq!((x * price).to_string(), "13.726375969298193");
        assert_eq!(((-x) * price).to_string(), "-13.726375969298196");
    }

    /// Random I80F48, uniformly distributed over number of value bits.
    fn rand_i80f48() -> I80F48 {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bits = rng.gen_range(0..128);
        if bits == 0 {
            return I80F48::ZERO;
        }
        let val = rng.gen_range(2u128.pow(bits - 1)..2u128.pow(bits));
        I80F48::from_bits(if rng.gen::<bool>() {
            -(val as i128)
        } else {
            val as i128
        })
    }

    /// Check that I80F48 division is correct
    #[test]
    pub fn test_i80f48_div() {
        for _ in 0..10000 {
            let a = rand_i80f48();
            let b = rand_i80f48();
            let actual = a.checked_div(b);
            let expected = (|| {
                use num::BigInt;
                if b.is_zero() {
                    return None;
                }
                let a = BigInt::from(a.to_bits()) * BigInt::from(2).pow(128);
                let b = BigInt::from(b.to_bits());
                let r: BigInt = (a / b) >> 80;
                if r > BigInt::from(i128::MAX) || r < BigInt::from(i128::MIN) {
                    return None;
                }
                let (sign, digits) = r.to_u64_digits();
                assert!(digits.len() < 3);
                let rv = match digits.len() {
                    0 => 0u128,
                    1 => digits[0] as u128,
                    2 => digits[0] as u128 + (digits[1] as u128) * (1 << 64),
                    _ => 0,
                };
                Some(I80F48::from_bits(match sign {
                    num::bigint::Sign::Minus => -(rv as i128),
                    _ => rv as i128,
                }))
            })();
            assert_eq!(actual.is_some(), expected.is_some());
            if actual.is_some() && expected.is_some() {
                let actual = actual.unwrap();
                let expected = expected.unwrap();
                if actual.is_positive() {
                    assert_eq!(actual, expected);
                } else {
                    // BigInt shr rounds differently than I80F48 div
                    assert!(actual == expected || actual == expected + I80F48::DELTA);
                }
            }
        }
    }

    #[test]
    pub fn test_i80f48_div_30bit_sanity() {
        let one = I80F48::ONE;
        let two = I80F48::from(2);
        let half = I80F48::from_num(0.5);
        assert_eq!(one.checked_div_30bit_precision(one), Some(one));
        assert_eq!(one.checked_div_30bit_precision(-one), Some(-one));
        assert_eq!((-one).checked_div_30bit_precision(one), Some(-one));
        assert_eq!((-one).checked_div_30bit_precision(-one), Some(one));
        assert_eq!(half.checked_div_30bit_precision(half), Some(one));
        assert_eq!(two.checked_div_30bit_precision(two), Some(one));
        assert_eq!(one.checked_div_30bit_precision(two), Some(half));
        assert_eq!(one.checked_div_30bit_precision(half), Some(two));
        assert_eq!(two.checked_div_30bit_precision(half), Some(I80F48::from(4)));
        assert_eq!(
            one.checked_div_30bit_precision(I80F48::from_bits(1)),
            Some(I80F48::from_bits(1 << 96))
        );

        for i in 0..127 {
            for j in 0..127 {
                println!("i {i}, j {j}");
                let a = I80F48::from_bits(1 << i);
                let b = I80F48::from_bits(1 << j);
                let s = i + 48 - j;
                let r = if s >= 0 && s < 127 {
                    Some(I80F48::from_bits(1 << s))
                } else if s < 0 {
                    Some(I80F48::ZERO)
                } else {
                    None
                };
                assert_eq!(a.checked_div_30bit_precision(b), r);
            }
        }
    }

    #[test]
    pub fn test_i80f48_div_30bit_random() {
        for _ in 0..10000 {
            let a = rand_i80f48();
            let b = rand_i80f48();
            let actual = a.checked_div_30bit_precision(b);
            let expected = a.checked_div(b);
            assert_eq!(actual.is_some(), expected.is_some());
            if actual.is_some() && expected.is_some() {
                let actual = actual.unwrap();
                let expected = expected.unwrap();
                let precision_bits = {
                    let zexp = expected.abs().to_bits().leading_zeros();
                    let err = (actual - expected).abs();
                    let zerr = err.to_bits().leading_zeros();
                    zerr as i32 - zexp as i32
                };
                let sign_ok = actual.is_negative() == expected.is_negative();

                // either at least 30 bits of precision, or all but the last bit correct (last bit may round differently)
                assert!(
                    (precision_bits >= 30 && sign_ok) || (actual - expected).abs() <= I80F48::DELTA
                );
            }
        }
    }

    #[test]
    pub fn test_i80f48_div_f64_random() {
        for _ in 0..10000 {
            let a = rand_i80f48();
            let b = rand_i80f48();
            let actual = a.checked_div_f64_precision(b);
            let expected = a.checked_div(b);
            assert_eq!(actual.is_some(), expected.is_some());
            if actual.is_some() && expected.is_some() {
                let actual = actual.unwrap();
                let expected = expected.unwrap();
                let precision_bits = {
                    let zexp = expected.abs().to_bits().leading_zeros();
                    let err = (actual - expected).abs();
                    let zerr = err.to_bits().leading_zeros();
                    zerr as i32 - zexp as i32
                };
                let sign_ok = actual.is_negative() == expected.is_negative();

                // either at least 50 bits of precision, or all but the last bit correct (last bit may round differently)
                assert!(
                    (precision_bits >= 50 && sign_ok) || (actual - expected).abs() <= I80F48::DELTA
                );
            }
        }
    }
}
