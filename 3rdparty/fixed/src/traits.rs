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
Traits for conversions and for generic use of fixed-point numbers.
*/

use crate::{
    helpers::{Sealed, Widest},
    types::extra::{LeEqU128, LeEqU16, LeEqU32, LeEqU64, LeEqU8, Unsigned},
    F128Bits, FixedI128, FixedI16, FixedI32, FixedI64, FixedI8, FixedU128, FixedU16, FixedU32,
    FixedU64, FixedU8, ParseFixedError,
};
#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;
#[cfg(feature = "borsh")]
use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{self, Pod, TransparentWrapper};
use core::{
    fmt::{Binary, Debug, Display, LowerHex, Octal, UpperHex},
    hash::Hash,
    iter::{Product, Sum},
    mem,
    num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
        NonZeroU32, NonZeroU64, NonZeroU8,
    },
    ops::{
        Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div,
        DivAssign, Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub,
        SubAssign,
    },
    str::FromStr,
};
use half::{bf16, f16};
#[cfg(feature = "num-traits")]
use num_traits::{
    bounds::Bounded,
    cast::{FromPrimitive, ToPrimitive},
    float::FloatConst,
    identities::Zero,
    ops::{
        checked::{
            CheckedAdd, CheckedDiv, CheckedMul, CheckedNeg, CheckedRem, CheckedShl, CheckedShr,
            CheckedSub,
        },
        inv::Inv,
        overflowing::{OverflowingAdd, OverflowingMul, OverflowingSub},
        saturating::{SaturatingAdd, SaturatingMul, SaturatingSub},
        wrapping::{WrappingAdd, WrappingMul, WrappingNeg, WrappingShl, WrappingShr, WrappingSub},
    },
};
#[cfg(feature = "serde")]
use serde::{de::Deserialize, ser::Serialize};

macro_rules! comment_features {
    ($comment:expr) => {
        #[cfg(all(
            not(feature = "arbitrary"),
            not(feature = "borsh"),
            not(feature = "num-traits"),
            not(feature = "serde")
        ))]
        doc_comment! {
            $comment;
            pub trait FixedOptionalFeatures: Sealed {}
        }

        #[cfg(all(
            not(feature = "arbitrary"),
            not(feature = "borsh"),
            not(feature = "num-traits"),
            feature = "serde"
        ))]
        doc_comment! {
            $comment;
            pub trait FixedOptionalFeatures: Sealed
            where
                Self: Serialize + for<'de> Deserialize<'de>,
            {
            }
        }

        // Do *not* add MulAdd constaint, as it conflicts with Fixed::mul_add
        #[cfg(all(
            not(feature = "arbitrary"),
            not(feature = "borsh"),
            feature = "num-traits",
            not(feature = "serde")
        ))]
        doc_comment! {
            $comment;
            pub trait FixedOptionalFeatures: Sealed
            where
                Self: Zero + Bounded + Inv,
                Self: CheckedAdd + CheckedSub + CheckedNeg + CheckedMul,
                Self: CheckedDiv + CheckedRem + CheckedShl + CheckedShr,
                Self: SaturatingAdd + SaturatingSub + SaturatingMul,
                Self: WrappingAdd + WrappingSub + WrappingNeg + WrappingMul,
                Self: WrappingShl + WrappingShr,
                Self: OverflowingAdd + OverflowingSub + OverflowingMul,
                Self: ToPrimitive + FromPrimitive + FloatConst,
            {
            }
        }

        // Do *not* add MulAdd constaint, as it conflicts with Fixed::mul_add
        #[cfg(all(
            not(feature = "arbitrary"),
            not(feature = "borsh"),
            feature = "num-traits",
            feature = "serde"
        ))]
        doc_comment! {
            $comment;
            pub trait FixedOptionalFeatures: Sealed
            where
                Self: Zero + Bounded + Inv,
                Self: CheckedAdd + CheckedSub + CheckedNeg + CheckedMul,
                Self: CheckedDiv + CheckedRem + CheckedShl + CheckedShr,
                Self: SaturatingAdd + SaturatingSub + SaturatingMul,
                Self: WrappingAdd + WrappingSub + WrappingNeg + WrappingMul,
                Self: WrappingShl + WrappingShr,
                Self: OverflowingAdd + OverflowingSub + OverflowingMul,
                Self: ToPrimitive + FromPrimitive + FloatConst,
                Self: Serialize + for<'de> Deserialize<'de>,
            {
            }
        }

        #[cfg(all(
            not(feature = "arbitrary"),
            feature = "borsh",
            not(feature = "num-traits"),
            not(feature = "serde")
        ))]
        doc_comment! {
            $comment;
            pub trait FixedOptionalFeatures: Sealed
            where
                Self: BorshSerialize + BorshDeserialize,
            {}
        }

        #[cfg(all(
            not(feature = "arbitrary"),
            feature = "borsh",
            not(feature = "num-traits"),
            feature = "serde"
        ))]
        doc_comment! {
            $comment;
            pub trait FixedOptionalFeatures: Sealed
            where
                Self: BorshSerialize + BorshDeserialize,
                Self: Serialize + for<'de> Deserialize<'de>,
            {
            }
        }

        // Do *not* add MulAdd constaint, as it conflicts with Fixed::mul_add
        #[cfg(all(
            not(feature = "arbitrary"),
            feature = "borsh",
            feature = "num-traits",
            not(feature = "serde")
        ))]
        doc_comment! {
            $comment;
            pub trait FixedOptionalFeatures: Sealed
            where
                Self: BorshSerialize + BorshDeserialize,
                Self: Zero + Bounded + Inv,
                Self: CheckedAdd + CheckedSub + CheckedNeg + CheckedMul,
                Self: CheckedDiv + CheckedRem + CheckedShl + CheckedShr,
                Self: SaturatingAdd + SaturatingSub + SaturatingMul,
                Self: WrappingAdd + WrappingSub + WrappingNeg + WrappingMul,
                Self: WrappingShl + WrappingShr,
                Self: OverflowingAdd + OverflowingSub + OverflowingMul,
                Self: ToPrimitive + FromPrimitive + FloatConst,
            {
            }
        }

        // Do *not* add MulAdd constaint, as it conflicts with Fixed::mul_add
        #[cfg(all(
            not(feature = "arbitrary"),
            feature = "borsh",
            feature = "num-traits",
            feature = "serde"
        ))]
        doc_comment! {
            $comment;
            pub trait FixedOptionalFeatures: Sealed
            where
                Self: BorshSerialize + BorshDeserialize,
                Self: Zero + Bounded + Inv,
                Self: CheckedAdd + CheckedSub + CheckedNeg + CheckedMul,
                Self: CheckedDiv + CheckedRem + CheckedShl + CheckedShr,
                Self: SaturatingAdd + SaturatingSub + SaturatingMul,
                Self: WrappingAdd + WrappingSub + WrappingNeg + WrappingMul,
                Self: WrappingShl + WrappingShr,
                Self: OverflowingAdd + OverflowingSub + OverflowingMul,
                Self: ToPrimitive + FromPrimitive + FloatConst,
                Self: Serialize + for<'de> Deserialize<'de>,
            {
            }
        }

        #[cfg(all(
            feature = "arbitrary",
            not(feature = "borsh"),
            not(feature = "num-traits"),
            not(feature = "serde")
        ))]
        doc_comment! {
            $comment;
            pub trait FixedOptionalFeatures: Sealed
            where
                Self: for<'a> Arbitrary<'a>,
            {
            }
        }

        #[cfg(all(
            feature = "arbitrary",
            not(feature = "borsh"),
            not(feature = "num-traits"),
            feature = "serde"
        ))]
        doc_comment! {
            $comment;
            pub trait FixedOptionalFeatures: Sealed
            where
                Self: for<'a> Arbitrary<'a>,
                Self: Serialize + for<'de> Deserialize<'de>
            {
            }
        }

        // Do *not* add MulAdd constaint, as it conflicts with Fixed::mul_add
        #[cfg(all(
            feature = "arbitrary",
            not(feature = "borsh"),
            feature = "num-traits",
            not(feature = "serde")
        ))]
        doc_comment! {
            $comment;
            pub trait FixedOptionalFeatures: Sealed
            where
                Self: for<'a> Arbitrary<'a>,
                Self: Zero + Bounded + Inv,
                Self: CheckedAdd + CheckedSub + CheckedNeg + CheckedMul,
                Self: CheckedDiv + CheckedRem + CheckedShl + CheckedShr,
                Self: SaturatingAdd + SaturatingSub + SaturatingMul,
                Self: WrappingAdd + WrappingSub + WrappingNeg + WrappingMul,
                Self: WrappingShl + WrappingShr,
                Self: OverflowingAdd + OverflowingSub + OverflowingMul,
                Self: ToPrimitive + FromPrimitive + FloatConst,
            {
            }
        }

        // Do *not* add MulAdd constaint, as it conflicts with Fixed::mul_add
        #[cfg(all(
            feature = "arbitrary",
            not(feature = "borsh"),
            feature = "num-traits",
            feature = "serde"
        ))]
        doc_comment! {
            $comment;
            pub trait FixedOptionalFeatures: Sealed
            where
                Self: for<'a> Arbitrary<'a>,
                Self: Zero + Bounded + Inv,
                Self: CheckedAdd + CheckedSub + CheckedNeg + CheckedMul,
                Self: CheckedDiv + CheckedRem + CheckedShl + CheckedShr,
                Self: SaturatingAdd + SaturatingSub + SaturatingMul,
                Self: WrappingAdd + WrappingSub + WrappingNeg + WrappingMul,
                Self: WrappingShl + WrappingShr,
                Self: OverflowingAdd + OverflowingSub + OverflowingMul,
                Self: ToPrimitive + FromPrimitive + FloatConst,
                Self: Serialize + for<'de> Deserialize<'de>,
            {
            }
        }

        #[cfg(all(
            feature = "arbitrary",
            feature = "borsh",
            not(feature = "num-traits"),
            not(feature = "serde")
        ))]
        doc_comment! {
            $comment;
            pub trait FixedOptionalFeatures: Sealed
            where
                Self: for<'a> Arbitrary<'a>,
                Self: BorshSerialize + BorshDeserialize,
            {
            }
        }

        #[cfg(all(
            feature = "arbitrary",
            feature = "borsh",
            not(feature = "num-traits"),
            feature = "serde"
        ))]
        doc_comment! {
            $comment;
            pub trait FixedOptionalFeatures: Sealed
            where
                Self: for<'a> Arbitrary<'a>,
                Self: BorshSerialize + BorshDeserialize,
                Self: Serialize + for<'de> Deserialize<'de>
            {
            }
        }

        // Do *not* add MulAdd constaint, as it conflicts with Fixed::mul_add
        #[cfg(all(
            feature = "arbitrary",
            feature = "borsh",
            feature = "num-traits",
            not(feature = "serde")
        ))]
        doc_comment! {
            $comment;
            pub trait FixedOptionalFeatures: Sealed
            where
                Self: for<'a> Arbitrary<'a>,
                Self: BorshSerialize + BorshDeserialize,
                Self: Zero + Bounded + Inv,
                Self: CheckedAdd + CheckedSub + CheckedNeg + CheckedMul,
                Self: CheckedDiv + CheckedRem + CheckedShl + CheckedShr,
                Self: SaturatingAdd + SaturatingSub + SaturatingMul,
                Self: WrappingAdd + WrappingSub + WrappingNeg + WrappingMul,
                Self: WrappingShl + WrappingShr,
                Self: OverflowingAdd + OverflowingSub + OverflowingMul,
                Self: ToPrimitive + FromPrimitive + FloatConst,
            {
            }
        }

        // Do *not* add MulAdd constaint, as it conflicts with Fixed::mul_add
        #[cfg(all(
            feature = "arbitrary",
            feature = "borsh",
            feature = "num-traits",
            feature = "serde"
        ))]
        doc_comment! {
            $comment;
            pub trait FixedOptionalFeatures: Sealed
            where
                Self: for<'a> Arbitrary<'a>,
                Self: BorshSerialize + BorshDeserialize,
                Self: Zero + Bounded + Inv,
                Self: CheckedAdd + CheckedSub + CheckedNeg + CheckedMul,
                Self: CheckedDiv + CheckedRem + CheckedShl + CheckedShr,
                Self: SaturatingAdd + SaturatingSub + SaturatingMul,
                Self: WrappingAdd + WrappingSub + WrappingNeg + WrappingMul,
                Self: WrappingShl + WrappingShr,
                Self: OverflowingAdd + OverflowingSub + OverflowingMul,
                Self: ToPrimitive + FromPrimitive + FloatConst,
                Self: Serialize + for<'de> Deserialize<'de>,
            {
            }
        }
    };
}

comment_features! {
    r#"This trait is used to provide supertraits to the [`Fixed`] trait
depending on the crate’s [optional features], and should not be used directly.

 1. If the `arbitrary` feature is enabled, [`Arbitrary`] is a supertrait of
    [`Fixed`].

 2. If the `borsh` experimental feature is enabled, [`BorshSerialize`] and
    [`BorshDeserialize`] are supertraits of [`Fixed`].

 3. If the `num-traits` experimental feature is enabled, the following
    are supertraits of [`Fixed`]:

      * [`Zero`]
      * [`Bounded`]
      * [`Inv`]
      * [`CheckedAdd`], [`CheckedSub`], [`CheckedNeg`],
        [`CheckedMul`], [`CheckedDiv`], [`CheckedRem`],
        [`CheckedShl`], [`CheckedShr`]
      * [`SaturatingAdd`], [`SaturatingSub`], [`SaturatingMul`]
      * [`WrappingAdd`], [`WrappingSub`], [`WrappingNeg`],
        [`WrappingMul`], [`WrappingShl`], [`WrappingShr`]
      * [`OverflowingAdd`], [`OverflowingSub`], [`OverflowingMul`]
      * [`ToPrimitive`], [`FromPrimitive`]
      * [`FloatConst`]

    The following are *not* supertraits of [`Fixed`], even though they
    are implemented for fixed-point numbers where applicable:

      * [`One`] because not all fixed-point numbers can represent the
        value 1
      * [`Num`] because it has [`One`] as a supertrait
      * [`MulAdd`], [`MulAddAssign`] because
        <code>[MulAdd][`MulAdd`]::[mul\_add][`mul_add`]</code>
        conflicts with
        <code>[Fixed]::[mul\_add][Fixed::mul_add]</code>

    Similarly, [`Signed`] and [`Unsigned`] are *not* supertraits of
    [`FixedSigned`] and [`FixedUnsigned`] because they have [`Num`] as
    a supertrait.

 4. If the `serde` feature is enabled, [`Serialize`] and
    [`Deserialize`] are supertraits of [`Fixed`].

[`MulAddAssign`]: num_traits::ops::mul_add::MulAddAssign
[`MulAdd`]: num_traits::ops::mul_add::MulAdd
[`Num`]: num_traits::Num
[`One`]: num_traits::identities::One
[`Signed`]: num_traits::sign::Signed
[`Unsigned`]: num_traits::sign::Unsigned
[`mul_add`]: num_traits::ops::mul_add::MulAdd::mul_add
[optional features]: crate#optional-features
"#
}

/// This trait provides methods common to all fixed-point numbers.
///
/// It can be helpful when writing generic code that makes use of
/// fixed-point numbers. For methods only available on signed
/// fixed-point numbers, use the [`FixedSigned`] trait instead, and
/// for methods only available on unsigned fixed-point numbers, use
/// [`FixedUnsigned`].
///
/// This trait is sealed and cannot be implemented for more types; it
/// is implemented for [`FixedI8`], [`FixedI16`], [`FixedI32`],
/// [`FixedI64`], [`FixedI128`], [`FixedU8`], [`FixedU16`],
/// [`FixedU32`], [`FixedU64`], and [`FixedU128`].
///
/// # Examples
///
/// ```rust
/// use fixed::{
///     traits::Fixed,
///     types::{I8F8, I16F16},
/// };
///
/// fn checked_add_twice<F: Fixed>(lhs: F, rhs: F) -> Option<F> {
///     lhs.checked_add(rhs)?.checked_add(rhs)
/// }
///
/// let val1 = checked_add_twice(I8F8::from_num(5), Fixed::from_num(1.75));
/// assert_eq!(val1, Some(Fixed::from_num(8.5)));
/// // can use with different fixed-point type
/// let val2 = checked_add_twice(I16F16::from_num(5), Fixed::from_num(1.75));
/// assert_eq!(val2, Some(Fixed::from_num(8.5)));
/// ```
///
/// The following example fails to compile, since the compiler cannot
/// infer that 500 in the `checked_mul_int` call is of type `F::Bits`.
///
/// ```rust,compile_fail
/// use fixed::traits::Fixed;
///
/// fn checked_add_times_500<F: Fixed>(lhs: F, rhs: F) -> Option<F> {
///     rhs.checked_mul_int(500)?.checked_add(lhs)
/// }
/// ```
///
/// One way to fix this is to add a trait bound indicating that any
/// [`u16`] (which can represent 500) can be converted into `F::Bits`.
///
/// ```rust
/// use fixed::{traits::Fixed, types::U12F4};
///
/// fn checked_add_times_500<F: Fixed>(lhs: F, rhs: F) -> Option<F>
/// where
///     u16: Into<F::Bits>,
/// {
///     rhs.checked_mul_int(500.into())?.checked_add(lhs)
/// }
///
/// let val = checked_add_times_500(U12F4::from_num(0.25), Fixed::from_num(1.5));
/// assert_eq!(val, Some(Fixed::from_num(750.25)));
/// ```
///
/// While this works in most cases, [`u16`] cannot be converted to
/// [`i16`], even if the value 500 does fit in [`i16`], so that the
/// following example would fail to compile.
///
/// ```rust,compile_fail
/// use fixed::{traits::Fixed, types::I12F4};
///
/// fn checked_add_times_500<F: Fixed>(lhs: F, rhs: F) -> Option<F>
/// where
///     u16: Into<F::Bits>,
/// {
///     rhs.checked_mul_int(500.into())?.checked_add(lhs)
/// }
///
/// // I12F4::Bits is i16, and u16 does not implement Into<i16>
/// let val = checked_add_times_500(I12F4::from_num(0.25), Fixed::from_num(1.5));
/// # let _ = val;
/// ```
///
/// We can use [`TryFrom`] to fix this, as we know that
/// `F::Bits::try_from(500_u16)` will work for both [`u16`] and
/// [`i16`]. (The function will always return [`None`] when `F::Bits`
/// is [`u8`] or [`i8`].)
///
/// ```rust
/// use fixed::{traits::Fixed, types::I12F4};
/// use core::convert::TryInto;
///
/// fn checked_add_times_500<F: Fixed>(lhs: F, rhs: F) -> Option<F>
/// where
///     u16: TryInto<F::Bits>,
/// {
///     rhs.checked_mul_int(500.try_into().ok()?)?.checked_add(lhs)
/// }
///
/// let val = checked_add_times_500(I12F4::from_num(0.25), Fixed::from_num(1.5));
/// assert_eq!(val, Some(Fixed::from_num(750.25)));
/// ```
///
/// [`TryFrom`]: core::convert::TryFrom
pub trait Fixed
where
    Self: Default + Hash + Ord,
    Self: Pod + TransparentWrapper<<Self as Fixed>::Bits>,
    Self: Debug + Display + Binary + Octal + LowerHex + UpperHex,
    Self: FromStr<Err = ParseFixedError>,
    Self: FromFixed + ToFixed,
    Self: Add<Output = Self> + AddAssign,
    Self: Sub<Output = Self> + SubAssign,
    Self: Mul<Output = Self> + MulAssign,
    Self: Div<Output = Self> + DivAssign,
    Self: Rem<Output = Self> + RemAssign,
    Self: Mul<<Self as Fixed>::Bits, Output = Self> + MulAssign<<Self as Fixed>::Bits>,
    Self: Div<<Self as Fixed>::Bits, Output = Self> + DivAssign<<Self as Fixed>::Bits>,
    Self: Rem<<Self as Fixed>::Bits, Output = Self> + RemAssign<<Self as Fixed>::Bits>,
    Self: Rem<<Self as Fixed>::NonZeroBits, Output = Self>,
    Self: RemAssign<<Self as Fixed>::NonZeroBits>,
    Self: Not<Output = Self>,
    Self: BitAnd<Output = Self> + BitAndAssign,
    Self: BitOr<Output = Self> + BitOrAssign,
    Self: BitXor<Output = Self> + BitXorAssign,
    Self: Shl<u32, Output = Self> + ShlAssign<u32>,
    Self: Shr<u32, Output = Self> + ShrAssign<u32>,
    Self: Sum + Product,
    Self: PartialOrd<i8> + PartialOrd<i16> + PartialOrd<i32>,
    Self: PartialOrd<i64> + PartialOrd<i128> + PartialOrd<isize>,
    Self: PartialOrd<u8> + PartialOrd<u16> + PartialOrd<u32>,
    Self: PartialOrd<u64> + PartialOrd<u128> + PartialOrd<usize>,
    Self: PartialOrd<f16> + PartialOrd<bf16>,
    Self: PartialOrd<f32> + PartialOrd<f64>,
    Self: PartialOrd<F128Bits>,
    Self: FixedOptionalFeatures,
    Self: Sealed,
{
    /// The primitive integer underlying type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{traits::Fixed, types::I16F16};
    /// // 32-bit DELTA is 0x0000_0001_i32
    /// const DELTA_BITS: <I16F16 as Fixed>::Bits = I16F16::DELTA.to_bits();
    /// assert_eq!(DELTA_BITS, 1i32);
    /// ```
    type Bits;

    /// The non-zero wrapped version of [`Bits`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{traits::Fixed, types::I16F16};
    /// let val = I16F16::from_num(31);
    /// let non_zero_5 = <I16F16 as Fixed>::NonZeroBits::new(5).unwrap();
    /// assert_eq!(val % non_zero_5, val % 5);
    /// ```
    ///
    /// [`Bits`]: Fixed::Bits
    type NonZeroBits;

    /// A byte array with the same size as the type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{traits::Fixed, types::I16F16};
    /// // 32-bit DELTA is 0x0000_0001_i32
    /// const DELTA_LE_BYTES: <I16F16 as Fixed>::Bytes = I16F16::DELTA.to_le_bytes();
    /// assert_eq!(DELTA_LE_BYTES, 1i32.to_le_bytes());
    /// ```
    type Bytes;

    /// The number of fractional bits as a compile-time [`Unsigned`] as provided
    /// by the [*typenum* crate].
    ///
    /// <code>\<F as [Fixed]>::Frac::[U32]</code> is equivalent to
    /// <code>\<F as [Fixed]>::[FRAC\_NBITS]</code>.
    ///
    /// `Frac` can be used as the generic parameter of fixed-point number types.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{traits::Fixed, types::extra::U16, FixedI32, FixedI64};
    /// type Fix1 = FixedI32::<U16>;
    /// assert_eq!(Fix1::FRAC_NBITS, 16);
    /// assert_eq!(Fix1::INT_NBITS, 32 - 16);
    /// type Fix2 = FixedI64::<<Fix1 as Fixed>::Frac>;
    /// assert_eq!(Fix2::FRAC_NBITS, 16);
    /// assert_eq!(Fix2::INT_NBITS, 64 - 16);
    /// ```
    ///
    /// [*typenum* crate]: https://crates.io/crates/typenum
    /// [U32]: crate::types::extra::Unsigned::U32
    /// [FRAC\_NBITS]: Fixed::FRAC_NBITS
    type Frac: Unsigned;

    /// An unsigned fixed-point number type with the same number of integer and
    /// fractional bits as `Self`.
    ///
    /// If `Self` is signed, then `Self::Signed` is the same as `Self`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{
    ///     traits::Fixed,
    ///     types::{I16F16, U16F16},
    /// };
    /// // I16F16::Signed is I16F16
    /// assert_eq!(<I16F16 as Fixed>::Signed::FRAC_NBITS, I16F16::FRAC_NBITS);
    /// assert_eq!(<I16F16 as Fixed>::Signed::INT_NBITS, I16F16::INT_NBITS);
    /// assert_eq!(<I16F16 as Fixed>::Signed::IS_SIGNED, I16F16::IS_SIGNED);
    /// // U16F16::Signed is I16F16
    /// assert_eq!(<U16F16 as Fixed>::Signed::FRAC_NBITS, I16F16::FRAC_NBITS);
    /// assert_eq!(<U16F16 as Fixed>::Signed::INT_NBITS, I16F16::INT_NBITS);
    /// assert_eq!(<U16F16 as Fixed>::Signed::IS_SIGNED, I16F16::IS_SIGNED);
    /// ```
    ///
    /// [I16F16]: crate::types::I16F16
    /// [U16F16]: crate::types::U16F16
    /// [types]: crate::types
    type Signed: FixedSigned;

    /// An unsigned fixed-point number type with the same number of integer and
    /// fractional bits as `Self`.
    ///
    /// If `Self` is unsigned, then `Self::Unsigned` is the same as `Self`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{
    ///     traits::Fixed,
    ///     types::{I16F16, U16F16},
    /// };
    /// // I16F16::Unsigned is U16F16
    /// assert_eq!(<I16F16 as Fixed>::Unsigned::FRAC_NBITS, U16F16::FRAC_NBITS);
    /// assert_eq!(<I16F16 as Fixed>::Unsigned::INT_NBITS, U16F16::INT_NBITS);
    /// assert_eq!(<I16F16 as Fixed>::Unsigned::IS_SIGNED, U16F16::IS_SIGNED);
    /// // U16F16::Unsigned is U16F16
    /// assert_eq!(<U16F16 as Fixed>::Unsigned::FRAC_NBITS, U16F16::FRAC_NBITS);
    /// assert_eq!(<U16F16 as Fixed>::Unsigned::INT_NBITS, U16F16::INT_NBITS);
    /// assert_eq!(<U16F16 as Fixed>::Unsigned::IS_SIGNED, U16F16::IS_SIGNED);
    /// ```
    ///
    /// [I16F16]: crate::types::I16F16
    /// [U16F16]: crate::types::U16F16
    /// [types]: crate::types
    type Unsigned: FixedUnsigned;

    /// Returns a reference to `self` as [`FixedSigned`] if the type is signed,
    /// or [`None`] if it is unsigned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{
    ///     traits::Fixed,
    ///     types::{I16F16, U16F16},
    /// };
    ///
    /// let i = I16F16::from_num(-3.5);
    /// match i.get_signed() {
    ///     Some(signed) => assert_eq!(signed.signum(), -1),
    ///     None => unreachable!(),
    /// }
    ///
    /// let u = U16F16::from_num(3.5);
    /// assert!(u.get_signed().is_none());
    /// ```
    #[inline]
    fn get_signed(&self) -> Option<&Self::Signed> {
        if Self::IS_SIGNED {
            Some(bytemuck::cast_ref(self))
        } else {
            None
        }
    }

    /// Returns a reference to `self` as [`FixedUnsigned`] if the type is
    /// unsigned, or [`None`] if it is signed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{
    ///     traits::Fixed,
    ///     types::{I16F16, U16F16},
    /// };
    ///
    /// let u = U16F16::from_num(3.5);
    /// match u.get_unsigned() {
    ///     Some(unsigned) => assert_eq!(unsigned.next_power_of_two(), 4),
    ///     None => unreachable!(),
    /// }
    ///
    /// let i = I16F16::from_num(3.5);
    /// assert!(i.get_unsigned().is_none());
    /// ```
    #[inline]
    fn get_unsigned(&self) -> Option<&Self::Unsigned> {
        if Self::IS_SIGNED {
            None
        } else {
            Some(bytemuck::cast_ref(self))
        }
    }

    /// Returns a mutable reference to `self` as [`FixedSigned`] if the type is
    /// signed, or [`None`] if it is unsigned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{
    ///     traits::Fixed,
    ///     types::{I16F16, U16F16},
    /// };
    ///
    /// let mut i = I16F16::from_num(-3.5);
    /// match i.get_signed_mut() {
    ///     Some(signed) => *signed = signed.signum(),
    ///     None => unreachable!(),
    /// }
    /// assert_eq!(i, -1);
    ///
    /// let mut u = U16F16::from_num(3.5);
    /// assert!(u.get_signed_mut().is_none());
    /// ```
    #[inline]
    fn get_signed_mut(&mut self) -> Option<&mut Self::Signed> {
        if Self::IS_SIGNED {
            Some(bytemuck::cast_mut(self))
        } else {
            None
        }
    }

    /// Returns a mutable reference to `self` as [`FixedUnsigned`] if the type
    /// is unsigned, or [`None`] if it is signed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{
    ///     traits::Fixed,
    ///     types::{I16F16, U16F16},
    /// };
    ///
    /// let mut u = U16F16::from_num(3.5);
    /// match u.get_unsigned_mut() {
    ///     Some(unsigned) => *unsigned = unsigned.next_power_of_two(),
    ///     None => unreachable!(),
    /// }
    /// assert_eq!(u, 4);
    ///
    /// let mut i = I16F16::from_num(3.5);
    /// assert!(i.get_unsigned_mut().is_none());
    /// ```
    #[inline]
    fn get_unsigned_mut(&mut self) -> Option<&mut Self::Unsigned> {
        if Self::IS_SIGNED {
            None
        } else {
            Some(bytemuck::cast_mut(self))
        }
    }

    /// Zero.
    ///
    /// See also <code>FixedI32::[ZERO][FixedI32::ZERO]</code> and
    /// <code>FixedU32::[ZERO][FixedU32::ZERO]</code>.
    const ZERO: Self;

    /// The difference between any two successive representable numbers, <i>Δ</i>.
    ///
    /// See also <code>FixedI32::[DELTA][FixedI32::DELTA]</code> and
    /// <code>FixedU32::[DELTA][FixedU32::DELTA]</code>.
    const DELTA: Self;

    /// The smallest value that can be represented.
    ///
    /// See also <code>FixedI32::[MIN][FixedI32::MIN]</code> and
    /// <code>FixedU32::[MIN][FixedU32::MIN]</code>.
    const MIN: Self;

    /// The largest value that can be represented.
    ///
    /// See also <code>FixedI32::[MAX][FixedI32::MAX]</code> and
    /// <code>FixedU32::[MAX][FixedU32::MAX]</code>.
    const MAX: Self;

    /// [`true`] if the type is signed.
    ///
    /// See also <code>FixedI32::[IS\_SIGNED][FixedI32::IS_SIGNED]</code> and
    /// <code>FixedU32::[IS\_SIGNED][FixedU32::IS_SIGNED]</code>.
    const IS_SIGNED: bool;

    /// The number of integer bits.
    ///
    /// See also <code>FixedI32::[INT\_NBITS][FixedI32::INT_NBITS]</code> and
    /// <code>FixedU32::[INT\_NBITS][FixedU32::INT_NBITS]</code>.
    const INT_NBITS: u32;

    /// The number of fractional bits.
    ///
    /// See also <code>FixedI32::[FRAC\_NBITS][FixedI32::FRAC_NBITS]</code> and
    /// <code>FixedU32::[FRAC\_NBITS][FixedU32::FRAC_NBITS]</code>.
    const FRAC_NBITS: u32;

    /// Creates a fixed-point number that has a bitwise representation
    /// identical to the given integer.
    ///
    /// See also <code>FixedI32::[from\_bits][FixedI32::from_bits]</code> and
    /// <code>FixedU32::[from\_bits][FixedU32::from_bits]</code>.
    fn from_bits(bits: Self::Bits) -> Self;

    /// Creates an integer that has a bitwise representation identical
    /// to the given fixed-point number.
    ///
    /// See also <code>FixedI32::[to\_bits][FixedI32::to_bits]</code> and
    /// <code>FixedU32::[to\_bits][FixedU32::to_bits]</code>.
    fn to_bits(self) -> Self::Bits;

    /// Converts a fixed-point number from big endian to the target’s endianness.
    ///
    /// See also <code>FixedI32::[from\_be][FixedI32::from_be]</code> and
    /// <code>FixedU32::[from\_be][FixedU32::from_be]</code>.
    fn from_be(fixed: Self) -> Self;

    /// Converts a fixed-point number from little endian to the target’s endianness.
    ///
    /// See also <code>FixedI32::[from\_le][FixedI32::from_le]</code> and
    /// <code>FixedU32::[from\_le][FixedU32::from_le]</code>.
    fn from_le(fixed: Self) -> Self;

    /// Converts this fixed-point number to big endian from the target’s endianness.
    ///
    /// See also <code>FixedI32::[to\_be][FixedI32::to_be]</code> and
    /// <code>FixedU32::[to\_be][FixedU32::to_be]</code>.
    fn to_be(self) -> Self;

    /// Converts this fixed-point number to little endian from the target’s endianness.
    ///
    /// See also <code>FixedI32::[to\_le][FixedI32::to_le]</code> and
    /// <code>FixedU32::[to\_le][FixedU32::to_le]</code>.
    fn to_le(self) -> Self;

    ///Reverses the byte order of the fixed-point number.
    ///
    /// See also <code>FixedI32::[swap\_bytes][FixedI32::swap_bytes]</code> and
    /// <code>FixedU32::[swap\_bytes][FixedU32::swap_bytes]</code>.
    fn swap_bytes(self) -> Self;

    /// Creates a fixed-point number from its representation as a byte
    /// array in big endian.
    ///
    /// See also
    /// <code>FixedI32::[from\_be\_bytes][FixedI32::from_be_bytes]</code> and
    /// <code>FixedU32::[from\_be\_bytes][FixedU32::from_be_bytes]</code>.
    fn from_be_bytes(bytes: Self::Bytes) -> Self;

    /// Creates a fixed-point number from its representation as a byte
    /// array in little endian.
    ///
    /// See also
    /// <code>FixedI32::[from\_le\_bytes][FixedI32::from_le_bytes]</code> and
    /// <code>FixedU32::[from\_le\_bytes][FixedU32::from_le_bytes]</code>.
    fn from_le_bytes(bytes: Self::Bytes) -> Self;

    /// Creates a fixed-point number from its representation as a byte
    /// array in native endian.
    ///
    /// See also
    /// <code>FixedI32::[from\_ne\_bytes][FixedI32::from_ne_bytes]</code> and
    /// <code>FixedU32::[from\_ne\_bytes][FixedU32::from_ne_bytes]</code>.
    fn from_ne_bytes(bytes: Self::Bytes) -> Self;

    /// Returns the memory representation of this fixed-point number
    /// as a byte array in big-endian byte order.
    ///
    /// See also <code>FixedI32::[to\_be\_bytes][FixedI32::to_be_bytes]</code>
    /// and <code>FixedU32::[to\_be\_bytes][FixedU32::to_be_bytes]</code>.
    fn to_be_bytes(self) -> Self::Bytes;

    /// Returns the memory representation of this fixed-point number
    /// as a byte array in little-endian byte order.
    ///
    /// See also <code>FixedI32::[to\_le\_bytes][FixedI32::to_le_bytes]</code>
    /// and <code>FixedU32::[to\_le\_bytes][FixedU32::to_le_bytes]</code>.
    fn to_le_bytes(self) -> Self::Bytes;

    /// Returns the memory representation of this fixed-point number
    /// as a byte array in native byte order.
    ///
    /// See also <code>FixedI32::[to\_ne\_bytes][FixedI32::to_ne_bytes]</code>
    /// and <code>FixedU32::[to\_ne\_bytes][FixedU32::to_ne_bytes]</code>.
    fn to_ne_bytes(self) -> Self::Bytes;

    /// Creates a fixed-point number from another number.
    ///
    /// Returns the same value as
    /// <code>src.[to\_fixed][ToFixed::to_fixed]\()</code>.
    ///
    /// See also <code>FixedI32::[from\_num][FixedI32::from_num]</code> and
    /// <code>FixedU32::[from\_num][FixedU32::from_num]</code>.
    fn from_num<Src: ToFixed>(src: Src) -> Self;

    /// Converts a fixed-point number to another number.
    ///
    /// Returns the same value as
    /// <code>Dst::[from\_fixed][FromFixed::from_fixed]\(self)</code>.
    ///
    /// See also <code>FixedI32::[to\_num][FixedI32::to_num]</code> and
    /// <code>FixedU32::[to\_num][FixedU32::to_num]</code>.
    fn to_num<Dst: FromFixed>(self) -> Dst;

    /// Creates a fixed-point number from another number if it fits,
    /// otherwise returns [`None`].
    ///
    /// Returns the same value as
    /// <code>src.[checked\_to\_fixed][ToFixed::checked_to_fixed]\()</code>.
    ///
    /// See also
    /// <code>FixedI32::[checked\_from\_num][FixedI32::checked_from_num]</code>
    /// and
    /// <code>FixedU32::[checked\_from\_num][FixedU32::checked_from_num]</code>.
    fn checked_from_num<Src: ToFixed>(src: Src) -> Option<Self>;

    /// Converts a fixed-point number to another number if it fits,
    /// otherwise returns [`None`].
    ///
    /// Returns the same value as
    /// <code>Dst::[checked\_from\_fixed][FromFixed::checked_from_fixed]\(self)</code>.
    ///
    /// See also
    /// <code>FixedI32::[checked\_to\_num][FixedI32::checked_to_num]</code> and
    /// <code>FixedU32::[checked\_to\_num][FixedU32::checked_to_num]</code>.
    fn checked_to_num<Dst: FromFixed>(self) -> Option<Dst>;

    /// Creates a fixed-point number from another number, saturating the
    /// value if it does not fit.
    ///
    /// Returns the same value as
    /// <code>src.[saturating\_to\_fixed][ToFixed::saturating_to_fixed]\()</code>.
    ///
    /// See also
    /// <code>FixedI32::[saturating\_from\_num][FixedI32::saturating_from_num]</code>
    /// and
    /// <code>FixedU32::[saturating\_from\_num][FixedU32::saturating_from_num]</code>.
    fn saturating_from_num<Src: ToFixed>(src: Src) -> Self;

    /// Converts a fixed-point number to another number, saturating the
    /// value if it does not fit.
    ///
    /// Returns the same value as
    /// <code>Dst::[saturating\_from\_fixed][FromFixed::saturating_from_fixed]\(self)</code>.
    ///
    /// See also
    /// <code>FixedI32::[saturating\_to\_num][FixedI32::saturating_to_num]</code>
    /// and
    /// <code>FixedU32::[saturating\_to\_num][FixedU32::saturating_to_num]</code>.
    fn saturating_to_num<Dst: FromFixed>(self) -> Dst;

    /// Creates a fixed-point number from another number, wrapping the
    /// value on overflow.
    ///
    /// Returns the same value as
    /// <code>src.[wrapping\_to\_fixed][ToFixed::wrapping_to_fixed]\()</code>.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_from\_num][FixedI32::wrapping_from_num]</code>
    /// and
    /// <code>FixedU32::[wrapping\_from\_num][FixedU32::wrapping_from_num]</code>.
    fn wrapping_from_num<Src: ToFixed>(src: Src) -> Self;

    /// Converts a fixed-point number to another number, wrapping the
    /// value on overflow.
    ///
    /// Returns the same value as
    /// <code>Dst::[wrapping\_from\_fixed][FromFixed::wrapping_from_fixed]\(self)</code>.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_to\_num][FixedI32::wrapping_to_num]</code>
    /// and
    /// <code>FixedU32::[wrapping\_to\_num][FixedU32::wrapping_to_num]</code>.
    fn wrapping_to_num<Dst: FromFixed>(self) -> Dst;

    /// Creates a fixed-point number from another number, panicking on overflow.
    ///
    /// Returns the same value as
    /// <code>src.[unwrapped\_to\_fixed][ToFixed::unwrapped_to_fixed]\()</code>.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_from\_num][FixedI32::unwrapped_from_num]</code>
    /// and
    /// <code>FixedU32::[unwrapped\_from\_num][FixedU32::unwrapped_from_num]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the value does not fit.
    #[track_caller]
    fn unwrapped_from_num<Src: ToFixed>(src: Src) -> Self;

    /// Converts a fixed-point number to another number, panicking on overflow.
    ///
    /// Returns the same value as
    /// <code>Dst::[unwrapped\_from\_fixed][FromFixed::unwrapped_from_fixed]\(self)</code>.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_to\_num][FixedI32::unwrapped_to_num]</code>
    /// and
    /// <code>FixedU32::[unwrapped\_to\_num][FixedU32::unwrapped_to_num]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the value does not fit.
    #[track_caller]
    fn unwrapped_to_num<Dst: FromFixed>(self) -> Dst;

    /// Creates a fixed-point number from another number.
    ///
    /// Returns the same value as
    /// <code>src.[overflowing\_to\_fixed][ToFixed::overflowing_to_fixed]\()</code>.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_from\_num][FixedI32::overflowing_from_num]</code>
    /// and
    /// <code>FixedU32::[overflowing\_from\_num][FixedU32::overflowing_from_num]</code>.
    fn overflowing_from_num<Src: ToFixed>(src: Src) -> (Self, bool);

    /// Converts a fixed-point number to another number.
    ///
    /// Returns the same value as
    /// <code>Dst::[overflowing\_from\_fixed][FromFixed::overflowing_from_fixed]\(self)</code>.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_to\_num][FixedI32::overflowing_to_num]</code>
    /// and
    /// <code>FixedU32::[overflowing\_to\_num][FixedU32::overflowing_to_num]</code>.
    fn overflowing_to_num<Dst: FromFixed>(self) -> (Dst, bool);

    /// Parses a string slice containing binary digits to return a fixed-point number.
    ///
    /// Rounding is to the nearest, with ties rounded to even.
    ///
    /// See also
    /// <code>FixedI32::[from\_str\_binary][FixedI32::from_str_binary]</code>
    /// and
    /// <code>FixedU32::[from\_str\_binary][FixedU32::from_str_binary]</code>.
    fn from_str_binary(src: &str) -> Result<Self, ParseFixedError>;

    /// Parses a string slice containing octal digits to return a fixed-point number.
    ///
    /// Rounding is to the nearest, with ties rounded to even.
    ///
    /// See also
    /// <code>FixedI32::[from\_str\_octal][FixedI32::from_str_octal]</code> and
    /// <code>FixedU32::[from\_str\_octal][FixedU32::from_str_octal]</code>.
    fn from_str_octal(src: &str) -> Result<Self, ParseFixedError>;

    /// Parses a string slice containing hexadecimal digits to return a fixed-point number.
    ///
    /// Rounding is to the nearest, with ties rounded to even.
    ///
    /// See also <code>FixedI32::[from\_str\_hex][FixedI32::from_str_hex]</code>
    /// and <code>FixedU32::[from\_str\_hex][FixedU32::from_str_hex]</code>.
    fn from_str_hex(src: &str) -> Result<Self, ParseFixedError>;

    /// Parses a string slice containing decimal digits to return a
    /// fixed-point number, saturating on overflow.
    ///
    /// Rounding is to the nearest, with ties rounded to even.
    ///
    /// See also
    /// <code>FixedI32::[saturating\_from\_str][FixedI32::saturating_from_str]</code>
    /// and
    /// <code>FixedU32::[saturating\_from\_str][FixedU32::saturating_from_str]</code>.
    fn saturating_from_str(src: &str) -> Result<Self, ParseFixedError>;

    /// Parses a string slice containing binary digits to return a
    /// fixed-point number, saturating on overflow.
    ///
    /// Rounding is to the nearest, with ties rounded to even.
    ///
    /// See also
    /// <code>FixedI32::[saturating\_from\_str\_binary][FixedI32::saturating_from_str_binary]</code>
    /// and
    /// <code>FixedU32::[saturating\_from\_str\_binary][FixedU32::saturating_from_str_binary]</code>.
    fn saturating_from_str_binary(src: &str) -> Result<Self, ParseFixedError>;

    /// Parses a string slice containing octal digits to return a
    /// fixed-point number, saturating on overflow.
    ///
    /// Rounding is to the nearest, with ties rounded to even.
    ///
    /// See also
    /// <code>FixedI32::[saturating\_from\_str\_octal][FixedI32::saturating_from_str_octal]</code>
    /// and
    /// <code>FixedU32::[saturating\_from\_str\_octal][FixedU32::saturating_from_str_octal]</code>.
    fn saturating_from_str_octal(src: &str) -> Result<Self, ParseFixedError>;

    /// Parses a string slice containing hexadecimal digits to return a
    /// fixed-point number, saturating on overflow.
    ///
    /// Rounding is to the nearest, with ties rounded to even.
    ///
    /// See also
    /// <code>FixedI32::[saturating\_from\_str\_hex][FixedI32::saturating_from_str_hex]</code>
    /// and
    /// <code>FixedU32::[saturating\_from\_str\_hex][FixedU32::saturating_from_str_hex]</code>.
    fn saturating_from_str_hex(src: &str) -> Result<Self, ParseFixedError>;

    /// Parses a string slice containing decimal digits to return a
    /// fixed-point number, wrapping on overflow.
    ///
    /// Rounding is to the nearest, with ties rounded to even.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_from\_str][FixedI32::wrapping_from_str]</code>
    /// and
    /// <code>FixedU32::[wrapping\_from\_str][FixedU32::wrapping_from_str]</code>.
    fn wrapping_from_str(src: &str) -> Result<Self, ParseFixedError>;

    /// Parses a string slice containing binary digits to return a
    /// fixed-point number, wrapping on overflow.
    ///
    /// Rounding is to the nearest, with ties rounded to even.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_from\_str\_binary][FixedI32::wrapping_from_str_binary]</code>
    /// and
    /// <code>FixedU32::[wrapping\_from\_str\_binary][FixedU32::wrapping_from_str_binary]</code>.
    fn wrapping_from_str_binary(src: &str) -> Result<Self, ParseFixedError>;

    /// Parses a string slice containing octal digits to return a
    /// fixed-point number, wrapping on overflow.
    ///
    /// Rounding is to the nearest, with ties rounded to even.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_from\_str\_octal][FixedI32::wrapping_from_str_octal]</code>
    /// and
    /// <code>FixedU32::[wrapping\_from\_str\_octal][FixedU32::wrapping_from_str_octal]</code>.
    fn wrapping_from_str_octal(src: &str) -> Result<Self, ParseFixedError>;

    /// Parses a string slice containing hexadecimal digits to return a
    /// fixed-point number, wrapping on overflow.
    ///
    /// Rounding is to the nearest, with ties rounded to even.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_from\_str\_hex][FixedI32::wrapping_from_str_hex]</code>
    /// and
    /// <code>FixedU32::[wrapping\_from\_str\_hex][FixedU32::wrapping_from_str_hex]</code>.
    fn wrapping_from_str_hex(src: &str) -> Result<Self, ParseFixedError>;

    /// Parses a string slice containing decimal digits to return a
    /// fixed-point number.
    ///
    /// Returns a [tuple] of the fixed-point number and a [`bool`],
    /// indicating whether an overflow has occurred. On overflow, the
    /// wrapped value is returned.
    ///
    /// Rounding is to the nearest, with ties rounded to even.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_from\_str][FixedI32::overflowing_from_str]</code>
    /// and
    /// <code>FixedU32::[overflowing\_from\_str][FixedU32::overflowing_from_str]</code>.
    fn overflowing_from_str(src: &str) -> Result<(Self, bool), ParseFixedError>;

    /// Parses a string slice containing binary digits to return a
    /// fixed-point number.
    ///
    /// Returns a [tuple] of the fixed-point number and a [`bool`],
    /// indicating whether an overflow has occurred. On overflow, the
    /// wrapped value is returned.
    ///
    /// Rounding is to the nearest, with ties rounded to even.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_from\_str\_binary][FixedI32::overflowing_from_str_binary]</code>
    /// and
    /// <code>FixedU32::[overflowing\_from\_str\_binary][FixedU32::overflowing_from_str_binary]</code>.
    fn overflowing_from_str_binary(src: &str) -> Result<(Self, bool), ParseFixedError>;

    /// Parses a string slice containing octal digits to return a
    /// fixed-point number.
    ///
    /// Returns a [tuple] of the fixed-point number and a [`bool`],
    /// indicating whether an overflow has occurred. On overflow, the
    /// wrapped value is returned.
    ///
    /// Rounding is to the nearest, with ties rounded to even.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_from\_str\_octal][FixedI32::overflowing_from_str_octal]</code>
    /// and
    /// <code>FixedU32::[overflowing\_from\_str\_octal][FixedU32::overflowing_from_str_octal]</code>.
    fn overflowing_from_str_octal(src: &str) -> Result<(Self, bool), ParseFixedError>;

    /// Parses a string slice containing hexadecimal digits to return a
    /// fixed-point number.
    ///
    /// Returns a [tuple] of the fixed-point number and a [`bool`],
    /// indicating whether an overflow has occurred. On overflow, the
    /// wrapped value is returned.
    ///
    /// Rounding is to the nearest, with ties rounded to even.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_from\_str\_hex][FixedI32::overflowing_from_str_hex]</code>
    /// and
    /// <code>FixedU32::[overflowing\_from\_str\_hex][FixedU32::overflowing_from_str_hex]</code>.
    fn overflowing_from_str_hex(src: &str) -> Result<(Self, bool), ParseFixedError>;

    /// Returns the integer part.
    ///
    /// See also <code>FixedI32::[int][FixedI32::int]</code> and
    /// <code>FixedU32::[int][FixedU32::int]</code>.
    fn int(self) -> Self;

    /// Returns the fractional part.
    ///
    /// See also <code>FixedI32::[frac][FixedI32::frac]</code> and
    /// <code>FixedU32::[frac][FixedU32::frac]</code>.
    fn frac(self) -> Self;

    /// Rounds to the next integer towards 0.
    ///
    /// See also
    /// <code>FixedI32::[round\_to\_zero][FixedI32::round_to_zero]</code> and
    /// <code>FixedU32::[round\_to\_zero][FixedU32::round_to_zero]</code>.
    fn round_to_zero(self) -> Self;

    /// Rounds to the next integer towards +∞.
    ///
    /// See also <code>FixedI32::[ceil][FixedI32::ceil]</code> and
    /// <code>FixedU32::[ceil][FixedU32::ceil]</code>.
    fn ceil(self) -> Self;

    /// Rounds to the next integer towards −∞.
    ///
    /// See also <code>FixedI32::[floor][FixedI32::floor]</code> and
    /// <code>FixedU32::[floor][FixedU32::floor]</code>.
    fn floor(self) -> Self;

    /// Rounds to the nearest integer, with ties rounded away from zero.
    ///
    /// See also <code>FixedI32::[round][FixedI32::round]</code> and
    /// <code>FixedU32::[round][FixedU32::round]</code>.
    fn round(self) -> Self;

    /// Rounds to the nearest integer, with ties rounded to even.
    ///
    /// See also
    /// <code>FixedI32::[round\_ties\_to\_even][FixedI32::round_ties_to_even]</code>
    /// and
    /// <code>FixedU32::[round\_ties\_to\_even][FixedU32::round_ties_to_even]</code>.
    fn round_ties_to_even(self) -> Self;

    /// Checked ceil. Rounds to the next integer towards +∞, returning
    /// [`None`] on overflow.
    ///
    /// See also <code>FixedI32::[checked\_ceil][FixedI32::checked_ceil]</code>
    /// and <code>FixedU32::[checked\_ceil][FixedU32::checked_ceil]</code>.
    fn checked_ceil(self) -> Option<Self>;

    /// Checked floor. Rounds to the next integer towards −∞, returning
    /// [`None`] on overflow.
    ///
    /// See also
    /// <code>FixedI32::[checked\_floor][FixedI32::checked_floor]</code> and
    /// <code>FixedU32::[checked\_floor][FixedU32::checked_floor]</code>.
    fn checked_floor(self) -> Option<Self>;

    /// Checked round. Rounds to the nearest integer, with ties
    /// rounded away from zero, returning [`None`] on overflow.
    ///
    /// See also
    /// <code>FixedI32::[checked\_round][FixedI32::checked_round]</code> and
    /// <code>FixedU32::[checked\_round][FixedU32::checked_round]</code>.
    fn checked_round(self) -> Option<Self>;

    /// Checked round. Rounds to the nearest integer, with ties
    /// rounded to even, returning [`None`] on overflow.
    ///
    /// See also
    /// <code>FixedI32::[checked\_round\_ties\_to\_even][FixedI32::checked_round_ties_to_even]</code>
    /// and
    /// <code>FixedU32::[checked\_round\_ties\_to\_even][FixedU32::checked_round_ties_to_even]</code>.
    fn checked_round_ties_to_even(self) -> Option<Self>;

    /// Saturating ceil. Rounds to the next integer towards +∞,
    /// saturating on overflow.
    ///
    /// See also
    /// <code>FixedI32::[saturating\_ceil][FixedI32::saturating_ceil]</code> and
    /// <code>FixedU32::[saturating\_ceil][FixedU32::saturating_ceil]</code>.
    fn saturating_ceil(self) -> Self;

    /// Saturating floor. Rounds to the next integer towards −∞,
    /// saturating on overflow.
    ///
    /// See also
    /// <code>FixedI32::[saturating\_floor][FixedI32::saturating_floor]</code>
    /// and
    /// <code>FixedU32::[saturating\_floor][FixedU32::saturating_floor]</code>.
    fn saturating_floor(self) -> Self;

    /// Saturating round. Rounds to the nearest integer, with ties
    /// rounded away from zero, and saturating on overflow.
    ///
    /// See also
    /// <code>FixedI32::[saturating\_round][FixedI32::saturating_round]</code>
    /// and
    /// <code>FixedU32::[saturating\_round][FixedU32::saturating_round]</code>.
    fn saturating_round(self) -> Self;

    /// Saturating round. Rounds to the nearest integer, with ties
    /// rounded to_even, and saturating on overflow.
    ///
    /// See also
    /// <code>FixedI32::[saturating\_round\_ties\_to\_even][FixedI32::saturating_round_ties_to_even]</code>
    /// and
    /// <code>FixedU32::[saturating\_round\_ties\_to\_even][FixedU32::saturating_round_ties_to_even]</code>.
    fn saturating_round_ties_to_even(self) -> Self;

    /// Wrapping ceil. Rounds to the next integer towards +∞, wrapping
    /// on overflow.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_ceil][FixedI32::wrapping_ceil]</code> and
    /// <code>FixedU32::[wrapping\_ceil][FixedU32::wrapping_ceil]</code>.
    fn wrapping_ceil(self) -> Self;

    /// Wrapping floor. Rounds to the next integer towards −∞,
    /// wrapping on overflow.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_floor][FixedI32::wrapping_floor]</code> and
    /// <code>FixedU32::[wrapping\_floor][FixedU32::wrapping_floor]</code>.
    fn wrapping_floor(self) -> Self;

    /// Wrapping round. Rounds to the next integer to the nearest,
    /// with ties rounded away from zero, and wrapping on overflow.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_round][FixedI32::wrapping_round]</code> and
    /// <code>FixedU32::[wrapping\_round][FixedU32::wrapping_round]</code>.
    fn wrapping_round(self) -> Self;

    /// Wrapping round. Rounds to the next integer to the nearest,
    /// with ties rounded to even, and wrapping on overflow.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_round\_ties\_to\_even][FixedI32::wrapping_round_ties_to_even]</code>
    /// and
    /// <code>FixedU32::[wrapping\_round\_ties\_to\_even][FixedU32::wrapping_round_ties_to_even]</code>.
    fn wrapping_round_ties_to_even(self) -> Self;

    /// Unwrapped ceil. Rounds to the next integer towards +∞,
    /// panicking on overflow.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_ceil][FixedI32::unwrapped_ceil]</code> and
    /// <code>FixedU32::[unwrapped\_ceil][FixedU32::unwrapped_ceil]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the result does not fit.
    #[track_caller]
    fn unwrapped_ceil(self) -> Self;

    /// Unwrapped floor. Rounds to the next integer towards −∞,
    /// panicking on overflow.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_floor][FixedI32::unwrapped_floor]</code> and
    /// <code>FixedU32::[unwrapped\_floor][FixedU32::unwrapped_floor]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the result does not fit.
    #[track_caller]
    fn unwrapped_floor(self) -> Self;

    /// Unwrapped round. Rounds to the next integer to the nearest,
    /// with ties rounded away from zero, and panicking on overflow.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_round][FixedI32::unwrapped_round]</code> and
    /// <code>FixedU32::[unwrapped\_round][FixedU32::unwrapped_round]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the result does not fit.
    #[track_caller]
    fn unwrapped_round(self) -> Self;

    /// Unwrapped round. Rounds to the next integer to the nearest,
    /// with ties rounded to even, and panicking on overflow.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_round\_ties\_to\_even][FixedI32::unwrapped_round_ties_to_even]</code>
    /// and
    /// <code>FixedU32::[unwrapped\_round\_ties\_to\_even][FixedU32::unwrapped_round_ties_to_even]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the result does not fit.
    #[track_caller]
    fn unwrapped_round_ties_to_even(self) -> Self;

    /// Overflowing ceil. Rounds to the next integer towards +∞.
    ///
    /// Returns a [tuple] of the fixed-point number and a [`bool`],
    /// indicating whether an overflow has occurred. On overflow, the
    /// wrapped value is returned.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_ceil][FixedI32::overflowing_ceil]</code>
    /// and
    /// <code>FixedU32::[overflowing\_ceil][FixedU32::overflowing_ceil]</code>.
    fn overflowing_ceil(self) -> (Self, bool);

    /// Overflowing floor. Rounds to the next integer towards −∞.
    ///
    /// Returns a [tuple] of the fixed-point number and a [`bool`],
    /// indicating whether an overflow has occurred. On overflow, the
    /// wrapped value is returned.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_floor][FixedI32::overflowing_floor]</code>
    /// and
    /// <code>FixedU32::[overflowing\_floor][FixedU32::overflowing_floor]</code>.
    fn overflowing_floor(self) -> (Self, bool);

    /// Overflowing round. Rounds to the next integer to the nearest,
    /// with ties rounded away from zero.
    ///
    /// Returns a [tuple] of the fixed-point number and a [`bool`],
    /// indicating whether an overflow has occurred. On overflow, the
    /// wrapped value is returned.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_round][FixedI32::overflowing_round]</code>
    /// and
    /// <code>FixedU32::[overflowing\_round][FixedU32::overflowing_round]</code>.
    fn overflowing_round(self) -> (Self, bool);

    /// Overflowing round. Rounds to the next integer to the nearest,
    /// with ties rounded to even.
    ///
    /// Returns a [tuple] of the fixed-point number and a [`bool`],
    /// indicating whether an overflow has occurred. On overflow, the
    /// wrapped value is returned.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_round\_ties\_to\_even][FixedI32::overflowing_round_ties_to_even]</code>
    /// and
    /// <code>FixedU32::[overflowing\_round\_ties\_to\_even][FixedU32::overflowing_round_ties_to_even]</code>.
    fn overflowing_round_ties_to_even(self) -> (Self, bool);

    /// Returns the number of ones in the binary representation.
    ///
    /// See also <code>FixedI32::[count\_ones][FixedI32::count_ones]</code> and
    /// <code>FixedU32::[count\_ones][FixedU32::count_ones]</code>.
    fn count_ones(self) -> u32;

    /// Returns the number of zeros in the binary representation.
    ///
    /// See also <code>FixedI32::[count\_zeros][FixedI32::count_zeros]</code>
    /// and <code>FixedU32::[count\_zeros][FixedU32::count_zeros]</code>.
    fn count_zeros(self) -> u32;

    /// Returns the number of leading ones in the binary representation.
    ///
    /// See also <code>FixedI32::[leading\_ones][FixedI32::leading_ones]</code>
    /// and <code>FixedU32::[leading\_ones][FixedU32::leading_ones]</code>.
    fn leading_ones(self) -> u32;

    /// Returns the number of leading zeros in the binary representation.
    ///
    /// See also
    /// <code>FixedI32::[leading\_zeros][FixedI32::leading_zeros]</code> and
    /// <code>FixedU32::[leading\_zeros][FixedU32::leading_zeros]</code>.
    fn leading_zeros(self) -> u32;

    /// Returns the number of trailing ones in the binary representation.
    ///
    /// See also
    /// <code>FixedI32::[trailing\_ones][FixedI32::trailing_ones]</code> and
    /// <code>FixedU32::[trailing\_ones][FixedU32::trailing_ones]</code>.
    fn trailing_ones(self) -> u32;

    /// Returns the number of trailing zeros in the binary representation.
    ///
    /// See also
    /// <code>FixedI32::[trailing\_zeros][FixedI32::trailing_zeros]</code> and
    /// <code>FixedU32::[trailing\_zeros][FixedU32::trailing_zeros]</code>.
    fn trailing_zeros(self) -> u32;

    /// Integer base-2 logarithm, rounded down.
    ///
    /// See also <code>FixedI32::[int\_log2][FixedI32::int_log2]</code> and
    /// <code>FixedU32::[int\_log2][FixedU32::int_log2]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the fixed-point number is ≤ 0.
    fn int_log2(self) -> i32;

    /// Integer base-10 logarithm, rounded down.
    ///
    /// See also <code>FixedI32::[int\_log10][FixedI32::int_log10]</code> and
    /// <code>FixedU32::[int\_log10][FixedU32::int_log10]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the fixed-point number is ≤ 0.
    fn int_log10(self) -> i32;

    /// Checked integer base-2 logarithm, rounded down. Returns the
    /// logarithm or [`None`] if the fixed-point number is ≤ 0.
    ///
    /// See also
    /// <code>FixedI32::[checked\_int\_log2][FixedI32::checked_int_log2]</code>
    /// and
    /// <code>FixedU32::[checked\_int\_log2][FixedU32::checked_int_log2]</code>.
    fn checked_int_log2(self) -> Option<i32>;

    /// Checked integer base-10 logarithm, rounded down. Returns the
    /// logarithm or [`None`] if the fixed-point number is ≤ 0.
    ///
    /// See also
    /// <code>FixedI32::[checked\_int\_log10][FixedI32::checked_int_log10]</code>
    /// and
    /// <code>FixedU32::[checked\_int\_log10][FixedU32::checked_int_log10]</code>.
    fn checked_int_log10(self) -> Option<i32>;

    /// Reverses the order of the bits of the fixed-point number.
    ///
    /// See also <code>FixedI32::[reverse\_bits][FixedI32::reverse_bits]</code>
    /// and <code>FixedU32::[reverse\_bits][FixedU32::reverse_bits]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn reverse_bits(self) -> Self;

    /// Shifts to the left by `n` bits, wrapping the truncated bits to the right end.
    ///
    /// See also <code>FixedI32::[rotate\_left][FixedI32::rotate_left]</code>
    /// and <code>FixedU32::[rotate\_left][FixedU32::rotate_left]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn rotate_left(self, n: u32) -> Self;

    /// Shifts to the right by `n` bits, wrapping the truncated bits to the left end.
    ///
    /// See also <code>FixedI32::[rotate\_right][FixedI32::rotate_right]</code>
    /// and <code>FixedU32::[rotate\_right][FixedU32::rotate_right]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn rotate_right(self, n: u32) -> Self;

    /// Returns [`true`] if the number is zero.
    ///
    /// See also <code>FixedI32::[is\_zero][FixedI32::is_zero]</code> and
    /// <code>FixedU32::[is\_zero][FixedU32::is_zero]</code>.
    fn is_zero(self) -> bool;

    /// Returns the distance from `self` to `other`.
    ///
    /// See also <code>FixedI32::[dist][FixedI32::dist]</code> and
    /// <code>FixedU32::[dist][FixedU32::dist]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn dist(self, other: Self) -> Self;

    /// Returns the mean of `self` and `other`.
    ///
    /// See also <code>FixedI32::[mean][FixedI32::mean]</code> and
    /// <code>FixedU32::[mean][FixedU32::mean]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn mean(self, other: Self) -> Self;

    /// Returns the reciprocal.
    ///
    /// See also <code>FixedI32::[recip][FixedI32::recip]</code> and
    /// <code>FixedU32::[recip][FixedU32::recip]</code>.
    ///
    /// # Panics
    ///
    /// Panics if `self` is zero.
    fn recip(self) -> Self;

    /// Multiply and add. Returns `self` × `mul` + `add`.
    ///
    /// Note that the inherent [`mul_add`] method is more flexible
    /// than this method and allows the `mul` parameter to have a
    /// fixed-point type like `self` but with a different number of
    /// fractional bits.
    ///
    /// [`mul_add`]: FixedI32::mul_add
    ///
    /// See also <code>FixedI32::[mul\_add][FixedI32::mul_add]</code> and
    /// <code>FixedU32::[mul\_add][FixedU32::mul_add]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn mul_add(self, mul: Self, add: Self) -> Self;

    /// Multiply and accumulate. Adds (`a` × `b`) to `self`.
    ///
    /// Note that the inherent [`mul_acc`] method is more flexible than this
    /// method and allows the `a` and `b` parameters to have a fixed-point type
    /// like `self` but with a different number of fractional bits.
    ///
    /// [`mul_acc`]: FixedI32::mul_acc
    ///
    /// See also <code>FixedI32::[mul\_acc][FixedI32::mul_acc]</code> and
    /// <code>FixedU32::[mul\_acc][FixedU32::mul_acc]</code>.
    fn mul_acc(&mut self, a: Self, b: Self);

    /// Euclidean division by an integer.
    ///
    /// See also <code>FixedI32::[div\_euclid][FixedI32::div_euclid]</code> and
    /// <code>FixedU32::[div\_euclid][FixedU32::div_euclid]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero or if the division results in overflow.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn div_euclid(self, rhs: Self) -> Self;

    /// Remainder for Euclidean division.
    ///
    /// See also <code>FixedI32::[rem\_euclid][FixedI32::rem_euclid]</code> and
    /// <code>FixedU32::[rem\_euclid][FixedU32::rem_euclid]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn rem_euclid(self, rhs: Self) -> Self;

    /// Euclidean division by an integer.
    ///
    /// See also
    /// <code>FixedI32::[div\_euclid\_int][FixedI32::div_euclid_int]</code> and
    /// <code>FixedU32::[div\_euclid\_int][FixedU32::div_euclid_int]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero or if the division results in overflow.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn div_euclid_int(self, rhs: Self::Bits) -> Self;

    /// Remainder for Euclidean division by an integer.
    ///
    /// See also
    /// <code>FixedI32::[rem\_euclid\_int][FixedI32::rem_euclid_int]</code> and
    /// <code>FixedU32::[rem\_euclid\_int][FixedU32::rem_euclid_int]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero or if the division results in overflow.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn rem_euclid_int(self, rhs: Self::Bits) -> Self;

    /// Linear interpolation between `start` and `end`.
    ///
    /// See also <code>FixedI32::[lerp][FixedI32::lerp]</code> and
    /// <code>FixedU32::[lerp][FixedU32::lerp]</code>.
    fn lerp(self, start: Self, end: Self) -> Self;

    /// Inverse linear interpolation between `start` and `end`.
    ///
    /// See also <code>FixedI32::[inv\_lerp][FixedI32::inv_lerp]</code> and
    /// <code>FixedU32::[inv\_lerp][FixedU32::inv_lerp]</code>.
    fn inv_lerp(self, start: Self, end: Self) -> Self;

    /// Checked negation. Returns the negated value, or [`None`] on overflow.
    ///
    /// See also <code>FixedI32::[checked\_neg][FixedI32::checked_neg]</code>
    /// and <code>FixedU32::[checked\_neg][FixedU32::checked_neg]</code>.
    fn checked_neg(self) -> Option<Self>;

    /// Checked addition. Returns the sum, or [`None`] on overflow.
    ///
    /// See also <code>FixedI32::[checked\_add][FixedI32::checked_add]</code>
    /// and <code>FixedU32::[checked\_add][FixedU32::checked_add]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn checked_add(self, rhs: Self) -> Option<Self>;

    /// Checked subtraction. Returns the difference, or [`None`] on overflow.
    ///
    /// See also <code>FixedI32::[checked\_sub][FixedI32::checked_sub]</code>
    /// and <code>FixedU32::[checked\_sub][FixedU32::checked_sub]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn checked_sub(self, rhs: Self) -> Option<Self>;

    /// Checked multiplication. Returns the product, or [`None`] on overflow.
    ///
    /// See also <code>FixedI32::[checked\_mul][FixedI32::checked_mul]</code>
    /// and <code>FixedU32::[checked\_mul][FixedU32::checked_mul]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn checked_mul(self, rhs: Self) -> Option<Self>;

    /// Checked division. Returns the quotient, or [`None`] if the
    /// divisor is zero or on overflow.
    ///
    /// See also <code>FixedI32::[checked\_div][FixedI32::checked_div]</code>
    /// and <code>FixedU32::[checked\_div][FixedU32::checked_div]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn checked_div(self, rhs: Self) -> Option<Self>;

    /// Checked remainder. Returns the remainder, or [`None`] if the
    /// divisor is zero.
    ///
    /// See also <code>FixedI32::[checked\_rem][FixedI32::checked_rem]</code>
    /// and <code>FixedU32::[checked\_rem][FixedU32::checked_rem]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn checked_rem(self, rhs: Self) -> Option<Self>;

    /// Checked reciprocal. Returns the reciprocal, or [`None`] if
    /// `self` is zero or on overflow.
    ///
    /// See also
    /// <code>FixedI32::[checked\_recip][FixedI32::checked_recip]</code> and
    /// <code>FixedU32::[checked\_recip][FixedU32::checked_recip]</code>.
    fn checked_recip(self) -> Option<Self>;

    /// Checked multiply and add. Returns `self` × `mul` + `add`, or [`None`] on overflow.
    ///
    /// See also
    /// <code>FixedI32::[checked\_mul\_add][FixedI32::checked_mul_add]</code>
    /// and
    /// <code>FixedU32::[checked\_mul\_add][FixedU32::checked_mul_add]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn checked_mul_add(self, mul: Self, add: Self) -> Option<Self>;

    /// Checked multiply and accumulate. Adds (`a` × `b`) to `self`, or returns
    /// [`None`] on overflow.
    ///
    /// See also
    /// <code>FixedI32::[checked\_mul\_acc][FixedI32::checked_mul_acc]</code>
    /// and
    /// <code>FixedU32::[checked\_mul\_acc][FixedU32::checked_mul_acc]</code>.
    #[must_use = "this `Option` may be a `None` variant indicating overflow, which should be handled"]
    fn checked_mul_acc(&mut self, a: Self, b: Self) -> Option<()>;

    /// Checked remainder for Euclidean division. Returns the
    /// remainder, or [`None`] if the divisor is zero or the division
    /// results in overflow.
    ///
    /// See also
    /// <code>FixedI32::[checked\_div\_euclid][FixedI32::checked_div_euclid]</code>
    /// and
    /// <code>FixedU32::[checked\_div\_euclid][FixedU32::checked_div_euclid]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn checked_div_euclid(self, rhs: Self) -> Option<Self>;

    /// Checked remainder for Euclidean division. Returns the
    /// remainder, or [`None`] if the divisor is zero.
    ///
    /// See also
    /// <code>FixedI32::[checked\_rem\_euclid][FixedI32::checked_rem_euclid]</code>
    /// and
    /// <code>FixedU32::[checked\_rem\_euclid][FixedU32::checked_rem_euclid]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn checked_rem_euclid(self, rhs: Self) -> Option<Self>;

    /// Checked multiplication by an integer. Returns the product, or
    /// [`None`] on overflow.
    ///
    /// See also
    /// <code>FixedI32::[checked\_mul\_int][FixedI32::checked_mul_int]</code>
    /// and
    /// <code>FixedU32::[checked\_mul\_int][FixedU32::checked_mul_int]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn checked_mul_int(self, rhs: Self::Bits) -> Option<Self>;

    /// Checked division by an integer. Returns the quotient, or
    /// [`None`] if the divisor is zero or if the division results in
    /// overflow.
    ///
    /// See also
    /// <code>FixedI32::[checked\_div\_int][FixedI32::checked_div_int]</code>
    /// and
    /// <code>FixedU32::[checked\_div\_int][FixedU32::checked_div_int]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn checked_div_int(self, rhs: Self::Bits) -> Option<Self>;

    /// Checked fixed-point remainder for division by an integer.
    /// Returns the remainder, or [`None`] if the divisor is zero or
    /// if the division results in overflow.
    ///
    /// See also
    /// <code>FixedI32::[checked\_rem\_int][FixedI32::checked_rem_int]</code>
    /// and
    /// <code>FixedU32::[checked\_rem\_int][FixedU32::checked_rem_int]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn checked_rem_int(self, rhs: Self::Bits) -> Option<Self>;

    /// Checked Euclidean division by an integer. Returns the
    /// quotient, or [`None`] if the divisor is zero or if the
    /// division results in overflow.
    ///
    /// See also
    /// <code>FixedI32::[checked\_div\_euclid\_int][FixedI32::checked_div_euclid_int]</code>
    /// and
    /// <code>FixedU32::[checked\_div\_euclid\_int][FixedU32::checked_div_euclid_int]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn checked_div_euclid_int(self, rhs: Self::Bits) -> Option<Self>;

    /// Checked remainder for Euclidean division by an integer.
    /// Returns the remainder, or [`None`] if the divisor is zero or
    /// if the remainder results in overflow.
    ///
    /// See also
    /// <code>FixedI32::[checked\_rem\_euclid\_int][FixedI32::checked_rem_euclid_int]</code>
    /// and
    /// <code>FixedU32::[checked\_rem\_euclid\_int][FixedU32::checked_rem_euclid_int]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn checked_rem_euclid_int(self, rhs: Self::Bits) -> Option<Self>;

    /// Checked shift left. Returns the shifted number, or [`None`] if
    /// `rhs` ≥ the number of bits.
    ///
    /// See also <code>FixedI32::[checked\_shl][FixedI32::checked_shl]</code>
    /// and <code>FixedU32::[checked\_shl][FixedU32::checked_shl]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn checked_shl(self, rhs: u32) -> Option<Self>;

    /// Checked shift right. Returns the shifted number, or [`None`]
    /// if `rhs` ≥ the number of bits.
    ///
    /// See also <code>FixedI32::[checked\_shr][FixedI32::checked_shr]</code>
    /// and <code>FixedU32::[checked\_shr][FixedU32::checked_shr]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn checked_shr(self, rhs: u32) -> Option<Self>;

    /// Checked distance. Returns the distance from `self` to `other`, or
    /// [`None`] on overflow.
    ///
    /// See also <code>FixedI32::[checked\_dist][FixedI32::checked_dist]</code>
    /// and <code>FixedU32::[checked\_dist][FixedU32::checked_dist]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn checked_dist(self, other: Self) -> Option<Self>;

    /// Checked linear interpolation between `start` and `end`. Returns [`None`]
    /// on overflow.
    ///
    /// See also <code>FixedI32::[checked\_lerp][FixedI32::checked_lerp]</code>
    /// and <code>FixedU32::[checked\_lerp][FixedU32::checked_lerp]</code>.
    fn checked_lerp(self, start: Self, end: Self) -> Option<Self>;

    /// Checked inverse linear interpolation between `start` and `end`. Returns
    /// [`None`] when `start` = `end` or on overflow.
    ///
    /// See also
    /// <code>FixedI32::[checked\_inv\_lerp][FixedI32::checked_inv_lerp]</code>
    /// and
    /// <code>FixedU32::[checked\_inv\_lerp][FixedU32::checked_inv_lerp]</code>.
    fn checked_inv_lerp(self, start: Self, end: Self) -> Option<Self>;

    /// Saturated negation. Returns the negated value, saturating on overflow.
    ///
    /// See also
    /// <code>FixedI32::[saturating\_neg][FixedI32::saturating_neg]</code> and
    /// <code>FixedU32::[saturating\_neg][FixedU32::saturating_neg]</code>.
    fn saturating_neg(self) -> Self;

    /// Saturating addition. Returns the sum, saturating on overflow.
    ///
    /// See also
    /// <code>FixedI32::[saturating\_add][FixedI32::saturating_add]</code> and
    /// <code>FixedU32::[saturating\_add][FixedU32::saturating_add]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn saturating_add(self, rhs: Self) -> Self;

    /// Saturating subtraction. Returns the difference, saturating on overflow.
    ///
    /// See also
    /// <code>FixedI32::[saturating\_sub][FixedI32::saturating_sub]</code> and
    /// <code>FixedU32::[saturating\_sub][FixedU32::saturating_sub]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn saturating_sub(self, rhs: Self) -> Self;

    /// Saturating multiplication. Returns the product, saturating on overflow.
    ///
    /// See also
    /// <code>FixedI32::[saturating\_mul][FixedI32::saturating_mul]</code> and
    /// <code>FixedU32::[saturating\_mul][FixedU32::saturating_mul]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn saturating_mul(self, rhs: Self) -> Self;

    /// Saturating division. Returns the quotient, saturating on overflow.
    ///
    /// See also
    /// <code>FixedI32::[saturating\_div][FixedI32::saturating_div]</code> and
    /// <code>FixedU32::[saturating\_div][FixedU32::saturating_div]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn saturating_div(self, rhs: Self) -> Self;

    /// Saturating reciprocal.
    ///
    /// See also
    /// <code>FixedI32::[saturating\_recip][FixedI32::saturating_recip]</code>
    /// and
    /// <code>FixedU32::[saturating\_recip][FixedU32::saturating_recip]</code>.
    ///
    /// # Panics
    ///
    /// Panics if `self` is zero.
    fn saturating_recip(self) -> Self;

    /// Saturating multiply and add. Returns `self` × `mul` + `add`, saturating on overflow.
    ///
    /// See also
    /// <code>FixedI32::[saturating\_mul\_add][FixedI32::saturating_mul_add]</code>
    /// and
    /// <code>FixedU32::[saturating\_mul\_add][FixedU32::saturating_mul_add]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn saturating_mul_add(self, mul: Self, add: Self) -> Self;

    /// Saturating multiply and add. Adds (`a` × `b`) to `self`, saturating on overflow.
    ///
    /// See also
    /// <code>FixedI32::[saturating\_mul\_acc][FixedI32::saturating_mul_acc]</code>
    /// and
    /// <code>FixedU32::[saturating\_mul\_acc][FixedU32::saturating_mul_acc]</code>.
    fn saturating_mul_acc(&mut self, a: Self, b: Self);

    /// Saturating Euclidean division. Returns the quotient, saturating on overflow.
    ///
    /// See also
    /// <code>FixedI32::[saturating\_div\_euclid][FixedI32::saturating_div_euclid]</code>
    /// and
    /// <code>FixedU32::[saturating\_div\_euclid][FixedU32::saturating_div_euclid]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn saturating_div_euclid(self, rhs: Self) -> Self;

    /// Saturating multiplication by an integer. Returns the product, saturating on overflow.
    ///
    /// See also
    /// <code>FixedI32::[saturating\_mul\_int][FixedI32::saturating_mul_int]</code>
    /// and
    /// <code>FixedU32::[saturating\_mul\_int][FixedU32::saturating_mul_int]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn saturating_mul_int(self, rhs: Self::Bits) -> Self;

    /// Saturating Euclidean division by an integer. Returns the
    /// quotient, saturating on overflow.
    ///
    /// See also
    /// <code>FixedI32::[saturating\_div\_euclid\_int][FixedI32::saturating_div_euclid_int]</code>
    /// and
    /// <code>FixedU32::[saturating\_div\_euclid\_int][FixedU32::saturating_div_euclid_int]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn saturating_div_euclid_int(self, rhs: Self::Bits) -> Self;

    /// Saturating remainder for Euclidean division by an integer.
    /// Returns the remainder, saturating on overflow.
    ///
    /// See also
    /// <code>FixedI32::[saturating\_rem\_euclid\_int][FixedI32::saturating_rem_euclid_int]</code>
    /// and
    /// <code>FixedU32::[saturating\_rem\_euclid\_int][FixedU32::saturating_rem_euclid_int]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn saturating_rem_euclid_int(self, rhs: Self::Bits) -> Self;

    /// Saturating distance. Returns the distance from `self` to `other`,
    /// saturating on overflow.
    ///
    /// See also
    /// <code>FixedI32::[saturating\_dist][FixedI32::saturating_dist]</code> and
    /// <code>FixedU32::[saturating\_dist][FixedU32::saturating_dist]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn saturating_dist(self, other: Self) -> Self;

    /// Linear interpolation between `start` and `end`, saturating on overflow.
    ///
    /// See also
    /// <code>FixedI32::[saturating\_lerp][FixedI32::saturating_lerp]</code> and
    /// <code>FixedU32::[saturating\_lerp][FixedU32::saturating_lerp]</code>.
    fn saturating_lerp(self, start: Self, end: Self) -> Self;

    /// Inverse linear interpolation between `start` and `end`, saturating on overflow.
    ///
    /// See also
    /// <code>FixedI32::[saturating\_inv\_lerp][FixedI32::saturating_inv_lerp]</code>
    /// and
    /// <code>FixedU32::[saturating\_inv\_lerp][FixedU32::saturating_inv_lerp]</code>.
    fn saturating_inv_lerp(self, start: Self, end: Self) -> Self;

    /// Wrapping negation. Returns the negated value, wrapping on overflow.
    ///
    /// See also <code>FixedI32::[wrapping\_neg][FixedI32::wrapping_neg]</code>
    /// and <code>FixedU32::[wrapping\_neg][FixedU32::wrapping_neg]</code>.
    fn wrapping_neg(self) -> Self;

    /// Wrapping addition. Returns the sum, wrapping on overflow.
    ///
    /// See also <code>FixedI32::[wrapping\_add][FixedI32::wrapping_add]</code>
    /// and <code>FixedU32::[wrapping\_add][FixedU32::wrapping_add]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn wrapping_add(self, rhs: Self) -> Self;

    /// Wrapping subtraction. Returns the difference, wrapping on overflow.
    ///
    /// See also <code>FixedI32::[wrapping\_sub][FixedI32::wrapping_sub]</code>
    /// and <code>FixedU32::[wrapping\_sub][FixedU32::wrapping_sub]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn wrapping_sub(self, rhs: Self) -> Self;

    /// Wrapping multiplication. Returns the product, wrapping on overflow.
    ///
    /// See also <code>FixedI32::[wrapping\_mul][FixedI32::wrapping_mul]</code>
    /// and <code>FixedU32::[wrapping\_mul][FixedU32::wrapping_mul]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn wrapping_mul(self, rhs: Self) -> Self;

    /// Wrapping division. Returns the quotient, wrapping on overflow.
    ///
    /// See also <code>FixedI32::[wrapping\_div][FixedI32::wrapping_div]</code>
    /// and <code>FixedU32::[wrapping\_div][FixedU32::wrapping_div]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn wrapping_div(self, rhs: Self) -> Self;

    /// Wrapping reciprocal.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_recip][FixedI32::wrapping_recip]</code> and
    /// <code>FixedU32::[wrapping\_recip][FixedU32::wrapping_recip]</code>.
    ///
    /// # Panics
    ///
    /// Panics if `self` is zero.
    fn wrapping_recip(self) -> Self;

    /// Wrapping multiply and add. Returns `self` × `mul` + `add`, wrapping on overflow.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_mul\_add][FixedI32::wrapping_mul_add]</code>
    /// and
    /// <code>FixedU32::[wrapping\_mul\_add][FixedU32::wrapping_mul_add]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn wrapping_mul_add(self, mul: Self, add: Self) -> Self;

    /// Wrapping multiply and accumulate. Adds (`a` × `b`) to `self`, wrapping on overflow.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_mul\_acc][FixedI32::wrapping_mul_acc]</code>
    /// and
    /// <code>FixedU32::[wrapping\_mul\_acc][FixedU32::wrapping_mul_acc]</code>.
    fn wrapping_mul_acc(&mut self, a: Self, b: Self);

    /// Wrapping Euclidean division. Returns the quotient, wrapping on overflow.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_div\_euclid][FixedI32::wrapping_div_euclid]</code>
    /// and
    /// <code>FixedU32::[wrapping\_div\_euclid][FixedU32::wrapping_div_euclid]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn wrapping_div_euclid(self, rhs: Self) -> Self;

    /// Wrapping multiplication by an integer. Returns the product, wrapping on overflow.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_mul\_int][FixedI32::wrapping_mul_int]</code>
    /// and
    /// <code>FixedU32::[wrapping\_mul\_int][FixedU32::wrapping_mul_int]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn wrapping_mul_int(self, rhs: Self::Bits) -> Self;

    /// Wrapping division by an integer. Returns the quotient, wrapping on overflow.
    ///
    /// Overflow can only occur when dividing the minimum value by −1.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_div\_int][FixedI32::wrapping_div_int]</code>
    /// and
    /// <code>FixedU32::[wrapping\_div\_int][FixedU32::wrapping_div_int]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn wrapping_div_int(self, rhs: Self::Bits) -> Self;

    /// Wrapping Euclidean division by an integer. Returns the
    /// quotient, wrapping on overflow.
    ///
    /// Overflow can only occur when dividing the minimum value by −1.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_div\_euclid\_int][FixedI32::wrapping_div_euclid_int]</code>
    /// and
    /// <code>FixedU32::[wrapping\_div\_euclid\_int][FixedU32::wrapping_div_euclid_int]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn wrapping_div_euclid_int(self, rhs: Self::Bits) -> Self;

    /// Wrapping remainder for Euclidean division by an integer.
    /// Returns the remainder, wrapping on overflow.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_rem\_euclid\_int][FixedI32::wrapping_rem_euclid_int]</code>
    /// and
    /// <code>FixedU32::[wrapping\_rem\_euclid\_int][FixedU32::wrapping_rem_euclid_int]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn wrapping_rem_euclid_int(self, rhs: Self::Bits) -> Self;

    /// Wrapping shift left. Wraps `rhs` if `rhs` ≥ the number of
    /// bits, then shifts and returns the number.
    ///
    /// See also <code>FixedI32::[wrapping\_shl][FixedI32::wrapping_shl]</code>
    /// and <code>FixedU32::[wrapping\_shl][FixedU32::wrapping_shl]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn wrapping_shl(self, rhs: u32) -> Self;

    /// Wrapping shift right. Wraps `rhs` if `rhs` ≥ the number of
    /// bits, then shifts and returns the number.
    ///
    /// See also <code>FixedI32::[wrapping\_shr][FixedI32::wrapping_shr]</code>
    /// and <code>FixedU32::[wrapping\_shr][FixedU32::wrapping_shr]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn wrapping_shr(self, rhs: u32) -> Self;

    /// Wrapping distance. Returns the distance from `self` to `other`, wrapping
    /// on overflow.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_dist][FixedI32::wrapping_dist]</code> and
    /// <code>FixedU32::[wrapping\_dist][FixedU32::wrapping_dist]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn wrapping_dist(self, other: Self) -> Self;

    /// Linear interpolation between `start` and `end`, wrapping on overflow.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_lerp][FixedI32::wrapping_lerp]</code> and
    /// <code>FixedU32::[wrapping\_lerp][FixedU32::wrapping_lerp]</code>.
    fn wrapping_lerp(self, start: Self, end: Self) -> Self;

    /// Inverse linear interpolation between `start` and `end`, wrapping on
    /// overflow.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_inv\_lerp][FixedI32::wrapping_inv_lerp]</code>
    /// and
    /// <code>FixedU32::[wrapping\_inv\_lerp][FixedU32::wrapping_inv_lerp]</code>.
    fn wrapping_inv_lerp(self, start: Self, end: Self) -> Self;

    /// Unwrapped negation. Returns the negated value, panicking on overflow.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_neg][FixedI32::unwrapped_neg]</code> and
    /// <code>FixedU32::[unwrapped\_neg][FixedU32::unwrapped_neg]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the result does not fit.
    #[track_caller]
    fn unwrapped_neg(self) -> Self;

    /// Unwrapped addition. Returns the sum, panicking on overflow.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_add][FixedI32::unwrapped_add]</code> and
    /// <code>FixedU32::[unwrapped\_add][FixedU32::unwrapped_add]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the result does not fit.
    #[track_caller]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn unwrapped_add(self, rhs: Self) -> Self;

    /// Unwrapped subtraction. Returns the difference, panicking on overflow.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_sub][FixedI32::unwrapped_sub]</code> and
    /// <code>FixedU32::[unwrapped\_sub][FixedU32::unwrapped_sub]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the result does not fit.
    #[track_caller]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn unwrapped_sub(self, rhs: Self) -> Self;

    /// Unwrapped multiplication. Returns the product, panicking on overflow.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_mul][FixedI32::unwrapped_mul]</code> and
    /// <code>FixedU32::[unwrapped\_mul][FixedU32::unwrapped_mul]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the result does not fit.
    #[track_caller]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn unwrapped_mul(self, rhs: Self) -> Self;

    /// Unwrapped division. Returns the quotient, panicking on overflow.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_div][FixedI32::unwrapped_div]</code> and
    /// <code>FixedU32::[unwrapped\_div][FixedU32::unwrapped_div]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero or if the result does not fit.
    #[track_caller]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn unwrapped_div(self, rhs: Self) -> Self;

    /// Unwrapped remainder. Returns the quotient, panicking if the divisor is zero.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_rem][FixedI32::unwrapped_rem]</code> and
    /// <code>FixedU32::[unwrapped\_rem][FixedU32::unwrapped_rem]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero.
    #[track_caller]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn unwrapped_rem(self, rhs: Self) -> Self;

    /// Unwrapped reciprocal. Returns reciprocal, panicking on overflow.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_recip][FixedI32::unwrapped_recip]</code> and
    /// <code>FixedU32::[unwrapped\_recip][FixedU32::unwrapped_recip]</code>.
    ///
    /// # Panics
    ///
    /// Panics if `self` is zero or on overflow.
    #[track_caller]
    fn unwrapped_recip(self) -> Self;

    /// Unwrapped multiply and add. Returns `self` × `mul` + `add`, panicking on overflow.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_mul\_add][FixedI32::unwrapped_mul_add]</code>
    /// and
    /// <code>FixedU32::[unwrapped\_mul\_add][FixedU32::unwrapped_mul_add]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the result does not fit.
    #[track_caller]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn unwrapped_mul_add(self, mul: Self, add: Self) -> Self;

    /// Unwrapped multiply and accumulate. Adds (`a` × `b`) to `self`, panicking on overflow.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_mul\_acc][FixedI32::unwrapped_mul_acc]</code>
    /// and
    /// <code>FixedU32::[unwrapped\_mul\_acc][FixedU32::unwrapped_mul_acc]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the result does not fit.
    #[track_caller]
    fn unwrapped_mul_acc(&mut self, a: Self, b: Self);

    /// Unwrapped Euclidean division. Returns the quotient, panicking on overflow.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_div\_euclid][FixedI32::unwrapped_div_euclid]</code>
    /// and
    /// <code>FixedU32::[unwrapped\_div\_euclid][FixedU32::unwrapped_div_euclid]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero or if the result does not fit.
    #[track_caller]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn unwrapped_div_euclid(self, rhs: Self) -> Self;

    /// Unwrapped remainder for Euclidean division. Returns the
    /// remainder, panicking if the divisor is zero.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_rem\_euclid][FixedI32::unwrapped_rem_euclid]</code>
    /// and
    /// <code>FixedU32::[unwrapped\_rem\_euclid][FixedU32::unwrapped_rem_euclid]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero.
    #[track_caller]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn unwrapped_rem_euclid(self, rhs: Self) -> Self;

    /// Unwrapped multiplication by an integer. Returns the product, panicking on overflow.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_mul\_int][FixedI32::unwrapped_mul_int]</code>
    /// and
    /// <code>FixedU32::[unwrapped\_mul\_int][FixedU32::unwrapped_mul_int]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the result does not fit.
    #[track_caller]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn unwrapped_mul_int(self, rhs: Self::Bits) -> Self;

    /// Unwrapped division by an integer. Returns the quotient, panicking on overflow.
    ///
    /// Overflow can only occur when dividing the minimum value by −1.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_div\_int][FixedI32::unwrapped_div_int]</code>
    /// and
    /// <code>FixedU32::[unwrapped\_div\_int][FixedU32::unwrapped_div_int]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero or if the result does not fit.
    #[track_caller]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn unwrapped_div_int(self, rhs: Self::Bits) -> Self;

    /// Unwrapped remainder for division by an integer. Returns the
    /// remainder, panicking if the divisor is zero.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_rem\_int][FixedI32::unwrapped_rem_int]</code>
    /// and
    /// <code>FixedU32::[unwrapped\_rem\_int][FixedU32::unwrapped_rem_int]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero.
    #[track_caller]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn unwrapped_rem_int(self, rhs: Self::Bits) -> Self;

    /// Unwrapped Euclidean division by an integer. Returns the
    /// quotient, panicking on overflow.
    ///
    /// Overflow can only occur when dividing the minimum value by −1.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_div\_euclid\_int][FixedI32::unwrapped_div_euclid_int]</code>
    /// and
    /// <code>FixedU32::[unwrapped\_div\_euclid\_int][FixedU32::unwrapped_div_euclid_int]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero or if the result does not fit.
    #[track_caller]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn unwrapped_div_euclid_int(self, rhs: Self::Bits) -> Self;

    /// Unwrapped remainder for Euclidean division by an integer.
    /// Returns the remainder, panicking on overflow.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_rem\_euclid\_int][FixedI32::unwrapped_rem_euclid_int]</code>
    /// and
    /// <code>FixedU32::[unwrapped\_rem\_euclid\_int][FixedU32::unwrapped_rem_euclid_int]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero or if the result does not fit.
    #[track_caller]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn unwrapped_rem_euclid_int(self, rhs: Self::Bits) -> Self;

    /// Unwrapped shift left. Panics if `rhs` ≥ the number of bits.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_shl][FixedI32::unwrapped_shl]</code> and
    /// <code>FixedU32::[unwrapped\_shl][FixedU32::unwrapped_shl]</code>.
    ///
    /// # Panics
    ///
    /// Panics if `rhs` ≥ the number of bits.
    #[track_caller]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn unwrapped_shl(self, rhs: u32) -> Self;

    /// Unwrapped shift right. Panics if `rhs` ≥ the number of bits.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_shr][FixedI32::unwrapped_shr]</code> and
    /// <code>FixedU32::[unwrapped\_shr][FixedU32::unwrapped_shr]</code>.
    ///
    /// # Panics
    ///
    /// Panics if `rhs` ≥ the number of bits.
    #[track_caller]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn unwrapped_shr(self, rhs: u32) -> Self;

    /// Unwrapped distance. Returns the distance from `self` to `other`,
    /// panicking on overflow.
    ///
    /// # Panics
    ///
    /// Panics if the result does not fit.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_dist][FixedI32::unwrapped_dist]</code> and
    /// <code>FixedU32::[unwrapped\_dist][FixedU32::unwrapped_dist]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn unwrapped_dist(self, other: Self) -> Self;

    /// Linear interpolation between `start` and `end`, panicking on overflow.
    ///
    /// # Panics
    ///
    /// Panics if the result does not fit.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_lerp][FixedI32::unwrapped_lerp]</code> and
    /// <code>FixedU32::[unwrapped\_lerp][FixedU32::unwrapped_lerp]</code>.
    fn unwrapped_lerp(self, start: Self, end: Self) -> Self;

    /// Inverse linear interpolation between `start` and `end`, panicking on overflow.
    ///
    /// # Panics
    ///
    /// Panics when `start` = `end` or when the results overflows.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_inv\_lerp][FixedI32::unwrapped_inv_lerp]</code>
    /// and
    /// <code>FixedU32::[unwrapped\_inv\_lerp][FixedU32::unwrapped_inv_lerp]</code>.
    fn unwrapped_inv_lerp(self, start: Self, end: Self) -> Self;

    /// Overflowing negation.
    ///
    /// Returns a [tuple] of the negated value and a [`bool`],
    /// indicating whether an overflow has occurred. On overflow, the
    /// wrapped value is returned.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_neg][FixedI32::overflowing_neg]</code> and
    /// <code>FixedU32::[overflowing\_neg][FixedU32::overflowing_neg]</code>.
    fn overflowing_neg(self) -> (Self, bool);

    /// Overflowing addition.
    ///
    /// Returns a [tuple] of the sum and a [`bool`], indicating whether
    /// an overflow has occurred. On overflow, the wrapped value is
    /// returned.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_add][FixedI32::overflowing_add]</code> and
    /// <code>FixedU32::[overflowing\_add][FixedU32::overflowing_add]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn overflowing_add(self, rhs: Self) -> (Self, bool);

    /// Overflowing subtraction.
    ///
    /// Returns a [tuple] of the difference and a [`bool`], indicating
    /// whether an overflow has occurred. On overflow, the wrapped
    /// value is returned.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_sub][FixedI32::overflowing_sub]</code> and
    /// <code>FixedU32::[overflowing\_sub][FixedU32::overflowing_sub]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn overflowing_sub(self, rhs: Self) -> (Self, bool);

    /// Overflowing multiplication.
    ///
    /// Returns a [tuple] of the product and a [`bool`], indicating
    /// whether an overflow has occurred. On overflow, the wrapped
    /// value is returned.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_mul][FixedI32::overflowing_mul]</code> and
    /// <code>FixedU32::[overflowing\_mul][FixedU32::overflowing_mul]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn overflowing_mul(self, rhs: Self) -> (Self, bool);

    /// Overflowing division.
    ///
    /// Returns a [tuple] of the quotient and a [`bool`], indicating
    /// whether an overflow has occurred. On overflow, the wrapped
    /// value is returned.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_div][FixedI32::overflowing_div]</code> and
    /// <code>FixedU32::[overflowing\_div][FixedU32::overflowing_div]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn overflowing_div(self, rhs: Self) -> (Self, bool);

    /// Overflowing reciprocal.
    ///
    /// Returns a [tuple] of the reciprocal of `self` and a [`bool`],
    /// indicating whether an overflow has occurred. On overflow, the
    /// wrapped value is returned.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_recip][FixedI32::overflowing_recip]</code>
    /// and
    /// <code>FixedU32::[overflowing\_recip][FixedU32::overflowing_recip]</code>.
    ///
    /// # Panics
    ///
    /// Panics if `self` is zero.
    fn overflowing_recip(self) -> (Self, bool);

    /// Overflowing multiply  and add.
    ///
    /// Returns a [tuple] of `self` × `mul` + `add` and a [`bool`],
    /// indicating whether an overflow has occurred. On overflow, the
    /// wrapped value is returned.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_mul\_add][FixedI32::overflowing_mul_add]</code>
    /// and
    /// <code>FixedU32::[overflowing\_mul\_add][FixedU32::overflowing_mul_add]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn overflowing_mul_add(self, mul: Self, add: Self) -> (Self, bool);

    /// Overflowing multiply and accumulate. Adds (`a` × `b`) to `self`,
    /// wrapping and returning [`true`] if overflow occurs.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_mul\_acc][FixedI32::overflowing_mul_acc]</code>
    /// and
    /// <code>FixedU32::[overflowing\_mul\_acc][FixedU32::overflowing_mul_acc]</code>.
    #[must_use = "this returns whether overflow occurs; use `wrapping_mul_acc` if the flag is not needed"]
    fn overflowing_mul_acc(&mut self, a: Self, b: Self) -> bool;

    /// Overflowing Euclidean division.
    ///
    /// Returns a [tuple] of the quotient and a [`bool`], indicating
    /// whether an overflow has occurred. On overflow, the wrapped
    /// value is returned.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_div\_euclid][FixedI32::overflowing_div_euclid]</code>
    /// and
    /// <code>FixedU32::[overflowing\_div\_euclid][FixedU32::overflowing_div_euclid]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn overflowing_div_euclid(self, rhs: Self) -> (Self, bool);

    /// Overflowing multiplication by an integer.
    ///
    /// Returns a [tuple] of the product and a [`bool`], indicating
    /// whether an overflow has occurred. On overflow, the wrapped
    /// value is returned.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_mul\_int][FixedI32::overflowing_mul_int]</code>
    /// and
    /// <code>FixedU32::[overflowing\_mul\_int][FixedU32::overflowing_mul_int]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn overflowing_mul_int(self, rhs: Self::Bits) -> (Self, bool);

    /// Overflowing division by an integer.
    ///
    /// Returns a [tuple] of the quotient and a [`bool`], indicating
    /// whether an overflow has occurred. On overflow, the wrapped
    /// value is returned.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_div\_int][FixedI32::overflowing_div_int]</code>
    /// and
    /// <code>FixedU32::[overflowing\_div\_int][FixedU32::overflowing_div_int]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn overflowing_div_int(self, rhs: Self::Bits) -> (Self, bool);

    /// Overflowing Euclidean division by an integer.
    ///
    /// Returns a [tuple] of the quotient and a [`bool`], indicating
    /// whether an overflow has occurred. On overflow, the wrapped
    /// value is returned.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_div\_euclid\_int][FixedI32::overflowing_div_euclid_int]</code>
    /// and
    /// <code>FixedU32::[overflowing\_div\_euclid\_int][FixedU32::overflowing_div_euclid_int]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn overflowing_div_euclid_int(self, rhs: Self::Bits) -> (Self, bool);

    /// Overflowing remainder for Euclidean division by an integer.
    ///
    /// Returns a [tuple] of the remainder and a [`bool`], indicating
    /// whether an overflow has occurred. On overflow, the wrapped
    /// value is returned.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_rem\_euclid\_int][FixedI32::overflowing_rem_euclid_int]</code>
    /// and
    /// <code>FixedU32::[overflowing\_rem\_euclid\_int][FixedU32::overflowing_rem_euclid_int]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn overflowing_rem_euclid_int(self, rhs: Self::Bits) -> (Self, bool);

    /// Overflowing shift left.
    ///
    /// Returns a [tuple] of the shifted value and a [`bool`],
    /// indicating whether an overflow has occurred. On overflow, the
    /// wrapped value is returned.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_shl][FixedI32::overflowing_shl]</code> and
    /// <code>FixedU32::[overflowing\_shl][FixedU32::overflowing_shl]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn overflowing_shl(self, rhs: u32) -> (Self, bool);

    /// Overflowing shift right.
    ///
    /// Returns a [tuple] of the shifted value and a [`bool`],
    /// indicating whether an overflow has occurred. On overflow, the
    /// wrapped value is returned.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_shr][FixedI32::overflowing_shr]</code> and
    /// <code>FixedU32::[overflowing\_shr][FixedU32::overflowing_shr]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn overflowing_shr(self, rhs: u32) -> (Self, bool);

    /// Overflowing distance.
    ///
    /// Returns a [tuple] of the distance from `self` to `other` and a [`bool`],
    /// indicating whether an overflow has occurred. On overflow, the wrapped
    /// value is returned.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_dist][FixedI32::overflowing_dist]</code>
    /// and
    /// <code>FixedU32::[overflowing\_dist][FixedU32::overflowing_dist]</code>.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn overflowing_dist(self, other: Self) -> (Self, bool);

    /// Overflowing linear interpolation between `start` and `end`.
    ///
    /// Returns a [tuple] of the interpolated value and a [`bool`], indicating
    /// whether an overflow has occurred. On overflow, the wrapped value is
    /// returned.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_lerp][FixedI32::overflowing_lerp]</code>
    /// and
    /// <code>FixedU32::[overflowing\_lerp][FixedU32::overflowing_lerp]</code>.
    fn overflowing_lerp(self, start: Self, end: Self) -> (Self, bool);

    /// Overflowing inverse linear interpolation between `start` and `end`.
    ///
    /// Returns a [tuple] of the computed value and a [`bool`], indicating
    /// whether an overflow has occurred. On overflow, the wrapped value is
    /// returned.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_inv\_lerp][FixedI32::overflowing_inv_lerp]</code>
    /// and
    /// <code>FixedU32::[overflowing\_inv\_lerp][FixedU32::overflowing_inv_lerp]</code>.
    fn overflowing_inv_lerp(self, start: Self, end: Self) -> (Self, bool);
}

/// This trait provides methods common to all signed fixed-point numbers.
///
/// Methods common to all fixed-point numbers including unsigned
/// fixed-point numbers are provided by the [`Fixed`] supertrait.
///
/// This trait is sealed and cannot be implemented for more types; it
/// is implemented for [`FixedI8`], [`FixedI16`], [`FixedI32`],
/// [`FixedI64`], and [`FixedI128`].
pub trait FixedSigned: Fixed
where
    Self: Neg<Output = Self>,
{
    /// Returns the number of bits required to represent the value.
    ///
    /// See also <code>FixedI32::[signed\_bits][FixedI32::signed_bits]</code>.
    fn signed_bits(self) -> u32;

    /// Returns [`true`] if the number is > 0.
    ///
    /// See also <code>FixedI32::[is\_positive][FixedI32::is_positive]</code>.
    fn is_positive(self) -> bool;

    /// Returns [`true`] if the number is < 0.
    ///
    /// See also <code>FixedI32::[is\_negative][FixedI32::is_negative]</code>.
    fn is_negative(self) -> bool;

    /// Returns the absolute value.
    ///
    /// See also <code>FixedI32::[abs][FixedI32::abs]</code>.
    fn abs(self) -> Self;

    /// Returns the absolute value using an unsigned type without any
    /// wrapping or panicking.
    ///
    /// See also <code>FixedI32::[unsigned\_abs][FixedI32::unsigned_abs]</code>.
    fn unsigned_abs(self) -> Self::Unsigned;

    /// Returns the distance from `self` to `other` using an unsigned type
    /// without any wrapping or panicking.
    ///
    /// See also
    /// <code>FixedI32::[unsigned\_dist][FixedI32::unsigned_dist]</code>.
    fn unsigned_dist(self, other: Self) -> Self::Unsigned;

    /// Returns a number representing the sign of `self`.
    ///
    /// See also <code>FixedI32::[signum][FixedI32::signum]</code>.
    ///
    /// # Panics
    ///
    /// When debug assertions are enabled, this method panics
    ///   * if the value is positive and the fixed-point number has
    ///     zero or one integer bits such that it cannot hold the
    ///     value 1.
    ///   * if the value is negative and the fixed-point number has
    ///     zero integer bits, such that it cannot hold the value −1.
    ///
    /// When debug assertions are not enabled, the wrapped value can
    /// be returned in those cases, but it is not considered a
    /// breaking change if in the future it panics; using this method
    /// when 1 and −1 cannot be represented is almost certainly a bug.
    fn signum(self) -> Self;

    /// Checked absolute value. Returns the absolute value, or [`None`] on overflow.
    ///
    /// Overflow can only occur when trying to find the absolute value of the minimum value.
    ///
    /// See also <code>FixedI32::[checked\_abs][FixedI32::checked_abs]</code>.
    fn checked_abs(self) -> Option<Self>;

    /// Checked signum. Returns a number representing the sign of
    /// `self`, or [`None`] on overflow.
    ///
    /// Overflow can only occur
    ///   * if the value is positive and the fixed-point number has zero
    ///     or one integer bits such that it cannot hold the value 1.
    ///   * if the value is negative and the fixed-point number has zero
    ///         integer bits, such that it cannot hold the value −1.
    ///
    /// See also
    /// <code>FixedI32::[checked\_signum][FixedI32::checked_signum]</code>.
    fn checked_signum(self) -> Option<Self>;

    /// Saturating absolute value. Returns the absolute value, saturating on overflow.
    ///
    /// Overflow can only occur when trying to find the absolute value of the minimum value.
    ///
    /// See also
    /// <code>FixedI32::[saturating\_abs][FixedI32::saturating_abs]</code>.
    fn saturating_abs(self) -> Self;

    /// Saturating signum. Returns a number representing the sign of
    /// `self`, saturating on overflow.
    ///
    /// Overflow can only occur
    ///   * if the value is positive and the fixed-point number has zero
    ///     or one integer bits such that it cannot hold the value 1.
    ///   * if the value is negative and the fixed-point number has zero
    ///         integer bits, such that it cannot hold the value −1.
    ///
    /// See also
    /// <code>FixedI32::[saturating\_signum][FixedI32::saturating_signum]</code>.
    fn saturating_signum(self) -> Self;

    /// Wrapping absolute value. Returns the absolute value, wrapping on overflow.
    ///
    /// Overflow can only occur when trying to find the absolute value of the minimum value.
    ///
    /// See also <code>FixedI32::[wrapping\_abs][FixedI32::wrapping_abs]</code>.
    fn wrapping_abs(self) -> Self;

    /// Wrapping signum. Returns a number representing the sign of
    /// `self`, wrapping on overflow.
    ///
    /// Overflow can only occur
    ///   * if the value is positive and the fixed-point number has zero
    ///     or one integer bits such that it cannot hold the value 1.
    ///   * if the value is negative and the fixed-point number has zero
    ///         integer bits, such that it cannot hold the value −1.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_signum][FixedI32::wrapping_signum]</code>.
    fn wrapping_signum(self) -> Self;

    /// Unwrapped absolute value. Returns the absolute value, panicking on overflow.
    ///
    /// Overflow can only occur when trying to find the absolute value of the minimum value.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_abs][FixedI32::unwrapped_abs]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the result does not fit.
    #[track_caller]
    fn unwrapped_abs(self) -> Self;

    /// Unwrapped signum. Returns a number representing the sign of
    /// `self`, panicking on overflow.
    ///
    /// Overflow can only occur
    ///   * if the value is positive and the fixed-point number has zero
    ///     or one integer bits such that it cannot hold the value 1.
    ///   * if the value is negative and the fixed-point number has zero
    ///         integer bits, such that it cannot hold the value −1.
    ///
    /// See also
    /// <code>FixedI32::[unwrapped\_signum][FixedI32::unwrapped_signum]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the result does not fit.
    #[track_caller]
    fn unwrapped_signum(self) -> Self;

    /// Overflowing absolute value.
    ///
    /// Returns a [tuple] of the fixed-point number and a [`bool`],
    /// indicating whether an overflow has occurred. On overflow, the
    /// wrapped value is returned.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_abs][FixedI32::overflowing_abs]</code>.
    fn overflowing_abs(self) -> (Self, bool);

    /// Overflowing signum.
    ///
    /// Returns a [tuple] of the signum and a [`bool`], indicating
    /// whether an overflow has occurred. On overflow, the wrapped
    /// value is returned.
    ///
    /// Overflow can only occur
    ///   * if the value is positive and the fixed-point number has zero
    ///     or one integer bits such that it cannot hold the value 1.
    ///   * if the value is negative and the fixed-point number has zero
    ///         integer bits, such that it cannot hold the value −1.
    ///
    /// See also
    /// <code>FixedI32::[overflowing\_signum][FixedI32::overflowing_signum]</code>.
    fn overflowing_signum(self) -> (Self, bool);
}

/// This trait provides methods common to all unsigned fixed-point numbers.
///
/// Methods common to all fixed-point numbers including signed
/// fixed-point numbers are provided by the [`Fixed`] supertrait.
///
/// This trait is sealed and cannot be implemented for more types; it
/// is implemented for [`FixedU8`], [`FixedU16`], [`FixedU32`],
/// [`FixedU64`], and [`FixedU128`].
pub trait FixedUnsigned: Fixed
where
    Self: Div<<Self as Fixed>::NonZeroBits, Output = Self>,
    Self: DivAssign<<Self as Fixed>::NonZeroBits>,
{
    /// Returns the number of bits required to represent the value.
    ///
    /// See also
    /// <code>FixedU32::[significant\_bits][FixedU32::significant_bits]</code>.
    fn significant_bits(self) -> u32;

    /// Returns [`true`] if the fixed-point number is
    /// 2<sup><i>k</i></sup> for some integer <i>k</i>.
    ///
    /// See also
    /// <code>FixedU32::[is\_power\_of\_two][FixedU32::is_power_of_two]</code>.
    fn is_power_of_two(self) -> bool;

    /// Returns the highest one in the binary representation, or zero
    /// if `self` is zero.
    ///
    /// See also <code>FixedU32::[highest\_one][FixedU32::highest_one]</code>.
    fn highest_one(self) -> Self;

    /// Returns the smallest power of two that is ≥ `self`.
    ///
    /// See also
    /// <code>FixedU32::[next\_power\_of\_two][FixedU32::next_power_of_two]</code>.
    fn next_power_of_two(self) -> Self;

    /// Returns the smallest power of two that is ≥ `self`, or [`None`] if the
    /// next power of two is too large to represent.
    ///
    /// See also
    /// <code>FixedU32::[checked\_next\_power\_of\_two][FixedU32::checked_next_power_of_two]</code>.
    fn checked_next_power_of_two(self) -> Option<Self>;

    /// Returns the smallest power of two that is ≥ `self`, wrapping
    /// to 0 if the next power of two is too large to represent.
    ///
    /// See also
    /// <code>FixedU32::[wrapping\_next\_power\_of\_two][FixedU32::wrapping_next_power_of_two]</code>.
    fn wrapping_next_power_of_two(self) -> Self;

    /// Returns the smallest power of two that is ≥ `self`, panicking
    /// if the next power of two is too large to represent.
    ///
    /// See also
    /// <code>FixedU32::[unwrapped\_next\_power\_of\_two][FixedU32::unwrapped_next_power_of_two]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the result does not fit.
    #[track_caller]
    fn unwrapped_next_power_of_two(self) -> Self;
}

/// This trait provides lossless conversions that might be fallible.
///
/// This trait is implemented for conversions between integer
/// primitives, floating-point primitives and fixed-point numbers.
///
/// # Examples
///
/// ```rust
/// use fixed::traits::LosslessTryFrom;
/// use fixed::types::{I24F8, I4F12};
/// // original is 0x000001.23, lossless is 0x1.230
/// let original = I24F8::from_bits(0x0000_0123);
/// let lossless = I4F12::lossless_try_from(original);
/// assert_eq!(lossless, Some(I4F12::from_bits(0x1230)));
/// // too_large is 0x000012.34, 0x12.340 does not fit in I4F12
/// let too_large = I24F8::from_bits(0x0000_1234);
/// let overflow = I4F12::lossless_try_from(too_large);
/// assert_eq!(overflow, None);
/// ```
pub trait LosslessTryFrom<Src>: Sized {
    /// Performs the conversion.
    fn lossless_try_from(src: Src) -> Option<Self>;
}

/// This trait provides lossless conversions that might be fallible.
/// This is the reciprocal of [`LosslessTryFrom`].
///
/// Usually [`LosslessTryFrom`] should be implemented instead of this
/// trait; there is a blanket implementation which provides this trait
/// when [`LosslessTryFrom`] is implemented (similar to [`Into`] and
/// [`From`]).
///
/// # Examples
///
/// ```rust
/// use fixed::traits::LosslessTryInto;
/// use fixed::types::{I24F8, I4F12};
/// // original is 0x000001.23, lossless is 0x1.230
/// let original = I24F8::from_bits(0x0000_0123);
/// let lossless: Option<I4F12> = original.lossless_try_into();
/// assert_eq!(lossless, Some(I4F12::from_bits(0x1230)));
/// // too_large is 0x000012.34, 0x12.340 does not fit in I4F12
/// let too_large = I24F8::from_bits(0x0000_1234);
/// let overflow: Option<I4F12> = too_large.lossless_try_into();
/// assert_eq!(overflow, None);
/// ```
pub trait LosslessTryInto<Dst> {
    /// Performs the conversion.
    fn lossless_try_into(self) -> Option<Dst>;
}

impl<Src, Dst> LosslessTryInto<Dst> for Src
where
    Dst: LosslessTryFrom<Src>,
{
    fn lossless_try_into(self) -> Option<Dst> {
        Dst::lossless_try_from(self)
    }
}

/// This trait provides infallible conversions that might be lossy.
///
/// This trait is implemented for conversions between integer
/// primitives, floating-point primitives and fixed-point numbers.
///
/// # Examples
///
/// ```rust
/// use fixed::traits::LossyFrom;
/// use fixed::types::{I12F4, I8F24};
/// // original is 0x12.345678, lossy is 0x012.3
/// let original = I8F24::from_bits(0x1234_5678);
/// let lossy = I12F4::lossy_from(original);
/// assert_eq!(lossy, I12F4::from_bits(0x0123));
/// ```
pub trait LossyFrom<Src> {
    /// Performs the conversion.
    fn lossy_from(src: Src) -> Self;
}

/// This trait provides infallible conversions that might be lossy.
/// This is the reciprocal of [`LossyFrom`].
///
/// Usually [`LossyFrom`] should be implemented instead of this trait;
/// there is a blanket implementation which provides this trait when
/// [`LossyFrom`] is implemented (similar to [`Into`] and [`From`]).
///
/// # Examples
///
/// ```rust
/// use fixed::traits::LossyInto;
/// use fixed::types::{I12F4, I8F24};
/// // original is 0x12.345678, lossy is 0x012.3
/// let original = I8F24::from_bits(0x1234_5678);
/// let lossy: I12F4 = original.lossy_into();
/// assert_eq!(lossy, I12F4::from_bits(0x0123));
/// ```
pub trait LossyInto<Dst> {
    /// Performs the conversion.
    fn lossy_into(self) -> Dst;
}

impl<Src, Dst> LossyInto<Dst> for Src
where
    Dst: LossyFrom<Src>,
{
    fn lossy_into(self) -> Dst {
        Dst::lossy_from(self)
    }
}

/// This trait provides checked conversions from fixed-point numbers.
///
/// This trait is implemented for conversions between integer
/// primitives, floating-point primitives and fixed-point numbers.
///
/// # Examples
///
/// ```rust
/// use fixed::traits::FromFixed;
/// use fixed::types::U8F8;
/// // 0x87.65
/// let f = U8F8::from_bits(0x8765);
/// assert_eq!(f32::from_fixed(f), f32::from(0x8765u16) / 256.0);
/// assert_eq!(i32::checked_from_fixed(f), Some(0x87));
/// assert_eq!(u8::saturating_from_fixed(f), 0x87);
/// // no fit
/// assert_eq!(i8::checked_from_fixed(f), None);
/// assert_eq!(i8::saturating_from_fixed(f), i8::MAX);
/// assert_eq!(i8::wrapping_from_fixed(f), 0x87u8 as i8);
/// assert_eq!(i8::overflowing_from_fixed(f), (0x87u8 as i8, true));
/// ```
pub trait FromFixed {
    /// Converts from a fixed-point number.
    ///
    /// Any extra fractional bits are discarded, which rounds towards −∞.
    ///
    /// # Panics
    ///
    /// When debug assertions are enabled, panics if the value does
    /// not fit. When debug assertions are not enabled, the wrapped
    /// value can be returned, but it is not considered a breaking
    /// change if in the future it panics; if wrapping is required use
    /// [`wrapping_from_fixed`] instead.
    ///
    /// [`wrapping_from_fixed`]: FromFixed::wrapping_from_fixed
    fn from_fixed<F: Fixed>(src: F) -> Self;

    /// Converts from a fixed-point number if it fits, otherwise returns [`None`].
    ///
    /// Any extra fractional bits are discarded, which rounds towards −∞.
    fn checked_from_fixed<F: Fixed>(src: F) -> Option<Self>
    where
        Self: Sized;

    /// Converts from a fixed-point number, saturating if it does not fit.
    ///
    /// Any extra fractional bits are discarded, which rounds towards −∞.
    fn saturating_from_fixed<F: Fixed>(src: F) -> Self;

    /// Converts from a fixed-point number, wrapping if it does not fit.
    ///
    /// Any extra fractional bits are discarded, which rounds towards −∞.
    fn wrapping_from_fixed<F: Fixed>(src: F) -> Self;

    /// Converts from a fixed-point number.
    ///
    /// Returns a [tuple] of the value and a [`bool`] indicating whether
    /// an overflow has occurred. On overflow, the wrapped value is
    /// returned.
    ///
    /// Any extra fractional bits are discarded, which rounds towards −∞.
    fn overflowing_from_fixed<F: Fixed>(src: F) -> (Self, bool)
    where
        Self: Sized;

    /// Converts from a fixed-point number, panicking if the value
    /// does not fit.
    ///
    /// Any extra fractional bits are discarded, which rounds towards −∞.
    ///
    /// # Panics
    ///
    /// Panics if the value does not fit, even when debug assertions
    /// are not enabled.
    #[inline]
    #[track_caller]
    fn unwrapped_from_fixed<F: Fixed>(src: F) -> Self
    where
        Self: Sized,
    {
        match Self::overflowing_from_fixed(src) {
            (val, false) => val,
            (_, true) => panic!("overflow"),
        }
    }
}

/// This trait provides checked conversions to fixed-point numbers.
///
/// This trait is implemented for conversions between integer
/// primitives, floating-point primitives and fixed-point numbers.
///
/// # Examples
///
/// ```rust
/// use fixed::traits::ToFixed;
/// use fixed::types::{U8F8, U16F16};
/// let f: U8F8 = 13.5f32.to_fixed();
/// assert_eq!(f, U8F8::from_bits((13 << 8) | (1 << 7)));
/// // 0x1234.5678 is too large and can be wrapped to 0x34.56
/// let too_large = U16F16::from_bits(0x1234_5678);
/// let checked: Option<U8F8> = too_large.checked_to_num();
/// assert_eq!(checked, None);
/// let saturating: U8F8 = too_large.saturating_to_num();
/// assert_eq!(saturating, U8F8::MAX);
/// let wrapping: U8F8 = too_large.wrapping_to_num();
/// assert_eq!(wrapping, U8F8::from_bits(0x3456));
/// let overflowing: (U8F8, bool) = too_large.overflowing_to_num();
/// assert_eq!(overflowing, (U8F8::from_bits(0x3456), true));
/// ```
pub trait ToFixed {
    /// Converts to a fixed-point number.
    ///
    /// Any extra fractional bits are discarded, which rounds towards −∞.
    ///
    /// # Panics
    ///
    /// Panics if `self` is a floating-point number that is not [finite].
    ///
    /// When debug assertions are enabled, also panics if the value
    /// does not fit. When debug assertions are not enabled, the
    /// wrapped value can be returned, but it is not considered a
    /// breaking change if in the future it panics; if wrapping is
    /// required use [`wrapping_to_fixed`] instead.
    ///
    /// [`wrapping_to_fixed`]: ToFixed::wrapping_to_fixed
    /// [finite]: f64::is_finite
    fn to_fixed<F: Fixed>(self) -> F;

    /// Converts to a fixed-point number if it fits, otherwise returns [`None`].
    ///
    /// Any extra fractional bits are discarded, which rounds towards −∞.
    fn checked_to_fixed<F: Fixed>(self) -> Option<F>;

    /// Converts to a fixed-point number, saturating if it does not fit.
    ///
    /// Any extra fractional bits are discarded, which rounds towards −∞.
    ///
    /// # Panics
    ///
    /// Panics if `self` is a floating-point number that is [NaN].
    ///
    /// [NaN]: f64::is_nan
    fn saturating_to_fixed<F: Fixed>(self) -> F;

    /// Converts to a fixed-point number, wrapping if it does not fit.
    ///
    /// Any extra fractional bits are discarded, which rounds towards −∞.
    ///
    /// # Panics
    ///
    /// Panics if `self` is a floating-point number that is not [finite].
    ///
    /// [finite]: f64::is_finite
    fn wrapping_to_fixed<F: Fixed>(self) -> F;

    /// Converts to a fixed-point number.
    ///
    /// Returns a [tuple] of the fixed-point number and a [`bool`]
    /// indicating whether an overflow has occurred. On overflow, the
    /// wrapped value is returned.
    ///
    /// Any extra fractional bits are discarded, which rounds towards −∞.
    ///
    /// # Panics
    ///
    /// Panics if `self` is a floating-point number that is not [finite].
    ///
    /// [finite]: f64::is_finite
    fn overflowing_to_fixed<F: Fixed>(self) -> (F, bool);

    /// Converts to a fixed-point number, panicking if it does not fit.
    ///
    /// Any extra fractional bits are discarded, which rounds towards −∞.
    ///
    /// # Panics
    ///
    /// Panics if `self` is a floating-point number that is not
    /// [finite] or if the value does not fit, even if debug
    /// assertions are not enabled.
    ///
    /// [finite]: f64::is_finite
    #[inline]
    #[track_caller]
    fn unwrapped_to_fixed<F: Fixed>(self) -> F
    where
        Self: Sized,
    {
        match self.overflowing_to_fixed() {
            (val, false) => val,
            (_, true) => panic!("overflow"),
        }
    }
}

/// This trait provides a way to convert a number to/from an equivalent
/// fixed-point number.
///
/// Implementations are provided for the signed integer primitives [`i8`],
/// [`i16`], [`i32`], [`i64`] and [`i128`], which have equivalent fixed-point
/// types [`I8F0`], [`I16F0`], [`I32F0`], [`I64F0`] and [`I128F0`]. Similar
/// implementations are provided for the unsigned integer primitives [`u8`],
/// [`u16`], [`u32`], [`u64`] and [`u128`].
///
/// # Examples
///
/// An [`i32`] can be treated as an [`I32F0`].
///
/// ```rust
/// use fixed::traits::{Fixed, FixedEquiv};
///
/// fn next_up<F: Fixed>(f: &mut F) {
///     *f += F::DELTA;
/// }
///
/// let mut i = 12i32;
/// // next_up is called with &mut i converted to &mut I32F0
/// next_up(i.as_fixed_equiv_mut());
/// assert_eq!(i, 13);
/// ```
///
/// Simlarly, an [`I32F0`] can be treated as an [`i32`].
///
/// ```rust
/// use fixed::{traits::FixedEquiv, types::I32F0};
///
/// fn increase_by_5(i: &mut i32) {
///     *i += 5;
/// }
///
/// let mut f = I32F0::from_num(12);
/// // increase_by_5 is called with &mut f converted to &mut i32
/// increase_by_5(i32::mut_from_fixed_equiv(&mut f));
/// assert_eq!(f, 17);
/// ```
///
/// [`I8F0`]: crate::types::I8F0
/// [`I16F0`]: crate::types::I16F0
/// [`I32F0`]: crate::types::I32F0
/// [`I64F0`]: crate::types::I64F0
/// [`I128F0`]: crate::types::I128F0
pub trait FixedEquiv {
    /// The equivalent fixed-point type.
    type Equiv: Fixed;

    /// Converts an owned value to the equivalent fixed-point type.
    fn to_fixed_equiv(self) -> Self::Equiv;

    /// Converts a reference into a reference to the equivalent fixed-point
    /// type.
    fn as_fixed_equiv(&self) -> &Self::Equiv;

    /// Converts a mutable reference into a mutable reference to the equivalent
    /// fixed-point type.
    fn as_fixed_equiv_mut(&mut self) -> &mut Self::Equiv;

    /// Converts an owned equivalent fixed-point type to this type.
    fn from_fixed_equiv(f: Self::Equiv) -> Self;

    /// Converts a reference to the equivalent fixed-point type into a reference
    /// to this type.
    fn ref_from_fixed_equiv(f: &Self::Equiv) -> &Self;

    /// Converts a mutable reference to the equivalent fixed-point type into a
    /// mutable reference to this type.
    fn mut_from_fixed_equiv(f: &mut Self::Equiv) -> &mut Self;
}

macro_rules! trait_delegate {
    (fn $method:ident($($param:ident: $Param:ty),*) -> $Ret:ty) => {
        #[inline]
        fn $method($($param: $Param),*) -> $Ret {
            Self::$method($($param),*)
        }
    };
    (fn $method:ident(self $(, $param:ident: $Param:ty)*) -> $Ret:ty) => {
        #[inline]
        fn $method(self $(, $param: $Param)*) -> $Ret {
            self.$method($($param),*)
        }
    };
    (fn $method:ident(&mut self $(, $param:ident: $Param:ty)*) $(-> $Ret:ty)*) => {
        #[inline]
        fn $method(&mut self $(, $param: $Param)*) $(-> $Ret)* {
            self.$method($($param),*)
        }
    };
    (fn $method:ident<$Gen:ident: $Trait:ident>($($param:ident: $Param:ty),*) -> $Ret:ty) => {
        #[inline]
        fn $method<$Gen: $Trait>($($param: $Param),*) -> $Ret {
            Self::$method($($param),*)
        }
    };
    (fn $method:ident<$Gen:ident: $Trait:ident>(self $(, $param:ident: $Param:ty)*) -> $Ret:ty) => {
        #[inline]
        fn $method<$Gen: $Trait>(self $(, $param: $Param)*) -> $Ret {
            self.$method($($param),*)
        }
    };
}

macro_rules! impl_fixed {
    (
        $Fixed:ident, $IFixed:ident, $UFixed:ident, $LeEqU:ident, $Bits:ident, $NonZeroBits:ident,
        $Signedness:tt
    ) => {
        impl<Frac: $LeEqU> FixedOptionalFeatures for $Fixed<Frac> {}

        impl<Frac: $LeEqU> Fixed for $Fixed<Frac> {
            type Bits = $Bits;
            type NonZeroBits = $NonZeroBits;
            type Bytes = [u8; mem::size_of::<$Bits>()];
            type Frac = Frac;
            type Signed = $IFixed<Frac>;
            type Unsigned = $UFixed<Frac>;
            const ZERO: Self = Self::ZERO;
            const DELTA: Self = Self::DELTA;
            const MIN: Self = Self::MIN;
            const MAX: Self = Self::MAX;
            const IS_SIGNED: bool = Self::IS_SIGNED;
            const INT_NBITS: u32 = Self::INT_NBITS;
            const FRAC_NBITS: u32 = Self::FRAC_NBITS;
            trait_delegate! { fn from_bits(bits: Self::Bits) -> Self }
            trait_delegate! { fn to_bits(self) -> Self::Bits }
            trait_delegate! { fn from_be(fixed: Self) -> Self }
            trait_delegate! { fn from_le(fixed: Self) -> Self }
            trait_delegate! { fn to_be(self) -> Self }
            trait_delegate! { fn to_le(self) -> Self }
            trait_delegate! { fn swap_bytes(self) -> Self }
            trait_delegate! { fn from_be_bytes(bits: Self::Bytes) -> Self }
            trait_delegate! { fn from_le_bytes(bits: Self::Bytes) -> Self }
            trait_delegate! { fn from_ne_bytes(bits: Self::Bytes) -> Self }
            trait_delegate! { fn to_be_bytes(self) -> Self::Bytes }
            trait_delegate! { fn to_le_bytes(self) -> Self::Bytes }
            trait_delegate! { fn to_ne_bytes(self) -> Self::Bytes }
            trait_delegate! { fn from_num<Src: ToFixed>(src: Src) -> Self }
            trait_delegate! { fn to_num<Dst: FromFixed>(self) -> Dst }
            trait_delegate! { fn checked_from_num<Src: ToFixed>(val: Src) -> Option<Self> }
            trait_delegate! { fn checked_to_num<Dst: FromFixed>(self) -> Option<Dst> }
            trait_delegate! { fn saturating_from_num<Src: ToFixed>(val: Src) -> Self }
            trait_delegate! { fn saturating_to_num<Dst: FromFixed>(self) -> Dst }
            trait_delegate! { fn wrapping_from_num<Src: ToFixed>(val: Src) -> Self }
            trait_delegate! { fn wrapping_to_num<Dst: FromFixed>(self) -> Dst }
            trait_delegate! { fn unwrapped_from_num<Src: ToFixed>(val: Src) -> Self }
            trait_delegate! { fn unwrapped_to_num<Dst: FromFixed>(self) -> Dst }
            trait_delegate! { fn overflowing_from_num<Src: ToFixed>(val: Src) -> (Self, bool) }
            trait_delegate! { fn overflowing_to_num<Dst: FromFixed>(self) -> (Dst, bool) }
            trait_delegate! { fn from_str_binary(src: &str) -> Result<Self, ParseFixedError> }
            trait_delegate! { fn from_str_octal(src: &str) -> Result<Self, ParseFixedError> }
            trait_delegate! { fn from_str_hex(src: &str) -> Result<Self, ParseFixedError> }
            trait_delegate! {
                fn saturating_from_str(src: &str) -> Result<Self, ParseFixedError>
            }
            trait_delegate! {
                fn saturating_from_str_binary(src: &str) -> Result<Self, ParseFixedError>
            }
            trait_delegate! {
                fn saturating_from_str_octal(src: &str) -> Result<Self, ParseFixedError>
            }
            trait_delegate! {
                fn saturating_from_str_hex(src: &str) -> Result<Self, ParseFixedError>
            }
            trait_delegate! {
                fn wrapping_from_str(src: &str) -> Result<Self, ParseFixedError>
            }
            trait_delegate! {
                fn wrapping_from_str_binary(src: &str) -> Result<Self, ParseFixedError>
            }
            trait_delegate! {
                fn wrapping_from_str_octal(src: &str) -> Result<Self, ParseFixedError>
            }
            trait_delegate! {
                fn wrapping_from_str_hex(src: &str) -> Result<Self, ParseFixedError>
            }
            trait_delegate! {
                fn overflowing_from_str(src: &str) -> Result<(Self, bool), ParseFixedError>
            }
            trait_delegate! {
                fn overflowing_from_str_binary(src: &str) -> Result<(Self, bool), ParseFixedError>
            }
            trait_delegate! {
                fn overflowing_from_str_octal(src: &str) -> Result<(Self, bool), ParseFixedError>
            }
            trait_delegate! {
                fn overflowing_from_str_hex(src: &str) -> Result<(Self, bool), ParseFixedError>
            }
            trait_delegate! { fn int(self) -> Self }
            trait_delegate! { fn frac(self) -> Self }
            trait_delegate! { fn ceil(self) -> Self }
            trait_delegate! { fn floor(self) -> Self }
            trait_delegate! { fn round_to_zero(self) -> Self }
            trait_delegate! { fn round(self) -> Self }
            trait_delegate! { fn round_ties_to_even(self) -> Self }
            trait_delegate! { fn checked_ceil(self) -> Option<Self> }
            trait_delegate! { fn checked_floor(self) -> Option<Self> }
            trait_delegate! { fn checked_round(self) -> Option<Self> }
            trait_delegate! { fn checked_round_ties_to_even(self) -> Option<Self> }
            trait_delegate! { fn saturating_ceil(self) -> Self }
            trait_delegate! { fn saturating_floor(self) -> Self }
            trait_delegate! { fn saturating_round(self) -> Self }
            trait_delegate! { fn saturating_round_ties_to_even(self) -> Self }
            trait_delegate! { fn wrapping_ceil(self) -> Self }
            trait_delegate! { fn wrapping_floor(self) -> Self }
            trait_delegate! { fn wrapping_round(self) -> Self }
            trait_delegate! { fn wrapping_round_ties_to_even(self) -> Self }
            trait_delegate! { fn unwrapped_ceil(self) -> Self }
            trait_delegate! { fn unwrapped_floor(self) -> Self }
            trait_delegate! { fn unwrapped_round(self) -> Self }
            trait_delegate! { fn unwrapped_round_ties_to_even(self) -> Self }
            trait_delegate! { fn overflowing_ceil(self) -> (Self, bool) }
            trait_delegate! { fn overflowing_floor(self) -> (Self, bool) }
            trait_delegate! { fn overflowing_round(self) -> (Self, bool) }
            trait_delegate! { fn overflowing_round_ties_to_even(self) -> (Self, bool) }
            trait_delegate! { fn count_ones(self) -> u32 }
            trait_delegate! { fn count_zeros(self) -> u32 }
            trait_delegate! { fn leading_ones(self) -> u32 }
            trait_delegate! { fn leading_zeros(self) -> u32 }
            trait_delegate! { fn trailing_ones(self) -> u32 }
            trait_delegate! { fn trailing_zeros(self) -> u32 }
            trait_delegate! { fn int_log2(self) -> i32 }
            trait_delegate! { fn int_log10(self) -> i32 }
            trait_delegate! { fn checked_int_log2(self) -> Option<i32> }
            trait_delegate! { fn checked_int_log10(self) -> Option<i32> }
            trait_delegate! { fn reverse_bits(self) -> Self }
            trait_delegate! { fn rotate_left(self, n: u32) -> Self }
            trait_delegate! { fn rotate_right(self, n: u32) -> Self }
            trait_delegate! { fn is_zero(self) -> bool }
            trait_delegate! { fn dist(self, other: Self) -> Self }
            trait_delegate! { fn mean(self, other: Self) -> Self }
            trait_delegate! { fn recip(self) -> Self }
            trait_delegate! { fn mul_add(self, mul: Self, add: Self) -> Self }
            trait_delegate! { fn mul_acc(&mut self, a: Self, b: Self) }
            trait_delegate! { fn div_euclid(self, rhs: Self) -> Self }
            trait_delegate! { fn rem_euclid(self, rhs: Self) -> Self }
            trait_delegate! { fn div_euclid_int(self, rhs: Self::Bits) -> Self }
            trait_delegate! { fn rem_euclid_int(self, rhs: Self::Bits) -> Self }
            trait_delegate! { fn lerp(self, start: Self, end: Self) -> Self }
            trait_delegate! { fn inv_lerp(self, start: Self, end: Self) -> Self }
            trait_delegate! { fn checked_neg(self) -> Option<Self> }
            trait_delegate! { fn checked_add(self, rhs: Self) -> Option<Self> }
            trait_delegate! { fn checked_sub(self, rhs: Self) -> Option<Self> }
            trait_delegate! { fn checked_mul(self, rhs: Self) -> Option<Self> }
            trait_delegate! { fn checked_div(self, rhs: Self) -> Option<Self> }
            trait_delegate! { fn checked_rem(self, rhs: Self) -> Option<Self> }
            trait_delegate! { fn checked_recip(self) -> Option<Self> }
            trait_delegate! { fn checked_mul_add(self, mul: Self, add: Self) -> Option<Self> }
            trait_delegate! { fn checked_mul_acc(&mut self, a: Self, b: Self) -> Option<()> }
            trait_delegate! { fn checked_div_euclid(self, rhs: Self) -> Option<Self> }
            trait_delegate! { fn checked_rem_euclid(self, rhs: Self) -> Option<Self> }
            trait_delegate! { fn checked_mul_int(self, rhs: Self::Bits) -> Option<Self> }
            trait_delegate! { fn checked_div_int(self, rhs: Self::Bits) -> Option<Self> }
            trait_delegate! { fn checked_rem_int(self, rhs: Self::Bits) -> Option<Self> }
            trait_delegate! { fn checked_div_euclid_int(self, rhs: Self::Bits) -> Option<Self> }
            trait_delegate! { fn checked_rem_euclid_int(self, rhs: Self::Bits) -> Option<Self> }
            trait_delegate! { fn checked_shl(self, rhs: u32) -> Option<Self> }
            trait_delegate! { fn checked_shr(self, rhs: u32) -> Option<Self> }
            trait_delegate! { fn checked_dist(self, other: Self) -> Option<Self> }
            trait_delegate! { fn checked_lerp(self, start: Self, end: Self) -> Option<Self> }
            trait_delegate! { fn checked_inv_lerp(self, start: Self, end: Self) -> Option<Self> }
            trait_delegate! { fn saturating_neg(self) -> Self }
            trait_delegate! { fn saturating_add(self, rhs: Self) -> Self }
            trait_delegate! { fn saturating_sub(self, rhs: Self) -> Self }
            trait_delegate! { fn saturating_mul(self, rhs: Self) -> Self }
            trait_delegate! { fn saturating_div(self, rhs: Self) -> Self }
            trait_delegate! { fn saturating_recip(self) -> Self }
            trait_delegate! { fn saturating_mul_add(self, mul: Self, add: Self) -> Self }
            trait_delegate! { fn saturating_mul_acc(&mut self, a: Self, b: Self) }
            trait_delegate! { fn saturating_div_euclid(self, rhs: Self) -> Self }
            trait_delegate! { fn saturating_mul_int(self, rhs: Self::Bits) -> Self }
            trait_delegate! { fn saturating_div_euclid_int(self, rhs: Self::Bits) -> Self }
            trait_delegate! { fn saturating_rem_euclid_int(self, rhs: Self::Bits) -> Self }
            trait_delegate! { fn saturating_dist(self, other: Self) -> Self }
            trait_delegate! { fn saturating_lerp(self, start: Self, end: Self) -> Self }
            trait_delegate! { fn saturating_inv_lerp(self, start: Self, end: Self) -> Self }
            trait_delegate! { fn wrapping_neg(self) -> Self }
            trait_delegate! { fn wrapping_add(self, rhs: Self) -> Self }
            trait_delegate! { fn wrapping_sub(self, rhs: Self) -> Self }
            trait_delegate! { fn wrapping_mul(self, rhs: Self) -> Self }
            trait_delegate! { fn wrapping_div(self, rhs: Self) -> Self }
            trait_delegate! { fn wrapping_recip(self) -> Self }
            trait_delegate! { fn wrapping_mul_add(self, mul: Self, add: Self) -> Self }
            trait_delegate! { fn wrapping_mul_acc(&mut self, a: Self, b: Self) }
            trait_delegate! { fn wrapping_div_euclid(self, rhs: Self) -> Self }
            trait_delegate! { fn wrapping_mul_int(self, rhs: Self::Bits) -> Self }
            trait_delegate! { fn wrapping_div_int(self, rhs: Self::Bits) -> Self }
            trait_delegate! { fn wrapping_div_euclid_int(self, rhs: Self::Bits) -> Self }
            trait_delegate! { fn wrapping_rem_euclid_int(self, rhs: Self::Bits) -> Self }
            trait_delegate! { fn wrapping_shl(self, rhs: u32) -> Self }
            trait_delegate! { fn wrapping_shr(self, rhs: u32) -> Self }
            trait_delegate! { fn wrapping_dist(self, other: Self) -> Self }
            trait_delegate! { fn wrapping_lerp(self, start: Self, end: Self) -> Self }
            trait_delegate! { fn wrapping_inv_lerp(self, start: Self, end: Self) -> Self }
            trait_delegate! { fn unwrapped_neg(self) -> Self }
            trait_delegate! { fn unwrapped_add(self, rhs: Self) -> Self }
            trait_delegate! { fn unwrapped_sub(self, rhs: Self) -> Self }
            trait_delegate! { fn unwrapped_mul(self, rhs: Self) -> Self }
            trait_delegate! { fn unwrapped_div(self, rhs: Self) -> Self }
            trait_delegate! { fn unwrapped_rem(self, rhs: Self) -> Self }
            trait_delegate! { fn unwrapped_recip(self) -> Self }
            trait_delegate! { fn unwrapped_mul_add(self, mul: Self, add: Self) -> Self }
            trait_delegate! { fn unwrapped_mul_acc(&mut self, a: Self, b: Self) }
            trait_delegate! { fn unwrapped_div_euclid(self, rhs: Self) -> Self }
            trait_delegate! { fn unwrapped_rem_euclid(self, rhs: Self) -> Self }
            trait_delegate! { fn unwrapped_mul_int(self, rhs: Self::Bits) -> Self }
            trait_delegate! { fn unwrapped_div_int(self, rhs: Self::Bits) -> Self }
            trait_delegate! { fn unwrapped_rem_int(self, rhs: Self::Bits) -> Self }
            trait_delegate! { fn unwrapped_div_euclid_int(self, rhs: Self::Bits) -> Self }
            trait_delegate! { fn unwrapped_rem_euclid_int(self, rhs: Self::Bits) -> Self }
            trait_delegate! { fn unwrapped_shl(self, rhs: u32) -> Self }
            trait_delegate! { fn unwrapped_shr(self, rhs: u32) -> Self }
            trait_delegate! { fn unwrapped_dist(self, other: Self) -> Self }
            trait_delegate! { fn unwrapped_lerp(self, start: Self, end: Self) -> Self }
            trait_delegate! { fn unwrapped_inv_lerp(self, start: Self, end: Self) -> Self }
            trait_delegate! { fn overflowing_neg(self) -> (Self, bool) }
            trait_delegate! { fn overflowing_add(self, rhs: Self) -> (Self, bool) }
            trait_delegate! { fn overflowing_sub(self, rhs: Self) -> (Self, bool) }
            trait_delegate! { fn overflowing_mul(self, rhs: Self) -> (Self, bool) }
            trait_delegate! { fn overflowing_div(self, rhs: Self) -> (Self, bool) }
            trait_delegate! { fn overflowing_recip(self) -> (Self, bool) }
            trait_delegate! { fn overflowing_mul_add(self, mul: Self, add: Self) -> (Self, bool) }
            trait_delegate! { fn overflowing_mul_acc(&mut self, a: Self, b: Self) -> bool }
            trait_delegate! { fn overflowing_div_euclid(self, rhs: Self) -> (Self, bool) }
            trait_delegate! { fn overflowing_mul_int(self, rhs: Self::Bits) -> (Self, bool) }
            trait_delegate! { fn overflowing_div_int(self, rhs: Self::Bits) -> (Self, bool) }
            trait_delegate! { fn overflowing_div_euclid_int(self, rhs: Self::Bits) -> (Self, bool) }
            trait_delegate! { fn overflowing_rem_euclid_int(self, rhs: Self::Bits) -> (Self, bool) }
            trait_delegate! { fn overflowing_shl(self, rhs: u32) -> (Self, bool) }
            trait_delegate! { fn overflowing_shr(self, rhs: u32) -> (Self, bool) }
            trait_delegate! { fn overflowing_dist(self, other: Self) -> (Self, bool) }
            trait_delegate! { fn overflowing_lerp(self, start: Self, end: Self) -> (Self, bool) }
            trait_delegate! {
                fn overflowing_inv_lerp(self, start: Self, end: Self) -> (Self, bool)
            }
        }

        impl<Frac: $LeEqU> FromFixed for $Fixed<Frac> {
            /// Converts a fixed-point number.
            ///
            /// Any extra fractional bits are discarded, which rounds towards −∞.
            ///
            /// # Panics
            ///
            /// When debug assertions are enabled, panics if the value
            /// does not fit. When debug assertions are not enabled,
            /// the wrapped value can be returned, but it is not
            /// considered a breaking change if in the future it
            /// panics; if wrapping is required use
            /// [`wrapping_from_fixed`] instead.
            ///
            /// [`wrapping_from_fixed`]: FromFixed::wrapping_from_fixed
            #[inline]
            fn from_fixed<F: Fixed>(src: F) -> Self {
                let (wrapped, overflow) = FromFixed::overflowing_from_fixed(src);
                maybe_assert!(!overflow, "{} overflows", src);
                let _ = overflow;
                wrapped
            }

            /// Converts a fixed-point number if it fits, otherwise returns [`None`].
            ///
            /// Any extra fractional bits are discarded, which rounds towards −∞.
            #[inline]
            fn checked_from_fixed<F: Fixed>(src: F) -> Option<Self> {
                match FromFixed::overflowing_from_fixed(src) {
                    (_, true) => None,
                    (wrapped, false) => Some(wrapped),
                }
            }

            /// Converts a fixed-point number, saturating if it does not fit.
            ///
            /// Any extra fractional bits are discarded, which rounds towards −∞.
            #[inline]
            fn saturating_from_fixed<F: Fixed>(src: F) -> Self {
                let conv = src.private_to_fixed_helper(Self::FRAC_NBITS, Self::INT_NBITS);
                if conv.overflow {
                    return if src < 0 { Self::MIN } else { Self::MAX };
                }
                let bits = if_signed_unsigned!(
                    $Signedness,
                    match conv.bits {
                        Widest::Unsigned(bits) => {
                            if (bits as $Bits) < 0 {
                                return Self::MAX;
                            }
                            bits as $Bits
                        }
                        Widest::Negative(bits) => bits as $Bits,
                    },
                    match conv.bits {
                        Widest::Unsigned(bits) => bits as $Bits,
                        Widest::Negative(_) => {
                            return Self::MIN;
                        }
                    },
                );
                Self::from_bits(bits)
            }

            /// Converts a fixed-point number, wrapping if it does not fit.
            ///
            /// Any extra fractional bits are discarded, which rounds towards −∞.
            #[inline]
            fn wrapping_from_fixed<F: Fixed>(src: F) -> Self {
                let (wrapped, _) = FromFixed::overflowing_from_fixed(src);
                wrapped
            }

            /// Converts a fixed-point number.
            ///
            /// Returns a [tuple] of the value and a [`bool`]
            /// indicating whether an overflow has occurred. On
            /// overflow, the wrapped value is returned.
            ///
            /// Any extra fractional bits are discarded, which rounds towards −∞.
            #[inline]
            fn overflowing_from_fixed<F: Fixed>(src: F) -> (Self, bool) {
                let conv = src.private_to_fixed_helper(Self::FRAC_NBITS, Self::INT_NBITS);
                let mut new_overflow = false;
                let bits = if_signed_unsigned!(
                    $Signedness,
                    match conv.bits {
                        Widest::Unsigned(bits) => {
                            if (bits as $Bits) < 0 {
                                new_overflow = true;
                            }
                            bits as $Bits
                        }
                        Widest::Negative(bits) => bits as $Bits,
                    },
                    match conv.bits {
                        Widest::Unsigned(bits) => bits as $Bits,
                        Widest::Negative(bits) => {
                            new_overflow = true;
                            bits as $Bits
                        }
                    },
                );
                (Self::from_bits(bits), conv.overflow || new_overflow)
            }

            /// Converts a fixed-point number, panicking if it does not fit.
            ///
            /// Any extra fractional bits are discarded, which rounds towards −∞.
            ///
            /// # Panics
            ///
            /// Panics if the value does not fit, even when debug
            /// assertions are not enabled.
            #[inline]
            fn unwrapped_from_fixed<F: Fixed>(src: F) -> Self {
                match FromFixed::overflowing_from_fixed(src) {
                    (val, false) => val,
                    (_, true) => panic!("overflow"),
                }
            }
        }

        impl<Frac: $LeEqU> ToFixed for $Fixed<Frac> {
            /// Converts a fixed-point number.
            ///
            /// Any extra fractional bits are discarded, which rounds towards −∞.
            ///
            /// # Panics
            ///
            /// When debug assertions are enabled, panics if the value
            /// does not fit. When debug assertions are not enabled,
            /// the wrapped value can be returned, but it is not
            /// considered a breaking change if in the future it
            /// panics; if wrapping is required use
            /// [`wrapping_to_fixed`] instead.
            ///
            /// [`wrapping_to_fixed`]: ToFixed::wrapping_to_fixed
            #[inline]
            fn to_fixed<F: Fixed>(self) -> F {
                FromFixed::from_fixed(self)
            }

            /// Converts a fixed-point number if it fits, otherwise returns [`None`].
            ///
            /// Any extra fractional bits are discarded, which rounds towards −∞.
            #[inline]
            fn checked_to_fixed<F: Fixed>(self) -> Option<F> {
                FromFixed::checked_from_fixed(self)
            }

            /// Converts a fixed-point number, saturating if it does not fit.
            ///
            /// Any extra fractional bits are discarded, which rounds towards −∞.
            #[inline]
            fn saturating_to_fixed<F: Fixed>(self) -> F {
                FromFixed::saturating_from_fixed(self)
            }

            /// Converts a fixed-point number, wrapping if it does not fit.
            ///
            /// Any extra fractional bits are discarded, which rounds towards −∞.
            #[inline]
            fn wrapping_to_fixed<F: Fixed>(self) -> F {
                FromFixed::wrapping_from_fixed(self)
            }

            /// Converts a fixed-point number.
            ///
            /// Returns a [tuple] of the value and a [`bool`]
            /// indicating whether an overflow has occurred. On
            /// overflow, the wrapped value is returned.
            ///
            /// Any extra fractional bits are discarded, which rounds towards −∞.
            #[inline]
            fn overflowing_to_fixed<F: Fixed>(self) -> (F, bool) {
                FromFixed::overflowing_from_fixed(self)
            }
            /// Converts a fixed-point number, panicking if it does not fit.
            ///
            /// Any extra fractional bits are discarded, which rounds towards −∞.
            ///
            /// # Panics
            ///
            /// Panics if the value does not fit, even when debug
            /// assertions are not enabled.
            #[inline]
            fn unwrapped_to_fixed<F: Fixed>(self) -> F {
                FromFixed::unwrapped_from_fixed(self)
            }
        }

        if_signed! {
            $Signedness;
            impl<Frac: $LeEqU> FixedSigned for $Fixed<Frac> {
                trait_delegate! { fn signed_bits(self) -> u32 }
                trait_delegate! { fn abs(self) -> Self }
                trait_delegate! { fn unsigned_abs(self) -> Self::Unsigned }
                trait_delegate! { fn unsigned_dist(self, other: Self) -> Self::Unsigned }
                trait_delegate! { fn signum(self) -> Self }
                trait_delegate! { fn checked_abs(self) -> Option<Self> }
                trait_delegate! { fn checked_signum(self) -> Option<Self> }
                trait_delegate! { fn saturating_abs(self) -> Self }
                trait_delegate! { fn saturating_signum(self) -> Self }
                trait_delegate! { fn wrapping_abs(self) -> Self }
                trait_delegate! { fn wrapping_signum(self) -> Self }
                trait_delegate! { fn unwrapped_abs(self) -> Self }
                trait_delegate! { fn unwrapped_signum(self) -> Self }
                trait_delegate! { fn overflowing_abs(self) -> (Self, bool) }
                trait_delegate! { fn overflowing_signum(self) -> (Self, bool) }
                trait_delegate! { fn is_positive(self) -> bool }
                trait_delegate! { fn is_negative(self) -> bool }
            }
        }

        if_unsigned! {
            $Signedness;
            impl<Frac: $LeEqU> FixedUnsigned for $Fixed<Frac> {
                trait_delegate! { fn significant_bits(self) -> u32 }
                trait_delegate! { fn is_power_of_two(self) -> bool }
                trait_delegate! { fn highest_one(self) -> Self }
                trait_delegate! { fn next_power_of_two(self) -> Self }
                trait_delegate! { fn checked_next_power_of_two(self) -> Option<Self> }
                trait_delegate! { fn wrapping_next_power_of_two(self) -> Self }
                trait_delegate! { fn unwrapped_next_power_of_two(self) -> Self }
            }
        }
    };
}

impl_fixed! { FixedI8, FixedI8, FixedU8, LeEqU8, i8, NonZeroI8, Signed }
impl_fixed! { FixedI16, FixedI16, FixedU16, LeEqU16, i16, NonZeroI16, Signed }
impl_fixed! { FixedI32, FixedI32, FixedU32, LeEqU32, i32, NonZeroI32, Signed }
impl_fixed! { FixedI64, FixedI64, FixedU64, LeEqU64, i64, NonZeroI64, Signed }
impl_fixed! { FixedI128, FixedI128, FixedU128, LeEqU128, i128, NonZeroI128, Signed }
impl_fixed! { FixedU8, FixedI8, FixedU8, LeEqU8, u8, NonZeroU8, Unsigned }
impl_fixed! { FixedU16, FixedI16, FixedU16, LeEqU16, u16, NonZeroU16, Unsigned }
impl_fixed! { FixedU32, FixedI32, FixedU32, LeEqU32, u32, NonZeroU32, Unsigned }
impl_fixed! { FixedU64, FixedI64, FixedU64, LeEqU64, u64, NonZeroU64, Unsigned }
impl_fixed! { FixedU128, FixedI128, FixedU128, LeEqU128, u128, NonZeroU128, Unsigned }
