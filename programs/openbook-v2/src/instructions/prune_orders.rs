use anchor_lang::prelude::*;

use crate::accounts_ix::*;
use crate::error::*;
use crate::logs::PruneOrdersLog;
use crate::state::*;

pub fn prune_orders(ctx: Context<PruneOrders>, limit: u8) -> Result<()> {
    let mut account = ctx.accounts.open_orders_account.load_mut()?;
    let market = ctx.accounts.market.load()?;

    require!(
        market.is_expired(Clock::get()?.unix_timestamp),
        OpenBookError::MarketHasNotExpired
    );

    let close_admin: Pubkey =
        Option::from(market.close_market_admin).ok_or(OpenBookError::NoCloseMarketAdmin)?;
    require!(
        ctx.accounts.close_market_admin.key() == close_admin,
        OpenBookError::InvalidCloseMarketAdmin
    );

    let mut book = Orderbook {
        bids: ctx.accounts.bids.load_mut()?,
        asks: ctx.accounts.asks.load_mut()?,
    };

    let quantity = book.cancel_all_orders(&mut account, *market, limit, None)?;

    emit!(PruneOrdersLog {
        open_orders_account: ctx.accounts.open_orders_account.key(),
        quantity,
        limit,
    });

    Ok(())
}
