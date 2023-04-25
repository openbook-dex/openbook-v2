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

macro_rules! fixed_round {
    ($Fixed:ident[$s_fixed:expr]($s_nbits:expr), $Signedness:tt) => {
        comment! {
            "Returns the integer part.

",
            if_signed_unsigned!(
                $Signedness,
                concat!(
                    "Note that since the numbers are stored in two’s
complement, negative numbers with non-zero fractional parts will be
rounded towards −∞, except in the case where there are no integer
bits, that is `", $s_fixed, "<U", $s_nbits, ">`, where the return value is always zero.",
                ),
                "Note that for unsigned numbers, this is equivalent to [`floor`].",
            ),
            "

# Examples

```rust
use fixed::{types::extra::U4, ", $s_fixed, "};
type Fix = ", $s_fixed, "<U4>;
// 0010.0000
let two = Fix::from_num(2);
// 0010.0100
let two_and_quarter = two + two / 8;
assert_eq!(two_and_quarter.int(), two);
",
            if_signed_else_empty_str! {
                $Signedness;
                "// 1101.0000
let three = Fix::from_num(3);
// 1101.1100
assert_eq!((-two_and_quarter).int(), -three);
",
            },
            "```
",
            if_unsigned_else_empty_str! {
                $Signedness;
                "
[`floor`]: Self::floor
"
            };
            #[inline]
            pub fn int(self) -> $Fixed<Frac> {
                Self::from_bits(self.to_bits() & Self::INT_MASK)
            }
        }

        comment! {
            "Returns the fractional part.

",
            if_signed_else_empty_str! {
                $Signedness;
                "Note that since the numbers are stored in two’s
complement, the returned fraction will be non-negative for negative
numbers, except in the case where there are no integer bits, that is
`", $s_fixed, "<U", $s_nbits, ">` where the return value is always equal to
`self`.

",
            },
            "# Examples

```rust
use fixed::{types::extra::U4, ", $s_fixed, "};
type Fix = ", $s_fixed, "<U4>;
// 0000.0100
let quarter = Fix::ONE / 4;
// 0010.0100
let two_and_quarter = quarter * 9;
assert_eq!(two_and_quarter.frac(), quarter);
",
            if_signed_else_empty_str! {
                $Signedness;
                "// 0000.1100
let three_quarters = quarter * 3;
// 1101.1100
assert_eq!((-two_and_quarter).frac(), three_quarters);
",
            },
            "```
";
            #[inline]
            pub fn frac(self) -> $Fixed<Frac> {
                Self::from_bits(self.to_bits() & Self::FRAC_MASK)
            }
        }

        comment! {
            "Rounds to the next integer towards 0.

",
            if_signed_unsigned!(
                $Signedness,
                "Note that for negative numbers, this is different
from truncating/discarding the fractional bits. This is because in
two’s-complement representations, the value of all the bits except for
the most significant bit is positive; discarding positive bits would
round towards −∞ unlike this method which rounds towards zero.",
                "Note that for unsigned numbers, this is equivalent to [`floor`].",
            ),
            "

# Examples

```rust
use fixed::{types::extra::U4, ", $s_fixed, "};
type Fix = ", $s_fixed, "<U4>;
assert_eq!(Fix::from_num(2.1).round_to_zero(), Fix::from_num(2));
assert_eq!(Fix::from_num(2.9).round_to_zero(), Fix::from_num(2));
",
            if_signed_else_empty_str! {
                $Signedness;
                "assert_eq!(Fix::from_num(-2.1).round_to_zero(), Fix::from_num(-2));
assert_eq!(Fix::from_num(-2.9).round_to_zero(), Fix::from_num(-2));
",
            },
            "```
",
            if_unsigned_else_empty_str! {
                $Signedness;
                "
[`floor`]: Self::floor
"
            };
            #[inline]
            pub fn round_to_zero(self) -> $Fixed<Frac> {
                if_signed! {
                    $Signedness;
                    if self.is_negative() && self.frac() != 0 {
                        let int = self.int();
                        let increment = Self::from_bits(Self::INT_LSB);
                        if Self::INT_NBITS == 1 {
                            // increment is -1, so subtract it
                            return int - increment;
                        }
                        return int + increment;
                    }
                }
                self.int()
            }
        }

        comment! {
            "Rounds to the next integer towards +∞.

# Panics

When debug assertions are enabled, panics if the result does not fit.
When debug assertions are not enabled, the wrapped result can be
returned, but it is not considered a breaking change if in the future
it panics; if wrapping is required use [`wrapping_ceil`] instead.

# Examples

```rust
use fixed::{types::extra::U4, ", $s_fixed, "};
type Fix = ", $s_fixed, "<U4>;
assert_eq!(Fix::from_num(2.5).ceil(), Fix::from_num(3));
",
            if_signed_else_empty_str! {
                $Signedness;
                "assert_eq!(Fix::from_num(-2.5).ceil(), Fix::from_num(-2));
",
            },
            "```

[`wrapping_ceil`]: Self::wrapping_ceil
";
            #[inline]
            pub fn ceil(self) -> $Fixed<Frac> {
                let (ceil, overflow) = self.overflowing_ceil();
                maybe_assert!(!overflow, "overflow");
                let _ = overflow;
                ceil
            }
        }

        comment! {
            "Rounds to the next integer towards −∞.

",
            if_signed_else_empty_str! {
                $Signedness;
                "# Panics

When debug assertions are enabled, panics if the result does not fit.
When debug assertions are not enabled, the wrapped result can be
returned, but it is not considered a breaking change if in the future
it panics; if wrapping is required use [`wrapping_floor`] instead.

Overflow can only occur when there are zero integer bits.

",
            },
            "# Examples

```rust
use fixed::{types::extra::U4, ", $s_fixed, "};
type Fix = ", $s_fixed, "<U4>;
assert_eq!(Fix::from_num(2.5).floor(), Fix::from_num(2));
",
            if_signed_else_empty_str! {
                $Signedness;
                "assert_eq!(Fix::from_num(-2.5).floor(), Fix::from_num(-3));
",
            },
            "```

[`wrapping_floor`]: Self::wrapping_floor
";
            #[inline]
            pub fn floor(self) -> $Fixed<Frac> {
                let (floor, overflow) = self.overflowing_floor();
                maybe_assert!(!overflow, "overflow");
                let _ = overflow;
                floor
            }
        }

        comment! {
            "Rounds to the nearest integer, with ties rounded away
from zero.

# Panics

When debug assertions are enabled, panics if the result does not fit.
When debug assertions are not enabled, the wrapped result can be
returned, but it is not considered a breaking change if in the future
it panics; if wrapping is required use [`wrapping_round`] instead.

# Examples

```rust
use fixed::{types::extra::U4, ", $s_fixed, "};
type Fix = ", $s_fixed, "<U4>;
assert_eq!(Fix::from_num(2.5).round(), Fix::from_num(3));
",
            if_signed_else_empty_str! {
                $Signedness;
                "assert_eq!(Fix::from_num(-2.5).round(), Fix::from_num(-3));
",
            },
            "```

[`wrapping_round`]: Self::wrapping_round
";
            #[inline]
            pub fn round(self) -> $Fixed<Frac> {
                let (round, overflow) = self.overflowing_round();
                maybe_assert!(!overflow, "overflow");
                let _ = overflow;
                round
            }
        }

        comment! {
            "Rounds to the nearest integer, with ties rounded to even.

# Panics

When debug assertions are enabled, panics if the result does not fit.
When debug assertions are not enabled, the wrapped result can be
returned, but it is not considered a breaking change if in the future
it panics; if wrapping is required use [`wrapping_round_ties_to_even`]
instead.

# Examples

```rust
use fixed::{types::extra::U4, ", $s_fixed, "};
type Fix = ", $s_fixed, "<U4>;
assert_eq!(Fix::from_num(2.5).round_ties_to_even(), Fix::from_num(2));
assert_eq!(Fix::from_num(3.5).round_ties_to_even(), Fix::from_num(4));
```

[`wrapping_round_ties_to_even`]: Self::wrapping_round_ties_to_even
";
            #[inline]
            pub fn round_ties_to_even(self) -> $Fixed<Frac> {
                let (round, overflow) = self.overflowing_round_ties_to_even();
                maybe_assert!(!overflow, "overflow");
                let _ = overflow;
                round
            }
        }

        comment! {
            "Checked ceil. Rounds to the next integer towards +∞,
returning [`None`] on overflow.

# Examples

```rust
use fixed::{types::extra::U4, ", $s_fixed, "};
type Fix = ", $s_fixed, "<U4>;
assert_eq!(Fix::from_num(2.5).checked_ceil(), Some(Fix::from_num(3)));
",
            if_signed_else_empty_str! {
                $Signedness;
                "assert_eq!(Fix::from_num(-2.5).checked_ceil(), Some(Fix::from_num(-2)));
",
            },
            "assert!(Fix::MAX.checked_ceil().is_none());
```
";
            #[inline]
            pub fn checked_ceil(self) -> Option<$Fixed<Frac>> {
                let (ceil, overflow) = self.overflowing_ceil();
                if overflow { None } else { Some(ceil) }
            }
        }

        comment! {
            "Checked floor. Rounds to the next integer towards −∞.",
            if_signed_unsigned!(
                $Signedness,
                "Returns [`None`] on overflow.

Overflow can only occur when there are zero integer bits.",
                "Always returns [`Some`] for unsigned values.",
            ),
            "

# Examples

```rust
use fixed::{",
            if_signed_unsigned!(
                $Signedness,
                concat!(
                    "
    types::extra::{U4, U", $s_nbits, "},
    ", $s_fixed, ",
",
                ),
                concat!("types::extra::U4, ", $s_fixed),
            ),
            "};
type Fix = ", $s_fixed, "<U4>;
assert_eq!(Fix::from_num(2.5).checked_floor(), Some(Fix::from_num(2)));
",
            if_signed_else_empty_str! {
                $Signedness;
                "assert_eq!(Fix::from_num(-2.5).checked_floor(), Some(Fix::from_num(-3)));
type AllFrac = ", $s_fixed, "<U", $s_nbits, ">;
assert!(AllFrac::MIN.checked_floor().is_none());
",
            },
            "```
";
            #[inline]
            pub fn checked_floor(self) -> Option<$Fixed<Frac>> {
                let (floor, overflow) = self.overflowing_floor();
                if overflow { None } else { Some(floor) }
            }
        }

        comment! {
            "Checked round. Rounds to the nearest integer, with ties
rounded away from zero, returning [`None`] on overflow.

# Examples

```rust
use fixed::{types::extra::U4, ", $s_fixed, "};
type Fix = ", $s_fixed, "<U4>;
assert_eq!(Fix::from_num(2.5).checked_round(), Some(Fix::from_num(3)));
",
            if_signed_else_empty_str! {
                $Signedness;
                "assert_eq!(Fix::from_num(-2.5).checked_round(), Some(Fix::from_num(-3)));
",
            },
            "assert!(Fix::MAX.checked_round().is_none());
```
";
            #[inline]
            pub fn checked_round(self) -> Option<$Fixed<Frac>> {
                let (round, overflow) = self.overflowing_round();
                if overflow { None } else { Some(round) }
            }
        }

        comment! {
            "Checked round. Rounds to the nearest integer, with ties
rounded to even, returning [`None`] on overflow.

# Examples

```rust
use fixed::{types::extra::U4, ", $s_fixed, "};
type Fix = ", $s_fixed, "<U4>;
assert_eq!(Fix::from_num(2.5).checked_round_ties_to_even(), Some(Fix::from_num(2)));
assert_eq!(Fix::from_num(3.5).checked_round_ties_to_even(), Some(Fix::from_num(4)));
assert!(Fix::MAX.checked_round_ties_to_even().is_none());
```
";
            #[inline]
            pub fn checked_round_ties_to_even(self) -> Option<$Fixed<Frac>> {
                let (round, overflow) = self.overflowing_round_ties_to_even();
                if overflow { None } else { Some(round) }
            }
        }

        comment! {
            "Saturating ceil. Rounds to the next integer towards +∞,
saturating on overflow.

# Examples

```rust
use fixed::{types::extra::U4, ", $s_fixed, "};
type Fix = ", $s_fixed, "<U4>;
assert_eq!(Fix::from_num(2.5).saturating_ceil(), Fix::from_num(3));
",
            if_signed_else_empty_str! {
                $Signedness;
                "assert_eq!(Fix::from_num(-2.5).saturating_ceil(), Fix::from_num(-2));
",
            },
            "assert_eq!(Fix::MAX.saturating_ceil(), Fix::MAX);
```
";
            #[inline]
            pub fn saturating_ceil(self) -> $Fixed<Frac> {
                let (ceil, overflow) = self.overflowing_ceil();
                if overflow { Self::MAX } else { ceil }
            }
        }

        comment! {
            "Saturating floor. Rounds to the next integer towards −∞",
            if_signed_unsigned!(
                $Signedness,
                ", saturating on overflow.

Overflow can only occur when there are zero integer bits.",
                ". Cannot overflow for unsigned values.",
            ),
            "

# Examples

```rust
use fixed::{",
            if_signed_unsigned!(
                $Signedness,
                concat!(
                    "
    types::extra::{U4, U", $s_nbits, "},
    ", $s_fixed, ",
",
                ),
                concat!("types::extra::U4, ", $s_fixed),
            ),
            "};
type Fix = ", $s_fixed, "<U4>;
assert_eq!(Fix::from_num(2.5).saturating_floor(), Fix::from_num(2));
",
            if_signed_else_empty_str! {
                $Signedness;
                "assert_eq!(Fix::from_num(-2.5).saturating_floor(), Fix::from_num(-3));
type AllFrac = ", $s_fixed, "<U", $s_nbits, ">;
assert_eq!(AllFrac::MIN.saturating_floor(), AllFrac::MIN);
",
            },
            "```
";
            #[inline]
            pub fn saturating_floor(self) -> $Fixed<Frac> {
                let (floor, overflow) = self.overflowing_floor();
                if overflow { Self::MIN } else { floor }
            }
        }

        comment! {
            "Saturating round. Rounds to the nearest integer, with
ties rounded away from zero, and saturating on overflow.

# Examples

```rust
use fixed::{types::extra::U4, ", $s_fixed, "};
type Fix = ", $s_fixed, "<U4>;
assert_eq!(Fix::from_num(2.5).saturating_round(), Fix::from_num(3));
",
            if_signed_else_empty_str! {
                $Signedness;
                "assert_eq!(Fix::from_num(-2.5).saturating_round(), Fix::from_num(-3));
",
            },
            "assert_eq!(Fix::MAX.saturating_round(), Fix::MAX);
```
";
            #[inline]
            pub fn saturating_round(self) -> $Fixed<Frac> {
                let saturated = if self.to_bits() > 0 {
                    $Fixed::MAX
                } else {
                    $Fixed::MIN
                };
                let (round, overflow) = self.overflowing_round();
                if overflow { saturated } else { round }
            }
        }

        comment! {
            "Saturating round. Rounds to the nearest integer, with
ties rounded to even, and saturating on overflow.

# Examples

```rust
use fixed::{types::extra::U4, ", $s_fixed, "};
type Fix = ", $s_fixed, "<U4>;
assert_eq!(Fix::from_num(2.5).saturating_round_ties_to_even(), Fix::from_num(2));
assert_eq!(Fix::from_num(3.5).saturating_round_ties_to_even(), Fix::from_num(4));
assert_eq!(Fix::MAX.saturating_round_ties_to_even(), Fix::MAX);
```
";
            #[inline]
            pub fn saturating_round_ties_to_even(self) -> $Fixed<Frac> {
                let saturated = if self.to_bits() > 0 {
                    $Fixed::MAX
                } else {
                    $Fixed::MIN
                };
                let (round, overflow) = self.overflowing_round_ties_to_even();
                if overflow { saturated } else { round }
            }
        }

        comment! {
            "Wrapping ceil. Rounds to the next integer towards +∞,
wrapping on overflow.

# Examples

```rust
use fixed::{types::extra::U4, ", $s_fixed, "};
type Fix = ", $s_fixed, "<U4>;
assert_eq!(Fix::from_num(2.5).wrapping_ceil(), Fix::from_num(3));
",
            if_signed_else_empty_str! {
                $Signedness;
                "assert_eq!(Fix::from_num(-2.5).wrapping_ceil(), Fix::from_num(-2));
",
            },
            "assert_eq!(Fix::MAX.wrapping_ceil(), Fix::MIN);
```
";
            #[inline]
            pub fn wrapping_ceil(self) -> $Fixed<Frac> {
                self.overflowing_ceil().0
            }
        }

        comment! {
            "Wrapping floor. Rounds to the next integer towards −∞",
            if_signed_unsigned!(
                $Signedness,
                ", wrapping on overflow.

Overflow can only occur when there are zero integer bits.",
                ". Cannot overflow for unsigned values.",
            ),
            "

# Examples

```rust
use fixed::{",
            if_signed_unsigned!(
                $Signedness,
                concat!(
                    "
    types::extra::{U4, U", $s_nbits, "},
    ", $s_fixed, ",
",
                ),
                concat!("types::extra::U4, ", $s_fixed),
            ),
            "};
type Fix = ", $s_fixed, "<U4>;
assert_eq!(Fix::from_num(2.5).wrapping_floor(), Fix::from_num(2));
",
            if_signed_else_empty_str! {
                $Signedness;
                "assert_eq!(Fix::from_num(-2.5).wrapping_floor(), Fix::from_num(-3));
type AllFrac = ", $s_fixed, "<U", $s_nbits, ">;
assert_eq!(AllFrac::MIN.wrapping_floor(), AllFrac::ZERO);
",
            },
            "```
";
            #[inline]
            pub fn wrapping_floor(self) -> $Fixed<Frac> {
                self.overflowing_floor().0
            }
        }

        comment! {
            "Wrapping round. Rounds to the next integer to the
nearest, with ties rounded away from zero, and wrapping on overflow.

# Examples

```rust
use fixed::{types::extra::U4, ", $s_fixed, "};
type Fix = ", $s_fixed, "<U4>;
assert_eq!(Fix::from_num(2.5).wrapping_round(), Fix::from_num(3));
",
            if_signed_else_empty_str! {
                $Signedness;
                "assert_eq!(Fix::from_num(-2.5).wrapping_round(), Fix::from_num(-3));
",
            },
            "assert_eq!(Fix::MAX.wrapping_round(), Fix::MIN);
```
";
            #[inline]
            pub fn wrapping_round(self) -> $Fixed<Frac> {
                self.overflowing_round().0
            }
        }

        comment! {
            "Wrapping round. Rounds to the next integer to the
nearest, with ties rounded to even, and wrapping on overflow.

# Examples

```rust
use fixed::{types::extra::U4, ", $s_fixed, "};
type Fix = ", $s_fixed, "<U4>;
assert_eq!(Fix::from_num(2.5).wrapping_round_ties_to_even(), Fix::from_num(2));
assert_eq!(Fix::from_num(3.5).wrapping_round_ties_to_even(), Fix::from_num(4));
assert_eq!(Fix::MAX.wrapping_round_ties_to_even(), Fix::MIN);
```
";
            #[inline]
            pub fn wrapping_round_ties_to_even(self) -> $Fixed<Frac> {
                self.overflowing_round_ties_to_even().0
            }
        }

        comment! {
            "Unwrapped ceil. Rounds to the next integer towards +∞,
panicking on overflow.

# Panics

Panics if the result does not fit.

# Examples

```rust
use fixed::{types::extra::U4, ", $s_fixed, "};
type Fix = ", $s_fixed, "<U4>;
assert_eq!(Fix::from_num(2.5).unwrapped_ceil(), Fix::from_num(3));
",
            if_signed_else_empty_str! {
                $Signedness;
                "assert_eq!(Fix::from_num(-2.5).unwrapped_ceil(), Fix::from_num(-2));
",
            },
            "```

The following panics because of overflow.

```should_panic
use fixed::{types::extra::U4, ", $s_fixed, "};
type Fix = ", $s_fixed, "<U4>;
let _overflow = Fix::MAX.unwrapped_ceil();
```
";
            #[inline]
            #[track_caller]
            pub fn unwrapped_ceil(self) -> $Fixed<Frac> {
                self.checked_ceil().expect("overflow")
            }
        }

        comment! {
            "Unwrapped floor. Rounds to the next integer towards −∞",
            if_signed_unsigned!(
                $Signedness,
                ", panicking on overflow.

Overflow can only occur when there are zero integer bits.

# Panics

Panics if the result does not fit.",
                ". Cannot overflow for unsigned values.",
            ),
            "

# Examples

```rust
use fixed::{types::extra::U4, ", $s_fixed, "};
type Fix = ", $s_fixed, "<U4>;
assert_eq!(Fix::from_num(2.5).unwrapped_floor(), Fix::from_num(2));
",
            if_signed_else_empty_str! {
                $Signedness;
                "assert_eq!(Fix::from_num(-2.5).unwrapped_floor(), Fix::from_num(-3));
```

The following panics because of overflow.

```should_panic
use fixed::{types::extra::U", $s_nbits, ", ", $s_fixed, "};
type AllFrac = ", $s_fixed, "<U", $s_nbits, ">;
let _overflow = AllFrac::MIN.unwrapped_floor();
",
            },
            "```
";
            #[inline]
            #[track_caller]
            pub fn unwrapped_floor(self) -> $Fixed<Frac> {
                self.checked_floor().expect("overflow")
            }
        }

        comment! {
            "Unwrapped round. Rounds to the next integer to the
nearest, with ties rounded away from zero, and panicking on overflow.

# Panics

Panics if the result does not fit.

# Examples

```rust
use fixed::{types::extra::U4, ", $s_fixed, "};
type Fix = ", $s_fixed, "<U4>;
assert_eq!(Fix::from_num(2.5).unwrapped_round(), Fix::from_num(3));
",
            if_signed_else_empty_str! {
                $Signedness;
                "assert_eq!(Fix::from_num(-2.5).unwrapped_round(), Fix::from_num(-3));
",
            },
            "```

The following panics because of overflow.

```should_panic
use fixed::{types::extra::U4, ", $s_fixed, "};
type Fix = ", $s_fixed, "<U4>;
let _overflow = Fix::MAX.unwrapped_round();
```
";
            #[inline]
            #[track_caller]
            pub fn unwrapped_round(self) -> $Fixed<Frac> {
                self.checked_round().expect("overflow")
            }
        }

        comment! {
            "Unwrapped round. Rounds to the next integer to the
nearest, with ties rounded to even, and panicking on overflow.

# Panics

Panics if the result does not fit.

# Examples

```rust
use fixed::{types::extra::U4, ", $s_fixed, "};
type Fix = ", $s_fixed, "<U4>;
assert_eq!(Fix::from_num(2.5).unwrapped_round_ties_to_even(), Fix::from_num(2));
assert_eq!(Fix::from_num(3.5).unwrapped_round_ties_to_even(), Fix::from_num(4));
```

The following panics because of overflow.

```should_panic
use fixed::{types::extra::U4, ", $s_fixed, "};
type Fix = ", $s_fixed, "<U4>;
let _overflow = Fix::MAX.unwrapped_round_ties_to_even();
```
";
            #[inline]
            #[track_caller]
            pub fn unwrapped_round_ties_to_even(self) -> $Fixed<Frac> {
                self.checked_round_ties_to_even().expect("overflow")
            }
        }

        comment! {
            "Overflowing ceil. Rounds to the next integer towards +∞.

Returns a [tuple] of the fixed-point number and a [`bool`], indicating
whether an overflow has occurred. On overflow, the wrapped value is
returned.

# Examples

```rust
use fixed::{types::extra::U4, ", $s_fixed, "};
type Fix = ", $s_fixed, "<U4>;
assert_eq!(Fix::from_num(2.5).overflowing_ceil(), (Fix::from_num(3), false));
",
            if_signed_else_empty_str! {
                $Signedness;
                "assert_eq!(Fix::from_num(-2.5).overflowing_ceil(), (Fix::from_num(-2), false));
"
            },
            "assert_eq!(Fix::MAX.overflowing_ceil(), (Fix::MIN, true));
```
";
            #[inline]
            pub fn overflowing_ceil(self) -> ($Fixed<Frac>, bool) {
                let int = self.int();
                if self.frac() == 0 {
                    return (int, false);
                }
                if Self::INT_NBITS == 0 {
                    return (int, self.to_bits() > 0);
                }
                let increment = Self::from_bits(Self::INT_LSB);
                if_signed! {
                    $Signedness;
                    if Self::INT_NBITS == 1 {
                        // increment is -1, so subtract it
                        return int.overflowing_sub(increment);
                    }
                }
                int.overflowing_add(increment)
            }
        }

        comment! {
            "Overflowing floor. Rounds to the next integer towards −∞.

Returns a [tuple] of the fixed-point number and
",
            if_signed_unsigned!(
                $Signedness,
                "a [`bool`], indicating whether an overflow has
occurred. On overflow, the wrapped value isreturned. Overflow can only
occur when there are zero integer bits.",
                "[`false`].",
            ),
            "

# Examples

```rust
use fixed::{",
            if_signed_unsigned!(
                $Signedness,
                concat!(
                    "
    types::extra::{U4, U", $s_nbits, "},
    ", $s_fixed, ",
",
                ),
                concat!("types::extra::U4, ", $s_fixed),
            ),
            "};
type Fix = ", $s_fixed, "<U4>;
assert_eq!(Fix::from_num(2.5).overflowing_floor(), (Fix::from_num(2), false));
",
            if_signed_else_empty_str! {
                $Signedness;
                "assert_eq!(Fix::from_num(-2.5).overflowing_floor(), (Fix::from_num(-3), false));
type AllFrac = ", $s_fixed, "<U", $s_nbits, ">;
assert_eq!(AllFrac::MIN.overflowing_floor(), (AllFrac::ZERO, true));
",
            },
            "```
";
            #[inline]
            pub fn overflowing_floor(self) -> ($Fixed<Frac>, bool) {
                let int = self.int();
                if_signed! {
                    $Signedness;
                    if Self::INT_NBITS == 0 {
                        return (int, self.to_bits() < 0);
                    }
                }
                (int, false)
            }
        }

        comment! {
            "Overflowing round. Rounds to the next integer to the
nearest, with ties rounded away from zero.

Returns a [tuple] of the fixed-point number and a [`bool`] indicating
whether an overflow has occurred. On overflow, the wrapped value is
returned.

# Examples

```rust
use fixed::{types::extra::U4, ", $s_fixed, "};
type Fix = ", $s_fixed, "<U4>;
assert_eq!(Fix::from_num(2.5).overflowing_round(), (Fix::from_num(3), false));
",
            if_signed_else_empty_str! {
                $Signedness;
                "assert_eq!(Fix::from_num(-2.5).overflowing_round(), (Fix::from_num(-3), false));
",
            },
            "assert_eq!(Fix::MAX.overflowing_round(), (Fix::MIN, true));
```
";
            #[inline]
            pub fn overflowing_round(self) -> ($Fixed<Frac>, bool) {
                let int = self.int();
                if (self.to_bits() & Self::FRAC_MSB) == 0 {
                    return (int, false);
                }
                let increment = Self::from_bits(Self::INT_LSB);
                if_signed! {
                    $Signedness;
                    let tie = self.frac().to_bits() == Self::FRAC_MSB;
                    if Self::INT_NBITS == 0 {
                        // if num is .100...00 = -0.5, we have overflow
                        // otherwise .100...01, 0 < x < -0.5,  no overflow
                        return (int, tie);
                    }
                    // If num is −int.100...00 = (-int) + 0.5, we simply truncate to move to −∞.
                    // If num is −int.100...01 = (-int) + 0.6, we add 1 to −int.
                    // If num is +int.100...00 = (+int) + 0.5, we add 1 to +int.
                    // If num is +int.100...01 = (+int) + 0.6, we add 1 to +int.
                    if tie && self.to_bits() < 0 {
                        return (int, false);
                    }
                    if Self::INT_NBITS == 1 {
                        // increment is -1, so subtract it
                        return int.overflowing_sub(increment);
                    }
                    int.overflowing_add(increment)
                }
                if_unsigned! {
                    $Signedness;
                    if Self::INT_NBITS == 0 {
                        return (int, true);
                    }
                    int.overflowing_add(increment)
                }
            }
        }

        comment! {
            "Overflowing round. Rounds to the next integer to the
nearest, with ties rounded to even.

Returns a [tuple] of the fixed-point number and a [`bool`] indicating
whether an overflow has occurred. On overflow, the wrapped value is
returned.

# Examples

```rust
use fixed::{types::extra::U4, ", $s_fixed, "};
type Fix = ", $s_fixed, "<U4>;
assert_eq!(Fix::from_num(2.5).overflowing_round_ties_to_even(), (Fix::from_num(2), false));
assert_eq!(Fix::from_num(3.5).overflowing_round_ties_to_even(), (Fix::from_num(4), false));
assert_eq!(Fix::MAX.overflowing_round_ties_to_even(), (Fix::MIN, true));
```
";
            #[inline]
            pub fn overflowing_round_ties_to_even(self) -> ($Fixed<Frac>, bool) {
                let int = self.int();
                if (self.to_bits() & Self::FRAC_MSB) == 0 {
                    return (int, false);
                }
                if self.frac().to_bits() == Self::FRAC_MSB && (int.to_bits() & Self::INT_LSB) == 0 {
                    return (int, false);
                }
                let increment = Self::from_bits(Self::INT_LSB);
                if_signed! {
                    $Signedness;
                    // If INT_NBITS is 0, increment is zero, and -0.5 ≤ self < 0.5,
                    // so we're fine returning 0.overflowing_add(0).
                    if Self::INT_NBITS == 1 {
                        // increment is -1, so subtract it
                        int.overflowing_sub(increment)
                    } else {
                        int.overflowing_add(increment)
                    }
                }
                if_unsigned! {
                    $Signedness;
                    if Self::INT_NBITS == 0 {
                        return (int, true);
                    }
                    int.overflowing_add(increment)
                }
            }
        }
    };
}
