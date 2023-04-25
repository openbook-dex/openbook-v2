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
    float_helper,
    helpers::{FloatKind, Widest},
    int_helper::{self, IntFixed},
    types::extra::{LeEqU128, LeEqU16, LeEqU32, LeEqU64, LeEqU8},
    F128Bits, FixedI128, FixedI16, FixedI32, FixedI64, FixedI8, FixedU128, FixedU16, FixedU32,
    FixedU64, FixedU8,
};
use core::cmp::Ordering;
use half::{bf16, f16};

macro_rules! fixed_cmp_fixed {
    (
        $Lhs:ident($LhsLeEqU:ident, $LhsInner:ident),
        $Rhs:ident($RhsLeEqU:ident, $RhsInner:ident)
    ) => {
        impl<FracLhs: $LhsLeEqU, FracRhs: $RhsLeEqU> PartialEq<$Rhs<FracRhs>> for $Lhs<FracLhs> {
            #[inline]
            fn eq(&self, rhs: &$Rhs<FracRhs>) -> bool {
                let conv = int_helper::$RhsInner::to_fixed_helper(
                    rhs.to_bits(),
                    <$Rhs<FracRhs>>::FRAC_NBITS as i32,
                    Self::FRAC_NBITS,
                    Self::INT_NBITS,
                );
                let (rhs_is_neg, rhs_bits) = match conv.bits {
                    Widest::Unsigned(bits) => (false, bits as $LhsInner),
                    Widest::Negative(bits) => (true, bits as $LhsInner),
                };
                conv.dir == Ordering::Equal
                    && !conv.overflow
                    && rhs_is_neg == int_helper::$LhsInner::is_negative(rhs_bits)
                    && rhs_bits == self.to_bits()
            }
        }

        impl<FracLhs: $LhsLeEqU, FracRhs: $RhsLeEqU> PartialOrd<$Rhs<FracRhs>> for $Lhs<FracLhs> {
            #[inline]
            fn partial_cmp(&self, rhs: &$Rhs<FracRhs>) -> Option<Ordering> {
                let lhs_is_neg = int_helper::$LhsInner::is_negative(self.to_bits());
                let rhs_is_neg = int_helper::$RhsInner::is_negative(rhs.to_bits());
                match (lhs_is_neg, rhs_is_neg) {
                    (false, true) => return Some(Ordering::Greater),
                    (true, false) => return Some(Ordering::Less),
                    _ => {}
                }
                let conv = int_helper::$RhsInner::to_fixed_helper(
                    rhs.to_bits(),
                    <$Rhs<FracRhs>>::FRAC_NBITS as i32,
                    Self::FRAC_NBITS,
                    Self::INT_NBITS,
                );
                let rhs_bits = match conv.bits {
                    Widest::Unsigned(bits) => bits as $LhsInner,
                    Widest::Negative(bits) => bits as $LhsInner,
                };
                if conv.overflow || int_helper::$LhsInner::is_negative(rhs_bits) != rhs_is_neg {
                    return if rhs_is_neg {
                        Some(Ordering::Greater)
                    } else {
                        Some(Ordering::Less)
                    };
                }
                Some(self.to_bits().cmp(&rhs_bits).then(conv.dir))
            }

            #[inline]
            fn lt(&self, rhs: &$Rhs<FracRhs>) -> bool {
                let lhs_is_neg = int_helper::$LhsInner::is_negative(self.to_bits());
                let rhs_is_neg = int_helper::$RhsInner::is_negative(rhs.to_bits());
                match (lhs_is_neg, rhs_is_neg) {
                    (false, true) => return false,
                    (true, false) => return true,
                    _ => {}
                }
                let conv = int_helper::$RhsInner::to_fixed_helper(
                    rhs.to_bits(),
                    <$Rhs<FracRhs>>::FRAC_NBITS as i32,
                    Self::FRAC_NBITS,
                    Self::INT_NBITS,
                );
                let rhs_bits = match conv.bits {
                    Widest::Unsigned(bits) => bits as $LhsInner,
                    Widest::Negative(bits) => bits as $LhsInner,
                };
                if conv.overflow || int_helper::$LhsInner::is_negative(rhs_bits) != rhs_is_neg {
                    return !rhs_is_neg;
                }
                let lhs_bits = self.to_bits();
                lhs_bits < rhs_bits || (lhs_bits == rhs_bits && conv.dir == Ordering::Less)
            }

            #[inline]
            fn le(&self, rhs: &$Rhs<FracRhs>) -> bool {
                !rhs.lt(self)
            }

            #[inline]
            fn gt(&self, rhs: &$Rhs<FracRhs>) -> bool {
                rhs.lt(self)
            }

            #[inline]
            fn ge(&self, rhs: &$Rhs<FracRhs>) -> bool {
                !self.lt(rhs)
            }
        }
    };
}

macro_rules! fixed_cmp_int {
    ($Fix:ident($LeEqU:ident), $Int:ident) => {
        impl<Frac: $LeEqU> PartialEq<$Int> for $Fix<Frac> {
            #[inline]
            fn eq(&self, rhs: &$Int) -> bool {
                self.eq(&IntFixed(*rhs).fixed())
            }
        }

        impl<Frac: $LeEqU> PartialEq<$Fix<Frac>> for $Int {
            #[inline]
            fn eq(&self, rhs: &$Fix<Frac>) -> bool {
                IntFixed(*self).fixed().eq(rhs)
            }
        }

        impl<Frac: $LeEqU> PartialOrd<$Int> for $Fix<Frac> {
            #[inline]
            fn partial_cmp(&self, rhs: &$Int) -> Option<Ordering> {
                self.partial_cmp(&IntFixed(*rhs).fixed())
            }

            #[inline]
            fn lt(&self, rhs: &$Int) -> bool {
                self.lt(&IntFixed(*rhs).fixed())
            }

            #[inline]
            fn le(&self, rhs: &$Int) -> bool {
                !rhs.lt(self)
            }

            #[inline]
            fn gt(&self, rhs: &$Int) -> bool {
                rhs.lt(self)
            }

            #[inline]
            fn ge(&self, rhs: &$Int) -> bool {
                !self.lt(rhs)
            }
        }

        impl<Frac: $LeEqU> PartialOrd<$Fix<Frac>> for $Int {
            #[inline]
            fn partial_cmp(&self, rhs: &$Fix<Frac>) -> Option<Ordering> {
                IntFixed(*self).fixed().partial_cmp(rhs)
            }

            #[inline]
            fn lt(&self, rhs: &$Fix<Frac>) -> bool {
                IntFixed(*self).fixed().lt(rhs)
            }

            #[inline]
            fn le(&self, rhs: &$Fix<Frac>) -> bool {
                !rhs.lt(self)
            }

            #[inline]
            fn gt(&self, rhs: &$Fix<Frac>) -> bool {
                rhs.lt(self)
            }

            #[inline]
            fn ge(&self, rhs: &$Fix<Frac>) -> bool {
                !self.lt(rhs)
            }
        }
    };
}

macro_rules! fixed_cmp_float {
    ($Fix:ident($LeEqU:ident, $Inner:ident), $Float:ident) => {
        impl<Frac: $LeEqU> PartialEq<$Float> for $Fix<Frac> {
            #[inline]
            fn eq(&self, rhs: &$Float) -> bool {
                let kind =
                    float_helper::$Float::to_float_kind(*rhs, Self::FRAC_NBITS, Self::INT_NBITS);
                let conv = match kind {
                    FloatKind::Finite { conv, .. } => conv,
                    _ => return false,
                };
                let (rhs_is_neg, rhs_bits) = match conv.bits {
                    Widest::Unsigned(bits) => (false, bits as $Inner),
                    Widest::Negative(bits) => (true, bits as $Inner),
                };
                conv.dir == Ordering::Equal
                    && !conv.overflow
                    && rhs_is_neg == int_helper::$Inner::is_negative(rhs_bits)
                    && rhs_bits == self.to_bits()
            }
        }

        impl<Frac: $LeEqU> PartialEq<$Fix<Frac>> for $Float {
            #[inline]
            fn eq(&self, rhs: &$Fix<Frac>) -> bool {
                rhs.eq(self)
            }
        }

        impl<Frac: $LeEqU> PartialOrd<$Float> for $Fix<Frac> {
            #[inline]
            fn partial_cmp(&self, rhs: &$Float) -> Option<Ordering> {
                let lhs_is_neg = int_helper::$Inner::is_negative(self.to_bits());
                let kind =
                    float_helper::$Float::to_float_kind(*rhs, Self::FRAC_NBITS, Self::INT_NBITS);
                let (rhs_is_neg, conv) = match kind {
                    FloatKind::NaN => return None,
                    FloatKind::Infinite { neg } => {
                        return if neg {
                            Some(Ordering::Greater)
                        } else {
                            Some(Ordering::Less)
                        };
                    }
                    FloatKind::Finite { neg, conv } => (neg, conv),
                };
                match (lhs_is_neg, rhs_is_neg) {
                    (false, true) => return Some(Ordering::Greater),
                    (true, false) => return Some(Ordering::Less),
                    _ => {}
                }
                let rhs_bits = match conv.bits {
                    Widest::Unsigned(bits) => bits as $Inner,
                    Widest::Negative(bits) => bits as $Inner,
                };
                if conv.overflow || int_helper::$Inner::is_negative(rhs_bits) != rhs_is_neg {
                    return if rhs_is_neg {
                        Some(Ordering::Greater)
                    } else {
                        Some(Ordering::Less)
                    };
                }
                Some(self.to_bits().cmp(&rhs_bits).then(conv.dir))
            }

            #[inline]
            fn lt(&self, rhs: &$Float) -> bool {
                let lhs_is_neg = int_helper::$Inner::is_negative(self.to_bits());
                let kind =
                    float_helper::$Float::to_float_kind(*rhs, Self::FRAC_NBITS, Self::INT_NBITS);
                let (rhs_is_neg, conv) = match kind {
                    FloatKind::NaN => return false,
                    FloatKind::Infinite { neg } => return !neg,
                    FloatKind::Finite { neg, conv } => (neg, conv),
                };

                match (lhs_is_neg, rhs_is_neg) {
                    (false, true) => return false,
                    (true, false) => return true,
                    _ => {}
                }
                let rhs_bits = match conv.bits {
                    Widest::Unsigned(bits) => bits as $Inner,
                    Widest::Negative(bits) => bits as $Inner,
                };
                if conv.overflow || int_helper::$Inner::is_negative(rhs_bits) != rhs_is_neg {
                    return !rhs_is_neg;
                }
                let lhs_bits = self.to_bits();
                lhs_bits < rhs_bits || (lhs_bits == rhs_bits && conv.dir == Ordering::Less)
            }

            #[inline]
            fn le(&self, rhs: &$Float) -> bool {
                !rhs.is_nan() && !rhs.lt(self)
            }

            #[inline]
            fn gt(&self, rhs: &$Float) -> bool {
                rhs.lt(self)
            }

            #[inline]
            fn ge(&self, rhs: &$Float) -> bool {
                !rhs.is_nan() && !self.lt(rhs)
            }
        }

        impl<Frac: $LeEqU> PartialOrd<$Fix<Frac>> for $Float {
            #[inline]
            fn partial_cmp(&self, rhs: &$Fix<Frac>) -> Option<Ordering> {
                rhs.partial_cmp(self).map(Ordering::reverse)
            }

            #[inline]
            fn lt(&self, rhs: &$Fix<Frac>) -> bool {
                let kind = float_helper::$Float::to_float_kind(
                    *self,
                    <$Fix<Frac>>::FRAC_NBITS,
                    <$Fix<Frac>>::INT_NBITS,
                );
                let (lhs_is_neg, conv) = match kind {
                    FloatKind::NaN => return false,
                    FloatKind::Infinite { neg } => return neg,
                    FloatKind::Finite { neg, conv } => (neg, conv),
                };
                let rhs_is_neg = int_helper::$Inner::is_negative(rhs.to_bits());
                match (lhs_is_neg, rhs_is_neg) {
                    (false, true) => return false,
                    (true, false) => return true,
                    _ => {}
                }
                let lhs_bits = match conv.bits {
                    Widest::Unsigned(bits) => bits as $Inner,
                    Widest::Negative(bits) => bits as $Inner,
                };
                if conv.overflow || int_helper::$Inner::is_negative(lhs_bits) != lhs_is_neg {
                    return lhs_is_neg;
                }
                let rhs_bits = rhs.to_bits();
                lhs_bits < rhs_bits || (lhs_bits == rhs_bits && conv.dir == Ordering::Greater)
            }

            #[inline]
            fn le(&self, rhs: &$Fix<Frac>) -> bool {
                !self.is_nan() && !rhs.lt(self)
            }

            #[inline]
            fn gt(&self, rhs: &$Fix<Frac>) -> bool {
                rhs.lt(self)
            }

            #[inline]
            fn ge(&self, rhs: &$Fix<Frac>) -> bool {
                !self.is_nan() && !self.lt(rhs)
            }
        }
    };
}

macro_rules! fixed_cmp_all {
    ($Fix:ident($LeEqU:ident, $Inner:ident)) => {
        impl<Frac: $LeEqU> Eq for $Fix<Frac> {}

        impl<Frac: $LeEqU> Ord for $Fix<Frac> {
            #[inline]
            fn cmp(&self, rhs: &$Fix<Frac>) -> Ordering {
                self.to_bits().cmp(&rhs.to_bits())
            }
        }

        fixed_cmp_fixed! { $Fix($LeEqU, $Inner), FixedI8(LeEqU8, i8) }
        fixed_cmp_fixed! { $Fix($LeEqU, $Inner), FixedI16(LeEqU16, i16) }
        fixed_cmp_fixed! { $Fix($LeEqU, $Inner), FixedI32(LeEqU32, i32) }
        fixed_cmp_fixed! { $Fix($LeEqU, $Inner), FixedI64(LeEqU64, i64) }
        fixed_cmp_fixed! { $Fix($LeEqU, $Inner), FixedI128(LeEqU128, i128) }
        fixed_cmp_fixed! { $Fix($LeEqU, $Inner), FixedU8(LeEqU8, u8) }
        fixed_cmp_fixed! { $Fix($LeEqU, $Inner), FixedU16(LeEqU16, u16) }
        fixed_cmp_fixed! { $Fix($LeEqU, $Inner), FixedU32(LeEqU32, u32) }
        fixed_cmp_fixed! { $Fix($LeEqU, $Inner), FixedU64(LeEqU64, u64) }
        fixed_cmp_fixed! { $Fix($LeEqU, $Inner), FixedU128(LeEqU128, u128) }
        fixed_cmp_int! { $Fix($LeEqU), i8 }
        fixed_cmp_int! { $Fix($LeEqU), i16 }
        fixed_cmp_int! { $Fix($LeEqU), i32 }
        fixed_cmp_int! { $Fix($LeEqU), i64 }
        fixed_cmp_int! { $Fix($LeEqU), i128 }
        fixed_cmp_int! { $Fix($LeEqU), isize }
        fixed_cmp_int! { $Fix($LeEqU), u8 }
        fixed_cmp_int! { $Fix($LeEqU), u16 }
        fixed_cmp_int! { $Fix($LeEqU), u32 }
        fixed_cmp_int! { $Fix($LeEqU), u64 }
        fixed_cmp_int! { $Fix($LeEqU), u128 }
        fixed_cmp_int! { $Fix($LeEqU), usize }
        fixed_cmp_float! { $Fix($LeEqU, $Inner), f16 }
        fixed_cmp_float! { $Fix($LeEqU, $Inner), bf16 }
        fixed_cmp_float! { $Fix($LeEqU, $Inner), f32 }
        fixed_cmp_float! { $Fix($LeEqU, $Inner), f64 }
        fixed_cmp_float! { $Fix($LeEqU, $Inner), F128Bits }
    };
}

fixed_cmp_all! { FixedI8(LeEqU8, i8) }
fixed_cmp_all! { FixedI16(LeEqU16, i16) }
fixed_cmp_all! { FixedI32(LeEqU32, i32) }
fixed_cmp_all! { FixedI64(LeEqU64, i64) }
fixed_cmp_all! { FixedI128(LeEqU128, i128) }
fixed_cmp_all! { FixedU8(LeEqU8, u8) }
fixed_cmp_all! { FixedU16(LeEqU16, u16) }
fixed_cmp_all! { FixedU32(LeEqU32, u32) }
fixed_cmp_all! { FixedU64(LeEqU64, u64) }
fixed_cmp_all! { FixedU128(LeEqU128, u128) }

macro_rules! fixed_cmp {
    ($Fixed:ident($Inner:ty, $Len:ty, $bits_count:expr)) => {};
}

fixed_cmp! { FixedU8(u8, LeEqU8, 8) }
fixed_cmp! { FixedU16(u16, LeEqU16, 16) }
fixed_cmp! { FixedU32(u32, LeEqU32, 32) }
fixed_cmp! { FixedU64(u64, LeEqU64, 64) }
fixed_cmp! { FixedU128(u128, LeEqU128, 128) }
fixed_cmp! { FixedI8(i8, LeEqU8, 8) }
fixed_cmp! { FixedI16(i16, LeEqU16, 16) }
fixed_cmp! { FixedI32(i32, LeEqU32, 32) }
fixed_cmp! { FixedI64(i64, LeEqU64, 64) }
fixed_cmp! { FixedI128(i128, LeEqU128, 128) }

#[cfg(test)]
#[allow(clippy::float_cmp, clippy::eq_op)]
mod tests {
    use crate::*;
    use core::f32;

    #[test]
    fn cmp_signed() {
        use core::cmp::Ordering::*;
        let neg1_16 = FixedI32::<types::extra::U16>::from_num(-1);
        let neg1_20 = FixedI32::<types::extra::U20>::from_num(-1);
        let mut a = neg1_16;
        let mut b = neg1_20;
        // a = ffff.0000 = -1, b = fff.00000 = -1
        assert!(a.eq(&b) && b.eq(&a));
        assert_eq!(a.partial_cmp(&b), Some(Equal));
        assert_eq!(b.partial_cmp(&a), Some(Equal));
        assert_eq!(a, -1i8);
        assert_eq!(b, -1i128);
        a >>= 16;
        b >>= 16;
        // a = ffff.ffff = -2^-16, b = fff.ffff0 = -2^-16
        assert!(a.eq(&b) && b.eq(&a));
        assert_eq!(a.partial_cmp(&b), Some(Equal));
        assert_eq!(b.partial_cmp(&a), Some(Equal));
        assert!(a < 0.0);
        assert_eq!(a.partial_cmp(&f32::INFINITY), Some(Less));
        assert!(a < f32::INFINITY);
        assert!(a != f32::INFINITY);
        assert_eq!(a.partial_cmp(&f32::NEG_INFINITY), Some(Greater));
        assert!(a > f32::NEG_INFINITY);
        assert_eq!(a, -(-16f32).exp2());
        assert!(a <= -(-16f32).exp2());
        assert!(a >= -(-16f32).exp2());
        assert!(a < (-16f32).exp2());
        assert_ne!(a, -0.75 * (-16f32).exp2());
        assert!(a < -0.75 * (-16f32).exp2());
        assert!(a <= -0.75 * (-16f32).exp2());
        assert!(a > -1.25 * (-16f32).exp2());
        assert!(a >= -1.25 * (-16f32).exp2());
        a >>= 1;
        b >>= 1;
        // a = ffff.ffff = -2^-16, b = fff.ffff8 = -2^-17
        assert!(a.ne(&b) && b.ne(&a));
        assert_eq!(a.partial_cmp(&b), Some(Less));
        assert_eq!(b.partial_cmp(&a), Some(Greater));
        a = neg1_16 << 11;
        b = neg1_20 << 11;
        // a = f800.0000 = -2^11, b = 800.00000 = -2^11
        assert!(a.eq(&b) && b.eq(&a));
        assert_eq!(a.partial_cmp(&b), Some(Equal));
        assert_eq!(b.partial_cmp(&a), Some(Equal));
        assert_eq!(a, -1i16 << 11);
        assert_eq!(b, -1i64 << 11);
        a <<= 1;
        b <<= 1;
        // a = f000.0000 = -2^-12, b = 000.00000 = 0
        assert!(a.ne(&b) && b.ne(&a));
        assert_eq!(a.partial_cmp(&b), Some(Less));
        assert_eq!(b.partial_cmp(&a), Some(Greater));
        assert!(a < 1u8);
        assert_eq!(b, 0);
    }

    #[test]
    fn cmp_unsigned() {
        use core::cmp::Ordering::*;
        let one_16 = FixedU32::<types::extra::U16>::ONE;
        let one_20 = FixedU32::<types::extra::U20>::ONE;
        let mut a = one_16;
        let mut b = one_20;
        // a = 0001.0000 = 1, b = 001.00000 = 1
        assert!(a.eq(&b) && b.eq(&a));
        assert_eq!(a.partial_cmp(&b), Some(Equal));
        assert_eq!(b.partial_cmp(&a), Some(Equal));
        assert_eq!(a, 1u8);
        assert_eq!(b, 1i128);
        a >>= 16;
        b >>= 16;
        // a = 0000.0001 = 2^-16, b = 000.00010 = 2^-16
        assert!(a.eq(&b) && b.eq(&a));
        assert_eq!(a.partial_cmp(&b), Some(Equal));
        assert_eq!(b.partial_cmp(&a), Some(Equal));
        assert!(a > 0.0);
        assert_eq!(a.partial_cmp(&f32::INFINITY), Some(Less));
        assert!(a < f32::INFINITY);
        assert!(a != f32::INFINITY);
        assert_eq!(a.partial_cmp(&f32::NEG_INFINITY), Some(Greater));
        assert!(a > f32::NEG_INFINITY);
        assert_eq!(a, (-16f64).exp2());
        assert!(a <= (-16f64).exp2());
        assert!(a >= (-16f64).exp2());
        assert!(a > -(-16f64).exp2());
        assert_ne!(a, 0.75 * (-16f64).exp2());
        assert!(a > 0.75 * (-16f64).exp2());
        assert!(a >= 0.75 * (-16f64).exp2());
        assert!(a < 1.25 * (-16f64).exp2());
        assert!(a <= 1.25 * (-16f64).exp2());
        a >>= 1;
        b >>= 1;
        // a = 0000.0000 = 0, b = 000.00008 = 2^-17
        assert!(a.ne(&b) && b.ne(&a));
        assert_eq!(a.partial_cmp(&b), Some(Less));
        assert_eq!(b.partial_cmp(&a), Some(Greater));
        a = one_16 << 11;
        b = one_20 << 11;
        // a = 0800.0000 = 2^11, b = 800.00000 = 2^11
        assert!(a.eq(&b) && b.eq(&a));
        assert_eq!(a.partial_cmp(&b), Some(Equal));
        assert_eq!(b.partial_cmp(&a), Some(Equal));
        assert_eq!(a, 1i16 << 11);
        assert_eq!(b, 1u64 << 11);
        a <<= 1;
        b <<= 1;
        // a = 1000.0000 = 2^12, b = 000.00000 = 0
        assert!(a.ne(&b) && b.ne(&a));
        assert_eq!(a.partial_cmp(&b), Some(Greater));
        assert_eq!(b.partial_cmp(&a), Some(Less));
        assert!(a > -1i8);
        assert_eq!(a, 1i32 << 12);
        assert_eq!(b, 0);
    }

    #[test]
    fn cmp_i0() {
        use crate::types::*;
        assert_eq!(I0F32::checked_from_num(0.5), None);
        for &float in &[-0.5, -0.25, 0., 0.25, 0.49] {
            let fixed = I0F32::from_num(float);
            let half = U0F32::from_num(0.5);
            assert_eq!(fixed < half, float < 0.5, "{} < {}", fixed, half);
            assert_eq!(fixed == half, float == 0.5, "{} == {}", fixed, half);
            assert_eq!(fixed > half, float > 0.5, "{} > {}", fixed, half);
            assert_eq!(
                fixed.partial_cmp(&half),
                float.partial_cmp(&0.5),
                "{}.partial_cmp(&{})",
                fixed,
                half
            );
            assert_eq!(half < fixed, fixed > half);
            assert_eq!(half == fixed, fixed == half);
            assert_eq!(half > fixed, fixed < half);
            assert_eq!(
                half.partial_cmp(&fixed),
                fixed.partial_cmp(&half).map(Ordering::reverse)
            );

            let half = I1F31::from_num(0.5);
            assert_eq!(fixed < half, float < 0.5, "{} < {}", fixed, half);
            assert_eq!(fixed == half, float == 0.5, "{} == {}", fixed, half);
            assert_eq!(fixed > half, float > 0.5, "{} > {}", fixed, half);
            assert_eq!(
                fixed.partial_cmp(&half),
                float.partial_cmp(&0.5),
                "{}.partial_cmp(&{})",
                fixed,
                half
            );
            assert_eq!(half < fixed, fixed > half);
            assert_eq!(half == fixed, fixed == half);
            assert_eq!(half > fixed, fixed < half);
            assert_eq!(
                half.partial_cmp(&fixed),
                fixed.partial_cmp(&half).map(Ordering::reverse)
            );

            let half = 0.5f32;
            assert_eq!(fixed < half, float < 0.5, "{} < {}", fixed, half);
            assert_eq!(fixed == half, float == 0.5, "{} == {}", fixed, half);
            assert_eq!(fixed > half, float > 0.5, "{} > {}", fixed, half);
            assert_eq!(
                fixed.partial_cmp(&half),
                float.partial_cmp(&0.5),
                "{}.partial_cmp(&{})",
                fixed,
                half
            );
            assert_eq!(half < fixed, fixed > half);
            assert_eq!(half == fixed, fixed == half);
            assert_eq!(half > fixed, fixed < half);
            assert_eq!(
                half.partial_cmp(&fixed),
                fixed.partial_cmp(&half).map(Ordering::reverse)
            );

            let m1 = I1F31::from_num(-1.0);
            assert_eq!(fixed < m1, float < -1.0, "{} < {}", fixed, m1);
            assert_eq!(fixed == m1, float == -1.0, "{} == {}", fixed, m1);
            assert_eq!(fixed > m1, float > -1.0, "{} > {}", fixed, m1);
            assert_eq!(
                fixed.partial_cmp(&m1),
                float.partial_cmp(&-1.0),
                "{}.partial_cmp(&{})",
                fixed,
                m1
            );
            assert_eq!(m1 < fixed, fixed > m1);
            assert_eq!(m1 == fixed, fixed == m1);
            assert_eq!(m1 > fixed, fixed < m1);
            assert_eq!(
                m1.partial_cmp(&fixed),
                fixed.partial_cmp(&m1).map(Ordering::reverse)
            );

            let m1 = -1.0f32;
            assert_eq!(fixed < m1, float < -1.0, "{} < {}", fixed, m1);
            assert_eq!(fixed == m1, float == -1.0, "{} == {}", fixed, m1);
            assert_eq!(fixed > m1, float > -1.0, "{} > {}", fixed, m1);
            assert_eq!(
                fixed.partial_cmp(&m1),
                float.partial_cmp(&-1.0),
                "{}.partial_cmp(&{})",
                fixed,
                m1
            );
            assert_eq!(m1 < fixed, fixed > m1);
            assert_eq!(m1 == fixed, fixed == m1);
            assert_eq!(m1 > fixed, fixed < m1);
            assert_eq!(
                m1.partial_cmp(&fixed),
                fixed.partial_cmp(&m1).map(Ordering::reverse)
            );

            let mhalf = I1F31::from_num(-0.5);
            assert_eq!(fixed < mhalf, float < -0.5, "{} < {}", fixed, mhalf);
            assert_eq!(fixed == mhalf, float == -0.5, "{} == {}", fixed, mhalf);
            assert_eq!(fixed > mhalf, float > -0.5, "{} > {}", fixed, mhalf);
            assert_eq!(
                fixed.partial_cmp(&mhalf),
                float.partial_cmp(&-0.5),
                "{}.partial_cmp(&{})",
                fixed,
                mhalf
            );
            assert_eq!(mhalf < fixed, fixed > mhalf);
            assert_eq!(mhalf == fixed, fixed == mhalf);
            assert_eq!(mhalf > fixed, fixed < mhalf);
            assert_eq!(
                mhalf.partial_cmp(&fixed),
                fixed.partial_cmp(&mhalf).map(Ordering::reverse)
            );

            let mhalf = -0.5f32;
            assert_eq!(fixed < mhalf, float < -0.5, "{} < {}", fixed, mhalf);
            assert_eq!(fixed == mhalf, float == -0.5, "{} == {}", fixed, mhalf);
            assert_eq!(fixed > mhalf, float > -0.5, "{} > {}", fixed, mhalf);
            assert_eq!(
                fixed.partial_cmp(&mhalf),
                float.partial_cmp(&-0.5),
                "{}.partial_cmp(&{})",
                fixed,
                mhalf
            );
            assert_eq!(mhalf < fixed, fixed > mhalf);
            assert_eq!(mhalf == fixed, fixed == mhalf);
            assert_eq!(mhalf > fixed, fixed < mhalf);
            assert_eq!(
                mhalf.partial_cmp(&fixed),
                fixed.partial_cmp(&mhalf).map(Ordering::reverse)
            );
        }
    }
}
