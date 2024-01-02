use crate::error::OpenBookError;
use crate::pubkey_option::NonZeroKey;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

#[derive(Accounts)]
pub struct PlaceTakeOrder<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub penalty_payer: Signer<'info>,

    #[account(
        mut,
        has_one = bids,
        has_one = asks,
        has_one = event_heap,
        has_one = market_base_vault,
        has_one = market_quote_vault,
        has_one = market_authority,
        constraint = market.load()?.oracle_a == oracle_a.non_zero_key(),
        constraint = market.load()?.oracle_b == oracle_b.non_zero_key(),
        constraint = market.load()?.open_orders_admin == open_orders_admin.non_zero_key() @ OpenBookError::InvalidOpenOrdersAdmin
    )]
    pub market: AccountLoader<'info, Market>,
    /// CHECK: checked on has_one in market
    pub market_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub bids: AccountLoader<'info, BookSide>,
    #[account(mut)]
    pub asks: AccountLoader<'info, BookSide>,
    #[account(mut)]
    pub market_base_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub market_quote_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub event_heap: AccountLoader<'info, EventHeap>,

    #[account(
        mut,
        token::mint = market_base_vault.mint
    )]
    pub user_base_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        token::mint = market_quote_vault.mint
    )]
    pub user_quote_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: The oracle can be one of several different account types and the pubkey is checked above
    pub oracle_a: Option<UncheckedAccount<'info>>,
    /// CHECK: The oracle can be one of several different account types and the pubkey is checked above
    pub oracle_b: Option<UncheckedAccount<'info>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub open_orders_admin: Option<Signer<'info>>,
}
