use anchor_lang::prelude::*;

use crate::accounts_ix::*;
use crate::state::*;

pub fn cancel_all_orders(
    ctx: Context<CancelOrder>,
    side_option: Option<Side>,
    limit: u8,
) -> Result<()> {
    let mut account = ctx.accounts.open_orders_account.load_mut()?;

    let market = ctx.accounts.market.load()?;
    let mut book = Orderbook {
        bids: ctx.accounts.bids.load_mut()?,
        asks: ctx.accounts.asks.load_mut()?,
    };

    book.cancel_all_orders(&mut account, *market, limit, side_option, None)?;

    Ok(())
}
