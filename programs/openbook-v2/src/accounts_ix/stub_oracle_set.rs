use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct StubOracleSet<'info> {
    pub owner: Signer<'info>,
    #[account(
        mut,
        has_one = owner
    )]
    pub oracle: AccountLoader<'info, StubOracle>,
}
