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

macro_rules! make_helper {
    ($Float:ident($Bits:ty, $IBits:ident, $prec:expr) $(; use $path:path)?) => {
        #[allow(non_snake_case)]
        pub mod $Float {
            use crate::{
                helpers::{FloatKind, ToFixedHelper, ToFloatHelper, Widest},
                int_helper,
            };
            use core::cmp::Ordering;
            $(use $path;)?

            const PREC: u32 = $prec;
            const EXP_BIAS: i32 = (1 << (<$Bits>::BITS - PREC - 1)) - 1;
            const EXP_MIN: i32 = 1 - EXP_BIAS;
            const EXP_MAX: i32 = EXP_BIAS;
            pub const SIGN_MASK: $Bits = 1 << (<$Bits>::BITS - 1);
            pub const EXP_MASK: $Bits = !(SIGN_MASK | MANT_MASK);
            const MANT_MASK: $Bits = (1 << (PREC - 1)) - 1;

            #[inline]
            fn parts(val: $Float) -> (bool, i32, $Bits) {
                let bits = val.to_bits();
                let neg = bits & SIGN_MASK != 0;
                let biased_exp = (bits & EXP_MASK) >> (PREC - 1);
                let exp = biased_exp as i32 - EXP_BIAS;
                let mant = bits & MANT_MASK;

                (neg, exp, mant)
            }

            #[inline]
            pub fn from_to_float_helper(
                val: ToFloatHelper,
                frac_bits: u32,
                int_bits: u32,
            ) -> $Float {
                let fix_bits = frac_bits + int_bits;

                let bits_sign = if val.neg { SIGN_MASK } else { 0 };

                let extra_zeros = 128 - fix_bits;
                let leading_zeros = val.abs.leading_zeros() - extra_zeros;
                let signif_bits = fix_bits - leading_zeros;
                if signif_bits == 0 {
                    return $Float::from_bits(bits_sign);
                }
                // remove leading zeros and implicit one
                let mut mantissa = val.abs << leading_zeros << 1;
                let exponent = int_bits as i32 - 1 - leading_zeros as i32;
                let biased_exponent = if exponent > EXP_MAX {
                    return $Float::from_bits(EXP_MASK | bits_sign);
                } else if exponent < EXP_MIN {
                    let lost_prec = EXP_MIN - exponent;
                    if lost_prec as u32 >= (int_bits + frac_bits) {
                        mantissa = 0;
                    } else {
                        // reinsert implicit one
                        mantissa = (mantissa >> 1) | !(!0 >> 1);
                        mantissa >>= lost_prec - 1;
                    }
                    0
                } else {
                    (exponent + EXP_MAX) as $Bits
                };
                // check for rounding
                let round_up = (fix_bits >= PREC) && {
                    let shift = PREC - 1;
                    let mid_bit = !(!0 >> 1) >> (shift + extra_zeros);
                    let lower_bits = mid_bit - 1;
                    if mantissa & mid_bit == 0 {
                        false
                    } else if mantissa & lower_bits != 0 {
                        true
                    } else {
                        // round to even
                        mantissa & (mid_bit << 1) != 0
                    }
                };
                let bits_exp = biased_exponent << (PREC - 1);
                let bits_mantissa = (if fix_bits >= PREC - 1 {
                    (mantissa >> (fix_bits - (PREC - 1))) as $Bits
                } else {
                    (mantissa as $Bits) << (PREC - 1 - fix_bits)
                }) & !(!0 << (PREC - 1));
                let mut bits_exp_mantissa = bits_exp | bits_mantissa;
                if round_up {
                    bits_exp_mantissa += 1;
                }
                $Float::from_bits(bits_sign | bits_exp_mantissa)
            }

            #[inline]
            pub fn to_float_kind(val: $Float, dst_frac_bits: u32, dst_int_bits: u32) -> FloatKind {
                let prec = PREC as i32;

                let (neg, exp, mut mantissa) = parts(val);
                if exp > EXP_MAX {
                    if mantissa == 0 {
                        return FloatKind::Infinite { neg };
                    } else {
                        return FloatKind::NaN;
                    };
                }
                // if not subnormal, add implicit bit
                if exp >= EXP_MIN {
                    mantissa |= 1 << (prec - 1);
                }
                if mantissa == 0 {
                    let conv = ToFixedHelper {
                        bits: Widest::Unsigned(0),
                        dir: Ordering::Equal,
                        overflow: false,
                    };
                    return FloatKind::Finite { neg, conv };
                }

                let mut src_frac_bits = prec - 1 - exp;
                let need_to_shr = src_frac_bits - dst_frac_bits as i32;
                if need_to_shr > prec {
                    let dir = if neg {
                        Ordering::Greater
                    } else {
                        Ordering::Less
                    };
                    let conv = ToFixedHelper {
                        bits: Widest::Unsigned(0),
                        dir,
                        overflow: false,
                    };
                    return FloatKind::Finite { neg, conv };
                }
                let mut dir = Ordering::Equal;
                if need_to_shr > 0 {
                    let removed_bits = mantissa & !(!0 << need_to_shr);
                    let will_be_lsb = 1 << need_to_shr;
                    let tie = will_be_lsb >> 1;
                    if removed_bits == 0 {
                        // removed nothing
                    } else if removed_bits < tie {
                        dir = Ordering::Less;
                    } else if removed_bits > tie || mantissa & will_be_lsb != 0 {
                        mantissa += will_be_lsb;
                        dir = Ordering::Greater;
                    } else {
                        dir = Ordering::Less;
                    };
                    mantissa >>= need_to_shr;
                    src_frac_bits -= need_to_shr;
                }
                let mut mantissa = mantissa as $IBits;
                if neg {
                    mantissa = -mantissa;
                    dir = dir.reverse();
                }
                let mut conv = int_helper::$IBits::to_fixed_helper(
                    mantissa,
                    src_frac_bits,
                    dst_frac_bits,
                    dst_int_bits,
                );
                conv.dir = dir;
                FloatKind::Finite { neg, conv }
            }
        }
    };
}

make_helper! { f16(u16, i16, 11); use half::f16 }
make_helper! { bf16(u16, i16, 8); use half::bf16 }
make_helper! { f32(u32, i32, 24) }
make_helper! { f64(u64, i64, 53) }
make_helper! { F128Bits(u128, i128, 113); use crate::F128Bits }
