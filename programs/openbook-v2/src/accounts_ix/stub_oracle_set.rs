use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct StubOracleSet<'info> {
    pub admin: Signer<'info>,
    #[account(mut)]
    pub oracle: AccountLoader<'info, StubOracle>,
}
