use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

#[derive(Accounts)]
pub struct SweepFees<'info> {
    pub collect_fee_admin: Signer<'info>,
    #[account(
        mut,
        has_one = market_quote_vault,
        has_one = collect_fee_admin,
        has_one = market_authority
    )]
    pub market: AccountLoader<'info, Market>,
    /// CHECK: checked on has_one in market
    pub market_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub market_quote_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = market_quote_vault.mint
    )]
    pub token_receiver_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}
