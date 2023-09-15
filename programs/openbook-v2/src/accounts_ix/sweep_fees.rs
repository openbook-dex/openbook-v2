use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use anchor_spl::token_interface::{TokenInterface, self};


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
    pub market_quote_vault: InterfaceAccount<'info, token_interface::TokenAccount>,

    #[account(
        mut,
        token::mint = market_quote_vault.mint
    )]
    pub token_receiver_account: InterfaceAccount<'info, token_interface::TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
}
