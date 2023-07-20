use anchor_lang::prelude::*;
use fixed::types::I80F48;
use static_assertions::const_assert_eq;
use std::convert::{TryFrom, TryInto};
use std::mem::size_of;

use crate::error::OpenBookError;
use crate::pubkey_option::NonZeroPubkeyOption;
use crate::state::oracle;
use crate::{accounts_zerocopy::KeyedAccountReader, state::orderbook::Side};

use super::{orderbook, OracleConfig};

pub type MarketIndex = u32;
pub const FEES_SCALE_FACTOR: i128 = 1_000_000;

#[account(zero_copy)]
#[derive(Debug)]
pub struct Market {
    /// Index of this market
    pub market_index: MarketIndex,
    /// PDA bump
    pub bump: u8,

    /// Number of decimals used for the base token.
    ///
    /// Used to convert the oracle's price into a native/native price.
    pub base_decimals: u8,
    pub quote_decimals: u8,

    pub padding1: [u8; 1],

    // Signer of the create market transaction
    pub signer_creator: Pubkey,

    /// No expiry = 0. Market will expire and no trading allowed after time_expiry
    pub time_expiry: i64,

    /// Admin who can collect fees from the market
    pub collect_fee_admin: Pubkey,
    /// Admin who must sign off on all order creations
    pub open_orders_admin: NonZeroPubkeyOption,
    /// Admin who must sign off on all event consumptions
    pub consume_events_admin: NonZeroPubkeyOption,
    /// Admin who can set market expired, prune orders and close the market
    pub close_market_admin: NonZeroPubkeyOption,

    /// Name. Trailing zero bytes are ignored.
    pub name: [u8; 16],

    /// Address of the BookSide account for bids
    pub bids: Pubkey,
    /// Address of the BookSide account for asks
    pub asks: Pubkey,
    /// Address of the EventQueue account
    pub event_queue: Pubkey,

    /// Oracles account address
    pub oracle_a: NonZeroPubkeyOption,
    pub oracle_b: NonZeroPubkeyOption,
    /// Oracle configuration
    pub oracle_config: OracleConfig,

    /// Number of quote native in a quote lot. Must be a power of 10.
    ///
    /// Primarily useful for increasing the tick size on the market: A lot price
    /// of 1 becomes a native price of quote_lot_size/base_lot_size becomes a
    /// ui price of quote_lot_size*base_decimals/base_lot_size/quote_decimals.
    pub quote_lot_size: i64,

    /// Number of base native in a base lot. Must be a power of 10.
    ///
    /// Example: If base decimals for the underlying asset is 6, base lot size
    /// is 100 and and base position lots is 10_000 then base position native is
    /// 1_000_000 and base position ui is 1.
    pub base_lot_size: i64,

    /// Total number of orders seen
    pub seq_num: u64,

    /// Timestamp in seconds that the market was registered at.
    pub registration_time: u64,

    /// Fees
    ///
    /// Fee (in 10^-6) when matching maker orders.
    /// maker_fee < 0 it means some of the taker_fees goes to the maker
    /// maker_fee > 0, it means no taker_fee to the maker, and maker fee goes to the referral
    pub maker_fee: i64,
    /// Fee (in 10^-6) for taker orders, always >= 0.
    pub taker_fee: i64,

    /// Total fees accrued in native quote
    pub fees_accrued: u64,
    // Total fees settled in native quote
    pub fees_to_referrers: u64,

    /// Cumulative taker volume in quote native units due to place take orders
    pub taker_volume_wo_oo: u64,

    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,

    pub base_vault: Pubkey,
    pub base_deposit_total: u64,

    pub quote_vault: Pubkey,
    pub quote_deposit_total: u64,

    pub quote_fees_accrued: u64,
    pub referrer_rebates_accrued: u64,

    pub reserved: [u8; 1768],
}

const_assert_eq!(
    size_of::<Market>(),
    32 +                        // signer_creator
    32 +                        // collect_fee_admin
    32 +                        // open_order_admin
    32 +                        // consume_event_admin
    32 +                        // close_market_admin
    size_of::<MarketIndex>() +  // MarketIndex
    1 +                         // bump
    1 +                         // base_decimals
    1 +                         // quote_decimals
    1 +                         // padding1
    8 +                         // time_expiry
    16 +                        // name
    3 * 32 +                    // bids, asks, and event_queue
    32 +                        // oracle_a
    32 +                        // oracle_b
    size_of::<OracleConfig>() + // oracle_config
    8 +                         // quote_lot_size
    8 +                         // base_lot_size
    8 +                         // seq_num
    8 +                         // registration_time
    8 +                         // maker_fee
    8 +                         // taker_fee
    8 +                         // fees_accrued
    8 +                         // fees_to_referrers
    8 +                         // taker_volume_wo_oo
    4 * 32 +                    // base_mint, quote_mint, base_vault, and quote_vault
    8 +                         // base_deposit_total
    8 +                         // quote_deposit_total
    8 +                         // base_fees_accrued
    8 +                         // referrer_rebates_accrued
    1768 // reserved
);
const_assert_eq!(size_of::<Market>(), 2448);
const_assert_eq!(size_of::<Market>() % 8, 0);

impl Market {
    pub fn name(&self) -> &str {
        std::str::from_utf8(&self.name)
            .unwrap()
            .trim_matches(char::from(0))
    }

    pub fn is_market_vault(&self, pubkey: Pubkey) -> bool {
        pubkey == self.quote_vault || pubkey == self.base_vault
    }

    pub fn gen_order_id(&mut self, side: Side, price_data: u64) -> u128 {
        self.seq_num += 1;
        orderbook::new_node_key(side, price_data, self.seq_num)
    }

    pub fn max_base_lots(&self) -> i64 {
        i64::MAX / self.base_lot_size
    }

    pub fn max_quote_lots(&self) -> i64 {
        i64::MAX / self.quote_lot_size
    }

    /// Convert from the price stored on the book to the price used in value calculations
    pub fn lot_to_native_price(&self, price: i64) -> I80F48 {
        I80F48::from_num(price) * I80F48::from_num(self.quote_lot_size)
            / I80F48::from_num(self.base_lot_size)
    }

    pub fn native_price_to_lot(&self, price: I80F48) -> Result<i64> {
        price
            .checked_mul(I80F48::from_num(self.base_lot_size))
            .and_then(|x| x.checked_div(I80F48::from_num(self.quote_lot_size)))
            .and_then(|x| x.checked_to_num())
            .ok_or_else(|| OpenBookError::InvalidOraclePrice.into())
    }

    pub fn oracle_price_from_a(
        &self,
        oracle_acc: &impl KeyedAccountReader,
        staleness_slot: u64,
    ) -> Result<I80F48> {
        assert_eq!(self.oracle_a, *oracle_acc.key());
        let (price, _err) =
            oracle::oracle_price_data(oracle_acc, &self.oracle_config, staleness_slot)?;

        let decimals = (self.quote_decimals as i8) - (self.base_decimals as i8);
        let decimal_adj = oracle::power_of_ten(decimals);
        Ok(price * decimal_adj)
    }

    pub fn oracle_price_from_a_and_b(
        &self,
        oracle_a_acc: &impl KeyedAccountReader,
        oracle_b_acc: &impl KeyedAccountReader,
        staleness_slot: u64,
    ) -> Result<I80F48> {
        assert_eq!(self.oracle_a, *oracle_a_acc.key());
        assert_eq!(self.oracle_b, *oracle_b_acc.key());

        let (price_a, err_a) =
            oracle::oracle_price_data(oracle_a_acc, &self.oracle_config, staleness_slot)?;

        let (price_b, err_b) =
            oracle::oracle_price_data(oracle_b_acc, &self.oracle_config, staleness_slot)?;

        let price = match self.oracle_config.price_relation.try_into().unwrap() {
            oracle::PriceRelation::Multiplication => price_a * price_b,
            oracle::PriceRelation::Division => price_a / price_b,
            _ => unreachable!(),
        };

        // no sqrt impl in fixed so we compare the squares
        let target_var = self.oracle_config.conf_filter * self.oracle_config.conf_filter;
        let var = {
            let relative_err_a = price_a / err_a;
            let relative_err_b = price_b / err_b;
            price * price * (relative_err_a * relative_err_a + relative_err_b * relative_err_b)
        };

        if var > target_var {
            msg!(
                "Combined variance too high; value {}, target {}",
                var,
                target_var
            );
            return Err(OpenBookError::OracleConfidence.into());
        }

        let decimals = (self.quote_decimals as i8) - (self.base_decimals as i8);
        let decimal_adj = oracle::power_of_ten(decimals);
        Ok(price * decimal_adj)
    }

    pub fn subtract_taker_fees(&self, quote: i64) -> i64 {
        ((quote as i128) * FEES_SCALE_FACTOR / (FEES_SCALE_FACTOR + (self.taker_fee as i128)))
            .try_into()
            .unwrap()
    }

    pub fn taker_fees_floor(self, amount: u64) -> u64 {
        (i128::from(amount) * i128::from(self.taker_fee) / FEES_SCALE_FACTOR)
            .try_into()
            .unwrap()
    }

    pub fn maker_fees_floor(self, amount: u64) -> u64 {
        if self.maker_fee.is_positive() {
            self.unsigned_maker_fees_floor(amount)
        } else {
            0
        }
    }

    pub fn maker_rebate_floor(self, amount: u64) -> u64 {
        if self.maker_fee.is_positive() {
            0
        } else {
            self.unsigned_maker_fees_floor(amount)
        }
    }

    pub fn maker_fees_ceil<T>(self, amount: T) -> T
    where
        T: Into<i128> + TryFrom<i128> + From<u8>,
        <T as TryFrom<i128>>::Error: std::fmt::Debug,
    {
        if self.maker_fee.is_positive() {
            self.ceil_fee_division(amount.into() * (self.maker_fee.abs() as i128))
                .try_into()
                .unwrap()
        } else {
            T::from(0)
        }
    }

    pub fn taker_fees_ceil<T>(self, amount: T) -> T
    where
        T: Into<i128> + TryFrom<i128>,
        <T as TryFrom<i128>>::Error: std::fmt::Debug,
    {
        self.ceil_fee_division(amount.into() * (self.taker_fee as i128))
            .try_into()
            .unwrap()
    }

    fn ceil_fee_division(self, numerator: i128) -> i128 {
        (numerator + (FEES_SCALE_FACTOR - 1_i128)) / FEES_SCALE_FACTOR
    }

    fn unsigned_maker_fees_floor(self, amount: u64) -> u64 {
        (i128::from(amount) * i128::from(self.maker_fee.abs()) / FEES_SCALE_FACTOR)
            .try_into()
            .unwrap()
    }
}

/// Generate signed seeds for the market
macro_rules! market_seeds {
    ($market:expr) => {
        &[
            b"Market".as_ref(),
            &$market.signer_creator.to_bytes(),
            &$market.market_index.to_le_bytes(),
            &[$market.bump],
        ]
    };
}
pub(crate) use market_seeds;
