use crate::state::OpenOrdersIndexer;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CreateOpenOrdersIndexer<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub owner: Signer<'info>,
    #[account(
        init,
        seeds = [b"OpenOrdersIndexer".as_ref(), owner.key().as_ref()],
        bump,
        payer = payer,
        space = OpenOrdersIndexer::space(0),
    )]
    pub open_orders_indexer: Account<'info, OpenOrdersIndexer>,
    pub system_program: Program<'info, System>,
}
