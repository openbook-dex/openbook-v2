use crate::accounts_ix::*;
use crate::error::OpenBookError;
use crate::state::*;
use anchor_lang::prelude::*;

pub fn close_market(ctx: Context<CloseMarket>) -> Result<()> {
    let market = ctx.accounts.market.load()?;
    let close_admin: Pubkey =
        Option::from(market.close_market_admin).ok_or(OpenBookError::NoCloseMarketAdmin)?;
    require!(
        ctx.accounts.close_market_admin.key() == close_admin,
        OpenBookError::InvalidCloseMarketAdmin
    );
    require!(
        market.is_expired(Clock::get()?.unix_timestamp),
        OpenBookError::MarketHasNotExpired
    );

    let book = Orderbook {
        bids: ctx.accounts.bids.load_mut()?,
        asks: ctx.accounts.asks.load_mut()?,
    };

    require!(book.is_empty(), OpenBookError::BookContainsElements);

    let event_queue = ctx.accounts.event_queue.load()?;
    require!(
        event_queue.is_empty(),
        OpenBookError::EventQueueContainsElements
    );
    Ok(())
}
