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
    FixedI128, FixedI16, FixedI32, FixedI64, FixedI8, FixedU128, FixedU16, FixedU32, FixedU64,
    FixedU8, Unwrapped, Wrapping,
};
use bytemuck::{Pod, TransparentWrapper, Zeroable};

macro_rules! unsafe_impl_traits {
    ($Fixed:ident, $LeEqU:ident, $Inner:ident) => {
        unsafe impl<Frac> Zeroable for $Fixed<Frac> {}
        unsafe impl<Frac: 'static> Pod for $Fixed<Frac> {}
        unsafe impl<Frac> TransparentWrapper<$Inner> for $Fixed<Frac> {}

        unsafe impl<Frac: $LeEqU> Zeroable for Wrapping<$Fixed<Frac>> {}
        unsafe impl<Frac: $LeEqU> Pod for Wrapping<$Fixed<Frac>> {}
        unsafe impl<Frac: $LeEqU> TransparentWrapper<$Fixed<Frac>> for Wrapping<$Fixed<Frac>> {}

        unsafe impl<Frac: $LeEqU> Zeroable for Unwrapped<$Fixed<Frac>> {}
        unsafe impl<Frac: $LeEqU> Pod for Unwrapped<$Fixed<Frac>> {}
        unsafe impl<Frac: $LeEqU> TransparentWrapper<$Fixed<Frac>> for Unwrapped<$Fixed<Frac>> {}
    };
}

// SAFETY: all fixed-point numbers are repr(transparent) over primitive integer
// types which are both Pod and Zeroable, and Wrapping and Unwrapped are both
// repr(transparent) over fixed-point numbers.
unsafe_impl_traits! { FixedI8, LeEqU8, i8 }
unsafe_impl_traits! { FixedI16, LeEqU16, i16 }
unsafe_impl_traits! { FixedI32, LeEqU32, i32 }
unsafe_impl_traits! { FixedI64, LeEqU64, i64 }
unsafe_impl_traits! { FixedI128, LeEqU128, i128 }
unsafe_impl_traits! { FixedU8, LeEqU8, u8 }
unsafe_impl_traits! { FixedU16, LeEqU16, u16 }
unsafe_impl_traits! { FixedU32, LeEqU32, u32 }
unsafe_impl_traits! { FixedU64, LeEqU64, u64 }
unsafe_impl_traits! { FixedU128, LeEqU128, u128 }
