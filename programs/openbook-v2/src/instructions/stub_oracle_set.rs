use crate::accounts_ix::*;
use anchor_lang::prelude::*;

require!(
oracle_account.owner == &pyth_program::ID,
ErrorCode::InvalidOracle
);
    let clock = Clock::get()?;
    let mut oracle = ctx.accounts.oracle.load_mut()?;

    oracle.price = price;
    oracle.last_update_ts = clock.unix_timestamp;
    oracle.last_update_slot = clock.slot;

    Ok(())
}
