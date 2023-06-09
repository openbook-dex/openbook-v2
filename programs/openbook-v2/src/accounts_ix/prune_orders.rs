use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

#[derive(Accounts)]
pub struct PruneOrders<'info> {
    pub close_market_admin: Signer<'info>,
    #[account(
        mut,
        // owner is not checked, only close_market_admin
    )]
    pub open_orders_account: AccountLoader<'info, OpenOrdersAccountFixed>,

    #[account(
        has_one = bids,
        has_one = asks,
    )]
    pub market: AccountLoader<'info, Market>,
    #[account(mut)]
    pub bids: AccountLoader<'info, BookSide>,
    #[account(mut)]
    pub asks: AccountLoader<'info, BookSide>,
}
