use anchor_lang::prelude::*;

use crate::accounts_ix::*;
use crate::state::*;

pub fn cancel_order_by_client_order_id(
    ctx: Context<CancelOrder>,
    client_order_id: u64,
) -> Result<i64> {
    let mut account = ctx.accounts.open_orders_account.load_mut()?;

    let market = ctx.accounts.market.load()?;
    let mut book = Orderbook {
        bids: ctx.accounts.bids.load_mut()?,
        asks: ctx.accounts.asks.load_mut()?,
    };

    book.cancel_all_orders(&mut account, *market, u8::MAX, None, Some(client_order_id))
}
