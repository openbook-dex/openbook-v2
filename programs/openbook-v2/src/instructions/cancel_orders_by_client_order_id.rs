use anchor_lang::prelude::*;

use crate::accounts_ix::*;
use crate::logs::CancelOrderLog;
use crate::state::*;

pub fn cancel_orders_by_client_order_id(
    ctx: Context<CancelOrder>,
    client_order_id: u64,
) -> Result<()> {
    let open_orders_account = ctx.accounts.open_orders_account.load()?;
    let open_orders_vec = open_orders_account.find_orders_with_client_order_id(client_order_id);

    let market = ctx.accounts.market.load()?;
    let mut book = Orderbook {
        bids: ctx.accounts.bids.load_mut()?,
        asks: ctx.accounts.asks.load_mut()?,
    };

    for oo in open_orders_vec.iter() {
        let order_id = oo.id;
        let order_side_and_tree = oo.side_and_tree();
        let mut open_orders_account = ctx.accounts.open_orders_account.load_mut()?;

        let leaf_node = book.cancel_order(
            &mut open_orders_account,
            order_id,
            order_side_and_tree,
            *market,
            Some(ctx.accounts.open_orders_account.key()),
        )?;
        emit!(CancelOrderLog {
            open_orders_account: ctx.accounts.open_orders_account.key(),
            slot: leaf_node.owner_slot,
            side: order_side_and_tree.side().into(),
            quantity: leaf_node.quantity,
        });
    }

    Ok(())
}
