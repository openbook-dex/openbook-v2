use crate::accounts_ix::*;
use crate::error::OpenBookError;
use anchor_lang::prelude::*;

pub fn set_market_expired(ctx: Context<SetMarketExpired>) -> Result<()> {
    let mut market = ctx.accounts.market.load_mut()?;
    require!(
        market.time_expiry == 0 || market.time_expiry > Clock::get()?.unix_timestamp,
        OpenBookError::MarketHasExpired
    );
    // Only markets with open_orders_admin
    let open_orders_admin = Option::<Pubkey>::from(market.open_orders_admin)
        .ok_or(OpenBookError::InvalidOpenOrdersAdmin)?;
    require_eq!(
        open_orders_admin,
        *ctx.accounts.open_orders_admin.key,
        OpenBookError::InvalidOpenOrdersAdmin
    );

    market.time_expiry = -1;

    Ok(())
}
