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
use arbitrary::{Arbitrary, Result as ArbitraryResult, Unstructured};

macro_rules! impl_trait {
    ($Fixed:ident, $LeEqU:ident, $Inner:ident) => {
        impl<'a, Frac> Arbitrary<'a> for $Fixed<Frac> {
            #[inline]
            fn arbitrary(u: &mut Unstructured<'a>) -> ArbitraryResult<Self> {
                Ok(Self::from_bits(<$Inner as Arbitrary<'a>>::arbitrary(u)?))
            }

            #[inline]
            fn size_hint(depth: usize) -> (usize, Option<usize>) {
                <$Inner as Arbitrary<'a>>::size_hint(depth)
            }
        }

        impl<'a, Frac: $LeEqU> Arbitrary<'a> for Wrapping<$Fixed<Frac>> {
            #[inline]
            fn arbitrary(u: &mut Unstructured<'a>) -> ArbitraryResult<Self> {
                Ok(Self::from_bits(<$Inner as Arbitrary<'a>>::arbitrary(u)?))
            }

            #[inline]
            fn size_hint(depth: usize) -> (usize, Option<usize>) {
                <$Inner as Arbitrary<'a>>::size_hint(depth)
            }
        }

        impl<'a, Frac: $LeEqU> Arbitrary<'a> for Unwrapped<$Fixed<Frac>> {
            #[inline]
            fn arbitrary(u: &mut Unstructured<'a>) -> ArbitraryResult<Self> {
                Ok(Self::from_bits(<$Inner as Arbitrary<'a>>::arbitrary(u)?))
            }

            #[inline]
            fn size_hint(depth: usize) -> (usize, Option<usize>) {
                <$Inner as Arbitrary<'a>>::size_hint(depth)
            }
        }
    };
}

impl_trait! { FixedI8, LeEqU8, i8 }
impl_trait! { FixedI16, LeEqU16, i16 }
impl_trait! { FixedI32, LeEqU32, i32 }
impl_trait! { FixedI64, LeEqU64, i64 }
impl_trait! { FixedI128, LeEqU128, i128 }
impl_trait! { FixedU8, LeEqU8, u8 }
impl_trait! { FixedU16, LeEqU16, u16 }
impl_trait! { FixedU32, LeEqU32, u32 }
impl_trait! { FixedU64, LeEqU64, u64 }
impl_trait! { FixedU128, LeEqU128, u128 }
