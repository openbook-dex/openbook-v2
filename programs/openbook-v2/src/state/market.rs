use anchor_lang::prelude::*;
use fixed::types::I80F48;
use static_assertions::const_assert_eq;
use std::convert::{TryFrom, TryInto};
use std::mem::size_of;

use crate::error::OpenBookError;
use crate::pod_option::PodOption;
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

    /// No expiry = 0. Market will expire and no trading allowed after time_expiry
    pub time_expiry: i64,

    /// Admin who can collect fees from the market
    pub collect_fee_admin: Pubkey,
    /// Admin who must sign off on all order creations
    pub open_orders_admin: PodOption<Pubkey>,
    /// Admin who must sign off on all event consumptions
    pub consume_events_admin: PodOption<Pubkey>,
    /// Admin who can set market expired, prune orders and close the market
    pub close_market_admin: PodOption<Pubkey>,

    /// Name. Trailing zero bytes are ignored.
    pub name: [u8; 16],

    /// Address of the BookSide account for bids
    pub bids: Pubkey,
    /// Address of the BookSide account for asks
    pub asks: Pubkey,
    /// Address of the EventQueue account
    pub event_queue: Pubkey,

    /// Oracle account address
    pub oracle: Pubkey,
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
    /// Fee (in quote native) to charge for ioc orders that don't match to avoid spam
    pub fee_penalty: u64,

    // Total (maker + taker) fees accrued in native quote.
    pub fees_accrued: u64,
    // Total fees settled in native quote
    pub fees_to_referrers: u64,

    /// Cumulative taker volume in quote native units due to take orders
    pub taker_volume_wo_oo: u64,

    // Fields related to MarketSate, related to the tokenAccounts
    pub vault_signer_nonce: u64,

    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,

    pub base_vault: Pubkey,
    pub base_deposit_total: u64,
    pub base_fees_accrued: u64,

    pub quote_vault: Pubkey,
    pub quote_deposit_total: u64,
    pub quote_fees_accrued: u64,
    pub referrer_rebates_accrued: u64,

    pub reserved: [u8; 1768],
}

const_assert_eq!(
    size_of::<Market>(),
    32 + // size of collect_fee_admin
    40 + // size of open_order_admin
    40 + // size of consume_event_admin
    40 + // size of close_market_admin
    size_of::<MarketIndex>() + // size of MarketIndex
    1 + // size of bump
    1 + // size of base_decimals
    1 + // size of quote_decimals
    1 + // size of padding1
    8 + // size of time_expiry
    16 + // size of name
    3 * 32 + // size of bids, asks, and event_queue
    32 + // size of oracle
    size_of::<OracleConfig>() + // size of oracle_config
    8 + // size of quote_lot_size
    8 + // size of base_lot_size
    8 + // size of seq_num
    8 + // size of registration_time
    8 + // size of maker_fee 
    8 + // size of taker_fee
    8 + // size of fee_penalty
    8 + // size of fees_accrued
    8 + // size of fees_to_referrers
    8 + // size of taker_volume_wo_oo
    8 + // size of vault_signer_nonce
    4 * 32 + // size of base_mint, quote_mint, base_vault, and quote_vault
    8 + // size of base_deposit_total
    8 + // size of base_fees_accrued
    8 + // size of quote_deposit_total
    8 + // size of quote_fees_accrued
    8 + // size of referrer_rebates_accrued
    1768 // size of reserved
);
const_assert_eq!(size_of::<Market>(), 2432);
const_assert_eq!(size_of::<Market>() % 8, 0);

impl Market {
    pub fn name(&self) -> &str {
        std::str::from_utf8(&self.name)
            .unwrap()
            .trim_matches(char::from(0))
    }

    pub fn gen_order_id(&mut self, side: Side, price_data: u64) -> u128 {
        self.seq_num += 1;
        orderbook::new_node_key(side, price_data, self.seq_num)
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

    pub fn oracle_price(
        &self,
        oracle_acc: &impl KeyedAccountReader,
        staleness_slot: u64,
    ) -> Result<I80F48> {
        require_keys_eq!(self.oracle, *oracle_acc.key());
        oracle::oracle_price(
            oracle_acc,
            &self.oracle_config,
            self.base_decimals,
            self.quote_decimals,
            staleness_slot,
        )
    }

    /// Update the market's quote fees acrued and returns the penalty fee
    pub fn apply_penalty(&mut self) -> u64 {
        self.quote_fees_accrued += self.fee_penalty;
        self.fee_penalty
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
            &$market.market_index.to_le_bytes(),
            &[$market.bump],
        ]
    };
}
pub(crate) use market_seeds;
