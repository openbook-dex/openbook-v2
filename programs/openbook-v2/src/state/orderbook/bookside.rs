use anchor_lang::prelude::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use static_assertions::const_assert_eq;

use super::*;

#[derive(
    Eq,
    PartialEq,
    Copy,
    Clone,
    TryFromPrimitive,
    IntoPrimitive,
    Debug,
    AnchorSerialize,
    AnchorDeserialize,
)]
#[repr(u8)]
pub enum BookSideOrderTree {
    Fixed = 0,
    OraclePegged = 1,
}

/// Reference to a node in a book side component
pub struct BookSideOrderHandle {
    pub node: NodeHandle,
    pub order_tree: BookSideOrderTree,
}

#[account(zero_copy)]
pub struct BookSide {
    pub roots: [OrderTreeRoot; 2],
    pub reserved_roots: [OrderTreeRoot; 4],
    pub reserved: [u8; 256],
    pub nodes: OrderTreeNodes,
}
const_assert_eq!(
    std::mem::size_of::<BookSide>(),
    std::mem::size_of::<OrderTreeNodes>() + 6 * std::mem::size_of::<OrderTreeRoot>() + 256
);
const_assert_eq!(std::mem::size_of::<BookSide>(), 90944);
const_assert_eq!(std::mem::size_of::<BookSide>() % 8, 0);

impl BookSide {
    /// Iterate over all entries in the book filtering out invalid orders
    ///
    /// smallest to highest for asks
    /// highest to smallest for bids
    pub fn iter_valid(
        &self,
        now_ts: u64,
        oracle_price_lots: Option<i64>,
    ) -> impl Iterator<Item = BookSideIterItem> {
        BookSideIter::new(self, now_ts, oracle_price_lots).filter(|it| it.is_valid())
    }

    /// Iterate over all entries, including invalid orders
    pub fn iter_all_including_invalid(
        &self,
        now_ts: u64,
        oracle_price_lots: Option<i64>,
    ) -> BookSideIter {
        BookSideIter::new(self, now_ts, oracle_price_lots)
    }

    pub fn node(&self, handle: NodeHandle) -> Option<&AnyNode> {
        self.nodes.node(handle)
    }

    pub fn node_mut(&mut self, handle: NodeHandle) -> Option<&mut AnyNode> {
        self.nodes.node_mut(handle)
    }

    pub fn root(&self, component: BookSideOrderTree) -> &OrderTreeRoot {
        &self.roots[component as usize]
    }

    pub fn root_mut(&mut self, component: BookSideOrderTree) -> &mut OrderTreeRoot {
        &mut self.roots[component as usize]
    }

    pub fn is_full(&self) -> bool {
        self.nodes.is_full()
    }

    pub fn is_empty(&self) -> bool {
        [BookSideOrderTree::Fixed, BookSideOrderTree::OraclePegged]
            .into_iter()
            .all(|component| self.nodes.iter(self.root(component)).count() == 0)
    }

    pub fn insert_leaf(
        &mut self,
        component: BookSideOrderTree,
        new_leaf: &LeafNode,
    ) -> Result<(NodeHandle, Option<LeafNode>)> {
        let root = &mut self.roots[component as usize];
        self.nodes.insert_leaf(root, new_leaf)
    }

    /// Remove the overall worst-price order.
    pub fn remove_worst(
        &mut self,
        now_ts: u64,
        oracle_price_lots: Option<i64>,
    ) -> Option<(LeafNode, i64)> {
        let worst_fixed = self.nodes.find_worst(&self.roots[0]);
        let worst_pegged = self.nodes.find_worst(&self.roots[1]);
        let side = self.nodes.order_tree_type().side();
        let worse = rank_orders(
            side,
            worst_fixed,
            worst_pegged,
            true,
            now_ts,
            oracle_price_lots,
        )?;
        let price = worse.price_lots;
        let key = worse.node.key;
        let order_tree = worse.handle.order_tree;
        let n = self.remove_by_key(order_tree, key)?;
        Some((n, price))
    }

    /// Remove the order with the lowest expiry timestamp in the component, if that's < now_ts.
    /// If there is none, try to remove the lowest expiry one from the other component.
    pub fn remove_one_expired(
        &mut self,
        component: BookSideOrderTree,
        now_ts: u64,
    ) -> Option<LeafNode> {
        let root = &mut self.roots[component as usize];
        if let Some(n) = self.nodes.remove_one_expired(root, now_ts) {
            return Some(n);
        }

        let other_component = match component {
            BookSideOrderTree::Fixed => BookSideOrderTree::OraclePegged,
            BookSideOrderTree::OraclePegged => BookSideOrderTree::Fixed,
        };
        let other_root = &mut self.roots[other_component as usize];
        self.nodes.remove_one_expired(other_root, now_ts)
    }

    pub fn remove_by_key(
        &mut self,
        component: BookSideOrderTree,
        search_key: u128,
    ) -> Option<LeafNode> {
        let root = &mut self.roots[component as usize];
        self.nodes.remove_by_key(root, search_key)
    }

    pub fn side(&self) -> Side {
        self.nodes.order_tree_type().side()
    }

    /// Return the quantity of orders that can be matched by an order at `limit_price_lots`
    pub fn quantity_at_price(
        &self,
        limit_price_lots: i64,
        now_ts: u64,
        oracle_price_lots: i64,
    ) -> i64 {
        let side = self.side();
        let mut sum = 0;
        for item in self.iter_valid(now_ts, Some(oracle_price_lots)) {
            if side.is_price_better(limit_price_lots, item.price_lots) {
                break;
            }
            sum += item.node.quantity;
        }
        sum
    }

    /// Return the price of the order closest to the spread
    pub fn best_price(&self, now_ts: u64, oracle_price_lots: Option<i64>) -> Option<i64> {
        Some(
            self.iter_valid(now_ts, oracle_price_lots)
                .next()?
                .price_lots,
        )
    }

    /// Walk up the book `quantity` units and return the price at that level. If `quantity` units
    /// not on book, return None
    pub fn impact_price(&self, quantity: i64, now_ts: u64, oracle_price_lots: i64) -> Option<i64> {
        let mut sum: i64 = 0;
        for order in self.iter_valid(now_ts, Some(oracle_price_lots)) {
            sum += order.node.quantity;
            if sum >= quantity {
                return Some(order.price_lots);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck::Zeroable;

    fn new_order_tree(order_tree_type: OrderTreeType) -> OrderTreeNodes {
        let mut ot = OrderTreeNodes::zeroed();
        ot.order_tree_type = order_tree_type.into();
        ot
    }

    fn bookside_iteration_random_helper(side: Side) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let order_tree_type = match side {
            Side::Bid => OrderTreeType::Bids,
            Side::Ask => OrderTreeType::Asks,
        };

        let mut order_tree = new_order_tree(order_tree_type);
        let mut root_fixed = OrderTreeRoot::zeroed();
        let mut root_pegged = OrderTreeRoot::zeroed();
        let new_leaf = |key: u128| LeafNode::new(0, key, Pubkey::default(), 0, 1, 0, -1, 0);

        // add 100 leaves to each BookSide, mostly random
        let mut keys = vec![];

        // ensure at least one oracle pegged order visible even at oracle price 1
        let key = new_node_key(side, oracle_pegged_price_data(20), 0);
        keys.push(key);
        order_tree
            .insert_leaf(&mut root_pegged, &new_leaf(key))
            .unwrap();

        while root_pegged.leaf_count < 100 {
            let price_data: u64 = oracle_pegged_price_data(rng.gen_range(-20..20));
            let seq_num: u64 = rng.gen_range(0..1000);
            let key = new_node_key(side, price_data, seq_num);
            if keys.contains(&key) {
                continue;
            }
            keys.push(key);
            order_tree
                .insert_leaf(&mut root_pegged, &new_leaf(key))
                .unwrap();
        }

        while root_fixed.leaf_count < 100 {
            let price_data: u64 = rng.gen_range(1..50);
            let seq_num: u64 = rng.gen_range(0..1000);
            let key = new_node_key(side, price_data, seq_num);
            if keys.contains(&key) {
                continue;
            }
            keys.push(key);
            order_tree
                .insert_leaf(&mut root_fixed, &new_leaf(key))
                .unwrap();
        }

        let bookside = BookSide {
            roots: [root_fixed, root_pegged],
            reserved_roots: [OrderTreeRoot::zeroed(); 4],
            reserved: [0; 256],
            nodes: order_tree,
        };

        // verify iteration order for different oracle prices
        for oracle_price_lots in 1..40 {
            let mut total = 0;
            let ascending = order_tree_type == OrderTreeType::Asks;
            let mut last_price = if ascending { 0 } else { i64::MAX };
            for order in bookside.iter_all_including_invalid(0, Some(oracle_price_lots)) {
                let price = order.price_lots;
                println!("{} {:?} {price}", order.node.key, order.handle.order_tree);
                if ascending {
                    assert!(price >= last_price);
                } else {
                    assert!(price <= last_price);
                }
                last_price = price;
                total += 1;
            }
            assert!(total >= 101); // some oracle peg orders could be skipped
            if oracle_price_lots > 20 {
                assert_eq!(total, 200);
            }
        }
    }

    #[test]
    fn bookside_iteration_random() {
        bookside_iteration_random_helper(Side::Bid);
        bookside_iteration_random_helper(Side::Ask);
    }

    fn bookside_setup() -> BookSide {
        use std::cell::RefCell;

        let side = Side::Bid;
        let order_tree_type = OrderTreeType::Bids;

        let order_tree = RefCell::new(new_order_tree(order_tree_type));
        let mut root_fixed = OrderTreeRoot::zeroed();
        let mut root_pegged = OrderTreeRoot::zeroed();
        let new_node = |key: u128, tif: u16, peg_limit: i64| {
            LeafNode::new(0, key, Pubkey::default(), 0, 1000, tif, peg_limit, 0)
        };
        let mut add_fixed = |price: i64, tif: u16| {
            let key = new_node_key(side, fixed_price_data(price).unwrap(), 0);
            order_tree
                .borrow_mut()
                .insert_leaf(&mut root_fixed, &new_node(key, tif, -1))
                .unwrap();
        };
        let mut add_pegged = |price_offset: i64, tif: u16, peg_limit: i64| {
            let key = new_node_key(side, oracle_pegged_price_data(price_offset), 0);
            order_tree
                .borrow_mut()
                .insert_leaf(&mut root_pegged, &new_node(key, tif, peg_limit))
                .unwrap();
        };

        add_fixed(100, 0);
        add_fixed(120, 5);
        add_pegged(-10, 0, 100);
        add_pegged(-15, 0, -1);
        add_pegged(-20, 7, 95);

        BookSide {
            roots: [root_fixed, root_pegged],
            reserved_roots: [OrderTreeRoot::zeroed(); 4],
            reserved: [0; 256],
            nodes: order_tree.into_inner(),
        }
    }

    #[test]
    fn bookside_order_filtering() {
        let bookside = bookside_setup();

        let order_prices = |now_ts: u64, oracle: i64| -> Vec<i64> {
            bookside
                .iter_valid(now_ts, Some(oracle))
                .map(|it| it.price_lots)
                .collect()
        };

        assert_eq!(order_prices(0, 100), vec![120, 100, 90, 85, 80]);
        assert_eq!(order_prices(1004, 100), vec![120, 100, 90, 85, 80]);
        assert_eq!(order_prices(1005, 100), vec![100, 90, 85, 80]);
        assert_eq!(order_prices(1006, 100), vec![100, 90, 85, 80]);
        assert_eq!(order_prices(1007, 100), vec![100, 90, 85]);
        assert_eq!(order_prices(0, 110), vec![120, 100, 100, 95, 90]);
        assert_eq!(order_prices(0, 111), vec![120, 100, 96, 91]);
        assert_eq!(order_prices(0, 115), vec![120, 100, 100, 95]);
        assert_eq!(order_prices(0, 116), vec![120, 101, 100]);
        assert_eq!(order_prices(0, 2015), vec![2000, 120, 100]);
        assert_eq!(order_prices(1010, 2015), vec![2000, 100]);
    }

    #[test]
    fn bookside_remove_worst() {
        use std::cell::RefCell;

        let bookside = RefCell::new(bookside_setup());

        let order_prices = |now_ts: u64, oracle: i64| -> Vec<i64> {
            bookside
                .borrow()
                .iter_valid(now_ts, Some(oracle))
                .map(|it| it.price_lots)
                .collect()
        };

        // remove pegged order
        assert_eq!(order_prices(0, 100), vec![120, 100, 90, 85, 80]);
        let (_, p) = bookside.borrow_mut().remove_worst(0, Some(100)).unwrap();
        assert_eq!(p, 80);
        assert_eq!(order_prices(0, 100), vec![120, 100, 90, 85]);

        // remove fixed order (order at 190=200-10 hits the peg limit)
        assert_eq!(order_prices(0, 200), vec![185, 120, 100]);
        let (_, p) = bookside.borrow_mut().remove_worst(0, Some(200)).unwrap();
        assert_eq!(p, 100);
        assert_eq!(order_prices(0, 200), vec![185, 120]);

        // remove until end

        assert_eq!(order_prices(0, 100), vec![120, 90, 85]);
        let (_, p) = bookside.borrow_mut().remove_worst(0, Some(100)).unwrap();
        assert_eq!(p, 85);
        assert_eq!(order_prices(0, 100), vec![120, 90]);
        let (_, p) = bookside.borrow_mut().remove_worst(0, Some(100)).unwrap();
        assert_eq!(p, 90);
        assert_eq!(order_prices(0, 100), vec![120]);
        let (_, p) = bookside.borrow_mut().remove_worst(0, Some(100)).unwrap();
        assert_eq!(p, 120);
        assert_eq!(order_prices(0, 100), Vec::<i64>::new());
    }

    // add test for oracle expired
}
