use anchor_lang::prelude::*;

use derivative::Derivative;
use static_assertions::const_assert_eq;
use std::mem::size_of;

use crate::state::*;

#[zero_copy]
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Position {
    /// Base lots in open bids
    pub bids_base_lots: i64,
    /// Base lots in open asks
    pub asks_base_lots: i64,

    pub base_free_native: u64,
    pub quote_free_native: u64,

    pub locked_maker_fees: u64,
    pub referrer_rebates_accrued: u64,

    /// Cumulative maker volume in quote native units (display only)
    pub maker_volume: u64,
    /// Cumulative taker volume in quote native units (display only)
    pub taker_volume: u64,

    #[derivative(Debug = "ignore")]
    pub reserved: [u8; 88],
}

const_assert_eq!(size_of::<Position>(), 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 88);
const_assert_eq!(size_of::<Position>(), 152);
const_assert_eq!(size_of::<Position>() % 8, 0);

impl Default for Position {
    fn default() -> Self {
        Self {
            bids_base_lots: 0,
            asks_base_lots: 0,
            base_free_native: 0,
            quote_free_native: 0,
            locked_maker_fees: 0,
            referrer_rebates_accrued: 0,
            maker_volume: 0,
            taker_volume: 0,
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
#[derive(Debug)]
pub struct OpenOrder {
    pub id: u128,
    pub client_id: u64,
    /// Price at which user's assets were locked
    pub locked_price: i64,

    pub is_free: u8,
    pub side_and_tree: u8, // SideAndOrderTree -- enums aren't POD
    pub padding: [u8; 6],
    pub reserved: [u8; 32],
}
const_assert_eq!(size_of::<OpenOrder>(), 16 + 8 + 8 + 1 + 1 + 6 + 32);
const_assert_eq!(size_of::<OpenOrder>(), 72);
const_assert_eq!(size_of::<OpenOrder>() % 8, 0);

impl Default for OpenOrder {
    fn default() -> Self {
        Self {
            is_free: true.into(),
            side_and_tree: SideAndOrderTree::BidFixed.into(),
            client_id: 0,
            locked_price: 0,
            id: 0,
            padding: [0; 6],
            reserved: [0u8; 32],
        }
    }
}

impl OpenOrder {
    pub fn is_free(&self) -> bool {
        self.is_free == u8::from(true)
    }

    pub fn side_and_tree(&self) -> SideAndOrderTree {
        SideAndOrderTree::try_from(self.side_and_tree).unwrap()
    }
}
