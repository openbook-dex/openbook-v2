use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

#[derive(Accounts)]
pub struct PlaceMultipleOrders<'info> {
    #[account(
        mut,
        has_one = market,
        // also is_owner_or_delegate check inside ix
    )]
    pub open_orders_account: AccountLoader<'info, OpenOrdersAccount>,
    pub owner_or_delegate: Signer<'info>,
    pub open_orders_admin: Option<Signer<'info>>,

    #[account(
        mut,
        constraint = token_quote_deposit_account.mint == market_quote_vault.mint
    )]
    pub token_quote_deposit_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = token_base_deposit_account.mint == market_base_vault.mint
    )]
    pub token_base_deposit_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        has_one = bids,
        has_one = asks,
        has_one = event_queue,
    )]
    pub market: AccountLoader<'info, Market>,
    #[account(mut)]
    pub bids: AccountLoader<'info, BookSide>,
    #[account(mut)]
    pub asks: AccountLoader<'info, BookSide>,
    #[account(mut)]
    pub event_queue: AccountLoader<'info, EventQueue>,
    #[account(
        mut,
        constraint = market_quote_vault.key() == market.load()?.quote_vault
    )]
    pub market_quote_vault: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = market_quote_vault.key() == market.load()?.base_vault
    )]
    pub market_base_vault: Box<Account<'info, TokenAccount>>,

    /// CHECK: The oracle can be one of several different account types and the pubkey is checked
    #[account(constraint = market.load()?.oracle == oracle.key())]
    pub oracle: Option<UncheckedAccount<'info>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
