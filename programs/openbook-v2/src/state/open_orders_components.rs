use anchor_lang::prelude::*;

use derivative::Derivative;
use fixed::types::I80F48;
use static_assertions::const_assert_eq;
use std::mem::size_of;

use crate::state::*;

pub const FREE_ORDER_SLOT: MarketIndex = MarketIndex::MAX;

#[zero_copy]
#[derive(AnchorSerialize, AnchorDeserialize, Derivative)]
#[derivative(Debug)]
pub struct Position {
    /// Base lots in open bids
    pub bids_base_lots: i64,
    /// Base lots in open asks
    pub asks_base_lots: i64,

    pub base_free_native: I80F48,
    pub quote_free_native: I80F48,

    pub referrer_rebates_accrued: u64,

    /// Cumulative maker volume in quote native units
    ///
    /// (Display only)
    pub maker_volume: u64,
    /// Cumulative taker volume in quote native units
    ///
    /// (Display only)
    pub taker_volume: u64,

    /// The native average entry price for the base lots of the current position.
    /// Reset to 0 when the base position reaches or crosses 0.
    pub avg_entry_price_per_base_lot: f64,

    #[derivative(Debug = "ignore")]
    pub reserved: [u8; 88],
}

const_assert_eq!(
    size_of::<Position>(),
    2 * size_of::<I80F48>() + 8 + 8 + 8 + 8 + 8 + 8 + 88
);
const_assert_eq!(size_of::<Position>(), 168);
const_assert_eq!(size_of::<Position>() % 8, 0);

impl Default for Position {
    fn default() -> Self {
        Self {
            bids_base_lots: 0,
            asks_base_lots: 0,
            base_free_native: I80F48::ZERO,
            quote_free_native: I80F48::ZERO,
            referrer_rebates_accrued: 0,
            maker_volume: 0,
            taker_volume: 0,
            avg_entry_price_per_base_lot: 0.0,
            reserved: [0; 88],
        }
    }
}

impl Position {
    /// Does the user have any orders on the book?
    ///
    /// Note that it's possible they were matched already: This only becomes
    /// false when the fill event is processed or the orders are cancelled.
    pub fn has_open_orders(&self) -> bool {
        self.asks_base_lots != 0 || self.bids_base_lots != 0
    }
}

#[zero_copy]
#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct OpenOrder {
    pub side_and_tree: u8, // SideAndOrderTree -- enums aren't POD
    pub padding1: [u8; 7],
    pub client_id: u64,
    pub peg_limit: i64,
    pub id: u128,
    pub reserved: [u8; 64],
}
const_assert_eq!(size_of::<OpenOrder>(), 1 + 7 + 8 + 8 + 16 + 64);
const_assert_eq!(size_of::<OpenOrder>(), 104);
const_assert_eq!(size_of::<OpenOrder>() % 8, 0);

impl Default for OpenOrder {
    fn default() -> Self {
        Self {
            side_and_tree: SideAndOrderTree::BidFixed.into(),
            padding1: Default::default(),
            client_id: 0,
            peg_limit: 0,
            id: 0,
            reserved: [0; 64],
        }
    }
}

impl OpenOrder {
    pub fn side_and_tree(&self) -> SideAndOrderTree {
        SideAndOrderTree::try_from(self.side_and_tree).unwrap()
    }
}
