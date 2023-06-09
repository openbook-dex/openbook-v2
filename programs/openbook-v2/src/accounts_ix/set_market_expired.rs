use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct SetMarketExpired<'info> {
    pub open_orders_admin: Signer<'info>,
    #[account(mut)]
    pub market: AccountLoader<'info, Market>,
}
