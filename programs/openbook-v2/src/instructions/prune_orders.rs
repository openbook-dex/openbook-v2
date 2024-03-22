use anchor_lang::prelude::*;

use crate::accounts_ix::*;
use crate::error::*;
use crate::state::*;

pub fn prune_orders(ctx: Context<PruneOrders>) -> Result<()> {
    let mut account = ctx.accounts.open_orders_account.load_mut()?;
    let market = ctx.accounts.market.load()?;

    require!(
        market.is_expired(Clock::get()?.unix_timestamp),
        OpenBookError::MarketHasNotExpired
    );

    let mut book = Orderbook {
        bids: ctx.accounts.bids.load_mut()?,
        asks: ctx.accounts.asks.load_mut()?,
    };

    book.cancel_all_orders(&mut account, *market, u8::MAX, None, None)?;

    Ok(())
}
