use anchor_lang::prelude::*;
use std::cmp;

use crate::accounts_ix::*;
use crate::accounts_zerocopy::AccountInfoRef;
use crate::error::*;
use crate::state::*;
use crate::token_utils::*;
use anchor_spl::token_interface::{TokenInterface, self, Mint, TokenAccount};

#[allow(clippy::too_many_arguments)]
pub fn cancel_and_place_orders<'info>(
    ctx: Context<'_, '_, '_, 'info, CancelAndPlaceOrders<'info>>,
    cancel_client_orders_ids: Vec<u64>,
    orders: Vec<Order>,
    limits: Vec<u8>,
) -> Result<Vec<Option<u128>>> {
    let mut open_orders_account = ctx.accounts.open_orders_account.load_mut()?;
    let open_orders_account_pk = ctx.accounts.open_orders_account.key();

    let clock = Clock::get()?;

    let remaining_accounts = ctx.remaining_accounts;

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

    let oracle_price = market.oracle_price(
        AccountInfoRef::borrow_some(ctx.accounts.oracle_a.as_ref())?.as_ref(),
        AccountInfoRef::borrow_some(ctx.accounts.oracle_b.as_ref())?.as_ref(),
        clock.slot,
    )?;

    for client_order_id in cancel_client_orders_ids {
        let oo = open_orders_account.find_order_with_client_order_id(client_order_id);
        if let Some(oo) = oo {
            let order_id = oo.id;
            let order_side_and_tree = oo.side_and_tree();

            let cancel_result = book.cancel_order(
                &mut open_orders_account,
                order_id,
                order_side_and_tree,
                *market,
                Some(ctx.accounts.open_orders_account.key()),
            );
            // Allow cancel fails due order ID not found. Otherwise propagates error
            if !cancel_result.is_anchor_error_with_code(OpenBookError::OrderIdNotFound.into()) {
                cancel_result?;
            }
        };
    }

    let mut base_amount = 0_u64;
    let mut quote_amount = 0_u64;
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
            &mut event_heap,
            oracle_price,
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

    // Getting actual base amount to be paid
    // let base_token_account_info = remaining_accounts[0];

    let base_token_fee_wrapped = {
        get_token_fee(remaining_accounts[0].to_account_info(), ctx.accounts.token_program.to_account_info(), deposit_base_amount)
    };
    let base_token_fee = base_token_fee_wrapped.unwrap().unwrap();

    let base_amount = deposit_base_amount - base_token_fee;

    // Getting actual quote native amount to be paid
    // let quote_token_account_info = remaining_accounts[1];

    let quote_token_fee_wrapped = {
        get_token_fee(remaining_accounts[1].to_account_info(), ctx.accounts.token_program.to_account_info(), deposit_quote_amount)
    };
    let quote_token_fee = quote_token_fee_wrapped.unwrap().unwrap();

    let quote_amount = deposit_quote_amount - quote_token_fee;

    let base_data = &mut &**remaining_accounts[0].try_borrow_data()?;
    let base_mint = Mint::try_deserialize(base_data).unwrap();
    let base_decimals = base_mint.decimals;

    let quote_data = &mut &**remaining_accounts[1].try_borrow_data()?;
    let quote_mint = Mint::try_deserialize(quote_data).unwrap();
    let quote_decimals = quote_mint.decimals;


    token_transfer(
        base_amount,
        &ctx.accounts.token_program,
        &ctx.accounts.user_quote_account,
        &ctx.accounts.market_quote_vault,
        &ctx.accounts.signer,
        remaining_accounts[0].to_account_info(),
        base_decimals,
    )?;

    token_transfer(
        quote_amount,
        &ctx.accounts.token_program,
        &ctx.accounts.user_base_account,
        &ctx.accounts.market_base_vault,
        &ctx.accounts.signer,
        remaining_accounts[1].to_account_info(),
        quote_decimals,
    )?;

    Ok(order_ids)
}
