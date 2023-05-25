use anchor_lang::prelude::*;
use fixed::types::I80F48;
use static_assertions::const_assert_eq;
use std::mem::size_of;

use crate::pod_option::PodOption;
use crate::state::oracle;
use crate::{accounts_zerocopy::KeyedAccountReader, state::orderbook::Side};

use super::{orderbook, OracleConfig, StablePriceModel};

pub type MarketIndex = u32;

#[account(zero_copy)]
#[derive(Debug)]
pub struct Market {
    /// Admin who can collect fees from the market
    pub collect_fee_admin: Pubkey,
    /// Admin who can manage the oracle
    pub manage_oracle_admin: PodOption<Pubkey>,
    /// Admin who must sign off on all order creations
    pub open_orders_admin: PodOption<Pubkey>,
    /// Admin who must sign off on all event consumptions
    pub consume_events_admin: PodOption<Pubkey>,
    /// Admin who can close the market
    pub close_market_admin: PodOption<Pubkey>,

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
    /// Maintains a stable price based on the oracle price that is less volatile.
    pub stable_price_model: StablePriceModel,

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
    /// Fee when matching maker orders.
    /// maker_fee < 0 it means some of the taker_fees goes to the maker
    /// maker_fee > 0, it means no taker_fee to the maker, and maker fee goes to the referral
    pub maker_fee: I80F48,
    /// Fee for taker orders, always >= 0.
    pub taker_fee: I80F48,
    /// Fee (in quote native) to charge for ioc orders that don't match to avoid spam
    pub fee_penalty: u64,

    // Total (maker + taker) fees accrued in native quote.
    // i64 due there is a case where maker fees are subtracted (process_fill_event) before taker fees
    pub fees_accrued: i64,
    // Total fees settled in native quote
    pub fees_to_referrers: u64,

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

    pub reserved: [u8; 1728],
}

const_assert_eq!(
    size_of::<Market>(),
    32 + // size of collect_fee_admin
    40 + // size of manage_oracle_admin
    40 + // size of open_order_admin
    40 + // size of consume_event_admin
    40 + // size of close_market_admin
    size_of::<MarketIndex>() + // size of MarketIndex
    1 + // size of bump
    1 + // size of base_decimals
    1 + // size of quote_decimals
    1 + // size of padding1
    16 + // size of name
    3 * 32 + // size of bids, asks, and event_queue
    32 + // size of oracle
    size_of::<OracleConfig>() + // size of oracle_config
    size_of::<StablePriceModel>() + // size of stable_price_model
    8 + // size of quote_lot_size
    8 + // size of base_lot_size
    8 + // size of seq_num
    8 + // size of registration_time
    2 * size_of::<I80F48>() + // size of maker_fee and taker_fee
    8 + // size of fee_penalty
    8 + // size of fees_accrued
    8 + // size of fees_to_referrers
    8 + // size of vault_signer_nonce
    4 * 32 + // size of base_mint, quote_mint, base_vault, and quote_vault
    8 + // size of base_deposit_total
    8 + // size of base_fees_accrued
    8 + // size of quote_deposit_total
    8 + // size of quote_fees_accrued
    8 + // size of referrer_rebates_accrued
    1728 // size of reserved
);
const_assert_eq!(size_of::<Market>(), 2720);
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

    pub fn native_price_to_lot(&self, price: I80F48) -> i64 {
        (price * I80F48::from_num(self.base_lot_size) / I80F48::from_num(self.quote_lot_size))
            .to_num()
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

    // TODO binye
    /// Creates default market for tests
    pub fn default_for_tests() -> Market {
        Market {
            collect_fee_admin: Pubkey::new_unique(),
            manage_oracle_admin: Some(Pubkey::new_unique()).into(),
            open_orders_admin: Some(Pubkey::new_unique()).into(),
            consume_events_admin: Some(Pubkey::new_unique()).into(),
            close_market_admin: Some(Pubkey::new_unique()).into(),
            market_index: 0,
            bump: 0,
            base_decimals: 0,
            quote_decimals: 0,
            padding1: Default::default(),
            name: Default::default(),
            bids: Pubkey::new_unique(),
            asks: Pubkey::new_unique(),
            event_queue: Pubkey::new_unique(),
            oracle: Pubkey::new_unique(),
            oracle_config: OracleConfig {
                conf_filter: I80F48::ZERO,
                max_staleness_slots: -1,
                reserved: [0; 72],
            },
            stable_price_model: StablePriceModel::default(),

            quote_lot_size: 1,
            base_lot_size: 1,
            seq_num: 0,
            registration_time: 0,
            maker_fee: I80F48::ZERO,
            taker_fee: I80F48::ZERO,
            fee_penalty: 0,
            fees_accrued: 0,
            fees_to_referrers: 0,
            vault_signer_nonce: 0,
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),

            base_vault: Pubkey::new_unique(),
            base_deposit_total: 0,
            base_fees_accrued: 0,

            quote_vault: Pubkey::new_unique(),
            quote_deposit_total: 0,
            quote_fees_accrued: 0,
            referrer_rebates_accrued: 0,
            reserved: [0; 1728],
        }
    }

    pub fn subtract_taker_fees(&self, quote: i64) -> i64 {
        (I80F48::from(quote) / (I80F48::ONE + self.taker_fee)).to_num()
    }
    // Only for maker_fee > 0
    pub fn subtract_maker_fees(&self, quote: i64) -> i64 {
        (I80F48::from(quote) / (I80F48::ONE + self.maker_fee)).to_num()
    }

    pub fn referrer_taker_rebate(&self, quote: u64) -> u64 {
        let quo = I80F48::from_num(quote);
        if self.maker_fee < 0 {
            (quo * (self.taker_fee + self.maker_fee)).to_num()
        } else {
            // Nothing goes to maker, all to referrer
            (quo * self.taker_fee).to_num()
        }
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
