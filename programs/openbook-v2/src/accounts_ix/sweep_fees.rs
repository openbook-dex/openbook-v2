use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

#[derive(Accounts)]
pub struct SweepFees<'info> {
    pub collect_fee_admin: Signer<'info>,

    #[account(mut, has_one = collect_fee_admin)]
    pub market: AccountLoader<'info, Market>,

    #[account(mut, constraint = token_receiver_account.owner == collect_fee_admin.key())]
    pub token_receiver_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub quote_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
