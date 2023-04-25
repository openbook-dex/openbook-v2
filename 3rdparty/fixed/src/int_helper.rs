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

use crate::{
    types::extra::U0, FixedI128, FixedI16, FixedI32, FixedI64, FixedI8, FixedU128, FixedU16,
    FixedU32, FixedU64, FixedU8,
};

macro_rules! make_helper_common {
    ($t:ident) => {
        use crate::helpers::{ToFixedHelper, Widest};
        use core::cmp::Ordering;
    };
}
macro_rules! make_helper {
    ($i: ident, $u:ident) => {
        pub mod $i {
            make_helper_common! { $i }

            #[inline]
            pub fn neg_abs(val: $i) -> (bool, $u) {
                if val < 0 {
                    (true, val.wrapping_neg() as $u)
                } else {
                    (false, val as $u)
                }
            }

            #[inline]
            pub fn is_negative(val: $i) -> bool {
                val.is_negative()
            }

            #[inline]
            pub fn to_fixed_helper(
                val: $i,
                src_frac_bits: i32,
                dst_frac_bits: u32,
                dst_int_bits: u32,
            ) -> ToFixedHelper {
                let src_bits = $i::BITS as i32;
                let dst_bits = (dst_frac_bits + dst_int_bits) as i32;

                if val == 0 {
                    return ToFixedHelper {
                        bits: Widest::Unsigned(0),
                        dir: Ordering::Equal,
                        overflow: false,
                    };
                }

                let need_to_shr = src_frac_bits - dst_frac_bits as i32;
                let leading = if val >= 0 {
                    val.leading_zeros()
                } else {
                    (!val).leading_zeros() - 1
                };
                let overflow = src_bits - dst_bits > need_to_shr + leading as i32;
                let bits_128 = i128::from(val);
                let (bits, lost_bits) = match need_to_shr {
                    -0x7fff_ffff..=-128 => (0, false),
                    -127..=-1 => (bits_128 << -need_to_shr, false),
                    0 => (bits_128, false),
                    1..=127 => {
                        let shifted = bits_128 >> need_to_shr;
                        (shifted, shifted << need_to_shr != bits_128)
                    }
                    128..=0x7fff_ffff => (bits_128 >> 127, true),
                    _ => unreachable!(),
                };
                let dir = if lost_bits {
                    Ordering::Less
                } else {
                    Ordering::Equal
                };
                let bits = if val >= 0 {
                    Widest::Unsigned(bits as u128)
                } else {
                    Widest::Negative(bits)
                };
                ToFixedHelper {
                    bits,
                    dir,
                    overflow,
                }
            }
        }

        pub mod $u {
            make_helper_common! { $u }

            #[inline]
            pub fn neg_abs(val: $u) -> (bool, $u) {
                (false, val)
            }

            #[inline]
            pub fn is_negative(val: $u) -> bool {
                let _ = val;
                false
            }

            #[inline]
            pub fn to_fixed_helper(
                val: $u,
                src_frac_bits: i32,
                dst_frac_bits: u32,
                dst_int_bits: u32,
            ) -> ToFixedHelper {
                let src_bits = $u::BITS as i32;
                let dst_bits = (dst_frac_bits + dst_int_bits) as i32;

                if val == 0 {
                    return ToFixedHelper {
                        bits: Widest::Unsigned(0),
                        dir: Ordering::Equal,
                        overflow: false,
                    };
                }

                let leading_zeros = val.leading_zeros();
                let need_to_shr = src_frac_bits - dst_frac_bits as i32;
                let overflow = src_bits - dst_bits > need_to_shr + leading_zeros as i32;
                let bits_128 = u128::from(val);
                let (bits, lost_bits) = match need_to_shr {
                    -0x7fff_ffff..=-128 => (0, false),
                    -127..=-1 => (bits_128 << -need_to_shr, false),
                    0 => (bits_128, false),
                    1..=127 => {
                        let shifted = bits_128 >> need_to_shr;
                        (shifted, shifted << need_to_shr != bits_128)
                    }
                    128..=0x7fff_ffff => (0, true),
                    _ => unreachable!(),
                };
                let dir = if lost_bits {
                    Ordering::Less
                } else {
                    Ordering::Equal
                };
                ToFixedHelper {
                    bits: Widest::Unsigned(bits),
                    dir,
                    overflow,
                }
            }
        }
    };
}

make_helper! { i8, u8 }
make_helper! { i16, u16 }
make_helper! { i32, u32 }
make_helper! { i64, u64 }
make_helper! { i128, u128 }

pub struct IntFixed<T>(pub T);

macro_rules! make_int_fixed {
    ($I:ident -> $F:ident) => {
        impl IntFixed<$I> {
            #[inline]
            pub fn fixed(self) -> $F<U0> {
                $F::<U0>::from_bits(self.0)
            }

            #[inline]
            pub fn int(f: $F<U0>) -> $I {
                f.to_bits()
            }
        }
    };
    ($T:ident as $I:ident -> $F:ident) => {
        impl IntFixed<$T> {
            #[inline]
            pub fn fixed(self) -> $F<U0> {
                $F::<U0>::from_bits(self.0 as $I)
            }

            #[inline]
            pub fn int(f: $F<U0>) -> $T {
                f.to_bits() as $T
            }
        }
    };
}

make_int_fixed! { i8 -> FixedI8 }
make_int_fixed! { i16 -> FixedI16 }
make_int_fixed! { i32 -> FixedI32 }
make_int_fixed! { i64 -> FixedI64 }
make_int_fixed! { i128 -> FixedI128 }
#[cfg(target_pointer_width = "16")]
make_int_fixed! { isize as i16 -> FixedI16 }
#[cfg(target_pointer_width = "32")]
make_int_fixed! { isize as i32 -> FixedI32 }
#[cfg(target_pointer_width = "64")]
make_int_fixed! { isize as i64 -> FixedI64 }
make_int_fixed! { u8 -> FixedU8 }
make_int_fixed! { u16 -> FixedU16 }
make_int_fixed! { u32 -> FixedU32 }
make_int_fixed! { u64 -> FixedU64 }
make_int_fixed! { u128 -> FixedU128 }
#[cfg(target_pointer_width = "16")]
make_int_fixed! { usize as u16 -> FixedU16 }
#[cfg(target_pointer_width = "32")]
make_int_fixed! { usize as u32 -> FixedU32 }
#[cfg(target_pointer_width = "64")]
make_int_fixed! { usize as u64 -> FixedU64 }
