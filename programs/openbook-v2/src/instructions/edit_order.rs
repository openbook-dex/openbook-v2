use crate::accounts_ix::*;
use crate::error::*;
use crate::state::{Order, Orderbook};
use anchor_lang::prelude::*;

pub fn edit_order<'info>(
    ctx: Context<'_, '_, '_, 'info, PlaceOrder<'info>>,
    cancel_client_order_id: u64,
    expected_cancel_size: i64,
    mut order: Order,
    price: i64,
    limit: u8,
) -> Result<Option<u128>> {
    let account = ctx.accounts.open_orders_account.load()?;
    let market = ctx.accounts.market.load()?;

    let book = Orderbook {
        bids: ctx.accounts.bids.load_mut()?,
        asks: ctx.accounts.asks.load_mut()?,
    };

    let oo = account
        .find_order_with_client_order_id(cancel_client_order_id)
        .ok_or_else(|| {
            error_msg_typed!(
                OpenBookError::OpenOrdersOrderNotFound,
                "client order id = {cancel_client_order_id}"
            )
        })?;
    let leaf_node_quantity = book.get_order(oo.id, oo.side_and_tree())?.quantity;

    let filled_amount = expected_cancel_size - leaf_node_quantity;
    if filled_amount > 0 {
        order.max_base_lots -= filled_amount;
        let new_max_quote_lots_before_fees = order.max_base_lots * price;
        order.max_quote_lots_including_fees =
            market.subtract_taker_fees(new_max_quote_lots_before_fees)
    }
    drop(account);
    drop(market);
    drop(book);

    crate::instructions::cancel_order_by_client_order_id(
        Context::new(
            ctx.program_id,
            &mut ctx.accounts.to_cancel_order(),
            ctx.remaining_accounts,
            ctx.bumps.clone(),
        ),
        cancel_client_order_id,
    )?;

    return crate::instructions::place_order(ctx, order, limit);
}
