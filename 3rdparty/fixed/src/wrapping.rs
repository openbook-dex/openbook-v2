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
    from_str::ParseFixedError,
    traits::{Fixed, FixedSigned, FixedUnsigned, FromFixed, ToFixed},
    types::extra::{LeEqU128, LeEqU16, LeEqU32, LeEqU64, LeEqU8},
    FixedI128, FixedI16, FixedI32, FixedI64, FixedI8, FixedU128, FixedU16, FixedU32, FixedU64,
    FixedU8,
};
use core::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    iter::{Product, Sum},
    mem,
    ops::{
        Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div,
        DivAssign, Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub,
        SubAssign,
    },
    str::FromStr,
};

/// Provides intentionally wrapped arithmetic on fixed-point numbers.
///
/// The underlying value can be retrieved through the `.0` index.
///
/// # Examples
///
/// ```rust
/// use fixed::{types::I16F16, Wrapping};
/// let max = Wrapping(I16F16::MAX);
/// let delta = Wrapping(I16F16::DELTA);
/// assert_eq!(I16F16::MIN, (max + delta).0);
/// ```
#[repr(transparent)]
#[derive(Clone, Copy, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Wrapping<F>(pub F);

impl<F: Fixed> Wrapping<F> {
    /// Zero.
    ///
    /// See also <code>FixedI32::[ZERO][FixedI32::ZERO]</code> and
    /// <code>FixedU32::[ZERO][FixedU32::ZERO]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// assert_eq!(Wrapping::<I16F16>::ZERO, Wrapping(I16F16::ZERO));
    /// ```
    pub const ZERO: Wrapping<F> = Wrapping(F::ZERO);

    /// The difference between any two successive representable numbers, <i>Δ</i>.
    ///
    /// See also <code>FixedI32::[DELTA][FixedI32::DELTA]</code> and
    /// <code>FixedU32::[DELTA][FixedU32::DELTA]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// assert_eq!(Wrapping::<I16F16>::DELTA, Wrapping(I16F16::DELTA));
    /// ```
    pub const DELTA: Wrapping<F> = Wrapping(F::DELTA);

    /// The smallest value that can be represented.
    ///
    /// See also <code>FixedI32::[MIN][FixedI32::MIN]</code> and
    /// <code>FixedU32::[MIN][FixedU32::MIN]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// assert_eq!(Wrapping::<I16F16>::MIN, Wrapping(I16F16::MIN));
    /// ```
    pub const MIN: Wrapping<F> = Wrapping(F::MIN);

    /// The largest value that can be represented.
    ///
    /// See also <code>FixedI32::[MAX][FixedI32::MAX]</code> and
    /// <code>FixedU32::[MAX][FixedU32::MAX]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// assert_eq!(Wrapping::<I16F16>::MAX, Wrapping(I16F16::MAX));
    /// ```
    pub const MAX: Wrapping<F> = Wrapping(F::MAX);

    /// [`true`] if the type is signed.
    ///
    /// See also <code>FixedI32::[IS\_SIGNED][FixedI32::IS_SIGNED]</code> and
    /// <code>FixedU32::[IS\_SIGNED][FixedU32::IS_SIGNED]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{
    ///     types::{I16F16, U16F16},
    ///     Wrapping,
    /// };
    /// assert!(Wrapping::<I16F16>::IS_SIGNED);
    /// assert!(!Wrapping::<U16F16>::IS_SIGNED);
    /// ```
    pub const IS_SIGNED: bool = F::IS_SIGNED;

    /// The number of integer bits.
    ///
    /// See also <code>FixedI32::[INT\_NBITS][FixedI32::INT_NBITS]</code> and
    /// <code>FixedU32::[INT\_NBITS][FixedU32::INT_NBITS]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// assert_eq!(Wrapping::<I16F16>::INT_NBITS, I16F16::INT_NBITS);
    /// ```
    pub const INT_NBITS: u32 = F::INT_NBITS;

    /// The number of fractional bits.
    ///
    /// See also <code>FixedI32::[FRAC\_NBITS][FixedI32::FRAC_NBITS]</code> and
    /// <code>FixedU32::[FRAC\_NBITS][FixedU32::FRAC_NBITS]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// assert_eq!(Wrapping::<I16F16>::FRAC_NBITS, I16F16::FRAC_NBITS);
    /// ```
    pub const FRAC_NBITS: u32 = F::FRAC_NBITS;

    /// Creates a fixed-point number that has a bitwise representation
    /// identical to the given integer.
    ///
    /// See also <code>FixedI32::[from\_bits][FixedI32::from_bits]</code> and
    /// <code>FixedU32::[from\_bits][FixedU32::from_bits]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// assert_eq!(Wrapping::<I16F16>::from_bits(0x1C), Wrapping(I16F16::from_bits(0x1C)));
    /// ```
    #[inline]
    pub fn from_bits(bits: F::Bits) -> Wrapping<F> {
        Wrapping(F::from_bits(bits))
    }

    /// Creates an integer that has a bitwise representation identical
    /// to the given fixed-point number.
    ///
    /// See also <code>FixedI32::[to\_bits][FixedI32::to_bits]</code> and
    /// <code>FixedU32::[to\_bits][FixedU32::to_bits]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// let w = Wrapping(I16F16::from_bits(0x1C));
    /// assert_eq!(w.to_bits(), 0x1C);
    /// ```
    #[inline]
    pub fn to_bits(self) -> F::Bits {
        self.0.to_bits()
    }

    /// Converts a fixed-point number from big endian to the target’s
    /// endianness.
    ///
    /// See also <code>FixedI32::[from\_be][FixedI32::from_be]</code> and
    /// <code>FixedU32::[from\_be][FixedU32::from_be]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// let w = Wrapping(I16F16::from_bits(0x1234_5678));
    /// if cfg!(target_endian = "big") {
    ///     assert_eq!(Wrapping::from_be(w), w);
    /// } else {
    ///     assert_eq!(Wrapping::from_be(w), w.swap_bytes());
    /// }
    /// ```
    #[inline]
    pub fn from_be(w: Self) -> Self {
        Wrapping(F::from_be(w.0))
    }

    /// Converts a fixed-point number from little endian to the
    /// target’s endianness.
    ///
    /// See also <code>FixedI32::[from\_le][FixedI32::from_le]</code> and
    /// <code>FixedU32::[from\_le][FixedU32::from_le]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// let w = Wrapping(I16F16::from_bits(0x1234_5678));
    /// if cfg!(target_endian = "little") {
    ///     assert_eq!(Wrapping::from_le(w), w);
    /// } else {
    ///     assert_eq!(Wrapping::from_le(w), w.swap_bytes());
    /// }
    /// ```
    #[inline]
    pub fn from_le(w: Self) -> Self {
        Wrapping(F::from_le(w.0))
    }

    /// Converts `self` to big endian from the target’s endianness.
    ///
    /// See also <code>FixedI32::[to\_be][FixedI32::to_be]</code> and
    /// <code>FixedU32::[to\_be][FixedU32::to_be]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// let w = Wrapping(I16F16::from_bits(0x1234_5678));
    /// if cfg!(target_endian = "big") {
    ///     assert_eq!(w.to_be(), w);
    /// } else {
    ///     assert_eq!(w.to_be(), w.swap_bytes());
    /// }
    /// ```
    #[inline]
    pub fn to_be(self) -> Self {
        Wrapping(self.0.to_be())
    }

    /// Converts `self` to little endian from the target’s endianness.
    ///
    /// See also <code>FixedI32::[to\_le][FixedI32::to_le]</code> and
    /// <code>FixedU32::[to\_le][FixedU32::to_le]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// let w = Wrapping(I16F16::from_bits(0x1234_5678));
    /// if cfg!(target_endian = "little") {
    ///     assert_eq!(w.to_le(), w);
    /// } else {
    ///     assert_eq!(w.to_le(), w.swap_bytes());
    /// }
    /// ```
    #[inline]
    pub fn to_le(self) -> Self {
        Wrapping(self.0.to_le())
    }

    /// Reverses the byte order of the fixed-point number.
    ///
    /// See also <code>FixedI32::[swap\_bytes][FixedI32::swap_bytes]</code> and
    /// <code>FixedU32::[swap\_bytes][FixedU32::swap_bytes]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// let w = Wrapping(I16F16::from_bits(0x1234_5678));
    /// let swapped = Wrapping(I16F16::from_bits(0x7856_3412));
    /// assert_eq!(w.swap_bytes(), swapped);
    /// ```
    #[inline]
    pub fn swap_bytes(self) -> Self {
        Wrapping(self.0.swap_bytes())
    }

    /// Creates a fixed-point number from its representation
    /// as a byte array in big endian.
    ///
    /// See also
    /// <code>FixedI32::[from\_be\_bytes][FixedI32::from_be_bytes]</code> and
    /// <code>FixedU32::[from\_be\_bytes][FixedU32::from_be_bytes]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// let bytes = [0x12, 0x34, 0x56, 0x78];
    /// assert_eq!(
    ///     Wrapping::<I16F16>::from_be_bytes(bytes),
    ///     Wrapping::<I16F16>::from_bits(0x1234_5678)
    /// );
    /// ```
    #[inline]
    pub fn from_be_bytes(bytes: F::Bytes) -> Self {
        Wrapping(F::from_be_bytes(bytes))
    }

    /// Creates a fixed-point number from its representation
    /// as a byte array in little endian.
    ///
    /// See also
    /// <code>FixedI32::[from\_le\_bytes][FixedI32::from_le_bytes]</code> and
    /// <code>FixedU32::[from\_le\_bytes][FixedU32::from_le_bytes]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// let bytes = [0x78, 0x56, 0x34, 0x12];
    /// assert_eq!(
    ///     Wrapping::<I16F16>::from_le_bytes(bytes),
    ///     Wrapping::<I16F16>::from_bits(0x1234_5678)
    /// );
    /// ```
    #[inline]
    pub fn from_le_bytes(bytes: F::Bytes) -> Self {
        Wrapping(F::from_le_bytes(bytes))
    }

    /// Creates a fixed-point number from its representation
    /// as a byte array in native endian.
    ///
    /// See also
    /// <code>FixedI32::[from\_ne\_bytes][FixedI32::from_ne_bytes]</code> and
    /// <code>FixedU32::[from\_ne\_bytes][FixedU32::from_ne_bytes]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// let bytes = if cfg!(target_endian = "big") {
    ///     [0x12, 0x34, 0x56, 0x78]
    /// } else {
    ///     [0x78, 0x56, 0x34, 0x12]
    /// };
    /// assert_eq!(
    ///     Wrapping::<I16F16>::from_ne_bytes(bytes),
    ///     Wrapping::<I16F16>::from_bits(0x1234_5678)
    /// );
    /// ```
    #[inline]
    pub fn from_ne_bytes(bytes: F::Bytes) -> Self {
        Wrapping(F::from_ne_bytes(bytes))
    }

    /// Returns the memory representation of this fixed-point
    /// number as a byte array in big-endian byte order.
    ///
    /// See also <code>FixedI32::[to\_be\_bytes][FixedI32::to_be_bytes]</code>
    /// and <code>FixedU32::[to\_be\_bytes][FixedU32::to_be_bytes]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// assert_eq!(
    ///     Wrapping::<I16F16>::from_bits(0x1234_5678).to_be_bytes(),
    ///     [0x12, 0x34, 0x56, 0x78]
    /// );
    /// ```
    #[inline]
    pub fn to_be_bytes(self) -> F::Bytes {
        self.0.to_be_bytes()
    }

    /// Returns the memory representation of this fixed-point
    /// number as a byte array in little-endian byte order.
    ///
    /// See also <code>FixedI32::[to\_le\_bytes][FixedI32::to_le_bytes]</code>
    /// and <code>FixedU32::[to\_le\_bytes][FixedU32::to_le_bytes]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// assert_eq!(
    ///     Wrapping::<I16F16>::from_bits(0x1234_5678).to_le_bytes(),
    ///     [0x78, 0x56, 0x34, 0x12]
    /// );
    /// ```
    #[inline]
    pub fn to_le_bytes(self) -> F::Bytes {
        self.0.to_le_bytes()
    }

    /// Returns the memory representation of this fixed-point
    /// number as a byte array in native-endian byte order.
    ///
    /// See also <code>FixedI32::[to\_ne\_bytes][FixedI32::to_ne_bytes]</code>
    /// and <code>FixedU32::[to\_ne\_bytes][FixedU32::to_ne_bytes]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// let bytes = if cfg!(target_endian = "big") {
    ///     [0x12, 0x34, 0x56, 0x78]
    /// } else {
    ///     [0x78, 0x56, 0x34, 0x12]
    /// };
    /// assert_eq!(
    ///     Wrapping::<I16F16>::from_bits(0x1234_5678).to_ne_bytes(),
    ///     bytes
    /// );
    /// ```
    #[inline]
    pub fn to_ne_bytes(self) -> F::Bytes {
        self.0.to_ne_bytes()
    }

    /// Wrapping conversion from another number.
    ///
    /// The other number can be:
    ///
    ///   * A fixed-point number. Any extra fractional bits are
    ///     discarded, which rounds towards −∞.
    ///   * An integer of type [`i8`], [`i16`], [`i32`], [`i64`], [`i128`],
    ///     [`isize`], [`u8`], [`u16`], [`u32`], [`u64`], [`u128`], or
    ///     [`usize`].
    ///   * A floating-point number of type [`f16`], [`bf16`],
    ///     [`f32`], [`f64`] or [`F128Bits`]. For this conversion, the
    ///     method rounds to the nearest, with ties rounding to even.
    ///   * Any other number `src` for which [`ToFixed`] is
    ///     implemented, in which case this method returns
    ///     <code>[Wrapping]\(src.[wrapping\_to\_fixed][ToFixed::wrapping_to_fixed]\())</code>.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_from\_num][FixedI32::wrapping_from_num]</code>
    /// and
    /// <code>FixedU32::[wrapping\_from\_num][FixedU32::wrapping_from_num]</code>.
    ///
    /// # Panics
    ///
    /// For floating-point numbers, panics if the value is not [finite].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{
    ///     types::{I4F4, I16F16},
    ///     Wrapping,
    /// };
    ///
    /// // 0x1234.5678 wraps into 0x4.5
    /// let src = I16F16::from_bits(0x1234_5678);
    /// let dst = Wrapping::<I4F4>::from_num(src);
    /// assert_eq!(dst, Wrapping(I4F4::from_bits(0x45)));
    ///
    /// // 0x1234 wraps into 0x4.0
    /// let src_int = 0x1234_i32;
    /// let dst_int = Wrapping::<I4F4>::from_num(src_int);
    /// assert_eq!(dst_int, Wrapping(I4F4::from_bits(0x40)));
    ///
    /// // 129.75 wrapped into 1.75 (binary 1.1100)
    /// let src_float = 129.75;
    /// let dst_float = Wrapping::<I4F4>::from_num(src_float);
    /// assert_eq!(dst_float, Wrapping(I4F4::from_bits(0b11100)));
    /// ```
    ///
    /// [`F128Bits`]: crate::F128Bits
    /// [`bf16`]: half::bf16
    /// [`f16`]: half::f16
    /// [finite]: f64::is_finite
    #[inline]
    pub fn from_num<Src: ToFixed>(src: Src) -> Wrapping<F> {
        Wrapping(src.wrapping_to_fixed())
    }

    /// Converts a fixed-point number to another number, wrapping the
    /// value on overflow.
    ///
    /// The other number can be:
    ///
    ///   * Another fixed-point number. Any extra fractional bits are
    ///     discarded, which rounds towards −∞.
    ///   * An integer of type [`i8`], [`i16`], [`i32`], [`i64`], [`i128`],
    ///     [`isize`], [`u8`], [`u16`], [`u32`], [`u64`], [`u128`], or
    ///     [`usize`]. Any fractional bits are discarded, which rounds
    ///     towards −∞.
    ///   * A floating-point number of type [`f16`], [`bf16`],
    ///     [`f32`], [`f64`] or [`F128Bits`]. For this conversion, the
    ///     method rounds to the nearest, with ties rounding to even.
    ///   * Any other type `Dst` for which [`FromFixed`] is
    ///     implemented, in which case this method returns
    ///     <code>Dst::[wrapping\_from\_fixed][FromFixed::wrapping_from_fixed]\(self.0)</code>.
    ///
    /// See also <code>FixedI32::[wrapping\_to\_num][FixedI32::wrapping_to_num]</code> and
    /// <code>FixedU32::[wrapping\_to\_num][FixedU32::wrapping_to_num]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{
    ///     types::{I16F16, I2F6, I4F4},
    ///     Wrapping,
    /// };
    ///
    /// // conversion that fits
    /// let src = Wrapping(I4F4::from_num(1.75));
    /// let expected = I16F16::from_num(1.75);
    /// assert_eq!(src.to_num::<I16F16>(), expected);
    ///
    /// // conversion that wraps
    /// let src = Wrapping(I4F4::MAX);
    /// let wrapped = I2F6::from_bits(I2F6::MAX.to_bits() << 2);
    /// assert_eq!(src.to_num::<I2F6>(), wrapped);
    /// ```
    ///
    /// [`F128Bits`]: crate::F128Bits
    /// [`bf16`]: half::bf16
    /// [`f16`]: half::f16
    #[inline]
    pub fn to_num<Dst: FromFixed>(self) -> Dst {
        Dst::wrapping_from_fixed(self.0)
    }

    /// Parses a string slice containing binary digits to return a fixed-point number.
    ///
    /// Rounding is to the nearest, with ties rounded to even.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_from\_str\_binary][FixedI32::wrapping_from_str_binary]</code>
    /// and
    /// <code>FixedU32::[wrapping\_from\_str\_binary][FixedU32::wrapping_from_str_binary]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I8F8, Wrapping};
    /// let check = Wrapping(I8F8::from_bits(0b1110001 << (8 - 1)));
    /// assert_eq!(Wrapping::<I8F8>::from_str_binary("101100111000.1"), Ok(check));
    /// ```
    #[inline]
    pub fn from_str_binary(src: &str) -> Result<Wrapping<F>, ParseFixedError> {
        F::wrapping_from_str_binary(src).map(Wrapping)
    }

    /// Parses a string slice containing octal digits to return a fixed-point number.
    ///
    /// Rounding is to the nearest, with ties rounded to even.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_from\_str\_octal][FixedI32::wrapping_from_str_octal]</code>
    /// and
    /// <code>FixedU32::[wrapping\_from\_str\_octal][FixedU32::wrapping_from_str_octal]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I8F8, Wrapping};
    /// let check = Wrapping(I8F8::from_bits(0o1654 << (8 - 3)));
    /// assert_eq!(Wrapping::<I8F8>::from_str_octal("7165.4"), Ok(check));
    /// ```
    #[inline]
    pub fn from_str_octal(src: &str) -> Result<Wrapping<F>, ParseFixedError> {
        F::wrapping_from_str_octal(src).map(Wrapping)
    }

    /// Parses a string slice containing hexadecimal digits to return a fixed-point number.
    ///
    /// Rounding is to the nearest, with ties rounded to even.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_from\_str\_hex][FixedI32::wrapping_from_str_hex]</code>
    /// and
    /// <code>FixedU32::[wrapping\_from\_str\_hex][FixedU32::wrapping_from_str_hex]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I8F8, Wrapping};
    /// let check = Wrapping(I8F8::from_bits(0xFFE));
    /// assert_eq!(Wrapping::<I8F8>::from_str_hex("C0F.FE"), Ok(check));
    /// ```
    #[inline]
    pub fn from_str_hex(src: &str) -> Result<Wrapping<F>, ParseFixedError> {
        F::wrapping_from_str_hex(src).map(Wrapping)
    }

    /// Returns the integer part.
    ///
    /// Note that since the numbers are stored in two’s complement,
    /// negative numbers with non-zero fractional parts will be
    /// rounded towards −∞, except in the case where there are no
    /// integer bits, for example for the type
    /// <code>[Wrapping]&lt;[I0F16]&gt;</code>, where the return value
    /// is always zero.
    ///
    /// See also <code>FixedI32::[int][FixedI32::int]</code> and
    /// <code>FixedU32::[int][FixedU32::int]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// assert_eq!(Wrapping(I16F16::from_num(12.25)).int(), Wrapping(I16F16::from_num(12)));
    /// assert_eq!(Wrapping(I16F16::from_num(-12.25)).int(), Wrapping(I16F16::from_num(-13)));
    /// ```
    ///
    /// [I0F16]: crate::types::I0F16
    #[inline]
    pub fn int(self) -> Wrapping<F> {
        Wrapping(self.0.int())
    }

    /// Returns the fractional part.
    ///
    /// Note that since the numbers are stored in two’s complement,
    /// the returned fraction will be non-negative for negative
    /// numbers, except in the case where there are no integer bits,
    /// for example for the type
    /// <code>[Wrapping]&lt;[I0F16]&gt;</code>,
    /// where the return value is always equal to `self`.
    ///
    /// See also <code>FixedI32::[frac][FixedI32::frac]</code> and
    /// <code>FixedU32::[frac][FixedU32::frac]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// assert_eq!(Wrapping(I16F16::from_num(12.25)).frac(), Wrapping(I16F16::from_num(0.25)));
    /// assert_eq!(Wrapping(I16F16::from_num(-12.25)).frac(), Wrapping(I16F16::from_num(0.75)));
    /// ```
    ///
    /// [I0F16]: crate::types::I0F16
    #[inline]
    pub fn frac(self) -> Wrapping<F> {
        Wrapping(self.0.frac())
    }

    /// Rounds to the next integer towards 0.
    ///
    /// See also
    /// <code>FixedI32::[round\_to\_zero][FixedI32::round_to_zero]</code> and
    /// <code>FixedU32::[round\_to\_zero][FixedU32::round_to_zero]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// let three = Wrapping(I16F16::from_num(3));
    /// assert_eq!(Wrapping(I16F16::from_num(3.9)).round_to_zero(), three);
    /// assert_eq!(Wrapping(I16F16::from_num(-3.9)).round_to_zero(), -three);
    /// ```
    #[inline]
    pub fn round_to_zero(self) -> Wrapping<F> {
        Wrapping(self.0.round_to_zero())
    }

    /// Wrapping ceil. Rounds to the next integer towards +∞, wrapping
    /// on overflow.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_ceil][FixedI32::wrapping_ceil]</code> and
    /// <code>FixedU32::[wrapping\_ceil][FixedU32::wrapping_ceil]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// let two_half = Wrapping(I16F16::from_num(5) / 2);
    /// assert_eq!(two_half.ceil(), Wrapping(I16F16::from_num(3)));
    /// assert_eq!(Wrapping(I16F16::MAX).ceil(), Wrapping(I16F16::MIN));
    /// ```
    #[inline]
    pub fn ceil(self) -> Wrapping<F> {
        Wrapping(self.0.wrapping_ceil())
    }

    /// Wrapping floor. Rounds to the next integer towards −∞,
    /// wrapping on overflow.
    ///
    /// Overflow can only occur for signed numbers with zero integer
    /// bits.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_floor][FixedI32::wrapping_floor]</code> and
    /// <code>FixedU32::[wrapping\_floor][FixedU32::wrapping_floor]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{
    ///     types::{I0F32, I16F16},
    ///     Wrapping,
    /// };
    /// let two_half = Wrapping(I16F16::from_num(5) / 2);
    /// assert_eq!(two_half.floor(), Wrapping(I16F16::from_num(2)));
    /// assert_eq!(Wrapping(I0F32::MIN).floor(), Wrapping(I0F32::ZERO));
    /// ```
    #[inline]
    pub fn floor(self) -> Wrapping<F> {
        Wrapping(self.0.wrapping_floor())
    }

    /// Wrapping round. Rounds to the next integer to the nearest,
    /// with ties rounded away from zero, and wrapping on overflow.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_round][FixedI32::wrapping_round]</code> and
    /// <code>FixedU32::[wrapping\_round][FixedU32::wrapping_round]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// let two_half = Wrapping(I16F16::from_num(5) / 2);
    /// assert_eq!(two_half.round(), Wrapping(I16F16::from_num(3)));
    /// assert_eq!((-two_half).round(), Wrapping(I16F16::from_num(-3)));
    /// assert_eq!(Wrapping(I16F16::MAX).round(), Wrapping(I16F16::MIN));
    /// ```
    #[inline]
    pub fn round(self) -> Wrapping<F> {
        Wrapping(self.0.wrapping_round())
    }

    /// Wrapping round. Rounds to the next integer to the nearest,
    /// with ties rounded to even, and wrapping on overflow.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_round\_ties\_to\_even][FixedI32::wrapping_round_ties_to_even]</code>
    /// and
    /// <code>FixedU32::[wrapping\_round\_ties\_to\_even][FixedU32::wrapping_round_ties_to_even]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// let two_half = Wrapping(I16F16::from_num(2.5));
    /// assert_eq!(two_half.round_ties_to_even(), Wrapping(I16F16::from_num(2)));
    /// let three_half = Wrapping(I16F16::from_num(3.5));
    /// assert_eq!(three_half.round_ties_to_even(), Wrapping(I16F16::from_num(4)));
    /// let max = Wrapping(I16F16::MAX);
    /// assert_eq!(max.round_ties_to_even(), Wrapping(I16F16::MIN));
    /// ```
    #[inline]
    pub fn round_ties_to_even(self) -> Wrapping<F> {
        Wrapping(self.0.wrapping_round_ties_to_even())
    }

    /// Returns the number of ones in the binary representation.
    ///
    /// See also <code>FixedI32::[count\_ones][FixedI32::count_ones]</code> and
    /// <code>FixedU32::[count\_ones][FixedU32::count_ones]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// let w = Wrapping(I16F16::from_bits(0x00FF_FF00));
    /// assert_eq!(w.count_ones(), w.0.count_ones());
    /// ```
    #[inline]
    pub fn count_ones(self) -> u32 {
        self.0.count_ones()
    }

    /// Returns the number of zeros in the binary representation.
    ///
    /// See also <code>FixedI32::[count\_zeros][FixedI32::count_zeros]</code>
    /// and <code>FixedU32::[count\_zeros][FixedU32::count_zeros]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// let w = Wrapping(I16F16::from_bits(0x00FF_FF00));
    /// assert_eq!(w.count_zeros(), w.0.count_zeros());
    /// ```
    #[inline]
    pub fn count_zeros(self) -> u32 {
        self.0.count_zeros()
    }

    /// Returns the number of leading ones in the binary representation.
    ///
    /// See also <code>FixedI32::[leading\_ones][FixedI32::leading_ones]</code>
    /// and <code>FixedU32::[leading\_ones][FixedU32::leading_ones]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::U16F16, Wrapping};
    /// let w = Wrapping(U16F16::from_bits(0xFF00_00FF));
    /// assert_eq!(w.leading_ones(), w.0.leading_ones());
    /// ```
    #[inline]
    pub fn leading_ones(self) -> u32 {
        self.0.leading_ones()
    }

    /// Returns the number of leading zeros in the binary representation.
    ///
    /// See also
    /// <code>FixedI32::[leading\_zeros][FixedI32::leading_zeros]</code> and
    /// <code>FixedU32::[leading\_zeros][FixedU32::leading_zeros]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// let w = Wrapping(I16F16::from_bits(0x00FF_FF00));
    /// assert_eq!(w.leading_zeros(), w.0.leading_zeros());
    /// ```
    #[inline]
    pub fn leading_zeros(self) -> u32 {
        self.0.leading_zeros()
    }

    /// Returns the number of trailing ones in the binary representation.
    ///
    /// See also
    /// <code>FixedI32::[trailing\_ones][FixedI32::trailing_ones]</code> and
    /// <code>FixedU32::[trailing\_ones][FixedU32::trailing_ones]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::U16F16, Wrapping};
    /// let w = Wrapping(U16F16::from_bits(0xFF00_00FF));
    /// assert_eq!(w.trailing_ones(), w.0.trailing_ones());
    /// ```
    #[inline]
    pub fn trailing_ones(self) -> u32 {
        self.0.trailing_ones()
    }

    /// Returns the number of trailing zeros in the binary representation.
    ///
    /// See also
    /// <code>FixedI32::[trailing\_zeros][FixedI32::trailing_zeros]</code> and
    /// <code>FixedU32::[trailing\_zeros][FixedU32::trailing_zeros]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// let w = Wrapping(I16F16::from_bits(0x00FF_FF00));
    /// assert_eq!(w.trailing_zeros(), w.0.trailing_zeros());
    /// ```
    #[inline]
    pub fn trailing_zeros(self) -> u32 {
        self.0.trailing_zeros()
    }

    /// Integer base-2 logarithm, rounded down.
    ///
    /// See also <code>FixedI32::[int\_log2][FixedI32::int_log2]</code> and
    /// <code>FixedU32::[int\_log2][FixedU32::int_log2]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the fixed-point number is ≤ 0.
    #[inline]
    pub fn int_log2(self) -> i32 {
        self.0.int_log2()
    }

    /// Integer base-10 logarithm, rounded down.
    ///
    /// See also <code>FixedI32::[int\_log10][FixedI32::int_log10]</code> and
    /// <code>FixedU32::[int\_log10][FixedU32::int_log10]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the fixed-point number is ≤ 0.
    #[inline]
    pub fn int_log10(self) -> i32 {
        self.0.int_log10()
    }

    /// Reverses the order of the bits of the fixed-point number.
    ///
    /// See also <code>FixedI32::[reverse\_bits][FixedI32::reverse_bits]</code>
    /// and <code>FixedU32::[reverse\_bits][FixedU32::reverse_bits]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// let i = I16F16::from_bits(0x1234_5678);
    /// assert_eq!(Wrapping(i).reverse_bits(), Wrapping(i.reverse_bits()));
    /// ```
    #[inline]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn reverse_bits(self) -> Wrapping<F> {
        Wrapping(self.0.reverse_bits())
    }

    /// Shifts to the left by `n` bits, wrapping the truncated bits to the right end.
    ///
    /// See also <code>FixedI32::[rotate\_left][FixedI32::rotate_left]</code>
    /// and <code>FixedU32::[rotate\_left][FixedU32::rotate_left]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// let i = I16F16::from_bits(0x00FF_FF00);
    /// assert_eq!(Wrapping(i).rotate_left(12), Wrapping(i.rotate_left(12)));
    /// ```
    #[inline]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn rotate_left(self, n: u32) -> Wrapping<F> {
        Wrapping(self.0.rotate_left(n))
    }

    /// Shifts to the right by `n` bits, wrapping the truncated bits to the left end.
    ///
    /// See also <code>FixedI32::[rotate\_right][FixedI32::rotate_right]</code>
    /// and <code>FixedU32::[rotate\_right][FixedU32::rotate_right]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// let i = I16F16::from_bits(0x00FF_FF00);
    /// assert_eq!(Wrapping(i).rotate_right(12), Wrapping(i.rotate_right(12)));
    /// ```
    #[inline]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn rotate_right(self, n: u32) -> Wrapping<F> {
        Wrapping(self.0.rotate_right(n))
    }

    /// Returns [`true`] if the number is zero.
    ///
    /// See also <code>FixedI32::[is\_zero][FixedI32::is_zero]</code> and
    /// <code>FixedU32::[is\_zero][FixedU32::is_zero]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// assert!(Wrapping(I16F16::ZERO).is_zero());
    /// assert!(!Wrapping(I16F16::from_num(4.3)).is_zero());
    /// ```
    #[inline]
    pub fn is_zero(self) -> bool {
        self.0.is_zero()
    }

    /// Returns the distance from `self` to `other`.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_dist][FixedI32::wrapping_dist]</code> and
    /// <code>FixedU32::[wrapping\_dist][FixedU32::wrapping_dist]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// type Wr = Wrapping<I16F16>;
    /// assert_eq!(Wr::from_num(-1).dist(Wr::from_num(4)), Wr::from_num(5));
    /// assert_eq!(Wr::MIN.dist(Wr::MAX), -Wr::DELTA);
    /// ```
    #[inline]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn dist(self, other: Wrapping<F>) -> Wrapping<F> {
        Wrapping(self.0.wrapping_dist(other.0))
    }

    /// Returns the mean of `self` and `other`.
    ///
    /// See also <code>FixedI32::[mean][FixedI32::mean]</code> and
    /// <code>FixedU32::[mean][FixedU32::mean]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// let three = Wrapping(I16F16::from_num(3));
    /// let four = Wrapping(I16F16::from_num(4));
    /// assert_eq!(three.mean(four), Wrapping(I16F16::from_num(3.5)));
    /// assert_eq!(three.mean(-four), Wrapping(I16F16::from_num(-0.5)));
    /// ```
    #[inline]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn mean(self, other: Wrapping<F>) -> Wrapping<F> {
        Wrapping(self.0.mean(other.0))
    }

    /// Returns the reciprocal (inverse), 1/`self`.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_recip][FixedI32::wrapping_recip]</code> and
    /// <code>FixedU32::[wrapping\_recip][FixedU32::wrapping_recip]</code>.
    ///
    /// # Panics
    ///
    /// Panics if `self` is zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I8F24, Wrapping};
    /// let quarter = Wrapping(I8F24::from_num(0.25));
    /// let frac_1_512 = Wrapping(I8F24::ONE / 512);
    /// assert_eq!(quarter.recip(), Wrapping(I8F24::from_num(4)));
    /// assert_eq!(frac_1_512.recip(), Wrapping(I8F24::ZERO));
    /// ```
    #[inline]
    pub fn recip(self) -> Wrapping<F> {
        Wrapping(self.0.wrapping_recip())
    }

    /// Multiply and add. Returns `self` × `mul` + `add`.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_mul\_add][FixedI32::wrapping_mul_add]</code>
    /// and
    /// <code>FixedU32::[wrapping\_mul\_add][FixedU32::wrapping_mul_add]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// let half = Wrapping(I16F16::from_num(0.5));
    /// let three = Wrapping(I16F16::from_num(3));
    /// let four = Wrapping(I16F16::from_num(4));
    /// let max = Wrapping(I16F16::MAX);
    /// assert_eq!(three.mul_add(half, four), Wrapping(I16F16::from_num(5.5)));
    /// assert_eq!(max.mul_add(three, max), Wrapping(I16F16::from_bits(!0 << 2)));
    /// ```
    #[inline]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn mul_add(self, mul: Wrapping<F>, add: Wrapping<F>) -> Wrapping<F> {
        Wrapping(self.0.wrapping_mul_add(mul.0, add.0))
    }

    /// Multiply and accumulate. Adds (`a` × `b`) to `self`.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_mul\_acc][FixedI32::wrapping_mul_acc]</code>
    /// and
    /// <code>FixedU32::[wrapping\_mul\_acc][FixedU32::wrapping_mul_acc]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// let mut acc = Wrapping(I16F16::from_num(3));
    /// acc.mul_acc(Wrapping(I16F16::from_num(4)), Wrapping(I16F16::from_num(0.5)));
    /// assert_eq!(acc, Wrapping(I16F16::from_num(5)));
    ///
    /// acc = Wrapping(I16F16::MAX);
    /// acc.mul_acc(Wrapping(I16F16::MAX), Wrapping(I16F16::from_num(3)));
    /// assert_eq!(acc, Wrapping(I16F16::MAX) * 4);
    /// ```
    #[inline]
    pub fn mul_acc(&mut self, a: Wrapping<F>, b: Wrapping<F>) {
        self.0.wrapping_mul_acc(a.0, b.0);
    }

    /// Euclidean division.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_div\_euclid][FixedI32::wrapping_div_euclid]</code>
    /// and
    /// <code>FixedU32::[wrapping\_div\_euclid][FixedU32::wrapping_div_euclid]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// let num = Wrapping(I16F16::from_num(7.5));
    /// let den = Wrapping(I16F16::from_num(2));
    /// assert_eq!(num.div_euclid(den), Wrapping(I16F16::from_num(3)));
    /// let quarter = Wrapping(I16F16::from_num(0.25));
    /// let check = (Wrapping(I16F16::MAX) * 4i32).round_to_zero();
    /// assert_eq!(Wrapping(I16F16::MAX).div_euclid(quarter), check);
    /// ```
    #[inline]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn div_euclid(self, divisor: Wrapping<F>) -> Wrapping<F> {
        Wrapping(self.0.wrapping_div_euclid(divisor.0))
    }

    /// Remainder for Euclidean division.
    ///
    /// See also <code>FixedI32::[rem\_euclid][FixedI32::rem_euclid]</code> and
    /// <code>FixedU32::[rem\_euclid][FixedU32::rem_euclid]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// let num = Wrapping(I16F16::from_num(7.5));
    /// let den = Wrapping(I16F16::from_num(2));
    /// assert_eq!(num.rem_euclid(den), Wrapping(I16F16::from_num(1.5)));
    /// assert_eq!((-num).rem_euclid(den), Wrapping(I16F16::from_num(0.5)));
    /// ```
    #[inline]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn rem_euclid(self, divisor: Wrapping<F>) -> Wrapping<F> {
        Wrapping(self.0.rem_euclid(divisor.0))
    }

    /// Euclidean division by an integer.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_div\_euclid\_int][FixedI32::wrapping_div_euclid_int]</code>
    /// and
    /// <code>FixedU32::[wrapping\_div\_euclid\_int][FixedU32::wrapping_div_euclid_int]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// let num = Wrapping(I16F16::from_num(7.5));
    /// assert_eq!(num.div_euclid_int(2), Wrapping(I16F16::from_num(3)));
    /// let min = Wrapping(I16F16::MIN);
    /// assert_eq!(min.div_euclid_int(-1), min);
    /// ```
    #[inline]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn div_euclid_int(self, divisor: F::Bits) -> Wrapping<F> {
        Wrapping(self.0.wrapping_div_euclid_int(divisor))
    }

    /// Remainder for Euclidean division.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_rem\_euclid\_int][FixedI32::wrapping_rem_euclid_int]</code>
    /// and
    /// <code>FixedU32::[wrapping\_rem\_euclid\_int][FixedU32::wrapping_rem_euclid_int]</code>.
    ///
    /// # Panics
    ///
    /// Panics if the divisor is zero.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// let num = Wrapping(I16F16::from_num(7.5));
    /// assert_eq!(num.rem_euclid_int(2), Wrapping(I16F16::from_num(1.5)));
    /// assert_eq!((-num).rem_euclid_int(2), Wrapping(I16F16::from_num(0.5)));
    /// ```
    #[inline]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn rem_euclid_int(self, divisor: F::Bits) -> Wrapping<F> {
        Wrapping(self.0.wrapping_rem_euclid_int(divisor))
    }

    /// Linear interpolation between `start` and `end`.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_lerp][FixedI32::wrapping_lerp]</code> and
    /// <code>FixedU32::[wrapping\_lerp][FixedU32::wrapping_lerp]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// type Wr = Wrapping<I16F16>;
    /// assert_eq!(Wr::from_num(0.5).lerp(Wr::ZERO, Wr::MAX), Wr::MAX / 2);
    /// assert_eq!(Wr::from_num(1.5).lerp(Wr::ZERO, Wr::MAX), Wr::MAX + Wr::MAX / 2);
    /// ```
    #[inline]
    pub fn lerp(self, start: Wrapping<F>, end: Wrapping<F>) -> Wrapping<F> {
        Wrapping(self.0.wrapping_lerp(start.0, end.0))
    }

    /// Inverse linear interpolation between `start` and `end`.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_inv\_lerp][FixedI32::wrapping_inv_lerp]</code> and
    /// <code>FixedU32::[wrapping\_inv\_lerp][FixedU32::wrapping_inv_lerp]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// type Wr = Wrapping<I16F16>;
    /// assert_eq!(
    ///     Wr::from_num(25).inv_lerp(Wr::from_num(20), Wr::from_num(40)),
    ///     Wr::from_num(0.25)
    /// );
    /// ```
    #[inline]
    pub fn inv_lerp(self, start: Wrapping<F>, end: Wrapping<F>) -> Wrapping<F> {
        Wrapping(self.0.wrapping_inv_lerp(start.0, end.0))
    }
}

impl<F: FixedSigned> Wrapping<F> {
    /// Returns the number of bits required to represent the value.
    ///
    /// The number of bits required includes an initial one for
    /// negative numbers, and an initial zero for non-negative
    /// numbers.
    ///
    /// See also <code>FixedI32::[signed\_bits][FixedI32::signed_bits]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I4F4, Wrapping};
    /// assert_eq!(Wrapping(I4F4::from_num(-3)).signed_bits(), 7);      // “_101.0000”
    /// assert_eq!(Wrapping(I4F4::from_num(-1)).signed_bits(), 5);      // “___1.0000”
    /// assert_eq!(Wrapping(I4F4::from_num(-0.0625)).signed_bits(), 1); // “____.___1”
    /// assert_eq!(Wrapping(I4F4::from_num(0)).signed_bits(), 1);       // “____.___0”
    /// assert_eq!(Wrapping(I4F4::from_num(0.0625)).signed_bits(), 2);  // “____.__01”
    /// assert_eq!(Wrapping(I4F4::from_num(1)).signed_bits(), 6);       // “__01.0000”
    /// assert_eq!(Wrapping(I4F4::from_num(3)).signed_bits(), 7);       // “_011.0000”
    /// ```
    #[inline]
    pub fn signed_bits(self) -> u32 {
        self.0.signed_bits()
    }

    /// Returns [`true`] if the number is > 0.
    ///
    /// See also <code>FixedI32::[is\_positive][FixedI32::is_positive]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// assert!(Wrapping(I16F16::from_num(4.3)).is_positive());
    /// assert!(!Wrapping(I16F16::ZERO).is_positive());
    /// assert!(!Wrapping(I16F16::from_num(-4.3)).is_positive());
    /// ```
    #[inline]
    pub fn is_positive(self) -> bool {
        self.0.is_positive()
    }

    /// Returns [`true`] if the number is < 0.
    ///
    /// See also <code>FixedI32::[is\_negative][FixedI32::is_negative]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// assert!(!Wrapping(I16F16::from_num(4.3)).is_negative());
    /// assert!(!Wrapping(I16F16::ZERO).is_negative());
    /// assert!(Wrapping(I16F16::from_num(-4.3)).is_negative());
    /// ```
    #[inline]
    pub fn is_negative(self) -> bool {
        self.0.is_negative()
    }

    /// Wrapping absolute value. Returns the absolute value, wrapping
    /// on overflow.
    ///
    /// Overflow can only occur when trying to find the absolute value
    /// of the minimum value.
    ///
    /// See also <code>FixedI32::[wrapping\_abs][FixedI32::wrapping_abs]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::I16F16, Wrapping};
    /// assert_eq!(Wrapping(I16F16::from_num(-5)).abs(), Wrapping(I16F16::from_num(5)));
    /// assert_eq!(Wrapping(I16F16::MIN).abs(), Wrapping(I16F16::MIN));
    /// ```
    #[inline]
    pub fn abs(self) -> Wrapping<F> {
        Wrapping(self.0.wrapping_abs())
    }

    /// Returns a number representing the sign of `self`.
    ///
    /// # Warning
    ///
    /// Using this method when 1 and −1 cannot be represented is
    /// almost certainly a bug, however, this is allowed and gives the
    /// following wrapped results.
    ///
    ///   * When there are no integer bits, for example for the type
    ///     <code>[Wrapping]&lt;[I0F16]&gt;</code>, the return value
    ///     is always zero.
    ///   * When there is one integer bit, for example for the type
    ///     <code>[Wrapping]&lt;[I1F15]&gt;</code>, the return value
    ///     is zero when `self` is zero, and −1 otherwise. This means
    ///     that for a positive number, −1 is returned, because +1
    ///     does not fit and is wrapped to −1.
    ///
    /// See also
    /// <code>FixedI32::[wrapping\_signum][FixedI32::wrapping_signum]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{
    ///     types::{I0F32, I1F31, I16F16},
    ///     Wrapping,
    /// };
    /// assert_eq!(Wrapping(<I16F16>::from_num(-3.9)).signum(), Wrapping(I16F16::from_num(-1)));
    /// assert_eq!(Wrapping(<I16F16>::ZERO).signum(), Wrapping(I16F16::ZERO));
    /// assert_eq!(Wrapping(<I16F16>::from_num(3.9)).signum(), Wrapping(I16F16::ONE));
    ///
    /// assert_eq!(Wrapping(<I1F31>::from_num(0.5)).signum(), Wrapping(I1F31::from_num(-1)));
    /// assert_eq!(Wrapping(<I0F32>::from_num(0.25)).signum(), Wrapping(I0F32::ZERO));
    /// assert_eq!(Wrapping(<I0F32>::from_num(-0.5)).signum(), Wrapping(I0F32::ZERO));
    /// ```
    ///
    /// [I0F16]: crate::types::I0F16
    /// [I1F15]: crate::types::I1F15
    #[inline]
    pub fn signum(self) -> Wrapping<F> {
        Wrapping(self.0.wrapping_signum())
    }
}

impl<F: FixedUnsigned> Wrapping<F> {
    /// Returns the number of bits required to represent the value.
    ///
    /// See also
    /// <code>FixedU32::[significant\_bits][FixedU32::significant_bits]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::U4F4, Wrapping};
    /// assert_eq!(Wrapping(U4F4::from_num(0)).significant_bits(), 0);      // “____.____”
    /// assert_eq!(Wrapping(U4F4::from_num(0.0625)).significant_bits(), 1); // “____.___1”
    /// assert_eq!(Wrapping(U4F4::from_num(1)).significant_bits(), 5);      // “___1.0000”
    /// assert_eq!(Wrapping(U4F4::from_num(3)).significant_bits(), 6);      // “__11.0000”
    /// ```
    #[inline]
    pub fn significant_bits(self) -> u32 {
        self.0.significant_bits()
    }

    /// Returns [`true`] if the fixed-point number is
    /// 2<sup><i>k</i></sup> for some integer <i>k</i>.
    ///
    /// See also
    /// <code>FixedU32::[is\_power\_of\_two][FixedU32::is_power_of_two]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::U16F16, Wrapping};
    /// assert!(Wrapping(U16F16::from_num(0.5)).is_power_of_two());
    /// assert!(Wrapping(U16F16::from_num(4)).is_power_of_two());
    /// assert!(!Wrapping(U16F16::from_num(5)).is_power_of_two());
    /// ```
    #[inline]
    pub fn is_power_of_two(self) -> bool {
        self.0.is_power_of_two()
    }

    /// Returns the highest one in the binary representation, or zero
    /// if `self` is zero.
    ///
    /// If `self` > 0, the highest one is equal to the largest power
    /// of two that is ≤ `self`.
    ///
    /// See also <code>FixedU32::[highest\_one][FixedU32::highest_one]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::U16F16, Wrapping};
    /// type T = Wrapping<U16F16>;
    /// assert_eq!(T::from_bits(0b11_0010).highest_one(), T::from_bits(0b10_0000));
    /// assert_eq!(T::from_num(0.3).highest_one(), T::from_num(0.25));
    /// assert_eq!(T::from_num(4).highest_one(), T::from_num(4));
    /// assert_eq!(T::from_num(6.5).highest_one(), T::from_num(4));
    /// assert_eq!(T::ZERO.highest_one(), T::ZERO);
    /// ```
    #[inline]
    pub fn highest_one(self) -> Wrapping<F> {
        Wrapping(self.0.highest_one())
    }

    /// Returns the smallest power of two that is ≥ `self`.
    ///
    /// If the next power of two is too large to fit, it is wrapped to zero.
    ///
    /// See also
    /// <code>FixedU32::[wrapping\_next\_power\_of\_two][FixedU32::wrapping_next_power_of_two]</code>.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fixed::{types::U16F16, Wrapping};
    /// type T = Wrapping<U16F16>;
    /// assert_eq!(T::from_bits(0b11_0010).next_power_of_two(), T::from_bits(0b100_0000));
    /// assert_eq!(T::from_num(0.3).next_power_of_two(), T::from_num(0.5));
    /// assert_eq!(T::from_num(4).next_power_of_two(), T::from_num(4));
    /// assert_eq!(T::from_num(6.5).next_power_of_two(), T::from_num(8));
    /// // if the next power of two is too large, it is wrapped to zero
    /// assert_eq!(T::MAX.next_power_of_two(), T::ZERO);
    /// ```
    #[inline]
    pub fn next_power_of_two(self) -> Wrapping<F> {
        Wrapping(self.0.wrapping_next_power_of_two())
    }
}

impl<F: Fixed> Display for Wrapping<F> {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        Display::fmt(&self.0, f)
    }
}

impl<F: Fixed> Debug for Wrapping<F> {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        Debug::fmt(&self.0, f)
    }
}

impl<F: Fixed> From<F> for Wrapping<F> {
    /// Wraps a fixed-point number.
    #[inline]
    fn from(src: F) -> Wrapping<F> {
        Wrapping(src)
    }
}

impl<F: Fixed> FromStr for Wrapping<F> {
    type Err = ParseFixedError;
    /// Parses a string slice containing decimal digits to return a fixed-point number.
    ///
    /// Rounding is to the nearest, with ties rounded to even.
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        F::wrapping_from_str(s).map(Wrapping)
    }
}

macro_rules! op {
    ($wrapping:ident, $Op:ident $op:ident, $OpAssign:ident $op_assign:ident) => {
        impl<F: Fixed> $Op<Wrapping<F>> for Wrapping<F> {
            type Output = Wrapping<F>;
            #[inline]
            fn $op(self, other: Wrapping<F>) -> Wrapping<F> {
                Wrapping((self.0).$wrapping(other.0))
            }
        }
        impl<F: Fixed> $Op<Wrapping<F>> for &Wrapping<F> {
            type Output = Wrapping<F>;
            #[inline]
            fn $op(self, other: Wrapping<F>) -> Wrapping<F> {
                Wrapping((self.0).$wrapping(other.0))
            }
        }
        impl<F: Fixed> $Op<&Wrapping<F>> for Wrapping<F> {
            type Output = Wrapping<F>;
            #[inline]
            fn $op(self, other: &Wrapping<F>) -> Wrapping<F> {
                Wrapping((self.0).$wrapping(other.0))
            }
        }
        impl<F: Fixed> $Op<&Wrapping<F>> for &Wrapping<F> {
            type Output = Wrapping<F>;
            #[inline]
            fn $op(self, other: &Wrapping<F>) -> Wrapping<F> {
                Wrapping((self.0).$wrapping(other.0))
            }
        }
        impl<F: Fixed> $OpAssign<Wrapping<F>> for Wrapping<F> {
            #[inline]
            fn $op_assign(&mut self, other: Wrapping<F>) {
                self.0 = (self.0).$wrapping(other.0);
            }
        }
        impl<F: Fixed> $OpAssign<&Wrapping<F>> for Wrapping<F> {
            #[inline]
            fn $op_assign(&mut self, other: &Wrapping<F>) {
                self.0 = (self.0).$wrapping(other.0);
            }
        }
    };
}

macro_rules! op_bitwise {
    ($Op:ident $op:ident, $OpAssign:ident $op_assign:ident) => {
        impl<F> $Op<Wrapping<F>> for Wrapping<F>
        where
            F: $Op<F, Output = F>,
        {
            type Output = Wrapping<F>;
            #[inline]
            fn $op(self, other: Wrapping<F>) -> Wrapping<F> {
                Wrapping((self.0).$op(other.0))
            }
        }
        impl<F> $Op<Wrapping<F>> for &Wrapping<F>
        where
            for<'a> &'a F: $Op<F, Output = F>,
        {
            type Output = Wrapping<F>;
            #[inline]
            fn $op(self, other: Wrapping<F>) -> Wrapping<F> {
                Wrapping((self.0).$op(other.0))
            }
        }
        impl<F> $Op<&Wrapping<F>> for Wrapping<F>
        where
            for<'a> F: $Op<&'a F, Output = F>,
        {
            type Output = Wrapping<F>;
            #[inline]
            fn $op(self, other: &Wrapping<F>) -> Wrapping<F> {
                Wrapping((self.0).$op(&other.0))
            }
        }
        impl<F> $Op<&Wrapping<F>> for &Wrapping<F>
        where
            for<'a, 'b> &'a F: $Op<&'b F, Output = F>,
        {
            type Output = Wrapping<F>;
            #[inline]
            fn $op(self, other: &Wrapping<F>) -> Wrapping<F> {
                Wrapping((self.0).$op(&other.0))
            }
        }
        impl<F> $OpAssign<Wrapping<F>> for Wrapping<F>
        where
            F: $OpAssign<F>,
        {
            #[inline]
            fn $op_assign(&mut self, other: Wrapping<F>) {
                (self.0).$op_assign(other.0);
            }
        }
        impl<F> $OpAssign<&Wrapping<F>> for Wrapping<F>
        where
            for<'a> F: $OpAssign<&'a F>,
        {
            #[inline]
            fn $op_assign(&mut self, other: &Wrapping<F>) {
                (self.0).$op_assign(&other.0);
            }
        }
    };
}

macro_rules! op_shift {
    (
        $Op:ident $op:ident, $OpAssign:ident $op_assign:ident;
        $($Rhs:ident),*
    ) => { $(
        impl<F> $Op<$Rhs> for Wrapping<F>
        where
            F: $Op<u32, Output = F>,
        {
            type Output = Wrapping<F>;
            #[inline]
            fn $op(self, other: $Rhs) -> Wrapping<F> {
                let nbits = mem::size_of::<F>() as u32 * 8;
                Wrapping((self.0).$op(other as u32 % nbits))
            }
        }
        impl<F> $Op<$Rhs> for &Wrapping<F>
        where
            for<'a> &'a F: $Op<u32, Output = F>,
        {
            type Output = Wrapping<F>;
            #[inline]
            fn $op(self, other: $Rhs) -> Wrapping<F> {
                let nbits = mem::size_of::<F>() as u32 * 8;
                Wrapping((self.0).$op(other as u32 % nbits))
            }
        }
        impl<F> $Op<&$Rhs> for Wrapping<F>
        where
            F: $Op<u32, Output = F>,
        {
            type Output = Wrapping<F>;
            #[inline]
            fn $op(self, other: &$Rhs) -> Wrapping<F> {
                let nbits = mem::size_of::<F>() as u32 * 8;
                Wrapping((self.0).$op(*other as u32 % nbits))
            }
        }
        impl<F> $Op<&$Rhs> for &Wrapping<F>
        where
            for<'a> &'a F: $Op<u32, Output = F>,
        {
            type Output = Wrapping<F>;
            #[inline]
            fn $op(self, other: &$Rhs) -> Wrapping<F> {
                let nbits = mem::size_of::<F>() as u32 * 8;
                Wrapping((self.0).$op(*other as u32 % nbits))
            }
        }
        impl<F> $OpAssign<$Rhs> for Wrapping<F>
        where
            F: $OpAssign<u32>,
        {
            #[inline]
            fn $op_assign(&mut self, other: $Rhs) {
                let nbits = mem::size_of::<F>() as u32 * 8;
                (self.0).$op_assign(other as u32 % nbits);
            }
        }
        impl<F> $OpAssign<&$Rhs> for Wrapping<F>
        where
            F: $OpAssign<u32>,
        {
            #[inline]
            fn $op_assign(&mut self, other: &$Rhs) {
                let nbits = mem::size_of::<F>() as u32 * 8;
                (self.0).$op_assign(*other as u32 % nbits);
            }
        }
    )* };
}

impl<F: Fixed> Neg for Wrapping<F> {
    type Output = Wrapping<F>;
    #[inline]
    fn neg(self) -> Wrapping<F> {
        Wrapping((self.0).wrapping_neg())
    }
}

impl<F: Fixed> Neg for &Wrapping<F> {
    type Output = Wrapping<F>;
    #[inline]
    fn neg(self) -> Wrapping<F> {
        Wrapping((self.0).wrapping_neg())
    }
}
op! { wrapping_add, Add add, AddAssign add_assign }
op! { wrapping_sub, Sub sub, SubAssign sub_assign }
op! { wrapping_mul, Mul mul, MulAssign mul_assign }
op! { wrapping_div, Div div, DivAssign div_assign }
op! { rem, Rem rem, RemAssign rem_assign }

impl<F> Not for Wrapping<F>
where
    F: Not<Output = F>,
{
    type Output = Wrapping<F>;
    #[inline]
    fn not(self) -> Wrapping<F> {
        Wrapping((self.0).not())
    }
}
impl<F> Not for &Wrapping<F>
where
    for<'a> &'a F: Not<Output = F>,
{
    type Output = Wrapping<F>;
    #[inline]
    fn not(self) -> Wrapping<F> {
        Wrapping((self.0).not())
    }
}
op_bitwise! { BitAnd bitand, BitAndAssign bitand_assign }
op_bitwise! { BitOr bitor, BitOrAssign bitor_assign }
op_bitwise! { BitXor bitxor, BitXorAssign bitxor_assign }

op_shift! {
    Shl shl, ShlAssign shl_assign;
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
}
op_shift! {
    Shr shr, ShrAssign shr_assign;
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
}

impl<F: Fixed> Sum<Wrapping<F>> for Wrapping<F> {
    fn sum<I>(iter: I) -> Wrapping<F>
    where
        I: Iterator<Item = Wrapping<F>>,
    {
        iter.fold(Wrapping(F::ZERO), Add::add)
    }
}

impl<'a, F: 'a + Fixed> Sum<&'a Wrapping<F>> for Wrapping<F> {
    fn sum<I>(iter: I) -> Wrapping<F>
    where
        I: Iterator<Item = &'a Wrapping<F>>,
    {
        iter.fold(Wrapping(F::ZERO), Add::add)
    }
}

impl<F: Fixed> Product<Wrapping<F>> for Wrapping<F> {
    fn product<I>(mut iter: I) -> Wrapping<F>
    where
        I: Iterator<Item = Wrapping<F>>,
    {
        match iter.next() {
            None => Wrapping(1.wrapping_to_fixed()),
            Some(first) => iter.fold(first, Mul::mul),
        }
    }
}

impl<'a, F: 'a + Fixed> Product<&'a Wrapping<F>> for Wrapping<F> {
    fn product<I>(mut iter: I) -> Wrapping<F>
    where
        I: Iterator<Item = &'a Wrapping<F>>,
    {
        match iter.next() {
            None => Wrapping(1.wrapping_to_fixed()),
            Some(first) => iter.fold(*first, Mul::mul),
        }
    }
}

// The following cannot be implemented for Wrapping<F> where F: Fixed,
// otherwise there will be a conflicting implementation error. For
// example we cannot implement both these without triggering E0119:
//
//     impl<F: Fixed> Op<F::Bits> for Wrapping<F> { /* ... */ }
//     impl<F: Fixed> Op<&F::Bits> for Wrapping<F> { /* ... */ }
//
// To work around this, we provide implementations like this:
//
//     impl<Frac> Op<i8> for Wrapping<FixedI8<Frac>> { /* ... */ }
//     impl<Frac> Op<&i8> for Wrapping<FixedI8<Frac>> { /* ... */ }
//     impl<Frac> Op<i16> for Wrapping<FixedI16<Frac>> { /* ... */ }
//     impl<Frac> Op<&i16> for Wrapping<FixedI16<Frac>> { /* ... */ }
//     ...

macro_rules! op_bits {
    (
        $Fixed:ident($Bits:ident $(, $LeEqU:ident)*)::$wrapping:ident,
        $Op:ident $op:ident,
        $OpAssign:ident $op_assign:ident
    ) => {
        impl<Frac $(: $LeEqU)*> $Op<$Bits> for Wrapping<$Fixed<Frac>> {
            type Output = Wrapping<$Fixed<Frac>>;
            #[inline]
            fn $op(self, other: $Bits) -> Wrapping<$Fixed<Frac>> {
                Wrapping((self.0).$wrapping(other))
            }
        }
        impl<Frac $(: $LeEqU)*> $Op<$Bits> for &Wrapping<$Fixed<Frac>> {
            type Output = Wrapping<$Fixed<Frac>>;
            #[inline]
            fn $op(self, other: $Bits) -> Wrapping<$Fixed<Frac>> {
                Wrapping((self.0).$wrapping(other))
            }
        }
        impl<Frac $(: $LeEqU)*> $Op<&$Bits> for Wrapping<$Fixed<Frac>> {
            type Output = Wrapping<$Fixed<Frac>>;
            #[inline]
            fn $op(self, other: &$Bits) -> Wrapping<$Fixed<Frac>> {
                Wrapping((self.0).$wrapping(*other))
            }
        }
        impl<Frac $(: $LeEqU)*> $Op<&$Bits> for &Wrapping<$Fixed<Frac>> {
            type Output = Wrapping<$Fixed<Frac>>;
            #[inline]
            fn $op(self, other: &$Bits) -> Wrapping<$Fixed<Frac>> {
                Wrapping((self.0).$wrapping(*other))
            }
        }
        impl<Frac $(: $LeEqU)*> $OpAssign<$Bits> for Wrapping<$Fixed<Frac>> {
            #[inline]
            fn $op_assign(&mut self, other: $Bits) {
                self.0 = (self.0).$wrapping(other);
            }
        }
        impl<Frac $(: $LeEqU)*> $OpAssign<&$Bits> for Wrapping<$Fixed<Frac>> {
            #[inline]
            fn $op_assign(&mut self, other: &$Bits) {
                self.0 = (self.0).$wrapping(*other);
            }
        }
    };
}

macro_rules! ops {
    ($Fixed:ident($Bits:ident, $LeEqU:ident)) => {
        op_bits! { $Fixed($Bits)::wrapping_mul_int, Mul mul, MulAssign mul_assign }
        op_bits! { $Fixed($Bits)::wrapping_div_int, Div div, DivAssign div_assign }
        op_bits! { $Fixed($Bits, $LeEqU)::rem, Rem rem, RemAssign rem_assign }
    };
}
ops! { FixedI8(i8, LeEqU8) }
ops! { FixedI16(i16, LeEqU16) }
ops! { FixedI32(i32, LeEqU32) }
ops! { FixedI64(i64, LeEqU64) }
ops! { FixedI128(i128, LeEqU128) }
ops! { FixedU8(u8, LeEqU8) }
ops! { FixedU16(u16, LeEqU16) }
ops! { FixedU32(u32, LeEqU32) }
ops! { FixedU64(u64, LeEqU64) }
ops! { FixedU128(u128, LeEqU128) }
