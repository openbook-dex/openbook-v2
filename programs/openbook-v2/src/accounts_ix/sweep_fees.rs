use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

#[derive(Accounts)]
pub struct SweepFees<'info> {
    pub collect_fee_admin: Signer<'info>,
    #[account(
        mut,
        has_one = quote_vault,
        has_one = collect_fee_admin
    )]
    pub market: AccountLoader<'info, Market>,
    #[account(
        seeds = [b"Market".as_ref(), market.key().to_bytes().as_ref()],
        bump,
    )]
    pub market_authority: AccountInfo<'info>,
    #[account(mut)]
    pub quote_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = quote_vault.mint
    )]
    pub token_receiver_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
