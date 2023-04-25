use anchor_lang::prelude::*;
use anchor_spl::token::Token;

use crate::{error::OpenBookError, state::*};

#[derive(Accounts)]
pub struct CloseMarket<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        has_one = bids,
        has_one = asks,
        has_one = event_queue,
        close = sol_destination
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
    pub event_queue: AccountLoader<'info, EventQueue>,

    #[account(mut)]
    /// CHECK: target for account rent needs no checks
    pub sol_destination: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}
