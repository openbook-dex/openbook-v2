use crate::state::OpenOrdersIndexer;
use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{TokenInterface, self};

#[derive(Accounts)]
pub struct CloseOpenOrdersIndexer<'info> {
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [b"OpenOrdersIndexer".as_ref(), owner.key().as_ref()],
        bump = open_orders_indexer.bump,
        close = sol_destination
    )]
    pub open_orders_indexer: Account<'info, OpenOrdersIndexer>,

    #[account(mut)]
    /// CHECK: target for account rent needs no checks
    pub sol_destination: UncheckedAccount<'info>,
    pub token_program: Interface<'info, TokenInterface>,
}
