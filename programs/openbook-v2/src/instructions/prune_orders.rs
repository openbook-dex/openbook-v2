use std::cmp;

use anchor_lang::prelude::*;

use anchor_spl::token::{self, Transfer};
use fixed::types::I80F48;

use crate::accounts_ix::*;
use crate::accounts_zerocopy::*;
use crate::error::*;
use crate::state::*;

// TODO
#[allow(clippy::too_many_arguments)]
pub fn prune_orders(ctx: Context<PruneOrders>, order: Order, limit: u8) -> Result<()> {
    let mut account = ctx.accounts.open_orders_account.load_full_mut()?;
    // no account.fixed.is_owner_or_delegate constraint
    // check market 
    require!(
        market.time_expiry == -1 || market.time_expiry < Clock::get()?.unix_timestamp,
        OpenBookError::MarketHasNotExpired
    );
    let market = ctx.accounts.market.load()?;
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