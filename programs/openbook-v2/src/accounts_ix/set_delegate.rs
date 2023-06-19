use anchor_lang::prelude::*;

use crate::state::OpenOrdersAccountFixed;

#[derive(Accounts)]
pub struct SetDelegate<'info> {
    #[account(
        mut,
        has_one = owner,
    )]
    pub open_orders_account: AccountLoader<'info, OpenOrdersAccountFixed>,
    #[account(mut)]
    pub owner: Signer<'info>,
    /// CHECK:
    pub delegate_account: Option<UncheckedAccount<'info>>,
}
