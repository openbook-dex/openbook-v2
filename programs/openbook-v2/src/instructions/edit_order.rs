use crate::accounts_ix::*;
use crate::error::*;
use crate::state::Order;
use anchor_lang::prelude::*;

pub fn edit_order<'c: 'info, 'info>(
    ctx: Context<'_, '_, 'c, 'info, PlaceOrder<'info>>,
    cancel_client_order_id: u64,
    expected_cancel_size: i64,
    mut order: Order,
    limit: u8,
) -> Result<Option<u128>> {
    require_gte!(
        expected_cancel_size,
        0,
        OpenBookError::InvalidInputCancelSize
    );

    let leaf_node_quantity = crate::instructions::cancel_order_by_client_order_id(
        Context::new(
            ctx.program_id,
            &mut ctx.accounts.to_cancel_order(),
            ctx.remaining_accounts,
            ctx.bumps.to_cancel_order(),
        ),
        cancel_client_order_id,
    )?;

    let filled_amount = expected_cancel_size - leaf_node_quantity;
    // note that order.max_base_lots is checked to be > 0 inside `place_order`
    if filled_amount > 0 && order.max_base_lots > filled_amount {
        // Do not reduce max_quote_lots_including_fees as implicitly it's limited by max_base_lots.
        order.max_base_lots -= filled_amount;
        return crate::instructions::place_order(ctx, order, limit);
    }
    Ok(None)
}
