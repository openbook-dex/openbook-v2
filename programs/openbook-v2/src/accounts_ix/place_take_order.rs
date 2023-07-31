use crate::error::OpenBookError;
use crate::pubkey_option::NonZeroKey;
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
        constraint = market.load()?.oracle_a == oracle_a.non_zero_key(),
        constraint = market.load()?.oracle_b == oracle_b.non_zero_key(),
        constraint = market.load()?.open_orders_admin == open_orders_admin.non_zero_key() @ OpenBookError::InvalidOpenOrdersAdmin
    )]
    pub market: AccountLoader<'info, Market>,
    #[account(
        seeds = [b"Market".as_ref(), market.key().to_bytes().as_ref()],
        bump,
    )]
    pub market_authority: AccountInfo<'info>,
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

    /// CHECK: The oracle can be one of several different account types and the pubkey is checked above
    pub oracle_a: Option<UncheckedAccount<'info>>,
    /// CHECK: The oracle can be one of several different account types and the pubkey is checked above
    pub oracle_b: Option<UncheckedAccount<'info>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub open_orders_admin: Option<Signer<'info>>,
}
