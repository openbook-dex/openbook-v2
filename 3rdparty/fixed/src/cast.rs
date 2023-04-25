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
    types::extra::{LeEqU128, LeEqU16, LeEqU32, LeEqU64, LeEqU8},
    F128Bits, FixedI128, FixedI16, FixedI32, FixedI64, FixedI8, FixedU128, FixedU16, FixedU32,
    FixedU64, FixedU8,
};
use az_crate::{Cast, CheckedCast, OverflowingCast, SaturatingCast, UnwrappedCast, WrappingCast};
use half::{bf16, f16};

macro_rules! cast {
    ($Src:ident($LeEqUSrc:ident); $Dst:ident($LeEqUDst:ident)) => {
        impl<FracSrc: $LeEqUSrc, FracDst: $LeEqUDst> Cast<$Dst<FracDst>> for $Src<FracSrc> {
            #[inline]
            fn cast(self) -> $Dst<FracDst> {
                self.to_num()
            }
        }

        impl<FracSrc: $LeEqUSrc, FracDst: $LeEqUDst> CheckedCast<$Dst<FracDst>> for $Src<FracSrc> {
            #[inline]
            fn checked_cast(self) -> Option<$Dst<FracDst>> {
                self.checked_to_num()
            }
        }

        impl<FracSrc: $LeEqUSrc, FracDst: $LeEqUDst> SaturatingCast<$Dst<FracDst>>
            for $Src<FracSrc>
        {
            #[inline]
            fn saturating_cast(self) -> $Dst<FracDst> {
                self.saturating_to_num()
            }
        }

        impl<FracSrc: $LeEqUSrc, FracDst: $LeEqUDst> WrappingCast<$Dst<FracDst>> for $Src<FracSrc> {
            #[inline]
            fn wrapping_cast(self) -> $Dst<FracDst> {
                self.wrapping_to_num()
            }
        }

        impl<FracSrc: $LeEqUSrc, FracDst: $LeEqUDst> OverflowingCast<$Dst<FracDst>>
            for $Src<FracSrc>
        {
            #[inline]
            fn overflowing_cast(self) -> ($Dst<FracDst>, bool) {
                self.overflowing_to_num()
            }
        }

        impl<FracSrc: $LeEqUSrc, FracDst: $LeEqUDst> UnwrappedCast<$Dst<FracDst>>
            for $Src<FracSrc>
        {
            #[inline]
            #[track_caller]
            fn unwrapped_cast(self) -> $Dst<FracDst> {
                self.unwrapped_to_num()
            }
        }
    };

    ($Fixed:ident($LeEqU:ident); $Dst:ident) => {
        impl<Frac: $LeEqU> Cast<$Dst> for $Fixed<Frac> {
            #[inline]
            fn cast(self) -> $Dst {
                self.to_num()
            }
        }

        impl<Frac: $LeEqU> CheckedCast<$Dst> for $Fixed<Frac> {
            #[inline]
            fn checked_cast(self) -> Option<$Dst> {
                self.checked_to_num()
            }
        }

        impl<Frac: $LeEqU> SaturatingCast<$Dst> for $Fixed<Frac> {
            #[inline]
            fn saturating_cast(self) -> $Dst {
                self.saturating_to_num()
            }
        }

        impl<Frac: $LeEqU> WrappingCast<$Dst> for $Fixed<Frac> {
            #[inline]
            fn wrapping_cast(self) -> $Dst {
                self.wrapping_to_num()
            }
        }

        impl<Frac: $LeEqU> OverflowingCast<$Dst> for $Fixed<Frac> {
            #[inline]
            fn overflowing_cast(self) -> ($Dst, bool) {
                self.overflowing_to_num()
            }
        }

        impl<Frac: $LeEqU> UnwrappedCast<$Dst> for $Fixed<Frac> {
            #[inline]
            #[track_caller]
            fn unwrapped_cast(self) -> $Dst {
                self.unwrapped_to_num()
            }
        }
    };

    ($Src:ident; $Fixed:ident($LeEqU:ident)) => {
        impl<Frac: $LeEqU> Cast<$Fixed<Frac>> for $Src {
            #[inline]
            fn cast(self) -> $Fixed<Frac> {
                <$Fixed<Frac>>::from_num(self)
            }
        }

        impl<Frac: $LeEqU> CheckedCast<$Fixed<Frac>> for $Src {
            #[inline]
            fn checked_cast(self) -> Option<$Fixed<Frac>> {
                <$Fixed<Frac>>::checked_from_num(self)
            }
        }

        impl<Frac: $LeEqU> SaturatingCast<$Fixed<Frac>> for $Src {
            #[inline]
            fn saturating_cast(self) -> $Fixed<Frac> {
                <$Fixed<Frac>>::saturating_from_num(self)
            }
        }

        impl<Frac: $LeEqU> WrappingCast<$Fixed<Frac>> for $Src {
            #[inline]
            fn wrapping_cast(self) -> $Fixed<Frac> {
                <$Fixed<Frac>>::wrapping_from_num(self)
            }
        }

        impl<Frac: $LeEqU> OverflowingCast<$Fixed<Frac>> for $Src {
            #[inline]
            fn overflowing_cast(self) -> ($Fixed<Frac>, bool) {
                <$Fixed<Frac>>::overflowing_from_num(self)
            }
        }

        impl<Frac: $LeEqU> UnwrappedCast<$Fixed<Frac>> for $Src {
            #[inline]
            #[track_caller]
            fn unwrapped_cast(self) -> $Fixed<Frac> {
                <$Fixed<Frac>>::unwrapped_from_num(self)
            }
        }
    };
}

macro_rules! cast_num {
    ($Src:ident($LeEqUSrc:ident); $($Dst:ident($LeEqUDst:ident),)*) => { $(
        cast! { $Src($LeEqUSrc); $Dst($LeEqUDst) }
    )* };
    ($Fixed:ident($LeEqU:ident); $($Num:ident,)*) => { $(
        cast! { $Fixed($LeEqU); $Num }
        cast! { $Num; $Fixed($LeEqU) }
    )* };
    ($($Fixed:ident($LeEqU:ident),)*) => { $(
        cast_num! {
            $Fixed($LeEqU);
            FixedI8(LeEqU8), FixedI16(LeEqU16), FixedI32(LeEqU32), FixedI64(LeEqU64),
            FixedI128(LeEqU128),
            FixedU8(LeEqU8), FixedU16(LeEqU16), FixedU32(LeEqU32), FixedU64(LeEqU64),
            FixedU128(LeEqU128),
        }
        cast! { bool; $Fixed($LeEqU) }
        cast_num! {
            $Fixed($LeEqU);
            i8, i16, i32, i64, i128, isize,
            u8, u16, u32, u64, u128, usize,
            f16, bf16, f32, f64, F128Bits,
        }
    )* };
}

cast_num! {
    FixedI8(LeEqU8), FixedI16(LeEqU16), FixedI32(LeEqU32), FixedI64(LeEqU64), FixedI128(LeEqU128),
    FixedU8(LeEqU8), FixedU16(LeEqU16), FixedU32(LeEqU32), FixedU64(LeEqU64), FixedU128(LeEqU128),
}
