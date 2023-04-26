use crate::state::{Market, Position};
use anchor_lang::prelude::*;
use borsh::BorshSerialize;

pub fn emit_balances(open_orders_acc: Pubkey, p: &Position, _m: &Market) {
    emit!(BalanceLog {
        open_orders_acc,
        base_position: p.base_position_lots(),
        quote_position: p.quote_position_native().to_bits(),
    });
}

#[event]
pub struct BalanceLog {
    pub open_orders_acc: Pubkey,
    pub base_position: i64,
    pub quote_position: i128, // I80F48
}

#[event]
pub struct TokenBalanceLog {
    pub open_orders_acc: Pubkey,
    pub token_index: u16,       // IDL doesn't support usize
    pub indexed_position: i128, // on client convert i128 to I80F48 easily by passing in the BN to I80F48 ctor
    pub deposit_index: i128,    // I80F48
    pub borrow_index: i128,     // I80F48
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct FlashLoanTokenDetail {
    pub token_index: u16,
    pub change_amount: i128,
    pub loan: i128,
    pub loan_origination_fee: i128,
    pub deposit_index: i128,
    pub borrow_index: i128,
    pub price: i128,
}

#[event]
pub struct WithdrawLog {
    pub open_orders_acc: Pubkey,
    pub signer: Pubkey,
    pub token_index: u16,
    pub quantity: u64,
    pub price: i128, // I80F48
}

#[event]
pub struct DepositLog {
    pub open_orders_acc: Pubkey,
    pub signer: Pubkey,
    pub token_index: u16,
    pub quantity: u64,
    pub price: i128, // I80F48
}

#[event]
pub struct FillLog {
    pub taker_side: u8, // side from the taker's POV
    pub maker_slot: u8,
    pub maker_out: bool, // true if maker order quantity == 0
    pub timestamp: u64,
    pub seq_num: u64, // note: usize same as u64

    pub maker: Pubkey,
    pub maker_order_id: u128,
    pub maker_fee: i128,

    // Timestamp of when the maker order was placed; copied over from the LeafNode
    pub maker_timestamp: u64,

    pub taker: Pubkey,
    pub taker_order_id: u128,
    pub taker_client_order_id: u64,
    pub taker_fee: i128,

    pub price: i64,
    pub quantity: i64, // number of base lots
}

#[event]
pub struct FillLogV2 {
    pub taker_side: u8, // side from the taker's POV
    pub maker_slot: u8,
    pub maker_out: bool, // true if maker order quantity == 0
    pub timestamp: u64,
    pub seq_num: u64, // note: usize same as u64

    pub maker: Pubkey,
    pub maker_client_order_id: u64,
    pub maker_fee: f32,

    // Timestamp of when the maker order was placed; copied over from the LeafNode
    pub maker_timestamp: u64,

    pub taker: Pubkey,
    pub taker_client_order_id: u64,
    pub taker_fee: f32,

    pub price: i64,
    pub quantity: i64, // number of base lots
}

#[event]
pub struct UpdateIndexLog {
    pub token_index: u16,
    pub deposit_index: i128,   // I80F48
    pub borrow_index: i128,    // I80F48
    pub avg_utilization: i128, // I80F48
    pub price: i128,           // I80F48
    pub stable_price: i128,    // I80F48
    pub collected_fees: i128,  // I80F48
    pub loan_fee_rate: i128,   // I80F48
    pub total_borrows: i128,
    pub total_deposits: i128,
    pub borrow_rate: i128,
    pub deposit_rate: i128,
}

#[derive(PartialEq, Copy, Clone, Debug, AnchorSerialize, AnchorDeserialize)]
#[repr(u8)]
pub enum LoanOriginationFeeInstruction {
    Unknown,
    LiqTokenBankruptcy,
    LiqTokenWithToken,
    Serum3LiqForceCancelOrders,
    Serum3PlaceOrder,
    Serum3SettleFunds,
    TokenWithdraw,
}

#[event]
pub struct DeactivateTokenPositionLog {
    pub open_orders_acc: Pubkey,
    pub token_index: u16,
    pub cumulative_deposit_interest: f64,
    pub cumulative_borrow_interest: f64,
}

#[event]
pub struct DeactivatePositionLog {
    pub open_orders_acc: Pubkey,
    pub market_index: u16,
    pub cumulative_long_funding: f64,
    pub cumulative_short_funding: f64,
    pub maker_volume: u64,
    pub taker_volume: u64,
    pub spot_transfers: i64,
}

#[event]
pub struct TokenMetaDataLog {
    pub mint: Pubkey,
    pub token_index: u16,
    pub mint_decimals: u8,
    pub oracle: Pubkey,
    pub mint_info: Pubkey,
}

#[event]
pub struct MarketMetaDataLog {
    pub market: Pubkey,
    pub market_index: u16,
    pub base_decimals: u8,
    pub quote_decimals: u8,
    pub base_lot_size: i64,
    pub quote_lot_size: i64,
    pub oracle: Pubkey,
}

#[event]
pub struct SettleFeesLog {
    pub open_orders_acc: Pubkey,
    pub market_index: u16,
    pub settlement: i128,
}

#[event]
pub struct AccountBuybackFeesWithMngoLog {
    pub open_orders_acc: Pubkey,
    pub buyback_fees: i128,
    pub buyback_mngo: i128,
    pub mngo_buyback_price: i128,
    pub oracle_price: i128,
}
