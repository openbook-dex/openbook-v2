use crate::error::OpenBookError;
use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct JanitorCloseOpenOrdersIndexer<'info> {
    #[account(
        mut,
        address = JANITOR @ OpenBookError::InvalidJanitor,
        constraint = janitor_active()? @ OpenBookError::JanitorNotActive,
    )]
    pub janitor: Signer<'info>,
    /// CHECK: receives the indexer's lamports, checked by the indexer seeds
    #[account(mut)]
    pub owner: UncheckedAccount<'info>,
    /// An indexer does not store its owner, its address is only derived from it: the janitor
    /// finds the owner off-chain (transaction history), the seeds prove it.
    #[account(
        mut,
        seeds = [b"OpenOrdersIndexer".as_ref(), owner.key().as_ref()],
        bump = open_orders_indexer.bump,
    )]
    pub open_orders_indexer: Account<'info, OpenOrdersIndexer>,
}
