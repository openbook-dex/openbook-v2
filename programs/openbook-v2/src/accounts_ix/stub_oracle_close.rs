use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{TokenInterface, self};

use crate::state::*;

#[derive(Accounts)]
pub struct StubOracleClose<'info> {
    pub owner: Signer<'info>,
    #[account(
        mut,
        has_one = owner,
        close = sol_destination
    )]
    pub oracle: AccountLoader<'info, StubOracle>,
    #[account(mut)]
    /// CHECK: target for account rent needs no checks
    pub sol_destination: UncheckedAccount<'info>,
    pub token_program: Interface<'info, TokenInterface>,
}
