use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CloseOpenOrdersAccount<'info> {
    pub owner: Signer<'info>,
    #[account(
        mut,
        has_one = owner,
        constraint = open_orders_indexer.load()?.market == open_orders_account.load()?.market
    )]
    pub open_orders_indexer: AccountLoader<'info, OpenOrdersIndexer>,

    #[account(
        mut,
        has_one = owner,
        close = sol_destination,
    )]
    pub open_orders_account: AccountLoader<'info, OpenOrdersAccount>,
    #[account(mut)]
    /// CHECK: target for account rent needs no checks
    pub sol_destination: UncheckedAccount<'info>,
}
