use anchor_lang::prelude::*;
use anchor_lang::Discriminator;
use fixed::types::U64F64;
use static_assertions::const_assert_eq;
use std::mem::size_of;
use switchboard_program::FastRoundResultAccountData;
use switchboard_solana::AggregatorAccountData;

use crate::accounts_zerocopy::*;
use crate::error::*;
use crate::state::raydium_internal;
use crate::state::raydium_internal::PoolState;

const DECIMAL_CONSTANT_ZERO_INDEX: i8 = 12;
const DECIMAL_CONSTANTS_F64: [f64; 25] = [
    1e-12, 1e-11, 1e-10, 1e-9, 1e-8, 1e-7, 1e-6, 1e-5, 1e-4, 1e-3, 1e-2, 1e-1, 1e0, 1e1, 1e2, 1e3,
    1e4, 1e5, 1e6, 1e7, 1e8, 1e9, 1e10, 1e11, 1e12,
];

pub const fn power_of_ten_float(decimals: i8) -> f64 {
    DECIMAL_CONSTANTS_F64[(decimals + DECIMAL_CONSTANT_ZERO_INDEX) as usize]
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
    pub conf_filter: f64,
    pub max_staleness_slots: i64,
    pub reserved: [u8; 72],
}
const_assert_eq!(size_of::<OracleConfig>(), 8 + 8 + 72);
const_assert_eq!(size_of::<OracleConfig>(), 88);
const_assert_eq!(size_of::<OracleConfig>() % 8, 0);

#[derive(AnchorDeserialize, AnchorSerialize, Debug, Clone)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct OracleConfigParams {
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub conf_filter: f32,
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub max_staleness_slots: Option<u32>,
}

impl OracleConfigParams {
    pub fn to_oracle_config(&self) -> OracleConfig {
        OracleConfig {
            conf_filter: self.conf_filter as f64,
            max_staleness_slots: self.max_staleness_slots.map(|v| v as i64).unwrap_or(-1),
            reserved: [0; 72],
        }
    }
}

#[derive(Clone, Copy, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub enum OracleType {
    Pyth,
    Stub,
    SwitchboardV1,
    SwitchboardV2,
    RaydiumCLMM,
}

pub struct OracleState {
    pub price: f64,
    pub deviation: f64,
    pub last_update_slot: u64,
    pub oracle_type: OracleType,
}

impl OracleState {
    pub fn is_stale(&self, oracle_pk: &Pubkey, config: &OracleConfig, now_slot: u64) -> bool {
        if config.max_staleness_slots >= 0
            && self
                .last_update_slot
                .saturating_add(config.max_staleness_slots as u64)
                < now_slot
        {
            msg!(
                "Oracle is stale; pubkey {}, price: {}, last_update_slot: {}, now_slot: {}",
                oracle_pk,
                self.price,
                self.last_update_slot,
                now_slot,
            );
            true
        } else {
            false
        }
    }

    pub fn has_valid_confidence(&self, oracle_pk: &Pubkey, config: &OracleConfig) -> bool {
        if self.deviation > config.conf_filter * self.price {
            msg!(
                "Oracle confidence not good enough: pubkey {}, price: {}, deviation: {}, conf_filter: {}",
                oracle_pk,
                self.price,
                self.deviation,
                config.conf_filter,
            );
            false
        } else {
            true
        }
    }

    pub fn has_valid_combined_confidence(&self, other: &Self, config: &OracleConfig) -> bool {
        // target uncertainty reads
        //   $ \sigma \approx \frac{A}{B} * \sqrt{(\sigma_A/A)^2 + (\sigma_B/B)^2} $
        // but alternatively, to avoid costly operations, we compute the square
        // Also note that the relative scaled var, i.e. without the \frac{A}{B} factor, is computed
        let relative_var =
            (self.deviation / self.price).powi(2) + (other.deviation / other.price).powi(2);

        let relative_target_var = config.conf_filter.powi(2);

        if relative_var > relative_target_var {
            msg!(
                "Combined confidence too high: computed^2: {}, conf_filter^2: {}",
                relative_var,
                relative_target_var
            );
            false
        } else {
            true
        }
    }
}

#[account(zero_copy)]
pub struct StubOracle {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub price: f64,
    pub last_update_ts: i64,
    pub last_update_slot: u64,
    pub deviation: f64,
    pub reserved: [u8; 104],
}
const_assert_eq!(size_of::<StubOracle>(), 32 + 32 + 8 + 8 + 8 + 8 + 104);
const_assert_eq!(size_of::<StubOracle>(), 200);
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
    } else if acc_info.owner() == &raydium_internal::ID {
        return Ok(OracleType::RaydiumCLMM);
    }

    Err(OpenBookError::UnknownOracleType.into())
}

/// Get the pyth agg price if it's available, otherwise take the prev price.
///
/// Returns the publish slot in addition to the price info.
///
/// Also see pyth's PriceAccount::get_price_no_older_than().
fn pyth_get_price(
    account: &pyth_sdk_solana::state::SolanaPriceAccount,
) -> (pyth_sdk_solana::Price, u64) {
    use pyth_sdk_solana::*;
    if account.agg.status == state::PriceStatus::Trading {
        (
            Price {
                conf: account.agg.conf,
                expo: account.expo,
                price: account.agg.price,
                publish_time: account.timestamp,
            },
            account.agg.pub_slot,
        )
    } else {
        (
            Price {
                conf: account.prev_conf,
                expo: account.expo,
                price: account.prev_price,
                publish_time: account.prev_timestamp,
            },
            account.prev_slot,
        )
    }
}

/// Returns the price of one native base token, in native quote tokens
///
/// Example: The for SOL at 40 USDC/SOL it would return 0.04 (the unit is USDC-native/SOL-native)
///
/// The staleness and confidence of the oracle is not checked. Use the functions on
/// OracleState to validate them if needed. That's why this function is called _unchecked.
pub fn oracle_state_unchecked(acc_info: &impl KeyedAccountReader) -> Result<OracleState> {
    let data = &acc_info.data();
    let oracle_type = determine_oracle_type(acc_info)?;

    Ok(match oracle_type {
        OracleType::Stub => {
            let stub = acc_info.load::<StubOracle>()?;
            let last_update_slot = if stub.last_update_slot == 0 {
                // ensure staleness checks will never fail
                u64::MAX
            } else {
                stub.last_update_slot
            };
            OracleState {
                price: stub.price,
                last_update_slot,
                deviation: stub.deviation,
                oracle_type: OracleType::Stub,
            }
        }
        OracleType::Pyth => {
            let price_account = pyth_sdk_solana::state::load_price_account(data).unwrap();
            let (price_data, last_update_slot) = pyth_get_price(price_account);

            let decimals = price_account.expo as i8;
            let decimal_adj = power_of_ten_float(decimals);
            let price = price_data.price as f64 * decimal_adj;
            let deviation = price_data.conf as f64 * decimal_adj;
            require_gte!(price, 0f64);
            OracleState {
                price,
                last_update_slot,
                deviation,
                oracle_type: OracleType::Pyth,
            }
        }
        OracleType::SwitchboardV2 => {
            fn from_foreign_error(e: impl std::fmt::Display) -> Error {
                error_msg!("{}", e)
            }

            let feed = bytemuck::from_bytes::<AggregatorAccountData>(&data[8..]);
            let feed_result = feed.get_result().map_err(from_foreign_error)?;
            let price: f64 = feed_result.try_into().map_err(from_foreign_error)?;
            let deviation: f64 = feed
                .latest_confirmed_round
                .std_deviation
                .try_into()
                .map_err(from_foreign_error)?;

            // The round_open_slot is an underestimate of the last update slot: Reporters will see
            // the round opening and only then start executing the price tasks.
            let last_update_slot = feed.latest_confirmed_round.round_open_slot;

            require_gte!(price, 0f64);
            OracleState {
                price,
                last_update_slot,
                deviation,
                oracle_type: OracleType::SwitchboardV2,
            }
        }
        OracleType::SwitchboardV1 => {
            let result = FastRoundResultAccountData::deserialize(data).unwrap();
            let price = result.result.result;

            let deviation = result.result.max_response - result.result.min_response;
            let last_update_slot = result.result.round_open_slot;
            require_gte!(price, 0f64);
            OracleState {
                price,
                last_update_slot,
                deviation,
                oracle_type: OracleType::SwitchboardV1,
            }
        }
        OracleType::RaydiumCLMM => {
            let pool = bytemuck::from_bytes::<PoolState>(&data[8..]);

            let sqrt_price = U64F64::from_bits(pool.sqrt_price_x64);

            let decimals: i8 = (pool.mint_decimals_0 as i8) - (pool.mint_decimals_1 as i8);
            let price: f64 =
                (sqrt_price * sqrt_price).to_num::<f64>() * power_of_ten_float(decimals);

            require_gte!(price, 0f64);
            OracleState {
                price,
                last_update_slot: u64::MAX, // ensure staleness slot will never fail
                deviation: 0f64,
                oracle_type: OracleType::RaydiumCLMM,
            }
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
            (
                "2QdhepnKRTLjjSqPL1PtKNwqrUkoLee5Gqs8bvZhRdMv",
                OracleType::RaydiumCLMM,
                raydium_internal::ID,
            ),
        ];

        for fixture in fixtures {
            let filename = format!("resources/test/{}.bin", fixture.0);
            let mut file_data = read_file(find_file(&filename).unwrap());
            let data = RefCell::new(&mut file_data[..]);
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
    pub fn test_raydium_price() -> Result<()> {
        let filename = format!(
            "resources/test/{}.bin",
            "2QdhepnKRTLjjSqPL1PtKNwqrUkoLee5Gqs8bvZhRdMv"
        );

        let mut file_data = read_file(find_file(&filename).unwrap());
        let data = RefCell::new(&mut file_data[..]);
        let ai = &AccountInfoRef {
            key: &Pubkey::default(),
            owner: &raydium_internal::ID,
            data: data.borrow(),
        };

        let oracle = oracle_state_unchecked(ai)?;

        let price_from_raydium_sdk = 24.470_087_964_273_85f64;
        println!("{:?}", oracle.price);
        assert!((oracle.price - price_from_raydium_sdk).abs() < 1e-10);

        Ok(())
    }

    #[test]
    pub fn lookup_test() {
        for idx in -12..0_i8 {
            let s = format!("0.{}1", str::repeat("0", (idx.unsigned_abs() as usize) - 1));
            assert_eq!(power_of_ten_float(idx), f64::from_str(&s).unwrap());
        }

        assert_eq!(power_of_ten_float(0), 1.);

        for idx in 1..=12_i8 {
            let s = format!("1{}", str::repeat("0", idx.unsigned_abs() as usize));
            assert_eq!(power_of_ten_float(idx), f64::from_str(&s).unwrap());
        }
    }
}
