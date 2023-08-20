use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CloseOpenOrdersAccount<'info> {
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [b"OpenOrdersIndexer".as_ref(), owner.key().as_ref()],
        bump = open_orders_indexer.bump,
    )]
    pub open_orders_indexer: Account<'info, OpenOrdersIndexer>,

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
