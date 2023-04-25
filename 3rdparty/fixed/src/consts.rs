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
Mathematical constants.

The constants have the maximum precision possible for a fixed-point
number, and are rounded down at that precision.

# Examples

```rust
use fixed::{consts, types::I4F28};
let tau = I4F28::from_num(consts::TAU);
println!("τ = 2π with eight binary places is {:.8b}", tau);
assert_eq!(format!("{:.8b}", tau), "110.01001000");
println!("τ = 2π with eight decimal places is {:.8}", tau);
assert_eq!(format!("{:.8}", tau), "6.28318531");
```
*/

use crate::types::{U0F128, U1F127, U2F126, U3F125};

/*
```rust
use core::{cmp::Ord, convert::TryFrom};
use rug::{
    float::{Constant, Round},
    Assign, Float, Integer,
};

fn decimal_string(val: &Float, prec: i32) -> String {
    let log10 = val.clone().log10();
    let floor_log10 = log10.to_i32_saturating_round(Round::Down).unwrap();
    let shift = u32::try_from(prec - 1 - floor_log10).unwrap();
    let val = val.clone() * Integer::from(Integer::u_pow_u(10, shift));
    let int = val.to_integer_round(Round::Down).unwrap().0;
    let padding = "0".repeat(usize::try_from(-floor_log10.min(0)).unwrap());
    let mut s = format!("{}{}", padding, int);
    s.insert(1, '.');
    s
}

fn hex_bits(val: &Float, frac_bits: i32) -> String {
    let val = val.clone() << frac_bits;
    let int = val.to_integer_round(Round::Down).unwrap().0;
    let mut s = format!("0x{:016X}", int);
    for i in 0..7 {
        s.insert(6 + 5 * i, '_');
    }
    s
}

fn print(doc: &str, name: &str, val: Float) {
    println!("/// {} = {}…", doc, decimal_string(&val, 6));
    println!("// {} = {}...", name, decimal_string(&val, 40));
    let int_bits = val.get_exp().unwrap().max(0);
    let frac_bits = 128 - int_bits;
    print!("pub const {}: U{}F{} = U{1}F{2}", name, int_bits, frac_bits,);
    println!("::from_bits({});", hex_bits(&val, frac_bits));
    println!();
}

fn float<T>(t: T) -> Float
where
    Float: Assign<T>,
{
    Float::with_val(1000, t)
}

fn main() {
    print("A turn, τ", "TAU", float(Constant::Pi) * 2);
    print("τ/2", "FRAC_TAU_2", float(Constant::Pi));
    print("τ/3", "FRAC_TAU_3", float(Constant::Pi) * 2 / 3);
    print("τ/4", "FRAC_TAU_4", float(Constant::Pi) / 2);
    print("τ/6", "FRAC_TAU_6", float(Constant::Pi) / 3);
    print("τ/8", "FRAC_TAU_8", float(Constant::Pi) / 4);
    print("τ/12", "FRAC_TAU_12", float(Constant::Pi) / 6);
    print("1/τ", "FRAC_1_TAU", 0.5 / float(Constant::Pi));
    print("2/τ", "FRAC_2_TAU", 1 / float(Constant::Pi));
    print("4/τ", "FRAC_4_TAU", 2 / float(Constant::Pi));
    print("Archimedes’ constant, π", "PI", float(Constant::Pi));
    print("π/2", "FRAC_PI_2", float(Constant::Pi) / 2);
    print("π/3", "FRAC_PI_3", float(Constant::Pi) / 3);
    print("π/4", "FRAC_PI_4", float(Constant::Pi) / 4);
    print("π/6", "FRAC_PI_6", float(Constant::Pi) / 6);
    print("π/8", "FRAC_PI_8", float(Constant::Pi) / 8);
    print("1/π", "FRAC_1_PI", 1 / float(Constant::Pi));
    print("2/π", "FRAC_2_PI", 2 / float(Constant::Pi));
    print("√π", "SQRT_PI", float(Constant::Pi).sqrt());
    print("1/√π", "FRAC_1_SQRT_PI", 1 / float(Constant::Pi).sqrt());
    print("2/√π", "FRAC_2_SQRT_PI", 2 / float(Constant::Pi).sqrt());
    print("√2", "SQRT_2", float(2).sqrt());
    print("1/√2", "FRAC_1_SQRT_2", float(0.5).sqrt());
    print("√3", "SQRT_3", float(3).sqrt());
    print("1/√3", "FRAC_1_SQRT_3", float(3).recip().sqrt());
    print("Euler’s number, e", "E", float(1).exp());
    print("√e", "SQRT_E", float(0.5).exp());
    print("log<sub>2</sub> 10", "LOG2_10", float(10).log2());
    print("log<sub>2</sub> e", "LOG2_E", float(1).exp().log2());
    print("log<sub>10</sub> 2", "LOG10_2", float(2).log10());
    print("log<sub>10</sub> e", "LOG10_E", float(1).exp().log10());
    print("ln 2", "LN_2", float(2).ln());
    print("ln 10", "LN_10", float(10).ln());
    print("The golden ratio, φ", "PHI", float(1.25).sqrt() + 0.5);
    print("The golden ratio conjugate, Φ = 1/φ", "FRAC_1_PHI", float(1.25).sqrt() - 0.5);
    print("√φ", "SQRT_PHI", (float(1.25).sqrt() + 0.5f32).sqrt());
    print("The Euler-Mascheroni constant, γ", "GAMMA", float(Constant::Euler));
    print("Catalan’s constant", "CATALAN", float(Constant::Catalan));
}
```
*/

/// A turn, τ = 6.28318…
// TAU = 6.283185307179586476925286766559005768394...
pub const TAU: U3F125 = U3F125::from_bits(0xC90F_DAA2_2168_C234_C4C6_628B_80DC_1CD1);

/// τ/2 = 3.14159…
// FRAC_TAU_2 = 3.141592653589793238462643383279502884197...
pub const FRAC_TAU_2: U2F126 = U2F126::from_bits(0xC90F_DAA2_2168_C234_C4C6_628B_80DC_1CD1);

/// τ/3 = 2.09439…
// FRAC_TAU_3 = 2.094395102393195492308428922186335256131...
pub const FRAC_TAU_3: U2F126 = U2F126::from_bits(0x860A_91C1_6B9B_2C23_2DD9_9707_AB3D_688B);

/// τ/4 = 1.57079…
// FRAC_TAU_4 = 1.570796326794896619231321691639751442098...
pub const FRAC_TAU_4: U1F127 = U1F127::from_bits(0xC90F_DAA2_2168_C234_C4C6_628B_80DC_1CD1);

/// τ/6 = 1.04719…
// FRAC_TAU_6 = 1.047197551196597746154214461093167628065...
pub const FRAC_TAU_6: U1F127 = U1F127::from_bits(0x860A_91C1_6B9B_2C23_2DD9_9707_AB3D_688B);

/// τ/8 = 0.785398…
// FRAC_TAU_8 = 0.7853981633974483096156608458198757210492...
pub const FRAC_TAU_8: U0F128 = U0F128::from_bits(0xC90F_DAA2_2168_C234_C4C6_628B_80DC_1CD1);

/// τ/12 = 0.523598…
// FRAC_TAU_12 = 0.5235987755982988730771072305465838140328...
pub const FRAC_TAU_12: U0F128 = U0F128::from_bits(0x860A_91C1_6B9B_2C23_2DD9_9707_AB3D_688B);

/// 1/τ = 0.159154…
// FRAC_1_TAU = 0.1591549430918953357688837633725143620344...
pub const FRAC_1_TAU: U0F128 = U0F128::from_bits(0x28BE_60DB_9391_054A_7F09_D5F4_7D4D_3770);

/// 2/τ = 0.318309…
// FRAC_2_TAU = 0.3183098861837906715377675267450287240689...
pub const FRAC_2_TAU: U0F128 = U0F128::from_bits(0x517C_C1B7_2722_0A94_FE13_ABE8_FA9A_6EE0);

/// 4/τ = 0.636619…
// FRAC_4_TAU = 0.6366197723675813430755350534900574481378...
pub const FRAC_4_TAU: U0F128 = U0F128::from_bits(0xA2F9_836E_4E44_1529_FC27_57D1_F534_DDC0);

/// Archimedes’ constant, π = 3.14159…
// PI = 3.141592653589793238462643383279502884197...
pub const PI: U2F126 = U2F126::from_bits(0xC90F_DAA2_2168_C234_C4C6_628B_80DC_1CD1);

/// π/2 = 1.57079…
// FRAC_PI_2 = 1.570796326794896619231321691639751442098...
pub const FRAC_PI_2: U1F127 = U1F127::from_bits(0xC90F_DAA2_2168_C234_C4C6_628B_80DC_1CD1);

/// π/3 = 1.04719…
// FRAC_PI_3 = 1.047197551196597746154214461093167628065...
pub const FRAC_PI_3: U1F127 = U1F127::from_bits(0x860A_91C1_6B9B_2C23_2DD9_9707_AB3D_688B);

/// π/4 = 0.785398…
// FRAC_PI_4 = 0.7853981633974483096156608458198757210492...
pub const FRAC_PI_4: U0F128 = U0F128::from_bits(0xC90F_DAA2_2168_C234_C4C6_628B_80DC_1CD1);

/// π/6 = 0.523598…
// FRAC_PI_6 = 0.5235987755982988730771072305465838140328...
pub const FRAC_PI_6: U0F128 = U0F128::from_bits(0x860A_91C1_6B9B_2C23_2DD9_9707_AB3D_688B);

/// π/8 = 0.392699…
// FRAC_PI_8 = 0.3926990816987241548078304229099378605246...
pub const FRAC_PI_8: U0F128 = U0F128::from_bits(0x6487_ED51_10B4_611A_6263_3145_C06E_0E68);

/// 1/π = 0.318309…
// FRAC_1_PI = 0.3183098861837906715377675267450287240689...
pub const FRAC_1_PI: U0F128 = U0F128::from_bits(0x517C_C1B7_2722_0A94_FE13_ABE8_FA9A_6EE0);

/// 2/π = 0.636619…
// FRAC_2_PI = 0.6366197723675813430755350534900574481378...
pub const FRAC_2_PI: U0F128 = U0F128::from_bits(0xA2F9_836E_4E44_1529_FC27_57D1_F534_DDC0);

/// √π = 1.77245…
// SQRT_PI = 1.772453850905516027298167483341145182797...
pub const SQRT_PI: U1F127 = U1F127::from_bits(0xE2DF_C48D_A77B_553C_E1D8_2906_AEDC_9C1F);

/// 1/√π = 0.564189…
// FRAC_1_SQRT_PI = 0.5641895835477562869480794515607725858440...
pub const FRAC_1_SQRT_PI: U0F128 = U0F128::from_bits(0x906E_BA82_14DB_688D_71D4_8A7F_6BFE_C344);

/// 2/√π = 1.12837…
// FRAC_2_SQRT_PI = 1.128379167095512573896158903121545171688...
pub const FRAC_2_SQRT_PI: U1F127 = U1F127::from_bits(0x906E_BA82_14DB_688D_71D4_8A7F_6BFE_C344);

/// √2 = 1.41421…
// SQRT_2 = 1.414213562373095048801688724209698078569...
pub const SQRT_2: U1F127 = U1F127::from_bits(0xB504_F333_F9DE_6484_597D_89B3_754A_BE9F);

/// 1/√2 = 0.707106…
// FRAC_1_SQRT_2 = 0.7071067811865475244008443621048490392848...
pub const FRAC_1_SQRT_2: U0F128 = U0F128::from_bits(0xB504_F333_F9DE_6484_597D_89B3_754A_BE9F);

/// √3 = 1.73205…
// SQRT_3 = 1.732050807568877293527446341505872366942...
pub const SQRT_3: U1F127 = U1F127::from_bits(0xDDB3_D742_C265_539D_92BA_16B8_3C5C_1DC4);

/// 1/√3 = 0.577350…
// FRAC_1_SQRT_3 = 0.5773502691896257645091487805019574556476...
pub const FRAC_1_SQRT_3: U0F128 = U0F128::from_bits(0x93CD_3A2C_8198_E269_0C7C_0F25_7D92_BE83);

/// Euler’s number, e = 2.71828…
// E = 2.718281828459045235360287471352662497757...
pub const E: U2F126 = U2F126::from_bits(0xADF8_5458_A2BB_4A9A_AFDC_5620_273D_3CF1);

/// √e = 1.64872…
// SQRT_E = 1.648721270700128146848650787814163571653...
pub const SQRT_E: U1F127 = U1F127::from_bits(0xD309_4C70_F034_DE4B_96FF_7D5B_6F99_FCD8);

/// log<sub>2</sub> 10 = 3.32192…
// LOG2_10 = 3.321928094887362347870319429489390175864...
pub const LOG2_10: U2F126 = U2F126::from_bits(0xD49A_784B_CD1B_8AFE_492B_F6FF_4DAF_DB4C);

/// log<sub>2</sub> e = 1.44269…
// LOG2_E = 1.442695040888963407359924681001892137426...
pub const LOG2_E: U1F127 = U1F127::from_bits(0xB8AA_3B29_5C17_F0BB_BE87_FED0_691D_3E88);

/// log<sub>10</sub> 2 = 0.301029…
// LOG10_2 = 0.3010299956639811952137388947244930267681...
pub const LOG10_2: U0F128 = U0F128::from_bits(0x4D10_4D42_7DE7_FBCC_47C4_ACD6_05BE_48BC);

/// log<sub>10</sub> e = 0.434294…
// LOG10_E = 0.4342944819032518276511289189166050822943...
pub const LOG10_E: U0F128 = U0F128::from_bits(0x6F2D_EC54_9B94_38CA_9AAD_D557_D699_EE19);

/// ln 2 = 0.693147…
// LN_2 = 0.6931471805599453094172321214581765680755...
pub const LN_2: U0F128 = U0F128::from_bits(0xB172_17F7_D1CF_79AB_C9E3_B398_03F2_F6AF);

/// ln 10 = 2.30258…
// LN_10 = 2.302585092994045684017991454684364207601...
pub const LN_10: U2F126 = U2F126::from_bits(0x935D_8DDD_AAA8_AC16_EA56_D62B_82D3_0A28);

/// The golden ratio, φ = 1.61803…
// PHI = 1.618033988749894848204586834365638117720...
pub const PHI: U1F127 = U1F127::from_bits(0xCF1B_BCDC_BFA5_3E0A_F9CE_6030_2E76_E41A);

/// The golden ratio conjugate, Φ = 1/φ = 0.618033…
// FRAC_1_PHI = 0.6180339887498948482045868343656381177203...
pub const FRAC_1_PHI: U0F128 = U0F128::from_bits(0x9E37_79B9_7F4A_7C15_F39C_C060_5CED_C834);

/// √φ = 1.27201…
// SQRT_PHI = 1.272019649514068964252422461737491491715...
pub const SQRT_PHI: U1F127 = U1F127::from_bits(0xA2D1_8A35_4422_AF49_2AA2_8089_0F62_6C86);

/// The Euler-Mascheroni constant, γ = 0.577215…
// GAMMA = 0.5772156649015328606065120900824024310421...
pub const GAMMA: U0F128 = U0F128::from_bits(0x93C4_67E3_7DB0_C7A4_D1BE_3F81_0152_CB56);

/// Catalan’s constant = 0.915965…
// CATALAN = 0.9159655941772190150546035149323841107741...
pub const CATALAN: U0F128 = U0F128::from_bits(0xEA7C_B89F_409A_E845_2158_22E3_7D32_D0C6);

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use crate::{
        consts::*,
        traits::{Fixed, FromFixed},
    };
    use core::{convert::TryFrom, f32, f64};

    #[test]
    fn cmp_f16() {
        use half::{self, f16};
        assert_eq!(f16::from_fixed(TAU), f16::from_f32(f32::consts::PI * 2.0));
        assert_eq!(f16::from_fixed(FRAC_TAU_2), f16::PI);
        assert_eq!(
            f16::from_fixed(FRAC_TAU_3),
            f16::from_f32(f32::consts::FRAC_PI_3 * 2.0)
        );
        assert_eq!(f16::from_fixed(FRAC_TAU_4), f16::FRAC_PI_2);
        assert_eq!(f16::from_fixed(FRAC_TAU_6), f16::FRAC_PI_3);
        assert_eq!(f16::from_fixed(FRAC_TAU_8), f16::FRAC_PI_4);
        assert_eq!(f16::from_fixed(FRAC_TAU_12), f16::FRAC_PI_6);
        assert_eq!(
            f16::from_fixed(FRAC_1_TAU),
            f16::from_f32(f32::consts::FRAC_1_PI * 0.5)
        );
        assert_eq!(f16::from_fixed(FRAC_2_TAU), f16::FRAC_1_PI);
        assert_eq!(f16::from_fixed(FRAC_4_TAU), f16::FRAC_2_PI);
        assert_eq!(f16::from_fixed(PI), f16::PI);
        assert_eq!(f16::from_fixed(FRAC_PI_2), f16::FRAC_PI_2);
        assert_eq!(f16::from_fixed(FRAC_PI_3), f16::FRAC_PI_3);
        assert_eq!(f16::from_fixed(FRAC_PI_4), f16::FRAC_PI_4);
        assert_eq!(f16::from_fixed(FRAC_PI_6), f16::FRAC_PI_6);
        assert_eq!(f16::from_fixed(FRAC_PI_8), f16::FRAC_PI_8);
        assert_eq!(f16::from_fixed(FRAC_1_PI), f16::FRAC_1_PI);
        assert_eq!(f16::from_fixed(FRAC_2_PI), f16::FRAC_2_PI);
        assert_eq!(
            f16::from_fixed(SQRT_PI),
            f16::from_f32(f32::consts::PI.sqrt())
        );
        assert_eq!(
            f16::from_fixed(FRAC_1_SQRT_PI),
            f16::from_f32(f32::consts::FRAC_2_SQRT_PI / 2.0)
        );
        assert_eq!(f16::from_fixed(FRAC_2_SQRT_PI), f16::FRAC_2_SQRT_PI);
        assert_eq!(f16::from_fixed(SQRT_2), f16::SQRT_2);
        assert_eq!(f16::from_fixed(FRAC_1_SQRT_2), f16::FRAC_1_SQRT_2);
        assert_eq!(f16::from_fixed(SQRT_3), f16::from_f32(3f32.sqrt()));
        assert_eq!(
            f16::from_fixed(FRAC_1_SQRT_3),
            f16::from_f32(3f32.powf(-0.5))
        );
        assert_eq!(f16::from_fixed(E), f16::E);
        assert_eq!(f16::from_fixed(SQRT_E), f16::from_f32(0.5f32.exp()));
        assert_eq!(f16::from_fixed(LOG2_10), f16::LOG2_10);
        assert_eq!(f16::from_fixed(LOG2_E), f16::LOG2_E);
        assert_eq!(f16::from_fixed(LOG10_2), f16::LOG10_2);
        assert_eq!(f16::from_fixed(LOG10_E), f16::LOG10_E);
        assert_eq!(f16::from_fixed(LN_2), f16::LN_2);
        assert_eq!(f16::from_fixed(LN_10), f16::LN_10);
        assert_eq!(f16::from_fixed(PHI), f16::from_f32(1.25f32.sqrt() + 0.5));
        assert_eq!(
            f16::from_fixed(FRAC_1_PHI),
            f16::from_f32(1.25f32.sqrt() - 0.5)
        );
        assert_eq!(
            f16::from_fixed(SQRT_PHI),
            f16::from_f32((1.25f32.sqrt() + 0.5f32).sqrt())
        );
    }

    #[test]
    fn cmp_bf16() {
        use half::{self, bf16};
        assert_eq!(bf16::from_fixed(TAU), bf16::from_f32(f32::consts::PI * 2.0));
        assert_eq!(bf16::from_fixed(FRAC_TAU_2), bf16::PI);
        assert_eq!(
            bf16::from_fixed(FRAC_TAU_3),
            bf16::from_f32(f32::consts::FRAC_PI_3 * 2.0)
        );
        assert_eq!(bf16::from_fixed(FRAC_TAU_4), bf16::FRAC_PI_2);
        assert_eq!(bf16::from_fixed(FRAC_TAU_6), bf16::FRAC_PI_3);
        assert_eq!(bf16::from_fixed(FRAC_TAU_8), bf16::FRAC_PI_4);
        assert_eq!(bf16::from_fixed(FRAC_TAU_12), bf16::FRAC_PI_6);
        assert_eq!(
            bf16::from_fixed(FRAC_1_TAU),
            bf16::from_f32(f32::consts::FRAC_1_PI * 0.5)
        );
        assert_eq!(bf16::from_fixed(FRAC_2_TAU), bf16::FRAC_1_PI);
        assert_eq!(bf16::from_fixed(FRAC_4_TAU), bf16::FRAC_2_PI);
        assert_eq!(bf16::from_fixed(PI), bf16::PI);
        assert_eq!(bf16::from_fixed(FRAC_PI_2), bf16::FRAC_PI_2);
        assert_eq!(bf16::from_fixed(FRAC_PI_3), bf16::FRAC_PI_3);
        assert_eq!(bf16::from_fixed(FRAC_PI_4), bf16::FRAC_PI_4);
        assert_eq!(bf16::from_fixed(FRAC_PI_6), bf16::FRAC_PI_6);
        assert_eq!(bf16::from_fixed(FRAC_PI_8), bf16::FRAC_PI_8);
        assert_eq!(bf16::from_fixed(FRAC_1_PI), bf16::FRAC_1_PI);
        assert_eq!(bf16::from_fixed(FRAC_2_PI), bf16::FRAC_2_PI);
        assert_eq!(
            bf16::from_fixed(SQRT_PI),
            bf16::from_f32(f32::consts::PI.sqrt())
        );
        assert_eq!(
            bf16::from_fixed(FRAC_1_SQRT_PI),
            bf16::from_f32(f32::consts::FRAC_2_SQRT_PI / 2.0)
        );
        assert_eq!(bf16::from_fixed(FRAC_2_SQRT_PI), bf16::FRAC_2_SQRT_PI);
        assert_eq!(bf16::from_fixed(SQRT_2), bf16::SQRT_2);
        assert_eq!(bf16::from_fixed(FRAC_1_SQRT_2), bf16::FRAC_1_SQRT_2);
        assert_eq!(bf16::from_fixed(SQRT_3), bf16::from_f32(3f32.sqrt()));
        assert_eq!(
            bf16::from_fixed(FRAC_1_SQRT_3),
            bf16::from_f32(3f32.powf(-0.5))
        );
        assert_eq!(bf16::from_fixed(E), bf16::E);
        assert_eq!(bf16::from_fixed(SQRT_E), bf16::from_f32(0.5f32.exp()));
        assert_eq!(bf16::from_fixed(LOG2_10), bf16::LOG2_10);
        assert_eq!(bf16::from_fixed(LOG2_E), bf16::LOG2_E);
        assert_eq!(bf16::from_fixed(LOG10_2), bf16::LOG10_2);
        assert_eq!(bf16::from_fixed(LOG10_E), bf16::LOG10_E);
        assert_eq!(bf16::from_fixed(LN_2), bf16::LN_2);
        assert_eq!(bf16::from_fixed(LN_10), bf16::LN_10);
        assert_eq!(bf16::from_fixed(PHI), bf16::from_f32(1.25f32.sqrt() + 0.5));
        assert_eq!(
            bf16::from_fixed(FRAC_1_PHI),
            bf16::from_f32(1.25f32.sqrt() - 0.5)
        );
        assert_eq!(
            bf16::from_fixed(SQRT_PHI),
            bf16::from_f32((1.25f32.sqrt() + 0.5f32).sqrt())
        );
    }

    #[test]
    fn cmp_f32() {
        assert_eq!(f32::from_fixed(TAU), f32::consts::PI * 2.0);
        assert_eq!(f32::from_fixed(FRAC_TAU_2), f32::consts::PI);
        assert_eq!(f32::from_fixed(FRAC_TAU_3), f32::consts::FRAC_PI_3 * 2.0);
        assert_eq!(f32::from_fixed(FRAC_TAU_4), f32::consts::FRAC_PI_2);
        assert_eq!(f32::from_fixed(FRAC_TAU_6), f32::consts::FRAC_PI_3);
        assert_eq!(f32::from_fixed(FRAC_TAU_8), f32::consts::FRAC_PI_4);
        assert_eq!(f32::from_fixed(FRAC_TAU_12), f32::consts::FRAC_PI_6);
        assert_eq!(f32::from_fixed(FRAC_1_TAU), f32::consts::FRAC_1_PI * 0.5);
        assert_eq!(f32::from_fixed(FRAC_2_TAU), f32::consts::FRAC_1_PI);
        assert_eq!(f32::from_fixed(FRAC_4_TAU), f32::consts::FRAC_2_PI);
        assert_eq!(f32::from_fixed(PI), f32::consts::PI);
        assert_eq!(f32::from_fixed(FRAC_PI_2), f32::consts::FRAC_PI_2);
        assert_eq!(f32::from_fixed(FRAC_PI_3), f32::consts::FRAC_PI_3);
        assert_eq!(f32::from_fixed(FRAC_PI_4), f32::consts::FRAC_PI_4);
        assert_eq!(f32::from_fixed(FRAC_PI_6), f32::consts::FRAC_PI_6);
        assert_eq!(f32::from_fixed(FRAC_PI_8), f32::consts::FRAC_PI_8);
        assert_eq!(f32::from_fixed(FRAC_1_PI), f32::consts::FRAC_1_PI);
        assert_eq!(f32::from_fixed(FRAC_2_PI), f32::consts::FRAC_2_PI);
        assert_eq!(f32::from_fixed(SQRT_PI), f64::consts::PI.sqrt() as f32);
        assert_eq!(
            f32::from_fixed(FRAC_1_SQRT_PI),
            f32::consts::FRAC_2_SQRT_PI / 2.0
        );
        assert_eq!(f32::from_fixed(FRAC_2_SQRT_PI), f32::consts::FRAC_2_SQRT_PI);
        assert_eq!(f32::from_fixed(SQRT_2), f32::consts::SQRT_2);
        assert_eq!(f32::from_fixed(FRAC_1_SQRT_2), f32::consts::FRAC_1_SQRT_2);
        assert_eq!(f32::from_fixed(SQRT_3), 3f32.sqrt());
        assert_eq!(f32::from_fixed(FRAC_1_SQRT_3), 3f32.powf(-0.5));
        assert_eq!(f32::from_fixed(E), f32::consts::E);
        assert_eq!(f32::from_fixed(SQRT_E), 0.5f32.exp());
        assert_eq!(f32::from_fixed(LOG2_10), f32::consts::LOG2_10);
        assert_eq!(f32::from_fixed(LOG2_E), f32::consts::LOG2_E);
        assert_eq!(f32::from_fixed(LOG10_2), f32::consts::LOG10_2);
        assert_eq!(f32::from_fixed(LOG10_E), f32::consts::LOG10_E);
        assert_eq!(f32::from_fixed(LN_2), f32::consts::LN_2);
        assert_eq!(f32::from_fixed(LN_10), f32::consts::LN_10);
        assert_eq!(f32::from_fixed(PHI), 1.25f32.sqrt() + 0.5);
        assert_eq!(f32::from_fixed(FRAC_1_PHI), (1.25f64.sqrt() - 0.5) as f32);
        assert_eq!(
            f32::from_fixed(SQRT_PHI),
            (1.25f64.sqrt() + 0.5f64).sqrt() as f32
        );
    }

    #[test]
    fn cmp_f64() {
        assert_eq!(f64::from_fixed(TAU), f64::consts::PI * 2.0);
        assert_eq!(f64::from_fixed(FRAC_TAU_2), f64::consts::PI);
        assert_eq!(f64::from_fixed(FRAC_TAU_3), f64::consts::FRAC_PI_3 * 2.0);
        assert_eq!(f64::from_fixed(FRAC_TAU_4), f64::consts::FRAC_PI_2);
        assert_eq!(f64::from_fixed(FRAC_TAU_6), f64::consts::FRAC_PI_3);
        assert_eq!(f64::from_fixed(FRAC_TAU_8), f64::consts::FRAC_PI_4);
        assert_eq!(f64::from_fixed(FRAC_TAU_12), f64::consts::FRAC_PI_6);
        assert_eq!(f64::from_fixed(FRAC_1_TAU), f64::consts::FRAC_1_PI * 0.5);
        assert_eq!(f64::from_fixed(FRAC_2_TAU), f64::consts::FRAC_1_PI);
        assert_eq!(f64::from_fixed(FRAC_4_TAU), f64::consts::FRAC_2_PI);
        assert_eq!(f64::from_fixed(PI), f64::consts::PI);
        assert_eq!(f64::from_fixed(FRAC_PI_2), f64::consts::FRAC_PI_2);
        assert_eq!(f64::from_fixed(FRAC_PI_3), f64::consts::FRAC_PI_3);
        assert_eq!(f64::from_fixed(FRAC_PI_4), f64::consts::FRAC_PI_4);
        assert_eq!(f64::from_fixed(FRAC_PI_6), f64::consts::FRAC_PI_6);
        assert_eq!(f64::from_fixed(FRAC_PI_8), f64::consts::FRAC_PI_8);
        assert_eq!(f64::from_fixed(FRAC_1_PI), f64::consts::FRAC_1_PI);
        assert_eq!(f64::from_fixed(FRAC_2_PI), f64::consts::FRAC_2_PI);
        // Since 1 < SQRT_PI < 2, we use EPSILON.
        assert!((f64::from_fixed(SQRT_PI) - f64::consts::PI.sqrt()).abs() <= f64::EPSILON);
        assert_eq!(
            f64::from_fixed(FRAC_1_SQRT_PI),
            f64::consts::FRAC_2_SQRT_PI / 2.0
        );
        assert_eq!(f64::from_fixed(FRAC_2_SQRT_PI), f64::consts::FRAC_2_SQRT_PI);
        assert_eq!(f64::from_fixed(SQRT_2), f64::consts::SQRT_2);
        assert_eq!(f64::from_fixed(FRAC_1_SQRT_2), f64::consts::FRAC_1_SQRT_2);
        assert_eq!(f64::from_fixed(SQRT_3), 3f64.sqrt());
        assert_eq!(f64::from_fixed(FRAC_1_SQRT_3), 3f64.powf(-0.5));
        assert_eq!(f64::from_fixed(E), f64::consts::E);
        assert_eq!(f64::from_fixed(SQRT_E), 0.5f64.exp());
        assert_eq!(f64::from_fixed(LOG2_10), f64::consts::LOG2_10);
        assert_eq!(f64::from_fixed(LOG2_E), f64::consts::LOG2_E);
        assert_eq!(f64::from_fixed(LOG10_2), f64::consts::LOG10_2);
        assert_eq!(f64::from_fixed(LOG10_E), f64::consts::LOG10_E);
        assert_eq!(f64::from_fixed(LN_2), f64::consts::LN_2);
        assert_eq!(f64::from_fixed(LN_10), f64::consts::LN_10);
        assert_eq!(f64::from_fixed(PHI), 1.25f64.sqrt() + 0.5);
        // Since 0.5 < FRAC_1_PHI < 1, we use EPSILON / 2.
        assert!((f64::from_fixed(FRAC_1_PHI) - (1.25f64.sqrt() - 0.5)).abs() <= f64::EPSILON / 2.0);
        // Since 1 < SQRT_PHI < 2, we use EPSILON.
        assert!(
            (f64::from_fixed(SQRT_PHI) - (1.25f64.sqrt() + 0.5f64).sqrt()).abs() <= f64::EPSILON
        );
    }

    fn compare_parse<F: Fixed>(f: F, s: &str)
    where
        F::Bits: TryFrom<u8>,
    {
        let sf = F::from_str(s).unwrap();
        let f_plus = f + F::from_bits(F::Bits::try_from(1).ok().unwrap());
        assert!(f <= sf && sf <= f_plus);
    }

    #[test]
    fn cmp_parse() {
        compare_parse(TAU, "6.283185307179586476925286766559005768394");
        compare_parse(FRAC_TAU_2, "3.141592653589793238462643383279502884197");
        compare_parse(FRAC_TAU_3, "2.094395102393195492308428922186335256131");
        compare_parse(FRAC_TAU_4, "1.570796326794896619231321691639751442098");
        compare_parse(FRAC_TAU_6, "1.047197551196597746154214461093167628065");
        compare_parse(FRAC_TAU_8, "0.7853981633974483096156608458198757210492");
        compare_parse(FRAC_TAU_12, "0.5235987755982988730771072305465838140328");
        compare_parse(FRAC_1_TAU, "0.1591549430918953357688837633725143620344");
        compare_parse(FRAC_2_TAU, "0.3183098861837906715377675267450287240689");
        compare_parse(FRAC_4_TAU, "0.6366197723675813430755350534900574481378");
        compare_parse(PI, "3.141592653589793238462643383279502884197");
        compare_parse(FRAC_PI_2, "1.570796326794896619231321691639751442098");
        compare_parse(FRAC_PI_3, "1.047197551196597746154214461093167628065");
        compare_parse(FRAC_PI_4, "0.7853981633974483096156608458198757210492");
        compare_parse(FRAC_PI_6, "0.5235987755982988730771072305465838140328");
        compare_parse(FRAC_PI_8, "0.3926990816987241548078304229099378605246");
        compare_parse(FRAC_1_PI, "0.3183098861837906715377675267450287240689");
        compare_parse(FRAC_2_PI, "0.6366197723675813430755350534900574481378");
        compare_parse(SQRT_PI, "1.772453850905516027298167483341145182797");
        compare_parse(FRAC_1_SQRT_PI, "0.5641895835477562869480794515607725858440");
        compare_parse(FRAC_2_SQRT_PI, "1.128379167095512573896158903121545171688");
        compare_parse(SQRT_2, "1.414213562373095048801688724209698078569");
        compare_parse(FRAC_1_SQRT_2, "0.7071067811865475244008443621048490392848");
        compare_parse(SQRT_3, "1.732050807568877293527446341505872366942");
        compare_parse(FRAC_1_SQRT_3, "0.5773502691896257645091487805019574556476");
        compare_parse(E, "2.718281828459045235360287471352662497757");
        compare_parse(SQRT_E, "1.648721270700128146848650787814163571653");
        compare_parse(LOG2_10, "3.321928094887362347870319429489390175864");
        compare_parse(LOG2_E, "1.442695040888963407359924681001892137426");
        compare_parse(LOG10_2, "0.3010299956639811952137388947244930267681");
        compare_parse(LOG10_E, "0.4342944819032518276511289189166050822943");
        compare_parse(LN_2, "0.6931471805599453094172321214581765680755");
        compare_parse(LN_10, "2.302585092994045684017991454684364207601");
        compare_parse(PHI, "1.618033988749894848204586834365638117720");
        compare_parse(FRAC_1_PHI, "0.6180339887498948482045868343656381177203");
        compare_parse(SQRT_PHI, "1.272019649514068964252422461737491491715");
        compare_parse(GAMMA, "0.5772156649015328606065120900824024310421");
        compare_parse(CATALAN, "0.9159655941772190150546035149323841107741");
    }
}
