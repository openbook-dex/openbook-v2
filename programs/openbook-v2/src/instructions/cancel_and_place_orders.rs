use anchor_lang::prelude::*;
use std::cmp;

use crate::accounts_ix::*;
use crate::accounts_zerocopy::*;
use crate::error::*;
use crate::logs::CancelOrdersLog;
use crate::state::*;
use crate::token_utils::token_transfer;

#[allow(clippy::too_many_arguments)]
pub fn cancel_and_place_orders(
    ctx: Context<CancelAndPlaceOrders>,
    cancel_client_orders_ids: Vec<u64>,
    orders: Vec<Order>,
    limits: Vec<u8>,
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
    let mut event_queue = ctx.accounts.event_queue.load_mut()?;

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

    let mut canceled_quantity = 0;
    for client_order_id in cancel_client_orders_ids {
        let oo = open_orders_account.find_order_with_client_order_id(client_order_id);
        if let Some(oo) = oo {
            let order_id = oo.id;
            let order_side_and_tree = oo.side_and_tree();
            let leaf_node = book.cancel_order(
                &mut open_orders_account,
                order_id,
                order_side_and_tree,
                *market,
                Some(ctx.accounts.open_orders_account.key()),
            )?;
            canceled_quantity += leaf_node.quantity;
        };
    }
    if canceled_quantity > 0 {
        emit!(CancelOrdersLog {
            open_orders_account: ctx.accounts.open_orders_account.key(),
            total_quantity: canceled_quantity,
        });
    }

    let mut deposit_quote_amount = 0;
    let mut deposit_base_amount = 0;
    let mut order_ids = Vec::new();
    for (order, limit) in orders.iter().zip(limits) {
        require_gte!(order.max_base_lots, 0, OpenBookError::InvalidInputLots);
        require_gte!(
            order.max_quote_lots_including_fees,
            0,
            OpenBookError::InvalidInputLots
        );

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
            &mut event_queue,
            oracle_price,
            Some(&mut open_orders_account),
            &open_orders_account_pk,
            now_ts,
            limit,
            ctx.remaining_accounts,
        )?;

        let position = &mut open_orders_account.position;
        match order.side {
            Side::Bid => {
                let free_quote = position.quote_free_native;
                let max_quote_including_fees =
                    total_quote_taken_native + posted_quote_native + taker_fees + maker_fees;

                let free_qty_to_lock = cmp::min(max_quote_including_fees, free_quote);
                let deposit_amount = max_quote_including_fees - free_qty_to_lock;

                // Update market deposit total
                position.quote_free_native -= free_qty_to_lock;
                market.quote_deposit_total += deposit_amount;

                deposit_quote_amount += deposit_amount;
            }

            Side::Ask => {
                let free_assets_native = position.base_free_native;
                let max_base_native = total_base_taken_native + posted_base_native;

                let free_qty_to_lock = cmp::min(max_base_native, free_assets_native);
                let deposit_amount = max_base_native - free_qty_to_lock;

                // Update market deposit total
                position.base_free_native -= free_qty_to_lock;
                market.base_deposit_total += deposit_amount;

                deposit_base_amount += deposit_amount;
            }
        };
        order_ids.push(order_id);
    }

    token_transfer(
        deposit_quote_amount,
        &ctx.accounts.token_program,
        &ctx.accounts.token_quote_deposit_account,
        &ctx.accounts.market_quote_vault,
        &ctx.accounts.signer,
    )?;
    token_transfer(
        deposit_base_amount,
        &ctx.accounts.token_program,
        &ctx.accounts.token_base_deposit_account,
        &ctx.accounts.market_base_vault,
        &ctx.accounts.signer,
    )?;

    Ok(order_ids)
}
