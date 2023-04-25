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
    consts,
    types::extra::{
        IsLessOrEqual, LeEqU128, LeEqU16, LeEqU32, LeEqU64, LeEqU8, True, U126, U127, U14, U15,
        U30, U31, U6, U62, U63, U7,
    },
    FixedI128, FixedI16, FixedI32, FixedI64, FixedI8, FixedU128, FixedU16, FixedU32, FixedU64,
    FixedU8, ParseFixedError,
};
use core::fmt::{Display, Formatter, Result as FmtResult};
use num_traits::{
    bounds::Bounded,
    cast::{FromPrimitive, ToPrimitive},
    float::FloatConst,
    identities::{One, Zero},
    ops::{
        checked::{
            CheckedAdd, CheckedDiv, CheckedMul, CheckedNeg, CheckedRem, CheckedShl, CheckedShr,
            CheckedSub,
        },
        inv::Inv,
        mul_add::{MulAdd, MulAddAssign},
        overflowing::{OverflowingAdd, OverflowingMul, OverflowingSub},
        saturating::{SaturatingAdd, SaturatingMul, SaturatingSub},
        wrapping::{WrappingAdd, WrappingMul, WrappingNeg, WrappingShl, WrappingShr, WrappingSub},
    },
    sign::{Signed, Unsigned},
    Num,
};
#[cfg(feature = "std")]
use std::error::Error;

/// An error which can be returned when parsing a fixed-point number
/// with a given radix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadixParseFixedError {
    /// The radix is not 2, 8, 10 or 16.
    UnsupportedRadix,
    /// The string could not be parsed as a fixed-point number.
    ParseFixedError(ParseFixedError),
}

impl RadixParseFixedError {
    fn message(&self) -> &str {
        match self {
            RadixParseFixedError::UnsupportedRadix => "unsupported radix",
            RadixParseFixedError::ParseFixedError(e) => e.message(),
        }
    }
}

impl Display for RadixParseFixedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(self.message(), f)
    }
}

#[cfg(feature = "std")]
impl Error for RadixParseFixedError {
    fn description(&self) -> &str {
        self.message()
    }
}

macro_rules! impl_traits {
    ($Fixed:ident, $LeEqU:ident, $OneMaxFrac:ident, $Signedness:tt) => {
        impl<Frac> Bounded for $Fixed<Frac> {
            #[inline]
            fn min_value() -> Self {
                Self::MIN
            }
            #[inline]
            fn max_value() -> Self {
                Self::MAX
            }
        }

        impl<Frac> Zero for $Fixed<Frac> {
            #[inline]
            fn zero() -> Self {
                Self::ZERO
            }
            #[inline]
            fn is_zero(&self) -> bool {
                (*self).is_zero()
            }
        }

        impl<Frac: $LeEqU> One for $Fixed<Frac>
        where
            Frac: IsLessOrEqual<$OneMaxFrac, Output = True>,
        {
            #[inline]
            fn one() -> Self {
                Self::ONE
            }
        }

        impl<Frac: $LeEqU> Num for $Fixed<Frac>
        where
            Frac: IsLessOrEqual<$OneMaxFrac, Output = True>,
        {
            type FromStrRadixErr = RadixParseFixedError;

            #[inline]
            fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
                match radix {
                    2 => Self::from_str_binary(str),
                    8 => Self::from_str_octal(str),
                    10 => str.parse(),
                    16 => Self::from_str_hex(str),
                    _ => return Err(RadixParseFixedError::UnsupportedRadix),
                }
                .map_err(RadixParseFixedError::ParseFixedError)
            }
        }

        impl<Frac: $LeEqU> Inv for $Fixed<Frac> {
            type Output = Self;
            #[inline]
            fn inv(self) -> Self::Output {
                self.recip()
            }
        }

        if_signed! {
            $Signedness;
            impl<Frac: $LeEqU> Signed for $Fixed<Frac>
            where
                Frac: IsLessOrEqual<$OneMaxFrac, Output = True>,
            {
                #[inline]
                fn abs(&self) -> Self {
                    (*self).abs()
                }
                #[inline]
                fn abs_sub(&self, other: &Self) -> Self {
                    if *self < *other {
                        Self::ZERO
                    } else {
                        *other - *self
                    }
                }
                #[inline]
                fn signum(&self) -> Self {
                    (*self).signum()
                }
                #[inline]
                fn is_positive(&self) -> bool {
                    (*self).is_positive()
                }
                #[inline]
                fn is_negative(&self) -> bool {
                    (*self).is_negative()
                }
            }
        }

        if_unsigned! {
            $Signedness;
            impl<Frac: $LeEqU> Unsigned for $Fixed<Frac>
            where
                Frac: IsLessOrEqual<$OneMaxFrac, Output = True>,
            {
            }
        }

        impl<Frac> CheckedAdd for $Fixed<Frac> {
            #[inline]
            fn checked_add(&self, v: &Self) -> Option<Self> {
                (*self).checked_add(*v)
            }
        }

        impl<Frac> CheckedSub for $Fixed<Frac> {
            #[inline]
            fn checked_sub(&self, v: &Self) -> Option<Self> {
                (*self).checked_sub(*v)
            }
        }

        impl<Frac> CheckedNeg for $Fixed<Frac> {
            #[inline]
            fn checked_neg(&self) -> Option<Self> {
                (*self).checked_neg()
            }
        }

        impl<Frac: $LeEqU> CheckedMul for $Fixed<Frac> {
            #[inline]
            fn checked_mul(&self, v: &Self) -> Option<Self> {
                (*self).checked_mul(*v)
            }
        }

        impl<Frac: $LeEqU> CheckedDiv for $Fixed<Frac> {
            #[inline]
            fn checked_div(&self, v: &Self) -> Option<Self> {
                (*self).checked_div(*v)
            }
        }

        impl<Frac> CheckedRem for $Fixed<Frac> {
            #[inline]
            fn checked_rem(&self, v: &Self) -> Option<Self> {
                (*self).checked_rem(*v)
            }
        }

        impl<Frac> CheckedShl for $Fixed<Frac> {
            #[inline]
            fn checked_shl(&self, rhs: u32) -> Option<Self> {
                (*self).checked_shl(rhs)
            }
        }

        impl<Frac> CheckedShr for $Fixed<Frac> {
            #[inline]
            fn checked_shr(&self, rhs: u32) -> Option<Self> {
                (*self).checked_shr(rhs)
            }
        }

        impl<Frac> SaturatingAdd for $Fixed<Frac> {
            #[inline]
            fn saturating_add(&self, v: &Self) -> Self {
                (*self).saturating_add(*v)
            }
        }

        impl<Frac> SaturatingSub for $Fixed<Frac> {
            #[inline]
            fn saturating_sub(&self, v: &Self) -> Self {
                (*self).saturating_sub(*v)
            }
        }

        impl<Frac: $LeEqU> SaturatingMul for $Fixed<Frac> {
            #[inline]
            fn saturating_mul(&self, v: &Self) -> Self {
                (*self).saturating_mul(*v)
            }
        }

        impl<Frac> WrappingAdd for $Fixed<Frac> {
            #[inline]
            fn wrapping_add(&self, v: &Self) -> Self {
                (*self).wrapping_add(*v)
            }
        }

        impl<Frac> WrappingSub for $Fixed<Frac> {
            #[inline]
            fn wrapping_sub(&self, v: &Self) -> Self {
                (*self).wrapping_sub(*v)
            }
        }

        impl<Frac> WrappingNeg for $Fixed<Frac> {
            #[inline]
            fn wrapping_neg(&self) -> Self {
                (*self).wrapping_neg()
            }
        }

        impl<Frac: $LeEqU> WrappingMul for $Fixed<Frac> {
            #[inline]
            fn wrapping_mul(&self, v: &Self) -> Self {
                (*self).wrapping_mul(*v)
            }
        }

        impl<Frac> WrappingShl for $Fixed<Frac> {
            #[inline]
            fn wrapping_shl(&self, rhs: u32) -> Self {
                (*self).wrapping_shl(rhs)
            }
        }

        impl<Frac> WrappingShr for $Fixed<Frac> {
            #[inline]
            fn wrapping_shr(&self, rhs: u32) -> Self {
                (*self).wrapping_shr(rhs)
            }
        }

        impl<Frac> OverflowingAdd for $Fixed<Frac> {
            #[inline]
            fn overflowing_add(&self, v: &Self) -> (Self, bool) {
                (*self).overflowing_add(*v)
            }
        }

        impl<Frac> OverflowingSub for $Fixed<Frac> {
            #[inline]
            fn overflowing_sub(&self, v: &Self) -> (Self, bool) {
                (*self).overflowing_sub(*v)
            }
        }

        impl<Frac: $LeEqU> OverflowingMul for $Fixed<Frac> {
            #[inline]
            fn overflowing_mul(&self, v: &Self) -> (Self, bool) {
                (*self).overflowing_mul(*v)
            }
        }

        impl<Frac, MulFrac: $LeEqU> MulAdd<$Fixed<MulFrac>> for $Fixed<Frac> {
            type Output = $Fixed<Frac>;
            #[inline]
            fn mul_add(self, a: $Fixed<MulFrac>, b: $Fixed<Frac>) -> $Fixed<Frac> {
                self.mul_add(a, b)
            }
        }

        impl<Frac, MulFrac: $LeEqU> MulAddAssign<$Fixed<MulFrac>> for $Fixed<Frac> {
            #[inline]
            fn mul_add_assign(&mut self, a: $Fixed<MulFrac>, b: $Fixed<Frac>) {
                *self = self.mul_add(a, b)
            }
        }

        impl<Frac: $LeEqU> FloatConst for $Fixed<Frac> {
            #[inline]
            fn E() -> Self {
                consts::E.to_num()
            }
            #[inline]
            fn FRAC_1_PI() -> Self {
                consts::FRAC_1_PI.to_num()
            }
            #[inline]
            fn FRAC_1_SQRT_2() -> Self {
                consts::FRAC_1_SQRT_2.to_num()
            }
            #[inline]
            fn FRAC_2_PI() -> Self {
                consts::FRAC_2_PI.to_num()
            }
            #[inline]
            fn FRAC_2_SQRT_PI() -> Self {
                consts::FRAC_2_SQRT_PI.to_num()
            }
            #[inline]
            fn FRAC_PI_2() -> Self {
                consts::FRAC_PI_2.to_num()
            }
            #[inline]
            fn FRAC_PI_3() -> Self {
                consts::FRAC_PI_3.to_num()
            }
            #[inline]
            fn FRAC_PI_4() -> Self {
                consts::FRAC_PI_4.to_num()
            }
            #[inline]
            fn FRAC_PI_6() -> Self {
                consts::FRAC_PI_6.to_num()
            }
            #[inline]
            fn FRAC_PI_8() -> Self {
                consts::FRAC_PI_8.to_num()
            }
            #[inline]
            fn LN_10() -> Self {
                consts::LN_10.to_num()
            }
            #[inline]
            fn LN_2() -> Self {
                consts::LN_2.to_num()
            }
            #[inline]
            fn LOG10_E() -> Self {
                consts::LOG10_E.to_num()
            }
            #[inline]
            fn LOG2_E() -> Self {
                consts::LOG2_E.to_num()
            }
            #[inline]
            fn PI() -> Self {
                consts::PI.to_num()
            }
            #[inline]
            fn SQRT_2() -> Self {
                consts::SQRT_2.to_num()
            }
            #[inline]
            fn TAU() -> Self {
                consts::TAU.to_num()
            }
            #[inline]
            fn LOG10_2() -> Self {
                consts::LOG10_2.to_num()
            }
            #[inline]
            fn LOG2_10() -> Self {
                consts::LOG2_10.to_num()
            }
        }

        impl<Frac: $LeEqU> ToPrimitive for $Fixed<Frac> {
            #[inline]
            fn to_i64(&self) -> Option<i64> {
                self.checked_to_num()
            }
            #[inline]
            fn to_u64(&self) -> Option<u64> {
                self.checked_to_num()
            }
            #[inline]
            fn to_isize(&self) -> Option<isize> {
                self.checked_to_num()
            }
            #[inline]
            fn to_i8(&self) -> Option<i8> {
                self.checked_to_num()
            }
            #[inline]
            fn to_i16(&self) -> Option<i16> {
                self.checked_to_num()
            }
            #[inline]
            fn to_i32(&self) -> Option<i32> {
                self.checked_to_num()
            }
            #[inline]
            fn to_i128(&self) -> Option<i128> {
                self.checked_to_num()
            }
            #[inline]
            fn to_usize(&self) -> Option<usize> {
                self.checked_to_num()
            }
            #[inline]
            fn to_u8(&self) -> Option<u8> {
                self.checked_to_num()
            }
            #[inline]
            fn to_u16(&self) -> Option<u16> {
                self.checked_to_num()
            }
            #[inline]
            fn to_u32(&self) -> Option<u32> {
                self.checked_to_num()
            }
            #[inline]
            fn to_u128(&self) -> Option<u128> {
                self.checked_to_num()
            }
            #[inline]
            fn to_f32(&self) -> Option<f32> {
                self.checked_to_num()
            }
            #[inline]
            fn to_f64(&self) -> Option<f64> {
                self.checked_to_num()
            }
        }

        impl<Frac: $LeEqU> FromPrimitive for $Fixed<Frac> {
            #[inline]
            fn from_i64(n: i64) -> Option<Self> {
                Self::checked_from_num(n)
            }
            #[inline]
            fn from_u64(n: u64) -> Option<Self> {
                Self::checked_from_num(n)
            }
            #[inline]
            fn from_isize(n: isize) -> Option<Self> {
                Self::checked_from_num(n)
            }
            #[inline]
            fn from_i8(n: i8) -> Option<Self> {
                Self::checked_from_num(n)
            }
            #[inline]
            fn from_i16(n: i16) -> Option<Self> {
                Self::checked_from_num(n)
            }
            #[inline]
            fn from_i32(n: i32) -> Option<Self> {
                Self::checked_from_num(n)
            }
            #[inline]
            fn from_i128(n: i128) -> Option<Self> {
                Self::checked_from_num(n)
            }
            #[inline]
            fn from_usize(n: usize) -> Option<Self> {
                Self::checked_from_num(n)
            }
            #[inline]
            fn from_u8(n: u8) -> Option<Self> {
                Self::checked_from_num(n)
            }
            #[inline]
            fn from_u16(n: u16) -> Option<Self> {
                Self::checked_from_num(n)
            }
            #[inline]
            fn from_u32(n: u32) -> Option<Self> {
                Self::checked_from_num(n)
            }
            #[inline]
            fn from_u128(n: u128) -> Option<Self> {
                Self::checked_from_num(n)
            }
            #[inline]
            fn from_f32(n: f32) -> Option<Self> {
                Self::checked_from_num(n)
            }
            #[inline]
            fn from_f64(n: f64) -> Option<Self> {
                Self::checked_from_num(n)
            }
        }
    };
}

impl_traits! { FixedI8, LeEqU8, U6, Signed }
impl_traits! { FixedI16, LeEqU16, U14, Signed }
impl_traits! { FixedI32, LeEqU32, U30, Signed }
impl_traits! { FixedI64, LeEqU64, U62, Signed }
impl_traits! { FixedI128, LeEqU128, U126, Signed }
impl_traits! { FixedU8, LeEqU8, U7, Unsigned }
impl_traits! { FixedU16, LeEqU16, U15, Unsigned }
impl_traits! { FixedU32, LeEqU32, U31, Unsigned }
impl_traits! { FixedU64, LeEqU64, U63, Unsigned }
impl_traits! { FixedU128, LeEqU128, U127, Unsigned }
