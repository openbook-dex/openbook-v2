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

// self must be positive
pub trait IntFracLog10 {
    fn int_part_log10(self) -> i32;
    fn frac_part_log10(self) -> i32;
}

impl IntFracLog10 for u8 {
    // 1 <= val <= MAX (255)
    // 0 <= log <= 2
    #[inline]
    fn int_part_log10(self) -> i32 {
        if self >= 100 {
            2
        } else if self >= 10 {
            1
        } else {
            maybe_assert!(self >= 1);
            0
        }
    }

    // MAX / 1000 (0) < val <= MAX (255)
    // -3 <= log <= -1
    #[inline]
    fn frac_part_log10(self) -> i32 {
        if self > 25 {
            -1
        } else if self > 2 {
            -2
        } else {
            maybe_assert!(self > 0);
            -3
        }
    }
}

impl IntFracLog10 for u16 {
    // 1 <= val <= MAX (65_535)
    // 0 <= log <= 4
    #[inline]
    fn int_part_log10(self) -> i32 {
        if self >= 10_000 {
            4
        } else if self >= 1000 {
            3
        } else if self >= 100 {
            2
        } else if self >= 10 {
            1
        } else {
            maybe_assert!(self >= 1);
            0
        }
    }

    // MAX / 100_000 (0) < val <= MAX (65_535)
    // -5 <= log <= -1
    #[inline]
    fn frac_part_log10(self) -> i32 {
        if self > 6553 {
            -1
        } else if self > 655 {
            -2
        } else if self > 65 {
            -3
        } else if self > 6 {
            -4
        } else {
            maybe_assert!(self > 0);
            -5
        }
    }
}

// 0 < val < 100_000_000
// 0 <= log <= 7
fn int_part_log10_less_than_8(mut val: u32) -> i32 {
    maybe_assert!(val < 100_000_000);
    let mut log = 0;
    if val >= 10_000 {
        val /= 10_000;
        log += 4;
    }
    log + if val >= 1000 {
        3
    } else if val >= 100 {
        2
    } else if val >= 10 {
        1
    } else {
        maybe_assert!(val >= 1);
        0
    }
}

// 0 < val < 10_000_000_000_000_000
// 0 <= log <= 15
fn int_part_log10_less_than_16(mut val: u64) -> i32 {
    maybe_assert!(val < 10_000_000_000_000_000);
    let mut log = 0;
    if val >= 100_000_000 {
        val /= 100_000_000;
        log += 8;
    }
    maybe_assert!((val >> 32) == 0);
    log + int_part_log10_less_than_8(val as u32)
}

// MAX / 100_000_000 < val <= MAX
// -8 <= log <= -1
fn frac_part_log10_greater_equal_m8_u32(mut val: u32) -> i32 {
    const MAX: u32 = u32::MAX;
    maybe_assert!(val > MAX / 100_000_000);
    let mut log = 0;
    if val <= MAX / 10_000 {
        val *= 10_000;
        log += -4;
    }
    log + if val > MAX / 10 {
        -1
    } else if val > MAX / 100 {
        -2
    } else if val > MAX / 1000 {
        -3
    } else {
        maybe_assert!(val > MAX / 10_000);
        -4
    }
}

// MAX / 100_000_000 < val <= MAX
// -8 <= log <= 1
fn frac_part_log10_greater_equal_m8_u64(mut val: u64) -> i32 {
    const MAX: u64 = u64::MAX;
    maybe_assert!(val > MAX / 100_000_000);
    let mut log = 0;
    if val <= MAX / 10_000 {
        val *= 10_000;
        log += -4;
    }
    log + if val > MAX / 10 {
        -1
    } else if val > MAX / 100 {
        -2
    } else if val > MAX / 1000 {
        -3
    } else {
        maybe_assert!(val > MAX / 10_000);
        -4
    }
}

impl IntFracLog10 for u32 {
    // 1 <= val <= MAX
    // 0 <= log <= 9
    fn int_part_log10(mut self) -> i32 {
        if self >= 100_000_000 {
            self /= 100_000_000;
            maybe_assert!(self < 100);
            if self >= 10 {
                9
            } else {
                maybe_assert!(self >= 1);
                8
            }
        } else {
            int_part_log10_less_than_8(self)
        }
    }

    // 0 < val <= MAX
    // -10 <= log <= -1
    fn frac_part_log10(mut self) -> i32 {
        const MAX: u32 = u32::MAX;
        if self <= MAX / 100_000_000 {
            self *= 100_000_000;
            // At this point, we have shifted out 8 digits, and we can only shift out 2 more.
            // We can only check up to -2 more because -10 <= log <= -8.
            if self > MAX / 10 {
                -9
            } else {
                maybe_assert!(self > MAX / 100);
                -10
            }
        } else {
            frac_part_log10_greater_equal_m8_u32(self)
        }
    }
}

impl IntFracLog10 for u64 {
    // 1 <= val <= MAX
    // 0 <= log <= 19
    fn int_part_log10(mut self) -> i32 {
        if self >= 10_000_000_000_000_000 {
            self /= 10_000_000_000_000_000;
            maybe_assert!(self < 10_000);
            if self >= 1000 {
                19
            } else if self >= 100 {
                18
            } else if self >= 10 {
                17
            } else {
                maybe_assert!(self >= 1);
                16
            }
        } else {
            int_part_log10_less_than_16(self)
        }
    }

    // 0 < val <= MAX
    // -20 <= log <= -1
    fn frac_part_log10(mut self) -> i32 {
        const MAX: u64 = u64::MAX;
        let mut log = 0;
        if self <= MAX / 10_000_000_000_000_000 {
            // After this, we can only check up to -4 more because -20 <= log <= -16.
            // That is, we can skip the checks against MAX / 100_000_000 and MAX / 10_000.
            self *= 10_000_000_000_000_000;
            log += -16;
        } else {
            if self <= MAX / 100_000_000 {
                self *= 100_000_000;
                log += -8;
            }
            if self <= MAX / 10_000 {
                self *= 10_000;
                log += -4;
            }
        }
        log + if self > MAX / 10 {
            -1
        } else if self > MAX / 100 {
            -2
        } else if self > MAX / 1000 {
            -3
        } else {
            maybe_assert!(self > MAX / 10_000);
            -4
        }
    }
}

impl IntFracLog10 for u128 {
    // 1 <= val <= MAX
    // 0 <= log <= 38
    fn int_part_log10(mut self) -> i32 {
        let mut log = 0;
        if self >= 100_000_000_000_000_000_000_000_000_000_000 {
            self /= 100_000_000_000_000_000_000_000_000_000_000;
            maybe_assert!(self <= u32::MAX as u128);
            return 32 + int_part_log10_less_than_8(self as u32);
        }
        if self >= 10_000_000_000_000_000 {
            self /= 10_000_000_000_000_000;
            log += 16;
        }
        maybe_assert!(self <= u64::MAX as u128);
        log + int_part_log10_less_than_16(self as u64)
    }

    // 0 < val <= MAX
    // -39 <= log <= -1
    fn frac_part_log10(mut self) -> i32 {
        const MAX: u128 = u128::MAX;
        let mut log = 0;
        if self <= MAX / 100_000_000_000_000_000_000_000_000_000_000 {
            self *= 100_000_000_000_000_000_000_000_000_000_000;
            // At this point we have shifted out 32 digits, and we can only shift out 7 more.
            // We can
            //   * use self >> 96 because we have shifted out 32 decimal digits (106 bits)
            //   * only check up to -8 more because -39 <= log <= -32
            return -32 + frac_part_log10_greater_equal_m8_u32((self >> 96) as u32);
        }
        if self <= MAX / 10_000_000_000_000_000 {
            self *= 10_000_000_000_000_000;
            log += -16;
        }
        if self <= MAX / 100_000_000 {
            self *= 100_000_000;
            log += -8;
        }
        if log == -24 {
            // At this point we have shifted out 24 digits, and we can only shift out 15 more.
            // We can
            //   * use self >> 64 because we have shifted out 24 decimal digits (79 bits)
            //   * only check up to -8 more because -32 <= log <= -24
            return -24 + frac_part_log10_greater_equal_m8_u64((self >> 64) as u64);
        }
        // We have *not* shifted out enough decimal digits, so we must *not* convert to u32 or u64.
        if self <= MAX / 10_000 {
            self *= 10_000;
            log += -4;
        }
        log + if self > MAX / 10 {
            -1
        } else if self > MAX / 100 {
            -2
        } else if self > MAX / 1000 {
            -3
        } else {
            maybe_assert!(self > MAX / 10_000);
            -4
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IntFracLog10;

    macro_rules! check_loop {
        ($T:ty) => {
            for i in 0..=<$T>::MAX.int_part_log10() {
                let p = (10 as $T).pow(i as u32);
                if i > 0 {
                    assert_eq!((p - 1).int_part_log10(), i - 1);
                }
                assert_eq!(p.int_part_log10(), i);
                assert_eq!((p + 1).int_part_log10(), i);
            }

            for i in 0..-(1 as $T).frac_part_log10() {
                let p = <$T>::MAX / (10 as $T).pow(i as u32);
                if p > 1 {
                    assert_eq!((p - 1).frac_part_log10(), -1 - i);
                }
                assert_eq!(p.frac_part_log10(), -1 - i);
                if i > 0 {
                    assert_eq!((p + 1).frac_part_log10(), -i);
                }
            }
        };
    }

    #[test]
    fn log10_u8() {
        assert_eq!(1u8.int_part_log10(), 0);
        assert_eq!(u8::MAX.int_part_log10(), 2);
        assert_eq!(1u8.frac_part_log10(), -3);
        assert_eq!(u8::MAX.frac_part_log10(), -1);

        check_loop! { u8 }
    }

    #[test]
    fn log10_u16() {
        assert_eq!(1u16.int_part_log10(), 0);
        assert_eq!(u16::MAX.int_part_log10(), 4);
        assert_eq!(1u16.frac_part_log10(), -5);
        assert_eq!(u16::MAX.frac_part_log10(), -1);

        check_loop! { u16 }
    }

    #[test]
    fn log10_u32() {
        assert_eq!(1u32.int_part_log10(), 0);
        assert_eq!(u32::MAX.int_part_log10(), 9);
        assert_eq!(1u32.frac_part_log10(), -10);
        assert_eq!(u32::MAX.frac_part_log10(), -1);

        check_loop! { u32 }
    }

    #[test]
    fn log10_u64() {
        assert_eq!(1u64.int_part_log10(), 0);
        assert_eq!(u64::MAX.int_part_log10(), 19);
        assert_eq!(1u64.frac_part_log10(), -20);
        assert_eq!(u64::MAX.frac_part_log10(), -1);

        check_loop! { u64 }
    }

    #[test]
    fn log10_u128() {
        assert_eq!(1u128.int_part_log10(), 0);
        assert_eq!(u128::MAX.int_part_log10(), 38);
        assert_eq!(1u128.frac_part_log10(), -39);
        assert_eq!(u128::MAX.frac_part_log10(), -1);

        check_loop! { u128 }
    }
}
