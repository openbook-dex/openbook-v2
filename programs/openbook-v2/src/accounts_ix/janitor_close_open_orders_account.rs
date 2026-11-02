use crate::error::OpenBookError;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::Token};

#[derive(Accounts)]
pub struct JanitorCloseOpenOrdersAccount<'info> {
    #[account(
        mut, address = JANITOR @ OpenBookError::InvalidJanitor,
        constraint = janitor_active()? @ OpenBookError::JanitorNotActive,
    )]
    pub janitor: Signer<'info>,
    /// CHECK: receives the remaining rent, checked by has_one on the open orders account
    #[account(mut)]
    pub owner: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"OpenOrdersIndexer".as_ref(), owner.key().as_ref()],
        bump = open_orders_indexer.bump,
    )]
    pub open_orders_indexer: Account<'info, OpenOrdersIndexer>,
    #[account(
        mut,
        has_one = owner,
        has_one = market,
    )]
    pub open_orders_account: AccountLoader<'info, OpenOrdersAccount>,
    /// CHECK: may already be closed, then settlement is skipped; loaded and checked in the handler
    #[account(mut)]
    pub market: UncheckedAccount<'info>,
    /// CHECK: checked against the market in the handler
    pub market_authority: UncheckedAccount<'info>,
    /// CHECK: checked against the market in the handler
    #[account(mut)]
    pub market_base_vault: UncheckedAccount<'info>,
    /// CHECK: checked against the market in the handler
    #[account(mut)]
    pub market_quote_vault: UncheckedAccount<'info>,
    /// CHECK: must be the owner's associated token account, created by the handler if needed
    #[account(mut)]
    pub owner_base_ata: UncheckedAccount<'info>,
    /// CHECK: must be the owner's associated token account, created by the handler if needed
    #[account(mut)]
    pub owner_quote_ata: UncheckedAccount<'info>,
    /// CHECK: checked against the market in the handler
    pub base_mint: UncheckedAccount<'info>,
    /// CHECK: checked against the market in the handler
    pub quote_mint: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
