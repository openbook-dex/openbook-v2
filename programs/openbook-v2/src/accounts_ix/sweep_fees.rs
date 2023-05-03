use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

#[derive(Accounts)]
pub struct SweepFees<'info> {
    #[account(mut)]
    pub market: AccountLoader<'info, Market>,

    #[account(mut)]
    pub receiver: Account<'info, TokenAccount>,

    #[account(mut)]
    pub base_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub quote_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
