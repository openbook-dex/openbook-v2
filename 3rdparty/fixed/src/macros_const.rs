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

macro_rules! shift {
    // in case of 128, split shift in two parts to avoid >> 128
    ($SRC:ident, $Fixed:ident<$Frac:ident>) => {
        $Fixed::<$Frac>::from_bits(
            (consts::$SRC.to_bits() >> (64 - $Frac::U32 / 2) >> (64 + $Frac::U32 / 2 - $Frac::U32))
                as _,
        )
    };
    ($SRC:ident, $src_frac_nbits:literal, $Fixed:ident<$Frac:ident>) => {
        $Fixed::<$Frac>::from_bits((consts::$SRC.to_bits() >> ($src_frac_nbits - $Frac::U32)) as _)
    };
}

macro_rules! fixed_const {
    (
        $Fixed:ident[$s_fixed:expr](
            $LeEqU:tt, $s_nbits:expr,
            $s_nbits_m1:expr, $s_nbits_m2:expr, $s_nbits_m3:expr, $s_nbits_m4:expr
        ),
        $LeEqU_C0:tt, $LeEqU_C1:tt, $LeEqU_C2:tt, $LeEqU_C3:tt,
        $Signedness:tt
    ) => {
        comment! {
            "This block contains constants in the range 0 < <i>x</i> < 0.5.

# Examples

```rust
use fixed::{consts, types::extra::U", $s_nbits, ", ", $s_fixed, "};
type Fix = ", $s_fixed, "<U", $s_nbits, ">;
assert_eq!(Fix::LOG10_2, Fix::from_num(consts::LOG10_2));
```
";
            impl<Frac: $LeEqU> $Fixed<Frac> {
                /// 1/τ = 0.159154…
                pub const FRAC_1_TAU: $Fixed<Frac> = shift!(FRAC_1_TAU, $Fixed<Frac>);

                /// 2/τ = 0.318309…
                pub const FRAC_2_TAU: $Fixed<Frac> = shift!(FRAC_2_TAU, $Fixed<Frac>);

                /// π/8 = 0.392699…
                pub const FRAC_PI_8: $Fixed<Frac> = shift!(FRAC_PI_8, $Fixed<Frac>);

                /// 1/π = 0.318309…
                pub const FRAC_1_PI: $Fixed<Frac> = shift!(FRAC_1_PI, $Fixed<Frac>);

                /// log<sub>10</sub> 2 = 0.301029…
                pub const LOG10_2: $Fixed<Frac> = shift!(LOG10_2, $Fixed<Frac>);

                /// log<sub>10</sub> e = 0.434294…
                pub const LOG10_E: $Fixed<Frac> = shift!(LOG10_E, $Fixed<Frac>);
            }
        }

        comment! {
            "This block contains constants in the range 0.5 ≤ <i>x</i> < 1.

",
            if_signed_else_empty_str! {
                $Signedness;
                "These constants are not representable in signed
fixed-point numbers with less than 1 integer bit.

"
            },
            "# Examples

```rust
use fixed::{consts, types::extra::U",
            if_signed_unsigned!($Signedness, $s_nbits_m1, $s_nbits),
            ", ", $s_fixed, "};
type Fix = ", $s_fixed, "<U",
            if_signed_unsigned!($Signedness, $s_nbits_m1, $s_nbits),
            ">;
assert_eq!(Fix::LN_2, Fix::from_num(consts::LN_2));
assert!(0.5 <= Fix::LN_2 && Fix::LN_2 < 1);
```
",
            if_signed_else_empty_str! {
                $Signedness;
                "
The following example fails to compile, since the maximum
representable value with ", $s_nbits, " fractional bits and 0 integer
bits is < 0.5.

```rust,compile_fail
use fixed::{consts, types::extra::U", $s_nbits, ", ", $s_fixed, "};
type Fix = ", $s_fixed, "<U", $s_nbits, ">;
let _ = Fix::LN_2;
```
"
            };
            impl<Frac: Unsigned> $Fixed<Frac>
            where
                Frac: IsLessOrEqual<$LeEqU_C0, Output = True>,
            {
                /// τ/8 = 0.785398…
                pub const FRAC_TAU_8: $Fixed<Frac> = shift!(FRAC_TAU_8, $Fixed<Frac>);

                /// τ/12 = 0.523598…
                pub const FRAC_TAU_12: $Fixed<Frac> = shift!(FRAC_TAU_12, $Fixed<Frac>);

                /// 4/τ = 0.636619…
                pub const FRAC_4_TAU: $Fixed<Frac> = shift!(FRAC_4_TAU, $Fixed<Frac>);

                /// π/4 = 0.785398…
                pub const FRAC_PI_4: $Fixed<Frac> = shift!(FRAC_PI_4, $Fixed<Frac>);

                /// π/6 = 0.523598…
                pub const FRAC_PI_6: $Fixed<Frac> = shift!(FRAC_PI_6, $Fixed<Frac>);

                /// 2/π = 0.636619…
                pub const FRAC_2_PI: $Fixed<Frac> = shift!(FRAC_2_PI, $Fixed<Frac>);

                /// 1/√π = 0.564189…
                pub const FRAC_1_SQRT_PI: $Fixed<Frac> = shift!(FRAC_1_SQRT_PI, $Fixed<Frac>);

                /// 1/√2 = 0.707106…
                pub const FRAC_1_SQRT_2: $Fixed<Frac> = shift!(FRAC_1_SQRT_2, $Fixed<Frac>);

                /// 1/√3 = 0.577350…
                pub const FRAC_1_SQRT_3: $Fixed<Frac> = shift!(FRAC_1_SQRT_3, $Fixed<Frac>);

                /// ln 2 = 0.693147…
                pub const LN_2: $Fixed<Frac> = shift!(LN_2, $Fixed<Frac>);

                /// The golden ratio conjugate, Φ = 1/φ = 0.618033…
                pub const FRAC_1_PHI: $Fixed<Frac> = shift!(FRAC_1_PHI, $Fixed<Frac>);

                /// The Euler-Mascheroni constant, γ = 0.577215…
                pub const GAMMA: $Fixed<Frac> = shift!(GAMMA, $Fixed<Frac>);

                /// Catalan’s constant = 0.915965…
                pub const CATALAN: $Fixed<Frac> = shift!(CATALAN, $Fixed<Frac>);
            }
        }

        comment! {
            "This block contains constants in the range 1 ≤ <i>x</i> < 2.

These constants are not representable in ",
            if_signed_unsigned!($Signedness, "signed", "unsigned"),
            " fixed-point numbers with less than ",
            if_signed_unsigned!($Signedness, "2 integer bits", "1 integer bit"),
            ".

# Examples

```rust
use fixed::{consts, types::extra::U",
            if_signed_unsigned!($Signedness, $s_nbits_m2, $s_nbits_m1),
            ", ", $s_fixed, "};
type Fix = ", $s_fixed, "<U",
            if_signed_unsigned!($Signedness, $s_nbits_m2, $s_nbits_m1),
            ">;
assert_eq!(Fix::LOG2_E, Fix::from_num(consts::LOG2_E));
assert!(1 <= Fix::LOG2_E && Fix::LOG2_E < 2);
```

The following example fails to compile, since the maximum
representable value with ",
            if_signed_unsigned!($Signedness, $s_nbits_m1, $s_nbits),
            " fractional bits and ",
            if_signed_unsigned!($Signedness, "1 integer bit", "0 integer bits"),
            " is < 1.

```rust,compile_fail
use fixed::{consts, types::extra::U",
            if_signed_unsigned!($Signedness, $s_nbits_m1, $s_nbits),
            ", ", $s_fixed, "};
type Fix = ", $s_fixed, "<U",
            if_signed_unsigned!($Signedness, $s_nbits_m1, $s_nbits),
            ">;
let _ = Fix::LOG2_E;
```
";
            impl<Frac: Unsigned> $Fixed<Frac>
            where
                Frac: IsLessOrEqual<$LeEqU_C1, Output = True>,
            {
                comment! {
                    "One.

# Examples

```rust
use fixed::{types::extra::U4, ", $s_fixed, "};
type Fix = ", $s_fixed, "<U4>;
assert_eq!(Fix::ONE, Fix::from_num(1));
```
";
                    pub const ONE: $Fixed<Frac> =
                        $Fixed::from_bits($Fixed::<Frac>::DELTA.to_bits() << Frac::U32);
                }

                /// τ/4 = 1.57079…
                pub const FRAC_TAU_4: $Fixed<Frac> = shift!(FRAC_TAU_4, 127, $Fixed<Frac>);

                /// τ/6 = 1.04719…
                pub const FRAC_TAU_6: $Fixed<Frac> = shift!(FRAC_TAU_6, 127, $Fixed<Frac>);

                /// π/2 = 1.57079…
                pub const FRAC_PI_2: $Fixed<Frac> = shift!(FRAC_PI_2, 127, $Fixed<Frac>);

                /// π/3 = 1.04719…
                pub const FRAC_PI_3: $Fixed<Frac> = shift!(FRAC_PI_3, 127, $Fixed<Frac>);

                /// √π = 1.77245…
                pub const SQRT_PI: $Fixed<Frac> = shift!(SQRT_PI, 127, $Fixed<Frac>);

                /// 2/√π = 1.12837…
                pub const FRAC_2_SQRT_PI: $Fixed<Frac> = shift!(FRAC_2_SQRT_PI, 127, $Fixed<Frac>);

                /// √2 = 1.41421…
                pub const SQRT_2: $Fixed<Frac> = shift!(SQRT_2, 127, $Fixed<Frac>);

                /// √3 = 1.73205…
                pub const SQRT_3: $Fixed<Frac> = shift!(SQRT_3, 127, $Fixed<Frac>);

                /// √e = 1.64872…
                pub const SQRT_E: $Fixed<Frac> = shift!(SQRT_E, 127, $Fixed<Frac>);

                /// log<sub>2</sub> e = 1.44269…
                pub const LOG2_E: $Fixed<Frac> = shift!(LOG2_E, 127, $Fixed<Frac>);

                /// The golden ratio, φ = 1.61803…
                pub const PHI: $Fixed<Frac> = shift!(PHI, 127, $Fixed<Frac>);

                /// √φ = 1.27201…
                pub const SQRT_PHI: $Fixed<Frac> = shift!(SQRT_PHI, 127, $Fixed<Frac>);
            }
        }

        comment! {
            "This block contains constants in the range 2 ≤ <i>x</i> < 4.

These constants are not representable in ",
            if_signed_unsigned!($Signedness, "signed", "unsigned"),
            " fixed-point numbers with less than ",
            if_signed_unsigned!($Signedness, "3", "2"),
            " integer bits.

# Examples

```rust
use fixed::{consts, types::extra::U",
            if_signed_unsigned!($Signedness, $s_nbits_m3, $s_nbits_m2),
            ", ", $s_fixed, "};
type Fix = ", $s_fixed, "<U",
            if_signed_unsigned!($Signedness, $s_nbits_m3, $s_nbits_m2),
            ">;
assert_eq!(Fix::E, Fix::from_num(consts::E));
assert!(2 <= Fix::E && Fix::E < 4);
```

The following example fails to compile, since the maximum
representable value with ",
            if_signed_unsigned!($Signedness, $s_nbits_m2, $s_nbits_m1),
            " fractional bits and ",
            if_signed_unsigned!($Signedness, "2 integer bits", "1 integer bit"),
            " is < 2.

```rust,compile_fail
use fixed::{consts, types::extra::U",
            if_signed_unsigned!($Signedness, $s_nbits_m2, $s_nbits_m1),
            ", ", $s_fixed, "};
type Fix = ", $s_fixed, "<U",
            if_signed_unsigned!($Signedness, $s_nbits_m2, $s_nbits_m1),
            ">;
let _ = Fix::E;
```
";
            impl<Frac: Unsigned> $Fixed<Frac>
            where
                Frac: IsLessOrEqual<$LeEqU_C2, Output = True>,
            {
                /// τ/2 = 3.14159…
                pub const FRAC_TAU_2: $Fixed<Frac> = shift!(FRAC_TAU_2, 126, $Fixed<Frac>);

                /// τ/3 = 2.09439…
                pub const FRAC_TAU_3: $Fixed<Frac> = shift!(FRAC_TAU_3, 126, $Fixed<Frac>);

                /// Archimedes’ constant, π = 3.14159…
                pub const PI: $Fixed<Frac> = shift!(PI, 126, $Fixed<Frac>);

                /// Euler’s number, e = 2.71828…
                pub const E: $Fixed<Frac> = shift!(E, 126, $Fixed<Frac>);

                /// log<sub>2</sub> 10 = 3.32192…
                pub const LOG2_10: $Fixed<Frac> = shift!(LOG2_10, 126, $Fixed<Frac>);

                /// ln 10 = 2.30258…
                pub const LN_10: $Fixed<Frac> = shift!(LN_10, 126, $Fixed<Frac>);
            }
        }

        comment! {
            "This block contains constants in the range 4 ≤ <i>x</i> < 8.

These constants are not representable in ",
            if_signed_unsigned!($Signedness, "signed", "unsigned"),
            " fixed-point numbers with less than ",
            if_signed_unsigned!($Signedness, "4", "3"),
            " integer bits.

# Examples

```rust
use fixed::{consts, types::extra::U",
            if_signed_unsigned!($Signedness, $s_nbits_m4, $s_nbits_m3),
            ", ", $s_fixed, "};
type Fix = ", $s_fixed, "<U",
            if_signed_unsigned!($Signedness, $s_nbits_m4, $s_nbits_m3),
            ">;
assert_eq!(Fix::TAU, Fix::from_num(consts::TAU));
assert!(4 <= Fix::TAU && Fix::TAU < 8);
```

The following example fails to compile, since the maximum
representable value with ",
            if_signed_unsigned!($Signedness, $s_nbits_m3, $s_nbits_m2),
            " fractional bits and ",
            if_signed_unsigned!($Signedness, "3", "2"),
            " integer bits is < 4.

```rust,compile_fail
use fixed::{consts, types::extra::U",
            if_signed_unsigned!($Signedness, $s_nbits_m3, $s_nbits_m2),
            ", ", $s_fixed, "};
type Fix = ", $s_fixed, "<U",
            if_signed_unsigned!($Signedness, $s_nbits_m3, $s_nbits_m2),
            ">;
let _ = Fix::TAU;
```
";
            impl<Frac: Unsigned> $Fixed<Frac>
            where
                Frac: IsLessOrEqual<$LeEqU_C3, Output = True>,
            {
                /// A turn, τ = 6.28318…
                pub const TAU: $Fixed<Frac> = shift!(TAU, 125, $Fixed<Frac>);
            }
        }
    };
}
