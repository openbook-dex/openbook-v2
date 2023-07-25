use crate::accounts_ix::*;
use crate::error::OpenBookError;
use anchor_lang::prelude::*;

pub fn set_market_expired(ctx: Context<SetMarketExpired>) -> Result<()> {
    let mut market = ctx.accounts.market.load_mut()?;
    require!(
        !market.is_expired(Clock::get()?.unix_timestamp),
        OpenBookError::MarketHasExpired
    );
    // Only markets with close_admin
    let close_market_admin = Option::<Pubkey>::from(market.close_market_admin)
        .ok_or(OpenBookError::InvalidOpenOrdersAdmin)?;
    require_eq!(
        close_market_admin,
        *ctx.accounts.close_market_admin.key,
        OpenBookError::InvalidOpenOrdersAdmin
    );

    market.time_expiry = -1;

    Ok(())
}
