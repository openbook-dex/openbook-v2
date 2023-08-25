use crate::accounts_ix::*;
use anchor_lang::prelude::*;

pub fn stub_oracle_set(ctx: Context<StubOracleSet>, price: f64) -> Result<()> {
    let clock = Clock::get()?;
    let mut oracle = ctx.accounts.oracle.load_mut()?;

    oracle.price = price;
    oracle.last_update_ts = clock.unix_timestamp;
    oracle.last_update_slot = clock.slot;

    Ok(())
}
