use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct SetMarketExpired<'info> {
    pub close_market_admin: Signer<'info>,
    #[account(mut)]
    pub market: AccountLoader<'info, Market>,
}
