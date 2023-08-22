use crate::accounts_ix::*;
use anchor_lang::prelude::*;

pub fn stub_oracle_set(ctx: Context<StubOracleSet>, price: f64) -> Result<()> {
    let mut oracle = ctx.accounts.oracle.load_mut()?;
    oracle.price = price;
    oracle.last_update_ts = Clock::get()?.unix_timestamp;

    Ok(())
}
