use anchor_lang::prelude::*;

use crate::accounts_ix::*;
use crate::error::*;
use crate::state::*;

pub fn cancel_order(ctx: Context<CancelOrder>, order_id: u128) -> Result<()> {
    require_gt!(order_id, 0, OpenBookError::InvalidInputOrderId);

    let mut account = ctx.accounts.open_orders_account.load_full_mut()?;
    // account constraint #1
    require!(
        account.fixed.is_owner_or_delegate(ctx.accounts.owner.key()),
        OpenBookError::NoOwnerOrDelegate
    );

    let market = ctx.accounts.market.load()?;
    let mut book = Orderbook {
        bids: ctx.accounts.bids.load_mut()?,
        asks: ctx.accounts.asks.load_mut()?,
    };

    let oo = account.find_order_with_order_id(order_id).ok_or_else(|| {
        error_msg_typed!(OpenBookError::OpenOrdersOrderNotFound, "id = {order_id}")
    })?;

    let order_id = oo.id;
    let order_side_and_tree = oo.side_and_tree();

    book.cancel_order(
        &mut account.borrow_mut(),
        order_id,
        order_side_and_tree,
        *market,
        Some(ctx.accounts.open_orders_account.key()),
    )?;

    Ok(())
}
