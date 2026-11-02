use crate::error::OpenBookError;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

#[derive(Accounts)]
pub struct JanitorCloseMarket<'info> {
    #[account(
        address = JANITOR @ OpenBookError::InvalidJanitor,
        constraint = janitor_active()? @ OpenBookError::JanitorNotActive,
    )]
    pub janitor: Signer<'info>,
    #[account(
        mut,
        has_one = bids,
        has_one = asks,
        has_one = event_heap,
        has_one = market_base_vault,
        has_one = market_quote_vault,
        has_one = market_authority,
        has_one = base_mint,
        has_one = quote_mint,
        close = escrow,
    )]
    pub market: AccountLoader<'info, Market>,
    #[account(mut, close = escrow)]
    pub bids: AccountLoader<'info, BookSide>,
    #[account(mut, close = escrow)]
    pub asks: AccountLoader<'info, BookSide>,
    #[account(mut, close = escrow)]
    pub event_heap: AccountLoader<'info, EventHeap>,
    #[account(mut)]
    pub market_base_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub market_quote_vault: Box<Account<'info, TokenAccount>>,
    /// CHECK: checked on has_one in market
    pub market_authority: UncheckedAccount<'info>,
    /// CHECK: checked on has_one in market, what is left in the vault is burned
    #[account(mut)]
    pub base_mint: UncheckedAccount<'info>,
    /// CHECK: checked on has_one in market
    #[account(mut)]
    pub quote_mint: UncheckedAccount<'info>,
    /// CHECK: rent destination
    #[account(mut, address = ESCROW)]
    pub escrow: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}
