use anchor_lang::prelude::*;

use crate::accounts_ix::*;

pub fn stub_oracle_create(ctx: Context<StubOracleCreate>, price: f64) -> Result<()> {
    let mut oracle = ctx.accounts.oracle.load_init()?;
    oracle.owner = ctx.accounts.owner.key();
    oracle.mint = ctx.accounts.mint.key();
    oracle.price = price;
    oracle.last_update_ts = Clock::get()?.unix_timestamp;

    Ok(())
}
