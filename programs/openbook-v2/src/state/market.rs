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

// For a 1bps taker fee, set taker_fee to 100, so taker_fee/FEES_SCALE_FACTOR = 10e-4
pub const FEES_SCALE_FACTOR: i128 = 1_000_000;
// taker pays 500 lamports penalty for every transaction that adds to the event heap
pub const PENALTY_EVENT_HEAP: u64 = 500;

#[account(zero_copy)]
#[derive(Debug)]
pub struct Market {
    /// PDA bump
    pub bump: u8,

    /// Number of decimals used for the base token.
    ///
    /// Used to convert the oracle's price into a native/native price.
    pub base_decimals: u8,
    pub quote_decimals: u8,

    pub padding1: [u8; 5],

    // Pda for signing vault txs
    pub market_authority: Pubkey,

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
    /// Address of the EventHeap account
    pub event_heap: Pubkey,

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
    pub registration_time: i64,

    /// Fees
    ///
    /// Fee (in 10^-6) when matching maker orders.
    /// maker_fee < 0 it means some of the taker_fees goes to the maker
    /// maker_fee > 0, it means no taker_fee to the maker, and maker fee goes to the referral
    pub maker_fee: i64,
    /// Fee (in 10^-6) for taker orders, always >= 0.
    pub taker_fee: i64,

    /// Total fees accrued in native quote
    pub fees_accrued: u128,
    /// Total fees settled in native quote
    pub fees_to_referrers: u128,

    /// Referrer rebates to be distributed
    pub referrer_rebates_accrued: u64,

    /// Fees generated and available to withdraw via sweep_fees
    pub fees_available: u64,

    /// Cumulative maker volume (same as taker volume) in quote native units
    pub maker_volume: u128,

    /// Cumulative taker volume in quote native units due to place take orders
    pub taker_volume_wo_oo: u128,

    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,

    pub market_base_vault: Pubkey,
    pub base_deposit_total: u64,

    pub market_quote_vault: Pubkey,
    pub quote_deposit_total: u64,

    pub reserved: [u8; 128],
}

const_assert_eq!(
    size_of::<Market>(),
    32 +                        // market_authority
    32 +                        // collect_fee_admin
    32 +                        // open_order_admin
    32 +                        // consume_event_admin
    32 +                        // close_market_admin
    1 +                         // bump
    1 +                         // base_decimals
    1 +                         // quote_decimals
    5 +                         // padding1
    8 +                         // time_expiry
    16 +                        // name
    3 * 32 +                    // bids, asks, and event_heap
    32 +                        // oracle_a
    32 +                        // oracle_b
    size_of::<OracleConfig>() + // oracle_config
    8 +                         // quote_lot_size
    8 +                         // base_lot_size
    8 +                         // seq_num
    8 +                         // registration_time
    8 +                         // maker_fee
    8 +                         // taker_fee
    16 +                        // fees_accrued
    16 +                        // fees_to_referrers
    16 +                        // maker_volume
    16 +                        // taker_volume_wo_oo
    4 * 32 +                    // base_mint, quote_mint, market_base_vault, and market_quote_vault
    8 +                         // base_deposit_total
    8 +                         // quote_deposit_total
    8 +                         // base_fees_accrued
    8 +                         // referrer_rebates_accrued
    128 // reserved
);
const_assert_eq!(size_of::<Market>(), 840);
const_assert_eq!(size_of::<Market>() % 8, 0);

impl Market {
    pub fn name(&self) -> &str {
        std::str::from_utf8(&self.name)
            .unwrap()
            .trim_matches(char::from(0))
    }

    pub fn is_expired(&self, timestamp: i64) -> bool {
        self.time_expiry != 0 && self.time_expiry < timestamp
    }

    pub fn is_empty(&self) -> bool {
        self.base_deposit_total == 0
            && self.quote_deposit_total == 0
            && self.fees_available == 0
            && self.referrer_rebates_accrued == 0
    }

    pub fn is_market_vault(&self, pubkey: Pubkey) -> bool {
        pubkey == self.market_quote_vault || pubkey == self.market_base_vault
    }

    pub fn get_vault_by_side(&self, side: Side) -> Pubkey {
        match side {
            Side::Ask => self.market_base_vault,
            Side::Bid => self.market_quote_vault,
        }
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

    pub fn max_base_lots_from_lamports(&self, lamports: u64) -> i64 {
        let base_lots = lamports / self.base_lot_size as u64;
        std::cmp::min(self.max_base_lots() as u64, base_lots)
            .try_into()
            .unwrap()
    }

    pub fn max_quote_lots_from_lamports(&self, lamports: u64) -> i64 {
        let quote_lots = lamports / self.quote_lot_size as u64;
        std::cmp::min(self.max_quote_lots() as u64, quote_lots)
            .try_into()
            .unwrap()
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

    pub fn oracle_price_lots(
        &self,
        oracle_a_acc: Option<&impl KeyedAccountReader>,
        oracle_b_acc: Option<&impl KeyedAccountReader>,
        slot: u64,
    ) -> Result<Option<i64>> {
        let oracle_price = self.oracle_price(oracle_a_acc, oracle_b_acc, slot)?;
        match oracle_price {
            Some(p) => Ok(Some(self.native_price_to_lot(p)?)),
            None => Ok(None),
        }
    }

    pub fn oracle_price(
        &self,
        oracle_a_acc: Option<&impl KeyedAccountReader>,
        oracle_b_acc: Option<&impl KeyedAccountReader>,
        slot: u64,
    ) -> Result<Option<I80F48>> {
        if self.oracle_a.is_some() && self.oracle_b.is_some() {
            self.oracle_price_from_a_and_b(oracle_a_acc.unwrap(), oracle_b_acc.unwrap(), slot)
        } else if self.oracle_a.is_some() {
            self.oracle_price_from_a(oracle_a_acc.unwrap(), slot)
        } else {
            Ok(None)
        }
    }

    fn oracle_price_from_a(
        &self,
        oracle_acc: &impl KeyedAccountReader,
        now_slot: u64,
    ) -> Result<Option<I80F48>> {
        assert_eq!(self.oracle_a, *oracle_acc.key());
        let oracle = oracle::oracle_state_unchecked(oracle_acc)?;

        if oracle.is_stale(oracle_acc.key(), &self.oracle_config, now_slot)
            || !oracle.has_valid_confidence(oracle_acc.key(), &self.oracle_config)
        {
            Ok(None)
        } else {
            let decimals = (self.quote_decimals as i8) - (self.base_decimals as i8);
            let decimal_adj = oracle::power_of_ten_float(decimals);
            Ok(I80F48::checked_from_num(oracle.price * decimal_adj))
        }
    }

    fn oracle_price_from_a_and_b(
        &self,
        oracle_a_acc: &impl KeyedAccountReader,
        oracle_b_acc: &impl KeyedAccountReader,
        now_slot: u64,
    ) -> Result<Option<I80F48>> {
        assert_eq!(self.oracle_a, *oracle_a_acc.key());
        assert_eq!(self.oracle_b, *oracle_b_acc.key());

        let oracle_a = oracle::oracle_state_unchecked(oracle_a_acc)?;
        let oracle_b = oracle::oracle_state_unchecked(oracle_b_acc)?;

        if oracle_a.is_stale(oracle_a_acc.key(), &self.oracle_config, now_slot)
            || oracle_b.is_stale(oracle_b_acc.key(), &self.oracle_config, now_slot)
            || !oracle_a.has_valid_combined_confidence(&oracle_b, &self.oracle_config)
        {
            Ok(None)
        } else {
            let price = oracle_a.price / oracle_b.price;
            let decimals = (self.quote_decimals as i8) - (self.base_decimals as i8);
            let decimal_adj = oracle::power_of_ten_float(decimals);
            Ok(I80F48::checked_from_num(price * decimal_adj))
        }
    }

    pub fn subtract_taker_fees(&self, quote: i64) -> i64 {
        ((quote as i128) * FEES_SCALE_FACTOR / (FEES_SCALE_FACTOR + (self.taker_fee as i128)))
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
    ($market:expr,$key:expr) => {
        &[b"Market".as_ref(), &$key.to_bytes(), &[$market.bump]]
    };
}
pub(crate) use market_seeds;
