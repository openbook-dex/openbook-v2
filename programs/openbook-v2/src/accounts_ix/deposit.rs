use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

#[derive(Accounts)]
pub struct Deposit<'info> {
    pub owner: Signer<'info>,
    #[account(
        mut,
        token::mint = market_base_vault.mint
    )]
    pub user_base_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = market_quote_vault.mint
    )]
    pub user_quote_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        has_one = market,
    )]
    pub open_orders_account: AccountLoader<'info, OpenOrdersAccount>,

    #[account(
        mut,
        has_one = market_base_vault,
        has_one = market_quote_vault,
    )]
    pub market: AccountLoader<'info, Market>,
    #[account(mut)]
    pub market_base_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub market_quote_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}
