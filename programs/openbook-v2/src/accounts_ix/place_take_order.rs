use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

#[derive(Accounts)]
pub struct PlaceTakeOrder<'info> {
    pub signer: Signer<'info>,

    #[account(
        mut,
        has_one = bids,
        has_one = asks,
        has_one = event_queue,
        has_one = base_vault,
        has_one = quote_vault,
    )]
    pub market: AccountLoader<'info, Market>,
    #[account(mut)]
    pub bids: AccountLoader<'info, BookSide>,
    #[account(mut)]
    pub asks: AccountLoader<'info, BookSide>,
    #[account(mut)]
    pub base_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub quote_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub event_queue: AccountLoader<'info, EventQueue>,

    #[account(
        mut,
        token::authority = signer.key(),
        constraint = token_deposit_account.mint == base_vault.mint || token_deposit_account.mint == quote_vault.mint
    )]
    pub token_deposit_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = token_deposit_account.mint == base_vault.mint || token_deposit_account.mint == quote_vault.mint
    )]
    pub token_receiver_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = quote_vault.mint
    )]
    pub referrer: Option<Box<Account<'info, TokenAccount>>>,

    /// CHECK: The oracle can be one of several different account types and the pubkey is checked
    #[account(constraint = market.load()?.oracle == oracle.key())]
    pub oracle: Option<UncheckedAccount<'info>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub open_orders_admin: Option<Signer<'info>>,
}
