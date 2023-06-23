use crate::logs::TotalOrderFillEvent;
use crate::state::open_orders_account::OpenOrdersLoader;
use crate::state::OpenOrdersAccountRefMut;
use crate::{
    error::*,
    state::{orderbook::bookside::*, EventQueue, Market, OpenOrdersAccountFixed},
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

pub struct OrderWithAmounts {
    pub order_id: Option<u128>,
    pub posted_base_native: i64,
    pub posted_quote_native: i64,
    pub total_base_taken_native: u64,
    pub total_quote_taken_native: u64,
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
    pub fn new_order(
        &mut self,
        order: &Order,
        open_book_market: &mut Market,
        event_queue: &mut EventQueue,
        oracle_price: I80F48,
        mut open_orders_acc: &mut Option<OpenOrdersAccountRefMut>,
        owner: &Pubkey,
        now_ts: u64,
        mut limit: u8,
        open_orders_admin_signer: Option<Pubkey>,
        remaining_accs: &[AccountInfo],
    ) -> std::result::Result<OrderWithAmounts, Error> {
        let market = open_book_market;
        if let Some(open_orders_admin) = Option::<Pubkey>::from(market.open_orders_admin) {
            let open_orders_admin_signer =
                open_orders_admin_signer.ok_or(OpenBookError::MissingOpenOrdersAdmin)?;
            require_eq!(
                open_orders_admin,
                open_orders_admin_signer,
                OpenBookError::InvalidOpenOrdersAdmin
            );
        }

        let side = order.side;

        let other_side = side.invert_side();
        let oracle_price_lots = market.native_price_to_lot(oracle_price)?;
        let post_only = order.is_post_only();
        let mut post_target = order.post_target();
        let (price_lots, price_data) = order.price(now_ts, oracle_price_lots, self)?;

        // generate new order id
        let order_id = market.gen_order_id(side, price_data);

        // Iterate through book and match against this new order.
        //
        // Any changes to matching orders on the other side of the book are collected in
        // matched_changes/matched_deletes and then applied after this loop.

        let order_max_quote_lots = match side {
            Side::Bid => market.subtract_taker_fees(order.max_quote_lots_including_fees),
            Side::Ask => order.max_quote_lots_including_fees,
        };

        let mut remaining_base_lots = order.max_base_lots;
        let mut remaining_quote_lots = order_max_quote_lots;
        let mut decremented_quote_lots = 0_i64;
        let mut referrer_amount = 0_u64;

        let mut matched_order_changes: Vec<(BookSideOrderHandle, i64)> = vec![];
        let mut matched_order_deletes: Vec<(BookSideOrderTree, u128)> = vec![];
        let mut number_of_dropped_expired_orders = 0;

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
                        event_queue.header.seq_num,
                        best_opposing.node.owner,
                        best_opposing.node.quantity,
                    );

                    process_out_event(
                        event,
                        market,
                        event_queue,
                        open_orders_acc,
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
            if max_match_by_quote == 0 {
                break;
            }

            let match_base_lots = remaining_base_lots
                .min(best_opposing.node.quantity)
                .min(max_match_by_quote);
            let match_quote_lots = match_base_lots * best_opposing_price;

            // Self-trade behaviour
            if owner == &best_opposing.node.owner {
                match order.self_trade_behavior {
                    SelfTradeBehavior::DecrementTake => {
                        // remember all decremented quote lots to only charge fees on not-self-trades
                        decremented_quote_lots += match_quote_lots;
                    }
                    SelfTradeBehavior::CancelProvide => {
                        // The open orders acc is always present in this case, no need event_queue
                        open_orders_acc.as_mut().unwrap().cancel_order(
                            best_opposing.node.owner_slot as usize,
                            best_opposing.node.quantity,
                            *market,
                        )?;
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
                event_queue.header.seq_num,
                best_opposing.node.owner,
                best_opposing.node.client_order_id,
                best_opposing.node.timestamp,
                *owner,
                order.client_order_id,
                best_opposing_price,
                match_base_lots,
            );

            process_fill_event(fill, market, event_queue, remaining_accs)?;

            limit -= 1;
        }

        let total_quote_lots_taken = order_max_quote_lots - remaining_quote_lots;
        let total_base_lots_taken = order.max_base_lots - remaining_base_lots;
        assert!(total_quote_lots_taken >= 0);
        assert!(total_base_lots_taken >= 0);

        let total_base_taken_native = total_base_lots_taken
            .checked_mul(market.base_lot_size)
            .ok_or(OpenBookError::InvalidOrderSize)? as u64;

        let mut total_quote_taken_native = total_quote_lots_taken
            .checked_mul(market.quote_lot_size)
            .ok_or(OpenBookError::InvalidOrderSize)?
            as u64;

        // Record the taker trade in the account already, even though it will only be
        // realized when the fill event gets executed
        let mut taker_fees = 0_u64;
        if total_quote_lots_taken > 0 || total_base_lots_taken > 0 {
            let total_quote_taken_native_wo_self =
                ((total_quote_lots_taken - decremented_quote_lots) * market.quote_lot_size) as u64;

            if total_quote_taken_native_wo_self > 0 {
                taker_fees = market.taker_fees_ceil(total_quote_taken_native_wo_self);
                // Only account taker fees now. Maker fees accounted once processing the event
                market.fees_accrued += taker_fees as i64;
            };

            if let Some(open_orders_acc) = &mut open_orders_acc {
                open_orders_acc.release_funds_apply_fees(
                    side,
                    market,
                    total_base_taken_native,
                    total_quote_taken_native,
                    taker_fees,
                )?;
            } else {
                // It's a taker order, transfer to referrer
                referrer_amount += market.referrer_taker_rebate(total_quote_taken_native_wo_self);
            }

            let total_quantity_paid: u64;
            let total_quantity_received: u64;
            match side {
                Side::Bid => {
                    total_quote_taken_native += taker_fees;
                    total_quantity_paid = total_quote_taken_native;
                    total_quantity_received = total_base_taken_native;
                }
                Side::Ask => {
                    total_quote_taken_native -= taker_fees;
                    total_quantity_paid = total_base_taken_native;
                    total_quantity_received = total_quote_taken_native;
                }
            };

            emit!(TotalOrderFillEvent {
                side: side.into(),
                taker: *owner,
                total_quantity_paid,
                total_quantity_received,
                fees: taker_fees,
            });
        } else if order.needs_penalty_fee() {
            // IOC orders have a fee penalty applied if not match to avoid spam
            total_quote_taken_native += market.apply_penalty();
        }

        // Update remaining based on quote_lots taken. If nothing taken, same as the beginning
        remaining_quote_lots =
            order.max_quote_lots_including_fees - total_quote_lots_taken - (taker_fees as i64);

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
        let price = if order.peg_limit() != -1 && order.side == Side::Bid {
            order.peg_limit()
        } else {
            price_lots
        };
        // If there are still quantity unmatched, place on the book
        let book_base_quantity_lots = if market.maker_fee.is_positive() {
            // Subtract fees
            remaining_quote_lots -= market.maker_fees_ceil(remaining_quote_lots);
            remaining_base_lots.min(remaining_quote_lots / price)
        } else {
            remaining_base_lots.min(remaining_quote_lots / price)
        };

        if book_base_quantity_lots <= 0 {
            post_target = None;
        }

        let mut maker_fees = 0;
        let mut posted_base_native = 0;
        let mut posted_quote_native = 0;

        if let Some(order_tree_target) = post_target {
            posted_base_native = book_base_quantity_lots
                .checked_mul(market.base_lot_size)
                .ok_or(OpenBookError::InvalidOrderSize)?;

            posted_quote_native = book_base_quantity_lots
                .checked_mul(price)
                .and_then(|book_quote_lots| book_quote_lots.checked_mul(market.quote_lot_size))
                .ok_or(OpenBookError::InvalidOrderSize)?;

            // Subtract maker fees in bid.
            if market.maker_fee.is_positive() && side == Side::Bid {
                maker_fees = market
                    .maker_fees_ceil(posted_quote_native)
                    .try_into()
                    .unwrap();
            }

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
                process_out_event(
                    event,
                    market,
                    event_queue,
                    open_orders_acc,
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
                    event_queue.header.seq_num,
                    worst_order.owner,
                    worst_order.quantity,
                );
                process_out_event(
                    event,
                    market,
                    event_queue,
                    open_orders_acc,
                    owner,
                    remaining_accs,
                )?;
            }

            // Open orders always exists in this case, unwrap
            let open_orders = open_orders_acc.as_mut().unwrap();
            let owner_slot = open_orders.next_order_slot()?;
            let new_order = LeafNode::new(
                owner_slot as u8,
                order_id,
                *owner,
                book_base_quantity_lots,
                now_ts,
                PostOrderType::Limit, // TODO: Support order types? needed?
                order.time_in_force,
                order.peg_limit(),
                order.client_order_id,
            );
            let _result = bookside.insert_leaf(order_tree_target, &new_order)?;

            // TODO OPT remove if PlaceOrder needs more compute
            msg!(
                "{} on book order_id={} quantity={} price_lots={}",
                match side {
                    Side::Bid => "bid",
                    Side::Ask => "ask",
                },
                order_id,
                book_base_quantity_lots,
                price_lots
            );

            open_orders.add_order(
                side,
                order_tree_target,
                &new_order,
                order.client_order_id,
                price,
            )?;
        }

        let placed_order_id = if post_target.is_some() {
            Some(order_id)
        } else {
            None
        };

        Ok(OrderWithAmounts {
            order_id: placed_order_id,
            posted_base_native,
            posted_quote_native,
            total_base_taken_native,
            total_quote_taken_native,
            referrer_amount,
            maker_fees,
        })
    }

    /// Cancels up to `limit` orders that are listed on the openorders account for the given market.
    /// Optionally filters by `side_to_cancel_option`.
    /// The orders are removed from the book and from the openorders account open order list.
    pub fn cancel_all_orders(
        &mut self,
        open_orders_acc: &mut OpenOrdersAccountRefMut,
        market: Market,
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

            if limit == 0 {
                msg!("Cancel orders limit reached");
                break;
            }

            let order_id = oo.id;

            let cancel_result =
                self.cancel_order(open_orders_acc, order_id, order_side_and_tree, market, None);
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
        }
        Ok(())
    }

    /// Cancels an order on a side, removing it from the book and the openorders account orders list
    pub fn cancel_order(
        &mut self,
        open_orders_acc: &mut OpenOrdersAccountRefMut,
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
        open_orders_acc.cancel_order(leaf_node.owner_slot as usize, leaf_node.quantity, market)?;

        Ok(leaf_node)
    }
}

pub fn process_out_event(
    event: OutEvent,
    market: &Market,
    event_queue: &mut EventQueue,
    mut open_orders_acc: &mut Option<OpenOrdersAccountRefMut>,
    owner: &Pubkey,
    remaining_accs: &[AccountInfo],
) -> Result<()> {
    if let Some(acc) = &mut open_orders_acc {
        if owner == &event.owner {
            acc.cancel_order(event.owner_slot as usize, event.quantity, *market)?;
            // Already canceled, return
            return Ok(());
        }
    }
    // Check if remaining is available so no event is pushed to event_queue
    if let Some(acc) = remaining_accs.iter().find(|ai| ai.key == &event.owner) {
        let ooa: AccountLoader<OpenOrdersAccountFixed> = AccountLoader::try_from(acc)?;
        let mut acc = ooa.load_full_mut()?;
        acc.cancel_order(event.owner_slot as usize, event.quantity, *market)?;
    } else {
        event_queue.push_back(cast(event));
    }
    Ok(())
}

pub fn process_fill_event(
    event: FillEvent,
    market: &mut Market,
    event_queue: &mut EventQueue,
    remaining_accs: &[AccountInfo],
) -> Result<()> {
    let loader = remaining_accs.iter().find(|ai| ai.key == &event.maker);
    if let Some(acc) = loader {
        let ooa: AccountLoader<OpenOrdersAccountFixed> = AccountLoader::try_from(acc)?;
        let mut maker = ooa.load_full_mut()?;

        maker.execute_maker(market, &event)?;
    } else {
        event_queue.push_back(cast(event));
    }
    Ok(())
}
