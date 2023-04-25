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
    display::Mul10,
    int256::{self, U256},
    types::extra::{LeEqU128, LeEqU16, LeEqU32, LeEqU64, LeEqU8},
    FixedI128, FixedI16, FixedI32, FixedI64, FixedI8, FixedU128, FixedU16, FixedU32, FixedU64,
    FixedU8,
};
use core::{
    cmp::Ordering,
    fmt::{Display, Formatter, Result as FmtResult},
    ops::{Add, Mul, Shl, Shr, Sub},
    str::FromStr,
};
#[cfg(feature = "std")]
use std::error::Error;

trait ParseHelper
where
    Self: Copy + Eq + Ord + From<u8>,
    Self: Shl<u32, Output = Self> + Shr<u32, Output = Self>,
    Self: Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self>,
    Self: Mul10 + DecToBin,
{
    const BITS: u32;

    fn is_odd(val: Self) -> bool;
    fn checked_inc(val: Self) -> Option<Self>;
    fn overflowing_add(lhs: Self, rhs: Self) -> (Self, bool);

    fn bin_str_int_to_bin(bytes: &[u8]) -> (Self, bool) {
        let max_len = Self::BITS as usize;
        let (bytes, overflow) = if bytes.len() > max_len {
            (&bytes[(bytes.len() - max_len)..], true)
        } else {
            (bytes, false)
        };
        let mut acc = Self::from(0);
        for &byte in bytes {
            acc = (acc << 1) + Self::from(byte - b'0');
        }
        (acc, overflow)
    }

    fn bin_str_frac_to_bin(bytes: &[u8], nbits: u32) -> Option<Self> {
        maybe_assert!(!bytes.is_empty());
        let dump_bits = Self::BITS - nbits;
        let mut rem_bits = nbits;
        let mut acc = Self::from(0);
        for (i, &byte) in bytes.iter().enumerate() {
            let val = byte - b'0';
            if rem_bits < 1 {
                if val != 0 {
                    // half bit is true, round up if we have more
                    // significant bits or currently acc is odd
                    if bytes.len() > i + 1 || Self::is_odd(acc) {
                        acc = Self::checked_inc(acc)?;
                    }
                }
                if dump_bits != 0 && acc >> nbits != Self::from(0) {
                    return None;
                }
                return Some(acc);
            }
            acc = (acc << 1) + Self::from(val);
            rem_bits -= 1;
        }
        Some(acc << rem_bits)
    }

    fn oct_str_int_to_bin(bytes: &[u8]) -> (Self, bool) {
        let max_len = (Self::BITS as usize + 2) / 3;
        let (bytes, mut overflow) = if bytes.len() > max_len {
            (&bytes[(bytes.len() - max_len)..], true)
        } else {
            (bytes, false)
        };
        let mut acc = Self::from(bytes[0] - b'0');
        if bytes.len() == max_len {
            let first_max_bits = Self::BITS - (max_len as u32 - 1) * 3;
            let first_max = (Self::from(1) << first_max_bits) - Self::from(1);
            if acc > first_max {
                overflow = true;
            }
        }
        for &byte in &bytes[1..] {
            acc = (acc << 3) + Self::from(byte - b'0');
        }
        (acc, overflow)
    }

    fn oct_str_frac_to_bin(bytes: &[u8], nbits: u32) -> Option<Self> {
        maybe_assert!(!bytes.is_empty());
        let dump_bits = Self::BITS - nbits;
        let mut rem_bits = nbits;
        let mut acc = Self::from(0);
        for (i, &byte) in bytes.iter().enumerate() {
            let val = byte - b'0';
            if rem_bits < 3 {
                acc = (acc << rem_bits) + Self::from(val >> (3 - rem_bits));
                let half = 1 << (2 - rem_bits);
                if val & half != 0 {
                    // half bit is true, round up if we have more
                    // significant bits or currently acc is odd
                    if val & (half - 1) != 0 || bytes.len() > i + 1 || Self::is_odd(acc) {
                        acc = Self::checked_inc(acc)?;
                    }
                }
                if dump_bits != 0 && acc >> nbits != Self::from(0) {
                    return None;
                }
                return Some(acc);
            }
            acc = (acc << 3) + Self::from(val);
            rem_bits -= 3;
        }
        Some(acc << rem_bits)
    }

    fn hex_str_int_to_bin(bytes: &[u8]) -> (Self, bool) {
        let max_len = (Self::BITS as usize + 3) / 4;
        let (bytes, mut overflow) = if bytes.len() > max_len {
            (&bytes[(bytes.len() - max_len)..], true)
        } else {
            (bytes, false)
        };
        let mut acc = Self::from(unchecked_hex_digit(bytes[0]));
        if bytes.len() == max_len {
            let first_max_bits = Self::BITS - (max_len as u32 - 1) * 4;
            let first_max = (Self::from(1) << first_max_bits) - Self::from(1);
            if acc > first_max {
                overflow = true;
            }
        }
        for &byte in &bytes[1..] {
            acc = (acc << 4) + Self::from(unchecked_hex_digit(byte));
        }
        (acc, overflow)
    }

    fn hex_str_frac_to_bin(bytes: &[u8], nbits: u32) -> Option<Self> {
        maybe_assert!(!bytes.is_empty());
        let dump_bits = Self::BITS - nbits;
        let mut rem_bits = nbits;
        let mut acc = Self::from(0);
        for (i, &byte) in bytes.iter().enumerate() {
            let val = unchecked_hex_digit(byte);
            if rem_bits < 4 {
                acc = (acc << rem_bits) + Self::from(val >> (4 - rem_bits));
                let half = 1 << (3 - rem_bits);
                if val & half != 0 {
                    // half bit is true, round up if we have more
                    // significant bits or currently acc is odd
                    if val & (half - 1) != 0 || bytes.len() > i + 1 || Self::is_odd(acc) {
                        acc = Self::checked_inc(acc)?;
                    }
                }
                if dump_bits != 0 && acc >> nbits != Self::from(0) {
                    return None;
                }
                return Some(acc);
            }
            acc = (acc << 4) + Self::from(val);
            rem_bits -= 4;
        }
        Some(acc << rem_bits)
    }

    fn dec_str_int_to_bin(bytes: &[u8]) -> (Self, bool) {
        let max_effective_len = Self::BITS as usize;
        let (bytes, mut overflow) = if bytes.len() > max_effective_len {
            (&bytes[(bytes.len() - max_effective_len)..], true)
        } else {
            (bytes, false)
        };
        let mut acc = Self::from(0);
        for &byte in bytes {
            let mul_overflow = Mul10::mul10_assign(&mut acc) != 0;
            let (add, add_overflow) = Self::overflowing_add(acc, Self::from(byte - b'0'));
            acc = add;
            overflow = overflow || mul_overflow || add_overflow;
        }
        (acc, overflow)
    }

    fn dec_str_frac_to_bin(bytes: &[u8], nbits: u32) -> Option<Self> {
        let (val, is_short) = Self::parse_is_short(bytes);
        let one = Self::from(1);
        let dump_bits = Self::BITS - nbits;
        // if is_short, dec_to_bin can round and give correct answer immediately
        let round = if is_short {
            Round::Nearest
        } else {
            Round::Floor
        };
        let floor = Self::dec_to_bin(val, nbits, round)?;
        if is_short {
            return Some(floor);
        }
        // since !is_short, we have a floor and we have to check whether we need to increment

        // add_5 is to add rounding when all bits are used
        let (mut boundary, mut add_5) = if nbits == 0 {
            (Self::from(1) << (Self::BITS - 1), false)
        } else if dump_bits == 0 {
            (floor, true)
        } else {
            ((floor << dump_bits) + (one << (dump_bits - 1)), false)
        };
        let mut tie = true;
        for &byte in bytes {
            if !add_5 && boundary == Self::from(0) {
                // since zeros are trimmed in bytes, there must be some byte > 0 eventually
                tie = false;
                break;
            }
            let mut boundary_digit = Mul10::mul10_assign(&mut boundary);
            if add_5 {
                let (wrapped, overflow) = Self::overflowing_add(boundary, Self::from(5));
                boundary = wrapped;
                if overflow {
                    boundary_digit += 1;
                }
                add_5 = false;
            }
            if byte - b'0' < boundary_digit {
                return Some(floor);
            }
            if byte - b'0' > boundary_digit {
                tie = false;
                break;
            }
        }
        if tie && !Self::is_odd(floor) {
            return Some(floor);
        }
        let next_up = Self::checked_inc(floor)?;
        if dump_bits != 0 && next_up >> nbits != Self::from(0) {
            None
        } else {
            Some(next_up)
        }
    }
}

fn unchecked_hex_digit(byte: u8) -> u8 {
    // We know that byte is a valid hex:
    //   * b'0'..=b'9' (0x30..=0x39) => byte & 0x0f
    //   * b'A'..=b'F' (0x41..=0x46) => byte & 0x0f + 9
    //   * b'a'..=b'f' (0x61..=0x66) => byte & 0x0f + 9
    (byte & 0x0f) + if byte >= 0x40 { 9 } else { 0 }
}

enum Round {
    Nearest,
    Floor,
}

// Decode fractional decimal digits into nbits fractional bits.
//
// For an output with BIN = 8 bits, we can take DEC = 3 decimal digits.
//
//     0 ≤ val ≤ 999, 0 ≤ nbits ≤ 8
//
// In general,
//
//     0 ≤ val ≤ 10^DEC - 1, 0 ≤ nbits ≤ BIN
//
// We can either floor the result or round to the nearest, with ties
// rounded to even. If rounding results in more than nbits bits,
// returns None.
//
// Examples: (for DEC = 3, BIN = 8)
//
//    dec_to_bin(999, 8, Round::Floor) -> floor(999 × 256 / 1000) -> 255 -> Some(255)
//    dec_to_bin(999, 8, Round::Nearest) -> floor(999 × 256 / 1000 + 0.5) -> 256 -> None
//    dec_to_bin(999, 5, Round::Floor) -> floor(999 × 32 / 1000) -> 31 -> Some(31)
//    dec_to_bin(999, 5, Round::Nearest) -> floor(999 × 32 / 1000 + 0.5) -> 32 -> None
//    dec_to_bin(499, 0, Round::Floor) -> floor(499 / 1000) -> 0 -> Some(0)
//    dec_to_bin(499, 0, Round::Nearest) -> floor(499 / 1000 + 0.5) -> 0 -> Some(0)
//    dec_to_bin(500, 0, Round::Nearest) -> floor(500 / 1000 + 0.5) -> 1 -> None
//
// For flooring:
//
//     floor(val × 2^nbits / 10^3) = floor(val × 2^(nbits - 3) / 5^3)
//
// For rounding:
//
//     floor(val × 2^nbits / 10^3 + 0.5) = floor((val × 2^(nbits - 2) + 5^3) / (2 × 5^3))
//
// Using integer arithmetic, this is equal to:
//
//     ((val << 6 >> (8 - nbits)) + if rounding { 125 } else { 0 }) / 250
//
// Note that val << 6 cannot overflow u16, as val < 1000 and 1000 × 2^6 < 2^16.
//
// In general:
//
//     ((val << (BIN - DEC + 1) >> (8 - nbits)) + if rounding { 5^DEC } else { 0 }) / (2 × 5^DEC)
//
// And we ensure that 10^DEC × 2^(BIN - DEC + 1) < 2^(2 × BIN), which simplifies to
//
//     5^DEC × 2 < 2^BIN
//
// From this it also follows that val << (BIN - DEC + 1) never overflows a (2 × BIN)-bit number.
//
// So for u8, BIN = 8, DEC  3
// So for u16, BIN = 16, DEC ≤ 6
// So for u32, BIN = 32, DEC ≤ 13
// So for u64, BIN = 64, DEC ≤ 27
// So for u128, BIN = 128, DEC ≤ 54
trait DecToBin: Sized {
    type Double;
    fn dec_to_bin(val: Self::Double, nbits: u32, round: Round) -> Option<Self>;
    fn parse_is_short(bytes: &[u8]) -> (Self::Double, bool);
}

macro_rules! impl_dec_to_bin {
    ($Single:ident, $Double:ident, $dec:expr, $bin:expr) => {
        impl DecToBin for $Single {
            type Double = $Double;
            fn dec_to_bin(val: $Double, nbits: u32, round: Round) -> Option<$Single> {
                maybe_assert!(val < $Double::pow(10, $dec));
                maybe_assert!(nbits <= $bin);
                let fives = $Double::pow(5, $dec);
                let denom = fives * 2;
                let mut numer = val << ($bin - $dec + 1) >> ($bin - nbits);
                match round {
                    Round::Nearest => {
                        // Round up, then round back down if we had a tie and the result is odd.
                        numer += fives;
                        // If unrounded division == 1 exactly, we actually have a tie at upper
                        // bound, which is rounded up to 1.0. This is even in all cases except
                        // when nbits == 0, in which case we must round it back down to 0.
                        if numer >> nbits >= denom {
                            // 0.5 exactly is 10^$dec / 2 = 5^dec * 2^dec / 2 = fives << ($dec - 1)
                            return if nbits == 0 && val == fives << ($dec - 1) {
                                Some(0)
                            } else {
                                None
                            };
                        }
                    }
                    Round::Floor => {}
                }
                let (mut div, tie) = (numer / denom, numer % denom == 0);
                if tie && ParseHelper::is_odd(div) {
                    div -= 1;
                }
                Some(div as $Single)
            }

            fn parse_is_short(bytes: &[u8]) -> ($Double, bool) {
                let (is_short, slice, pad) =
                    if let Some(rem) = usize::checked_sub($dec, bytes.len()) {
                        (true, bytes, $Double::pow(10, rem as u32))
                    } else {
                        (false, &bytes[..$dec], 1)
                    };
                let val = <$Double as ParseHelper>::dec_str_int_to_bin(slice).0 * pad;
                (val, is_short)
            }
        }
    };
}
impl_dec_to_bin! { u8, u16, 3, 8 }
impl_dec_to_bin! { u16, u32, 6, 16 }
impl_dec_to_bin! { u32, u64, 13, 32 }
impl_dec_to_bin! { u64, u128, 27, 64 }

impl DecToBin for u128 {
    type Double = (u128, u128);
    fn dec_to_bin((hi, lo): (u128, u128), nbits: u32, round: Round) -> Option<u128> {
        maybe_assert!(hi < 10u128.pow(27));
        maybe_assert!(lo < 10u128.pow(27));
        maybe_assert!(nbits <= 128);
        let fives = 5u128.pow(54);
        let denom = fives * 2;
        // we need to combine (10^27*hi + lo) << (128 - 54 + 1)
        let (hi_hi, hi_lo) = mul_hi_lo(hi, 10u128.pow(27));
        let (val_lo, overflow) = hi_lo.overflowing_add(lo);
        let val_hi = if overflow { hi_hi + 1 } else { hi_hi };
        let (mut numer_lo, mut numer_hi) = (val_lo, val_hi);
        match nbits.cmp(&(54 - 1)) {
            Ordering::Less => {
                let shr = (54 - 1) - nbits;
                numer_lo = (numer_lo >> shr) | (numer_hi << (128 - shr));
                numer_hi >>= shr;
            }
            Ordering::Greater => {
                let shl = nbits - (54 - 1);
                numer_hi = (numer_hi << shl) | (numer_lo >> (128 - shl));
                numer_lo <<= shl;
            }
            Ordering::Equal => {}
        };
        match round {
            Round::Nearest => {
                // Round up, then round back down if we had a tie and the result is odd.
                let (wrapped, overflow) = numer_lo.overflowing_add(fives);
                numer_lo = wrapped;
                if overflow {
                    numer_hi += 1;
                }
                let check_overflow = if nbits == 128 {
                    numer_hi
                } else if nbits == 0 {
                    numer_lo
                } else {
                    (numer_lo >> nbits) | (numer_hi << (128 - nbits))
                };
                // If unrounded division == 1 exactly, we actually have a tie at upper
                // bound, which is rounded up to 1.0. This is even in all cases except
                // when nbits == 0, in which case we must round it back down to 0.
                if check_overflow >= denom {
                    // 0.5 exactly is 10^$dec / 2 = 5^dec * 2^dec / 2 = fives << ($dec - 1)
                    let half_hi = fives >> (128 - (54 - 1));
                    let half_lo = fives << (54 - 1);
                    return if nbits == 0 && val_hi == half_hi && val_lo == half_lo {
                        Some(0)
                    } else {
                        None
                    };
                }
            }
            Round::Floor => {}
        }
        let (mut div, tie) = div_tie(numer_hi, numer_lo, denom);
        if tie && ParseHelper::is_odd(div) {
            div -= 1;
        }
        Some(div)
    }

    fn parse_is_short(bytes: &[u8]) -> ((u128, u128), bool) {
        if let Some(rem) = 27usize.checked_sub(bytes.len()) {
            let hi = <u128 as ParseHelper>::dec_str_int_to_bin(bytes).0 * 10u128.pow(rem as u32);
            ((hi, 0), true)
        } else {
            let hi = <u128 as ParseHelper>::dec_str_int_to_bin(&bytes[..27]).0;

            let (is_short, slice, pad) = if let Some(rem) = 54usize.checked_sub(bytes.len()) {
                (true, &bytes[27..], 10u128.pow(rem as u32))
            } else {
                (false, &bytes[27..54], 1)
            };
            let lo = <u128 as ParseHelper>::dec_str_int_to_bin(slice).0 * pad;
            ((hi, lo), is_short)
        }
    }
}

fn mul_hi_lo(lhs: u128, rhs: u128) -> (u128, u128) {
    const LO: u128 = !(!0 << 64);
    let (lhs_hi, lhs_lo) = (lhs >> 64, lhs & LO);
    let (rhs_hi, rhs_lo) = (rhs >> 64, rhs & LO);
    let lhs_lo_rhs_lo = lhs_lo.wrapping_mul(rhs_lo);
    let lhs_hi_rhs_lo = lhs_hi.wrapping_mul(rhs_lo);
    let lhs_lo_rhs_hi = lhs_lo.wrapping_mul(rhs_hi);
    let lhs_hi_rhs_hi = lhs_hi.wrapping_mul(rhs_hi);

    let col01 = lhs_lo_rhs_lo;
    let (col01_hi, col01_lo) = (col01 >> 64, col01 & LO);
    let partial_col12 = lhs_hi_rhs_lo + col01_hi;
    let (col12, carry_col3) = partial_col12.overflowing_add(lhs_lo_rhs_hi);
    let (col12_hi, col12_lo) = (col12 >> 64, col12 & LO);
    let ans01 = (col12_lo << 64) + col01_lo;
    let ans23 = lhs_hi_rhs_hi + col12_hi + if carry_col3 { 1u128 << 64 } else { 0 };
    (ans23, ans01)
}
fn div_tie(dividend_hi: u128, dividend_lo: u128, divisor: u128) -> (u128, bool) {
    let dividend = U256 {
        lo: dividend_lo,
        hi: dividend_hi,
    };
    let (quot, rem) = int256::div_rem_u256_u128(dividend, divisor);
    (quot.lo, rem == 0)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Parse<'a> {
    neg: bool,
    int: &'a [u8],
    frac: &'a [u8],
}

/**
An error which can be returned when parsing a fixed-point number.

# Examples

```rust
use fixed::{types::I16F16, ParseFixedError};
// This string is not a fixed-point number.
let s = "something completely different (_!_!_)";
let error: ParseFixedError = match s.parse::<I16F16>() {
    Ok(_) => unreachable!(),
    Err(error) => error,
};
println!("Parse error: {}", error);
```
*/
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseFixedError {
    kind: ParseErrorKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ParseErrorKind {
    InvalidDigit,
    NoDigits,
    TooManyPoints,
    Overflow,
}

impl From<ParseErrorKind> for ParseFixedError {
    #[inline]
    fn from(kind: ParseErrorKind) -> ParseFixedError {
        ParseFixedError { kind }
    }
}

impl ParseFixedError {
    pub(crate) fn message(&self) -> &str {
        use self::ParseErrorKind::*;
        match self.kind {
            InvalidDigit => "invalid digit found in string",
            NoDigits => "string has no digits",
            TooManyPoints => "more than one decimal point found in string",
            Overflow => "overflow",
        }
    }
}

impl Display for ParseFixedError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(self.message(), f)
    }
}

#[cfg(feature = "std")]
impl Error for ParseFixedError {
    fn description(&self) -> &str {
        self.message()
    }
}

// also trims zeros at start of int and at end of frac
fn parse_bounds(bytes: &[u8], radix: u32) -> Result<Parse<'_>, ParseFixedError> {
    let mut sign: Option<bool> = None;
    let mut trimmed_int_start: Option<usize> = None;
    let mut point: Option<usize> = None;
    let mut trimmed_frac_end: Option<usize> = None;
    let mut has_any_digit = false;

    for (index, &byte) in bytes.iter().enumerate() {
        match (byte, radix) {
            (b'+', _) => {
                if sign.is_some() || point.is_some() || has_any_digit {
                    return Err(ParseErrorKind::InvalidDigit.into());
                }
                sign = Some(false);
                continue;
            }
            (b'-', _) => {
                if sign.is_some() || point.is_some() || has_any_digit {
                    return Err(ParseErrorKind::InvalidDigit.into());
                }
                sign = Some(true);
                continue;
            }
            (b'.', _) => {
                if point.is_some() {
                    return Err(ParseErrorKind::TooManyPoints.into());
                }
                point = Some(index);
                trimmed_frac_end = Some(index + 1);
                continue;
            }
            (b'0'..=b'1', 2)
            | (b'0'..=b'7', 8)
            | (b'0'..=b'9', 10)
            | (b'0'..=b'9', 16)
            | (b'a'..=b'f', 16)
            | (b'A'..=b'F', 16) => {
                if trimmed_int_start.is_none() && point.is_none() && byte != b'0' {
                    trimmed_int_start = Some(index);
                }
                if trimmed_frac_end.is_some() && byte != b'0' {
                    trimmed_frac_end = Some(index + 1);
                }
                has_any_digit = true;
            }
            _ => return Err(ParseErrorKind::InvalidDigit.into()),
        }
    }
    if !has_any_digit {
        return Err(ParseErrorKind::NoDigits.into());
    }
    let neg = sign.unwrap_or(false);
    let int = match (trimmed_int_start, point) {
        (Some(start), Some(point)) => &bytes[start..point],
        (Some(start), None) => &bytes[start..],
        (None, _) => &bytes[..0],
    };
    let frac = match (point, trimmed_frac_end) {
        (Some(point), Some(end)) => &bytes[(point + 1)..end],
        _ => &bytes[..0],
    };
    Ok(Parse { neg, int, frac })
}

fn frac_is_half(bytes: &[u8], radix: u32) -> bool {
    // since zeros are trimmed, there must be exatly one byte
    bytes.len() == 1 && bytes[0] - b'0' == (radix as u8) / 2
}

// Only radices 2, 8, 10 and 16 are supported.
pub(crate) trait FromStrRadix: Sized {
    type Err;
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, Self::Err>;
    fn saturating_from_str_radix(s: &str, radix: u32) -> Result<Self, Self::Err>;
    fn wrapping_from_str_radix(s: &str, radix: u32) -> Result<Self, Self::Err>;
    fn overflowing_from_str_radix(s: &str, radix: u32) -> Result<(Self, bool), Self::Err>;
}

macro_rules! impl_from_str_traits {
    ($Fixed:ident($Bits:ident), $LeEqU:ident; fn $from:ident) => {
        impl<Frac: $LeEqU> FromStr for $Fixed<Frac> {
            type Err = ParseFixedError;
            /// Parses a string slice to return a fixed-point number.
            ///
            /// Rounding is to the nearest, with ties rounded to even.
            #[inline]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::from_str_radix(s, 10)
            }
        }
        impl<Frac: $LeEqU> FromStrRadix for $Fixed<Frac> {
            type Err = ParseFixedError;
            #[inline]
            fn from_str_radix(s: &str, radix: u32) -> Result<Self, Self::Err> {
                match Self::overflowing_from_str_radix(s, radix) {
                    Ok((val, false)) => Ok(val),
                    Ok((_, true)) => Err(ParseErrorKind::Overflow.into()),
                    Err(e) => Err(e),
                }
            }
            #[inline]
            fn saturating_from_str_radix(s: &str, radix: u32) -> Result<Self, Self::Err> {
                match Self::overflowing_from_str_radix(s, radix) {
                    Ok((val, false)) => Ok(val),
                    Ok((_, true)) => {
                        if s.starts_with('-') {
                            Ok(Self::MIN)
                        } else {
                            Ok(Self::MAX)
                        }
                    }
                    Err(e) => Err(e),
                }
            }
            #[inline]
            fn wrapping_from_str_radix(s: &str, radix: u32) -> Result<Self, Self::Err> {
                Self::overflowing_from_str_radix(s, radix).map(|(val, _)| val)
            }
            #[inline]
            fn overflowing_from_str_radix(s: &str, radix: u32) -> Result<(Self, bool), Self::Err> {
                $from(s.as_bytes(), radix, Self::INT_NBITS, Self::FRAC_NBITS)
                    .map(|(bits, overflow)| (Self::from_bits(bits), overflow))
            }
        }
    };
}

macro_rules! impl_from_str {
    (
        $FixedI:ident($BitsI:ident), $FixedU:ident($BitsU:ident), $LeEqU:ident;
        fn $from_i:ident;
        fn $from_u:ident;
        fn $get_int_frac:ident;
        fn $get_int:ident, ($get_int_half:ident, $attempt_int_half:expr);
        fn $get_frac:ident, ($get_frac_half:ident, $attempt_frac_half:expr);
    ) => {
        impl_from_str_traits! { $FixedI($BitsI), $LeEqU; fn $from_i }
        impl_from_str_traits! { $FixedU($BitsU), $LeEqU; fn $from_u }

        impl ParseHelper for $BitsU {
            const BITS: u32 = $BitsU::BITS;

            fn is_odd(val: $BitsU) -> bool {
                val & 1 != 0
            }

            fn checked_inc(val: $BitsU) -> Option<$BitsU> {
                val.checked_add(1)
            }

            fn overflowing_add(lhs: $BitsU, rhs: $BitsU) -> ($BitsU, bool) {
                lhs.overflowing_add(rhs)
            }
        }

        fn $from_i(
            bytes: &[u8],
            radix: u32,
            int_nbits: u32,
            frac_nbits: u32,
        ) -> Result<($BitsI, bool), ParseFixedError> {
            let (neg, abs, mut overflow) = $get_int_frac(bytes, radix, int_nbits, frac_nbits)?;
            let bound = if !neg { $BitsI::MAX } else { $BitsI::MIN };
            if abs > bound.unsigned_abs() {
                overflow = true;
            }
            let abs = if neg { abs.wrapping_neg() } else { abs } as $BitsI;
            Ok((abs, overflow))
        }

        fn $from_u(
            bytes: &[u8],
            radix: u32,
            int_nbits: u32,
            frac_nbits: u32,
        ) -> Result<($BitsU, bool), ParseFixedError> {
            let (neg, abs, mut overflow) = $get_int_frac(bytes, radix, int_nbits, frac_nbits)?;
            if neg && abs > 0 {
                overflow = true;
            }
            let abs = if neg { abs.wrapping_neg() } else { abs };
            Ok((abs, overflow))
        }

        fn $get_int_frac(
            bytes: &[u8],
            radix: u32,
            int_nbits: u32,
            frac_nbits: u32,
        ) -> Result<(bool, $BitsU, bool), ParseFixedError> {
            let Parse { neg, int, frac } = parse_bounds(bytes, radix)?;
            let (int_val, mut overflow) = $get_int(int, radix, int_nbits);
            let (frac_val, frac_overflow) = match $get_frac(frac, radix, frac_nbits) {
                Some(val) => (val, false),
                None => (0, true),
            };
            let mut val = int_val | frac_val;
            // frac_overflow does not catch the case where:
            //  1. int is odd
            //  2. frac_nbits is 0
            //  3. frac_bytes is exactly half, e.g. "5" for decimal
            // In this case, get_frac returns 0.5 rounded to even 0.0,
            // as it does not have a way to know that int is odd.
            if frac_overflow
                || (ParseHelper::is_odd(int_val) && frac_nbits == 0 && frac_is_half(frac, radix))
            {
                let (new_val, new_overflow) = if int_nbits == 0 {
                    (val, true)
                } else {
                    val.overflowing_add(1 << frac_nbits)
                };
                if new_overflow {
                    overflow = true;
                }
                val = new_val;
            }
            Ok((neg, val, overflow))
        }

        fn $get_int(int: &[u8], radix: u32, nbits: u32) -> ($BitsU, bool) {
            const HALF: u32 = $BitsU::BITS / 2;
            if $attempt_int_half && nbits <= HALF {
                let (half, overflow) = $get_int_half(int, radix, nbits);
                return ($BitsU::from(half) << HALF, overflow);
            }

            if int.is_empty() {
                return (0, false);
            }
            let (mut parsed_int, mut overflow): ($BitsU, bool) = match radix {
                2 => ParseHelper::bin_str_int_to_bin(int),
                8 => ParseHelper::oct_str_int_to_bin(int),
                16 => ParseHelper::hex_str_int_to_bin(int),
                10 => ParseHelper::dec_str_int_to_bin(int),
                _ => unreachable!(),
            };
            let remove_bits = $BitsU::BITS - nbits;
            if nbits == 0 {
                overflow = true;
                parsed_int = 0;
            } else if remove_bits > 0 {
                if (parsed_int >> nbits) != 0 {
                    overflow = true;
                }
                parsed_int <<= remove_bits;
            }
            (parsed_int, overflow)
        }

        fn $get_frac(frac: &[u8], radix: u32, nbits: u32) -> Option<$BitsU> {
            if $attempt_frac_half && nbits <= $BitsU::BITS / 2 {
                return $get_frac_half(frac, radix, nbits).map($BitsU::from);
            }
            if frac.is_empty() {
                return Some(0);
            }
            match radix {
                2 => ParseHelper::bin_str_frac_to_bin(frac, nbits),
                8 => ParseHelper::oct_str_frac_to_bin(frac, nbits),
                16 => ParseHelper::hex_str_frac_to_bin(frac, nbits),
                10 => ParseHelper::dec_str_frac_to_bin(frac, nbits),
                _ => unreachable!(),
            }
        }
    };
}

impl_from_str! {
    FixedI8(i8), FixedU8(u8), LeEqU8;
    fn from_str_i8;
    fn from_str_u8;
    fn get_int_frac8;
    fn get_int8, (get_int8, false);
    fn get_frac8, (get_frac8, false);
}
impl_from_str! {
    FixedI16(i16), FixedU16(u16), LeEqU16;
    fn from_str_i16;
    fn from_str_u16;
    fn get_int_frac16;
    fn get_int16, (get_int8, true);
    fn get_frac16, (get_frac8, true);
}
impl_from_str! {
    FixedI32(i32), FixedU32(u32), LeEqU32;
    fn from_str_i32;
    fn from_str_u32;
    fn get_int_frac32;
    fn get_int32, (get_int16, true);
    fn get_frac32, (get_frac16, true);
}
impl_from_str! {
    FixedI64(i64), FixedU64(u64), LeEqU64;
    fn from_str_i64;
    fn from_str_u64;
    fn get_int_frac64;
    fn get_int64, (get_int32, true);
    fn get_frac64, (get_frac32, false);
}
impl_from_str! {
    FixedI128(i128), FixedU128(u128), LeEqU128;
    fn from_str_i128;
    fn from_str_u128;
    fn get_int_frac128;
    fn get_int128, (get_int64, true);
    fn get_frac128, (get_frac64, true);
}

#[cfg(test)]
mod tests {
    use crate::{
        from_str::*,
        traits::{Fixed, ToFixed},
        types::*,
    };
    use std::{
        fmt::Debug,
        format,
        string::{String, ToString},
    };

    #[test]
    fn overflowing() {
        let overflow = ParseFixedError {
            kind: ParseErrorKind::Overflow,
        };
        assert_eq!(
            U4F4::overflowing_from_str("15.5"),
            Ok((U4F4::from_bits(0xF8), false))
        );
        assert_eq!(U4F4::from_str("15.5"), Ok(U4F4::from_bits(0xF8)));
        assert_eq!(
            U4F4::overflowing_from_str("31.5"),
            Ok((U4F4::from_bits(0xF8), true))
        );
        assert_eq!(U4F4::from_str("31.5"), Err(overflow));
        assert_eq!(
            U4F4::overflowing_from_str("271.5"),
            Ok((U4F4::from_bits(0xF8), true))
        );
        assert_eq!(
            U8F0::overflowing_from_str("271"),
            Ok((U8F0::from_bits(0x0F), true))
        );
        let longer_than_8 = format!("{}", (1 << 30) + 15);
        assert_eq!(
            U8F0::overflowing_from_str(&longer_than_8),
            Ok((U8F0::from_bits(0x0F), true))
        );

        assert_eq!(
            U4F4::overflowing_from_str_binary("1111.1000"),
            Ok((U4F4::from_bits(0xF8), false))
        );
        assert_eq!(
            U4F4::from_str_binary("1111.1000"),
            Ok(U4F4::from_bits(0xF8))
        );
        assert_eq!(
            U4F4::overflowing_from_str_binary("11111.1000"),
            Ok((U4F4::from_bits(0xF8), true))
        );
        assert_eq!(U4F4::from_str_binary("11111.1000"), Err(overflow));
        assert_eq!(
            U8F0::overflowing_from_str_binary("100001111"),
            Ok((U8F0::from_bits(0x0F), true))
        );

        assert_eq!(
            U4F4::overflowing_from_str_octal("17.7"),
            Ok((U4F4::from_bits(0xFE), false))
        );
        assert_eq!(U4F4::from_str_octal("17.7"), Ok(U4F4::from_bits(0xFE)));
        assert_eq!(
            U4F4::overflowing_from_str_octal("77.7"),
            Ok((U4F4::from_bits(0xFE), true))
        );
        assert_eq!(U4F4::from_str_octal("77.7"), Err(overflow));
        assert_eq!(
            U4F4::overflowing_from_str_octal("707.7"),
            Ok((U4F4::from_bits(0x7E), true))
        );
        assert_eq!(
            U8F0::overflowing_from_str_octal("1307"),
            Ok((U8F0::from_bits(0o307), true))
        );

        assert_eq!(
            U6F10::overflowing_from_str_hex("3F.8"),
            Ok((U6F10::from_bits(0xFE00), false))
        );
        assert_eq!(U6F10::from_str_hex("3F.8"), Ok(U6F10::from_bits(0xFE00)));
        assert_eq!(
            U6F10::overflowing_from_str_hex("FF.8"),
            Ok((U6F10::from_bits(0xFE00), true))
        );
        assert_eq!(U6F10::from_str_hex("FF.8"), Err(overflow));
        assert_eq!(
            U6F10::overflowing_from_str_hex("F0F.8"),
            Ok((U6F10::from_bits(0x3E00), true))
        );
        assert_eq!(
            U16F0::overflowing_from_str_hex("100FF"),
            Ok((U16F0::from_bits(0x00FF), true))
        );
    }

    #[test]
    fn check_dec_8() {
        let two_pow = 8f64.exp2();
        let limit = 1000;
        for i in 0..limit {
            let ans = <u8 as DecToBin>::dec_to_bin(i, 8, Round::Nearest);
            let approx = two_pow * f64::from(i) / f64::from(limit);
            let error = (ans.map(f64::from).unwrap_or(two_pow) - approx).abs();
            assert!(
                error <= 0.5,
                "i {} ans {:?}  approx {} error {}",
                i,
                ans,
                approx,
                error
            );
        }
    }

    #[test]
    fn check_dec_16() {
        let two_pow = 16f64.exp2();
        let limit = 1_000_000;
        for i in 0..limit {
            let ans = <u16 as DecToBin>::dec_to_bin(i, 16, Round::Nearest);
            let approx = two_pow * f64::from(i) / f64::from(limit);
            let error = (ans.map(f64::from).unwrap_or(two_pow) - approx).abs();
            assert!(
                error <= 0.5,
                "i {} ans {:?}  approx {} error {}",
                i,
                ans,
                approx,
                error
            );
        }
    }

    #[test]
    fn check_dec_32() {
        let two_pow = 32f64.exp2();
        let limit = 10_000_000_000_000;
        for iter in 0..1_000_000 {
            for &i in &[
                iter,
                limit / 4 - 1 - iter,
                limit / 4 + iter,
                limit / 3 - 1 - iter,
                limit / 3 + iter,
                limit / 2 - 1 - iter,
                limit / 2 + iter,
                limit - iter - 1,
            ] {
                let ans = <u32 as DecToBin>::dec_to_bin(i, 32, Round::Nearest);
                let approx = two_pow * i as f64 / limit as f64;
                let error = (ans.map(f64::from).unwrap_or(two_pow) - approx).abs();
                assert!(
                    error <= 0.5,
                    "i {} ans {:?}  approx {} error {}",
                    i,
                    ans,
                    approx,
                    error
                );
            }
        }
    }

    #[test]
    fn check_dec_64() {
        let two_pow = 64f64.exp2();
        let limit = 1_000_000_000_000_000_000_000_000_000;
        for iter in 0..200_000 {
            for &i in &[
                iter,
                limit / 4 - 1 - iter,
                limit / 4 + iter,
                limit / 3 - 1 - iter,
                limit / 3 + iter,
                limit / 2 - 1 - iter,
                limit / 2 + iter,
                limit - iter - 1,
            ] {
                let ans = <u64 as DecToBin>::dec_to_bin(i, 64, Round::Nearest);
                let approx = two_pow * i as f64 / limit as f64;
                let error = (ans.map(|x| x as f64).unwrap_or(two_pow) - approx).abs();
                assert!(
                    error <= 0.5,
                    "i {} ans {:?}  approx {} error {}",
                    i,
                    ans,
                    approx,
                    error
                );
            }
        }
    }

    #[test]
    fn check_dec_128() {
        let nines = 10u128.pow(27) - 1;
        let zeros = 0;
        let too_big = <u128 as DecToBin>::dec_to_bin((nines, nines), 128, Round::Nearest);
        assert_eq!(too_big, None);
        let big = <u128 as DecToBin>::dec_to_bin((nines, zeros), 128, Round::Nearest);
        assert_eq!(
            big,
            Some(340_282_366_920_938_463_463_374_607_091_485_844_535)
        );
        let small = <u128 as DecToBin>::dec_to_bin((zeros, nines), 128, Round::Nearest);
        assert_eq!(small, Some(340_282_366_921));
        let zero = <u128 as DecToBin>::dec_to_bin((zeros, zeros), 128, Round::Nearest);
        assert_eq!(zero, Some(0));
        let x = <u128 as DecToBin>::dec_to_bin(
            (
                123_456_789_012_345_678_901_234_567,
                987_654_321_098_765_432_109_876_543,
            ),
            128,
            Round::Nearest,
        );
        assert_eq!(x, Some(42_010_168_377_579_896_403_540_037_811_203_677_112));

        let eights = 888_888_888_888_888_888_888_888_888;
        let narrow = <u128 as DecToBin>::dec_to_bin((eights, zeros), 40, Round::Nearest);
        assert_eq!(narrow, Some(977_343_669_134));
    }

    #[test]
    fn check_parse_bounds() {
        let Parse { neg, int, frac } = parse_bounds(b"-12.34", 10).unwrap();
        assert_eq!((neg, int, frac), (true, &b"12"[..], &b"34"[..]));
        let Parse { neg, int, frac } = parse_bounds(b"012.", 10).unwrap();
        assert_eq!((neg, int, frac), (false, &b"12"[..], &b""[..]));
        let Parse { neg, int, frac } = parse_bounds(b"+.340", 10).unwrap();
        assert_eq!((neg, int, frac), (false, &b""[..], &b"34"[..]));
        let Parse { neg, int, frac } = parse_bounds(b"0", 10).unwrap();
        assert_eq!((neg, int, frac), (false, &b""[..], &b""[..]));
        let Parse { neg, int, frac } = parse_bounds(b"-.C1A0", 16).unwrap();
        assert_eq!((neg, int, frac), (true, &b""[..], &b"C1A"[..]));

        let ParseFixedError { kind } = parse_bounds(b"0 ", 10).unwrap_err();
        assert_eq!(kind, ParseErrorKind::InvalidDigit);
        let ParseFixedError { kind } = parse_bounds(b"+-", 10).unwrap_err();
        assert_eq!(kind, ParseErrorKind::InvalidDigit);
        let ParseFixedError { kind } = parse_bounds(b"+.", 10).unwrap_err();
        assert_eq!(kind, ParseErrorKind::NoDigits);
        let ParseFixedError { kind } = parse_bounds(b".1.", 10).unwrap_err();
        assert_eq!(kind, ParseErrorKind::TooManyPoints);
        let ParseFixedError { kind } = parse_bounds(b"1+2", 10).unwrap_err();
        assert_eq!(kind, ParseErrorKind::InvalidDigit);
        let ParseFixedError { kind } = parse_bounds(b"1-2", 10).unwrap_err();
        assert_eq!(kind, ParseErrorKind::InvalidDigit);
    }

    fn assert_ok<F>(s: &str, radix: u32, bits: F::Bits, overflow: bool)
    where
        F: Fixed + FromStrRadix<Err = ParseFixedError>,
        F::Bits: Eq + Debug,
    {
        match F::overflowing_from_str_radix(s, radix) {
            Ok((f, o)) => {
                assert_eq!(f.to_bits(), bits, "{} -> ({}, {})", s, f, o);
                assert_eq!(o, overflow, "{} -> ({}, {})", s, f, o);
            }
            Err(e) => panic!("could not parse {}: {}", s, e),
        }
    }

    #[test]
    fn check_i8_u8_from_str() {
        assert_ok::<I0F8>("-1", 10, 0x00, true);
        assert_ok::<I0F8>("-0.502", 10, 0x7F, true);
        assert_ok::<I0F8>("-0.501", 10, -0x80, false);
        assert_ok::<I0F8>("0.498", 10, 0x7F, false);
        assert_ok::<I0F8>("0.499", 10, -0x80, true);
        assert_ok::<I0F8>("1", 10, 0x00, true);

        assert_ok::<I4F4>("-8.04", 10, 0x7F, true);
        assert_ok::<I4F4>("-8.03", 10, -0x80, false);
        assert_ok::<I4F4>("7.96", 10, 0x7F, false);
        assert_ok::<I4F4>("7.97", 10, -0x80, true);

        assert_ok::<I8F0>("-128.501", 10, 0x7F, true);
        // exact tie, round up to even
        assert_ok::<I8F0>("-128.5", 10, -0x80, false);
        assert_ok::<I8F0>("127.499", 10, 0x7F, false);
        // exact tie, round up to even
        assert_ok::<I8F0>("127.5", 10, -0x80, true);

        assert_ok::<U0F8>("-0", 10, 0x00, false);
        assert_ok::<U0F8>("0.498", 10, 0x7F, false);
        assert_ok::<U0F8>("0.499", 10, 0x80, false);
        assert_ok::<U0F8>("0.998", 10, 0xFF, false);
        assert_ok::<U0F8>("0.999", 10, 0x00, true);
        assert_ok::<U0F8>("1", 10, 0x00, true);

        assert_ok::<U4F4>("7.96", 10, 0x7F, false);
        assert_ok::<U4F4>("7.97", 10, 0x80, false);
        assert_ok::<U4F4>("15.96", 10, 0xFF, false);
        assert_ok::<U4F4>("15.97", 10, 0x00, true);

        assert_ok::<U8F0>("127.499", 10, 0x7F, false);
        // exact tie, round up to even
        assert_ok::<U8F0>("127.5", 10, 0x80, false);
        assert_ok::<U8F0>("255.499", 10, 0xFF, false);
        // exact tie, round up to even
        assert_ok::<U8F0>("255.5", 10, 0x00, true);
    }

    #[test]
    fn check_i16_u16_from_str() {
        assert_ok::<I0F16>("-1", 10, 0x00, true);
        assert_ok::<I0F16>("-0.500008", 10, 0x7FFF, true);
        assert_ok::<I0F16>("-0.500007", 10, -0x8000, false);
        assert_ok::<I0F16>("+0.499992", 10, 0x7FFF, false);
        assert_ok::<I0F16>("+0.499993", 10, -0x8000, true);
        assert_ok::<I0F16>("1", 10, 0x0000, true);

        assert_ok::<I8F8>("-128.002", 10, 0x7FFF, true);
        assert_ok::<I8F8>("-128.001", 10, -0x8000, false);
        assert_ok::<I8F8>("+127.998", 10, 0x7FFF, false);
        assert_ok::<I8F8>("+127.999", 10, -0x8000, true);

        assert_ok::<I16F0>("-32768.500001", 10, 0x7FFF, true);
        // exact tie, round up to even
        assert_ok::<I16F0>("-32768.5", 10, -0x8000, false);
        assert_ok::<I16F0>("+32767.499999", 10, 0x7FFF, false);
        // exact tie, round up to even
        assert_ok::<I16F0>("+32767.5", 10, -0x8000, true);

        assert_ok::<U0F16>("-0", 10, 0x0000, false);
        assert_ok::<U0F16>("0.499992", 10, 0x7FFF, false);
        assert_ok::<U0F16>("0.499993", 10, 0x8000, false);
        assert_ok::<U0F16>("0.999992", 10, 0xFFFF, false);
        assert_ok::<U0F16>("0.999993", 10, 0x0000, true);
        assert_ok::<U0F16>("1", 10, 0x0000, true);

        assert_ok::<U8F8>("127.998", 10, 0x7FFF, false);
        assert_ok::<U8F8>("127.999", 10, 0x8000, false);
        assert_ok::<U8F8>("255.998", 10, 0xFFFF, false);
        assert_ok::<U8F8>("255.999", 10, 0x0000, true);

        assert_ok::<U16F0>("32767.499999", 10, 0x7FFF, false);
        // exact tie, round up to even
        assert_ok::<U16F0>("32767.5", 10, 0x8000, false);
        assert_ok::<U16F0>("65535.499999", 10, 0xFFFF, false);
        // exact tie, round up to even
        assert_ok::<U16F0>("65535.5", 10, 0x0000, true);
    }

    #[test]
    fn check_i32_u32_from_str() {
        assert_ok::<I0F32>("-1", 10, 0x0000_0000, true);
        assert_ok::<I0F32>("-0.5000000002", 10, 0x7FFF_FFFF, true);
        assert_ok::<I0F32>("-0.5000000001", 10, -0x8000_0000, false);
        assert_ok::<I0F32>("0.4999999998", 10, 0x7FFF_FFFF, false);
        assert_ok::<I0F32>("0.4999999999", 10, -0x8000_0000, true);
        assert_ok::<I0F32>("1", 10, 0x0000_0000, true);

        assert_ok::<I16F16>("-32768.000008", 10, 0x7FFF_FFFF, true);
        assert_ok::<I16F16>("-32768.000007", 10, -0x8000_0000, false);
        assert_ok::<I16F16>("32767.999992", 10, 0x7FFF_FFFF, false);
        assert_ok::<I16F16>("32767.999993", 10, -0x8000_0000, true);

        assert_ok::<I32F0>("-2147483648.5000000001", 10, 0x7FFF_FFFF, true);
        // exact tie, round up to even
        assert_ok::<I32F0>("-2147483648.5", 10, -0x8000_0000, false);
        assert_ok::<I32F0>("2147483647.4999999999", 10, 0x7FFF_FFFF, false);
        // exact tie, round up to even
        assert_ok::<I32F0>("2147483647.5", 10, -0x8000_0000, true);

        assert_ok::<U0F32>("-0", 10, 0x0000_0000, false);
        assert_ok::<U0F32>("0.4999999998", 10, 0x7FFF_FFFF, false);
        assert_ok::<U0F32>("0.4999999999", 10, 0x8000_0000, false);
        assert_ok::<U0F32>("0.9999999998", 10, 0xFFFF_FFFF, false);
        assert_ok::<U0F32>("0.9999999999", 10, 0x0000_0000, true);
        assert_ok::<U0F32>("1", 10, 0x0000_0000, true);

        assert_ok::<U16F16>("32767.999992", 10, 0x7FFF_FFFF, false);
        assert_ok::<U16F16>("32767.999993", 10, 0x8000_0000, false);
        assert_ok::<U16F16>("65535.999992", 10, 0xFFFF_FFFF, false);
        assert_ok::<U16F16>("65535.999993", 10, 0x0000_0000, true);

        assert_ok::<U32F0>("2147483647.4999999999", 10, 0x7FFF_FFFF, false);
        // exact tie, round up to even
        assert_ok::<U32F0>("2147483647.5", 10, 0x8000_0000, false);
        assert_ok::<U32F0>("4294967295.4999999999", 10, 0xFFFF_FFFF, false);
        // exact tie, round up to even
        assert_ok::<U32F0>("4294967295.5", 10, 0x0000_0000, true);
    }

    #[test]
    fn check_i64_u64_from_str() {
        assert_ok::<I0F64>("-1", 10, 0x0000_0000_0000_0000, true);
        assert_ok::<I0F64>("-0.50000000000000000003", 10, 0x7FFF_FFFF_FFFF_FFFF, true);
        assert_ok::<I0F64>("-0.50000000000000000002", 10, -0x8000_0000_0000_0000, false);
        assert_ok::<I0F64>("+0.49999999999999999997", 10, 0x7FFF_FFFF_FFFF_FFFF, false);
        assert_ok::<I0F64>("+0.49999999999999999998", 10, -0x8000_0000_0000_0000, true);
        assert_ok::<I0F64>("1", 10, 0x0000_0000_0000_0000, true);

        assert_ok::<I32F32>("-2147483648.0000000002", 10, 0x7FFF_FFFF_FFFF_FFFF, true);
        assert_ok::<I32F32>("-2147483648.0000000001", 10, -0x8000_0000_0000_0000, false);
        assert_ok::<I32F32>("2147483647.9999999998", 10, 0x7FFF_FFFF_FFFF_FFFF, false);
        assert_ok::<I32F32>("2147483647.9999999999", 10, -0x8000_0000_0000_0000, true);

        assert_ok::<I64F0>(
            "-9223372036854775808.50000000000000000001",
            10,
            0x7FFF_FFFF_FFFF_FFFF,
            true,
        );
        // exact tie, round up to even
        assert_ok::<I64F0>("-9223372036854775808.5", 10, -0x8000_0000_0000_0000, false);
        assert_ok::<I64F0>(
            "9223372036854775807.49999999999999999999",
            10,
            0x7FFF_FFFF_FFFF_FFFF,
            false,
        );
        // exact tie, round up to even
        assert_ok::<I64F0>("9223372036854775807.5", 10, -0x8000_0000_0000_0000, true);

        assert_ok::<U0F64>("-0", 10, 0x0000_0000_0000_0000, false);
        assert_ok::<U0F64>("0.49999999999999999997", 10, 0x7FFF_FFFF_FFFF_FFFF, false);
        assert_ok::<U0F64>("0.49999999999999999998", 10, 0x8000_0000_0000_0000, false);
        assert_ok::<U0F64>("0.99999999999999999997", 10, 0xFFFF_FFFF_FFFF_FFFF, false);
        assert_ok::<U0F64>("0.99999999999999999998", 10, 0x0000_0000_0000_0000, true);
        assert_ok::<U0F64>("1", 10, 0x0000_0000_0000_0000, true);

        assert_ok::<U32F32>("2147483647.9999999998", 10, 0x7FFF_FFFF_FFFF_FFFF, false);
        assert_ok::<U32F32>("2147483647.9999999999", 10, 0x8000_0000_0000_0000, false);
        assert_ok::<U32F32>("4294967295.9999999998", 10, 0xFFFF_FFFF_FFFF_FFFF, false);
        assert_ok::<U32F32>("4294967295.9999999999", 10, 0x0000_0000_0000_0000, true);

        assert_ok::<U64F0>(
            "9223372036854775807.49999999999999999999",
            10,
            0x7FFF_FFFF_FFFF_FFFF,
            false,
        );
        // exact tie, round up to even
        assert_ok::<U64F0>("9223372036854775807.5", 10, 0x8000_0000_0000_0000, false);
        assert_ok::<U64F0>(
            "18446744073709551615.49999999999999999999",
            10,
            0xFFFF_FFFF_FFFF_FFFF,
            false,
        );
        // exact tie, round up to even
        assert_ok::<U64F0>("18446744073709551615.5", 10, 0x0000_0000_0000_0000, true);
    }

    #[test]
    fn check_i128_u128_from_str() {
        assert_ok::<I0F128>("-1", 10, 0x0000_0000_0000_0000_0000_0000_0000_0000, true);
        assert_ok::<I0F128>(
            "-0.500000000000000000000000000000000000002",
            10,
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
            true,
        );
        assert_ok::<I0F128>(
            "-0.500000000000000000000000000000000000001",
            10,
            -0x8000_0000_0000_0000_0000_0000_0000_0000,
            false,
        );
        assert_ok::<I0F128>(
            "0.499999999999999999999999999999999999998",
            10,
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
            false,
        );
        assert_ok::<I0F128>(
            "0.499999999999999999999999999999999999999",
            10,
            -0x8000_0000_0000_0000_0000_0000_0000_0000,
            true,
        );
        assert_ok::<I0F128>("1", 10, 0x0000_0000_0000_0000_0000_0000_0000_0000, true);

        assert_ok::<I64F64>(
            "-9223372036854775808.00000000000000000003",
            10,
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
            true,
        );
        assert_ok::<I64F64>(
            "-9223372036854775808.00000000000000000002",
            10,
            -0x8000_0000_0000_0000_0000_0000_0000_0000,
            false,
        );
        assert_ok::<I64F64>(
            "9223372036854775807.99999999999999999997",
            10,
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
            false,
        );
        assert_ok::<I64F64>(
            "9223372036854775807.99999999999999999998",
            10,
            -0x8000_0000_0000_0000_0000_0000_0000_0000,
            true,
        );

        assert_ok::<I128F0>(
            "-170141183460469231731687303715884105728.5000000000000000000000000000000000000001",
            10,
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
            true,
        );
        // exact tie, round up to even
        assert_ok::<I128F0>(
            "-170141183460469231731687303715884105728.5",
            10,
            -0x8000_0000_0000_0000_0000_0000_0000_0000,
            false,
        );
        assert_ok::<I128F0>(
            "170141183460469231731687303715884105727.4999999999999999999999999999999999999999",
            10,
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
            false,
        );
        // exact tie, round up to even
        assert_ok::<I128F0>(
            "170141183460469231731687303715884105727.5",
            10,
            -0x8000_0000_0000_0000_0000_0000_0000_0000,
            true,
        );

        assert_ok::<U0F128>("-0", 10, 0x0000_0000_0000_0000_0000_0000_0000_0000, false);
        assert_ok::<U0F128>(
            "0.499999999999999999999999999999999999998",
            10,
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
            false,
        );
        assert_ok::<U0F128>(
            "0.499999999999999999999999999999999999999",
            10,
            0x8000_0000_0000_0000_0000_0000_0000_0000,
            false,
        );
        assert_ok::<U0F128>(
            "0.999999999999999999999999999999999999998",
            10,
            0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
            false,
        );
        assert_ok::<U0F128>(
            "0.999999999999999999999999999999999999999",
            10,
            0x0000_0000_0000_0000_0000_0000_0000_0000,
            true,
        );
        assert_ok::<U0F128>("1", 10, 0x0000_0000_0000_0000_0000_0000_0000_0000, true);

        assert_ok::<U64F64>(
            "9223372036854775807.99999999999999999997",
            10,
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
            false,
        );
        assert_ok::<U64F64>(
            "9223372036854775807.99999999999999999998",
            10,
            0x8000_0000_0000_0000_0000_0000_0000_0000,
            false,
        );
        assert_ok::<U64F64>(
            "18446744073709551615.99999999999999999997",
            10,
            0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
            false,
        );
        assert_ok::<U64F64>(
            "18446744073709551615.99999999999999999998",
            10,
            0x0000_0000_0000_0000_0000_0000_0000_0000,
            true,
        );

        assert_ok::<U128F0>(
            "170141183460469231731687303715884105727.4999999999999999999999999999999999999999",
            10,
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
            false,
        );
        // exact tie, round up to even
        assert_ok::<U128F0>(
            "170141183460469231731687303715884105727.5",
            10,
            0x8000_0000_0000_0000_0000_0000_0000_0000,
            false,
        );
        assert_ok::<U128F0>(
            "340282366920938463463374607431768211455.4999999999999999999999999999999999999999",
            10,
            0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
            false,
        );
        // exact tie, round up to even
        assert_ok::<U128F0>(
            "340282366920938463463374607431768211455.5",
            10,
            0x0000_0000_0000_0000_0000_0000_0000_0000,
            true,
        );
    }

    #[test]
    fn check_i16_u16_from_str_binary() {
        assert_ok::<I0F16>("-1", 2, 0x0000, true);
        assert_ok::<I0F16>("-0.100000000000000011", 2, 0x7FFF, true);
        assert_ok::<I0F16>("-0.100000000000000010", 2, -0x8000, false);
        assert_ok::<I0F16>("-0.011111111111111110", 2, -0x8000, false);
        assert_ok::<I0F16>("+0.011111111111111101", 2, 0x7FFF, false);
        assert_ok::<I0F16>("+0.011111111111111110", 2, -0x8000, true);
        assert_ok::<I0F16>("1", 2, 0x0000, true);

        assert_ok::<I8F8>("-10000000.0000000011", 2, 0x7FFF, true);
        assert_ok::<I8F8>("-10000000.0000000010", 2, -0x8000, false);
        assert_ok::<I8F8>("-01111111.1111111110", 2, -0x8000, false);
        assert_ok::<I8F8>("+01111111.1111111101", 2, 0x7FFF, false);
        assert_ok::<I8F8>("+01111111.1111111110", 2, -0x8000, true);

        assert_ok::<I16F0>("-1000000000000000.11", 2, 0x7FFF, true);
        assert_ok::<I16F0>("-1000000000000000.10", 2, -0x8000, false);
        assert_ok::<I16F0>("-0111111111111111.10", 2, -0x8000, false);
        assert_ok::<I16F0>("+0111111111111111.01", 2, 0x7FFF, false);
        assert_ok::<I16F0>("+0111111111111111.10", 2, -0x8000, true);

        assert_ok::<U0F16>("-0", 2, 0x0000, false);
        assert_ok::<U0F16>("0.011111111111111101", 2, 0x7FFF, false);
        assert_ok::<U0F16>("0.011111111111111110", 2, 0x8000, false);
        assert_ok::<U0F16>("0.111111111111111101", 2, 0xFFFF, false);
        assert_ok::<U0F16>("0.111111111111111110", 2, 0x0000, true);
        assert_ok::<U0F16>("1", 2, 0x0000, true);

        assert_ok::<U8F8>("01111111.1111111101", 2, 0x7FFF, false);
        assert_ok::<U8F8>("01111111.1111111110", 2, 0x8000, false);
        assert_ok::<U8F8>("11111111.1111111101", 2, 0xFFFF, false);
        assert_ok::<U8F8>("11111111.1111111110", 2, 0x0000, true);

        assert_ok::<U16F0>("0111111111111111.01", 2, 0x7FFF, false);
        assert_ok::<U16F0>("0111111111111111.10", 2, 0x8000, false);
        assert_ok::<U16F0>("1111111111111111.01", 2, 0xFFFF, false);
        assert_ok::<U16F0>("1111111111111111.10", 2, 0x0000, true);
    }

    #[test]
    fn check_i16_u16_from_str_octal() {
        assert_ok::<I0F16>("-1", 8, 0x0000, true);
        assert_ok::<I0F16>("-0.400003", 8, 0x7FFF, true);
        assert_ok::<I0F16>("-0.400002", 8, -0x8000, false);
        assert_ok::<I0F16>("-0.377776", 8, -0x8000, false);
        assert_ok::<I0F16>("+0.377775", 8, 0x7FFF, false);
        assert_ok::<I0F16>("+0.377776", 8, -0x8000, true);
        assert_ok::<I0F16>("1", 8, 0x0000, true);

        assert_ok::<I8F8>("-200.0011", 8, 0x7FFF, true);
        assert_ok::<I8F8>("-200.0010", 8, -0x8000, false);
        assert_ok::<I8F8>("-177.7770", 8, -0x8000, false);
        assert_ok::<I8F8>("+177.7767", 8, 0x7FFF, false);
        assert_ok::<I8F8>("+177.7770", 8, -0x8000, true);

        assert_ok::<I16F0>("-100000.5", 8, 0x7FFF, true);
        assert_ok::<I16F0>("-100000.4", 8, -0x8000, false);
        assert_ok::<I16F0>("-077777.4", 8, -0x8000, false);
        assert_ok::<I16F0>("+077777.3", 8, 0x7FFF, false);
        assert_ok::<I16F0>("+077777.4", 8, -0x8000, true);

        assert_ok::<U0F16>("-0", 8, 0x0000, false);
        assert_ok::<U0F16>("0.377775", 8, 0x7FFF, false);
        assert_ok::<U0F16>("0.377776", 8, 0x8000, false);
        assert_ok::<U0F16>("0.777775", 8, 0xFFFF, false);
        assert_ok::<U0F16>("0.777776", 8, 0x0000, true);
        assert_ok::<U0F16>("1", 8, 0x0000, true);

        assert_ok::<U8F8>("177.7767", 8, 0x7FFF, false);
        assert_ok::<U8F8>("177.7770", 8, 0x8000, false);
        assert_ok::<U8F8>("377.7767", 8, 0xFFFF, false);
        assert_ok::<U8F8>("377.7770", 8, 0x0000, true);

        assert_ok::<U16F0>("077777.3", 8, 0x7FFF, false);
        assert_ok::<U16F0>("077777.4", 8, 0x8000, false);
        assert_ok::<U16F0>("177777.3", 8, 0xFFFF, false);
        assert_ok::<U16F0>("177777.4", 8, 0x0000, true);
    }

    #[test]
    fn check_i16_u16_from_str_hex() {
        assert_ok::<I0F16>("-1", 16, 0x0000, true);
        assert_ok::<I0F16>("-0.80009", 16, 0x7FFF, true);
        assert_ok::<I0F16>("-0.80008", 16, -0x8000, false);
        assert_ok::<I0F16>("-0.7FFF8", 16, -0x8000, false);
        assert_ok::<I0F16>("+0.7FFF7", 16, 0x7FFF, false);
        assert_ok::<I0F16>("+0.7FFF8", 16, -0x8000, true);
        assert_ok::<I0F16>("1", 16, 0x0000, true);

        assert_ok::<I8F8>("-80.009", 16, 0x7FFF, true);
        assert_ok::<I8F8>("-80.008", 16, -0x8000, false);
        assert_ok::<I8F8>("-7F.FF8", 16, -0x8000, false);
        assert_ok::<I8F8>("+7F.FF7", 16, 0x7FFF, false);
        assert_ok::<I8F8>("+7F.FF8", 16, -0x8000, true);

        assert_ok::<I16F0>("-8000.9", 16, 0x7FFF, true);
        assert_ok::<I16F0>("-8000.8", 16, -0x8000, false);
        assert_ok::<I16F0>("-7FFF.8", 16, -0x8000, false);
        assert_ok::<I16F0>("+7FFF.7", 16, 0x7FFF, false);
        assert_ok::<I16F0>("+7FFF.8", 16, -0x8000, true);

        assert_ok::<U0F16>("-0", 16, 0x0000, false);
        assert_ok::<U0F16>("0.7FFF7", 16, 0x7FFF, false);
        assert_ok::<U0F16>("0.7FFF8", 16, 0x8000, false);
        assert_ok::<U0F16>("0.FFFF7", 16, 0xFFFF, false);
        assert_ok::<U0F16>("0.FFFF8", 16, 0x0000, true);
        assert_ok::<U0F16>("1", 16, 0x0000, true);

        assert_ok::<U8F8>("7F.FF7", 16, 0x7FFF, false);
        assert_ok::<U8F8>("7F.FF8", 16, 0x8000, false);
        assert_ok::<U8F8>("FF.FF7", 16, 0xFFFF, false);
        assert_ok::<U8F8>("FF.FF8", 16, 0x0000, true);

        assert_ok::<U16F0>("7FFF.7", 16, 0x7FFF, false);
        assert_ok::<U16F0>("7FFF.8", 16, 0x8000, false);
        assert_ok::<U16F0>("FFFF.7", 16, 0xFFFF, false);
        assert_ok::<U16F0>("FFFF.8", 16, 0x0000, true);
    }

    #[test]
    fn check_i128_u128_from_str_hex() {
        assert_ok::<I0F128>("-1", 16, 0x0000_0000_0000_0000_0000_0000_0000_0000, true);
        assert_ok::<I0F128>(
            "-0.800000000000000000000000000000009",
            16,
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
            true,
        );
        assert_ok::<I0F128>(
            "-0.800000000000000000000000000000008",
            16,
            -0x8000_0000_0000_0000_0000_0000_0000_0000,
            false,
        );
        assert_ok::<I0F128>(
            "-0.7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF8",
            16,
            -0x8000_0000_0000_0000_0000_0000_0000_0000,
            false,
        );
        assert_ok::<I0F128>(
            "+0.7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF7",
            16,
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
            false,
        );
        assert_ok::<I0F128>(
            "+0.7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF8",
            16,
            -0x8000_0000_0000_0000_0000_0000_0000_0000,
            true,
        );
        assert_ok::<I0F128>("1", 16, 0x0000_0000_0000_0000_0000_0000_0000_0000, true);

        assert_ok::<I64F64>(
            "-8000000000000000.00000000000000009",
            16,
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
            true,
        );
        assert_ok::<I64F64>(
            "-8000000000000000.00000000000000008",
            16,
            -0x8000_0000_0000_0000_0000_0000_0000_0000,
            false,
        );
        assert_ok::<I64F64>(
            "-7FFFFFFFFFFFFFFF.FFFFFFFFFFFFFFFF8",
            16,
            -0x8000_0000_0000_0000_0000_0000_0000_0000,
            false,
        );
        assert_ok::<I64F64>(
            "+7FFFFFFFFFFFFFFF.FFFFFFFFFFFFFFFF7",
            16,
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
            false,
        );
        assert_ok::<I64F64>(
            "+7FFFFFFFFFFFFFFF.FFFFFFFFFFFFFFFF8",
            16,
            -0x8000_0000_0000_0000_0000_0000_0000_0000,
            true,
        );

        assert_ok::<I128F0>(
            "-80000000000000000000000000000000.9",
            16,
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
            true,
        );
        assert_ok::<I128F0>(
            "-80000000000000000000000000000000.8",
            16,
            -0x8000_0000_0000_0000_0000_0000_0000_0000,
            false,
        );
        assert_ok::<I128F0>(
            "-7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF.8",
            16,
            -0x8000_0000_0000_0000_0000_0000_0000_0000,
            false,
        );
        assert_ok::<I128F0>(
            "+7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF.7",
            16,
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
            false,
        );
        assert_ok::<I128F0>(
            "+7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF.8",
            16,
            -0x8000_0000_0000_0000_0000_0000_0000_0000,
            true,
        );

        assert_ok::<U0F128>("-0", 16, 0x0000_0000_0000_0000_0000_0000_0000_0000, false);
        assert_ok::<U0F128>(
            "0.7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF7",
            16,
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
            false,
        );
        assert_ok::<U0F128>(
            "0.7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF8",
            16,
            0x8000_0000_0000_0000_0000_0000_0000_0000,
            false,
        );
        assert_ok::<U0F128>(
            "0.FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF7",
            16,
            0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
            false,
        );
        assert_ok::<U0F128>(
            "0.FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF8",
            16,
            0x0000_0000_0000_0000_0000_0000_0000_0000,
            true,
        );
        assert_ok::<U0F128>("1", 16, 0x0000_0000_0000_0000_0000_0000_0000_0000, true);

        assert_ok::<U64F64>(
            "7FFFFFFFFFFFFFFF.FFFFFFFFFFFFFFFF7",
            16,
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
            false,
        );
        assert_ok::<U64F64>(
            "7FFFFFFFFFFFFFFF.FFFFFFFFFFFFFFFF8",
            16,
            0x8000_0000_0000_0000_0000_0000_0000_0000,
            false,
        );
        assert_ok::<U64F64>(
            "FFFFFFFFFFFFFFFF.FFFFFFFFFFFFFFFF7",
            16,
            0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
            false,
        );
        assert_ok::<U64F64>(
            "FFFFFFFFFFFFFFFF.FFFFFFFFFFFFFFFF8",
            16,
            0x0000_0000_0000_0000_0000_0000_0000_0000,
            true,
        );

        assert_ok::<U128F0>(
            "7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF.7",
            16,
            0x7FFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
            false,
        );
        assert_ok::<U128F0>(
            "7FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF.8",
            16,
            0x8000_0000_0000_0000_0000_0000_0000_0000,
            false,
        );
        assert_ok::<U128F0>(
            "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF.7",
            16,
            0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF,
            false,
        );
        assert_ok::<U128F0>(
            "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF.8",
            16,
            0x0000_0000_0000_0000_0000_0000_0000_0000,
            true,
        );
    }

    // For an odd prefix, e.g. eps = 0.125
    // zero = 0.125
    // gt_0 = 0.125000001
    // max = max_int.874999999
    // overflow = max_int.875
    struct Fractions {
        zero: String,
        gt_0: String,
        max: String,
        over: String,
    }
    fn without_last(a: &str) -> &str {
        &a[..a.len() - 1]
    }
    fn make_fraction_strings(max_int: &str, eps_frac: &str) -> Fractions {
        let eps_frac_compl: String = eps_frac
            .chars()
            .map(|digit| (b'0' + b'9' - digit as u8) as char)
            .collect();

        let zero = String::from("0.") + eps_frac;
        let gt_0 = String::from(&*zero) + "000001";
        let max = String::from(max_int) + &eps_frac_compl + "999999";
        let over = String::from(max_int) + without_last(&eps_frac_compl) + "5";
        Fractions {
            zero,
            gt_0,
            max,
            over,
        }
    }

    // check that for example for four fractional bits,
    //   * 0.03125 (1/32) is parsed as 0
    //   * 0.03125000001 (just above 1/32) is parsed as 0.0625 (1/16)
    //   * odd.96874999999 (just below 31/32) is parsed as 0.9375 (15/16)
    //   * odd.96875 (31/32) is parsed as odd + 1
    #[test]
    fn check_exact_decimal() {
        let max_int_0 = String::from("0.");
        let max_int_4 = String::from("15.");
        let max_int_8 = format!("{}.", !0u8);
        let max_int_16 = format!("{}.", !0u16);
        let max_int_28 = format!("{}.", !0u32 >> 4);
        let max_int_32 = format!("{}.", !0u32);
        let max_int_64 = format!("{}.", !0u64);
        let max_int_124 = format!("{}.", !0u128 >> 4);
        let max_int_128 = format!("{}.", !0u128);

        // Note: fractions can be generated with this:
        //
        //     use rug::Integer;
        //     for &i in &[0, 4, 8, 16, 28, 32, 64, 124, 128] {
        //         let eps = Integer::from(Integer::u_pow_u(5, i + 1));
        //         println!("let eps_{} = \"{:02$}\";", i, eps, i as usize + 1);
        //     }

        // eps_0 = 0.5 >> 0 = 0.5
        // eps_4 = 0.5 >> 4 = 0.03125
        // eps_8 = 0.5 >> 8 = 0.001953125
        // etc.
        let eps_0 = "5";
        let eps_4 = "03125";
        let eps_8 = "001953125";
        let eps_16 = "00000762939453125";
        let eps_28 = "00000000186264514923095703125";
        let eps_32 = "000000000116415321826934814453125";
        let eps_64 = "00000000000000000002710505431213761085018632002174854278564453125";
        let eps_124 = "0000000000000000000000000000000000000235098870164457501593747307\
                       4444491355637331113544175043017503412556834518909454345703125";
        let eps_128 = "0000000000000000000000000000000000000014693679385278593849609206\
                       71527807097273331945965109401885939632848021574318408966064453125";

        let frac_0_8 = make_fraction_strings(&max_int_0, eps_8);
        assert_ok::<U0F8>(&frac_0_8.zero, 10, 0, false);
        assert_ok::<U0F8>(&frac_0_8.gt_0, 10, 1, false);
        assert_ok::<U0F8>(&frac_0_8.max, 10, !0, false);
        assert_ok::<U0F8>(&frac_0_8.over, 10, 0, true);

        let frac_4_4 = make_fraction_strings(&max_int_4, eps_4);
        assert_ok::<U4F4>(&frac_4_4.zero, 10, 0, false);
        assert_ok::<U4F4>(&frac_4_4.gt_0, 10, 1, false);
        assert_ok::<U4F4>(&frac_4_4.max, 10, !0, false);
        assert_ok::<U4F4>(&frac_4_4.over, 10, 0, true);

        let frac_8_0 = make_fraction_strings(&max_int_8, eps_0);
        assert_ok::<U8F0>(&frac_8_0.zero, 10, 0, false);
        assert_ok::<U8F0>(&frac_8_0.gt_0, 10, 1, false);
        assert_ok::<U8F0>(&frac_8_0.max, 10, !0, false);
        assert_ok::<U8F0>(&frac_8_0.over, 10, 0, true);

        let frac_0_32 = make_fraction_strings(&max_int_0, eps_32);
        assert_ok::<U0F32>(&frac_0_32.zero, 10, 0, false);
        assert_ok::<U0F32>(&frac_0_32.gt_0, 10, 1, false);
        assert_ok::<U0F32>(&frac_0_32.max, 10, !0, false);
        assert_ok::<U0F32>(&frac_0_32.over, 10, 0, true);

        let frac_4_28 = make_fraction_strings(&max_int_4, eps_28);
        assert_ok::<U4F28>(&frac_4_28.zero, 10, 0, false);
        assert_ok::<U4F28>(&frac_4_28.gt_0, 10, 1, false);
        assert_ok::<U4F28>(&frac_4_28.max, 10, !0, false);
        assert_ok::<U4F28>(&frac_4_28.over, 10, 0, true);

        let frac_16_16 = make_fraction_strings(&max_int_16, eps_16);
        assert_ok::<U16F16>(&frac_16_16.zero, 10, 0, false);
        assert_ok::<U16F16>(&frac_16_16.gt_0, 10, 1, false);
        assert_ok::<U16F16>(&frac_16_16.max, 10, !0, false);
        assert_ok::<U16F16>(&frac_16_16.over, 10, 0, true);

        let frac_28_4 = make_fraction_strings(&max_int_28, eps_4);
        assert_ok::<U28F4>(&frac_28_4.zero, 10, 0, false);
        assert_ok::<U28F4>(&frac_28_4.gt_0, 10, 1, false);
        assert_ok::<U28F4>(&frac_28_4.max, 10, !0, false);
        assert_ok::<U28F4>(&frac_28_4.over, 10, 0, true);

        let frac_32_0 = make_fraction_strings(&max_int_32, eps_0);
        assert_ok::<U32F0>(&frac_32_0.zero, 10, 0, false);
        assert_ok::<U32F0>(&frac_32_0.gt_0, 10, 1, false);
        assert_ok::<U32F0>(&frac_32_0.max, 10, !0, false);
        assert_ok::<U32F0>(&frac_32_0.over, 10, 0, true);

        let frac_0_128 = make_fraction_strings(&max_int_0, eps_128);
        assert_ok::<U0F128>(&frac_0_128.zero, 10, 0, false);
        assert_ok::<U0F128>(&frac_0_128.gt_0, 10, 1, false);
        assert_ok::<U0F128>(&frac_0_128.max, 10, !0, false);
        assert_ok::<U0F128>(&frac_0_128.over, 10, 0, true);

        let frac_4_124 = make_fraction_strings(&max_int_4, eps_124);
        assert_ok::<U4F124>(&frac_4_124.zero, 10, 0, false);
        assert_ok::<U4F124>(&frac_4_124.gt_0, 10, 1, false);
        assert_ok::<U4F124>(&frac_4_124.max, 10, !0, false);
        assert_ok::<U4F124>(&frac_4_124.over, 10, 0, true);

        let frac_64_64 = make_fraction_strings(&max_int_64, eps_64);
        assert_ok::<U64F64>(&frac_64_64.zero, 10, 0, false);
        assert_ok::<U64F64>(&frac_64_64.gt_0, 10, 1, false);
        assert_ok::<U64F64>(&frac_64_64.max, 10, !0, false);
        assert_ok::<U64F64>(&frac_64_64.over, 10, 0, true);

        let frac_124_4 = make_fraction_strings(&max_int_124, eps_4);
        assert_ok::<U124F4>(&frac_124_4.zero, 10, 0, false);
        assert_ok::<U124F4>(&frac_124_4.gt_0, 10, 1, false);
        assert_ok::<U124F4>(&frac_124_4.max, 10, !0, false);
        assert_ok::<U124F4>(&frac_124_4.over, 10, 0, true);

        let frac_128_0 = make_fraction_strings(&max_int_128, eps_0);
        assert_ok::<U128F0>(&frac_128_0.zero, 10, 0, false);
        assert_ok::<U128F0>(&frac_128_0.gt_0, 10, 1, false);
        assert_ok::<U128F0>(&frac_128_0.max, 10, !0, false);
        assert_ok::<U128F0>(&frac_128_0.over, 10, 0, true);

        // some other cases
        // 13/32 = 6.5/16, to even 6/16
        assert_ok::<U4F4>(
            "0.40624999999999999999999999999999999999999999999999",
            10,
            0x06,
            false,
        );
        assert_ok::<U4F4>("0.40625", 10, 0x06, false);
        assert_ok::<U4F4>(
            "0.40625000000000000000000000000000000000000000000001",
            10,
            0x07,
            false,
        );
        // 14/32 = 7/16
        assert_ok::<U4F4>("0.4375", 10, 0x07, false);
        // 15/32 = 7.5/16, to even 8/16
        assert_ok::<U4F4>(
            "0.46874999999999999999999999999999999999999999999999",
            10,
            0x07,
            false,
        );
        assert_ok::<U4F4>("0.46875", 10, 0x08, false);
        assert_ok::<U4F4>(
            "0.46875000000000000000000000000000000000000000000001",
            10,
            0x08,
            false,
        );
        // 16/32 = 8/16
        assert_ok::<U4F4>("0.5", 10, 0x08, false);
        // 17/32 = 8.5/16, to even 8/16
        assert_ok::<U4F4>(
            "0.53124999999999999999999999999999999999999999999999",
            10,
            0x08,
            false,
        );
        assert_ok::<U4F4>("0.53125", 10, 0x08, false);
        assert_ok::<U4F4>(
            "0.53125000000000000000000000000000000000000000000001",
            10,
            0x09,
            false,
        );
        // 18/32 = 9/16
        assert_ok::<U4F4>("0.5625", 10, 0x09, false);
    }

    #[test]
    fn frac4() {
        for u in 0..=255u8 {
            let (ifix, ufix) = (I4F4::from_bits(u as i8), U4F4::from_bits(u));
            let (ifix_str, ufix_str) = (ifix.to_string(), ufix.to_string());
            assert_eq!(I4F4::from_str(&ifix_str).unwrap(), ifix);
            assert_eq!(U4F4::from_str(&ufix_str).unwrap(), ufix);
        }
    }

    fn similar<F: Fixed, G: ToFixed>(a: F, b: F, max_diff: G) -> bool {
        let abs_diff = if a > b { a - b } else { b - a };
        abs_diff <= max_diff.to_fixed::<F>()
    }

    #[test]
    fn frac17() {
        for u in 0..(1 << 17) {
            let fix = U15F17::from_bits(u) + U15F17::from_num(99);
            let fix_pos = I15F17::from_num(fix);
            let fix_neg = -fix_pos;
            let fix_str = fix.to_string();
            let fix_pos_str = fix_pos.to_string();
            let fix_neg_str = fix_neg.to_string();
            assert_eq!(fix_str, fix_pos_str);
            if u != 0 {
                assert_eq!(&fix_neg_str[..1], "-");
                assert_eq!(&fix_neg_str[1..], fix_pos_str);
            }
            assert_eq!(U15F17::from_str(&fix_str).unwrap(), fix);
            assert_eq!(I15F17::from_str(&fix_pos_str).unwrap(), fix_pos);
            assert_eq!(I15F17::from_str(&fix_neg_str).unwrap(), fix_neg);

            let fix_str3 = format!("{:.3}", fix);
            let fix_pos_str3 = format!("{:.3}", fix_pos);
            let fix_neg_str3 = format!("{:.3}", fix_neg);
            assert_eq!(fix_str3, fix_pos_str3);
            if u != 0 {
                assert_eq!(&fix_neg_str3[..1], "-");
                assert_eq!(&fix_neg_str3[1..], fix_pos_str3);
            }
            let max_diff = U15F17::from_bits((5 << 17) / 10000 + 1);
            let from_fix_str3 = U15F17::from_str(&fix_str3).unwrap();
            assert!(similar(from_fix_str3, fix, max_diff));
            let from_fix_pos_str3 = I15F17::from_str(&fix_pos_str3).unwrap();
            assert!(similar(from_fix_pos_str3, fix_pos, max_diff));
            let from_fix_neg_str3 = I15F17::from_str(&fix_neg_str3).unwrap();
            assert!(similar(from_fix_neg_str3, fix_neg, max_diff));

            let fix_str9 = format!("{:.9}", fix);
            let fix_pos_str9 = format!("{:.9}", fix_pos);
            let fix_neg_str9 = format!("{:.9}", fix_neg);
            assert_eq!(fix_str9, fix_pos_str9);
            if u != 0 {
                assert_eq!(&fix_neg_str9[..1], "-");
                assert_eq!(&fix_neg_str9[1..], fix_pos_str9);
            }
            assert_eq!(U15F17::from_str(&fix_str9).unwrap(), fix);
            assert_eq!(I15F17::from_str(&fix_pos_str9).unwrap(), fix_pos);
            assert_eq!(I15F17::from_str(&fix_neg_str9).unwrap(), fix_neg);
        }
    }
}
