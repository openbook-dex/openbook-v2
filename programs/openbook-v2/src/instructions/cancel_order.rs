use anchor_lang::prelude::*;

use crate::accounts_ix::*;
use crate::error::*;
use crate::state::*;

pub fn cancel_order(ctx: Context<CancelOrder>, order_id: u128) -> Result<()> {
    require_gt!(order_id, 0, OpenBookError::InvalidInputOrderId);

    let mut open_orders_account = ctx.accounts.open_orders_account.load_mut()?;
    let oo = open_orders_account
        .find_order_with_order_id(order_id)
        .ok_or_else(|| {
            error_msg_typed!(OpenBookError::OpenOrdersOrderNotFound, "id = {order_id}")
        })?;

    let order_id = oo.id;
    let order_side_and_tree = oo.side_and_tree();

    let market = ctx.accounts.market.load()?;
    let mut book = Orderbook {
        bids: ctx.accounts.bids.load_mut()?,
        asks: ctx.accounts.asks.load_mut()?,
    };

    book.cancel_order(
        &mut open_orders_account,
        order_id,
        order_side_and_tree,
        *market,
        Some(ctx.accounts.open_orders_account.key()),
    )?;

    Ok(())
}
