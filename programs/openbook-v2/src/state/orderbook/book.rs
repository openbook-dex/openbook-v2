use crate::logs::*;
use crate::state::MAX_OPEN_ORDERS;
use crate::{
    error::*,
    state::{orderbook::bookside::*, EventHeap, Market, OpenOrdersAccount},
};
use anchor_lang::prelude::*;
use bytemuck::cast;
use std::cell::RefMut;

use super::*;

/// Drop at most this many expired orders from a BookSide when trying to match orders.
/// This exists as a guard against excessive compute use.
pub const DROP_EXPIRED_ORDER_LIMIT: usize = 5;

/// Process up to this remaining accounts in the fill event
pub const FILL_EVENT_REMAINING_LIMIT: usize = 15;

pub struct Orderbook<'a> {
    pub bids: RefMut<'a, BookSide>,
    pub asks: RefMut<'a, BookSide>,
}

pub struct OrderWithAmounts {
    pub order_id: Option<u128>,
    pub posted_base_native: u64,
    pub posted_quote_native: u64,
    pub total_base_taken_native: u64,
    pub total_quote_taken_native: u64,
    pub taker_fees: u64,
    pub maker_fees: u64,
    pub referrer_amount: u64,
}

impl<'a> Orderbook<'a> {
    pub fn init(&mut self) {
        self.bids.nodes.order_tree_type = OrderTreeType::Bids.into();
        self.asks.nodes.order_tree_type = OrderTreeType::Asks.into();
    }

    pub fn is_empty(&self) -> bool {
        self.bids.is_empty() && self.asks.is_empty()
    }

    pub fn bookside_mut(&mut self, side: Side) -> &mut BookSide {
        match side {
            Side::Bid => &mut self.bids,
            Side::Ask => &mut self.asks,
        }
    }

    pub fn bookside(&self, side: Side) -> &BookSide {
        match side {
            Side::Bid => &self.bids,
            Side::Ask => &self.asks,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_order<'c: 'info, 'info>(
        &mut self,
        order: &Order,
        open_book_market: &mut Market,
        market_pk: &Pubkey,
        event_heap: &mut EventHeap,
        oracle_price_lots: Option<i64>,
        mut open_orders_account: Option<&mut OpenOrdersAccount>,
        owner: &Pubkey,
        now_ts: u64,
        mut limit: u8,
        remaining_accs: &'c [AccountInfo<'info>],
    ) -> std::result::Result<OrderWithAmounts, Error> {
        let market = open_book_market;

        let side = order.side;

        let other_side = side.invert_side();
        let post_only = order.is_post_only();
        let fill_or_kill = order.is_fill_or_kill();
        let mut post_target = order.post_target();
        let (price_lots, price_data) = order.price(now_ts, oracle_price_lots, self)?;

        // generate new order id
        let order_id = market.gen_order_id(side, price_data);

        // Iterate through book and match against this new order.
        //
        // Any changes to matching orders on the other side of the book are collected in
        // matched_changes/matched_deletes and then applied after this loop.

        let order_max_base_lots = order.max_base_lots;
        let order_max_quote_lots = if side == Side::Bid && !post_only {
            market.subtract_taker_fees(order.max_quote_lots_including_fees)
        } else {
            order.max_quote_lots_including_fees
        };

        require_gte!(
            market.max_base_lots(),
            order_max_base_lots,
            OpenBookError::InvalidInputLotsSize
        );

        require_gte!(
            market.max_quote_lots(),
            order_max_quote_lots,
            OpenBookError::InvalidInputLotsSize
        );

        let mut remaining_base_lots = order_max_base_lots;
        let mut remaining_quote_lots = order_max_quote_lots;
        let mut decremented_quote_lots = 0_i64;

        let mut referrer_amount = 0_u64;
        let mut maker_rebates_acc = 0_u64;

        let mut matched_order_changes: Vec<(BookSideOrderHandle, i64)> = vec![];
        let mut matched_order_deletes: Vec<(BookSideOrderTree, u128)> = vec![];
        let mut number_of_dropped_expired_orders = 0;
        let mut number_of_processed_fill_events = 0;

        let opposing_bookside = self.bookside_mut(other_side);
        for best_opposing in opposing_bookside.iter_all_including_invalid(now_ts, oracle_price_lots)
        {
            if remaining_base_lots == 0 || remaining_quote_lots == 0 {
                break;
            }

            if !best_opposing.is_valid() {
                // Remove the order from the book unless we've done that enough
                if number_of_dropped_expired_orders < DROP_EXPIRED_ORDER_LIMIT {
                    number_of_dropped_expired_orders += 1;
                    let event = OutEvent::new(
                        other_side,
                        best_opposing.node.owner_slot,
                        now_ts,
                        event_heap.header.seq_num,
                        best_opposing.node.owner,
                        best_opposing.node.quantity,
                    );

                    process_out_event(
                        event,
                        market,
                        event_heap,
                        open_orders_account.as_deref_mut(),
                        owner,
                        remaining_accs,
                    )?;
                    matched_order_deletes
                        .push((best_opposing.handle.order_tree, best_opposing.node.key));
                }
                continue;
            }

            let best_opposing_price = best_opposing.price_lots;

            if !side.is_price_within_limit(best_opposing_price, price_lots) {
                break;
            }
            if post_only {
                msg!("Order could not be placed due to PostOnly");
                post_target = None;
                break; // return silently to not fail other instructions in tx
            }
            if limit == 0 {
                msg!("Order matching limit reached");
                post_target = None;
                break;
            }

            let max_match_by_quote = remaining_quote_lots / best_opposing_price;
            // Do not post orders in the book due to bad pricing and negative spread
            if max_match_by_quote == 0 {
                post_target = None;
                break;
            }

            let match_base_lots = remaining_base_lots
                .min(best_opposing.node.quantity)
                .min(max_match_by_quote);
            let match_quote_lots = match_base_lots * best_opposing_price;

            // Self-trade behaviour
            if open_orders_account.is_some() && owner == &best_opposing.node.owner {
                match order.self_trade_behavior {
                    SelfTradeBehavior::DecrementTake => {
                        // remember all decremented quote lots to only charge fees on not-self-trades
                        decremented_quote_lots += match_quote_lots;
                    }
                    SelfTradeBehavior::CancelProvide => {
                        // The open orders acc is always present in this case, no need event_heap
                        open_orders_account.as_mut().unwrap().cancel_order(
                            best_opposing.node.owner_slot as usize,
                            best_opposing.node.quantity,
                            *market,
                        );
                        matched_order_deletes
                            .push((best_opposing.handle.order_tree, best_opposing.node.key));

                        // skip actual matching
                        continue;
                    }
                    SelfTradeBehavior::AbortTransaction => {
                        return err!(OpenBookError::WouldSelfTrade)
                    }
                }
                assert!(order.self_trade_behavior == SelfTradeBehavior::DecrementTake);
            } else {
                maker_rebates_acc +=
                    market.maker_rebate_floor((match_quote_lots * market.quote_lot_size) as u64);
            }

            remaining_base_lots -= match_base_lots;
            remaining_quote_lots -= match_quote_lots;
            assert!(remaining_quote_lots >= 0);

            let new_best_opposing_quantity = best_opposing.node.quantity - match_base_lots;
            let maker_out = new_best_opposing_quantity == 0;
            if maker_out {
                matched_order_deletes
                    .push((best_opposing.handle.order_tree, best_opposing.node.key));
            } else {
                matched_order_changes.push((best_opposing.handle, new_best_opposing_quantity));
            }

            let fill = FillEvent::new(
                side,
                maker_out,
                best_opposing.node.owner_slot,
                now_ts,
                market.seq_num,
                best_opposing.node.owner,
                best_opposing.node.client_order_id,
                best_opposing.node.timestamp,
                *owner,
                order.client_order_id,
                best_opposing_price,
                best_opposing.node.peg_limit,
                match_base_lots,
            );

            emit_stack(TakerSignatureLog {
                market: *market_pk,
                seq_num: market.seq_num,
            });

            process_fill_event(
                fill,
                market,
                event_heap,
                remaining_accs,
                &mut number_of_processed_fill_events,
            )?;

            limit -= 1;
        }

        let total_quote_lots_taken = order_max_quote_lots - remaining_quote_lots;
        let total_base_lots_taken = order.max_base_lots - remaining_base_lots;
        assert!(total_quote_lots_taken >= 0);
        assert!(total_base_lots_taken >= 0);

        let total_base_taken_native = (total_base_lots_taken * market.base_lot_size) as u64;
        let total_quote_taken_native = (total_quote_lots_taken * market.quote_lot_size) as u64;

        // Record the taker trade in the account already, even though it will only be
        // realized when the fill event gets executed
        let mut taker_fees_native = 0_u64;
        if total_quote_lots_taken > 0 || total_base_lots_taken > 0 {
            let total_quote_taken_native_wo_self =
                ((total_quote_lots_taken - decremented_quote_lots) * market.quote_lot_size) as u64;

            if total_quote_taken_native_wo_self > 0 {
                taker_fees_native = market.taker_fees_ceil(total_quote_taken_native_wo_self);

                // Only account taker fees now. Maker fees accounted once processing the event
                referrer_amount = taker_fees_native - maker_rebates_acc;
                market.fees_accrued += referrer_amount as u128;
            };

            if let Some(open_orders_account) = &mut open_orders_account {
                open_orders_account.execute_taker(
                    market,
                    side,
                    total_base_taken_native,
                    total_quote_taken_native,
                    taker_fees_native,
                    referrer_amount,
                );
            } else {
                market.taker_volume_wo_oo += total_quote_taken_native as u128;
            }

            let (total_quantity_paid, total_quantity_received) = match side {
                Side::Bid => (
                    total_quote_taken_native + taker_fees_native,
                    total_base_taken_native,
                ),
                Side::Ask => (
                    total_base_taken_native,
                    total_quote_taken_native - taker_fees_native,
                ),
            };

            emit_stack(TotalOrderFillEvent {
                side: side.into(),
                taker: *owner,
                total_quantity_paid,
                total_quantity_received,
                fees: taker_fees_native,
            });
        }

        // The native taker fees in lots, rounded up.
        //
        // Imagine quote_lot_size = 10. A new bid comes in with max_quote lots = 10. It matches against
        // other orders for 5 quote lots total. The taker_fees_native is 15, taker_fees_lots is 2. That
        // means only up the 3 quote lots may be placed on the book.
        let taker_fees_lots =
            (taker_fees_native as i64 + market.quote_lot_size - 1) / market.quote_lot_size;

        // Update remaining based on quote_lots taken. If nothing taken, same as the beginning
        remaining_quote_lots =
            order.max_quote_lots_including_fees - total_quote_lots_taken - taker_fees_lots;

        // Apply changes to matched asks (handles invalidate on delete!)
        for (handle, new_quantity) in matched_order_changes {
            opposing_bookside
                .node_mut(handle.node)
                .unwrap()
                .as_leaf_mut()
                .unwrap()
                .quantity = new_quantity;
        }
        for (component, key) in matched_order_deletes {
            let _removed_leaf = opposing_bookside.remove_by_key(component, key).unwrap();
        }

        //
        // Place remainder on the book if requested
        //

        // To calculate max quantity to post, for oracle peg orders & bids take the peg_limit as
        // it's the upper price limitation
        let is_oracle_peg = order.peg_limit() != -1;
        let price = if is_oracle_peg && order.side == Side::Bid {
            order.peg_limit()
        } else {
            price_lots
        };

        // If there are still quantity unmatched, place on the book
        let book_base_quantity_lots = {
            remaining_quote_lots -= market.maker_fees_ceil(remaining_quote_lots);
            remaining_base_lots.min(remaining_quote_lots / price)
        };

        if book_base_quantity_lots <= 0 {
            post_target = None;
        }

        if is_oracle_peg && side.is_price_better(price_lots, order.peg_limit()) {
            msg!(
                "Posting on book disallowed due to peg_limit, order price {:?}, limit {:?}",
                price_lots,
                order.peg_limit(),
            );
            post_target = None;
        }

        // There is still quantity, but it's a fill or kill order -> kill
        if fill_or_kill && remaining_base_lots > 0 {
            return err!(OpenBookError::WouldExecutePartially);
        }

        let mut maker_fees_native = 0;
        let mut posted_base_native = 0;
        let mut posted_quote_native = 0;

        if let Some(order_tree_target) = post_target {
            require_gte!(
                market.max_quote_lots(),
                book_base_quantity_lots * price,
                OpenBookError::InvalidPostAmount
            );

            posted_base_native = book_base_quantity_lots * market.base_lot_size;
            posted_quote_native = book_base_quantity_lots * price * market.quote_lot_size;

            // Open orders always exists in this case
            let open_orders = open_orders_account.as_mut().unwrap();

            // Subtract maker fees in bid.
            if side == Side::Bid {
                maker_fees_native = market
                    .maker_fees_ceil(posted_quote_native)
                    .try_into()
                    .unwrap();

                open_orders.position.locked_maker_fees += maker_fees_native;
            }

            let bookside = self.bookside_mut(side);
            // Drop an expired order if possible
            if let Some(expired_order) = bookside.remove_one_expired(order_tree_target, now_ts) {
                let event = OutEvent::new(
                    side,
                    expired_order.owner_slot,
                    now_ts,
                    event_heap.header.seq_num,
                    expired_order.owner,
                    expired_order.quantity,
                );
                process_out_event(
                    event,
                    market,
                    event_heap,
                    Some(open_orders),
                    owner,
                    remaining_accs,
                )?;
            }

            if bookside.is_full() {
                // If this bid is higher than lowest bid, boot that bid and insert this one
                let (worst_order, worst_price) =
                    bookside.remove_worst(now_ts, oracle_price_lots).unwrap();
                // OpenBookErrorCode::OutOfSpace
                require!(
                    side.is_price_better(price_lots, worst_price),
                    OpenBookError::SomeError
                );
                let event = OutEvent::new(
                    side,
                    worst_order.owner_slot,
                    now_ts,
                    event_heap.header.seq_num,
                    worst_order.owner,
                    worst_order.quantity,
                );
                process_out_event(
                    event,
                    market,
                    event_heap,
                    Some(open_orders),
                    owner,
                    remaining_accs,
                )?;
            }

            let owner_slot = open_orders.next_order_slot()?;
            let new_order = LeafNode::new(
                owner_slot as u8,
                order_id,
                *owner,
                book_base_quantity_lots,
                now_ts,
                order.time_in_force,
                order.peg_limit(),
                order.client_order_id,
            );
            let _result = bookside.insert_leaf(order_tree_target, &new_order)?;

            open_orders.add_order(
                side,
                order_tree_target,
                &new_order,
                order.client_order_id,
                price,
            );
        }

        let placed_order_id = if post_target.is_some() {
            Some(order_id)
        } else {
            None
        };

        Ok(OrderWithAmounts {
            order_id: placed_order_id,
            posted_base_native: posted_base_native as u64,
            posted_quote_native: posted_quote_native as u64,
            total_base_taken_native,
            total_quote_taken_native,
            referrer_amount,
            taker_fees: taker_fees_native,
            maker_fees: maker_fees_native,
        })
    }

    /// Cancels up to `limit` orders that are listed on the openorders account for the given market.
    /// Optionally filters by `side_to_cancel_option`.
    /// The orders are removed from the book and from the openorders account open order list.
    pub fn cancel_all_orders(
        &mut self,
        open_orders_account: &mut OpenOrdersAccount,
        market: Market,
        mut limit: u8,
        side_to_cancel_option: Option<Side>,
        client_id_option: Option<u64>,
    ) -> Result<i64> {
        let mut total_quantity = 0_i64;
        for i in 0..MAX_OPEN_ORDERS {
            let oo = open_orders_account.open_orders[i];
            if oo.is_free() {
                continue;
            }

            let order_side_and_tree = oo.side_and_tree();
            if let Some(side_to_cancel) = side_to_cancel_option {
                if side_to_cancel != order_side_and_tree.side() {
                    continue;
                }
            }

            if let Some(client_id) = client_id_option {
                if client_id != oo.client_id {
                    continue;
                }
            }

            if limit == 0 {
                msg!("Cancel orders limit reached");
                break;
            }

            let order_id = oo.id;

            let cancel_result = self.cancel_order(
                open_orders_account,
                order_id,
                order_side_and_tree,
                market,
                None,
            );
            if cancel_result.is_anchor_error_with_code(OpenBookError::OrderIdNotFound.into()) {
                // It's possible for the order to be filled or expired already.
                // There will be an event on the heap, the perp order slot is freed once
                // it is processed.
                msg!(
                    "order {} was not found on orderbook, expired or filled already",
                    order_id
                );
            } else {
                total_quantity += cancel_result?.quantity;
            }

            limit -= 1;
        }
        Ok(total_quantity)
    }

    /// Cancels an order on a side, removing it from the book and the openorders account orders list
    pub fn cancel_order(
        &mut self,
        open_orders_account: &mut OpenOrdersAccount,
        order_id: u128,
        side_and_tree: SideAndOrderTree,
        market: Market,
        expected_owner: Option<Pubkey>,
    ) -> Result<LeafNode> {
        let side = side_and_tree.side();
        let book_component = side_and_tree.order_tree();
        let leaf_node = self.bookside_mut(side).
        remove_by_key(book_component, order_id).ok_or_else(|| {
            // possibly already filled or expired?
            error_msg_typed!(OpenBookError::OrderIdNotFound, "no order with id {order_id}, side {side:?}, component {book_component:?} found on the orderbook")
        })?;
        if let Some(owner) = expected_owner {
            require_keys_eq!(leaf_node.owner, owner);
        }
        open_orders_account.cancel_order(leaf_node.owner_slot as usize, leaf_node.quantity, market);

        Ok(leaf_node)
    }
}

pub fn process_out_event<'c: 'info, 'info>(
    event: OutEvent,
    market: &Market,
    event_heap: &mut EventHeap,
    open_orders_account: Option<&mut OpenOrdersAccount>,
    owner: &Pubkey,
    remaining_accs: &'c [AccountInfo<'info>],
) -> Result<()> {
    if let Some(acc) = open_orders_account {
        if owner == &event.owner {
            acc.cancel_order(event.owner_slot as usize, event.quantity, *market);
            return Ok(());
        }
    }

    if let Some(acc) = remaining_accs.iter().find(|ai| ai.key == &event.owner) {
        let ooa: AccountLoader<OpenOrdersAccount> = AccountLoader::try_from(acc)?;
        let mut acc = ooa.load_mut()?;
        acc.cancel_order(event.owner_slot as usize, event.quantity, *market);
    } else {
        event_heap.push_back(cast(event));
    }

    Ok(())
}

pub fn process_fill_event<'c: 'info, 'info>(
    event: FillEvent,
    market: &mut Market,
    event_heap: &mut EventHeap,
    remaining_accs: &'c [AccountInfo<'info>],
    number_of_processed_fill_events: &mut usize,
) -> Result<()> {
    let mut is_processed = false;
    if *number_of_processed_fill_events < FILL_EVENT_REMAINING_LIMIT {
        if let Some(acc) = remaining_accs.iter().find(|ai| ai.key == &event.maker) {
            let ooa: AccountLoader<OpenOrdersAccount> = AccountLoader::try_from(acc)?;
            let mut maker = ooa.load_mut()?;
            maker.execute_maker(market, &event);
            is_processed = true;
            *number_of_processed_fill_events += 1;
        }
    }

    if !is_processed {
        event_heap.push_back(cast(event));
    }

    Ok(())
}
