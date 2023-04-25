use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

#[derive(Accounts)]
pub struct Deposit<'info> {
    pub owner: Signer<'info>,
    #[account(mut)]
    pub payer_base: Account<'info, TokenAccount>,
    #[account(mut)]
    pub payer_quote: Account<'info, TokenAccount>,
    #[account(mut)]
    pub open_orders_account: AccountLoader<'info, OpenOrdersAccountFixed>,

    pub market: AccountLoader<'info, Market>,

    #[account(mut)]
    pub base_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub quote_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
