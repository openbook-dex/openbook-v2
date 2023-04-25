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

/*!
Extra types that do not need to be handled directly.

These types are mostly reexports from the [*typenum* crate].

[*typenum* crate]: https://crates.io/crates/typenum
*/

pub use typenum::{
    Diff, IsLessOrEqual, Sum, True, Unsigned, U0, U1, U10, U100, U101, U102, U103, U104, U105,
    U106, U107, U108, U109, U11, U110, U111, U112, U113, U114, U115, U116, U117, U118, U119, U12,
    U120, U121, U122, U123, U124, U125, U126, U127, U128, U13, U14, U15, U16, U17, U18, U19, U2,
    U20, U21, U22, U23, U24, U25, U26, U27, U28, U29, U3, U30, U31, U32, U33, U34, U35, U36, U37,
    U38, U39, U4, U40, U41, U42, U43, U44, U45, U46, U47, U48, U49, U5, U50, U51, U52, U53, U54,
    U55, U56, U57, U58, U59, U6, U60, U61, U62, U63, U64, U65, U66, U67, U68, U69, U7, U70, U71,
    U72, U73, U74, U75, U76, U77, U78, U79, U8, U80, U81, U82, U83, U84, U85, U86, U87, U88, U89,
    U9, U90, U91, U92, U93, U94, U95, U96, U97, U98, U99,
};

/// Implemented for all [`Unsigned`] integers ≤ 8.
pub trait LeEqU8: Unsigned + IsLessOrEqual<U8, Output = True> {}
impl<T: Unsigned + IsLessOrEqual<U8, Output = True>> LeEqU8 for T {}
/// Implemented for all [`Unsigned`] integers ≤ 16.
pub trait LeEqU16: Unsigned + IsLessOrEqual<U16, Output = True> {}
impl<T: Unsigned + IsLessOrEqual<U16, Output = True>> LeEqU16 for T {}
/// Implemented for all [`Unsigned`] integers ≤ 32.
pub trait LeEqU32: Unsigned + IsLessOrEqual<U32, Output = True> {}
impl<T: Unsigned + IsLessOrEqual<U32, Output = True>> LeEqU32 for T {}
/// Implemented for all [`Unsigned`] integers ≤ 64.
pub trait LeEqU64: Unsigned + IsLessOrEqual<U64, Output = True> {}
impl<T: Unsigned + IsLessOrEqual<U64, Output = True>> LeEqU64 for T {}
/// Implemented for all [`Unsigned`] integers ≤ 128.
pub trait LeEqU128: Unsigned + IsLessOrEqual<U128, Output = True> {}
impl<T: Unsigned + IsLessOrEqual<U128, Output = True>> LeEqU128 for T {}
