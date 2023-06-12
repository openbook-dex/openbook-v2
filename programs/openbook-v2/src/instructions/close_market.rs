use crate::accounts_ix::*;
use crate::error::OpenBookError;
use anchor_lang::prelude::*;

pub fn close_market(ctx: Context<CloseMarket>) -> Result<()> {
    let market = ctx.accounts.market.load()?;
    let close_admin =
        Option::from(market.close_market_admin).ok_or(OpenBookError::NoCloseMarketAdmin)?;
    require!(
        ctx.accounts.close_market_admin.key() == close_admin,
        OpenBookError::InvalidCloseMarketAdmin
    );
    // check market is expired
    require!(
        market.time_expiry == -1 || market.time_expiry < Clock::get()?.unix_timestamp,
        OpenBookError::MarketHasNotExpired
    );

    let event_queue = ctx.accounts.event_queue.load()?;
    require!(
        event_queue.is_empty(),
        OpenBookError::EventQueueContainsElements
    );
    Ok(())
}
