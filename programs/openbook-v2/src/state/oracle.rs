use std::mem::size_of;

use anchor_lang::prelude::*;
use anchor_lang::Discriminator;
use fixed::types::I80F48;

use num_enum::{IntoPrimitive, TryFromPrimitive};
use static_assertions::const_assert_eq;
use switchboard_program::FastRoundResultAccountData;
use switchboard_v2::AggregatorAccountData;

use crate::accounts_zerocopy::*;

use crate::error::*;

const DECIMAL_CONSTANT_ZERO_INDEX: i8 = 12;
const DECIMAL_CONSTANTS: [I80F48; 25] = [
    I80F48::from_bits((1 << 48) / 10i128.pow(12u32)),
    I80F48::from_bits((1 << 48) / 10i128.pow(11u32) + 1),
    I80F48::from_bits((1 << 48) / 10i128.pow(10u32)),
    I80F48::from_bits((1 << 48) / 10i128.pow(9u32) + 1),
    I80F48::from_bits((1 << 48) / 10i128.pow(8u32) + 1),
    I80F48::from_bits((1 << 48) / 10i128.pow(7u32) + 1),
    I80F48::from_bits((1 << 48) / 10i128.pow(6u32) + 1),
    I80F48::from_bits((1 << 48) / 10i128.pow(5u32)),
    I80F48::from_bits((1 << 48) / 10i128.pow(4u32)),
    I80F48::from_bits((1 << 48) / 10i128.pow(3u32) + 1), // 0.001
    I80F48::from_bits((1 << 48) / 10i128.pow(2u32) + 1), // 0.01
    I80F48::from_bits((1 << 48) / 10i128.pow(1u32) + 1), // 0.1
    I80F48::from_bits((1 << 48) * 10i128.pow(0u32)),     // 1, index 12
    I80F48::from_bits((1 << 48) * 10i128.pow(1u32)),     // 10
    I80F48::from_bits((1 << 48) * 10i128.pow(2u32)),     // 100
    I80F48::from_bits((1 << 48) * 10i128.pow(3u32)),     // 1000
    I80F48::from_bits((1 << 48) * 10i128.pow(4u32)),
    I80F48::from_bits((1 << 48) * 10i128.pow(5u32)),
    I80F48::from_bits((1 << 48) * 10i128.pow(6u32)),
    I80F48::from_bits((1 << 48) * 10i128.pow(7u32)),
    I80F48::from_bits((1 << 48) * 10i128.pow(8u32)),
    I80F48::from_bits((1 << 48) * 10i128.pow(9u32)),
    I80F48::from_bits((1 << 48) * 10i128.pow(10u32)),
    I80F48::from_bits((1 << 48) * 10i128.pow(11u32)),
    I80F48::from_bits((1 << 48) * 10i128.pow(12u32)),
];
pub const fn power_of_ten(decimals: i8) -> I80F48 {
    DECIMAL_CONSTANTS[(decimals + DECIMAL_CONSTANT_ZERO_INDEX) as usize]
}

pub mod switchboard_v1_devnet_oracle {
    use solana_program::declare_id;
    declare_id!("7azgmy1pFXHikv36q1zZASvFq5vFa39TT9NweVugKKTU");
}
pub mod switchboard_v2_mainnet_oracle {
    use solana_program::declare_id;
    declare_id!("DtmE9D2CSB4L5D6A15mraeEjrGMm6auWVzgaD8hK2tZM");
}

#[zero_copy]
#[derive(AnchorDeserialize, AnchorSerialize, Debug)]
pub struct OracleConfig {
    pub conf_filter: I80F48,
    pub max_staleness_slots: i64,
    pub price_relation: u8,
    pub reserved: [u8; 71],
}
const_assert_eq!(size_of::<OracleConfig>(), 16 + 8 + 1 + 71);
const_assert_eq!(size_of::<OracleConfig>(), 96);
const_assert_eq!(size_of::<OracleConfig>() % 8, 0);

#[derive(
    Eq,
    PartialEq,
    Copy,
    Clone,
    Default,
    TryFromPrimitive,
    IntoPrimitive,
    Debug,
    AnchorSerialize,
    AnchorDeserialize,
)]
#[repr(u8)]
pub enum PriceRelation {
    #[default]
    None = 0,
    Multiplication = 1,
    Division = 2,
}

#[derive(AnchorDeserialize, AnchorSerialize, Debug)]
pub struct OracleConfigParams {
    pub conf_filter: f32,
    pub max_staleness_slots: Option<u32>,
    pub price_relation: u8,
}

impl OracleConfigParams {
    pub fn to_oracle_config(&self) -> OracleConfig {
        OracleConfig {
            conf_filter: I80F48::checked_from_num(self.conf_filter).unwrap_or(I80F48::MAX),
            max_staleness_slots: self.max_staleness_slots.map(|v| v as i64).unwrap_or(-1),
            price_relation: self.price_relation,
            reserved: [0; 71],
        }
    }
}

#[derive(PartialEq)]
pub enum OracleType {
    Pyth,
    Stub,
    SwitchboardV1,
    SwitchboardV2,
}

#[account(zero_copy)]
pub struct StubOracle {
    // ABI: Clients rely on this being at offset 40
    pub mint: Pubkey,
    pub price: I80F48,
    pub last_updated: i64,
    pub reserved: [u8; 128],
}
const_assert_eq!(size_of::<StubOracle>(), 32 + 16 + 8 + 128);
const_assert_eq!(size_of::<StubOracle>(), 184);
const_assert_eq!(size_of::<StubOracle>() % 8, 0);

pub fn determine_oracle_type(acc_info: &impl KeyedAccountReader) -> Result<OracleType> {
    let data = acc_info.data();

    if u32::from_le_bytes(data[0..4].try_into().unwrap()) == pyth_sdk_solana::state::MAGIC {
        return Ok(OracleType::Pyth);
    } else if data[0..8] == StubOracle::discriminator() {
        return Ok(OracleType::Stub);
    }
    // https://github.com/switchboard-xyz/switchboard-v2/blob/main/libraries/rs/src/aggregator.rs#L114
    // note: disc is not public, hence the copy pasta
    else if data[0..8] == [217, 230, 65, 101, 201, 162, 27, 125] {
        return Ok(OracleType::SwitchboardV2);
    }
    // note: this is the only known way of checking this
    else if acc_info.owner() == &switchboard_v1_devnet_oracle::ID
        || acc_info.owner() == &switchboard_v2_mainnet_oracle::ID
    {
        return Ok(OracleType::SwitchboardV1);
    }

    Err(OpenBookError::UnknownOracleType.into())
}
pub fn oracle_price(
    _acc_info: &impl KeyedAccountReader,
    _config: &OracleConfig,
    _base_decimals: u8,
    _quote_decimals: u8,
    _staleness_slot: u64,
) -> Result<I80F48> {
    todo!()
}

/// Read the price & uncertainty of the given oracle
pub fn oracle_price_data(
    acc_info: &impl KeyedAccountReader,
    max_staleness_slots: i64,
    staleness_slot: u64,
) -> Result<(I80F48, I80F48)> {
    let data = &acc_info.data();
    let oracle_type = determine_oracle_type(acc_info)?;

    Ok(match oracle_type {
        OracleType::Stub => (acc_info.load::<StubOracle>()?.price, fixed::FixedI128::ZERO),
        OracleType::Pyth => {
            let price_account = pyth_sdk_solana::state::load_price_account(data).unwrap();
            let price_data = price_account.to_price();
            let price = I80F48::from_num(price_data.price);
            let error = I80F48::from_num(price_data.conf);

            // The last_slot is when the price was actually updated
            let last_slot = price_account.last_slot;
            if max_staleness_slots >= 0
                && price_account
                    .last_slot
                    .saturating_add(max_staleness_slots as u64)
                    < staleness_slot
            {
                msg!(
                    "Pyth price too stale; pubkey {} price: {} last slot: {}",
                    acc_info.key(),
                    price.to_num::<f64>(),
                    last_slot,
                );

                return Err(OpenBookError::OracleStale.into());
            }

            (price, error)
        }
        OracleType::SwitchboardV2 => {
            fn from_foreign_error(e: impl std::fmt::Display) -> Error {
                error_msg!("{}", e)
            }

            let feed = bytemuck::from_bytes::<AggregatorAccountData>(&data[8..]);
            let feed_result = feed.get_result().map_err(from_foreign_error)?;
            let price_decimal: f64 = feed_result.try_into().map_err(from_foreign_error)?;
            let price = I80F48::from_num(price_decimal);
            let error = I80F48::from_num(
                TryInto::<f64>::try_into(feed.latest_confirmed_round.std_deviation)
                    .map_err(from_foreign_error)?,
            );

            // The round_open_slot is an overestimate of the oracle staleness: Reporters will see
            // the round opening and only then start executing the price tasks.
            let round_open_slot = feed.latest_confirmed_round.round_open_slot;
            if max_staleness_slots >= 0
                && round_open_slot.saturating_add(max_staleness_slots as u64) < staleness_slot
            {
                msg!(
                    "Switchboard v2 price too stale; pubkey {} price: {} latest_confirmed_round.round_open_slot: {}",
                    acc_info.key(),
                    price.to_num::<f64>(),
                    round_open_slot,
                );
                return Err(OpenBookError::OracleConfidence.into());
            }

            (price, error)
        }
        OracleType::SwitchboardV1 => {
            let result = FastRoundResultAccountData::deserialize(data).unwrap();
            let price = I80F48::from_num(result.result.result);
            let min_response = I80F48::from_num(result.result.min_response);
            let max_response = I80F48::from_num(result.result.max_response);

            let round_open_slot = result.result.round_open_slot;
            if max_staleness_slots >= 0
                && round_open_slot.saturating_add(max_staleness_slots as u64) < staleness_slot
            {
                msg!(
                    "Switchboard v1 price too stale; pubkey {} price: {} round_open_slot: {}",
                    acc_info.key(),
                    price.to_num::<f64>(),
                    round_open_slot,
                );
                return Err(OpenBookError::OracleConfidence.into());
            }

            (price, max_response - min_response)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program_test::{find_file, read_file};
    use std::{cell::RefCell, path::PathBuf, str::FromStr};

    #[test]
    pub fn test_oracles() -> Result<()> {
        // add ability to find fixtures
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test");

        let fixtures = vec![
            (
                "J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix",
                OracleType::Pyth,
                Pubkey::default(),
            ),
            (
                "8k7F9Xb36oFJsjpCKpsXvg4cgBRoZtwNTc3EzG5Ttd2o",
                OracleType::SwitchboardV1,
                switchboard_v1_devnet_oracle::ID,
            ),
            (
                "GvDMxPzN1sCj7L26YDK2HnMRXEQmQ2aemov8YBtPS7vR",
                OracleType::SwitchboardV2,
                Pubkey::default(),
            ),
        ];

        for fixture in fixtures {
            let filename = format!("resources/test/{}.bin", fixture.0);
            let mut pyth_price_data = read_file(find_file(&filename).unwrap());
            let data = RefCell::new(&mut pyth_price_data[..]);
            let ai = &AccountInfoRef {
                key: &Pubkey::from_str(fixture.0).unwrap(),
                owner: &fixture.2,
                data: data.borrow(),
            };
            assert!(determine_oracle_type(ai).unwrap() == fixture.1);
        }

        Ok(())
    }

    #[test]
    pub fn lookup_test() {
        for idx in -12..0 {
            assert_eq!(
                power_of_ten(idx),
                I80F48::from_str(&format!(
                    "0.{}1",
                    str::repeat("0", (idx.unsigned_abs() as usize) - 1)
                ))
                .unwrap()
            )
        }

        assert_eq!(power_of_ten(0), I80F48::ONE);

        for idx in 1..=12 {
            assert_eq!(
                power_of_ten(idx),
                I80F48::from_str(&format!(
                    "1{}",
                    str::repeat("0", idx.unsigned_abs() as usize)
                ))
                .unwrap()
            )
        }
    }
}
