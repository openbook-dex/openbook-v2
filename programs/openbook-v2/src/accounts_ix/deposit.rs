use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

#[derive(Accounts)]
#[instruction(account_num: u32)]
pub struct Deposit<'info> {
    pub owner: Signer<'info>,
    #[account(mut)]
    pub token_base_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub token_quote_account: Account<'info, TokenAccount>,
    #[account(mut,
        has_one = market,
    )]
    pub open_orders_account: AccountLoader<'info, OpenOrdersAccountFixed>,
    #[account(mut)]
    pub market: AccountLoader<'info, Market>,

    #[account(mut)]
    pub base_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub quote_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
