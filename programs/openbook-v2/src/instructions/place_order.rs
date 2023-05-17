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
pub fn place_order(ctx: Context<PlaceOrder>, order: Order, limit: u8) -> Result<Option<u128>> {
    require_gte!(order.max_base_lots, 0);
    require_gte!(order.max_quote_lots_including_fees, 0);

    let mut open_orders_account = ctx.accounts.open_orders_account.load_full_mut()?;
    // account constraint #1
    require!(
        open_orders_account
            .fixed
            .is_owner_or_delegate(ctx.accounts.owner.key()),
        OpenBookError::SomeError
    );
    let open_orders_account_pk = ctx.accounts.open_orders_account.key();

    let mut market = ctx.accounts.market.load_mut()?;
    let mut book = Orderbook {
        bids: ctx.accounts.bids.load_mut()?,
        asks: ctx.accounts.asks.load_mut()?,
    };
    let mut event_queue = ctx.accounts.event_queue.load_mut()?;

    let now_ts: u64 = Clock::get()?.unix_timestamp.try_into().unwrap();
    let oracle_price = market.oracle_price(
        &AccountInfoRef::borrow(ctx.accounts.oracle.as_ref())?,
        Clock::get()?.slot,
    )?;

    let OrderWithAmounts {
        order_id,
        total_base_taken_native,
        total_quote_taken_native,
        placed_quantity,
        maker_fees,
        ..
    } = book.new_order(
        ctx.remaining_accounts,
        &order,
        &mut market,
        &mut event_queue,
        oracle_price,
        Some(open_orders_account.borrow_mut()),
        &open_orders_account_pk,
        now_ts,
        limit,
    )?;

    let position = &mut open_orders_account.fixed_mut().position;
    let (to_vault, deposit_amount) = match order.side {
        Side::Bid => {
            let free_quote = position.quote_free_native;

            let max_quote_including_fees = if let Some(order_id) = order_id {
                let price = match order.params {
                    OrderParams::OraclePegged { peg_limit, .. } => I80F48::from(peg_limit),
                    OrderParams::Fixed { .. } => I80F48::from((order_id >> 64) as u64),
                    _ => unreachable!(),
                };
                total_quote_taken_native
                    + I80F48::from_num(placed_quantity)
                        * I80F48::from_num(market.quote_lot_size)
                        * price
                    + maker_fees
            } else {
                total_quote_taken_native
            };

            let free_qty_to_lock = cmp::min(max_quote_including_fees, free_quote);
            position.quote_free_native -= free_qty_to_lock;

            // Update market deposit total
            market.quote_deposit_total += ((max_quote_including_fees - free_qty_to_lock)
                - (total_quote_taken_native * (market.taker_fee - market.maker_fee)))
                .to_num::<u64>();

            (
                ctx.accounts.quote_vault.to_account_info(),
                max_quote_including_fees - free_qty_to_lock,
            )
        }

        Side::Ask => {
            let free_assets_native = position.base_free_native;
            let max_base_native = total_base_taken_native
                + I80F48::from_num(placed_quantity) * I80F48::from_num(market.base_lot_size);

            let free_qty_to_lock = cmp::min(max_base_native, free_assets_native);
            position.base_free_native -= free_qty_to_lock;

            // Update market deposit total
            market.base_deposit_total += (max_base_native - free_qty_to_lock).to_num::<u64>();

            (
                ctx.accounts.base_vault.to_account_info(),
                max_base_native - free_qty_to_lock,
            )
        }
    };

    // Transfer funds
    if deposit_amount > 0 {
        let cpi_context = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.payer.to_account_info(),
                to: to_vault,
                authority: ctx.accounts.owner.to_account_info(),
            },
        );
        // TODO Binye check if this is correct
        token::transfer(cpi_context, deposit_amount.ceil().to_num())?;
    }
    Ok(order_id)
}
