// Copyright © 2018–2021 Trevor Spiteri

// This library is free software: you can redistribute it and/or
// modify it under the terms of either
//
//   * the Apache License, Version 2.0 or
//   * the MIT License
//
// at your option.
//
// You should have recieved copies of the Apache License and the MIT
// License along with the library. If not, see
// <https://www.apache.org/licenses/LICENSE-2.0> and
// <https://opensource.org/licenses/MIT>.

use az_crate::WrappingAs;

#[derive(Clone, Copy, Debug)]
pub struct U256 {
    pub lo: u128,
    pub hi: u128,
}

#[derive(Clone, Copy, Debug)]
pub struct I256 {
    pub lo: u128,
    pub hi: i128,
}

#[inline]
pub fn u256_wrapping_as_i256(a: U256) -> I256 {
    I256 {
        lo: a.lo,
        hi: a.hi.wrapping_as::<i128>(),
    }
}

#[inline]
pub fn wrapping_add_u256_u128(a: U256, b: u128) -> U256 {
    let (lo, carry) = a.lo.overflowing_add(b);
    let hi = a.hi.wrapping_add(u128::from(carry));
    U256 { lo, hi }
}

#[inline]
pub fn overflowing_add_u128_u256(a: u128, b: U256) -> (u128, bool) {
    let (lo, carry) = a.overflowing_add(b.lo);
    (lo, carry | (b.hi != 0))
}

#[inline]
pub fn overflowing_sub_u128_u256(a: u128, b: U256) -> (u128, bool) {
    let (lo, borrow) = a.overflowing_sub(b.lo);
    (lo, borrow | (b.hi != 0))
}

#[inline]
pub fn wrapping_neg_u256(a: U256) -> U256 {
    let (lo, carry) = (!a.lo).overflowing_add(1);
    let hi = (!a.hi).wrapping_add(u128::from(carry));
    U256 { lo, hi }
}

#[inline]
pub fn overflowing_add_i256_i128(a: I256, b: i128) -> (I256, bool) {
    let b = I256 {
        lo: b.wrapping_as::<u128>(),
        hi: b >> 127,
    };
    let (lo, carry) = a.lo.overflowing_add(b.lo);
    // b.hi is in {-1, 0}, and carry is in {0, 1}, so we can add them wrappingly
    let b_hi_plus_carry = b.hi.wrapping_add(i128::from(carry));
    let (hi, overflow) = a.hi.overflowing_add(b_hi_plus_carry);
    (I256 { lo, hi }, overflow)
}

#[inline]
fn u128_lo_hi(u: u128) -> (u128, u128) {
    (u & !(!0 << 64), u >> 64)
}

#[inline]
fn u128_from_lo_hi(lo: u128, hi: u128) -> u128 {
    maybe_assert!(hi >> 64 == 0);
    lo + (hi << 64)
}

#[inline]
fn i128_lo_hi(i: i128) -> (i128, i128) {
    (i & !(!0 << 64), i >> 64)
}

#[inline]
fn i128_from_lo_hi(lo: i128, hi: i128) -> i128 {
    maybe_assert!(hi >> 64 == 0);
    lo + (hi << 64)
}

#[inline]
pub fn wide_mul_u128(lhs: u128, rhs: u128) -> U256 {
    let (ll, lh) = u128_lo_hi(lhs);
    let (rl, rh) = u128_lo_hi(rhs);
    let ll_rl = ll.wrapping_mul(rl);
    let lh_rl = lh.wrapping_mul(rl);
    let ll_rh = ll.wrapping_mul(rh);
    let lh_rh = lh.wrapping_mul(rh);

    let col01 = ll_rl;
    let (col01_lo, col01_hi) = u128_lo_hi(col01);
    let partial_col12 = lh_rl + col01_hi;
    let (col12, carry_col3) = partial_col12.overflowing_add(ll_rh);
    let carry_col3 = carry_col3 as u128;
    let (col12_lo, col12_hi) = u128_lo_hi(col12);
    let (carry_col3_lo, _) = u128_lo_hi(carry_col3);
    let ans01 = u128_from_lo_hi(col01_lo, col12_lo);
    let ans23 = u128_from_lo_hi(lh_rh + col12_hi, carry_col3_lo);
    U256 {
        lo: ans01,
        hi: ans23,
    }
}

#[inline]
pub fn wide_mul_i128(lhs: i128, rhs: i128) -> I256 {
    let (ll, lh) = i128_lo_hi(lhs);
    let (rl, rh) = i128_lo_hi(rhs);
    let ll_rl = ll.wrapping_mul(rl);
    let lh_rl = lh.wrapping_mul(rl);
    let ll_rh = ll.wrapping_mul(rh);
    let lh_rh = lh.wrapping_mul(rh);

    let col01 = ll_rl as u128;
    let (col01_lo, col01_hi) = u128_lo_hi(col01);
    let partial_col12 = lh_rl + col01_hi as i128;
    let (col12, carry_col3) = partial_col12.overflowing_add(ll_rh);
    let carry_col3 = if carry_col3 {
        if col12 < 0 {
            1i128
        } else {
            -1i128
        }
    } else {
        0i128
    };
    let (col12_lo, col12_hi) = i128_lo_hi(col12);
    let (carry_col3_lo, _) = i128_lo_hi(carry_col3);
    let ans01 = u128_from_lo_hi(col01_lo, col12_lo as u128);
    let ans23 = i128_from_lo_hi(lh_rh + col12_hi, carry_col3_lo);
    I256 {
        lo: ans01,
        hi: ans23,
    }
}

#[inline]
pub fn shl_u256_max_128(a: U256, sh: u32) -> U256 {
    if sh == 0 {
        a
    } else if sh == 128 {
        U256 { lo: a.hi, hi: 0 }
    } else {
        U256 {
            lo: (a.lo >> sh) | (a.hi << (128 - sh)),
            hi: a.hi >> sh,
        }
    }
}

#[inline]
pub fn shl_i256_max_128(a: I256, sh: u32) -> I256 {
    if sh == 0 {
        a
    } else if sh == 128 {
        I256 {
            lo: a.hi as u128,
            hi: a.hi >> 127,
        }
    } else {
        I256 {
            lo: (a.lo >> sh) | (a.hi << (128 - sh)) as u128,
            hi: a.hi >> sh,
        }
    }
}

#[inline]
pub fn div_half_u128(r: &mut u128, d: u128, next_half: u128) -> u128 {
    let (dl, dh) = u128_lo_hi(d);
    let (mut q, rr) = (*r / dh, *r % dh);
    let m = q * dl;
    *r = u128_from_lo_hi(next_half, rr);
    if *r < m {
        q -= 1;
        let (new_r, overflow) = r.overflowing_add(d);
        *r = if !overflow && new_r < m {
            q -= 1;
            new_r.wrapping_add(d)
        } else {
            new_r
        };
    }
    *r = r.wrapping_sub(m);
    q
}

#[inline]
pub fn div_rem_u256_u128(mut n: U256, mut d: u128) -> (U256, u128) {
    assert!(d != 0, "division by zero");
    let zeros = d.leading_zeros();
    let mut r = if zeros == 0 {
        0
    } else {
        d <<= zeros;
        let n2 = n.hi >> (128 - zeros);
        n.hi = n.hi << zeros | n.lo >> (128 - zeros);
        n.lo <<= zeros;
        n2
    };

    let (nhl, nhh) = u128_lo_hi(n.hi);
    let qhh = div_half_u128(&mut r, d, nhh);
    let qhl = div_half_u128(&mut r, d, nhl);
    let (nll, nlh) = u128_lo_hi(n.lo);
    let qlh = div_half_u128(&mut r, d, nlh);
    let qll = div_half_u128(&mut r, d, nll);
    let q = U256 {
        lo: u128_from_lo_hi(qll, qlh),
        hi: u128_from_lo_hi(qhl, qhh),
    };
    r >>= zeros;
    (q, r)
}

#[inline]
pub fn div_rem_i256_i128(n: I256, d: i128) -> (I256, i128) {
    let (n_neg, n_abs) = if n.hi < 0 {
        let (nl, overflow) = n.lo.overflowing_neg();
        let nh = if overflow {
            !n.hi as u128
        } else {
            n.hi.wrapping_neg() as u128
        };
        (true, U256 { lo: nl, hi: nh })
    } else {
        let nl = n.lo;
        let nh = n.hi as u128;
        (false, U256 { lo: nl, hi: nh })
    };
    let (d_neg, d_abs) = if d < 0 {
        (true, d.wrapping_neg() as u128)
    } else {
        (false, d as u128)
    };

    let (q_abs, r_abs) = div_rem_u256_u128(n_abs, d_abs);

    let q = if n_neg != d_neg {
        let (ql, overflow) = q_abs.lo.overflowing_neg();
        let qh = if overflow {
            !q_abs.hi as i128
        } else {
            q_abs.hi.wrapping_neg() as i128
        };
        I256 { lo: ql, hi: qh }
    } else {
        let ql = q_abs.lo;
        let qh = q_abs.hi as i128;
        I256 { lo: ql, hi: qh }
    };
    let r = if n_neg {
        r_abs.wrapping_neg() as i128
    } else {
        r_abs as i128
    };
    (q, r)
}

#[inline]
pub fn overflowing_shl_u256_into_u128(a: U256, sh: u32) -> (u128, bool) {
    if sh == 128 {
        (a.hi, false)
    } else if sh == 0 {
        (a.lo, a.hi != 0)
    } else {
        let lo = a.lo >> sh;
        let hi = a.hi << (128 - sh);
        (lo | hi, a.hi >> sh != 0)
    }
}

#[inline]
pub fn overflowing_shl_i256_into_i128(a: I256, sh: u32) -> (i128, bool) {
    if sh == 128 {
        (a.hi, false)
    } else if sh == 0 {
        let ans = a.lo as i128;
        (ans, a.hi != ans >> 127)
    } else {
        let lo = (a.lo >> sh) as i128;
        let hi = a.hi << (128 - sh);
        let ans = lo | hi;
        (ans, a.hi >> sh != ans >> 127)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_udiv_rem(num: U256, den: u128) {
        let (quot, rem) = div_rem_u256_u128(num, den);
        assert!(rem <= den);

        let ql_d = wide_mul_u128(quot.lo, den);
        let qh_d = wide_mul_u128(quot.hi, den);
        assert!(qh_d.hi == 0);
        let prod_lo = ql_d.lo;
        let prod_hi = ql_d.hi.checked_add(qh_d.lo).unwrap();
        let (sum_lo, carry) = prod_lo.overflowing_add(rem);
        let sum_hi = prod_hi.checked_add(u128::from(carry)).unwrap();
        assert!(sum_lo == num.lo && sum_hi == num.hi);
    }

    fn check_idiv_rem_signs(num: I256, den: i128) {
        let (quot, rem) = div_rem_i256_i128(num, den);
        assert!(rem.unsigned_abs() <= den.unsigned_abs());

        if num.hi < 0 {
            assert!(rem <= 0);
        } else {
            assert!(rem >= 0);
        }

        if (num.hi < 0) != (den < 0) {
            assert!(quot.hi <= 0);
        } else {
            assert!(quot.hi >= 0);
        }
    }

    #[test]
    fn test_udiv_rem() {
        for d in 1u8..=255 {
            for n1 in (0u8..=255).step_by(15) {
                for n0 in (0u8..=255).step_by(15) {
                    let d = u128::from(d) << 120 | 1;
                    let n1 = u128::from(n1) << 120 | 1;
                    let n0 = u128::from(n0) << 120 | 1;
                    check_udiv_rem(U256 { lo: n0, hi: n1 }, d);
                }
            }
        }
    }

    #[test]
    fn test_idiv_rem_signs() {
        for d in -128..=127 {
            if d == 0 {
                continue;
            }
            for n1 in (-120..=120).step_by(15) {
                for n0 in (0u8..=255).step_by(15) {
                    let d = i128::from(d) << 120 | 1;
                    let n1 = i128::from(n1) << 120 | 1;
                    let n0 = u128::from(n0) << 120 | 1;
                    check_idiv_rem_signs(I256 { lo: n0, hi: n1 }, d);
                }
            }
        }
    }
}
