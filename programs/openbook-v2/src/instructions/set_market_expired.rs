use crate::accounts_ix::*;
use crate::error::OpenBookError;
use anchor_lang::prelude::*;

pub fn set_market_expired(ctx: Context<SetMarketExpired>) -> Result<()> {
    let mut market = ctx.accounts.market.load_mut()?;
    require!(
        !market.is_expired(Clock::get()?.unix_timestamp),
        OpenBookError::MarketHasExpired
    );

    market.time_expiry = -1;

    Ok(())
}
