use crate::error::OpenBookError;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::Token;

#[derive(Accounts)]
pub struct CloseMarket<'info> {
    pub close_market_admin: Signer<'info>,
    #[account(
        mut,
        has_one = bids,
        has_one = asks,
        has_one = event_heap,
        close = sol_destination,
        constraint = market.load()?.close_market_admin.is_some() @ OpenBookError::NoCloseMarketAdmin,
        constraint = market.load()?.close_market_admin == close_market_admin.key() @ OpenBookError::InvalidCloseMarketAdmin
    )]
    pub market: AccountLoader<'info, Market>,

    #[account(
        mut,
        close = sol_destination
    )]
    pub bids: AccountLoader<'info, BookSide>,

    #[account(
        mut,
        close = sol_destination
    )]
    pub asks: AccountLoader<'info, BookSide>,

    #[account(
        mut,
        close = sol_destination
    )]
    pub event_heap: AccountLoader<'info, EventHeap>,

    #[account(mut)]
    /// CHECK: target for account rent needs no checks
    pub sol_destination: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}
