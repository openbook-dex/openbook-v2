use super::*;

pub struct BookSideIterItem<'a> {
    pub handle: BookSideOrderHandle,
    pub node: &'a LeafNode,
    pub price_lots: i64,
    pub state: OrderState,
}

impl<'a> BookSideIterItem<'a> {
    pub fn is_valid(&self) -> bool {
        self.state == OrderState::Valid
    }
}

/// Iterates the fixed and oracle_pegged OrderTrees simultaneously, allowing users to
/// walk the orderbook without caring about where an order came from.
///
/// This will skip over orders that are not currently matchable, but might be valid
/// in the future.
///
/// This may return invalid orders (tif expired, peg_limit exceeded; see is_valid) which
/// users are supposed to remove from the orderbook if they can.
pub struct BookSideIter<'a> {
    fixed_iter: OrderTreeIter<'a>,
    oracle_pegged_iter: OrderTreeIter<'a>,
    now_ts: u64,
    oracle_price_lots: i64,
}

impl<'a> BookSideIter<'a> {
    pub fn new(book_side: &'a BookSide, now_ts: u64, oracle_price_lots: i64) -> Self {
        Self {
            fixed_iter: book_side
                .nodes
                .iter(book_side.root(BookSideOrderTree::Fixed)),
            oracle_pegged_iter: book_side
                .nodes
                .iter(book_side.root(BookSideOrderTree::OraclePegged)),
            now_ts,
            oracle_price_lots,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum OrderState {
    Valid,
    Invalid,
    Skipped,
}

/// Returns the state and current price of an oracle pegged order.
///
/// For pegged orders with offsets that let the price escape the 1..i64::MAX range,
/// this function returns Skipped and clamps `price` to that range.
///
/// Orders that exceed their peg_limit will have Invalid state.
fn oracle_pegged_price(oracle_price_lots: i64, node: &LeafNode, side: Side) -> (OrderState, i64) {
    let price_data = node.price_data();
    let price_offset = oracle_pegged_price_offset(price_data);
    let price = oracle_price_lots.saturating_add(price_offset);
    if (1..i64::MAX).contains(&price) {
        if node.peg_limit != -1 && side.is_price_better(price, node.peg_limit) {
            return (OrderState::Invalid, price);
        } else {
            return (OrderState::Valid, price);
        }
    }
    (OrderState::Skipped, price.max(1))
}

/// Replace the price data in a binary tree `key` with the fixed order price data at `price_lots`.
///
/// Used to convert oracle pegged keys into a form that allows comparison with fixed order keys.
fn key_for_fixed_price(key: u128, price_lots: i64) -> u128 {
    // We know this can never fail, because oracle pegged price will always be >= 1
    assert!(price_lots >= 1);
    let price_data = fixed_price_data(price_lots).unwrap();
    let upper = (price_data as u128) << 64;
    let lower = (key as u64) as u128;
    upper | lower
}

/// Helper for the iterator returning a fixed order
fn fixed_to_result(fixed: (NodeHandle, &LeafNode), now_ts: u64) -> BookSideIterItem {
    let (handle, node) = fixed;
    let expired = node.is_expired(now_ts);
    BookSideIterItem {
        handle: BookSideOrderHandle {
            order_tree: BookSideOrderTree::Fixed,
            node: handle,
        },
        node,
        price_lots: fixed_price_lots(node.price_data()),
        state: if expired {
            OrderState::Invalid
        } else {
            OrderState::Valid
        },
    }
}

/// Helper for the iterator returning a pegged order
fn oracle_pegged_to_result(
    pegged: (NodeHandle, &LeafNode, i64, OrderState),
    now_ts: u64,
) -> BookSideIterItem {
    let (handle, node, price_lots, state) = pegged;
    let expired = node.is_expired(now_ts);
    BookSideIterItem {
        handle: BookSideOrderHandle {
            order_tree: BookSideOrderTree::OraclePegged,
            node: handle,
        },
        node,
        price_lots,
        state: if expired { OrderState::Invalid } else { state },
    }
}

/// Compares the `fixed` and `oracle_pegged` order and returns the one that would match first.
///
/// (or the worse one, if `return_worse` is set)
pub fn rank_orders<'a>(
    side: Side,
    fixed: Option<(NodeHandle, &'a LeafNode)>,
    oracle_pegged: Option<(NodeHandle, &'a LeafNode)>,
    return_worse: bool,
    now_ts: u64,
    oracle_price_lots: i64,
) -> Option<BookSideIterItem<'a>> {
    // Enrich with data that'll always be needed
    let oracle_pegged = oracle_pegged.map(|(handle, node)| {
        let (state, price_lots) = oracle_pegged_price(oracle_price_lots, node, side);
        (handle, node, price_lots, state)
    });

    match (fixed, oracle_pegged) {
        (Some(f), Some(o)) => {
            let is_better = if side == Side::Bid {
                |a, b| a > b
            } else {
                |a, b| a < b
            };

            if is_better(f.1.key, key_for_fixed_price(o.1.key, o.2)) ^ return_worse {
                Some(fixed_to_result(f, now_ts))
            } else {
                Some(oracle_pegged_to_result(o, now_ts))
            }
        }
        (None, Some(o)) => Some(oracle_pegged_to_result(o, now_ts)),
        (Some(f), None) => Some(fixed_to_result(f, now_ts)),
        (None, None) => None,
    }
}

impl<'a> Iterator for BookSideIter<'a> {
    type Item = BookSideIterItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let side = self.fixed_iter.side();

        // Skip all the oracle pegged orders that aren't representable with the current oracle
        // price. Example: iterating asks, but the best ask is at offset -100 with the oracle at 50.
        // We need to skip asks until we find the first that has a price >= 1.
        let mut o_peek = self.oracle_pegged_iter.peek();
        while let Some((_, o_node)) = o_peek {
            if oracle_pegged_price(self.oracle_price_lots, o_node, side).0 != OrderState::Skipped {
                break;
            }
            o_peek = self.oracle_pegged_iter.next()
        }

        let f_peek = self.fixed_iter.peek();

        let better = rank_orders(
            side,
            f_peek,
            o_peek,
            false,
            self.now_ts,
            self.oracle_price_lots,
        )?;
        match better.handle.order_tree {
            BookSideOrderTree::Fixed => self.fixed_iter.next(),
            BookSideOrderTree::OraclePegged => self.oracle_pegged_iter.next(),
        };

        Some(better)
    }
}
