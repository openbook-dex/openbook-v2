use crate::accounts_ix::*;
use crate::state::Order;
use anchor_lang::prelude::*;

pub fn edit_order<'info>(
    ctx: Context<'_, '_, '_, 'info, PlaceOrder<'info>>,
    cancel_client_order_id: u64,
    expected_cancel_size: i64,
    mut order: Order,
    price: i64,
    limit: u8,
) -> Result<Option<u128>> {
    let leaf_node_quantity = crate::instructions::cancel_order_by_client_order_id(
        Context::new(
            ctx.program_id,
            &mut ctx.accounts.to_cancel_order(),
            ctx.remaining_accounts,
            ctx.bumps.clone(),
        ),
        cancel_client_order_id,
    )?;

    let market = ctx.accounts.market.load()?;
    let filled_amount = expected_cancel_size - leaf_node_quantity;
    if filled_amount > 0 {
        order.max_base_lots -= filled_amount;
        let new_max_quote_lots_before_fees = order.max_base_lots * price;
        order.max_quote_lots_including_fees =
            market.subtract_taker_fees(new_max_quote_lots_before_fees)
    }
    drop(market);

    crate::instructions::place_order(ctx, order, limit)
}
