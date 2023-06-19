use anchor_lang::prelude::*;

use crate::state::{Market, OpenOrdersAccount, OpenOrdersAccountFixed};

#[derive(Accounts)]
pub struct SetDelegate<'info> {
    #[account(
        mut,
        has_one = market,
    )]
    pub open_orders_account: AccountLoader<'info, OpenOrdersAccountFixed>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account()]
    pub delegate_account: Option<UncheckedAccount<'info>>,
    #[account(mut)]
    pub market: AccountLoader<'info, Market>,
}
