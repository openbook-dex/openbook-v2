use crate::error::OpenBookError;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

#[derive(Accounts)]
pub struct SweepFees<'info> {
    /// The market's `collect_fee_admin`, or the janitor sweeping on its behalf
    pub collect_fee_admin: Signer<'info>,
    #[account(
        mut,
        has_one = market_quote_vault,
        has_one = market_authority,
        constraint = market.load()?.collect_fee_admin == collect_fee_admin.key()
            || (collect_fee_admin.key() == JANITOR && janitor_active()?)
                @ ErrorCode::ConstraintHasOne
    )]
    pub market: AccountLoader<'info, Market>,
    /// CHECK: checked on has_one in market
    pub market_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub market_quote_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = market_quote_vault.mint,
        // the janitor may only sweep into an account of the collect_fee_admin
        constraint = market.load()?.collect_fee_admin == collect_fee_admin.key()
            || market.load()?.collect_fee_admin == token_receiver_account.owner
                @ OpenBookError::InvalidFeeReceiver
    )]
    pub token_receiver_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}
