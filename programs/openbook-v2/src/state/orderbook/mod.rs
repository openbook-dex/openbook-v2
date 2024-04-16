pub use book::*;
pub use bookside::*;
pub use bookside_iterator::*;
pub use heap::*;
pub use nodes::*;
pub use order::*;
pub use order_type::*;
pub use ordertree::*;
pub use ordertree_iterator::*;

mod book;
mod bookside;
mod bookside_iterator;
mod heap;
mod nodes;
mod order;
mod order_type;
mod ordertree;
mod ordertree_iterator;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Market, OpenOrdersAccount, FEES_SCALE_FACTOR};
    use bytemuck::Zeroable;
    use fixed::types::I80F48;
    use solana_program::pubkey::Pubkey;
    use std::cell::RefCell;

    fn order_tree_leaf_by_key(bookside: &BookSide, key: u128) -> Option<&LeafNode> {
        for component in [BookSideOrderTree::Fixed, BookSideOrderTree::OraclePegged] {
            for (_, leaf) in bookside.nodes.iter(bookside.root(component)) {
                if leaf.key == key {
                    return Some(leaf);
                }
            }
        }
        None
    }

    fn order_tree_contains_key(bookside: &BookSide, key: u128) -> bool {
        order_tree_leaf_by_key(bookside, key).is_some()
    }

    fn order_tree_contains_price(bookside: &BookSide, price_data: u64) -> bool {
        for component in [BookSideOrderTree::Fixed, BookSideOrderTree::OraclePegged] {
            for (_, leaf) in bookside.nodes.iter(bookside.root(component)) {
                if leaf.price_data() == price_data {
                    return true;
                }
            }
        }
        false
    }

    struct OrderbookAccounts {
        bids: Box<RefCell<BookSide>>,
        asks: Box<RefCell<BookSide>>,
    }

    impl OrderbookAccounts {
        fn new() -> Self {
            let s = Self {
                bids: Box::new(RefCell::new(BookSide::zeroed())),
                asks: Box::new(RefCell::new(BookSide::zeroed())),
            };
            s.bids.borrow_mut().nodes.order_tree_type = OrderTreeType::Bids.into();
            s.asks.borrow_mut().nodes.order_tree_type = OrderTreeType::Asks.into();
            s
        }

        fn orderbook(&self) -> Orderbook {
            Orderbook {
                bids: self.bids.borrow_mut(),
                asks: self.asks.borrow_mut(),
            }
        }
    }

    fn test_setup(price: f64) -> (Market, Option<i64>, EventHeap, OrderbookAccounts) {
        let book = OrderbookAccounts::new();

        let event_heap = EventHeap::zeroed();

        let mut openbook_market = Market::zeroed();
        openbook_market.quote_lot_size = 1;
        openbook_market.base_lot_size = 1;

        let oracle_price_lots = openbook_market
            .native_price_to_lot(I80F48::from_num(price))
            .ok();

        (openbook_market, oracle_price_lots, event_heap, book)
    }

    // Check what happens when one side of the book fills up
    #[test]
    fn book_bids_full() {
        let (mut openbook_market, oracle_price_lots, mut event_heap, book_accs) =
            test_setup(5000.0);
        let mut book = book_accs.orderbook();
        let market_pk = Pubkey::new_unique();

        let mut new_order =
            |book: &mut Orderbook, event_heap: &mut EventHeap, side, price_lots, now_ts| -> u128 {
                let mut account = OpenOrdersAccount::default_for_tests();

                let max_base_lots = 1;
                let time_in_force = 100;

                book.new_order(
                    &Order {
                        side,
                        max_base_lots,
                        max_quote_lots_including_fees: i64::MAX / openbook_market.quote_lot_size,
                        client_order_id: 0,
                        time_in_force,
                        params: OrderParams::Fixed {
                            price_lots,
                            order_type: PostOrderType::Limit,
                        },
                        self_trade_behavior: SelfTradeBehavior::DecrementTake,
                    },
                    &mut openbook_market,
                    &market_pk,
                    event_heap,
                    oracle_price_lots,
                    Some(&mut account),
                    &Pubkey::new_unique(),
                    now_ts,
                    u8::MAX,
                    &[],
                )
                .unwrap();
                account.open_order_by_raw_index(0).id
            };

        // insert bids until book side is full
        for i in 1..10 {
            new_order(
                &mut book,
                &mut event_heap,
                Side::Bid,
                1000 + i as i64,
                1000000 + i as u64,
            );
        }
        for i in 10..1000 {
            new_order(
                &mut book,
                &mut event_heap,
                Side::Bid,
                1000 + i as i64,
                1000011_u64,
            );
            if book.bids.is_full() {
                break;
            }
        }
        assert!(book.bids.is_full());
        assert_eq!(
            book.bids
                .nodes
                .min_leaf(&book.bids.roots[0])
                .unwrap()
                .1
                .price_data(),
            1001
        );
        assert_eq!(
            fixed_price_lots(
                book.bids
                    .nodes
                    .max_leaf(&book.bids.roots[0])
                    .unwrap()
                    .1
                    .price_data()
            ),
            (1000 + book.bids.roots[0].leaf_count) as i64
        );

        // add another bid at a higher price before expiry, replacing the lowest-price one (1001)
        new_order(&mut book, &mut event_heap, Side::Bid, 1005, 1000000 - 1);
        assert_eq!(
            book.bids
                .nodes
                .min_leaf(&book.bids.roots[0])
                .unwrap()
                .1
                .price_data(),
            1002
        );
        assert_eq!(event_heap.len(), 1);

        // adding another bid after expiry removes the soonest-expiring order (1005)
        new_order(&mut book, &mut event_heap, Side::Bid, 999, 2000000);
        assert_eq!(
            book.bids
                .nodes
                .min_leaf(&book.bids.roots[0])
                .unwrap()
                .1
                .price_data(),
            999
        );
        assert!(!order_tree_contains_key(&book.bids, 1005));
        assert_eq!(event_heap.len(), 2);

        // adding an ask will wipe up to three expired bids at the top of the book
        let bids_max = book
            .bids
            .nodes
            .max_leaf(&book.bids.roots[0])
            .unwrap()
            .1
            .price_data();
        let bids_count = book.bids.roots[0].leaf_count;
        new_order(&mut book, &mut event_heap, Side::Ask, 6000, 1500000);
        assert_eq!(book.bids.roots[0].leaf_count, bids_count - 5);
        assert_eq!(book.asks.roots[0].leaf_count, 1);
        assert_eq!(event_heap.len(), 2 + 5);
        assert!(!order_tree_contains_price(&book.bids, bids_max));
        assert!(!order_tree_contains_price(&book.bids, bids_max - 1));
        assert!(!order_tree_contains_price(&book.bids, bids_max - 2));
        assert!(!order_tree_contains_price(&book.bids, bids_max - 3));
        assert!(!order_tree_contains_price(&book.bids, bids_max - 4));
        assert!(order_tree_contains_price(&book.bids, bids_max - 5));
    }

    #[test]
    fn book_new_order() {
        let (mut market, oracle_price_lots, mut event_heap, book_accs) = test_setup(1000.0);
        let mut book = book_accs.orderbook();
        let market_pk = Pubkey::new_unique();

        // Add lots and fees to make sure to exercise unit conversion
        market.base_lot_size = 10;
        market.quote_lot_size = 100;
        let maker_fee = 100;
        let taker_fee = 1000;
        market.maker_fee = maker_fee;
        market.taker_fee = taker_fee;

        let mut maker = OpenOrdersAccount::default_for_tests();
        let mut taker = OpenOrdersAccount::default_for_tests();

        let maker_pk = Pubkey::new_unique();
        let taker_pk = Pubkey::new_unique();
        let now_ts = 1000000;

        // Place a maker-bid
        let price_lots = 1000 * market.base_lot_size / market.quote_lot_size;
        let bid_quantity = 10;
        book.new_order(
            &Order {
                side: Side::Bid,
                max_base_lots: bid_quantity,
                max_quote_lots_including_fees: i64::MAX / market.quote_lot_size,
                client_order_id: 42,
                time_in_force: 0,
                params: OrderParams::Fixed {
                    price_lots,
                    order_type: PostOrderType::Limit,
                },
                self_trade_behavior: SelfTradeBehavior::DecrementTake,
            },
            &mut market,
            &market_pk,
            &mut event_heap,
            oracle_price_lots,
            Some(&mut maker),
            &maker_pk,
            now_ts,
            u8::MAX,
            &[],
        )
        .unwrap();
        let order =
            order_tree_leaf_by_key(&book.bids, maker.open_order_by_raw_index(0).id).unwrap();
        assert_eq!(order.client_order_id, 42);
        assert_eq!(order.quantity, bid_quantity);
        assert!(maker.open_order_by_raw_index(1).is_free());
        assert_ne!(maker.open_order_by_raw_index(0).id, 0);
        assert_eq!(maker.open_order_by_raw_index(0).client_id, 42);
        assert_eq!(
            maker.open_order_by_raw_index(0).side_and_tree(),
            SideAndOrderTree::BidFixed
        );
        assert!(order_tree_contains_key(
            &book.bids,
            maker.open_order_by_raw_index(0).id
        ));
        assert!(order_tree_contains_price(&book.bids, price_lots as u64));
        assert_eq!(maker.position.bids_base_lots, bid_quantity);
        assert_eq!(maker.position.bids_quote_lots, bid_quantity * price_lots);
        assert_eq!(maker.position.asks_base_lots, 0);
        assert_eq!(event_heap.len(), 0);

        // Take the order partially
        let match_quantity = 5;
        book.new_order(
            &Order {
                side: Side::Ask,
                max_base_lots: match_quantity,
                max_quote_lots_including_fees: i64::MAX / market.quote_lot_size,
                client_order_id: 43,
                time_in_force: 0,
                params: OrderParams::Fixed {
                    price_lots,
                    order_type: PostOrderType::Limit,
                },
                self_trade_behavior: SelfTradeBehavior::DecrementTake,
            },
            &mut market,
            &market_pk,
            &mut event_heap,
            oracle_price_lots,
            Some(&mut taker),
            &taker_pk,
            now_ts,
            u8::MAX,
            &[],
        )
        .unwrap();
        // the remainder of the maker order is still on the book
        // (the maker account is unchanged: it was not even passed in)
        let order =
            order_tree_leaf_by_key(&book.bids, maker.open_order_by_raw_index(0).id).unwrap();
        assert_eq!(fixed_price_lots(order.price_data()), price_lots);
        assert_eq!(order.quantity, bid_quantity - match_quantity);

        // fees were immediately accrued
        let match_quote = match_quantity * price_lots * market.quote_lot_size;
        assert_eq!(
            market.fees_accrued as i64,
            match_quote * (taker_fee) / (FEES_SCALE_FACTOR as i64)
        );

        // the taker account is updated
        assert!(taker.open_order_by_raw_index(1).is_free());
        assert_eq!(taker.position.bids_base_lots, 0);
        assert_eq!(taker.position.bids_quote_lots, 0);
        assert_eq!(taker.position.asks_base_lots, 0);
        // the fill gets added to the event heap
        assert_eq!(event_heap.len(), 1);
        let event = event_heap.front().unwrap();
        assert_eq!(event.event_type, EventType::Fill as u8);
        let fill: &FillEvent = bytemuck::cast_ref(event);
        assert_eq!(fill.quantity, match_quantity);
        assert_eq!(fill.price, price_lots);
        assert_eq!(fill.taker_client_order_id, 43);
        assert_eq!(fill.maker, maker_pk);
        assert_eq!(fill.taker, taker_pk);

        // simulate event heap processing
        maker.execute_maker(&mut market, fill);
        taker.execute_taker(&mut market, Side::Ask, 0, 0, 0, 0);

        assert_eq!(maker.position.bids_base_lots, bid_quantity - match_quantity);
        assert_eq!(maker.position.asks_base_lots, 0);

        assert_eq!(taker.position.bids_base_lots, 0);
        assert_eq!(taker.position.bids_quote_lots, 0);
        assert_eq!(taker.position.asks_base_lots, 0);
        // Maker fee is accrued now
        assert_eq!(
            market.fees_accrued as i64,
            match_quote * (maker_fee + taker_fee) / (FEES_SCALE_FACTOR as i64)
        );
    }

    // Check that there are no zero-quantity fills when max_quote_lots is not
    // enough for a single lot
    #[test]
    fn book_max_quote_lots() {
        let (mut market, oracle_price_lots, mut event_heap, book_accs) = test_setup(5000.0);
        let quote_lot_size = market.quote_lot_size;
        let mut book = book_accs.orderbook();
        let market_pk = Pubkey::new_unique();

        let mut new_order = |book: &mut Orderbook,
                             event_heap: &mut EventHeap,
                             side,
                             price_lots,
                             max_base_lots: i64,
                             max_quote_lots_including_fees: i64|
         -> u128 {
            let mut account = OpenOrdersAccount::default_for_tests();

            book.new_order(
                &Order {
                    side,
                    max_base_lots,
                    max_quote_lots_including_fees,
                    client_order_id: 0,
                    time_in_force: 0,
                    params: OrderParams::Fixed {
                        price_lots,
                        order_type: PostOrderType::Limit,
                    },
                    self_trade_behavior: SelfTradeBehavior::DecrementTake,
                },
                &mut market,
                &market_pk,
                event_heap,
                oracle_price_lots,
                Some(&mut account),
                &Pubkey::default(),
                0, // now_ts
                u8::MAX,
                &[],
            )
            .unwrap();
            account.open_order_by_raw_index(0).id
        };

        // Setup
        new_order(
            &mut book,
            &mut event_heap,
            Side::Ask,
            5000,
            5,
            i64::MAX / quote_lot_size,
        );
        new_order(
            &mut book,
            &mut event_heap,
            Side::Ask,
            5001,
            5,
            i64::MAX / quote_lot_size,
        );
        new_order(
            &mut book,
            &mut event_heap,
            Side::Ask,
            5002,
            5,
            i64::MAX / quote_lot_size,
        );

        // Try taking: the quote limit allows only one base lot to be taken.
        new_order(&mut book, &mut event_heap, Side::Bid, 5005, 30, 6000);
        // Only one fill event is generated, the matching aborts even though neither the base nor quote limit
        // is exhausted.
        assert_eq!(event_heap.len(), 1);

        // Try taking: the quote limit allows no fills
        new_order(&mut book, &mut event_heap, Side::Bid, 5005, 30, 1);
        assert_eq!(event_heap.len(), 1);
    }
}
