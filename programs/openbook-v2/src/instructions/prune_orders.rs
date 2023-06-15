use anchor_lang::prelude::*;

use crate::accounts_ix::*;
use crate::error::*;
use crate::state::*;

pub fn prune_orders(ctx: Context<PruneOrders>, limit: u8) -> Result<()> {
    let mut account = ctx.accounts.open_orders_account.load_full_mut()?;
    let market = ctx.accounts.market.load()?;

    // check market is expired
    require!(
        market.time_expiry == -1 || market.time_expiry < Clock::get()?.unix_timestamp,
        OpenBookError::MarketHasNotExpired
    );
    let close_admin =
        Option::from(market.close_market_admin).ok_or(OpenBookError::NoCloseMarketAdmin)?;
    require!(
        ctx.accounts.close_market_admin.key() == close_admin,
        OpenBookError::InvalidCloseMarketAdmin
    );

    let mut book = Orderbook {
        bids: ctx.accounts.bids.load_mut()?,
        asks: ctx.accounts.asks.load_mut()?,
    };

    book.cancel_all_orders(&mut account.borrow_mut(), *market, limit, None)?;

    Ok(())
}
