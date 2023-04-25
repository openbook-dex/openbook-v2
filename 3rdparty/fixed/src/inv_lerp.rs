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

use crate::int256::{self, U256};
use az_crate::{OverflowingAs, WrappingAs};
use core::cmp::Ordering;

macro_rules! make_inv_lerp {
    ($i:ident, $u:ident, $ii:ident, $uu:ident, $uuf:expr) => {
        pub fn $i(v: $i, start: $i, end: $i, frac_bits: u32) -> ($i, bool) {
            assert_ne!(start, end, "empty range");
            // 0x00 ≤ diff_abs ≤ 0xff
            let (diff_abs, diff_neg) = if v >= start {
                (v.wrapping_sub(start).wrapping_as::<$u>(), false)
            } else {
                (start.wrapping_sub(v).wrapping_as::<$u>(), true)
            };
            let wide_diff_abs = $uu::from(diff_abs) << $u::BITS;
            // 0x01 ≤ range_abs ≤ 0xff
            let (range_abs, range_neg) = if end >= start {
                (end.wrapping_sub(start).wrapping_as::<$u>(), false)
            } else {
                (start.wrapping_sub(end).wrapping_as::<$u>(), true)
            };
            let neg = diff_neg != range_neg;
            let (wide_ret, overflow1) = if neg {
                // ensure that difference is not 0
                if diff_abs == 0 {
                    return (0, false);
                }
                // 0x0001 ≤ ceil_div ≤ 0xff00
                let ceil_div = (wide_diff_abs - 1) / $uu::from(range_abs) + 1;
                // 0x0000 ≤ add ≤ 0x00ff
                let add = $uuf >> frac_bits;
                // If frac_bits is 0: add = 0x00ff; 0x0000 ≤ shifted ≤ 0x00ff
                // If frac_bits is max: add = 0x0000; 0x0000 ≤ shifted ≤ 0xff00
                let shifted = (ceil_div + add) >> ($u::BITS - frac_bits);
                let wide_ret = shifted.wrapping_neg().wrapping_as::<i16>();
                (wide_ret, wide_ret > 0)
            } else {
                // 0x0000 ≤ floor_div ≤ 0xff00
                let floor_div = wide_diff_abs / $uu::from(range_abs);
                let shifted = floor_div >> ($u::BITS - frac_bits);
                let wide_ret = shifted.wrapping_as::<i16>();
                (wide_ret, wide_ret < 0)
            };
            let (ret, overflow2) = wide_ret.overflowing_as::<$i>();
            (ret, overflow1 | overflow2)
        }

        pub fn $u(v: $u, start: $u, end: $u, frac_bits: u32) -> ($u, bool) {
            assert_ne!(start, end, "empty range");
            // 0x00 ≤ diff ≤ 0xff
            let (diff_abs, diff_neg) = if v >= start {
                (v - start, false)
            } else {
                (start - v, true)
            };
            let wide_diff_abs = $uu::from(diff_abs) << $u::BITS;
            // 0x01 ≤ range_abs ≤ 0xff
            let (range_abs, range_neg) = if end >= start {
                (end - start, false)
            } else {
                (start - end, true)
            };
            let neg = diff_neg != range_neg;
            let (wide_ret, overflow1) = if neg {
                // ensure that difference is not 0
                if diff_abs == 0 {
                    return (0, false);
                }
                // 0x0001 ≤ ceil_div ≤ 0xff00
                let ceil_div = (wide_diff_abs - 1) / $uu::from(range_abs) + 1;
                // 0x0000 ≤ add ≤ 0x00ff
                let add = $uuf >> frac_bits;
                // If frac_bits is 0: add = 0x00ff; 0x0000 ≤ shifted ≤ 0x00ff
                // If frac_bits is max: add = 0x0000; 0x0000 ≤ shifted ≤ 0xff00
                let shifted = (ceil_div + add) >> ($u::BITS - frac_bits);
                let wide_ret = shifted.wrapping_neg();
                (wide_ret, wide_ret > 0)
            } else {
                // 0x0000 ≤ floor_div ≤ 0xff00
                let floor_div = wide_diff_abs / $uu::from(range_abs);
                let shifted = floor_div >> ($u::BITS - frac_bits);
                (shifted, false)
            };
            let (ret, overflow2) = wide_ret.overflowing_as::<$u>();
            (ret, overflow1 | overflow2)
        }
    };
}

make_inv_lerp! { i8, u8, i16, u16, 0xff_u16 }
make_inv_lerp! { i16, u16, i32, u32, 0xffff_u32 }
make_inv_lerp! { i32, u32, i64, u64, 0xffff_ffff_u64 }
make_inv_lerp! { i64, u64, i128, u128, 0xffff_ffff_ffff_ffff_u128 }

pub fn i128(v: i128, start: i128, end: i128, frac_bits: u32) -> (i128, bool) {
    assert_ne!(start, end, "empty range");
    // 0x00 ≤ diff_abs ≤ 0xff
    let (diff_abs, diff_neg) = if v >= start {
        (v.wrapping_sub(start).wrapping_as::<u128>(), false)
    } else {
        (start.wrapping_sub(v).wrapping_as::<u128>(), true)
    };
    let wide_diff_abs = U256 {
        lo: 0,
        hi: diff_abs,
    };
    // 0x01 ≤ range_abs ≤ 0xff
    let (range_abs, range_neg) = if end >= start {
        (end.wrapping_sub(start).wrapping_as::<u128>(), false)
    } else {
        (start.wrapping_sub(end).wrapping_as::<u128>(), true)
    };
    let neg = diff_neg != range_neg;
    if neg {
        // ensure that difference is not 0
        if diff_abs == 0 {
            return (0, false);
        }
        // 0x0001 ≤ ceil_div ≤ 0xff00
        let (div, rem) = int256::div_rem_u256_u128(wide_diff_abs, range_abs);
        let ceil_div = if rem > 0 {
            int256::wrapping_add_u256_u128(div, 1)
        } else {
            div
        };
        // 0x0000 ≤ add ≤ 0x00ff
        let add = if frac_bits == 128 {
            0
        } else {
            u128::MAX >> frac_bits
        };
        // If frac_bits is 0: add = 0x00ff; 0x0000 ≤ shifted ≤ 0x00ff
        // If frac_bits is max: add = 0x0000; 0x0000 ≤ shifted ≤ 0xff00
        let sum = int256::wrapping_add_u256_u128(ceil_div, add);
        let shifted = int256::shl_u256_max_128(sum, 128 - frac_bits);
        let neg_shifted = int256::wrapping_neg_u256(shifted);
        let ret = neg_shifted.lo.wrapping_as::<i128>();
        let overflow = match ret.cmp(&0) {
            Ordering::Greater => true,
            Ordering::Equal => neg_shifted.hi != 0,
            Ordering::Less => neg_shifted.hi != u128::MAX,
        };
        (ret, overflow)
    } else {
        // 0x0000 ≤ floor_div ≤ 0xff00
        let (floor_div, _) = int256::div_rem_u256_u128(wide_diff_abs, range_abs);
        let shifted = int256::shl_u256_max_128(floor_div, 128 - frac_bits);
        let ret = shifted.lo.wrapping_as::<i128>();
        let overflow = (ret < 0) | (shifted.hi != 0);
        (ret, overflow)
    }
}

pub fn u128(v: u128, start: u128, end: u128, frac_bits: u32) -> (u128, bool) {
    assert_ne!(start, end, "empty range");
    // 0x00 ≤ diff ≤ 0xff
    let (diff_abs, diff_neg) = if v >= start {
        (v - start, false)
    } else {
        (start - v, true)
    };
    let wide_diff_abs = U256 {
        lo: 0,
        hi: diff_abs,
    };
    // 0x01 ≤ range_abs ≤ 0xff
    let (range_abs, range_neg) = if end >= start {
        (end - start, false)
    } else {
        (start - end, true)
    };
    let neg = diff_neg != range_neg;
    if neg {
        // ensure that difference is not 0
        if diff_abs == 0 {
            return (0, false);
        }
        // 0x0001 ≤ ceil_div ≤ 0xff00
        let (div, rem) = int256::div_rem_u256_u128(wide_diff_abs, range_abs);
        let ceil_div = if rem > 0 {
            int256::wrapping_add_u256_u128(div, 1)
        } else {
            div
        };
        // 0x0000 ≤ add ≤ 0x00ff
        let add = if frac_bits == 128 {
            0
        } else {
            u128::MAX >> frac_bits
        };
        // If frac_bits is 0: add = 0x00ff; 0x0000 ≤ shifted ≤ 0x00ff
        // If frac_bits is max: add = 0x0000; 0x0000 ≤ shifted ≤ 0xff00
        let sum = int256::wrapping_add_u256_u128(ceil_div, add);
        let shifted = int256::shl_u256_max_128(sum, 128 - frac_bits);
        let neg_shifted = int256::wrapping_neg_u256(shifted);
        let ret = neg_shifted.lo;
        let overflow = (ret != 0) | (neg_shifted.hi != 0);
        (ret, overflow)
    } else {
        // 0x0000 ≤ floor_div ≤ 0xff00
        let (floor_div, _) = int256::div_rem_u256_u128(wide_diff_abs, range_abs);
        let shifted = int256::shl_u256_max_128(floor_div, 128 - frac_bits);
        let ret = shifted.lo;
        let overflow = shifted.hi != 0;
        (ret, overflow)
    }
}

#[cfg(test)]
mod tests {
    use crate::inv_lerp;

    #[test]
    fn inv_lerp_i8() {
        assert_eq!(inv_lerp::i8(-128, -128, 127, 0), (0, false));
        assert_eq!(inv_lerp::i8(127, 127, -128, 8), (0, false));
        assert_eq!(inv_lerp::i8(-1, -128, 127, 6), (31, false));
        assert_eq!(inv_lerp::i8(-90, -100, -110, 6), (-64, false));

        assert_eq!(inv_lerp::i8(50, 0, 2, 2), (100, false));
        assert_eq!(inv_lerp::i8(100, 0, 2, 2), (-56, true));
        assert_eq!(inv_lerp::i8(-26, -1, 0, 2), (-100, false));
        assert_eq!(inv_lerp::i8(50, 0, -1, 2), (56, true));
    }

    #[test]
    fn inv_lerp_u8() {
        assert_eq!(inv_lerp::u8(0, 0, 255, 0), (0, false));
        assert_eq!(inv_lerp::u8(255, 255, 0, 8), (0, false));
        assert_eq!(inv_lerp::u8(255, 0, 255, 0), (1, false));
        assert_eq!(inv_lerp::u8(128, 255, 0, 7), (63, false));
        assert_eq!(inv_lerp::u8(51, 52, 53, 6), (192, true));

        assert_eq!(inv_lerp::u8(100, 0, 2, 2), (200, false));
        assert_eq!(inv_lerp::u8(200, 0, 2, 2), (200 - 56, true));
    }

    #[test]
    fn inv_lerp_i128() {
        assert_eq!(inv_lerp::i128(-128, -128, 127, 0), (0, false));
        assert_eq!(inv_lerp::i128(127, 127, -128, 128), (0, false));
        assert_eq!(inv_lerp::i128(-1, -128, 127, 6), (31, false));
        assert_eq!(inv_lerp::i128(-90, -100, -110, 126), ((-64) << 120, false));

        assert_eq!(
            inv_lerp::i128(i128::MAX / 2, 0, 2, 2),
            (i128::MAX / 2 * 2, false)
        );
        assert_eq!(
            inv_lerp::i128(i128::MAX, 0, 2, 2),
            (i128::MAX.wrapping_mul(2), true)
        );
        assert_eq!(
            inv_lerp::i128(i128::MIN / 4 - 1, -1, 0, 2),
            (i128::MIN, false)
        );
        assert_eq!(
            inv_lerp::i128(i128::MAX, 0, -1, 2),
            (i128::MAX.wrapping_mul(-4), true)
        );
    }

    #[test]
    fn inv_lerp_u128() {
        assert_eq!(inv_lerp::u128(0, 0, 255, 0), (0, false));
        assert_eq!(inv_lerp::u128(255, 255, 0, 128), (0, false));
        assert_eq!(inv_lerp::u128(255, 0, 255, 0), (1, false));
        assert_eq!(
            inv_lerp::u128(1 << 127, u128::MAX, 0, 127),
            ((1 << 126) - 1, false)
        );
        assert_eq!(inv_lerp::u128(51, 52, 53, 126), (192 << 120, true));

        assert_eq!(
            inv_lerp::u128(u128::MAX / 2, 0, 2, 2),
            (u128::MAX / 2 * 2, false)
        );
        assert_eq!(
            inv_lerp::u128(u128::MAX, 0, 2, 2),
            (u128::MAX.wrapping_mul(2), true)
        );
    }
}
