use crate::state::{Market, OpenOrdersAccount, OpenOrdersIndexer};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CreateOpenOrdersAccount<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub owner: Signer<'info>,
    /// CHECK:
    pub delegate_account: Option<UncheckedAccount<'info>>,
    #[account(
        mut,
        seeds = [b"OpenOrdersIndexer".as_ref(), owner.key().as_ref()],
        bump = open_orders_indexer.bump,
        realloc = OpenOrdersIndexer::space(open_orders_indexer.addresses.len()+1),
        realloc::payer = payer,
        realloc::zero = false,
        constraint = open_orders_indexer.addresses.len() < 256,
    )]
    pub open_orders_indexer: Account<'info, OpenOrdersIndexer>,
    #[account(
        init,
        seeds = [b"OpenOrders".as_ref(), owner.key().as_ref(), &(open_orders_indexer.created_counter + 1).to_le_bytes()],
        bump,
        payer = payer,
        space = OpenOrdersAccount::space(),
    )]
    pub open_orders_account: AccountLoader<'info, OpenOrdersAccount>,
    pub market: AccountLoader<'info, Market>,
    pub system_program: Program<'info, System>,
}
