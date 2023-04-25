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

use crate::int256;
use az_crate::{OverflowingAs, WrappingAs};

macro_rules! make_lerp {
    ($i:ident, $u:ident, $ii:ident, $uu:ident, $uu1:expr) => {
        pub fn $i(r: $i, start: $i, end: $i, frac_bits: u32) -> ($i, bool) {
            // 0x00 ≤ r_abs ≤ 0x80
            let (r_abs, r_neg) = if r >= 0 {
                (r.wrapping_as::<$u>(), false)
            } else {
                (r.wrapping_neg().wrapping_as::<$u>(), true)
            };
            // 0x00 ≤ range_abs ≤ 0xff
            let (range_abs, range_neg) = if end >= start {
                (end.wrapping_sub(start).wrapping_as::<$u>(), false)
            } else {
                (start.wrapping_sub(end).wrapping_as::<$u>(), true)
            };
            // 0x0000 ≤ wide_abs ≤ 0x7f80
            let wide_abs = $uu::from(r_abs) * $uu::from(range_abs);
            let wide_neg = r_neg != range_neg;
            let wide_uns = if wide_neg {
                // 0x0000 ≤ add ≤ 0x00ff
                let add = ($uu1 << frac_bits) - 1;
                // If frac_bits is 0: add = 0x0000; 0x0000 ≤ shifted ≤ 0x7f80
                // If frac_bits is max: add = 0x00ff; 0x0000 ≤ shifted ≤ 0x0080
                let shifted = (wide_abs + add) >> frac_bits;
                shifted.wrapping_neg()
            } else {
                wide_abs >> frac_bits
            };
            let wide = wide_uns.wrapping_as::<$ii>();
            let (wide_ret, overflow1) = wide.overflowing_add($ii::from(start));
            let (ret, overflow2) = wide_ret.overflowing_as::<$i>();
            (ret, overflow1 | overflow2)
        }

        pub fn $u(r: $u, start: $u, end: $u, frac_bits: u32) -> ($u, bool) {
            // 0x00 ≤ range_abs ≤ 0xff
            let (range_abs, range_neg) = if end >= start {
                (end - start, false)
            } else {
                (start - end, true)
            };
            // 0x0000 ≤ wide_abs ≤  0xfe01
            let wide_abs = $uu::from(r) * $uu::from(range_abs);
            let (wide_ret, overflow1) = if range_neg {
                // 0x0000 ≤ add ≤ 0x00ff
                let add = ($uu1 << frac_bits) - 1;
                // If frac_bits is 0: add = 0x0000; 0x0000 ≤ shifted ≤ 0xfe01
                // If frac_bits is max: add = 0x00ff; 0x0000 ≤ shifted ≤ 0x00ff
                let shifted = (wide_abs + add) >> frac_bits;
                $uu::from(start).overflowing_sub(shifted)
            } else {
                let shifted = wide_abs >> frac_bits;
                $uu::from(start).overflowing_add(shifted)
            };
            let (ret, overflow2) = wide_ret.overflowing_as::<$u>();
            (ret, overflow1 | overflow2)
        }
    };
}

make_lerp! { i8, u8, i16, u16, 1u16 }
make_lerp! { i16, u16, i32, u32, 1u32 }
make_lerp! { i32, u32, i64, u64, 1u64 }
make_lerp! { i64, u64, i128, u128, 1u128 }

pub fn i128(r: i128, start: i128, end: i128, frac_bits: u32) -> (i128, bool) {
    // 0x00 ≤ r_abs ≤ 0x80
    let (r_abs, r_neg) = if r >= 0 {
        (r.wrapping_as::<u128>(), false)
    } else {
        (r.wrapping_neg().wrapping_as::<u128>(), true)
    };
    // 0x00 ≤ range_abs ≤ 0xff
    let (range_abs, range_neg) = if end >= start {
        (end.wrapping_sub(start).wrapping_as::<u128>(), false)
    } else {
        (start.wrapping_sub(end).wrapping_as::<u128>(), true)
    };
    // 0x0000 ≤ wide_abs ≤ 0x7f80
    let wide_abs = int256::wide_mul_u128(r_abs, range_abs);
    let wide_neg = r_neg != range_neg;
    let wide_uns = if wide_neg {
        // 0x0000 ≤ add ≤ 0x00ff
        let add = if frac_bits == 128 {
            u128::MAX
        } else {
            (1u128 << frac_bits) - 1
        };
        // If frac_bits is 0: add = 0x0000; 0x0000 ≤ shifted ≤ 0x7f80
        // If frac_bits is max: add = 0x00ff; 0x0000 ≤ shifted ≤ 0x0080
        let sum = int256::wrapping_add_u256_u128(wide_abs, add);
        let shifted = int256::shl_u256_max_128(sum, frac_bits);
        int256::wrapping_neg_u256(shifted)
    } else {
        int256::shl_u256_max_128(wide_abs, frac_bits)
    };
    let wide = int256::u256_wrapping_as_i256(wide_uns);
    let (wide_ret, overflow1) = int256::overflowing_add_i256_i128(wide, start);
    let ret = wide_ret.lo.wrapping_as::<i128>();
    let overflow2 = if ret < 0 {
        wide_ret.hi != -1
    } else {
        wide_ret.hi != 0
    };
    (ret, overflow1 | overflow2)
}

pub fn u128(r: u128, start: u128, end: u128, frac_bits: u32) -> (u128, bool) {
    // 0x00 ≤ range_abs ≤ 0xff
    let (range_abs, range_neg) = if end >= start {
        (end - start, false)
    } else {
        (start - end, true)
    };
    // 0x0000 ≤ wide_abs ≤  0xfe01
    let wide_abs = int256::wide_mul_u128(r, range_abs);
    if range_neg {
        // 0x0000 ≤ add ≤ 0x00ff
        let add = if frac_bits == 128 {
            u128::MAX
        } else {
            (1u128 << frac_bits) - 1
        };
        // If frac_bits is 0: add = 0x0000; 0x0000 ≤ shifted ≤ 0xfe01
        // If frac_bits is max: add = 0x00ff; 0x0000 ≤ shifted ≤ 0x00ff
        let sum = int256::wrapping_add_u256_u128(wide_abs, add);
        let shifted = int256::shl_u256_max_128(sum, frac_bits);
        int256::overflowing_sub_u128_u256(start, shifted)
    } else {
        let shifted = int256::shl_u256_max_128(wide_abs, frac_bits);
        int256::overflowing_add_u128_u256(start, shifted)
    }
}

#[cfg(test)]
mod tests {
    use crate::lerp;

    #[test]
    fn lerp_i8() {
        assert_eq!(lerp::i8(0, -128, 127, 0), (-128, false));
        assert_eq!(lerp::i8(0, 127, -128, 8), (127, false));
        assert_eq!(lerp::i8(1, -128, 127, 1), (-1, false));
        assert_eq!(lerp::i8(-64, -100, -110, 6), (-90, false));

        assert_eq!(lerp::i8(2 << 2, 0, 50, 2), (100, false));
        assert_eq!(lerp::i8(2 << 2, 0, 100, 2), (-56, true));
        assert_eq!(lerp::i8((-1) << 2, 50, 0, 2), (100, false));
        assert_eq!(lerp::i8((-1) << 2, 100, 0, 2), (-56, true));
    }

    #[test]
    fn lerp_u8() {
        assert_eq!(lerp::u8(0, 0, 255, 0), (0, false));
        assert_eq!(lerp::u8(0, 255, 0, 8), (255, false));
        assert_eq!(lerp::u8(1, 0, 255, 0), (255, false));
        assert_eq!(lerp::u8(128, 255, 0, 7), (0, false));

        assert_eq!(lerp::u8(2 << 2, 0, 100, 2), (200, false));
        assert_eq!(lerp::u8(2 << 2, 0, 200, 2), (200 - 56, true));
    }

    #[test]
    fn lerp_i128() {
        assert_eq!(lerp::i128(0, -128, 127, 0), (-128, false));
        assert_eq!(lerp::i128(0, 127, -128, 128), (127, false));
        assert_eq!(lerp::i128(1, -128, 127, 1), (-1, false));
        assert_eq!(lerp::i128(1 << 98, 127, -128, 100), (63, false));
        assert_eq!(lerp::i128((-64) << 120, -100, -110, 126), (-90, false));

        assert_eq!(
            lerp::i128(2 << 2, 0, i128::MAX / 2, 2),
            (i128::MAX / 2 * 2, false)
        );
        assert_eq!(
            lerp::i128(2 << 2, 0, i128::MAX, 2),
            (i128::MAX.wrapping_mul(2), true)
        );
        assert_eq!(
            lerp::i128((-1) << 2, i128::MAX / 2, 0, 2),
            (i128::MAX / 2 * 2, false)
        );
        assert_eq!(
            lerp::i128((-1) << 2, i128::MAX, 0, 2),
            (i128::MAX.wrapping_mul(2), true)
        );
    }

    #[test]
    fn lerp_u128() {
        assert_eq!(lerp::u128(0, 0, 255, 0), (0, false));
        assert_eq!(lerp::u128(0, 255, 0, 128), (255, false));
        assert_eq!(lerp::u128(1, 0, 255, 0), (255, false));
        assert_eq!(lerp::u128(128 << 120, 255, 0, 127), (0, false));

        assert_eq!(
            lerp::u128(2 << 2, 0, u128::MAX / 2, 2),
            (u128::MAX / 2 * 2, false)
        );
        assert_eq!(
            lerp::u128(2 << 2, 0, u128::MAX, 2),
            (u128::MAX.wrapping_mul(2), true)
        );
    }
}
