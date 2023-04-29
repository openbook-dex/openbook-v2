use crate::state::OpenOrdersAccountRefMut;
use crate::{
    error::*,
    state::{orderbook::bookside::*, EventQueue, Market},
};
use anchor_lang::prelude::*;
use bytemuck::cast;
use fixed::types::I80F48;
use std::cell::RefMut;

use super::*;

/// Drop at most this many expired orders from a BookSide when trying to match orders.
/// This exists as a guard against excessive compute use.
const DROP_EXPIRED_ORDER_LIMIT: usize = 5;

pub struct Orderbook<'a> {
    pub bids: RefMut<'a, BookSide>,
    pub asks: RefMut<'a, BookSide>,
}

impl<'a> Orderbook<'a> {
    pub fn init(&mut self) {
        self.bids.nodes.order_tree_type = OrderTreeType::Bids.into();
        self.asks.nodes.order_tree_type = OrderTreeType::Asks.into();
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
    pub fn new_order(
        &mut self,
        order: Order,
        open_book_market: &mut Market,
        event_queue: &mut EventQueue,
        oracle_price: I80F48,
        mut open_orders_acc: OpenOrdersAccountRefMut,
        owner: &Pubkey,
        now_ts: u64,
        mut limit: u8,
    ) -> std::result::Result<Option<u128>, Error> {
        let side = order.side;

        let other_side = side.invert_side();
        let market = open_book_market;
        let oracle_price_lots = market.native_price_to_lot(oracle_price);
        let post_only = order.is_post_only();
        let mut post_target = order.post_target();
        let (price_lots, price_data) = order.price(now_ts, oracle_price_lots, self)?;

        // generate new order id
        let order_id = market.gen_order_id(side, price_data);

        // // IOC orders have a fee penalty applied regardless of match
        if order.needs_penalty_fee() {
            apply_penalty(market, &mut open_orders_acc)?;
        }

        // Iterate through book and match against this new order.
        //
        // Any changes to matching orders on the other side of the book are collected in
        // matched_changes/matched_deletes and then applied after this loop.
        let mut remaining_base_lots = order.max_base_lots;
        let mut remaining_quote_lots = order.max_quote_lots_including_fees;
        let mut matched_order_changes: Vec<(BookSideOrderHandle, i64)> = vec![];
        let mut matched_order_deletes: Vec<(BookSideOrderTree, u128)> = vec![];
        let mut number_of_dropped_expired_orders = 0;
        let opposing_bookside = self.bookside_mut(other_side);

        // Substract fees in case of bid
        if side == Side::Bid {
            remaining_quote_lots = market.substract_taker_fees(remaining_quote_lots);
            msg!("11xxx {}", remaining_quote_lots);
        }

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
                        event_queue.header.seq_num,
                        best_opposing.node.owner,
                        best_opposing.node.quantity,
                    );
                    event_queue.push_back(cast(event)).unwrap();
                    matched_order_deletes
                        .push((best_opposing.handle.order_tree, best_opposing.node.key));
                }
                continue;
            }

            let best_opposing_price = best_opposing.price_lots;

            if !side.is_price_within_limit(best_opposing_price, price_lots) {
                break;
            } else if post_only {
                msg!("Order could not be placed due to PostOnly");
                post_target = None;
                break; // return silently to not fail other instructions in tx
            } else if limit == 0 {
                msg!("Order matching limit reached");
                post_target = None;
                break;
            }

            let max_match_by_quote = remaining_quote_lots / best_opposing_price;
            let match_base_lots = remaining_base_lots
                .min(best_opposing.node.quantity)
                .min(max_match_by_quote);

            let match_quote_lots = match_base_lots * best_opposing_price;
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
                event_queue.header.seq_num,
                best_opposing.node.owner,
                best_opposing.node.client_order_id,
                best_opposing.node.timestamp,
                *owner,
                order.client_order_id,
                best_opposing_price,
                match_base_lots,
            );
            event_queue.push_back(cast(fill)).unwrap();
            limit -= 1;
        }
        let total_quote_lots_taken = order.max_quote_lots_including_fees - remaining_quote_lots;
        let total_base_lots_taken = order.max_base_lots - remaining_base_lots;
        assert!(total_quote_lots_taken >= 0);
        assert!(total_base_lots_taken >= 0);

        // Record the taker trade in the account already, even though it will only be
        // realized when the fill event gets executed
        if total_quote_lots_taken > 0 || total_base_lots_taken > 0 {
            open_orders_acc.fixed.position.add_taker_trade(
                side,
                total_base_lots_taken,
                total_quote_lots_taken,
            );
            apply_fees(market, &mut open_orders_acc, total_quote_lots_taken)?;
            // Update remaining based on quote_lots taken. If nothing taken, same as the beggining
            remaining_quote_lots = order.max_quote_lots_including_fees
                - total_quote_lots_taken
                - (market.taker_fee * I80F48::from_num(total_quote_lots_taken)).to_num::<i64>();
        }
        msg!("remaining_quote_lots {}", remaining_quote_lots);

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

        // If there are still quantity unmatched, place on the book
        msg!(
            "remaining_base_lots {}, remaining_quote_lots {}, price_lots {}",
            remaining_base_lots,
            remaining_quote_lots,
            price_lots
        );
        let book_base_quantity = remaining_base_lots.min(remaining_quote_lots / price_lots);
        if book_base_quantity <= 0 {
            post_target = None;
        }

        if let Some(order_tree_target) = post_target {
            let bookside = self.bookside_mut(side);
            // Drop an expired order if possible
            if let Some(expired_order) = bookside.remove_one_expired(order_tree_target, now_ts) {
                let event = OutEvent::new(
                    side,
                    expired_order.owner_slot,
                    now_ts,
                    event_queue.header.seq_num,
                    expired_order.owner,
                    expired_order.quantity,
                );
                event_queue.push_back(cast(event)).unwrap();
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
                    event_queue.header.seq_num,
                    worst_order.owner,
                    worst_order.quantity,
                );
                event_queue.push_back(cast(event)).unwrap();
            }

            let owner_slot = open_orders_acc.next_order_slot()?;
            let new_order = LeafNode::new(
                owner_slot as u8,
                order_id,
                *owner,
                book_base_quantity,
                now_ts,
                PostOrderType::Limit, // TODO: Support order types? needed?
                order.time_in_force,
                order.peg_limit(),
                order.client_order_id,
            );
            let _result = bookside.insert_leaf(order_tree_target, &new_order)?;

            // TODO OPT remove if PlaceOrder needs more compute
            msg!(
                "{} on book order_id={} quantity={} price={}",
                match side {
                    Side::Bid => "bid",
                    Side::Ask => "ask",
                },
                order_id,
                book_base_quantity,
                price_lots
            );

            open_orders_acc.add_order(
                side,
                order_tree_target,
                &new_order,
                order.client_order_id,
            )?;
        }

        if post_target.is_some() {
            Ok(Some(order_id))
        } else {
            Ok(None)
        }
    }

    /// Cancels up to `limit` orders that are listed on the openorders account for the given market.
    /// Optionally filters by `side_to_cancel_option`.
    /// The orders are removed from the book and from the openorders account open order list.
    pub fn cancel_all_orders(
        &mut self,
        open_orders_acc: &mut OpenOrdersAccountRefMut,
        _open_book_market: &mut Market,
        mut limit: u8,
        side_to_cancel_option: Option<Side>,
    ) -> Result<()> {
        for i in 0..open_orders_acc.header.oo_count() {
            let oo = open_orders_acc.order_by_raw_index(i);

            let order_side_and_tree = oo.side_and_tree();
            if let Some(side_to_cancel) = side_to_cancel_option {
                if side_to_cancel != order_side_and_tree.side() {
                    continue;
                }
            }

            let order_id = oo.id;

            let cancel_result =
                self.cancel_order(open_orders_acc, order_id, order_side_and_tree, None);
            if cancel_result.is_anchor_error_with_code(OpenBookError::OrderIdNotFound.into()) {
                // It's possible for the order to be filled or expired already.
                // There will be an event on the queue, the perp order slot is freed once
                // it is processed.
                msg!(
                    "order {} was not found on orderbook, expired or filled already",
                    order_id
                );
            } else {
                cancel_result?;
            }

            limit -= 1;
            if limit == 0 {
                break;
            }
        }
        Ok(())
    }

    /// Cancels an order on a side, removing it from the book and the openorders account orders list
    pub fn cancel_order(
        &mut self,
        open_orders_acc: &mut OpenOrdersAccountRefMut,
        order_id: u128,
        side_and_tree: SideAndOrderTree,
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
        open_orders_acc.remove_order(leaf_node.owner_slot as usize, leaf_node.quantity, true)?;

        Ok(leaf_node)
    }
}

/// Apply taker fees to the taker account and update the markets' fees_accrued for
/// both the maker and taker fees.
fn apply_fees(
    market: &mut Market,
    open_orders_acc: &mut OpenOrdersAccountRefMut,
    quote_lots: i64,
) -> Result<()> {
    let quote_native = I80F48::from_num(market.quote_lot_size * quote_lots);

    // The maker fees apply to the maker's account only when the fill event is consumed.
    let maker_fees = quote_native * market.maker_fee;

    let taker_fees = quote_native * market.taker_fee;

    // taker fees should never be negative
    require_gte!(taker_fees, 0);

    // Part of the taker fees that go to the dao, instead of paying for maker rebates
    let taker_dao_fees = (taker_fees + maker_fees.min(I80F48::ZERO)).max(I80F48::ZERO);
    open_orders_acc
        .fixed
        .accrue_buyback_fees(taker_dao_fees.floor().to_num::<u64>());

    open_orders_acc
        .fixed
        .position
        .record_trading_fee(taker_fees);
    open_orders_acc.fixed.position.taker_volume += taker_fees.to_num::<u64>();

    // Accrue maker fees immediately: they can be negative and applying them later
    // risks that fees_accrued is settled to 0 before they apply. It going negative
    // breaks assumptions.
    market.fees_accrued += taker_fees + maker_fees;

    Ok(())
}

/// Applies a fixed penalty fee to the account, and update the market's fees_accrued
fn apply_penalty(market: &mut Market, open_orders_acc: &mut OpenOrdersAccountRefMut) -> Result<()> {
    let fee_penalty = I80F48::from_num(market.fee_penalty);
    open_orders_acc
        .fixed
        .accrue_buyback_fees(fee_penalty.floor().to_num::<u64>());

    open_orders_acc
        .fixed
        .position
        .record_trading_fee(fee_penalty);
    market.fees_accrued += fee_penalty;
    Ok(())
}
