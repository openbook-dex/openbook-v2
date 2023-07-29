use crate::state::{Market, OpenOrdersIndexer};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CreateOpenOrdersIndexer<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub owner: Signer<'info>,
    #[account(
        init,
        seeds = [b"OpenOrdersIndexer".as_ref(), owner.key().as_ref(), market.key().as_ref()],
        bump,
        payer = payer,
        space = OpenOrdersIndexer::space(),
    )]
    pub open_orders_indexer: AccountLoader<'info, OpenOrdersIndexer>,
    pub market: AccountLoader<'info, Market>,
    pub system_program: Program<'info, System>,
}
