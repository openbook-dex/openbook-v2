use anchor_lang::prelude::*;
use std::cmp;

use crate::accounts_ix::*;
use crate::accounts_zerocopy::*;
use crate::error::*;
use crate::state::*;
use crate::token_utils::*;

#[allow(clippy::too_many_arguments)]
pub fn place_order(ctx: Context<PlaceOrder>, order: Order, limit: u8) -> Result<Option<u128>> {
    require_gte!(order.max_base_lots, 0, OpenBookError::InvalidInputLots);
    require_gte!(
        order.max_quote_lots_including_fees,
        0,
        OpenBookError::InvalidInputLots
    );

    let mut open_orders_account = ctx.accounts.open_orders_account.load_mut()?;
    let open_orders_account_pk = ctx.accounts.open_orders_account.key();

    let clock = Clock::get()?;

    let mut market = ctx.accounts.market.load_mut()?;
    require!(
        !market.is_expired(clock.unix_timestamp),
        OpenBookError::MarketHasExpired
    );

    let mut book = Orderbook {
        bids: ctx.accounts.bids.load_mut()?,
        asks: ctx.accounts.asks.load_mut()?,
    };
    let mut event_heap = ctx.accounts.event_heap.load_mut()?;
    let event_heap_size_before = event_heap.len();

    let now_ts: u64 = clock.unix_timestamp.try_into().unwrap();
    let oracle_price = if market.oracle_a.is_some() && market.oracle_b.is_some() {
        Some(market.oracle_price_from_a_and_b(
            &AccountInfoRef::borrow(ctx.accounts.oracle_a.as_ref().unwrap())?,
            &AccountInfoRef::borrow(ctx.accounts.oracle_b.as_ref().unwrap())?,
            clock.slot,
        )?)
    } else if market.oracle_a.is_some() {
        Some(market.oracle_price_from_a(
            &AccountInfoRef::borrow(ctx.accounts.oracle_a.as_ref().unwrap())?,
            clock.slot,
        )?)
    } else {
        None
    };

    let OrderWithAmounts {
        order_id,
        total_base_taken_native,
        total_quote_taken_native,
        posted_base_native,
        posted_quote_native,
        taker_fees,
        maker_fees,
        ..
    } = book.new_order(
        &order,
        &mut market,
        &mut event_heap,
        oracle_price,
        Some(&mut open_orders_account),
        &open_orders_account_pk,
        now_ts,
        limit,
        ctx.remaining_accounts,
    )?;

    let position = &mut open_orders_account.position;
    let deposit_amount = match order.side {
        Side::Bid => {
            let free_quote = position.quote_free_native;
            let max_quote_including_fees =
                total_quote_taken_native + posted_quote_native + taker_fees + maker_fees;

            let free_qty_to_lock = cmp::min(max_quote_including_fees, free_quote);
            let deposit_amount = max_quote_including_fees - free_qty_to_lock;

            // Update market deposit total
            position.quote_free_native -= free_qty_to_lock;
            market.quote_deposit_total += deposit_amount;

            deposit_amount
        }

        Side::Ask => {
            let free_assets_native = position.base_free_native;
            let max_base_native = total_base_taken_native + posted_base_native;

            let free_qty_to_lock = cmp::min(max_base_native, free_assets_native);
            let deposit_amount = max_base_native - free_qty_to_lock;

            // Update market deposit total
            position.base_free_native -= free_qty_to_lock;
            market.base_deposit_total += deposit_amount;

            deposit_amount
        }
    };

    token_transfer(
        deposit_amount,
        &ctx.accounts.token_program,
        &ctx.accounts.user_token_account,
        &ctx.accounts.market_vault,
        &ctx.accounts.signer,
    )?;

    if event_heap.len() > event_heap_size_before {
        system_program_transfer(
            PENALTY_EVENT_HEAP,
            &ctx.accounts.system_program,
            &ctx.accounts.signer,
            &ctx.accounts.market,
        )?;
    }

    Ok(order_id)
}
