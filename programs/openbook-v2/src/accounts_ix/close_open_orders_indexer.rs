use crate::state::{Market, OpenOrdersIndexer};
use anchor_lang::prelude::*;
use anchor_spl::token::Token;

#[derive(Accounts)]
pub struct CloseOpenOrdersIndexer<'info> {
    pub owner: Signer<'info>,
    #[account(
        mut,
        has_one = owner,
        has_one = market,
        close = sol_destination
    )]
    pub open_orders_indexer: AccountLoader<'info, OpenOrdersIndexer>,
    pub market: AccountLoader<'info, Market>,

    #[account(mut)]
    /// CHECK: target for account rent needs no checks
    pub sol_destination: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}
