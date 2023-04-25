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
    debug_hex::{self, IsDebugHex},
    int_helper,
    types::extra::{LeEqU128, LeEqU16, LeEqU32, LeEqU64, LeEqU8},
    FixedI128, FixedI16, FixedI32, FixedI64, FixedI8, FixedU128, FixedU16, FixedU32, FixedU64,
    FixedU8,
};
use az_crate::{WrappingAs, WrappingCast};
use core::{
    cmp::{self, Ordering},
    fmt::{
        Alignment, Binary, Debug, Display, Formatter, LowerHex, Octal, Result as FmtResult,
        UpperHex,
    },
    ops::{Add, Shl, Shr},
    str,
};

// We need 130 bytes: 128 digits, one radix point, one leading zero.
//
// The leading zero has two purposes:
//
//  1. If there are no integer digits, we still want to start with "0.".
//  2. If rounding causes a carry, we can overflow into this extra zero.
//
// In the end the layout should be:
//
//   * data[0..int_digits + 1]: integer digits with potentially one extra zero
//   * data[int_digits + 1..int_digits + 2]: '.'
//   * data[int_digits + 2..int_digits + frac_digits + 2]: fractional digits
struct Buffer {
    int_digits: usize,
    frac_digits: usize,
    data: [u8; 130],
}

impl Buffer {
    fn new() -> Buffer {
        Buffer {
            int_digits: 0,
            frac_digits: 0,
            data: [0; 130],
        }
    }

    // Do not combine with new to avoid copying data, otherwise the
    // buffer will be created, modified with the '.', then copied.
    fn set_len(&mut self, int_digits: u32, frac_digits: u32) {
        assert!(int_digits + frac_digits < 130, "out of bounds");
        self.int_digits = int_digits as usize;
        self.frac_digits = frac_digits as usize;
        self.data[1 + self.int_digits] = b'.';
    }

    // does not include leading zero
    fn int(&mut self) -> &mut [u8] {
        let begin = 1;
        let end = begin + self.int_digits;
        &mut self.data[begin..end]
    }

    fn frac(&mut self) -> &mut [u8] {
        let begin = 1 + self.int_digits + 1;
        let end = begin + self.frac_digits;
        &mut self.data[begin..end]
    }

    fn finish(
        &mut self,
        radix: Radix,
        is_neg: bool,
        frac_rem_cmp_msb: Ordering,
        fmt: &mut Formatter,
    ) -> FmtResult {
        self.round_and_trim(radix.max(), frac_rem_cmp_msb);
        self.encode_digits(radix == Radix::UpHex);
        self.pad_and_print(is_neg, radix.prefix(), fmt)
    }

    fn round_and_trim(&mut self, max: u8, frac_rem_cmp_msb: Ordering) {
        let len = if self.frac_digits > 0 {
            self.int_digits + self.frac_digits + 2
        } else {
            self.int_digits + 1
        };

        let round_up = frac_rem_cmp_msb == Ordering::Greater
            || frac_rem_cmp_msb == Ordering::Equal && (self.data[len - 1] & 1 != 0);
        if round_up {
            for b in self.data[0..len].iter_mut().rev() {
                if *b < max {
                    *b += 1;
                    break;
                }
                if *b == b'.' {
                    maybe_assert!(self.frac_digits == 0);
                    continue;
                }
                *b = 0;
                if self.frac_digits > 0 {
                    self.frac_digits -= 1;
                }
            }
        } else {
            let mut trim = 0;
            for b in self.frac().iter().rev() {
                if *b != 0 {
                    break;
                }
                trim += 1;
            }
            self.frac_digits -= trim;
        }
    }

    fn encode_digits(&mut self, upper: bool) {
        for digit in self.data[..self.int_digits + self.frac_digits + 2].iter_mut() {
            if *digit < 10 {
                *digit += b'0';
            } else if *digit < 16 {
                *digit += if upper { b'A' - 10 } else { b'a' - 10 };
            }
        }
    }

    fn pad_and_print(&self, is_neg: bool, maybe_prefix: &str, fmt: &mut Formatter) -> FmtResult {
        use core::fmt::Write;

        let sign = if is_neg {
            "-"
        } else if fmt.sign_plus() {
            "+"
        } else {
            ""
        };
        let prefix = if fmt.alternate() { maybe_prefix } else { "" };

        // For numbers with no significant integer bits:
        //   * data starts  with "0." and begin = 0.
        //
        // For numbers with some significant integer bits, data can have:
        //   * no leading zeros => begin = 0
        //   * one leading zero => begin = 1
        //   * two leading zeros => begin = 2
        //
        // Two leading zeros can happen for decimal only. For example
        // with four significant integer bits, we could get anything
        // between 8 and 15, so two decimal digits are allocated apart
        // from the initial padding zero. This means that for 8, data
        // would begin as "008.", and begin = 2.
        let abs_begin = if self.data[0] != b'0' || self.data[1] == b'.' {
            0
        } else if self.data[1] == b'0' {
            2
        } else {
            1
        };
        let end_zeros = fmt.precision().map(|x| x - self.frac_digits).unwrap_or(0);
        let abs_end = if self.frac_digits > 0 {
            self.int_digits + self.frac_digits + 2
        } else if end_zeros > 0 {
            self.int_digits + 2
        } else {
            self.int_digits + 1
        };

        let req_width = sign.len() + prefix.len() + abs_end - abs_begin + end_zeros;
        let pad = fmt
            .width()
            .and_then(|w| w.checked_sub(req_width))
            .unwrap_or(0);
        let (pad_left, pad_zeros, pad_right) = if fmt.sign_aware_zero_pad() {
            (0, pad, 0)
        } else {
            match fmt.align() {
                Some(Alignment::Left) => (0, 0, pad),
                Some(Alignment::Center) => (pad / 2, 0, pad - pad / 2),
                None | Some(Alignment::Right) => (pad, 0, 0),
            }
        };
        let fill = fmt.fill();

        for _ in 0..pad_left {
            fmt.write_char(fill)?;
        }
        fmt.write_str(sign)?;
        fmt.write_str(prefix)?;
        for _ in 0..pad_zeros {
            fmt.write_char('0')?;
        }
        fmt.write_str(str::from_utf8(&self.data[abs_begin..abs_end]).unwrap())?;
        for _ in 0..end_zeros {
            fmt.write_char('0')?;
        }
        for _ in 0..pad_right {
            fmt.write_char(fill)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Radix {
    Bin,
    Oct,
    LowHex,
    UpHex,
    Dec,
}
impl Radix {
    fn digit_bits(self) -> u32 {
        match self {
            Radix::Bin => 1,
            Radix::Oct => 3,
            Radix::LowHex => 4,
            Radix::UpHex => 4,
            Radix::Dec => 4,
        }
    }
    fn max(self) -> u8 {
        match self {
            Radix::Bin => 1,
            Radix::Oct => 7,
            Radix::LowHex => 15,
            Radix::UpHex => 15,
            Radix::Dec => 9,
        }
    }
    fn prefix(self) -> &'static str {
        match self {
            Radix::Bin => "0b",
            Radix::Oct => "0o",
            Radix::LowHex => "0x",
            Radix::UpHex => "0x",
            Radix::Dec => "",
        }
    }
}

trait FmtHelper
where
    Self: Copy + Ord,
    Self: Shl<u32, Output = Self> + Shr<u32, Output = Self> + Add<Output = Self>,
    Self: WrappingCast<u8> + Mul10 + From<u8>,
{
    const ZERO: Self;
    const MSB: Self;
    const BITS: u32;

    type Half: FmtHelper;

    fn int_used_nbits(int: Self) -> u32;
    fn frac_used_nbits(frac: Self) -> u32;
    fn as_half(val: Self) -> Self::Half;
    fn div_rem_10(val: Self) -> (Self, u8);
    fn wrapping_neg(val: Self) -> Self;

    fn write_int(mut int: Self, radix: Radix, nbits: u32, buf: &mut Buffer) {
        if Self::Half::BITS == Self::BITS / 2 && nbits <= Self::Half::BITS {
            return FmtHelper::write_int(Self::as_half(int), radix, nbits, buf);
        }
        let digit_bits = radix.digit_bits();
        let mask = radix.max();
        for b in buf.int().iter_mut().rev() {
            maybe_assert!(int != Self::ZERO);
            *b = int.wrapping_as::<u8>() & mask;
            int = int >> digit_bits;
        }
        maybe_assert!(int == Self::ZERO);
    }

    fn write_frac(mut frac: Self, radix: Radix, nbits: u32, buf: &mut Buffer) -> Ordering {
        if Self::Half::BITS == Self::BITS / 2 && nbits <= Self::Half::BITS {
            return FmtHelper::write_frac(
                Self::as_half(frac >> Self::Half::BITS),
                radix,
                nbits,
                buf,
            );
        }
        let digit_bits = radix.digit_bits();
        let compl_digit_bits = Self::BITS - digit_bits;
        for b in buf.frac().iter_mut() {
            maybe_assert!(frac != Self::ZERO);
            *b = (frac >> compl_digit_bits).wrapping_as::<u8>();
            frac = frac << digit_bits;
        }
        frac.cmp(&Self::MSB)
    }

    fn write_int_dec(mut int: Self, nbits: u32, buf: &mut Buffer) {
        if Self::Half::BITS == Self::BITS / 2 && nbits <= Self::Half::BITS {
            return FmtHelper::write_int_dec(Self::as_half(int), nbits, buf);
        }
        for b in buf.int().iter_mut().rev() {
            let (q, r) = Self::div_rem_10(int);
            int = q;
            *b = r;
        }
        maybe_assert!(int == Self::ZERO);
    }

    fn write_frac_dec(mut frac: Self, nbits: u32, auto_prec: bool, buf: &mut Buffer) -> Ordering {
        if Self::Half::BITS == Self::BITS / 2 && nbits <= Self::Half::BITS {
            return FmtHelper::write_frac_dec(
                Self::as_half(frac >> Self::Half::BITS),
                nbits,
                auto_prec,
                buf,
            );
        }

        // add_5 is to add rounding when all bits are used
        let (mut tie, mut add_5) = if nbits == Self::BITS {
            (Self::ZERO, true)
        } else {
            (Self::MSB >> nbits, false)
        };
        let mut trim_to = None;
        for (i, b) in buf.frac().iter_mut().enumerate() {
            *b = Mul10::mul10_assign(&mut frac);

            // Check if very close to zero, to avoid things like 0.19999999 and 0.20000001.
            // This takes place even if we have a precision.
            if frac < Self::from(10) || Self::wrapping_neg(frac) < Self::from(10) {
                trim_to = Some(i + 1);
                break;
            }

            if auto_prec {
                // tie might overflow in last iteration when i = frac_digits - 1,
                // but it has no effect as all it can do is set trim_to = Some(i + 1)
                Mul10::mul10_assign(&mut tie);
                if add_5 {
                    tie = tie + Self::from(5);
                    add_5 = false;
                }
                if frac < tie || Self::wrapping_neg(frac) < tie {
                    trim_to = Some(i + 1);
                    break;
                }
            }
        }
        if let Some(trim_to) = trim_to {
            buf.frac_digits = trim_to;
        }
        frac.cmp(&Self::MSB)
    }
}

macro_rules! impl_radix_helper {
    ($U:ident, $H:ident) => {
        impl FmtHelper for $U {
            const ZERO: $U = 0;
            const MSB: $U = 1 << ($U::BITS - 1);
            const BITS: u32 = $U::BITS;

            type Half = $H;

            fn int_used_nbits(int: $U) -> u32 {
                $U::BITS - int.leading_zeros()
            }

            fn frac_used_nbits(frac: $U) -> u32 {
                $U::BITS - frac.trailing_zeros()
            }

            fn as_half(val: $U) -> Self::Half {
                val as Self::Half
            }

            fn div_rem_10(val: $U) -> ($U, u8) {
                (val / 10, (val % 10).wrapping_cast())
            }

            fn wrapping_neg(val: $U) -> $U {
                val.wrapping_neg()
            }
        }
    };
}

impl_radix_helper! { u8, u8 }
impl_radix_helper! { u16, u8 }
impl_radix_helper! { u32, u16 }
impl_radix_helper! { u64, u32 }
impl_radix_helper! { u128, u64 }

fn fmt_dec<U: FmtHelper>((neg, abs): (bool, U), frac_nbits: u32, fmt: &mut Formatter) -> FmtResult {
    let (int, frac) = if frac_nbits == 0 {
        (abs, U::ZERO)
    } else if frac_nbits == U::BITS {
        (U::ZERO, abs)
    } else {
        (abs >> frac_nbits, abs << (U::BITS - frac_nbits))
    };
    let int_used_nbits = FmtHelper::int_used_nbits(int);
    let int_digits = ceil_log10_2_times(int_used_nbits);
    let frac_used_nbits = FmtHelper::frac_used_nbits(frac);
    let (frac_digits, auto_prec) = if let Some(precision) = fmt.precision() {
        // frac_used_nbits fits in usize, but precision might wrap to 0 in u32
        (cmp::min(frac_used_nbits as usize, precision) as u32, false)
    } else {
        (ceil_log10_2_times(frac_nbits), true)
    };

    let mut buf = Buffer::new();
    buf.set_len(int_digits, frac_digits);
    FmtHelper::write_int_dec(int, int_used_nbits, &mut buf);
    let frac_rem_cmp_msb = FmtHelper::write_frac_dec(frac, frac_nbits, auto_prec, &mut buf);
    buf.finish(Radix::Dec, neg, frac_rem_cmp_msb, fmt)
}

fn fmt_radix2<U: FmtHelper>(
    (neg, abs): (bool, U),
    frac_nbits: u32,
    radix: Radix,
    fmt: &mut Formatter,
) -> FmtResult {
    let (int, frac) = if frac_nbits == 0 {
        (abs, U::ZERO)
    } else if frac_nbits == U::BITS {
        (U::ZERO, abs)
    } else {
        (abs >> frac_nbits, abs << (U::BITS - frac_nbits))
    };
    let digit_bits = radix.digit_bits();
    let int_used_nbits = FmtHelper::int_used_nbits(int);
    let int_digits = (int_used_nbits + digit_bits - 1) / digit_bits;
    let frac_used_nbits = FmtHelper::frac_used_nbits(frac);
    let mut frac_digits = (frac_used_nbits + digit_bits - 1) / digit_bits;
    if let Some(precision) = fmt.precision() {
        // frac_digits fits in usize, but precision might wrap to 0 in u32
        frac_digits = cmp::min(frac_digits as usize, precision) as u32;
    }

    let mut buf = Buffer::new();
    buf.set_len(int_digits, frac_digits);
    FmtHelper::write_int(int, radix, int_used_nbits, &mut buf);
    // for bin, oct, hex, we can simply pass frac_used_bits to write_frac
    let frac_rem_cmp_msb = FmtHelper::write_frac(frac, radix, frac_used_nbits, &mut buf);
    buf.finish(radix, neg, frac_rem_cmp_msb, fmt)
}

macro_rules! impl_fmt {
    ($Fixed:ident($LeEqU:ident, $Inner:ident)) => {
        impl<Frac: $LeEqU> Display for $Fixed<Frac> {
            fn fmt(&self, f: &mut Formatter) -> FmtResult {
                let neg_abs = int_helper::$Inner::neg_abs(self.to_bits());
                fmt_dec(neg_abs, Self::FRAC_NBITS, f)
            }
        }

        impl<Frac: $LeEqU> Debug for $Fixed<Frac> {
            fn fmt(&self, f: &mut Formatter) -> FmtResult {
                let neg_abs = int_helper::$Inner::neg_abs(self.to_bits());
                match debug_hex::is_debug_hex(f) {
                    IsDebugHex::Lower => fmt_radix2(neg_abs, Self::FRAC_NBITS, Radix::LowHex, f),
                    IsDebugHex::Upper => fmt_radix2(neg_abs, Self::FRAC_NBITS, Radix::UpHex, f),
                    IsDebugHex::No => fmt_dec(neg_abs, Self::FRAC_NBITS, f),
                }
            }
        }

        impl<Frac: $LeEqU> Binary for $Fixed<Frac> {
            fn fmt(&self, f: &mut Formatter) -> FmtResult {
                let neg_abs = int_helper::$Inner::neg_abs(self.to_bits());
                fmt_radix2(neg_abs, Self::FRAC_NBITS, Radix::Bin, f)
            }
        }

        impl<Frac: $LeEqU> Octal for $Fixed<Frac> {
            fn fmt(&self, f: &mut Formatter) -> FmtResult {
                let neg_abs = int_helper::$Inner::neg_abs(self.to_bits());
                fmt_radix2(neg_abs, Self::FRAC_NBITS, Radix::Oct, f)
            }
        }

        impl<Frac: $LeEqU> LowerHex for $Fixed<Frac> {
            fn fmt(&self, f: &mut Formatter) -> FmtResult {
                let neg_abs = int_helper::$Inner::neg_abs(self.to_bits());
                fmt_radix2(neg_abs, Self::FRAC_NBITS, Radix::LowHex, f)
            }
        }

        impl<Frac: $LeEqU> UpperHex for $Fixed<Frac> {
            fn fmt(&self, f: &mut Formatter) -> FmtResult {
                let neg_abs = int_helper::$Inner::neg_abs(self.to_bits());
                fmt_radix2(neg_abs, Self::FRAC_NBITS, Radix::UpHex, f)
            }
        }
    };
}

impl_fmt! { FixedU8(LeEqU8, u8) }
impl_fmt! { FixedU16(LeEqU16, u16) }
impl_fmt! { FixedU32(LeEqU32, u32) }
impl_fmt! { FixedU64(LeEqU64, u64) }
impl_fmt! { FixedU128(LeEqU128, u128) }
impl_fmt! { FixedI8(LeEqU8, i8) }
impl_fmt! { FixedI16(LeEqU16, i16) }
impl_fmt! { FixedI32(LeEqU32, i32) }
impl_fmt! { FixedI64(LeEqU64, i64) }
impl_fmt! { FixedI128(LeEqU128, i128) }

// ceil(i × log_10 2), works for input < 112_816
fn ceil_log10_2_times(int_bits: u32) -> u32 {
    maybe_assert!(int_bits < 112_816);
    ((u64::from(int_bits) * 0x4D10_4D43 + 0xFFFF_FFFF) >> 32) as u32
}

pub(crate) trait Mul10: Sized {
    fn mul10_assign(slf: &mut Self) -> u8;
}
macro_rules! mul10_widen {
    ($Single:ty, $Double:ty) => {
        impl Mul10 for $Single {
            #[inline]
            fn mul10_assign(x: &mut $Single) -> u8 {
                let prod = <$Double>::from(*x) * 10;
                *x = prod as $Single;
                (prod >> <$Single>::BITS) as u8
            }
        }
    };
}
mul10_widen! { u8, u16 }
mul10_widen! { u16, u32 }
mul10_widen! { u32, u64 }
mul10_widen! { u64, u128 }
impl Mul10 for u128 {
    #[inline]
    fn mul10_assign(x: &mut u128) -> u8 {
        const LO_MASK: u128 = !(!0 << 64);
        let hi = (*x >> 64) * 10;
        let lo = (*x & LO_MASK) * 10;
        // Workaround for https://github.com/rust-lang/rust/issues/63384
        // let (wrapped, overflow) = (hi << 64).overflowing_add(lo);
        // ((hi >> 64) as u8 + u8::from(overflow), wrapped)
        let (hi_lo, hi_hi) = (hi as u64, (hi >> 64) as u64);
        let (lo_lo, lo_hi) = (lo as u64, (lo >> 64) as u64);
        let (wrapped, overflow) = hi_lo.overflowing_add(lo_hi);
        *x = (u128::from(wrapped) << 64) | u128::from(lo_lo);
        hi_hi as u8 + u8::from(overflow)
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use crate::{display, types::*};
    use std::{
        format,
        string::{String, ToString},
    };

    #[test]
    fn format() {
        let pos = I16F16::from_num(12.3);
        assert_eq!(format!("{:+}", pos), "+12.3");
        assert_eq!(format!("{:+08}", pos), "+00012.3");
        assert_eq!(format!("{:+#08}", pos), "+00012.3");
        assert_eq!(format!("{:+08X}", pos), "+0C.4CCD");
        assert_eq!(format!("{:+08.1X}", pos), "+0000C.5");
        assert_eq!(format!("{:+#08X}", pos), "+0xC.4CCD");
        assert_eq!(format!("{:+#08.1X}", pos), "+0x00C.5");

        assert_eq!(format!("{:#<8}", pos), "12.3####");
        assert_eq!(format!("{:#^8}", pos), "##12.3##");
        assert_eq!(format!("{:#^9}", pos), "##12.3###");
        assert_eq!(format!("{:#>8}", pos), "####12.3");
        assert_eq!(format!("{:#^08}", pos), "000012.3");
    }

    fn trim_frac_zeros(mut x: &str) -> &str {
        while x.ends_with('0') {
            x = &x[..x.len() - 1];
        }
        if x.ends_with('.') {
            x = &x[..x.len() - 1];
        }
        x
    }

    fn up_frac_digits(x: &mut String, frac_digits: usize) {
        if let Some(point) = x.find('.') {
            if let Some(additional) = frac_digits.checked_sub(x.len() - point - 1) {
                x.reserve(additional);
                for _ in 0..additional {
                    x.push('0');
                }
            }
        } else {
            x.reserve(frac_digits + 1);
            x.push('.');
            for _ in 0..frac_digits {
                x.push('0');
            }
        }
    }

    #[test]
    fn hex() {
        for i in 0..(1u32 << 7) {
            let p = 0x1234_5678_9abc_def0u64 ^ u64::from(i);
            let n = -0x1234_5678_9abc_def0i64 ^ i64::from(i);
            let f_p = U57F7::from_bits(p);
            let f_n = I57F7::from_bits(n);
            let mut check_p = format!("{:x}.{:02x}", p >> 7, (p & 0x7f) << 1);
            up_frac_digits(&mut check_p, 1000);
            let trimmed_p = trim_frac_zeros(&check_p);
            let mut check_n = format!("-{:x}.{:02x}", n.abs() >> 7, (n.abs() & 0x7f) << 1);
            up_frac_digits(&mut check_n, 1000);
            let trimmed_n = trim_frac_zeros(&check_n);
            assert_eq!(format!("{:.1000x}", f_p), check_p);
            assert_eq!(format!("{:x}", f_p), trimmed_p);
            assert_eq!(format!("{:.1000x}", f_n), check_n);
            assert_eq!(format!("{:x}", f_n), trimmed_n);
        }
    }

    #[test]
    fn debug_hex() {
        let v = I16F16::MAX;
        assert_eq!(format!("{:?}", v), "32767.99998");
        assert_eq!(format!("{:x?}", v), "7fff.ffff");
        assert_eq!(format!("{:X?}", v), "7FFF.FFFF");
        assert_eq!(format!("{:010X?}", v), "07FFF.FFFF");
    }

    #[test]
    fn dec() {
        for i in 0..(1 << 7) {
            // use 24 bits of precision to be like f32
            let bits = (!0u32 >> 8) ^ i;
            let fix = U25F7::from_bits(bits);
            let flt = (bits as f32) / 7f32.exp2();
            assert_eq!(format!("{}", fix), format!("{}", flt));
            assert_eq!(U25F7::from_num(flt), fix);
            assert_eq!(fix.to_num::<f32>(), flt);
        }
    }

    #[test]
    fn display_frac() {
        assert_eq!(
            format!("{:X}", I0F128::from_bits(!0)),
            "-0.00000000000000000000000000000001"
        );
        assert_eq!(format!("{:X}", I0F64::from_bits(!0)), "-0.0000000000000001");
        assert_eq!(format!("{:X}", I0F32::from_bits(!0)), "-0.00000001");
        assert_eq!(format!("{:X}", I0F16::from_bits(!0)), "-0.0001");
        assert_eq!(format!("{:X}", I0F8::from_bits(!0)), "-0.01");
        assert_eq!(
            format!("{:X}", U0F128::from_bits(!0)),
            "0.FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
        );
        assert_eq!(format!("{:X}", U0F64::from_bits(!0)), "0.FFFFFFFFFFFFFFFF");
        assert_eq!(format!("{:X}", U0F32::from_bits(!0)), "0.FFFFFFFF");
        assert_eq!(format!("{:X}", U0F16::from_bits(!0)), "0.FFFF");
        assert_eq!(format!("{:X}", U0F8::from_bits(!0)), "0.FF");

        assert_eq!(
            format!("{}", I0F128::from_bits(!0)),
            "-0.000000000000000000000000000000000000003"
        );
        assert_eq!(
            format!("{}", I0F64::from_bits(!0)),
            "-0.00000000000000000005"
        );
        assert_eq!(format!("{}", I0F32::from_bits(!0)), "-0.0000000002");
        assert_eq!(format!("{}", I0F16::from_bits(!0)), "-0.00002");
        assert_eq!(format!("{}", I0F8::from_bits(!0)), "-0.004");
        assert_eq!(
            format!("{}", U0F128::from_bits(!0)),
            "0.999999999999999999999999999999999999997"
        );
        assert_eq!(
            format!("{}", U0F64::from_bits(!0)),
            "0.99999999999999999995"
        );
        assert_eq!(format!("{}", U0F32::from_bits(!0)), "0.9999999998");
        assert_eq!(format!("{}", U0F16::from_bits(!0)), "0.99998");
        assert_eq!(format!("{}", U0F8::from_bits(!0)), "0.996");

        // check overflow issues in <u128 as Mul10>::mul10
        let no_internal_overflow_bits = 0xe666_6666_6666_6665_ffff_ffff_ffff_ffffu128;
        let internal_overflow_bits = 0xe666_6666_6666_6666_ffff_ffff_ffff_ffffu128;
        assert_eq!(
            format!("{:X}", U0F128::from_bits(no_internal_overflow_bits)),
            "0.E666666666666665FFFFFFFFFFFFFFFF"
        );
        assert_eq!(
            format!("{:X}", U0F128::from_bits(internal_overflow_bits)),
            "0.E666666666666666FFFFFFFFFFFFFFFF"
        );
        assert_eq!(
            format!("{}", U0F128::from_bits(no_internal_overflow_bits)),
            "0.899999999999999999978315956550289911317"
        );
        assert_eq!(
            format!("{}", U0F128::from_bits(internal_overflow_bits)),
            "0.900000000000000000032526065174565133017"
        );
    }

    #[test]
    fn close_to_round_decimal() {
        for i in 0..1000u16 {
            // f32 has 24 bits of precision, so we use 1 bit for the
            // integer part to have exactly 23 bits for the fraction
            let float = f32::from(i + 1000) / 1000.;
            let fix = U9F23::from_num(float);
            let check = format!("1.{:03}", i);
            assert_eq!(format!("{}", fix), trim_frac_zeros(&check));
            assert_eq!(format!("{}", fix), format!("{}", float));
            for prec in 0..10 {
                assert_eq!(format!("{:.*}", prec, fix), format!("{:.*}", prec, float));
            }
        }
    }

    #[test]
    fn check_ceil_log10_2_times() {
        for i in 0..112_816 {
            let check = (f64::from(i) * 2f64.log10()).ceil() as u32;
            assert_eq!(display::ceil_log10_2_times(i), check);
        }
    }

    #[test]
    fn rounding() {
        let i = U8F8::from_bits(0xFF80);
        assert_eq!(format!("{}", i), "255.5");
        assert_eq!(format!("{:?}", i), "255.5");
        assert_eq!(format!("{:.0}", i), "256");
        assert_eq!(format!("{:b}", i), "11111111.1");
        assert_eq!(format!("{:.0b}", i), "100000000");
        assert_eq!(format!("{:o}", i), "377.4");
        assert_eq!(format!("{:.0o}", i), "400");
        assert_eq!(format!("{:X}", i), "FF.8");
        assert_eq!(format!("{:.0X}", i), "100");

        let i = U8F8::from_bits(0xFE80);
        assert_eq!(format!("{}", i), "254.5");
        assert_eq!(format!("{:?}", i), "254.5");
        assert_eq!(format!("{:.0}", i), "254");
        assert_eq!(format!("{:b}", i), "11111110.1");
        assert_eq!(format!("{:.0b}", i), "11111110");
        assert_eq!(format!("{:o}", i), "376.4");
        assert_eq!(format!("{:.0o}", i), "376");
        assert_eq!(format!("{:X}", i), "FE.8");
        assert_eq!(format!("{:.0X}", i), "FE");

        let i = U8F8::from_bits(0xDDDD);
        assert_eq!(format!("{}", i), "221.863");
        assert_eq!(format!("{:?}", i), "221.863");
        assert_eq!(format!("{:.0}", i), "222");
        assert_eq!(format!("{:.1}", i), "221.9");
        assert_eq!(format!("{:.2}", i), "221.86");
        assert_eq!(format!("{:.3}", i), "221.863");
        assert_eq!(format!("{:.4}", i), "221.8633");
        assert_eq!(format!("{:.5}", i), "221.86328");
        assert_eq!(format!("{:.6}", i), "221.863281");
        assert_eq!(format!("{:.7}", i), "221.8632812");
        assert_eq!(format!("{:.8}", i), "221.86328125");
        assert_eq!(format!("{:.9}", i), "221.863281250");
        assert_eq!(format!("{:b}", i), "11011101.11011101");
        assert_eq!(format!("{:.0b}", i), "11011110");
        assert_eq!(format!("{:.1b}", i), "11011110.0");
        assert_eq!(format!("{:.2b}", i), "11011101.11");
        assert_eq!(format!("{:.3b}", i), "11011101.111");
        assert_eq!(format!("{:.4b}", i), "11011101.1110");
        assert_eq!(format!("{:.5b}", i), "11011101.11100");
        assert_eq!(format!("{:.6b}", i), "11011101.110111");
        assert_eq!(format!("{:.7b}", i), "11011101.1101110");
        assert_eq!(format!("{:.8b}", i), "11011101.11011101");
        assert_eq!(format!("{:.9b}", i), "11011101.110111010");
        assert_eq!(format!("{:o}", i), "335.672");
        assert_eq!(format!("{:.0o}", i), "336");
        assert_eq!(format!("{:.1o}", i), "335.7");
        assert_eq!(format!("{:.2o}", i), "335.67");
        assert_eq!(format!("{:.3o}", i), "335.672");
        assert_eq!(format!("{:.4o}", i), "335.6720");
        assert_eq!(format!("{:X}", i), "DD.DD");
        assert_eq!(format!("{:.0X}", i), "DE");
        assert_eq!(format!("{:.0X}", i), "DE");
        assert_eq!(format!("{:.1X}", i), "DD.E");
        assert_eq!(format!("{:.2X}", i), "DD.DD");
        assert_eq!(format!("{:.3X}", i), "DD.DD0");
    }

    #[test]
    fn compare_frac0_int() {
        for u in 0..=255u8 {
            let i = u as i8;
            let (ifix, ufix) = (I8F0::from_bits(i), U8F0::from_bits(u));
            assert_eq!(ifix.to_string(), i.to_string());
            assert_eq!(ufix.to_string(), u.to_string());
            if i >= 0 {
                assert_eq!(format!("{:#X}", ifix), format!("{:#X}", i));
                assert_eq!(format!("{:#b}", ifix), format!("{:#b}", i));
            } else {
                let abs_i = i.wrapping_neg() as u8;
                assert_eq!(format!("{:#X}", ifix), format!("-{:#X}", abs_i));
                assert_eq!(format!("{:#b}", ifix), format!("-{:#b}", abs_i));
            }
            assert_eq!(format!("{:#x}", ufix), format!("{:#x}", u));
            assert_eq!(format!("{:#o}", ufix), format!("{:#o}", u));
        }
    }

    #[test]
    fn compare_frac4_float() {
        for u in 0..=255u8 {
            // I4F4 and U4F4 are displayed like f32 when the f32
            // display precision is the number of fractional digits
            // displayed for fixed-point. This verifies correct display
            // of the integer part.
            let (ifix, ufix) = (I4F4::from_bits(u as i8), U4F4::from_bits(u));
            let (iflo, uflo) = (ifix.to_num::<f32>(), ufix.to_num::<f32>());
            let (sifix, sufix) = (ifix.to_string(), ufix.to_string());
            let pifix = sifix.find('.').map(|p| sifix.len() - 1 - p).unwrap_or(0);
            let pufix = sufix.find('.').map(|p| sufix.len() - 1 - p).unwrap_or(0);
            let (siflo, suflo) = (format!("{:.*}", pifix, iflo), format!("{:.*}", pufix, uflo));
            assert_eq!(sifix, siflo);
            assert_eq!(sufix, suflo);

            // I28F4 and U28F4 are displayed like f32 when the f32 has
            // four bits of precision dedicated to the fractional
            // part. For f32, this requires the magnitude’s integer
            // part to have 20 significant bits: (1 << 19)..(1 << 20).
            let ifixed =
                I28F4::from(ifix) + I28F4::from_num(i32::from(ifix.to_bits().signum()) << 19);
            let ufixed = U28F4::from(ufix) + U28F4::from_num(1 << 19);
            let (ifloat, ufloat) = (ifixed.to_num::<f32>(), ufixed.to_num::<f32>());
            let (sifixed, sufixed) = (ifixed.to_string(), ufixed.to_string());
            let (sifloat, sufloat) = (ifloat.to_string(), ufloat.to_string());
            assert_eq!(sifixed, sifloat);
            assert_eq!(sufixed, sufloat);

            // The fractional parts of I4F4 and U4F4 are displayed
            // like the fractional parts of I28F4 and U28F4
            // respectively.
            let sifix_frac = sifix.find('.').map(|i| &sifix[i..]);
            let sifixed_frac = sifixed.find('.').map(|i| &sifixed[i..]);
            assert_eq!(sifix_frac, sifixed_frac);
            let sufix_frac = sufix.find('.').map(|i| &sufix[i..]);
            let sufixed_frac = sufixed.find('.').map(|i| &sufixed[i..]);
            assert_eq!(sufix_frac, sufixed_frac);
        }
    }

    #[test]
    fn compare_frac17_float() {
        for u in 0..(1 << 17) {
            // 24 bits of precision: 17 fractional bits + 7 significant integer bits
            let fix = U15F17::from_bits(u) + U15F17::from_num(99);
            let fix_pos = I15F17::from_num(fix);
            let fix_neg = -fix_pos;
            let (flo, flo_neg) = (fix.to_num::<f32>(), fix_neg.to_num::<f32>());

            let fix_str = fix.to_string();
            let fix_pos_str = fix_pos.to_string();
            let fix_neg_str = fix_neg.to_string();
            assert_eq!(fix_str, flo.to_string());
            assert_eq!(fix_str, fix_pos_str);
            assert_eq!(fix_neg_str, flo_neg.to_string());
            if u != 0 {
                assert_eq!(&fix_neg_str[..1], "-");
                assert_eq!(&fix_neg_str[1..], fix_pos_str);
            }

            let fix_str3 = format!("{:.3}", fix);
            let fix_pos_str3 = format!("{:.3}", fix_pos);
            let fix_neg_str3 = format!("{:.3}", fix_neg);
            assert_eq!(fix_str3, format!("{:.3}", flo));
            assert_eq!(fix_str3, fix_pos_str3);
            assert_eq!(fix_neg_str3, format!("{:.3}", flo_neg));
            if u != 0 {
                assert_eq!(&fix_neg_str3[..1], "-");
                assert_eq!(&fix_neg_str3[1..], fix_pos_str3);
            }
        }
    }
}
