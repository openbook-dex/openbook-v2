use anchor_lang::prelude::*;

use crate::accounts_ix::*;
use crate::error::*;
use crate::state::*;

pub fn cancel_order_by_client_order_id(
    ctx: Context<CancelOrder>,
    client_order_id: u64,
) -> Result<()> {
    let mut account = ctx.accounts.open_orders_account.load_mut()?;
    require!(
        account.is_owner_or_delegate(ctx.accounts.owner.key()),
        OpenBookError::SomeError
    );

    let market = ctx.accounts.market.load()?;
    let mut book = Orderbook {
        bids: ctx.accounts.bids.load_mut()?,
        asks: ctx.accounts.asks.load_mut()?,
    };

    let oo = account
        .find_order_with_client_order_id(client_order_id)
        .ok_or_else(|| {
            error_msg_typed!(
                OpenBookError::OpenOrdersOrderNotFound,
                "client order id = {client_order_id}"
            )
        })?;

    let order_id = oo.id;
    let order_side_and_tree = oo.side_and_tree();

    book.cancel_order(
        &mut account,
        order_id,
        order_side_and_tree,
        *market,
        Some(ctx.accounts.open_orders_account.key()),
    )?;

    Ok(())
}
