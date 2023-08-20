use crate::accounts_ix::*;
use crate::error::OpenBookError;
use crate::state::*;
use anchor_lang::prelude::*;

pub fn close_market(ctx: Context<CloseMarket>) -> Result<()> {
    let market = ctx.accounts.market.load()?;
    require!(
        market.is_expired(Clock::get()?.unix_timestamp),
        OpenBookError::MarketHasNotExpired
    );
    require!(market.is_empty(), OpenBookError::NonEmptyMarket);

    let book = Orderbook {
        bids: ctx.accounts.bids.load_mut()?,
        asks: ctx.accounts.asks.load_mut()?,
    };
    require!(book.is_empty(), OpenBookError::BookContainsElements);

    let event_heap = ctx.accounts.event_heap.load()?;
    require!(
        event_heap.is_empty(),
        OpenBookError::EventHeapContainsElements
    );

    Ok(())
}
