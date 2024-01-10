use anchor_lang::prelude::*;
use borsh::BorshSerialize;

#[event]
pub struct DepositLog {
    pub open_orders_account: Pubkey,
    pub signer: Pubkey,
    pub base_amount: u64,
    pub quote_amount: u64,
}

#[event]
pub struct FillLog {
    pub market: Pubkey,
    pub taker_side: u8, // side from the taker's POV
    pub maker_slot: u8,
    pub maker_out: bool, // true if maker order quantity == 0
    pub timestamp: u64,
    pub seq_num: u64, // note: usize same as u64

    pub maker: Pubkey,
    pub maker_client_order_id: u64,
    pub maker_fee: u64, // native quote

    // Timestamp of when the maker order was placed; copied over from the LeafNode
    pub maker_timestamp: u64,

    pub taker: Pubkey,
    pub taker_client_order_id: u64,
    pub taker_fee: u64, // native quote

    pub price: i64,
    pub quantity: i64, // number of base lots
}

#[event]
pub struct MarketMetaDataLog {
    pub market: Pubkey,
    pub name: String,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_decimals: u8,
    pub quote_decimals: u8,
    pub base_lot_size: i64,
    pub quote_lot_size: i64,
}

#[event]
pub struct TotalOrderFillEvent {
    pub side: u8,
    pub taker: Pubkey,
    pub total_quantity_paid: u64,
    pub total_quantity_received: u64,
    pub fees: u64,
}

#[event]
pub struct SetDelegateLog {
    pub open_orders_account: Pubkey,
    pub delegate: Option<Pubkey>,
}

#[event]
pub struct SettleFundsLog {
    pub open_orders_account: Pubkey,
    pub base_native: u64,
    pub quote_native: u64,
    pub referrer_rebate: u64,
    pub referrer: Option<Pubkey>,
}

#[event]
pub struct SweepFeesLog {
    pub market: Pubkey,
    pub amount: u64,
    pub receiver: Pubkey,
}

#[event]
pub struct OpenOrdersPositionLog {
    pub owner: Pubkey,
    pub open_orders_account_num: u32,
    pub market: Pubkey,
    /// Base lots in open bids
    pub bids_base_lots: i64,
    /// Quote lots in open bids
    pub bids_quote_lots: i64,
    /// Base lots in open asks
    pub asks_base_lots: i64,
    pub base_free_native: u64,
    pub quote_free_native: u64,
    pub locked_maker_fees: u64,
    pub referrer_rebates_available: u64,
    /// Cumulative maker volume in quote native units (display only)
    pub maker_volume: u128,
    /// Cumulative taker volume in quote native units (display only)
    pub taker_volume: u128,
}
