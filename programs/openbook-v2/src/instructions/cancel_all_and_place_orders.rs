use anchor_lang::prelude::*;
use std::cmp;

use crate::accounts_ix::*;
use crate::accounts_zerocopy::AccountInfoRef;
use crate::error::*;
use crate::state::*;
use crate::token_utils::*;

#[allow(clippy::too_many_arguments)]
pub fn cancel_all_and_place_orders<'c: 'info, 'info>(
    ctx: Context<'_, '_, 'c, 'info, CancelAllAndPlaceOrders<'info>>,
    cancel: bool,
    mut orders: Vec<Order>,
    limit: u8,
) -> Result<Vec<Option<u128>>> {
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

    let oracle_price_lots = market.oracle_price_lots(
        AccountInfoRef::borrow_some(ctx.accounts.oracle_a.as_ref())?.as_ref(),
        AccountInfoRef::borrow_some(ctx.accounts.oracle_b.as_ref())?.as_ref(),
        clock.slot,
    )?;

    if cancel {
        book.cancel_all_orders(&mut open_orders_account, *market, u8::MAX, None, None)?;
    }

    let mut base_amount = 0_u64;
    let mut quote_amount = 0_u64;
    let mut order_ids = Vec::new();
    for order in orders.iter_mut() {
        order.max_base_lots = market.max_base_lots();
        require_gte!(
            order.max_quote_lots_including_fees,
            0,
            OpenBookError::InvalidInputLots
        );

        match order.side {
            Side::Ask => {
                let max_available_base = ctx.accounts.user_base_account.amount
                    + open_orders_account.position.base_free_native
                    - base_amount;
                order.max_base_lots = std::cmp::min(
                    order.max_base_lots,
                    market.max_base_lots_from_lamports(max_available_base),
                );
            }
            Side::Bid => {
                let max_available_quote = ctx.accounts.user_quote_account.amount
                    + open_orders_account.position.quote_free_native
                    - quote_amount;
                order.max_quote_lots_including_fees = std::cmp::min(
                    order.max_quote_lots_including_fees,
                    market.max_quote_lots_from_lamports(max_available_quote),
                );
            }
        }

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
            order,
            &mut market,
            &ctx.accounts.market.key(),
            &mut event_heap,
            oracle_price_lots,
            Some(&mut open_orders_account),
            &open_orders_account_pk,
            now_ts,
            limit,
            ctx.remaining_accounts,
        )?;

        match order.side {
            Side::Bid => {
                quote_amount = quote_amount
                    .checked_add(
                        total_quote_taken_native + posted_quote_native + taker_fees + maker_fees,
                    )
                    .ok_or(OpenBookError::InvalidInputOrdersAmounts)?;
            }
            Side::Ask => {
                base_amount = base_amount
                    .checked_add(total_base_taken_native + posted_base_native)
                    .ok_or(OpenBookError::InvalidInputOrdersAmounts)?;
            }
        };

        order_ids.push(order_id);
    }

    let position = &mut open_orders_account.position;

    let free_base_to_lock = cmp::min(base_amount, position.base_free_native);
    let free_quote_to_lock = cmp::min(quote_amount, position.quote_free_native);

    let deposit_base_amount = base_amount - free_base_to_lock;
    let deposit_quote_amount = quote_amount - free_quote_to_lock;

    position.base_free_native -= free_base_to_lock;
    position.quote_free_native -= free_quote_to_lock;

    market.base_deposit_total += deposit_base_amount;
    market.quote_deposit_total += deposit_quote_amount;

    if event_heap.len() > event_heap_size_before {
        position.penalty_heap_count += 1;
    }

    token_transfer(
        deposit_quote_amount,
        &ctx.accounts.token_program,
        &ctx.accounts.user_quote_account,
        &ctx.accounts.market_quote_vault,
        &ctx.accounts.signer,
    )?;
    token_transfer(
        deposit_base_amount,
        &ctx.accounts.token_program,
        &ctx.accounts.user_base_account,
        &ctx.accounts.market_base_vault,
        &ctx.accounts.signer,
    )?;

    Ok(order_ids)
}
