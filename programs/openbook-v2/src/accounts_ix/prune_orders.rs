use crate::error::OpenBookError;
use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct PruneOrders<'info> {
    pub close_market_admin: Signer<'info>,
    #[account(
        mut,
        has_one = market
    )]
    pub open_orders_account: AccountLoader<'info, OpenOrdersAccount>,
    #[account(
        has_one = bids,
        has_one = asks,
        constraint = market.load()?.close_market_admin.is_some() @ OpenBookError::NoCloseMarketAdmin,
        constraint = market.load()?.close_market_admin == close_market_admin.key() @ OpenBookError::InvalidCloseMarketAdmin
    )]
    pub market: AccountLoader<'info, Market>,
    #[account(mut)]
    pub bids: AccountLoader<'info, BookSide>,
    #[account(mut)]
    pub asks: AccountLoader<'info, BookSide>,
}
