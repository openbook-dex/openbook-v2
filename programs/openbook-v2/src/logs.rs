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
    pub taker_side: u8, // side from the taker's POV
    pub maker_slot: u8,
    pub maker_out: bool, // true if maker order quantity == 0
    pub timestamp: u64,
    pub seq_num: u64, // note: usize same as u64

    pub maker: Pubkey,
    pub maker_client_order_id: u64,
    pub maker_fee: i64,

    // Timestamp of when the maker order was placed; copied over from the LeafNode
    pub maker_timestamp: u64,

    pub taker: Pubkey,
    pub taker_client_order_id: u64,
    pub taker_fee: i64,

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
pub struct CancelOrderLog {
    pub open_orders_account: Pubkey,
    pub slot: u8,
    pub side: u8,
    pub quantity: i64,
}

#[event]
pub struct CancelOrdersLog {
    pub open_orders_account: Pubkey,
    pub total_quantity: i64,
}

#[event]
pub struct CancelAllOrdersLog {
    pub open_orders_account: Pubkey,
    pub side: Option<u8>,
    pub quantity: i64,
    pub limit: u8,
}

#[event]
pub struct PruneOrdersLog {
    pub open_orders_account: Pubkey,
    pub quantity: i64,
    pub limit: u8,
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
