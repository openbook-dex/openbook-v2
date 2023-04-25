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
    int_helper::IntFixed,
    traits::{FromFixed, LosslessTryFrom, LossyFrom, ToFixed},
    types::extra::{
        Diff, IsLessOrEqual, LeEqU128, LeEqU16, LeEqU32, LeEqU64, LeEqU8, True, U0, U1, U127, U128,
        U15, U16, U24, U31, U32, U63, U64, U7, U8,
    },
    F128Bits, FixedI128, FixedI16, FixedI32, FixedI64, FixedI8, FixedU128, FixedU16, FixedU32,
    FixedU64, FixedU8,
};
use core::{convert::TryFrom, ops::Sub};
use half::{bf16, f16};

macro_rules! convert {
    (
        ($SrcU:ident, $SrcI:ident, $SrcBits:ident, $SrcLeEqU:ident) ->
            ($DstU:ident, $DstI:ident, $DstBits:ident, $DstBitsM1:ident, $DstLeEqU:ident)
    ) => {
        impl<FracSrc: $SrcLeEqU, FracDst: $DstLeEqU> From<$SrcU<FracSrc>> for $DstU<FracDst>
        where
            FracSrc: IsLessOrEqual<FracDst, Output = True>,
            $SrcBits: Sub<FracSrc>,
            $DstBits: Sub<FracDst>,
            Diff<$SrcBits, FracSrc>: IsLessOrEqual<Diff<$DstBits, FracDst>, Output = True>,
        {
            /// Converts a fixed-pint number.
            ///
            /// This conversion never fails (infallible) and does not
            /// lose any precision (lossless).
            #[inline]
            fn from(src: $SrcU<FracSrc>) -> Self {
                let unshifted = Self::from_bits(src.to_bits().into()).to_bits();
                let shift = FracDst::U32 - FracSrc::U32;
                Self::from_bits(unshifted << shift)
            }
        }

        impl<FracSrc: $SrcLeEqU, FracDst: $DstLeEqU> From<$SrcI<FracSrc>> for $DstI<FracDst>
        where
            FracSrc: IsLessOrEqual<FracDst, Output = True>,
            $SrcBits: Sub<FracSrc>,
            $DstBits: Sub<FracDst>,
            Diff<$SrcBits, FracSrc>: IsLessOrEqual<Diff<$DstBits, FracDst>, Output = True>,
        {
            /// Converts a fixed-pint number.
            ///
            /// This conversion never fails (infallible) and does not
            /// lose any precision (lossless).
            #[inline]
            fn from(src: $SrcI<FracSrc>) -> Self {
                let unshifted = Self::from_bits(src.to_bits().into()).to_bits();
                let shift = FracDst::U32 - FracSrc::U32;
                Self::from_bits(unshifted << shift)
            }
        }

        impl<FracSrc: $SrcLeEqU, FracDst: $DstLeEqU> From<$SrcU<FracSrc>> for $DstI<FracDst>
        where
            FracSrc: IsLessOrEqual<FracDst, Output = True>,
            $SrcBits: Sub<FracSrc>,
            $DstBitsM1: Sub<FracDst>,
            Diff<$SrcBits, FracSrc>: IsLessOrEqual<Diff<$DstBitsM1, FracDst>, Output = True>,
        {
            /// Converts a fixed-pint number.
            ///
            /// This conversion never fails (infallible) and does not
            /// lose any precision (lossless).
            #[inline]
            fn from(src: $SrcU<FracSrc>) -> Self {
                let unshifted = Self::from_bits(src.to_bits().into()).to_bits();
                let shift = FracDst::U32 - FracSrc::U32;
                Self::from_bits(unshifted << shift)
            }
        }
    };
}

macro_rules! convert_lossless {
    (
        ($Src:ident, $SrcBits:ident, $SrcLeEqU:ident) ->
            ($Dst:ident, $DstBits:ident, $DstLeEqU:ident)
    ) => {
        // lossless because Src::FRAC_NBITS <= Dst::FRAC_NBITS
        impl<FracSrc: $SrcLeEqU, FracDst: $DstLeEqU> LosslessTryFrom<$Src<FracSrc>>
            for $Dst<FracDst>
        where
            FracSrc: IsLessOrEqual<FracDst, Output = True>,
        {
            /// Converts a fixed-pint number.
            ///
            /// This conversion may fail (fallible) but does not lose
            /// precision (lossless).
            #[inline]
            fn lossless_try_from(src: $Src<FracSrc>) -> Option<Self> {
                Self::checked_from_fixed(src)
            }
        }
    };
    ($Src:ident, $SrcBits:ident, $SrcLeEqU:ident) => {
        convert_lossless! { ($Src, $SrcBits, $SrcLeEqU) -> (FixedI8, U8, LeEqU8) }
        convert_lossless! { ($Src, $SrcBits, $SrcLeEqU) -> (FixedI16, U16, LeEqU16) }
        convert_lossless! { ($Src, $SrcBits, $SrcLeEqU) -> (FixedI32, U32, LeEqU32) }
        convert_lossless! { ($Src, $SrcBits, $SrcLeEqU) -> (FixedI64, U64, LeEqU64) }
        convert_lossless! { ($Src, $SrcBits, $SrcLeEqU) -> (FixedI128, U128, LeEqU128) }
        convert_lossless! { ($Src, $SrcBits, $SrcLeEqU) -> (FixedU8, U8, LeEqU8) }
        convert_lossless! { ($Src, $SrcBits, $SrcLeEqU) -> (FixedU16, U16, LeEqU16) }
        convert_lossless! { ($Src, $SrcBits, $SrcLeEqU) -> (FixedU32, U32, LeEqU32) }
        convert_lossless! { ($Src, $SrcBits, $SrcLeEqU) -> (FixedU64, U64, LeEqU64) }
        convert_lossless! { ($Src, $SrcBits, $SrcLeEqU) -> (FixedU128, U128, LeEqU128) }
    };
}

macro_rules! convert_lossy {
    (
        ($SrcU:ident, $SrcI:ident, $SrcBits:ident, $SrcLeEqU:ident) ->
            ($DstU:ident, $DstI:ident, $DstBits:ident, $DstBitsM1:ident, $DstLeEqU:ident)
    ) => {
        // unsigned -> unsigned, infallible because Src::INT_NBITS <= Dst::INT_NBITS
        impl<FracSrc: $SrcLeEqU, FracDst: $DstLeEqU> LossyFrom<$SrcU<FracSrc>> for $DstU<FracDst>
        where
            $SrcBits: Sub<FracSrc>,
            $DstBits: Sub<FracDst>,
            Diff<$SrcBits, FracSrc>: IsLessOrEqual<Diff<$DstBits, FracDst>, Output = True>,
        {
            /// Converts a fixed-pint number.
            ///
            /// This conversion never fails (infallible) but may lose
            /// precision (lossy). Any fractional bits in the source
            /// that cannot be represented in the destination are
            /// discarded, which rounds towards −∞.
            #[inline]
            fn lossy_from(src: $SrcU<FracSrc>) -> Self {
                src.to_num()
            }
        }

        // signed -> signed, infallible because Src::INT_NBITS <= Dst::INT_NBITS
        impl<FracSrc: $SrcLeEqU, FracDst: $DstLeEqU> LossyFrom<$SrcI<FracSrc>> for $DstI<FracDst>
        where
            $SrcBits: Sub<FracSrc>,
            $DstBits: Sub<FracDst>,
            Diff<$SrcBits, FracSrc>: IsLessOrEqual<Diff<$DstBits, FracDst>, Output = True>,
        {
            /// Converts a fixed-pint number.
            ///
            /// This conversion never fails (infallible) but may lose
            /// precision (lossy). Any fractional bits in the source
            /// that cannot be represented in the destination are
            /// discarded, which rounds towards −∞.
            #[inline]
            fn lossy_from(src: $SrcI<FracSrc>) -> Self {
                src.to_num()
            }
        }

        // signed -> signed, infallible because Src::INT_NBITS <= Dst::INT_NBITS - 1
        impl<FracSrc: $SrcLeEqU, FracDst: $DstLeEqU> LossyFrom<$SrcU<FracSrc>> for $DstI<FracDst>
        where
            $SrcBits: Sub<FracSrc>,
            $DstBitsM1: Sub<FracDst>,
            Diff<$SrcBits, FracSrc>: IsLessOrEqual<Diff<$DstBitsM1, FracDst>, Output = True>,
        {
            /// Converts a fixed-pint number.
            ///
            /// This conversion never fails (infallible) but may lose
            /// precision (lossy). Any fractional bits in the source
            /// that cannot be represented in the destination are
            /// discarded, which rounds towards −∞.
            #[inline]
            fn lossy_from(src: $SrcU<FracSrc>) -> Self {
                src.to_num()
            }
        }
    };
    ($SrcU:ident, $SrcI:ident, $SrcBits:ident, $SrcLeEqU:ident) => {
        convert_lossy! {
            ($SrcU, $SrcI, $SrcBits, $SrcLeEqU) -> (FixedU8, FixedI8, U8, U7, LeEqU8)
        }
        convert_lossy! {
            ($SrcU, $SrcI, $SrcBits, $SrcLeEqU) -> (FixedU16, FixedI16, U16, U15, LeEqU16)
        }
        convert_lossy! {
            ($SrcU, $SrcI, $SrcBits, $SrcLeEqU) -> (FixedU32, FixedI32, U32, U31, LeEqU32)
        }
        convert_lossy! {
            ($SrcU, $SrcI, $SrcBits, $SrcLeEqU) -> (FixedU64, FixedI64, U64, U63, LeEqU64)
        }
        convert_lossy! {
            ($SrcU, $SrcI, $SrcBits, $SrcLeEqU) -> (FixedU128, FixedI128, U128, U127, LeEqU128)
        }
    };
}

convert! { (FixedU8, FixedI8, U8, LeEqU8) -> (FixedU16, FixedI16, U16, U15, LeEqU16) }
convert! { (FixedU8, FixedI8, U8, LeEqU8) -> (FixedU32, FixedI32, U32, U31, LeEqU32) }
convert! { (FixedU8, FixedI8, U8, LeEqU8) -> (FixedU64, FixedI64, U64, U63, LeEqU64) }
convert! { (FixedU8, FixedI8, U8, LeEqU8) -> (FixedU128, FixedI128, U128, U127, LeEqU128) }

convert! { (FixedU16, FixedI16, U16, LeEqU16) -> (FixedU32, FixedI32, U32, U31, LeEqU32) }
convert! { (FixedU16, FixedI16, U16, LeEqU16) -> (FixedU64, FixedI64, U64, U63, LeEqU64) }
convert! { (FixedU16, FixedI16, U16, LeEqU16) -> (FixedU128, FixedI128, U128, U127, LeEqU128) }

convert! { (FixedU32, FixedI32, U32, LeEqU32) -> (FixedU64, FixedI64, U64, U63, LeEqU64) }
convert! { (FixedU32, FixedI32, U32, LeEqU32) -> (FixedU128, FixedI128, U128, U127, LeEqU128) }

convert! { (FixedU64, FixedI64, U64, LeEqU64) -> (FixedU128, FixedI128, U128, U127, LeEqU128) }

convert_lossless! { FixedI8, U8, LeEqU8 }
convert_lossless! { FixedI16, U16, LeEqU16 }
convert_lossless! { FixedI32, U32, LeEqU32 }
convert_lossless! { FixedI64, U64, LeEqU64 }
convert_lossless! { FixedI128, U128, LeEqU128 }
convert_lossless! { FixedU8, U8, LeEqU8 }
convert_lossless! { FixedU16, U16, LeEqU16 }
convert_lossless! { FixedU32, U32, LeEqU32 }
convert_lossless! { FixedU64, U64, LeEqU64 }
convert_lossless! { FixedU128, U128, LeEqU128 }

convert_lossy! { FixedU8, FixedI8, U8, LeEqU8 }
convert_lossy! { FixedU16, FixedI16, U16, LeEqU16 }
convert_lossy! { FixedU32, FixedI32, U32, LeEqU32 }
convert_lossy! { FixedU64, FixedI64, U64, LeEqU64 }
convert_lossy! { FixedU128, FixedI128, U128, LeEqU128 }

macro_rules! int_to_fixed {
    ($Src:ident, $Dst:ident, $LeEqU:ident) => {
        impl<Frac: $LeEqU> LosslessTryFrom<$Src> for $Dst<Frac> {
            /// Converts an integer to a fixed-point number.
            ///
            /// This conversion may fail (fallible) but cannot lose
            /// any fractional bits (lossless).
            #[inline]
            fn lossless_try_from(src: $Src) -> Option<Self> {
                src.checked_to_fixed()
            }
        }
    };

    ($Src:ident $(, $Dst:ty)?) => {
        $(impl From<$Src> for $Dst {
            /// Converts an integer to a fixed-point number.
            ///
            /// This conversion never fails (infallible) and cannot
            /// lose any fractional bits (lossless).
            #[inline]
            fn from(src: $Src) -> Self {
                Self::from_bits(src)
            }
        })?

        int_to_fixed! { $Src, FixedI8, LeEqU8 }
        int_to_fixed! { $Src, FixedI16, LeEqU16 }
        int_to_fixed! { $Src, FixedI32, LeEqU32 }
        int_to_fixed! { $Src, FixedI64, LeEqU64 }
        int_to_fixed! { $Src, FixedI128, LeEqU128 }
        int_to_fixed! { $Src, FixedU8, LeEqU8 }
        int_to_fixed! { $Src, FixedU16, LeEqU16 }
        int_to_fixed! { $Src, FixedU32, LeEqU32 }
        int_to_fixed! { $Src, FixedU64, LeEqU64 }
        int_to_fixed! { $Src, FixedU128, LeEqU128 }

        $(impl LossyFrom<$Src> for $Dst {
            /// Converts an integer to a fixed-point number.
            ///
            /// This conversion never fails (infallible) and actually
            /// does not lose any precision (lossless).
            #[inline]
            fn lossy_from(src: $Src) -> Self {
                Self::from_bits(src)
            }
        })?
    };
}

int_to_fixed! { bool }
int_to_fixed! { i8, FixedI8<U0> }
int_to_fixed! { i16, FixedI16<U0> }
int_to_fixed! { i32, FixedI32<U0> }
int_to_fixed! { i64, FixedI64<U0> }
int_to_fixed! { i128, FixedI128<U0> }
int_to_fixed! { isize }
int_to_fixed! { u8, FixedU8<U0> }
int_to_fixed! { u16, FixedU16<U0> }
int_to_fixed! { u32, FixedU32<U0> }
int_to_fixed! { u64, FixedU64<U0> }
int_to_fixed! { u128, FixedU128<U0> }
int_to_fixed! { usize }

macro_rules! int_to_wider_fixed {
    (
        ($SrcU:ident, $SrcI:ident, $SrcBits:ident, $SrcLeEqU:ident) ->
            ($DstU:ident, $DstI:ident, $DstBits:ident, $DstBitsM1:ident, $DstLeEqU:ident)
    ) => {
        impl<FracDst: $DstLeEqU> From<$SrcU> for $DstU<FracDst>
        where
            $DstBits: Sub<FracDst>,
            $SrcBits: IsLessOrEqual<Diff<$DstBits, FracDst>, Output = True>,
        {
            /// Converts an integer to a fixed-point number.
            ///
            /// This conversion never fails (infallible) and cannot
            /// lose any fractional bits, so it is actually lossless.
            #[inline]
            fn from(src: $SrcU) -> Self {
                let unshifted = Self::from_bits(src.into()).to_bits();
                let shift = FracDst::U32;
                Self::from_bits(unshifted << shift)
            }
        }

        impl<FracDst: $DstLeEqU> From<$SrcI> for $DstI<FracDst>
        where
            $DstBits: Sub<FracDst>,
            $SrcBits: IsLessOrEqual<Diff<$DstBits, FracDst>, Output = True>,
        {
            /// Converts an integer to a fixed-point number.
            ///
            /// This conversion never fails (infallible) and cannot
            /// lose any fractional bits, so it is actually lossless.
            #[inline]
            fn from(src: $SrcI) -> Self {
                let unshifted = Self::from_bits(src.into()).to_bits();
                let shift = FracDst::U32;
                Self::from_bits(unshifted << shift)
            }
        }

        impl<FracDst: $DstLeEqU> From<$SrcU> for $DstI<FracDst>
        where
            $DstBitsM1: Sub<FracDst>,
            $SrcBits: IsLessOrEqual<Diff<$DstBitsM1, FracDst>, Output = True>,
        {
            /// Converts an integer to a fixed-point number.
            ///
            /// This conversion never fails (infallible) and cannot
            /// lose any fractional bits, so it is actually lossless.
            #[inline]
            fn from(src: $SrcU) -> Self {
                let unshifted = Self::from_bits(src.into()).to_bits();
                let shift = FracDst::U32;
                Self::from_bits(unshifted << shift)
            }
        }

        impl<FracDst: $DstLeEqU> LossyFrom<$SrcU> for $DstU<FracDst>
        where
            $DstBits: Sub<FracDst>,
            $SrcBits: IsLessOrEqual<Diff<$DstBits, FracDst>, Output = True>,
        {
            /// Converts an integer to a fixed-point number.
            ///
            /// This conversion never fails (infallible) and cannot
            /// lose any fractional bits, so it is actually lossless.
            #[inline]
            fn lossy_from(src: $SrcU) -> Self {
                src.into()
            }
        }

        impl<FracDst: $DstLeEqU> LossyFrom<$SrcI> for $DstI<FracDst>
        where
            $DstBits: Sub<FracDst>,
            $SrcBits: IsLessOrEqual<Diff<$DstBits, FracDst>, Output = True>,
        {
            /// Converts an integer to a fixed-point number.
            ///
            /// This conversion never fails (infallible) and cannot
            /// lose any fractional bits, so it is actually lossless.
            #[inline]
            fn lossy_from(src: $SrcI) -> Self {
                src.into()
            }
        }

        impl<FracDst: $DstLeEqU> LossyFrom<$SrcU> for $DstI<FracDst>
        where
            $DstBitsM1: Sub<FracDst>,
            $SrcBits: IsLessOrEqual<Diff<$DstBitsM1, FracDst>, Output = True>,
        {
            /// Converts an integer to a fixed-point number.
            ///
            /// This conversion never fails (infallible) and cannot
            /// lose any fractional bits, so it is actually lossless.
            #[inline]
            fn lossy_from(src: $SrcU) -> Self {
                src.into()
            }
        }
    };
}

int_to_wider_fixed! { (u8, i8, U8, LeEqU8) -> (FixedU16, FixedI16, U16, U15, LeEqU16) }
int_to_wider_fixed! { (u8, i8, U8, LeEqU8) -> (FixedU32, FixedI32, U32, U31, LeEqU32) }
int_to_wider_fixed! { (u8, i8, U8, LeEqU8) -> (FixedU64, FixedI64, U64, U63, LeEqU64) }
int_to_wider_fixed! { (u8, i8, U8, LeEqU8) -> (FixedU128, FixedI128, U128, U127, LeEqU128) }
int_to_wider_fixed! { (u16, i16, U16, LeEqU16) -> (FixedU32, FixedI32, U32, U31, LeEqU32) }
int_to_wider_fixed! { (u16, i16, U16, LeEqU16) -> (FixedU64, FixedI64, U64, U63, LeEqU64) }
int_to_wider_fixed! { (u16, i16, U16, LeEqU16) -> (FixedU128, FixedI128, U128, U127, LeEqU128) }
int_to_wider_fixed! { (u32, i32, U32, LeEqU32) -> (FixedU64, FixedI64, U64, U63, LeEqU64) }
int_to_wider_fixed! { (u32, i32, U32, LeEqU32) -> (FixedU128, FixedI128, U128, U127, LeEqU128) }
int_to_wider_fixed! { (u64, i64, U64, LeEqU64) -> (FixedU128, FixedI128, U128, U127, LeEqU128) }

macro_rules! bool_to_fixed {
    ($DstU:ident, $DstI:ident, $DstBits:ident, $DstBitsM1:ident, $DstLeEqU:ident) => {
        impl<FracDst: $DstLeEqU> From<bool> for $DstU<FracDst>
        where
            $DstBits: Sub<FracDst>,
            U1: IsLessOrEqual<Diff<$DstBits, FracDst>, Output = True>,
        {
            /// Converts a [`bool`] to a fixed-point number.
            ///
            /// This conversion never fails (infallible) and cannot
            /// lose any fractional bits (lossless).
            #[inline]
            fn from(src: bool) -> Self {
                let unshifted = Self::from_bits(src.into()).to_bits();
                let shift = FracDst::U32;
                Self::from_bits(unshifted << shift)
            }
        }

        impl<FracDst: $DstLeEqU> From<bool> for $DstI<FracDst>
        where
            $DstBitsM1: Sub<FracDst>,
            U1: IsLessOrEqual<Diff<$DstBitsM1, FracDst>, Output = True>,
        {
            /// Converts a [`bool`] to a fixed-point number.
            ///
            /// This conversion never fails (infallible) and cannot
            /// lose any fractional bits (lossless).
            #[inline]
            fn from(src: bool) -> Self {
                let unshifted = Self::from_bits(src.into()).to_bits();
                let shift = FracDst::U32;
                Self::from_bits(unshifted << shift)
            }
        }

        impl<FracDst: $DstLeEqU> LossyFrom<bool> for $DstU<FracDst>
        where
            $DstBits: Sub<FracDst>,
            U1: IsLessOrEqual<Diff<$DstBits, FracDst>, Output = True>,
        {
            /// Converts a [`bool`] to a fixed-point number.
            ///
            /// This conversion never fails (infallible) and cannot
            /// lose any fractional bits, so it is actually lossless.
            #[inline]
            fn lossy_from(src: bool) -> Self {
                src.into()
            }
        }

        impl<FracDst: $DstLeEqU> LossyFrom<bool> for $DstI<FracDst>
        where
            $DstBitsM1: Sub<FracDst>,
            U1: IsLessOrEqual<Diff<$DstBitsM1, FracDst>, Output = True>,
        {
            /// Converts a [`bool`] to a fixed-point number.
            ///
            /// This conversion never fails (infallible) and cannot
            /// lose any fractional bits, so it is actually lossless.
            #[inline]
            fn lossy_from(src: bool) -> Self {
                src.into()
            }
        }
    };
}

bool_to_fixed! { FixedU8, FixedI8, U8, U7, LeEqU8 }
bool_to_fixed! { FixedU16, FixedI16, U16, U15, LeEqU16 }
bool_to_fixed! { FixedU32, FixedI32, U32, U31, LeEqU32 }
bool_to_fixed! { FixedU64, FixedI64, U64, U63, LeEqU64 }
bool_to_fixed! { FixedU128, FixedI128, U128, U127, LeEqU128 }

macro_rules! fixed_to_int {
    (($SrcU:ident, $SrcI:ident) -> ($DstU:ident, $DstI:ident)) => {
        impl From<$SrcU<U0>> for $DstU {
            /// Converts a fixed-point number with no fractional bits to an integer.
            ///
            /// This conversion never fails (infallible) and cannot
            /// lose any fractional bits (lossless).
            #[inline]
            fn from(src: $SrcU<U0>) -> Self {
                src.to_bits().into()
            }
        }

        impl From<$SrcI<U0>> for $DstI {
            /// Converts a fixed-point number with no fractional bits to an integer.
            ///
            /// This conversion never fails (infallible) and cannot
            /// lose any fractional bits (lossless).
            #[inline]
            fn from(src: $SrcI<U0>) -> Self {
                src.to_bits().into()
            }
        }
    };
    (($SrcU:ident, $SrcI:ident) -> wider ($DstU:ident, $DstI:ident)) => {
        fixed_to_int! { ($SrcU, $SrcI) -> ($DstU, $DstI) }

        impl From<$SrcU<U0>> for $DstI {
            /// Converts a fixed-point number with no fractional bits to an integer.
            ///
            /// This conversion never fails (infallible) and cannot
            /// lose any fractional bits (lossless).
            #[inline]
            fn from(src: $SrcU<U0>) -> Self {
                src.to_bits().into()
            }
        }
    };
}

fixed_to_int! { (FixedU8, FixedI8) -> (u8, i8) }
fixed_to_int! { (FixedU8, FixedI8) -> wider (u16, i16) }
fixed_to_int! { (FixedU8, FixedI8) -> wider (u32, i32) }
fixed_to_int! { (FixedU8, FixedI8) -> wider (u64, i64) }
fixed_to_int! { (FixedU8, FixedI8) -> wider (u128, i128) }
fixed_to_int! { (FixedU8, FixedI8) -> wider (usize, isize) }

fixed_to_int! { (FixedU16, FixedI16) -> (u16, i16) }
fixed_to_int! { (FixedU16, FixedI16) -> wider (u32, i32) }
fixed_to_int! { (FixedU16, FixedI16) -> wider (u64, i64) }
fixed_to_int! { (FixedU16, FixedI16) -> wider (u128, i128) }
fixed_to_int! { (FixedU16, FixedI16) -> (usize, isize) }

fixed_to_int! { (FixedU32, FixedI32) -> (u32, i32) }
fixed_to_int! { (FixedU32, FixedI32) -> wider (u64, i64) }
fixed_to_int! { (FixedU32, FixedI32) -> wider (u128, i128) }

fixed_to_int! { (FixedU64, FixedI64) -> (u64, i64) }
fixed_to_int! { (FixedU64, FixedI64) -> wider (u128, i128) }

fixed_to_int! { (FixedU128, FixedI128) -> (u128, i128) }

macro_rules! fixed_to_int_lossless {
    ($Src:ty, $Dst:ident) => {
        impl LosslessTryFrom<$Src> for $Dst {
            /// Converts a fixed-point number to an integer.
            ///
            /// This conversion may fail (fallible) but cannot lose
            /// any fractional bits (lossless).
            #[inline]
            fn lossless_try_from(src: $Src) -> Option<Self> {
                $Dst::checked_from_fixed(src)
            }
        }
    };

    ($Src:ty) => {
        fixed_to_int_lossless! { $Src, i8 }
        fixed_to_int_lossless! { $Src, i16 }
        fixed_to_int_lossless! { $Src, i32 }
        fixed_to_int_lossless! { $Src, i64 }
        fixed_to_int_lossless! { $Src, i128 }
        fixed_to_int_lossless! { $Src, isize }
        fixed_to_int_lossless! { $Src, u8 }
        fixed_to_int_lossless! { $Src, u16 }
        fixed_to_int_lossless! { $Src, u32 }
        fixed_to_int_lossless! { $Src, u64 }
        fixed_to_int_lossless! { $Src, u128 }
        fixed_to_int_lossless! { $Src, usize }
    };
}

fixed_to_int_lossless! { FixedI8<U0>}
fixed_to_int_lossless! { FixedI16<U0>}
fixed_to_int_lossless! { FixedI32<U0>}
fixed_to_int_lossless! { FixedI64<U0>}
fixed_to_int_lossless! { FixedI128<U0>}
fixed_to_int_lossless! { FixedU8<U0>}
fixed_to_int_lossless! { FixedU16<U0>}
fixed_to_int_lossless! { FixedU32<U0>}
fixed_to_int_lossless! { FixedU64<U0>}
fixed_to_int_lossless! { FixedU128<U0>}

macro_rules! fixed_to_int_lossy {
    (
        ($SrcU:ident, $SrcI:ident, $SrcBits:ident, $SrcLeEqU:ident) ->
            ($DstU:ident, $DstI:ident, $DstBits:ident, $DstBitsM1:ident, $DstLeEqU:ident)
    ) => {
        impl<FracSrc: $SrcLeEqU> LossyFrom<$SrcU<FracSrc>> for $DstU
        where
            $SrcBits: Sub<FracSrc>,
            Diff<$SrcBits, FracSrc>: IsLessOrEqual<$DstBits, Output = True>,
        {
            /// Converts a fixed-point number to an integer.
            ///
            /// This conversion never fails (infallible) but may lose
            /// precision (lossy). Any fractional bits in the source
            /// are discarded, which rounds towards −∞.
            #[inline]
            fn lossy_from(src: $SrcU<FracSrc>) -> Self {
                src.to_num()
            }
        }

        impl<FracSrc: $SrcLeEqU> LossyFrom<$SrcI<FracSrc>> for $DstI
        where
            $SrcBits: Sub<FracSrc>,
            Diff<$SrcBits, FracSrc>: IsLessOrEqual<$DstBits, Output = True>,
        {
            /// Converts a fixed-point number to an integer.
            ///
            /// This conversion never fails (infallible) but may lose
            /// precision (lossy). Any fractional bits in the source
            /// are discarded, which rounds towards −∞.
            #[inline]
            fn lossy_from(src: $SrcI<FracSrc>) -> Self {
                src.to_num()
            }
        }

        impl<FracSrc: $SrcLeEqU> LossyFrom<$SrcU<FracSrc>> for $DstI
        where
            $SrcBits: Sub<FracSrc>,
            Diff<$SrcBits, FracSrc>: IsLessOrEqual<$DstBitsM1, Output = True>,
        {
            /// Converts a fixed-point number to an integer.
            ///
            /// This conversion never fails (infallible) but may lose
            /// precision (lossy). Any fractional bits in the source
            /// are discarded, which rounds towards −∞.
            #[inline]
            fn lossy_from(src: $SrcU<FracSrc>) -> Self {
                src.to_num()
            }
        }
    };
    ($SrcU:ident, $SrcI:ident, $SrcBits:ident, $SrcLeEqU:ident) => {
        fixed_to_int_lossy! {
            ($SrcU, $SrcI, $SrcBits, $SrcLeEqU) -> (u8, i8, U8, U7, LeEqU8)
        }
        fixed_to_int_lossy! {
            ($SrcU, $SrcI, $SrcBits, $SrcLeEqU) -> (u16, i16, U16, U15, LeEqU16)
        }
        fixed_to_int_lossy! {
            ($SrcU, $SrcI, $SrcBits, $SrcLeEqU) -> (u32, i32, U32, U31, LeEqU32)
        }
        fixed_to_int_lossy! {
            ($SrcU, $SrcI, $SrcBits, $SrcLeEqU) -> (u64, i64, U64, U63, LeEqU64)
        }
        fixed_to_int_lossy! {
            ($SrcU, $SrcI, $SrcBits, $SrcLeEqU) -> (u128, i128, U128, U127, LeEqU128)
        }
        fixed_to_int_lossy! {
            ($SrcU, $SrcI, $SrcBits, $SrcLeEqU) -> (usize, isize, U16, U15, LeEqU16)
        }
    };
}

fixed_to_int_lossy! { FixedU8, FixedI8, U8, LeEqU8 }
fixed_to_int_lossy! { FixedU16, FixedI16, U16, LeEqU16 }
fixed_to_int_lossy! { FixedU32, FixedI32, U32, LeEqU32 }
fixed_to_int_lossy! { FixedU64, FixedI64, U64, LeEqU64 }
fixed_to_int_lossy! { FixedU128, FixedI128, U128, LeEqU128 }

// f16 has minimum subnormal == 2 ^ -(14 + 10) => 24 fractional bits
// bf16 has minimum subnormal == 2 ^ -(126 + 7) => 133 fractional bits
// f32 has minimum subnormal == 2 ^ -(126 + 23) => 149 fractional bits
// f64 has minimum subnormal == 2 ^ -(1022 + 52) => 1074 fractional bits
//
// The only lossless float to fixed possible is from f16 to
// fixed-point numbers with 24 or more fractional bits.
macro_rules! float_to_fixed {
    ($Src:ident, $Dst:ident, $LeEqU:ident) => {
        impl<Frac: $LeEqU> LosslessTryFrom<$Src> for $Dst<Frac>
        where
            U24: IsLessOrEqual<Frac, Output = True>,
        {
            /// Converts a floating-point number to a fixed-point
            /// number.
            ///
            /// This conversion may fail (fallible) but cannot lose
            /// any fractional bits (lossless).
            #[inline]
            fn lossless_try_from(src: $Src) -> Option<Self> {
                src.checked_to_fixed()
            }
        }
    };
}
float_to_fixed! { f16, FixedI32, LeEqU32 }
float_to_fixed! { f16, FixedI64, LeEqU64 }
float_to_fixed! { f16, FixedI128, LeEqU128 }

macro_rules! fixed_to_float {
    ($Fixed:ident($LeEqU:ident) -> $Float:ident) => {
        impl<Frac: $LeEqU> From<$Fixed<Frac>> for $Float {
            /// Converts a fixed-point number to a floating-point number.
            ///
            /// This conversion never fails (infallible) and does not
            /// lose any precision (lossless).
            #[inline]
            fn from(src: $Fixed<Frac>) -> $Float {
                $Float::from_fixed(src)
            }
        }

        impl<Frac: $LeEqU> LosslessTryFrom<$Fixed<Frac>> for $Float {
            /// Converts a fixed-point number to a floating-point number.
            ///
            /// This conversion actually never fails (infallible) but
            /// does not lose any precision (lossless).
            #[inline]
            fn lossless_try_from(src: $Fixed<Frac>) -> Option<$Float> {
                Some($Float::from_fixed(src))
            }
        }
    };
}

fixed_to_float! { FixedI8(LeEqU8) -> f16 }
fixed_to_float! { FixedU8(LeEqU8) -> f16 }
fixed_to_float! { FixedI8(LeEqU8) -> f32 }
fixed_to_float! { FixedI16(LeEqU16) -> f32 }
fixed_to_float! { FixedU8(LeEqU8) -> f32 }
fixed_to_float! { FixedU16(LeEqU16) -> f32 }
fixed_to_float! { FixedI8(LeEqU8) -> f64 }
fixed_to_float! { FixedI16(LeEqU16) -> f64 }
fixed_to_float! { FixedI32(LeEqU32) -> f64 }
fixed_to_float! { FixedU8(LeEqU8) -> f64 }
fixed_to_float! { FixedU16(LeEqU16) -> f64 }
fixed_to_float! { FixedU32(LeEqU32) -> f64 }
fixed_to_float! { FixedI8(LeEqU8) -> F128Bits }
fixed_to_float! { FixedI16(LeEqU16) -> F128Bits }
fixed_to_float! { FixedI32(LeEqU32) -> F128Bits }
fixed_to_float! { FixedI64(LeEqU64) -> F128Bits }
fixed_to_float! { FixedU8(LeEqU8) -> F128Bits }
fixed_to_float! { FixedU16(LeEqU16) -> F128Bits }
fixed_to_float! { FixedU32(LeEqU32) -> F128Bits }
fixed_to_float! { FixedU64(LeEqU64) -> F128Bits }

macro_rules! fixed_to_float_lossy {
    ($Fixed:ident($LeEqU:ident) -> $Float:ident) => {
        impl<Frac: $LeEqU> LossyFrom<$Fixed<Frac>> for $Float {
            /// Converts a fixed-point number to a floating-point number.
            ///
            /// This conversion never fails (infallible) but may lose
            /// precision (lossy). Rounding is to the nearest, with
            /// ties rounded to even.
            #[inline]
            fn lossy_from(src: $Fixed<Frac>) -> $Float {
                src.to_num()
            }
        }
    };
    ($Fixed:ident($LeEqU:ident)) => {
        fixed_to_float_lossy! { $Fixed($LeEqU) -> f16 }
        fixed_to_float_lossy! { $Fixed($LeEqU) -> bf16 }
        fixed_to_float_lossy! { $Fixed($LeEqU) -> f32 }
        fixed_to_float_lossy! { $Fixed($LeEqU) -> f64 }
        fixed_to_float_lossy! { $Fixed($LeEqU) -> F128Bits }
    };
}

fixed_to_float_lossy! { FixedI8(LeEqU8) }
fixed_to_float_lossy! { FixedI16(LeEqU16) }
fixed_to_float_lossy! { FixedI32(LeEqU32) }
fixed_to_float_lossy! { FixedI64(LeEqU64) }
fixed_to_float_lossy! { FixedI128(LeEqU128) }
fixed_to_float_lossy! { FixedU8(LeEqU8) }
fixed_to_float_lossy! { FixedU16(LeEqU16) }
fixed_to_float_lossy! { FixedU32(LeEqU32) }
fixed_to_float_lossy! { FixedU64(LeEqU64) }
fixed_to_float_lossy! { FixedU128(LeEqU128) }

macro_rules! int_to_float_lossy_lossless {
    ($Int:ident -> $($Lossy:ident)*; $($Lossless:ident)*) => {
        $(
            impl LossyFrom<$Int> for $Lossy {
                /// Converts an integer to a floating-point number.
                ///
                /// This conversion never fails (infallible) but may
                /// lose precision (lossy). Rounding is to the
                /// nearest, with ties rounded to even.
                #[inline]
                fn lossy_from(src: $Int) -> $Lossy {
                    Self::from_fixed(IntFixed(src).fixed())
                }
            }
        )*
        $(
            impl LosslessTryFrom<$Int> for $Lossless {
                /// Converts an integer to a floating-point number.
                ///
                /// This conversion actually never fails (infallible)
                /// and does not lose precision (lossless).
                #[inline]
                fn lossless_try_from(src: $Int) -> Option<$Lossless> {
                    Some(Self::from_fixed(IntFixed(src).fixed()))
                }
            }

            impl LossyFrom<$Int> for $Lossless {
                /// Converts an integer to a floating-point number.
                ///
                /// This conversion never fails (infallible) and
                /// actually does not lose precision (lossless).
                #[inline]
                fn lossy_from(src: $Int) -> $Lossless {
                    Self::from_fixed(IntFixed(src).fixed())
                }
            }
        )*
    };
}

int_to_float_lossy_lossless! { i8 -> bf16; f16 f32 f64 F128Bits }
int_to_float_lossy_lossless! { i16 -> bf16 f16; f32 f64 F128Bits }
int_to_float_lossy_lossless! { i32 -> bf16 f16 f32; f64 F128Bits }
int_to_float_lossy_lossless! { i64 -> bf16 f16 f32 f64; F128Bits }
int_to_float_lossy_lossless! { i128 -> bf16 f16 f32 f64 F128Bits; }
int_to_float_lossy_lossless! { isize -> bf16 f16 f32 f64 F128Bits; }

int_to_float_lossy_lossless! { u8 -> bf16; f16 f32 f64 F128Bits }
int_to_float_lossy_lossless! { u16 -> bf16 f16; f32 f64 F128Bits }
int_to_float_lossy_lossless! { u32 -> bf16 f16 f32; f64 F128Bits }
int_to_float_lossy_lossless! { u64 -> bf16 f16 f32 f64; F128Bits }
int_to_float_lossy_lossless! { u128 -> bf16 f16 f32 f64 F128Bits; }
int_to_float_lossy_lossless! { usize -> bf16 f16 f32 f64 F128Bits; }

macro_rules! into {
    ($Src:ty: $($Dst:ty),*) => { $(
        impl LosslessTryFrom<$Src> for $Dst {
            /// Converts a number.
            ///
            /// This conversion actually never fails (infallible) and
            /// does not lose any precision (lossless).
            #[inline]
            fn lossless_try_from(src: $Src) -> Option<Self> {
                Some(Self::from(src))
            }
        }

        impl LossyFrom<$Src> for $Dst {
            /// Converts a number.
            ///
            /// This conversion never fails (infallible) and actually
            /// does not lose any precision (lossless).
            #[inline]
            fn lossy_from(src: $Src) -> Self {
                Self::from(src)
            }
        }
    )* };
}

macro_rules! try_into {
    ($Src:ty: $($Dst:ty),*) => { $(
        impl LosslessTryFrom<$Src> for $Dst {
            /// Converts a number.
            ///
            /// This conversion may fail (fallible) but does not lose
            /// any precision (lossless).
            #[inline]
            fn lossless_try_from(src: $Src) -> Option<Self> {
                Self::try_from(src).ok()
            }
        }
    )* };
}

into! { bool: bool, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize }

into! { i8: i8, i16, i32, i64, i128, isize }
try_into! { i8: u8, u16, u32, u64, u128, usize }
into! { i16: i16, i32, i64, i128, isize }
try_into! { i16: i8, u8, u16, u32, u64, u128, usize }
into! { i32: i32, i64, i128 }
try_into! { i32: i8, i16, isize, u8, u16, u32, u64, u128, usize }
into! { i64: i64, i128 }
try_into! { i64: i8, i16, i32, isize, u8, u16, u32, u64, u128, usize }
into! { i128: i128 }
try_into! { i128: i8, i16, i32, i64, isize, u8, u16, u32, u64, u128, usize }
into! { isize: isize }
try_into! { isize: i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, usize }

into! { u8: i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize }
try_into! { u8: i8 }
into! { u16: i32, i64, i128, u16, u32, u64, u128, usize }
try_into! { u16: i8, i16, isize, u8 }
into! { u32: i64, i128, u32, u64, u128 }
try_into! { u32: i8, i16, i32, isize, u8, u16, usize }
into! { u64: i128, u64, u128 }
try_into! { u64: i8, i16, i32, i64, isize, u8, u16, u32, usize }
into! { u128: u128 }
try_into! { u128: i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, usize }
into! { usize: usize }
try_into! { usize: i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128 }

macro_rules! lossy {
    ($Src:ty: $Dst:ty; $src:ident -> $conv:expr) => {
        impl LossyFrom<$Src> for $Dst {
            #[inline]
            /// Converts a number.
            ///
            /// This conversion never fails (infallible) but may lose
            /// precision (lossy). Rounding is to the nearest, with
            /// ties rounded to even.
            fn lossy_from($src: $Src) -> $Dst {
                $conv
            }
        }
    };
}

into! { f16: f16 }
lossy! { f16: bf16; src -> bf16::from_f32(src.into()) }
into! { f16: f32, f64 }

lossy! { bf16: f16; src -> f16::from_f32(src.into()) }
into! { bf16: bf16, f32, f64 }

lossy! { f32: f16; src -> f16::from_f32(src) }
lossy! { f32: bf16; src -> bf16::from_f32(src) }
into! { f32: f32, f64 }

lossy! { f64: f16; src -> f16::from_f64(src) }
lossy! { f64: bf16; src -> bf16::from_f64(src) }
lossy! { f64: f32; src -> src as f32 }
into! { f64: f64 }

/// These are doc tests that should not appear in the docs, but are
/// useful as doc tests can check to ensure compilation failure.
///
/// The first snippet succeeds, and acts as a control.
///
/// ```rust
/// use fixed::{
///     traits::{LosslessTryFrom, LossyFrom},
///     types::*,
/// };
/// let _ = I8F8::from(I4F4::default());
/// let _ = I8F8::from(U7F1::default());
/// let _ = U8F8::from(U4F4::default());
/// let _ = I8F8::lossless_try_from(U8F8::default());
/// let _ = I8F8::lossless_try_from(I56F8::default());
/// let _ = U8F8::lossless_try_from(U56F8::default());
/// let _ = I8F8::lossy_from(I8F56::default());
/// let _ = I8F8::lossy_from(U7F57::default());
/// let _ = U8F8::lossy_from(U8F56::default());
/// let _ = usize::from(U16F0::default());
/// let _ = isize::from(I16F0::default());
/// let _ = isize::from(U8F0::default());
/// let _ = usize::lossless_try_from(I8F0::default());
/// let _ = usize::lossless_try_from(I16F0::default());
/// let _ = isize::lossless_try_from(U16F0::default());
/// ```
///
/// The rest of the tests should all fail compilation.
///
/// Not enough fractional bits.
/// ```rust,compile_fail
/// use fixed::types::*;
/// let _ = I8F8::from(I7F9::default());
/// ```
/// ```rust,compile_fail
/// use fixed::{traits::LosslessTryFrom, types::*};
/// let _ = I8F8::lossless_try_from(I7F9::default());
/// ```
///
/// Not enough integer bits.
/// ```rust,compile_fail
/// use fixed::types::*;
/// let _ = I8F8::from(I9F7::default());
/// ```
/// ```rust,compile_fail
/// use fixed::{traits::LossyFrom, types::*};
/// let _ = I8F8::lossy_from(I9F7::default());
/// ```
///
/// Not enough integer bits for unsigned to signed.
/// ```rust,compile_fail
/// use fixed::types::*;
/// let _ = I8F8::from(U8F0::default());
/// ```
/// ```rust,compile_fail
/// use fixed::{traits::LossyFrom, types::*};
/// let _ = I8F8::lossy_from(U8F0::default());
/// ```
///
/// Not  enough fractional bits.
/// ```rust,compile_fail
/// use fixed::types::*;
/// let _ = U8F8::from(U7F9::default());
/// ```
/// ```rust,compile_fail
/// use fixed::{traits::LosslessTryFrom, types::*};
/// let _ = U8F8::lossless_try_from(U7F9::default());
/// ```
///
/// Not enough integer bits.
/// ```rust,compile_fail
/// use fixed::types::*;
/// let _ = U8F8::from(U9F7::default());
/// ```
/// ```rust,compile_fail
/// use fixed::{traits::LossyFrom, types::*};
/// let _ = U8F8::lossy_from(U9F7::default());
/// ```
///
/// Signed to unsigned.
/// ```rust,compile_fail
/// use fixed::types::*;
/// let _ = U8F8::from(I4F4::default());
/// ```
/// ```rust,compile_fail
/// use fixed::{traits::LossyFrom, types::*};
/// let _ = U8F8::lossy_from(I4F4::default());
/// ```
///
/// Not enough fractional bits.
/// ```rust,compile_fail
/// use fixed::{traits::LosslessTryFrom, types::*};
/// let _ = I8F8::lossless_try_from(I55F9::default());
/// ```
///
/// Not enough integer bits.
/// ```rust,compile_fail
/// use fixed::{traits::LossyFrom, types::*};
/// let _ = I8F8::lossy_from(I9F55::default());
/// ```
/// ```rust,compile_fail
/// use fixed::{traits::LossyFrom, types::*};
/// let _ = U8F8::lossy_from(U9F55::default());
/// ```
///
/// Signed to unsigned.
/// ```rust,compile_fail
/// use fixed::{traits::LossyFrom, types::*};
/// let _ = I8F8::lossy_from(U8F56::default());
/// ```
///
/// Not enough fractional bits.
/// ```rust,compile_fail
/// use fixed::types::*;
/// let _ = usize::from(U16F16::default());
/// ```
/// ```rust,compile_fail
/// use fixed::types::*;
/// let _ = isize::from(I16F16::default());
/// ```
///
/// usize from i8 or i16.
/// ```rust,compile_fail
/// use fixed::types::*;
/// let _ = usize::from(I8F0::default());
/// ```
/// ```rust,compile_fail
/// use fixed::types::*;
/// let _ = usize::from(I16F0::default());
/// ```
///
/// isize form u16.
/// ```rust,compile_fail
/// use fixed::types::*;
/// let _ = isize::from(U16F0::default());
/// ```
fn _compile_fail_tests() {}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use crate::types::*;

    #[test]
    fn expanding_from_unsigned() {
        type L8 = U8F0;
        type Ll16 = U16F0;
        type Lh16 = U8F8;
        type Ll128 = U128F0;
        type Lh128 = U8F120;

        type H8 = U0F8;
        type Hl16 = U8F8;
        type Hh16 = U0F16;
        type Hl128 = U120F8;
        type Hh128 = U0F128;

        let vals: &[u8] = &[0x00, 0x7f, 0x80, 0xff];
        for &val in vals {
            let val16 = u16::from(val);
            let val128 = u128::from(val);

            let l = L8::from_bits(val);
            assert_eq!(l, L8::from(val));
            assert_eq!(val, u8::from(l));
            assert_eq!(Ll16::from(l), Ll16::from_bits(val16));
            assert_eq!(Lh16::from(l), Lh16::from_bits(val16 << 8));
            assert_eq!(Ll128::from(l), Ll128::from_bits(val128));
            assert_eq!(Lh128::from(l), Lh128::from_bits(val128 << 120));

            let h = H8::from_bits(val);
            assert_eq!(Hl16::from(h), Hl16::from_bits(val16));
            assert_eq!(Hh16::from(h), Hh16::from_bits(val16 << 8));
            assert_eq!(Hl128::from(h), Hl128::from_bits(val128));
            assert_eq!(Hh128::from(h), Hh128::from_bits(val128 << 120));
        }
    }

    #[test]
    fn expanding_from_signed() {
        type L8 = I8F0;
        type Ll16 = I16F0;
        type Lh16 = I8F8;
        type Ll128 = I128F0;
        type Lh128 = I8F120;

        type H8 = I0F8;
        type Hl16 = I8F8;
        type Hh16 = I0F16;
        type Hl128 = I120F8;
        type Hh128 = I0F128;

        let vals: &[i8] = &[0x00, 0x7f, -0x80, -0x01];
        for &val in vals {
            let val16 = i16::from(val);
            let val128 = i128::from(val);

            let l = L8::from_bits(val);
            assert_eq!(l, L8::from(val));
            assert_eq!(val, i8::from(l));
            assert_eq!(Ll16::from(l), Ll16::from_bits(val16));
            assert_eq!(Lh16::from(l), Lh16::from_bits(val16 << 8));
            assert_eq!(Ll128::from(l), Ll128::from_bits(val128));
            assert_eq!(Lh128::from(l), Lh128::from_bits(val128 << 120));

            let h = H8::from_bits(val);
            assert_eq!(Hl16::from(h), Hl16::from_bits(val16));
            assert_eq!(Hh16::from(h), Hh16::from_bits(val16 << 8));
            assert_eq!(Hl128::from(h), Hl128::from_bits(val128));
            assert_eq!(Hh128::from(h), Hh128::from_bits(val128 << 120));
        }
    }

    #[test]
    fn expanding_from_unsigned_to_signed() {
        type L8 = U8F0;
        type Ll16 = I16F0;
        type Lh16 = I9F7;
        type Ll128 = I128F0;
        type Lh128 = I9F119;

        type H8 = U0F8;
        type Hl16 = I8F8;
        type Hh16 = I1F15;
        type Hl128 = I120F8;
        type Hh128 = I1F127;

        let vals: &[u8] = &[0x00, 0x7f, 0x80, 0xff];
        for &val in vals {
            let val16 = i16::from(val);
            let val128 = i128::from(val);

            let l = L8::from_bits(val);
            assert_eq!(l, L8::from(val));
            assert_eq!(val, u8::from(l));
            assert_eq!(Ll16::from(l), Ll16::from_bits(val16));
            assert_eq!(Lh16::from(l), Lh16::from_bits(val16 << 7));
            assert_eq!(Ll128::from(l), Ll128::from_bits(val128));
            assert_eq!(Lh128::from(l), Lh128::from_bits(val128 << 119));

            let h = H8::from_bits(val);
            assert_eq!(Hl16::from(h), Hl16::from_bits(val16));
            assert_eq!(Hh16::from(h), Hh16::from_bits(val16 << 7));
            assert_eq!(Hl128::from(h), Hl128::from_bits(val128));
            assert_eq!(Hh128::from(h), Hh128::from_bits(val128 << 119));
        }
    }

    #[test]
    fn from_bool() {
        assert_eq!(I2F6::from(true), 1);
        assert_eq!(I2F6::from(false), 0);
        assert_eq!(I64F64::from(true), 1);
        assert_eq!(U1F127::from(true), 1);
    }

    #[test]
    fn to_size() {
        let min_i24 = I24F8::MIN;
        let max_i24 = I24F8::MAX;
        let max_u24 = U24F8::MAX;
        assert_eq!(min_i24.overflowing_to_num::<isize>(), (!0 << 23, false));
        assert_eq!(max_i24.overflowing_to_num::<isize>(), (!(!0 << 23), false));
        assert_eq!(max_u24.overflowing_to_num::<isize>(), (!(!0 << 24), false));
        assert_eq!(min_i24.overflowing_to_num::<usize>(), (!0 << 23, true));
        assert_eq!(max_i24.overflowing_to_num::<usize>(), (!(!0 << 23), false));
        assert_eq!(max_u24.overflowing_to_num::<usize>(), (!(!0 << 24), false));

        let min_i56 = I56F8::MIN;
        let max_i56 = I56F8::MAX;
        let max_u56 = U56F8::MAX;
        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(min_i56.overflowing_to_num::<isize>(), (0, true));
            assert_eq!(max_i56.overflowing_to_num::<isize>(), (!0, true));
            assert_eq!(max_u56.overflowing_to_num::<isize>(), (!0, true));
            assert_eq!(min_i56.overflowing_to_num::<usize>(), (0, true));
            assert_eq!(max_i56.overflowing_to_num::<usize>(), (!0, true));
            assert_eq!(max_u56.overflowing_to_num::<usize>(), (!0, true));
        }
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(min_i56.overflowing_to_num::<isize>(), (!0 << 55, false));
            assert_eq!(max_i56.overflowing_to_num::<isize>(), (!(!0 << 55), false));
            assert_eq!(max_u56.overflowing_to_num::<isize>(), (!(!0 << 56), false));
            assert_eq!(min_i56.overflowing_to_num::<usize>(), (!0 << 55, true));
            assert_eq!(max_i56.overflowing_to_num::<usize>(), (!(!0 << 55), false));
            assert_eq!(max_u56.overflowing_to_num::<usize>(), (!(!0 << 56), false));
        }

        let min_i120 = I120F8::MIN;
        let max_i120 = I120F8::MAX;
        let max_u120 = U120F8::MAX;
        assert_eq!(min_i120.overflowing_to_num::<isize>(), (0, true));
        assert_eq!(max_i120.overflowing_to_num::<isize>(), (!0, true));
        assert_eq!(max_u120.overflowing_to_num::<isize>(), (!0, true));
        assert_eq!(min_i120.overflowing_to_num::<usize>(), (0, true));
        assert_eq!(max_i120.overflowing_to_num::<usize>(), (!0, true));
        assert_eq!(max_u120.overflowing_to_num::<usize>(), (!0, true));
    }

    #[test]
    fn signed_from_float() {
        type Fix = I4F4;
        // 1.1 -> 0001.1000
        assert_eq!(Fix::from_num(3.0 / 2.0), Fix::from_bits(24));
        // 0.11 -> 0000.1100
        assert_eq!(Fix::from_num(3.0 / 4.0), Fix::from_bits(12));
        // 0.011 -> 0000.0110
        assert_eq!(Fix::from_num(3.0 / 8.0), Fix::from_bits(6));
        // 0.0011 -> 0000.0011
        assert_eq!(Fix::from_num(3.0 / 16.0), Fix::from_bits(3));
        // 0.00011 -> 0000.0010 (tie to even)
        assert_eq!(Fix::from_num(3.0 / 32.0), Fix::from_bits(2));
        // 0.00101 -> 0000.0010 (tie to even)
        assert_eq!(Fix::from_num(5.0 / 32.0), Fix::from_bits(2));
        // 0.000011 -> 0000.0001 (nearest)
        assert_eq!(Fix::from_num(3.0 / 64.0), Fix::DELTA);
        // 0.00001 -> 0000.0000 (tie to even)
        assert_eq!(Fix::from_num(1.0 / 32.0), Fix::ZERO);

        // -1.1 -> -0001.1000
        assert_eq!(Fix::from_num(-3.0 / 2.0), Fix::from_bits(-24));
        // -0.11 -> -0000.1100
        assert_eq!(Fix::from_num(-3.0 / 4.0), Fix::from_bits(-12));
        // -0.011 -> -0000.0110
        assert_eq!(Fix::from_num(-3.0 / 8.0), Fix::from_bits(-6));
        // -0.0011 -> -0000.0011
        assert_eq!(Fix::from_num(-3.0 / 16.0), Fix::from_bits(-3));
        // -0.00011 -> -0000.0010 (tie to even)
        assert_eq!(Fix::from_num(-3.0 / 32.0), Fix::from_bits(-2));
        // -0.00101 -> -0000.0010 (tie to even)
        assert_eq!(Fix::from_num(-5.0 / 32.0), Fix::from_bits(-2));
        // -0.000011 -> -0000.0001 (nearest)
        assert_eq!(Fix::from_num(-3.0 / 64.0), Fix::from_bits(-1));
        // -0.00001 -> 0000.0000 (tie to even)
        assert_eq!(Fix::from_num(-1.0 / 32.0), Fix::ZERO);

        // 111.1111 -> 111.1111
        assert_eq!(Fix::from_num(127.0 / 16.0), Fix::from_bits(127));
        // 111.11111 -> 1000.0000, too large (tie to even)
        assert_eq!(
            Fix::overflowing_from_num(255.0 / 32.0),
            (Fix::from_bits(-128), true)
        );

        // -111.1111 -> -111.1111
        assert_eq!(Fix::from_num(-127.0 / 16.0), Fix::from_bits(-127));
        // -111.11111 -> -1000.0000 (tie to even)
        assert_eq!(Fix::from_num(-255.0 / 32.0), Fix::from_bits(-128));
        // -1000.00001 -> -1000.0000 (tie to even)
        assert_eq!(Fix::from_num(-257.0 / 32.0), Fix::from_bits(-128));
        // -1000.0001 -> too small
        assert_eq!(
            Fix::overflowing_from_num(-129.0 / 16.0),
            (Fix::from_bits(127), true)
        );
    }

    #[test]
    fn unsigned_from_num() {
        type Fix = U4F4;
        // 1.1 -> 0001.1000
        assert_eq!(Fix::from_num(3.0 / 2.0), Fix::from_bits(24));
        // 0.11 -> 0000.1100
        assert_eq!(Fix::from_num(3.0 / 4.0), Fix::from_bits(12));
        // 0.011 -> 0000.0110
        assert_eq!(Fix::from_num(3.0 / 8.0), Fix::from_bits(6));
        // 0.0011 -> 0000.0011
        assert_eq!(Fix::from_num(3.0 / 16.0), Fix::from_bits(3));
        // 0.00011 -> 0000.0010 (tie to even)
        assert_eq!(Fix::from_num(3.0 / 32.0), Fix::from_bits(2));
        // 0.00101 -> 0000.0010 (tie to even)
        assert_eq!(Fix::from_num(5.0 / 32.0), Fix::from_bits(2));
        // 0.000011 -> 0000.0001 (nearest)
        assert_eq!(Fix::from_num(3.0 / 64.0), Fix::DELTA);
        // 0.00001 -> 0000.0000 (tie to even)
        assert_eq!(Fix::from_num(1.0 / 32.0), Fix::ZERO);
        // -0.00001 -> 0000.0000 (tie to even)
        assert_eq!(Fix::from_num(-1.0 / 32.0), Fix::ZERO);
        // -0.0001 -> too small
        assert_eq!(
            Fix::overflowing_from_num(-1.0 / 16.0),
            (Fix::from_bits(255), true)
        );

        // 1111.1111 -> 1111.1111
        assert_eq!(Fix::from_num(255.0 / 16.0), Fix::from_bits(255));
        // 1111.11111 -> too large (tie to even)
        assert_eq!(Fix::overflowing_from_num(511.0 / 32.0), (Fix::ZERO, true));
    }

    #[test]
    fn to_f16() {
        use half::f16;
        for u in 0x00..=0xff {
            let fu = U1F7::from_bits(u);
            assert_eq!(fu.to_num::<f16>(), f16::from_f32(f32::from(u) / 128.0));
            let i = u as i8;
            let fi = I1F7::from_bits(i);
            assert_eq!(fi.to_num::<f16>(), f16::from_f32(f32::from(i) / 128.0));

            for hi in &[
                0u32,
                0x0000_0100,
                0x7fff_ff00,
                0x8000_0000,
                0x8100_0000,
                0xffff_fe00,
                0xffff_ff00,
            ] {
                let uu = *hi | u32::from(u);
                let fuu = U25F7::from_bits(uu);
                assert_eq!(fuu.to_num::<f16>(), f16::from_f32(uu as f32 / 128.0));
                let ii = uu as i32;
                let fii = I25F7::from_bits(ii);
                assert_eq!(fii.to_num::<f16>(), f16::from_f32(ii as f32 / 128.0));
            }

            for hi in &[
                0u128,
                0x0000_0000_0000_0000_0000_0000_0000_0100,
                0x7fff_ffff_ffff_ffff_ffff_ffff_ffff_ff00,
                0x8000_0000_0000_0000_0000_0000_0000_0000,
                0x8100_0000_0000_0000_0000_0000_0000_0000,
                0xffff_ffff_ffff_ffff_ffff_ffff_ffff_fe00,
                0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ff00,
            ] {
                let uu = *hi | u128::from(u);
                let fuu = U121F7::from_bits(uu);
                assert_eq!(fuu.to_num::<f16>(), f16::from_f64(uu as f64 / 128.0));
                let ii = uu as i128;
                let fii = I121F7::from_bits(ii);
                assert_eq!(fii.to_num::<f16>(), f16::from_f64(ii as f64 / 128.0));
            }
        }
    }

    #[test]
    fn to_bf16() {
        use half::bf16;
        for u in 0x00..=0xff {
            let fu = U1F7::from_bits(u);
            assert_eq!(fu.to_num::<bf16>(), bf16::from_f32(f32::from(u) / 128.0));
            let i = u as i8;
            let fi = I1F7::from_bits(i);
            assert_eq!(fi.to_num::<bf16>(), bf16::from_f32(f32::from(i) / 128.0));

            for hi in &[
                0u32,
                0x0000_0100,
                0x7fff_ff00,
                0x8000_0000,
                0x8100_0000,
                0xffff_fe00,
                0xffff_ff00,
            ] {
                let uu = *hi | u32::from(u);
                let fuu = U25F7::from_bits(uu);
                assert_eq!(fuu.to_num::<bf16>(), bf16::from_f32(uu as f32 / 128.0));
                let ii = uu as i32;
                let fii = I25F7::from_bits(ii);
                assert_eq!(fii.to_num::<bf16>(), bf16::from_f32(ii as f32 / 128.0));
            }

            for hi in &[
                0u128,
                0x0000_0000_0000_0000_0000_0000_0000_0100,
                0x7fff_ffff_ffff_ffff_ffff_ffff_ffff_ff00,
                0x8000_0000_0000_0000_0000_0000_0000_0000,
                0x8100_0000_0000_0000_0000_0000_0000_0000,
                0xffff_ffff_ffff_ffff_ffff_ffff_ffff_fe00,
                0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ff00,
            ] {
                let uu = *hi | u128::from(u);
                let fuu = U121F7::from_bits(uu);
                assert_eq!(fuu.to_num::<bf16>(), bf16::from_f64(uu as f64 / 128.0));
                let ii = uu as i128;
                let fii = I121F7::from_bits(ii);
                assert_eq!(fii.to_num::<bf16>(), bf16::from_f64(ii as f64 / 128.0));
            }
        }
    }

    #[test]
    fn to_f32() {
        for u in 0x00..=0xff {
            let fu = U1F7::from_bits(u);
            assert_eq!(fu.to_num::<f32>(), f32::from(u) / 128.0);
            let i = u as i8;
            let fi = I1F7::from_bits(i);
            assert_eq!(fi.to_num::<f32>(), f32::from(i) / 128.0);

            for hi in &[
                0u32,
                0x0000_0100,
                0x7fff_ff00,
                0x8000_0000,
                0x8100_0000,
                0xffff_fe00,
                0xffff_ff00,
            ] {
                let uu = *hi | u32::from(u);
                let fuu = U25F7::from_bits(uu);
                assert_eq!(fuu.to_num::<f32>(), uu as f32 / 128.0);
                let ii = uu as i32;
                let fii = I25F7::from_bits(ii);
                assert_eq!(fii.to_num::<f32>(), ii as f32 / 128.0);
            }

            for hi in &[
                0u128,
                0x0000_0000_0000_0000_0000_0000_0000_0100,
                0x7fff_ffff_ffff_ffff_ffff_ffff_ffff_ff00,
                0x8000_0000_0000_0000_0000_0000_0000_0000,
                0x8100_0000_0000_0000_0000_0000_0000_0000,
                0xffff_ffff_ffff_ffff_ffff_ffff_ffff_fe00,
                0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ff00,
            ] {
                let uu = *hi | u128::from(u);
                let fuu = U121F7::from_bits(uu);
                assert_eq!(fuu.to_num::<f32>(), (uu as f64 / 128.0) as f32);
                let ii = uu as i128;
                let fii = I121F7::from_bits(ii);
                assert_eq!(fii.to_num::<f32>(), (ii as f64 / 128.0) as f32);
            }
        }
    }

    #[test]
    fn to_infinite_f32() {
        // too_large is 1.ffff_ffff_ffff... << 127,
        // which will be rounded to 1.0 << 128.
        let too_large = U128F0::MAX;
        assert_eq!(too_large.count_ones(), 128);
        assert!(too_large.to_num::<f32>().is_infinite());

        // still_too_large is 1.ffff_ff << 127,
        // which is exactly midway between 1.0 << 128 (even)
        // and the largest normal f32 that is 1.ffff_fe << 127 (odd).
        // The tie will be rounded to even, which is to 1.0 << 128.
        let still_too_large = too_large << 103u32;
        assert_eq!(still_too_large.count_ones(), 25);
        assert!(still_too_large.to_num::<f32>().is_infinite());

        // not_too_large is 1.ffff_feff_ffff... << 127,
        // which will be rounded to 1.ffff_fe << 127.
        let not_too_large = still_too_large - U128F0::DELTA;
        assert_eq!(not_too_large.count_ones(), 127);
        assert!(!not_too_large.to_num::<f32>().is_infinite());

        // min_128 is -1.0 << 127.
        let min_i128 = I128F0::MIN;
        assert_eq!(min_i128.count_ones(), 1);
        assert_eq!(min_i128.to_num::<f32>(), -(127f32.exp2()));
    }

    #[test]
    fn to_f64() {
        for u in 0x00..=0xff {
            let fu = U1F7::from_bits(u);
            assert_eq!(fu.to_num::<f64>(), f64::from(u) / 128.0);
            let i = u as i8;
            let fi = I1F7::from_bits(i);
            assert_eq!(fi.to_num::<f64>(), f64::from(i) / 128.0);

            for hi in &[
                0u64,
                0x0000_0000_0000_0100,
                0x7fff_ffff_ffff_ff00,
                0x8000_0000_0000_0000,
                0x8100_0000_0000_0000,
                0xffff_ffff_ffff_fe00,
                0xffff_ffff_ffff_ff00,
            ] {
                let uu = *hi | u64::from(u);
                let fuu = U57F7::from_bits(uu);
                assert_eq!(fuu.to_num::<f64>(), uu as f64 / 128.0);
                let ii = uu as i64;
                let fii = I57F7::from_bits(ii);
                assert_eq!(fii.to_num::<f64>(), ii as f64 / 128.0);
            }

            for hi in &[
                0u128,
                0x0000_0000_0000_0000_0000_0000_0000_0100,
                0x7fff_ffff_ffff_ffff_ffff_ffff_ffff_ff00,
                0x8000_0000_0000_0000_0000_0000_0000_0000,
                0x8100_0000_0000_0000_0000_0000_0000_0000,
                0xffff_ffff_ffff_ffff_ffff_ffff_ffff_fe00,
                0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ff00,
            ] {
                let uu = *hi | u128::from(u);
                let fuu = U121F7::from_bits(uu);
                assert_eq!(fuu.to_num::<f64>(), uu as f64 / 128.0);
                let ii = uu as i128;
                let fii = I121F7::from_bits(ii);
                assert_eq!(fii.to_num::<f64>(), ii as f64 / 128.0);
            }
        }
    }

    #[test]
    fn to_f128_bits() {
        use crate::F128Bits;
        // -01.3A -> sign 1, biased exp 3FFF, mantissa 3A00 << 96
        assert_eq!(
            I8F8::from_bits(-0x013A).to_num::<F128Bits>(),
            F128Bits(0xBFFF_3A00_u128 << 96)
        );
        // -1 -> sign 1, biased exp 3FFF, mantissa 0
        assert_eq!(
            I8F8::from_num(-1).to_num::<F128Bits>(),
            F128Bits(0xBFFF_u128 << 112)
        );
        // -0.5 -> sign 1, biased exp 3FFE, mantissa 0
        assert_eq!(
            I8F8::from_num(-0.5).to_num::<F128Bits>(),
            F128Bits(0xBFFE_u128 << 112)
        );
        // -1 >> 128 -> sign 1, biased exp 3F7F, mantissa 0
        assert_eq!(
            I0F128::from_bits(-1).to_num::<F128Bits>(),
            F128Bits(0xBF7F_u128 << 112)
        );
        // 0 -> sign 0, biased exp 0, mantissa 0
        assert_eq!(I8F8::ZERO.to_num::<F128Bits>(), F128Bits(0));
        // 1 >> 128 -> sign 0, biased exp 3F7F, mantissa 0
        assert_eq!(
            I0F128::DELTA.to_num::<F128Bits>(),
            F128Bits(0x3F7F_u128 << 112)
        );
        // 0.5 -> sign 0, biased exp 3FFE, mantissa 0
        assert_eq!(
            I8F8::from_num(0.5).to_num::<F128Bits>(),
            F128Bits(0x3FFE_u128 << 112)
        );
        // 1 -> sign 0, biased exp 3FFF, mantissa 0
        assert_eq!(I8F8::ONE.to_num::<F128Bits>(), F128Bits(0x3FFF_u128 << 112));
        // 01.3A -> sign 0, biased exp 3FFF, mantissa 3A00 << 96
        assert_eq!(
            I8F8::from_bits(0x013A).to_num::<F128Bits>(),
            F128Bits(0x3FFF_3A00_u128 << 96)
        );
    }

    #[test]
    fn lossy_f16() {
        use crate::traits::LossyFrom;
        use core::{f32, f64};
        use half::f16;

        assert_eq!(f16::lossy_from(f32::NEG_INFINITY), f16::NEG_INFINITY);
        assert!(f16::lossy_from(f32::NAN).is_nan());
        assert_eq!(f16::lossy_from(1e-37f32), f16::ZERO);
        // -1.625 << 15 is 1 11110 1010000000 is FA80
        assert_eq!(f16::lossy_from(-32768f32 * 1.625), f16::from_bits(0xFA80));
        assert_eq!(f16::lossy_from(32768f32 * 2.), f16::INFINITY);
        // 0x8020 is 0x1.004 << 15 is 0 11110 0000000001
        assert_eq!(
            f16::lossy_from(f32::from(0x8020u16)),
            f16::from_bits(0x7801)
        );
        // 0x8030 is rounded to 0x8040 (ties to even)
        assert_eq!(
            f16::lossy_from(f32::from(0x8030u16)),
            f16::from_bits(0x7802)
        );
        // 0x8050 is rounded to 0x8040 (ties to even)
        assert_eq!(
            f16::lossy_from(f32::from(0x8050u16)),
            f16::from_bits(0x7802)
        );
        // 1.0 >> 24 is minimum non-zero subnormal 0 0000 0000000001
        assert_eq!(f16::lossy_from((-24f32).exp2()), f16::from_bits(0x0001));
        assert_eq!(
            f16::lossy_from((-24f32).exp2() * 0.5001),
            f16::from_bits(0x0001)
        );
        assert_eq!(f16::lossy_from((-24f32).exp2() * 0.5), f16::ZERO);

        assert_eq!(f16::lossy_from(f64::NEG_INFINITY), f16::NEG_INFINITY);
        assert!(f16::lossy_from(f64::NAN).is_nan());
        assert_eq!(f16::lossy_from(1e-37f64), f16::ZERO);
        // -1.625 << 15 is 1 11110 1010000000 is FA80
        assert_eq!(f16::lossy_from(-32768f64 * 1.625), f16::from_bits(0xFA80));
        assert_eq!(f16::lossy_from(32768f64 * 2.), f16::INFINITY);
        // 0x8020 is 0x1.004 << 15 is 0 11110 0000000001
        assert_eq!(
            f16::lossy_from(f64::from(0x8020u16)),
            f16::from_bits(0x7801)
        );
        // 0x8030 is rounded to 0x8040 (ties to even)
        assert_eq!(
            f16::lossy_from(f64::from(0x8030u16)),
            f16::from_bits(0x7802)
        );
        // 0x8050 is rounded to 0x8040 (ties to even)
        assert_eq!(
            f16::lossy_from(f64::from(0x8050u16)),
            f16::from_bits(0x7802)
        );
        // 1.0 >> 24 is minimum non-zero subnormal 0 0000 0000000001
        assert_eq!(f16::lossy_from((-24f64).exp2()), f16::from_bits(0x0001));
        assert_eq!(
            f16::lossy_from((-24f64).exp2() * 0.5001),
            f16::from_bits(0x0001)
        );
        assert_eq!(f16::lossy_from((-24f32).exp2() * 0.5), f16::ZERO);
    }

    #[test]
    fn lossy_bf16() {
        use crate::traits::LossyFrom;
        use core::{f32, f64};
        use half::bf16;

        assert_eq!(bf16::lossy_from(f32::NEG_INFINITY), bf16::NEG_INFINITY);
        assert!(bf16::lossy_from(f32::NAN).is_nan());
        assert_eq!(bf16::lossy_from(f32::MIN_POSITIVE), bf16::MIN_POSITIVE);
        // -1.625 << 127 is 1 11111110 1010000 is FF50
        assert_eq!(
            bf16::lossy_from(127f32.exp2() * -1.625),
            bf16::from_bits(0xFF50)
        );
        // max is rounded up
        assert_eq!(bf16::lossy_from(f32::MAX), bf16::INFINITY);
        assert_eq!(
            bf16::lossy_from(f32::from_bits(0x4175_7FFF)),
            bf16::from_bits(0x4175)
        );
        assert_eq!(
            bf16::lossy_from(f32::from_bits(0x4175_8000)),
            bf16::from_bits(0x4176)
        );
        assert_eq!(
            bf16::lossy_from(f32::from_bits(0x4175_8001)),
            bf16::from_bits(0x4176)
        );
        assert_eq!(
            bf16::lossy_from(f32::from_bits(0x4176_7FFF)),
            bf16::from_bits(0x4176)
        );
        assert_eq!(
            bf16::lossy_from(f32::from_bits(0x4176_8000)),
            bf16::from_bits(0x4176)
        );
        assert_eq!(
            bf16::lossy_from(f32::from_bits(0x4176_8001)),
            bf16::from_bits(0x4177)
        );

        assert_eq!(bf16::lossy_from(f64::NEG_INFINITY), bf16::NEG_INFINITY);
        assert!(bf16::lossy_from(f64::NAN).is_nan());
        assert_eq!(bf16::lossy_from(1e-100f64), bf16::ZERO);
        // -1.625 << 127 is 1 11111110 1010000 is FF50
        assert_eq!(
            bf16::lossy_from(127f64.exp2() * -1.625),
            bf16::from_bits(0xFF50)
        );
        assert_eq!(bf16::lossy_from(128f64.exp2()), bf16::INFINITY);
        // 1.0 >> 133 is minimum non-zero subnormal 0 0000000 0000001
        assert_eq!(bf16::lossy_from((-133f64).exp2()), bf16::from_bits(0x0001));
        assert_eq!(
            bf16::lossy_from((-133f64).exp2() * 0.5001),
            bf16::from_bits(0x0001)
        );
        assert_eq!(bf16::lossy_from((-133f32).exp2() * 0.5), bf16::ZERO);
    }

    #[test]
    fn keep_nothing() {
        assert_eq!(I32F0::from_num(I0F32::MIN), -1);
        assert_eq!(I32F0::from_num(I0F32::MAX), 0);
        assert_eq!(I32F0::from_num(U0F32::MIN), 0);
        assert_eq!(I32F0::from_num(U0F32::MAX), 0);
        assert_eq!(U32F0::checked_from_num(I0F32::MIN), None);
        assert_eq!(U32F0::from_num(I0F32::MAX), 0);
        assert_eq!(U32F0::from_num(U0F32::MIN), 0);
        assert_eq!(U32F0::from_num(U0F32::MAX), 0);

        assert_eq!(I0F32::from_num(I32F0::ZERO), 0);
        assert_eq!(I0F32::from_num(U32F0::ZERO), 0);
        assert_eq!(U0F32::from_num(I32F0::ZERO), 0);
        assert_eq!(U0F32::from_num(U32F0::ZERO), 0);

        assert_eq!(I128F0::from_num(I0F128::MIN), -1);
        assert_eq!(I128F0::from_num(I0F128::MAX), 0);
        assert_eq!(I128F0::from_num(U0F128::MIN), 0);
        assert_eq!(I128F0::from_num(U0F128::MAX), 0);
        assert_eq!(U128F0::checked_from_num(I0F128::MIN), None);
        assert_eq!(U128F0::from_num(I0F128::MAX), 0);
        assert_eq!(U128F0::from_num(U0F128::MIN), 0);
        assert_eq!(U128F0::from_num(U0F128::MAX), 0);

        assert_eq!(I0F128::from_num(I128F0::ZERO), 0);
        assert_eq!(I0F128::from_num(U128F0::ZERO), 0);
        assert_eq!(U0F128::from_num(I128F0::ZERO), 0);
        assert_eq!(U0F128::from_num(U128F0::ZERO), 0);
    }
}
