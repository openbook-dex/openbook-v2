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
# Fixed-point numbers

The [*fixed* crate] provides fixed-point numbers.

  * [`FixedI8`] and [`FixedU8`] are eight-bit fixed-point numbers.
  * [`FixedI16`] and [`FixedU16`] are 16-bit fixed-point numbers.
  * [`FixedI32`] and [`FixedU32`] are 32-bit fixed-point numbers.
  * [`FixedI64`] and [`FixedU64`] are 64-bit fixed-point numbers.
  * [`FixedI128`] and [`FixedU128`] are 128-bit fixed-point numbers.

An <i>n</i>-bit fixed-point number has <i>f</i> = `Frac` fractional bits where
0 ≤ <i>f</i> ≤ <i>n</i>, and <i>n</i> − <i>f</i> integer bits. For example,
<code>[FixedI32]\<[U24]></code> is a 32-bit signed fixed-point number with
<i>n</i> = 32 total bits, <i>f</i> = 24 fractional bits, and
<i>n</i> − <i>f</i> = 8 integer bits. <code>[FixedI32]\<[U0]></code> behaves
like [`i32`], and <code>[FixedU32]\<[U0]></code> behaves like [`u32`].

The difference between any two successive representable numbers is constant
throughout the possible range for a fixed-point number:
<i>Δ</i> = 1/2<sup><i>f</i></sup>. When <i>f</i> = 0, like in
<code>[FixedI32]\<[U0]></code>, <i>Δ</i> = 1 because representable numbers are
integers, and the difference between two successive integers is 1. When
<i>f</i> = <i>n</i>, <i>Δ</i> = 1/2<sup><i>n</i></sup> and the value lies in the
range −0.5 ≤ <i>x</i> < 0.5 for signed numbers like
<code>[FixedI32]\<[U32]></code>, and in the range 0 ≤ <i>x</i> < 1 for unsigned
numbers like <code>[FixedU32]\<[U32]></code>.

In version 1 the [*typenum* crate] is used for the fractional bit count `Frac`;
the plan is to to have a major version 2 with [const generics] instead when the
Rust compiler support for them is powerful enough.

The main features are

  * Representation of binary fixed-point numbers up to 128 bits wide.
  * Conversions between fixed-point numbers and numeric primitives.
  * Comparisons between fixed-point numbers and numeric primitives.
  * Parsing from strings in decimal, binary, octal and hexadecimal.
  * Display as decimal, binary, octal and hexadecimal.
  * Arithmetic and logic operations.

This crate does *not* provide decimal fixed-point numbers. For example 0.001
cannot be represented exactly, as it is 1/10<sup>3</sup>. It is binary fractions
like 1/2<sup>4</sup> (0.0625) that can be represented exactly, provided there
are enough fractional bits.

This crate does *not* provide general analytic functions.

  * No algebraic functions are provided, for example no `sqrt` or `pow`.
  * No trigonometric functions are provided, for example no `sin` or `cos`.
  * No other transcendental functions are provided, for example no `log` or
    `exp`.

These functions are not provided because different implementations can have
different trade-offs, for example trading some correctness for speed.
Implementations can be provided in other crates.

  * The [*fixed-sqrt* crate] provides the square root operation.
  * The [*cordic* crate] provides various functions implemented using the
    [CORDIC] algorithm.

The conversions supported cover the following cases.

  * Infallible lossless conversions between fixed-point numbers and numeric
    primitives are provided using [`From`] and [`Into`]. These never fail
    (infallible) and do not lose any bits (lossless).
  * Infallible lossy conversions between fixed-point numbers and numeric
    primitives are provided using the [`LossyFrom`] and [`LossyInto`] traits.
    The source can have more fractional bits than the destination.
  * Checked lossless conversions between fixed-point numbers and numeric
    primitives are provided using the [`LosslessTryFrom`] and
    [`LosslessTryInto`] traits. The source cannot have more fractional bits than
    the destination.
  * Checked conversions between fixed-point numbers and numeric primitives are
    provided using the [`FromFixed`] and [`ToFixed`] traits, or using the
    [`from_num`] and [`to_num`] methods and [their checked
    versions][`checked_from_num`].
  * Additionally, [`az`] casts are implemented for conversion between
    fixed-point nubmers and numeric primitives.
  * Fixed-point numbers can be parsed from decimal strings using [`FromStr`],
    and from binary, octal and hexadecimal strings using the
    [`from_str_binary`], [`from_str_octal`] and [`from_str_hex`] methods. The
    result is rounded to the nearest, with ties rounded to even.
  * Fixed-point numbers can be converted to strings using [`Display`],
    [`Binary`], [`Octal`], [`LowerHex`] and [`UpperHex`]. The output is rounded
    to the nearest, with ties rounded to even.
  * All fixed-point numbers are plain old data, so [`bytemuck`] bit casting
    conversions can be used.

## Quick examples

```rust
use fixed::types::I20F12;

// 19/3 = 6 1/3
let six_and_third = I20F12::from_num(19) / 3;
// four decimal digits for 12 binary digits
assert_eq!(six_and_third.to_string(), "6.3333");
// find the ceil and convert to i32
assert_eq!(six_and_third.ceil().to_num::<i32>(), 7);
// we can also compare directly to integers
assert_eq!(six_and_third.ceil(), 7);
```

The type [`I20F12`] is a 32-bit fixed-point signed number with 20 integer bits
and 12 fractional bits. It is an alias to <code>[FixedI32]\<[U12]></code>. The
unsigned counterpart would be [`U20F12`]. Aliases are provided for all
combinations of integer and fractional bits adding up to a total of eight, 16,
32, 64 or 128 bits.

```rust
use fixed::types::{I4F4, I4F12};

// −8 ≤ I4F4 < 8 with steps of 1/16 (~0.06)
let a = I4F4::from_num(1);
// multiplication and division by integers are possible
let ans1 = a / 5 * 17;
// 1 / 5 × 17 = 3 2/5 (3.4), but we get 3 3/16 (~3.2)
assert_eq!(ans1, I4F4::from_bits((3 << 4) + 3));
assert_eq!(ans1.to_string(), "3.2");

// −8 ≤ I4F12 < 8 with steps of 1/4096 (~0.0002)
let wider_a = I4F12::from(a);
let wider_ans = wider_a / 5 * 17;
let ans2 = I4F4::from_num(wider_ans);
// now the answer is the much closer 3 6/16 (~3.4)
assert_eq!(ans2, I4F4::from_bits((3 << 4) + 6));
assert_eq!(ans2.to_string(), "3.4");
```

The second example shows some precision and conversion issues. The low precision
of `a` means that `a / 5` is 3⁄16 instead of 1⁄5, leading to an inaccurate
result `ans1` = 3 3⁄16 (~3.2). With a higher precision, we get `wider_a / 5`
equal to 819⁄4096, leading to a more accurate intermediate result `wider_ans` =
3 1635⁄4096. When we convert back to four fractional bits, we get `ans2` = 3
6⁄16 (~3.4).

Note that we can convert from [`I4F4`] to [`I4F12`] using [`From`], as the
target type has the same number of integer bits and a larger number of
fractional bits. Converting from [`I4F12`] to [`I4F4`] cannot use [`From`] as we
have less fractional bits, so we use [`from_num`] instead.

## Writing fixed-point constants and values literally

The [*fixed-macro* crate] provides a convenient macro to write down fixed-point
constants literally in the code.

```rust
# #[cfg(feature = "skip-this-test")] {
use fixed::types::I16F16;
use fixed_macro::fixed;

const NUM1: I16F16 = fixed!(12.75: I16F16);
let num2 = NUM1 + fixed!(13.125: I16F16);
assert_eq!(num2, 25.875);
# }
```

## Using the *fixed* crate

The *fixed* crate is available on [crates.io][*fixed* crate]. To use it in your
crate, add it as a dependency inside [*Cargo.toml*]:

```toml
[dependencies]
fixed = "1.11"
```

The *fixed* crate requires rustc version 1.53.0 or later.

## Optional features

The *fixed* crate has these optional feature:

 1. `arbitrary`, disabled by default. This provides the generation of arbitrary
    fixed-point numbers from raw, unstructured data. This feature requires the
    [*arbitrary* crate].
 2. `serde`, disabled by default. This provides serialization support for the
    fixed-point types. This feature requires the [*serde* crate].
 3. `std`, disabled by default. This is for features that are not possible under
    `no_std`: currently the implementation of the [`Error`] trait for
    [`ParseFixedError`].
 4. `serde-str`, disabled by default. Fixed-point numbers are serialized as
    strings showing the value when using human-readable formats. This feature
    requires the `serde` and the `std` optional features. **Warning:** numbers
    serialized when this feature is enabled cannot be deserialized when this
    feature is disabled, and vice versa.

To enable features, you can add the dependency like this to [*Cargo.toml*]:

```toml
[dependencies.fixed]
version = "1.11"
features = ["serde"]
```

## Experimental optional features

It is not considered a breaking change if the following experimental features
are removed. The removal of experimental features would however require a minor
version bump. Similarly, on a minor version bump, optional dependencies can be
updated to an incompatible newer version.

 1. `borsh`, disabled by default. This implements serialization and
    deserialization using the [*borsh* crate]. (The plan is to promote this to
    an optional feature once the [*borsh* crate] reaches version 1.0.0.)
 2. `num-traits`, disabled by default. This implements some traits from the
    [*num-traits* crate]. (The plan is to promote this to an optional feature
    once the [*num-traits* crate] reaches version 1.0.0.)

## Deprecated optional features

The following optional features are deprecated and may be removed in the next
major version of the crate.

 1. `az`, has no effect. Previously required for the [`az`] cast traits. Now
    these cast traits are always provided.
 2. `f16`, has no effect. Previously required for conversion to/from [`f16`] and
    [`bf16`]. Now these conversions are always provided.

## License

This crate is free software: you can redistribute it and/or modify it under the
terms of either

  * the [Apache License, Version 2.0][LICENSE-APACHE] or
  * the [MIT License][LICENSE-MIT]

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache License, Version 2.0,
shall be dual licensed as above, without any additional terms or conditions.

[*Cargo.toml*]: https://doc.rust-lang.org/cargo/guide/dependencies.html
[*arbitrary* crate]: https://crates.io/crates/arbitrary
[*borsh* crate]: https://crates.io/crates/borsh
[*cordic* crate]: https://crates.io/crates/cordic
[*fixed* crate]: https://crates.io/crates/fixed
[*fixed-macro* crate]: https://crates.io/crates/fixed-macro
[*fixed-sqrt* crate]: https://crates.io/crates/fixed-sqrt
[*half* crate]: https://crates.io/crates/half
[*num-traits* crate]: https://crates.io/crates/num-traits
[*serde* crate]: https://crates.io/crates/serde
[*typenum* crate]: https://crates.io/crates/typenum
[CORDIC]: https://en.wikipedia.org/wiki/CORDIC
[LICENSE-APACHE]: https://www.apache.org/licenses/LICENSE-2.0
[LICENSE-MIT]: https://opensource.org/licenses/MIT
[U0]: crate::types::extra::U0
[U24]: crate::types::extra::U24
[`Binary`]: core::fmt::Binary
[`Display`]: core::fmt::Display
[`Error`]: std::error::Error
[`FromStr`]: core::str::FromStr
[`I20F12`]: crate::types::I20F12
[`I4F12`]: crate::types::I4F12
[`I4F4`]: crate::types::I4F4
[`LosslessTryFrom`]: traits::LosslessTryFrom
[`LosslessTryInto`]: traits::LosslessTryInto
[`LossyFrom`]: traits::LossyFrom
[`LossyInto`]: traits::LossyInto
[`LowerHex`]: core::fmt::LowerHex
[`Octal`]: core::fmt::Octal
[`U20F12`]: types::U20F12
[`UpperHex`]: core::fmt::UpperHex
[`az`]: az_crate
[`bf16`]: half::bf16
[`checked_from_num`]: FixedI32::checked_from_num
[`f16`]: half::f16
[`from_num`]: FixedI32::from_num
[`from_str_binary`]: FixedI32::from_str_binary
[`from_str_hex`]: FixedI32::from_str_hex
[`from_str_octal`]: FixedI32::from_str_octal
[`to_num`]: FixedI32::to_num
[const generics]: https://github.com/rust-lang/rust/issues/44580
*/
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]
#![warn(unsafe_op_in_unsafe_fn)]
#![doc(html_root_url = "https://docs.rs/fixed/~1.11")]
#![doc(test(attr(deny(warnings))))]
#![cfg_attr(feature = "fail-on-warnings", deny(warnings))]
#![allow(clippy::wrong_self_convention)]

#[cfg(all(not(feature = "std"), test))]
extern crate std;

#[macro_use]
mod macros;

mod arith;
#[cfg(feature = "borsh")]
mod borshize;
mod cast;
mod cmp;
pub mod consts;
mod convert;
mod debug_hex;
mod display;
mod float_helper;
mod from_str;
mod helpers;
#[cfg(feature = "arbitrary")]
mod impl_arbitrary;
mod impl_bytemuck;
#[cfg(feature = "num-traits")]
mod impl_num_traits;
mod int256;
mod int_helper;
mod inv_lerp;
mod lerp;
mod log10;
mod prim_traits;
#[cfg(feature = "serde")]
mod serdeize;
pub mod traits;
pub mod types;
mod unwrapped;
mod wrapping;

#[cfg(feature = "num-traits")]
pub use crate::impl_num_traits::RadixParseFixedError;
use crate::{
    from_str::FromStrRadix,
    log10::IntFracLog10,
    traits::{FromFixed, ToFixed},
    types::extra::{
        IsLessOrEqual, LeEqU128, LeEqU16, LeEqU32, LeEqU64, LeEqU8, Sum, True, Unsigned, U12, U124,
        U125, U126, U127, U128, U13, U14, U15, U16, U28, U29, U30, U31, U32, U4, U5, U6, U60, U61,
        U62, U63, U64, U7, U8,
    },
};
pub use crate::{from_str::ParseFixedError, unwrapped::Unwrapped, wrapping::Wrapping};
use core::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    marker::PhantomData,
    mem,
    ops::Add,
};

/// A prelude to import useful traits.
///
/// This prelude is similar to the [standard library’s prelude][std::prelude] in
/// that you’ll almost always want to import its entire contents, but unlike the
/// standard library’s prelude you’ll have to do so manually:
///
/// ```rust
/// # #[allow(unused_imports)]
/// use fixed::prelude::*;
/// ```
///
/// The prelude may grow over time as additional items see ubiquitous use.
///
/// # Contents
///
/// The prelude re-exports the following:
///
///  * <code>[traits]::{[FromFixed], [ToFixed]}</code>, checked conversions
///     from/to fixed-point numbers.
///  * <code>[traits]::{[LossyFrom], [LossyInto]}</code>, infallible lossy
///    conversions.
///  * <code>[traits]::{[LosslessTryFrom], [LosslessTryInto]}</code>, checked
///    lossless conversions.
///
/// [LosslessTryFrom]: crate::traits::LosslessTryFrom
/// [LosslessTryInto]: crate::traits::LosslessTryInto
/// [LossyFrom]: crate::traits::LossyFrom
/// [LossyInto]: crate::traits::LossyInto
pub mod prelude {
    pub use crate::traits::{
        FromFixed, LosslessTryFrom, LosslessTryInto, LossyFrom, LossyInto, ToFixed,
    };
}

#[macro_use]
mod macros_from_to;
#[macro_use]
mod macros_round;
#[macro_use]
mod macros_no_frac;
#[macro_use]
mod macros_frac;
#[macro_use]
mod macros_const;

macro_rules! fixed {
    (
        $description:expr,
        $Fixed:ident(
            $Inner:ident, $LeEqU:tt, $s_nbits:expr,
            $s_nbits_m1:expr, $s_nbits_m2:expr, $s_nbits_m3:expr, $s_nbits_m4:expr
        ),
        $nbytes:expr, $bytes_val:expr, $rev_bytes_val:expr, $be_bytes:expr, $le_bytes:expr,
        $UFixed:ident, $UInner:ty, $Signedness:tt,
        $LeEqU_C0:tt, $LeEqU_C1:tt, $LeEqU_C2:tt, $LeEqU_C3:tt,
        $Double:ident, $DoubleInner:ty, $s_nbits_2:expr, $HasDouble:tt
    ) => {
        fixed! {
            $description,
            $Fixed[stringify!($Fixed)](
                $Inner[stringify!($Inner)], $LeEqU, $s_nbits,
                $s_nbits_m1, $s_nbits_m2, $s_nbits_m3, $s_nbits_m4
            ),
            $nbytes, $bytes_val, $rev_bytes_val, $be_bytes, $le_bytes,
            $UFixed[stringify!($UFixed)], $UInner, $Signedness,
            $LeEqU_C0, $LeEqU_C1, $LeEqU_C2, $LeEqU_C3,
            $Double, $DoubleInner, $s_nbits_2, $HasDouble
        }
    };
    (
        $description:expr,
        $Fixed:ident[$s_fixed:expr](
            $Inner:ident[$s_inner:expr], $LeEqU:tt, $s_nbits:expr,
            $s_nbits_m1:expr, $s_nbits_m2:expr, $s_nbits_m3:expr, $s_nbits_m4:expr
        ),
        $nbytes:expr, $bytes_val:expr, $rev_bytes_val:expr, $be_bytes:expr, $le_bytes:expr,
        $UFixed:ident[$s_ufixed:expr], $UInner:ty, $Signedness:tt,
        $LeEqU_C0:tt, $LeEqU_C1:tt, $LeEqU_C2:tt, $LeEqU_C3:tt,
        $Double:ident, $DoubleInner:ty, $s_nbits_2:expr, $HasDouble:tt
    ) => {
        comment! {
            $description, "-bit ",
            if_signed_unsigned!($Signedness, "signed", "unsigned"),
            " number with `Frac` fractional bits.

The number has ", $s_nbits, " bits, of which <i>f</i> = `Frac` are fractional
bits and ", $s_nbits, " − <i>f</i> are integer bits. The value <i>x</i> can lie
in the range ",
            if_signed_unsigned!(
                $Signedness,
                concat!("−2<sup>", $s_nbits_m1, "</sup>/2<sup><i>f</i></sup>"),
                "0",
            ),
            " ≤ <i>x</i> < 2<sup>",
            if_signed_unsigned!($Signedness, $s_nbits_m1, $s_nbits),
            "</sup>/2<sup><i>f</i></sup>. The difference between successive
numbers is constant throughout the range: <i>Δ</i> = 1/2<sup><i>f</i></sup>.

For <code>", $s_fixed, "\\<[U0]></code>, <i>f</i> = 0 and <i>Δ</i> = 1, and the
fixed-point number behaves like ",
            if_signed_unsigned!($Signedness, "an", "a"),
            " [`", $s_inner, "`] with the value lying in the range ",
            if_signed_unsigned!(
                $Signedness,
                concat!("−2<sup>", $s_nbits_m1, "</sup>"),
                "0",
            ),
            " ≤ <i>x</i> < 2<sup>",
            if_signed_unsigned!($Signedness, $s_nbits_m1, $s_nbits),
            "</sup>. For <code>", $s_fixed, "\\<[U", $s_nbits, "]></code>,
<i>f</i> = ", $s_nbits, " and <i>Δ</i> = 1/2<sup>", $s_nbits, "</sup>, and the
value lies in the range ",
            if_signed_unsigned!($Signedness, "−1/2 ≤ <i>x</i> < 1/2", "0 ≤ <i>x</i> < 1"),
            ".

`Frac` is an [`Unsigned`] as provided by the [*typenum* crate]; the plan is to
to have a major version 2 with [const generics] instead when the Rust compiler
support for them is powerful enough.

`", $s_fixed, "<Frac>` has the same size, alignment and ABI as [`", $s_inner, "`];
it is `#[repr(transparent)]` with [`", $s_inner, "`] as the only non-zero-sized field.

# Examples

```rust
use fixed::{types::extra::U3, ", $s_fixed, "};
let eleven = ", $s_fixed, "::<U3>::from_num(11);
assert_eq!(eleven, ", $s_fixed, "::<U3>::from_bits(11 << 3));
assert_eq!(eleven, 11);
assert_eq!(eleven.to_string(), \"11\");
let two_point_75 = eleven / 4;
assert_eq!(two_point_75, ", $s_fixed, "::<U3>::from_bits(11 << 1));
assert_eq!(two_point_75, 2.75);
assert_eq!(two_point_75.to_string(), \"2.8\");
```

[*typenum* crate]: https://crates.io/crates/typenum
[U0]: crate::types::extra::U0
[U", $s_nbits, "]: crate::types::extra::U", $s_nbits, "
[const generics]: https://github.com/rust-lang/rust/issues/44580
";
            #[repr(transparent)]
            pub struct $Fixed<Frac> {
                pub(crate) bits: $Inner,
                phantom: PhantomData<Frac>,
            }
        }

        impl<Frac> Clone for $Fixed<Frac> {
            #[inline]
            fn clone(&self) -> $Fixed<Frac> {
                $Fixed {
                    bits: self.bits,
                    phantom: PhantomData,
                }
            }
        }

        impl<Frac> Copy for $Fixed<Frac> {}

        impl<Frac> Default for $Fixed<Frac> {
            #[inline]
            fn default() -> Self {
                $Fixed {
                    bits: Default::default(),
                    phantom: PhantomData,
                }
            }
        }

        impl<Frac> Hash for $Fixed<Frac> {
            #[inline]
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.bits.hash(state);
            }
        }

        // inherent methods that do not require Frac bounds, some of which can thus be const
        fixed_no_frac! {
            $Fixed[$s_fixed]($Inner[$s_inner], $LeEqU, $s_nbits, $s_nbits_m1),
            $nbytes, $bytes_val, $rev_bytes_val, $be_bytes, $le_bytes,
            $UFixed[$s_ufixed], $UInner, $Signedness,
            $Double, $DoubleInner, $s_nbits_2, $HasDouble
        }
        // inherent methods that require Frac bounds, and cannot be const
        fixed_frac! {
            $Fixed[$s_fixed]($Inner[$s_inner], $LeEqU, $s_nbits, $s_nbits_m1, $s_nbits_m4),
            $UFixed, $UInner, $Signedness
        }
        fixed_const! {
            $Fixed[$s_fixed]($LeEqU, $s_nbits, $s_nbits_m1, $s_nbits_m2, $s_nbits_m3, $s_nbits_m4),
            $LeEqU_C0, $LeEqU_C1, $LeEqU_C2, $LeEqU_C3,
            $Signedness
        }
    };
}

fixed! {
    "An eight",
    FixedU8(u8, LeEqU8, "8", "7", "6", "5", "4"),
    1, "0x12", "0x12", "[0x12]", "[0x12]",
    FixedU8, u8, Unsigned,
    U8, U7, U6, U5,
    FixedU16, u16, "16", True
}
fixed! {
    "A 16",
    FixedU16(u16, LeEqU16, "16", "15", "14", "13", "12"),
    2, "0x1234", "0x3412", "[0x12, 0x34]", "[0x34, 0x12]",
    FixedU16, u16, Unsigned,
    U16, U15, U14, U13,
    FixedU32, u32, "32", True
}
fixed! {
    "A 32",
    FixedU32(u32, LeEqU32, "32", "31", "30", "29", "28"),
    4, "0x1234_5678", "0x7856_3412", "[0x12, 0x34, 0x56, 0x78]", "[0x78, 0x56, 0x34, 0x12]",
    FixedU32, u32, Unsigned,
    U32, U31, U30, U29,
    FixedU64, u64, "64", True
}
fixed! {
    "A 64",
    FixedU64(u64, LeEqU64, "64", "63", "62", "61", "60"),
    8, "0x1234_5678_9ABC_DE0F", "0x0FDE_BC9A_7856_3412",
    "[0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0x0F]",
    "[0x0F, 0xDE, 0xBC, 0x9A, 0x78, 0x56, 0x34, 0x12]",
    FixedU64, u64, Unsigned,
    U64, U63, U62, U61,
    FixedU128, u128, "128", True
}
fixed! {
    "A 128",
    FixedU128(u128, LeEqU128, "128", "127", "126", "125", "124"),
    16, "0x1234_5678_9ABC_DEF0_0102_0304_0506_0708",
    "0x0807_0605_0403_0201_F0DE_BC9A_7856_3412",
    "[0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, \
     0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]",
    "[0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01, \
     0xF0, 0xDE, 0xBC, 0x9A, 0x78, 0x56, 0x34, 0x12]",
    FixedU128, u128, Unsigned,
    U128, U127, U126, U125,
    FixedU128, u128, "128", False
}
fixed! {
    "An eight",
    FixedI8(i8, LeEqU8, "8", "7", "6", "5", "4"),
    1, "0x12", "0x12", "[0x12]", "[0x12]",
    FixedU8, u8, Signed,
    U7, U6, U5, U4,
    FixedI16, i16, "16", True
}
fixed! {
    "A 16",
    FixedI16(i16, LeEqU16, "16", "15", "14", "13", "12"),
    2, "0x1234", "0x3412", "[0x12, 0x34]", "[0x34, 0x12]",
    FixedU16, u16, Signed,
    U15, U14, U13, U12,
    FixedI32, i32, "32", True
}
fixed! {
    "A 32",
    FixedI32(i32, LeEqU32, "32", "31", "30", "29", "28"),
    4, "0x1234_5678", "0x7856_3412", "[0x12, 0x34, 0x56, 0x78]", "[0x78, 0x56, 0x34, 0x12]",
    FixedU32, u32, Signed,
    U31, U30, U29, U28,
    FixedI64, i64, "64", True
}
fixed! {
    "A 64",
    FixedI64(i64, LeEqU64, "64", "63", "62", "61", "60"),
    8, "0x1234_5678_9ABC_DE0F", "0x0FDE_BC9A_7856_3412",
    "[0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0x0F]",
    "[0x0F, 0xDE, 0xBC, 0x9A, 0x78, 0x56, 0x34, 0x12]",
    FixedU64, u64, Signed,
    U63, U62, U61, U60,
    FixedI128, i128, "128", True
}
fixed! {
    "A 128",
    FixedI128(i128, LeEqU128, "128", "127", "126", "125", "124"),
    16, "0x1234_5678_9ABC_DEF0_0102_0304_0506_0708",
    "0x0807_0605_0403_0201_F0DE_BC9A_7856_3412",
    "[0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, \
     0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]",
    "[0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01, \
     0xF0, 0xDE, 0xBC, 0x9A, 0x78, 0x56, 0x34, 0x12]",
    FixedU128, u128, Signed,
    U127, U126, U125, U124,
    FixedI128, i128, "128", False
}

/// The bit representation of a *binary128* floating-point number (`f128`).
///
/// This type can be used to
///
///   * convert between fixed-point numbers and the bit representation of
///     128-bit floating-point numbers.
///   * compare fixed-point numbers and the bit representation of 128-bit
///     floating-point numbers.
///
/// # Examples
///
/// ```rust
/// use fixed::{types::I16F16, F128Bits};
/// // binary128 representation for 1.0 is 0x3FFF << 112
/// let one = F128Bits(0x3FFF_u128 << 112);
///
/// assert_eq!(I16F16::ONE.to_num::<F128Bits>(), one);
/// assert_eq!(I16F16::from_num(one), I16F16::ONE);
///
/// // fixed-point numbers can be compared directly to F128Bits values
/// assert!(I16F16::from_num(1.5) > one);
/// assert!(I16F16::from_num(0.5) < one);
/// ```
#[repr(transparent)]
#[derive(Clone, Copy, Default, Hash, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct F128Bits(pub u128);

impl F128Bits {
    #[inline]
    pub(crate) fn to_bits(self) -> u128 {
        self.0
    }

    #[inline]
    pub(crate) fn from_bits(bits: u128) -> F128Bits {
        F128Bits(bits)
    }

    #[inline]
    pub(crate) fn is_nan(self) -> bool {
        (self.to_bits() & !float_helper::F128Bits::SIGN_MASK) > float_helper::F128Bits::EXP_MASK
    }
}

/// Defines constant fixed-point numbers from integer expressions.
///
/// This macro is useful because [`from_num`] cannot be used in constant
/// expressions.
///
/// # Alternative
///
/// The [*fixed-macro* crate] provides a convenient macro to write down
/// fixed-point constants literally in code which has two advantages over this
/// macro:
///
///  1. It can handle fixed-point numbers with fractions, not just integers.
///  2. It can be used anywhere an expression or constant expression can be
///     used, not just to define a constant.
///
/// # Examples
///
/// ```rust
/// use fixed::{const_fixed_from_int, types::I16F16};
/// const_fixed_from_int! {
///     // define a constant using an integer
///     const FIVE: I16F16 = 5;
///     // define a constant using an integer expression
///     const SUM: I16F16 = 3 + 2;
/// }
/// assert_eq!(FIVE, 5);
/// assert_eq!(SUM, 5);
/// ```
///
/// The following would fail to compile because
/// <code>[i32]::[MAX][i32::MAX]</code> is not representable by [`I16F16`].
///
/// ```rust,compile_fail
/// use fixed::{const_fixed_from_int, types::I16F16};
/// const_fixed_from_int! {
///     // fails because i32::MAX > I16F16::MAX
///     const _OVERFLOW: I16F16 = i32::MAX;
/// }
/// ```
///
/// The following would fail to compile because [`I16F16`] is an alias for
/// <code>[FixedI32]\<[U32]></code>, and this macro can define [`FixedI32`]
/// constants using [`i32`] expressions, not [`i16`] expressions.
///
/// ```rust,compile_fail
/// use fixed::{const_fixed_from_int, types::I16F16};
/// const_fixed_from_int! {
///     // fails because 0i16 is not of type i32
///     const _MISMATCH: I16F16 = 0i16;
/// }
/// ```
///
/// [*fixed-macro* crate]: https://crates.io/crates/fixed-macro
/// [`I16F16`]: crate::types::I16F16
/// [`from_num`]: FixedI32::from_num
#[macro_export]
macro_rules! const_fixed_from_int {
    ($(const $NAME:ident: $Fixed:ty = $int:expr;)*) => { $(
        const $NAME: $Fixed = <$Fixed>::from_bits({
            // Coerce type.
            let int = <$Fixed>::from_bits($int).to_bits();
            // Divide shift into two parts for cases where $Fixed cannot represent 1.
            let frac_nbits = <$Fixed>::FRAC_NBITS;
            let one_a = <$Fixed>::DELTA.to_bits() << (frac_nbits / 2);
            let one_b = <$Fixed>::DELTA.to_bits() << (frac_nbits - frac_nbits / 2);
            int * one_a * one_b
        });
    )* };
}

/// These are doc tests that should not appear in the docs, but are useful as
/// doc tests can check to ensure compilation failure.
///
/// The first snippet succeeds, and acts as a control.
///
/// ```rust
/// use fixed::{const_fixed_from_int, types::*};
/// const_fixed_from_int! {
///     const ZERO_I0: I0F32 = 0;
///     const ZERO_I1: I32F0 = 0;
///     const ZERO_U0: U0F32 = 0;
///     const ZERO_U1: U32F0 = 0;
///
///     const ONE_I0: I2F30 = 1;
///     const ONE_I1: I32F0 = 1;
///     const ONE_U0: U1F31 = 1;
///     const ONE_U1: U32F0 = 1;
///
///     const MINUS_ONE_I0: I1F31 = -1;
///     const MINUS_ONE_I1: I32F0 = -1;
///
///     const MINUS_TWO_I0: I2F30 = -2;
///     const MINUS_TWO_I1: I32F0 = -2;
/// }
/// assert_eq!(ZERO_I0, 0);
/// assert_eq!(ZERO_I1, 0);
/// assert_eq!(ZERO_U0, 0);
/// assert_eq!(ZERO_U1, 0);
///
/// assert_eq!(ONE_I0, 1);
/// assert_eq!(ONE_I1, 1);
/// assert_eq!(ONE_U0, 1);
/// assert_eq!(ONE_U1, 1);
///
/// assert_eq!(MINUS_ONE_I0, -1);
/// assert_eq!(MINUS_ONE_I1, -1);
///
/// assert_eq!(MINUS_TWO_I0, -2);
/// assert_eq!(MINUS_TWO_I1, -2);
/// ```
///
/// The rest of the tests should all fail compilation.
///
/// Not enough integer bits for 1.
/// ```rust,compile_fail
/// use fixed::{const_fixed_from_int, types::*};
/// const_fixed_from_int! {
///     const _ONE: I0F32 = 1;
/// }
/// ```
/// ```rust,compile_fail
/// use fixed::{const_fixed_from_int, types::*};
/// const_fixed_from_int! {
///     const _ONE: I1F31 = 1;
/// }
/// ```
/// ```rust,compile_fail
/// use fixed::{const_fixed_from_int, types::*};
/// const_fixed_from_int! {
///     const _ONE: U0F32 = 1;
/// }
/// ```
///
/// Not enough integer bits for -1.
/// ```rust,compile_fail
/// use fixed::{const_fixed_from_int, types::*};
/// const_fixed_from_int! {
///     const _MINUS_ONE: I0F32 = -1;
/// }
/// ```
///
/// Not enough integer bits for -2.
/// ```rust,compile_fail
/// use fixed::{const_fixed_from_int, types::*};
/// const_fixed_from_int! {
///     const _MINUS_TWO: I1F31 = -2;
/// }
/// ```
fn _compile_fail_tests() {}

#[cfg(test)]
mod tests {
    use crate::types::{I0F32, I16F16, I1F31, U0F32, U16F16};

    #[test]
    fn rounding_signed() {
        // -0.5
        let f = I0F32::from_bits(-1 << 31);
        assert_eq!(f.to_num::<i32>(), -1);
        assert_eq!(f.round_to_zero(), 0);
        assert_eq!(f.overflowing_ceil(), (I0F32::ZERO, false));
        assert_eq!(f.overflowing_floor(), (I0F32::ZERO, true));
        assert_eq!(f.overflowing_round(), (I0F32::ZERO, true));
        assert_eq!(f.overflowing_round_ties_to_even(), (I0F32::ZERO, false));

        // -0.5 + Δ
        let f = I0F32::from_bits((-1 << 31) + 1);
        assert_eq!(f.to_num::<i32>(), -1);
        assert_eq!(f.round_to_zero(), 0);
        assert_eq!(f.overflowing_ceil(), (I0F32::ZERO, false));
        assert_eq!(f.overflowing_floor(), (I0F32::ZERO, true));
        assert_eq!(f.overflowing_round(), (I0F32::ZERO, false));
        assert_eq!(f.overflowing_round_ties_to_even(), (I0F32::ZERO, false));

        // 0
        let f = I0F32::from_bits(0);
        assert_eq!(f.to_num::<i32>(), 0);
        assert_eq!(f.round_to_zero(), 0);
        assert_eq!(f.overflowing_ceil(), (I0F32::ZERO, false));
        assert_eq!(f.overflowing_floor(), (I0F32::ZERO, false));
        assert_eq!(f.overflowing_round(), (I0F32::ZERO, false));
        assert_eq!(f.overflowing_round_ties_to_even(), (I0F32::ZERO, false));

        // 0.5 - Δ
        let f = I0F32::from_bits((1 << 30) - 1 + (1 << 30));
        assert_eq!(f.to_num::<i32>(), 0);
        assert_eq!(f.round_to_zero(), 0);
        assert_eq!(f.overflowing_ceil(), (I0F32::ZERO, true));
        assert_eq!(f.overflowing_floor(), (I0F32::ZERO, false));
        assert_eq!(f.overflowing_round(), (I0F32::ZERO, false));
        assert_eq!(f.overflowing_round_ties_to_even(), (I0F32::ZERO, false));

        // -1
        let f = I1F31::from_bits((-1) << 31);
        assert_eq!(f.to_num::<i32>(), -1);
        assert_eq!(f.round_to_zero(), -1);
        assert_eq!(f.overflowing_ceil(), (I1F31::from_num(-1), false));
        assert_eq!(f.overflowing_floor(), (I1F31::from_num(-1), false));
        assert_eq!(f.overflowing_round(), (I1F31::from_num(-1), false));
        assert_eq!(
            f.overflowing_round_ties_to_even(),
            (I1F31::from_num(-1), false)
        );

        // -0.5 - Δ
        let f = I1F31::from_bits(((-1) << 30) - 1);
        assert_eq!(f.to_num::<i32>(), -1);
        assert_eq!(f.round_to_zero(), 0);
        assert_eq!(f.overflowing_ceil(), (I1F31::ZERO, false));
        assert_eq!(f.overflowing_floor(), (I1F31::from_num(-1), false));
        assert_eq!(f.overflowing_round(), (I1F31::from_num(-1), false));
        assert_eq!(
            f.overflowing_round_ties_to_even(),
            (I1F31::from_num(-1), false)
        );

        // -0.5
        let f = I1F31::from_bits((-1) << 30);
        assert_eq!(f.to_num::<i32>(), -1);
        assert_eq!(f.round_to_zero(), 0);
        assert_eq!(f.overflowing_ceil(), (I1F31::ZERO, false));
        assert_eq!(f.overflowing_floor(), (I1F31::from_num(-1), false));
        assert_eq!(f.overflowing_round(), (I1F31::from_num(-1), false));
        assert_eq!(f.overflowing_round_ties_to_even(), (I1F31::ZERO, false));

        // -0.5 + Δ
        let f = I1F31::from_bits(((-1) << 30) + 1);
        assert_eq!(f.to_num::<i32>(), -1);
        assert_eq!(f.round_to_zero(), 0);
        assert_eq!(f.overflowing_ceil(), (I1F31::ZERO, false));
        assert_eq!(f.overflowing_floor(), (I1F31::from_num(-1), false));
        assert_eq!(f.overflowing_round(), (I1F31::ZERO, false));
        assert_eq!(f.overflowing_round_ties_to_even(), (I1F31::ZERO, false));

        // 0.5 - Δ
        let f = I1F31::from_bits((1 << 30) - 1);
        assert_eq!(f.to_num::<i32>(), 0);
        assert_eq!(f.round_to_zero(), 0);
        assert_eq!(f.overflowing_ceil(), (I1F31::from_num(-1), true));
        assert_eq!(f.overflowing_floor(), (I1F31::ZERO, false));
        assert_eq!(f.overflowing_round(), (I1F31::ZERO, false));
        assert_eq!(f.overflowing_round_ties_to_even(), (I1F31::ZERO, false));

        // 0.5
        let f = I1F31::from_bits(1 << 30);
        assert_eq!(f.to_num::<i32>(), 0);
        assert_eq!(f.round_to_zero(), 0);
        assert_eq!(f.overflowing_ceil(), (I1F31::from_num(-1), true));
        assert_eq!(f.overflowing_floor(), (I1F31::ZERO, false));
        assert_eq!(f.overflowing_round(), (I1F31::from_num(-1), true));
        assert_eq!(f.overflowing_round_ties_to_even(), (I1F31::ZERO, false));

        // 0
        let f = I1F31::from_bits(0);
        assert_eq!(f.to_num::<i32>(), 0);
        assert_eq!(f.round_to_zero(), 0);
        assert_eq!(f.overflowing_ceil(), (I1F31::ZERO, false));
        assert_eq!(f.overflowing_floor(), (I1F31::ZERO, false));
        assert_eq!(f.overflowing_round(), (I1F31::ZERO, false));
        assert_eq!(f.overflowing_round_ties_to_even(), (I1F31::ZERO, false));

        // 0.5 + Δ
        let f = I1F31::from_bits((1 << 30) + 1);
        assert_eq!(f.to_num::<i32>(), 0);
        assert_eq!(f.round_to_zero(), 0);
        assert_eq!(f.overflowing_ceil(), (I1F31::from_num(-1), true));
        assert_eq!(f.overflowing_floor(), (I1F31::ZERO, false));
        assert_eq!(f.overflowing_round(), (I1F31::from_num(-1), true));
        assert_eq!(
            f.overflowing_round_ties_to_even(),
            (I1F31::from_num(-1), true)
        );

        // -3.5 - Δ
        let f = I16F16::from_bits(((-7) << 15) - 1);
        assert_eq!(f.to_num::<i32>(), -4);
        assert_eq!(f.round_to_zero(), -3);
        assert_eq!(f.overflowing_ceil(), (I16F16::from_num(-3), false));
        assert_eq!(f.overflowing_floor(), (I16F16::from_num(-4), false));
        assert_eq!(f.overflowing_round(), (I16F16::from_num(-4), false));
        assert_eq!(
            f.overflowing_round_ties_to_even(),
            (I16F16::from_num(-4), false)
        );

        // -3.5
        let f = I16F16::from_bits((-7) << 15);
        assert_eq!(f.to_num::<i32>(), -4);
        assert_eq!(f.round_to_zero(), -3);
        assert_eq!(f.overflowing_ceil(), (I16F16::from_num(-3), false));
        assert_eq!(f.overflowing_floor(), (I16F16::from_num(-4), false));
        assert_eq!(f.overflowing_round(), (I16F16::from_num(-4), false));
        assert_eq!(
            f.overflowing_round_ties_to_even(),
            (I16F16::from_num(-4), false)
        );

        // -3.5 + Δ
        let f = I16F16::from_bits(((-7) << 15) + 1);
        assert_eq!(f.to_num::<i32>(), -4);
        assert_eq!(f.round_to_zero(), -3);
        assert_eq!(f.overflowing_ceil(), (I16F16::from_num(-3), false));
        assert_eq!(f.overflowing_floor(), (I16F16::from_num(-4), false));
        assert_eq!(f.overflowing_round(), (I16F16::from_num(-3), false));
        assert_eq!(
            f.overflowing_round_ties_to_even(),
            (I16F16::from_num(-3), false)
        );

        // -2.5 - Δ
        let f = I16F16::from_bits(((-5) << 15) - 1);
        assert_eq!(f.to_num::<i32>(), -3);
        assert_eq!(f.round_to_zero(), -2);
        assert_eq!(f.overflowing_ceil(), (I16F16::from_num(-2), false));
        assert_eq!(f.overflowing_floor(), (I16F16::from_num(-3), false));
        assert_eq!(f.overflowing_round(), (I16F16::from_num(-3), false));
        assert_eq!(
            f.overflowing_round_ties_to_even(),
            (I16F16::from_num(-3), false)
        );

        // -2.5
        let f = I16F16::from_bits((-5) << 15);
        assert_eq!(f.to_num::<i32>(), -3);
        assert_eq!(f.round_to_zero(), -2);
        assert_eq!(f.overflowing_ceil(), (I16F16::from_num(-2), false));
        assert_eq!(f.overflowing_floor(), (I16F16::from_num(-3), false));
        assert_eq!(f.overflowing_round(), (I16F16::from_num(-3), false));
        assert_eq!(
            f.overflowing_round_ties_to_even(),
            (I16F16::from_num(-2), false)
        );

        // -2.5 + Δ
        let f = I16F16::from_bits(((-5) << 15) + 1);
        assert_eq!(f.to_num::<i32>(), -3);
        assert_eq!(f.round_to_zero(), -2);
        assert_eq!(f.overflowing_ceil(), (I16F16::from_num(-2), false));
        assert_eq!(f.overflowing_floor(), (I16F16::from_num(-3), false));
        assert_eq!(f.overflowing_round(), (I16F16::from_num(-2), false));
        assert_eq!(
            f.overflowing_round_ties_to_even(),
            (I16F16::from_num(-2), false)
        );

        // -1
        let f = I16F16::from_bits((-1) << 16);
        assert_eq!(f.to_num::<i32>(), -1);
        assert_eq!(f.round_to_zero(), -1);
        assert_eq!(f.overflowing_ceil(), (I16F16::from_num(-1), false));
        assert_eq!(f.overflowing_floor(), (I16F16::from_num(-1), false));
        assert_eq!(f.overflowing_round(), (I16F16::from_num(-1), false));
        assert_eq!(
            f.overflowing_round_ties_to_even(),
            (I16F16::from_num(-1), false)
        );

        // -0.5 - Δ
        let f = I16F16::from_bits(((-1) << 15) - 1);
        assert_eq!(f.to_num::<i32>(), -1);
        assert_eq!(f.round_to_zero(), 0);
        assert_eq!(f.overflowing_ceil(), (I16F16::ZERO, false));
        assert_eq!(f.overflowing_floor(), (I16F16::from_num(-1), false));
        assert_eq!(f.overflowing_round(), (I16F16::from_num(-1), false));
        assert_eq!(
            f.overflowing_round_ties_to_even(),
            (I16F16::from_num(-1), false)
        );

        // -0.5
        let f = I16F16::from_bits((-1) << 15);
        assert_eq!(f.to_num::<i32>(), -1);
        assert_eq!(f.round_to_zero(), 0);
        assert_eq!(f.overflowing_ceil(), (I16F16::ZERO, false));
        assert_eq!(f.overflowing_floor(), (I16F16::from_num(-1), false));
        assert_eq!(f.overflowing_round(), (I16F16::from_num(-1), false));
        assert_eq!(f.overflowing_round_ties_to_even(), (I16F16::ZERO, false));

        // -0.5 + Δ
        let f = I16F16::from_bits(((-1) << 15) + 1);
        assert_eq!(f.to_num::<i32>(), -1);
        assert_eq!(f.round_to_zero(), 0);
        assert_eq!(f.overflowing_ceil(), (I16F16::ZERO, false));
        assert_eq!(f.overflowing_floor(), (I16F16::from_num(-1), false));
        assert_eq!(f.overflowing_round(), (I16F16::ZERO, false));
        assert_eq!(f.overflowing_round_ties_to_even(), (I16F16::ZERO, false));

        // 0
        let f = I16F16::from_bits(0);
        assert_eq!(f.to_num::<i32>(), 0);
        assert_eq!(f.round_to_zero(), 0);
        assert_eq!(f.overflowing_ceil(), (I16F16::ZERO, false));
        assert_eq!(f.overflowing_floor(), (I16F16::ZERO, false));
        assert_eq!(f.overflowing_round(), (I16F16::ZERO, false));
        assert_eq!(f.overflowing_round_ties_to_even(), (I16F16::ZERO, false));

        // 0.5 - Δ
        let f = I16F16::from_bits((1 << 15) - 1);
        assert_eq!(f.to_num::<i32>(), 0);
        assert_eq!(f.round_to_zero(), 0);
        assert_eq!(f.overflowing_ceil(), (I16F16::ONE, false));
        assert_eq!(f.overflowing_floor(), (I16F16::ZERO, false));
        assert_eq!(f.overflowing_round(), (I16F16::ZERO, false));
        assert_eq!(f.overflowing_round_ties_to_even(), (I16F16::ZERO, false));

        // 0.5
        let f = I16F16::from_bits(1 << 15);
        assert_eq!(f.to_num::<i32>(), 0);
        assert_eq!(f.round_to_zero(), 0);
        assert_eq!(f.overflowing_ceil(), (I16F16::ONE, false));
        assert_eq!(f.overflowing_floor(), (I16F16::ZERO, false));
        assert_eq!(f.overflowing_round(), (I16F16::ONE, false));
        assert_eq!(f.overflowing_round_ties_to_even(), (I16F16::ZERO, false));

        // 0.5 + Δ
        let f = I16F16::from_bits((1 << 15) + 1);
        assert_eq!(f.to_num::<i32>(), 0);
        assert_eq!(f.round_to_zero(), 0);
        assert_eq!(f.overflowing_ceil(), (I16F16::ONE, false));
        assert_eq!(f.overflowing_floor(), (I16F16::ZERO, false));
        assert_eq!(f.overflowing_round(), (I16F16::ONE, false));
        assert_eq!(f.overflowing_round_ties_to_even(), (I16F16::ONE, false));

        // 1
        let f = I16F16::from_bits(1 << 16);
        assert_eq!(f.to_num::<i32>(), 1);
        assert_eq!(f.round_to_zero(), 1);
        assert_eq!(f.overflowing_ceil(), (I16F16::ONE, false));
        assert_eq!(f.overflowing_floor(), (I16F16::ONE, false));
        assert_eq!(f.overflowing_round(), (I16F16::ONE, false));
        assert_eq!(f.overflowing_round_ties_to_even(), (I16F16::ONE, false));

        // 2.5 - Δ
        let f = I16F16::from_bits((5 << 15) - 1);
        assert_eq!(f.to_num::<i32>(), 2);
        assert_eq!(f.round_to_zero(), 2);
        assert_eq!(f.overflowing_ceil(), (I16F16::from_num(3), false));
        assert_eq!(f.overflowing_floor(), (I16F16::from_num(2), false));
        assert_eq!(f.overflowing_round(), (I16F16::from_num(2), false));
        assert_eq!(
            f.overflowing_round_ties_to_even(),
            (I16F16::from_num(2), false)
        );

        // 2.5
        let f = I16F16::from_bits(5 << 15);
        assert_eq!(f.to_num::<i32>(), 2);
        assert_eq!(f.round_to_zero(), 2);
        assert_eq!(f.overflowing_ceil(), (I16F16::from_num(3), false));
        assert_eq!(f.overflowing_floor(), (I16F16::from_num(2), false));
        assert_eq!(f.overflowing_round(), (I16F16::from_num(3), false));
        assert_eq!(
            f.overflowing_round_ties_to_even(),
            (I16F16::from_num(2), false)
        );

        // 2.5 + Δ
        let f = I16F16::from_bits((5 << 15) + 1);
        assert_eq!(f.to_num::<i32>(), 2);
        assert_eq!(f.round_to_zero(), 2);
        assert_eq!(f.overflowing_ceil(), (I16F16::from_num(3), false));
        assert_eq!(f.overflowing_floor(), (I16F16::from_num(2), false));
        assert_eq!(f.overflowing_round(), (I16F16::from_num(3), false));
        assert_eq!(
            f.overflowing_round_ties_to_even(),
            (I16F16::from_num(3), false)
        );

        // 3.5 - Δ
        let f = I16F16::from_bits((7 << 15) - 1);
        assert_eq!(f.to_num::<i32>(), 3);
        assert_eq!(f.round_to_zero(), 3);
        assert_eq!(f.overflowing_ceil(), (I16F16::from_num(4), false));
        assert_eq!(f.overflowing_floor(), (I16F16::from_num(3), false));
        assert_eq!(f.overflowing_round(), (I16F16::from_num(3), false));
        assert_eq!(
            f.overflowing_round_ties_to_even(),
            (I16F16::from_num(3), false)
        );

        // 3.5
        let f = I16F16::from_bits(7 << 15);
        assert_eq!(f.to_num::<i32>(), 3);
        assert_eq!(f.round_to_zero(), 3);
        assert_eq!(f.overflowing_ceil(), (I16F16::from_num(4), false));
        assert_eq!(f.overflowing_floor(), (I16F16::from_num(3), false));
        assert_eq!(f.overflowing_round(), (I16F16::from_num(4), false));
        assert_eq!(
            f.overflowing_round_ties_to_even(),
            (I16F16::from_num(4), false)
        );

        // 3.5 + Δ
        let f = I16F16::from_bits((7 << 15) + 1);
        assert_eq!(f.to_num::<i32>(), 3);
        assert_eq!(f.round_to_zero(), 3);
        assert_eq!(f.overflowing_ceil(), (I16F16::from_num(4), false));
        assert_eq!(f.overflowing_floor(), (I16F16::from_num(3), false));
        assert_eq!(f.overflowing_round(), (I16F16::from_num(4), false));
        assert_eq!(
            f.overflowing_round_ties_to_even(),
            (I16F16::from_num(4), false)
        );
    }

    #[test]
    fn rounding_unsigned() {
        // 0
        let f = U0F32::from_bits(0);
        assert_eq!(f.to_num::<i32>(), 0);
        assert_eq!(f.round_to_zero(), 0);
        assert_eq!(f.overflowing_ceil(), (U0F32::ZERO, false));
        assert_eq!(f.overflowing_floor(), (U0F32::ZERO, false));
        assert_eq!(f.overflowing_round(), (U0F32::ZERO, false));
        assert_eq!(f.overflowing_round_ties_to_even(), (U0F32::ZERO, false));

        // 0.5 - Δ
        let f = U0F32::from_bits((1 << 31) - 1);
        assert_eq!(f.to_num::<i32>(), 0);
        assert_eq!(f.round_to_zero(), 0);
        assert_eq!(f.overflowing_ceil(), (U0F32::ZERO, true));
        assert_eq!(f.overflowing_floor(), (U0F32::ZERO, false));
        assert_eq!(f.overflowing_round(), (U0F32::ZERO, false));
        assert_eq!(f.overflowing_round_ties_to_even(), (U0F32::ZERO, false));

        // 0.5
        let f = U0F32::from_bits(1 << 31);
        assert_eq!(f.to_num::<i32>(), 0);
        assert_eq!(f.round_to_zero(), 0);
        assert_eq!(f.overflowing_ceil(), (U0F32::ZERO, true));
        assert_eq!(f.overflowing_floor(), (U0F32::ZERO, false));
        assert_eq!(f.overflowing_round(), (U0F32::ZERO, true));
        assert_eq!(f.overflowing_round_ties_to_even(), (U0F32::ZERO, false));

        // 0.5 + Δ
        let f = U0F32::from_bits((1 << 31) + 1);
        assert_eq!(f.to_num::<i32>(), 0);
        assert_eq!(f.round_to_zero(), 0);
        assert_eq!(f.overflowing_ceil(), (U0F32::ZERO, true));
        assert_eq!(f.overflowing_floor(), (U0F32::ZERO, false));
        assert_eq!(f.overflowing_round(), (U0F32::ZERO, true));
        assert_eq!(f.overflowing_round_ties_to_even(), (U0F32::ZERO, true));

        // 0
        let f = U16F16::from_bits(0);
        assert_eq!(f.to_num::<i32>(), 0);
        assert_eq!(f.round_to_zero(), 0);
        assert_eq!(f.overflowing_ceil(), (U16F16::ZERO, false));
        assert_eq!(f.overflowing_floor(), (U16F16::ZERO, false));
        assert_eq!(f.overflowing_round(), (U16F16::ZERO, false));
        assert_eq!(f.overflowing_round_ties_to_even(), (U16F16::ZERO, false));

        // 0.5 - Δ
        let f = U16F16::from_bits((1 << 15) - 1);
        assert_eq!(f.to_num::<i32>(), 0);
        assert_eq!(f.round_to_zero(), 0);
        assert_eq!(f.overflowing_ceil(), (U16F16::ONE, false));
        assert_eq!(f.overflowing_floor(), (U16F16::ZERO, false));
        assert_eq!(f.overflowing_round(), (U16F16::ZERO, false));
        assert_eq!(f.overflowing_round_ties_to_even(), (U16F16::ZERO, false));

        // 0.5
        let f = U16F16::from_bits(1 << 15);
        assert_eq!(f.to_num::<i32>(), 0);
        assert_eq!(f.round_to_zero(), 0);
        assert_eq!(f.overflowing_ceil(), (U16F16::ONE, false));
        assert_eq!(f.overflowing_floor(), (U16F16::ZERO, false));
        assert_eq!(f.overflowing_round(), (U16F16::ONE, false));
        assert_eq!(f.overflowing_round_ties_to_even(), (U16F16::ZERO, false));

        // 0.5 + Δ
        let f = U16F16::from_bits((1 << 15) + 1);
        assert_eq!(f.to_num::<i32>(), 0);
        assert_eq!(f.round_to_zero(), 0);
        assert_eq!(f.overflowing_ceil(), (U16F16::ONE, false));
        assert_eq!(f.overflowing_floor(), (U16F16::ZERO, false));
        assert_eq!(f.overflowing_round(), (U16F16::ONE, false));
        assert_eq!(f.overflowing_round_ties_to_even(), (U16F16::ONE, false));

        // 1
        let f = U16F16::from_bits(1 << 16);
        assert_eq!(f.to_num::<i32>(), 1);
        assert_eq!(f.round_to_zero(), 1);
        assert_eq!(f.overflowing_ceil(), (U16F16::ONE, false));
        assert_eq!(f.overflowing_floor(), (U16F16::ONE, false));
        assert_eq!(f.overflowing_round(), (U16F16::ONE, false));
        assert_eq!(f.overflowing_round_ties_to_even(), (U16F16::ONE, false));

        // 2.5 - Δ
        let f = U16F16::from_bits((5 << 15) - 1);
        assert_eq!(f.to_num::<i32>(), 2);
        assert_eq!(f.round_to_zero(), 2);
        assert_eq!(f.overflowing_ceil(), (U16F16::from_num(3), false));
        assert_eq!(f.overflowing_floor(), (U16F16::from_num(2), false));
        assert_eq!(f.overflowing_round(), (U16F16::from_num(2), false));
        assert_eq!(
            f.overflowing_round_ties_to_even(),
            (U16F16::from_num(2), false)
        );

        // 2.5
        let f = U16F16::from_bits(5 << 15);
        assert_eq!(f.to_num::<i32>(), 2);
        assert_eq!(f.round_to_zero(), 2);
        assert_eq!(f.overflowing_ceil(), (U16F16::from_num(3), false));
        assert_eq!(f.overflowing_floor(), (U16F16::from_num(2), false));
        assert_eq!(f.overflowing_round(), (U16F16::from_num(3), false));
        assert_eq!(
            f.overflowing_round_ties_to_even(),
            (U16F16::from_num(2), false)
        );

        // 2.5 + Δ
        let f = U16F16::from_bits((5 << 15) + 1);
        assert_eq!(f.to_num::<i32>(), 2);
        assert_eq!(f.round_to_zero(), 2);
        assert_eq!(f.overflowing_ceil(), (U16F16::from_num(3), false));
        assert_eq!(f.overflowing_floor(), (U16F16::from_num(2), false));
        assert_eq!(f.overflowing_round(), (U16F16::from_num(3), false));
        assert_eq!(
            f.overflowing_round_ties_to_even(),
            (U16F16::from_num(3), false)
        );

        // 3.5 - Δ
        let f = U16F16::from_bits((7 << 15) - 1);
        assert_eq!(f.to_num::<i32>(), 3);
        assert_eq!(f.round_to_zero(), 3);
        assert_eq!(f.overflowing_ceil(), (U16F16::from_num(4), false));
        assert_eq!(f.overflowing_floor(), (U16F16::from_num(3), false));
        assert_eq!(f.overflowing_round(), (U16F16::from_num(3), false));
        assert_eq!(
            f.overflowing_round_ties_to_even(),
            (U16F16::from_num(3), false)
        );

        // 3.5
        let f = U16F16::from_bits(7 << 15);
        assert_eq!(f.to_num::<i32>(), 3);
        assert_eq!(f.round_to_zero(), 3);
        assert_eq!(f.overflowing_ceil(), (U16F16::from_num(4), false));
        assert_eq!(f.overflowing_floor(), (U16F16::from_num(3), false));
        assert_eq!(f.overflowing_round(), (U16F16::from_num(4), false));
        assert_eq!(
            f.overflowing_round_ties_to_even(),
            (U16F16::from_num(4), false)
        );

        // 3.5 + Δ
        let f = U16F16::from_bits((7 << 15) + 1);
        assert_eq!(f.to_num::<i32>(), 3);
        assert_eq!(f.round_to_zero(), 3);
        assert_eq!(f.overflowing_ceil(), (U16F16::from_num(4), false));
        assert_eq!(f.overflowing_floor(), (U16F16::from_num(3), false));
        assert_eq!(f.overflowing_round(), (U16F16::from_num(4), false));
        assert_eq!(
            f.overflowing_round_ties_to_even(),
            (U16F16::from_num(4), false)
        );
    }

    #[test]
    fn reciprocals() {
        // 4/3 wraps to 1/3 = 0x0.5555_5555
        assert_eq!(
            U0F32::from_num(0.75).overflowing_recip(),
            (U0F32::from_bits(0x5555_5555), true)
        );
        // 8/3 wraps to 2/3 = 0x0.AAAA_AAAA
        assert_eq!(
            U0F32::from_num(0.375).overflowing_recip(),
            (U0F32::from_bits(0xAAAA_AAAA), true)
        );

        // 8/3 wraps to 2/3 = 0x0.AAAA_AAAA, which is -0x0.5555_5556
        assert_eq!(
            I0F32::from_num(0.375).overflowing_recip(),
            (I0F32::from_bits(-0x5555_5556), true)
        );
        assert_eq!(
            I0F32::from_num(-0.375).overflowing_recip(),
            (I0F32::from_bits(0x5555_5556), true)
        );
        // -2 wraps to 0
        assert_eq!(
            I0F32::from_num(-0.5).overflowing_recip(),
            (I0F32::ZERO, true)
        );

        // 8/3 wraps to 2/3 = 0x0.AAAA_AAAA (bits 0x5555_5555)
        assert_eq!(
            I1F31::from_num(0.375).overflowing_recip(),
            (I1F31::from_bits(0x5555_5555), true)
        );
        assert_eq!(
            I1F31::from_num(-0.375).overflowing_recip(),
            (I1F31::from_bits(-0x5555_5555), true)
        );
        // 4/3 = 0x1.5555_5554 (bits 0xAAAA_AAAA, or -0x5555_5556)
        assert_eq!(
            I1F31::from_num(0.75).overflowing_recip(),
            (I1F31::from_bits(-0x5555_5556), true)
        );
        assert_eq!(
            I1F31::from_num(-0.75).overflowing_recip(),
            (I1F31::from_bits(0x5555_5556), true)
        );
        // -2 wraps to 0
        assert_eq!(
            I1F31::from_num(-0.5).overflowing_recip(),
            (I1F31::ZERO, true)
        );
        // -1 does not overflow
        assert_eq!(
            I1F31::from_num(-1).overflowing_recip(),
            (I1F31::from_num(-1), false)
        );
    }
}
