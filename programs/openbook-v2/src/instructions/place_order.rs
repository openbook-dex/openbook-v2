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

    let _now_ts: u64 = Clock::get()?.unix_timestamp.try_into().unwrap();
    let oracle_price;
    {
        let market = ctx.accounts.market.load_mut()?;
        oracle_price = market.oracle_price(
            &AccountInfoRef::borrow(ctx.accounts.oracle.as_ref())?,
            Some(Clock::get()?.slot),
        )?;
    }
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

    let max_quote_lots_including_fees = order.max_quote_lots_including_fees;
    let max_base_lots = order.max_base_lots;
    let side = order.side;

    let TakenQuantitiesIncludingFees {
        order_id,
        total_base_taken_native,
        total_quote_taken_native,
        referrer_amount: _,
    } = book.new_order(
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
    let (to_vault, deposit_amount) = match side {
        Side::Bid => {
            let free_assets_native = position.quote_free_native;

            let max_native_including_fees: I80F48 = match order.params {
                OrderParams::Market | OrderParams::ImmediateOrCancel { .. } => {
                    total_quote_taken_native
                }
                OrderParams::Fixed {
                    price_lots: _,
                    order_type,
                } => {
                    // For PostOnly If existing orders can match with this order, do nothing
                    if order_type == PostOrderType::PostOnly && order_id.is_none() {
                        I80F48::ZERO
                    } else {
                        I80F48::from_num(max_quote_lots_including_fees)
                            * I80F48::from_num(market.quote_lot_size)
                    }
                }
                // TODO use peg_limit
                OrderParams::OraclePegged { .. } => {
                    I80F48::from_num(max_quote_lots_including_fees)
                        * I80F48::from_num(market.quote_lot_size)
                }
            };
            let free_qty_to_lock = cmp::min(max_native_including_fees, free_assets_native);
            position.quote_free_native -= free_qty_to_lock;

            // Update market deposit total
            market.quote_deposit_total += ((max_native_including_fees - free_qty_to_lock)
                - (total_quote_taken_native * (market.taker_fee - market.maker_fee)))
                .to_num::<u64>();

            (
                ctx.accounts.quote_vault.to_account_info(),
                max_native_including_fees - free_qty_to_lock,
            )
        }

        Side::Ask => {
            let free_assets_native = position.base_free_native;

            let max_base_native: I80F48 = match order.params {
                OrderParams::Market | OrderParams::ImmediateOrCancel { .. } => {
                    total_base_taken_native
                }
                OrderParams::Fixed {
                    price_lots: _,
                    order_type,
                } => {
                    // For PostOnly If existing orders can match with this order, do nothing
                    if order_type == PostOrderType::PostOnly && order_id.is_none() {
                        I80F48::ZERO
                    } else {
                        I80F48::from_num(max_base_lots) * I80F48::from_num(market.base_lot_size)
                    }
                }
                // TODO use peg_limit
                OrderParams::OraclePegged { .. } => {
                    I80F48::from_num(max_base_lots) * I80F48::from_num(market.base_lot_size)
                }
            };

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
