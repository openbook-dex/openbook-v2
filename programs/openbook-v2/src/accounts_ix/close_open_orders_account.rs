use crate::error::OpenBookError;
use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CloseOpenOrdersAccount<'info> {
    pub signer: Signer<'info>,
    #[account(
        mut,
        close = sol_destination,
        constraint = open_orders_account.load()?.owner == signer.key() @ OpenBookError::NoOwner
    )]
    pub open_orders_account: AccountLoader<'info, OpenOrdersAccount>,

    #[account(mut)]
    /// CHECK: target for account rent needs no checks
    pub sol_destination: UncheckedAccount<'info>,
}
