use std::mem::{align_of, size_of};

use anchor_lang::prelude::*;
use bytemuck::{cast_mut, cast_ref};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use static_assertions::const_assert_eq;

use super::order_type::Side;

pub type NodeHandle = u32;
const NODE_SIZE: usize = 88;

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum NodeTag {
    Uninitialized = 0,
    InnerNode = 1,
    LeafNode = 2,
    FreeNode = 3,
    LastFreeNode = 4,
}

/// Creates a binary tree node key.
///
/// It's used for sorting nodes (ascending for asks, descending for bids)
/// and encodes price data in the top 64 bits followed by an ordering number
/// in the lower bits.
///
/// The `seq_num` that's passed should monotonically increase. It's used to choose
/// the ordering number such that orders placed later for the same price data
/// are ordered after earlier orders.
pub fn new_node_key(side: Side, price_data: u64, seq_num: u64) -> u128 {
    let seq_num = if side == Side::Bid { !seq_num } else { seq_num };

    let upper = (price_data as u128) << 64;
    upper | (seq_num as u128)
}

/// Creates price data for an oracle pegged order from the price offset
///
/// Reverse of oracle_pegged_price_offset()
pub fn oracle_pegged_price_data(price_offset_lots: i64) -> u64 {
    // Price data is used for ordering in the bookside's top bits of the u128 key.
    // Map i64::MIN to be 0 and i64::MAX to u64::MAX, that way comparisons on the
    // u64 produce the same result as on the source i64.
    // Equivalent: (price_offset_lots as i128 - (i64::MIN as i128) as u64
    (price_offset_lots as u64).wrapping_add(u64::MAX / 2 + 1)
}

/// Retrieves the price offset (in lots) from an oracle pegged order's price data
///
/// Reverse of oracle_pegged_price_data()
pub fn oracle_pegged_price_offset(price_data: u64) -> i64 {
    price_data.wrapping_sub(u64::MAX / 2 + 1) as i64
}

/// Creates price data for a fixed order's price
///
/// Reverse of fixed_price_lots()
pub fn fixed_price_data(price_lots: i64) -> Result<u64> {
    require_gte!(price_lots, 1);
    Ok(price_lots as u64)
}

/// Retrieves the price (in lots) from a fixed order's price data
///
/// Reverse of fixed_price_data().
pub fn fixed_price_lots(price_data: u64) -> i64 {
    assert!(price_data <= i64::MAX as u64);
    price_data as i64
}

/// InnerNodes and LeafNodes compose the binary tree of orders.
///
/// Each InnerNode has exactly two children, which are either InnerNodes themselves,
/// or LeafNodes. The children share the top `prefix_len` bits of `key`. The left
/// child has a 0 in the next bit, and the right a 1.
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, AnchorSerialize, AnchorDeserialize)]
#[repr(C)]
pub struct InnerNode {
    pub tag: u8, // NodeTag
    pub padding: [u8; 3],
    /// number of highest `key` bits that all children share
    /// e.g. if it's 2, the two highest bits of `key` will be the same on all children
    pub prefix_len: u32,

    /// only the top `prefix_len` bits of `key` are relevant
    pub key: u128,

    /// indexes into `BookSide::nodes`
    pub children: [NodeHandle; 2],

    /// The earliest expiry timestamp for the left and right subtrees.
    ///
    /// Needed to be able to find and remove expired orders without having to
    /// iterate through the whole bookside.
    pub child_earliest_expiry: [u64; 2],

    pub reserved: [u8; 40],
}
const_assert_eq!(size_of::<InnerNode>(), 4 + 4 + 16 + 4 * 2 + 8 * 2 + 40);
const_assert_eq!(size_of::<InnerNode>(), NODE_SIZE);
const_assert_eq!(size_of::<InnerNode>() % 8, 0);

impl InnerNode {
    pub fn new(prefix_len: u32, key: u128) -> Self {
        Self {
            tag: NodeTag::InnerNode.into(),
            padding: Default::default(),
            prefix_len,
            key,
            children: [0; 2],
            child_earliest_expiry: [u64::MAX; 2],
            reserved: [0; NODE_SIZE - 48],
        }
    }

    /// Returns the handle of the child that may contain the search key
    /// and 0 or 1 depending on which child it was.
    pub(crate) fn walk_down(&self, search_key: u128) -> (NodeHandle, bool) {
        let crit_bit_mask = 1u128 << (127 - self.prefix_len);
        let crit_bit = (search_key & crit_bit_mask) != 0;
        (self.children[crit_bit as usize], crit_bit)
    }

    /// The lowest timestamp at which one of the contained LeafNodes expires.
    #[inline(always)]
    pub fn earliest_expiry(&self) -> u64 {
        std::cmp::min(self.child_earliest_expiry[0], self.child_earliest_expiry[1])
    }
}

/// LeafNodes represent an order in the binary tree
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    bytemuck::Pod,
    bytemuck::Zeroable,
    AnchorSerialize,
    AnchorDeserialize,
)]
#[repr(C)]
pub struct LeafNode {
    /// NodeTag
    pub tag: u8,

    /// Index into the owning OpenOrdersAccount's OpenOrders
    pub owner_slot: u8,

    /// Time in seconds after `timestamp` at which the order expires.
    /// A value of 0 means no expiry.
    pub time_in_force: u16,

    pub padding: [u8; 4],

    /// The binary tree key, see new_node_key()
    pub key: u128,

    /// Address of the owning OpenOrdersAccount
    pub owner: Pubkey,

    /// Number of base lots to buy or sell, always >=1
    pub quantity: i64,

    /// The time the order was placed
    pub timestamp: u64,

    /// If the effective price of an oracle pegged order exceeds this limit,
    /// it will be considered invalid and may be removed.
    ///
    /// Only applicable in the oracle_pegged OrderTree
    pub peg_limit: i64,

    /// User defined id for this order, used in FillEvents
    pub client_order_id: u64,
}
const_assert_eq!(
    size_of::<LeafNode>(),
    4 + 1 + 1 + 1 + 1 + 16 + 32 + 8 + 8 + 8 + 8
);
const_assert_eq!(size_of::<LeafNode>(), NODE_SIZE);
const_assert_eq!(size_of::<LeafNode>() % 8, 0);

impl LeafNode {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        owner_slot: u8,
        key: u128,
        owner: Pubkey,
        quantity: i64,
        timestamp: u64,
        time_in_force: u16,
        peg_limit: i64,
        client_order_id: u64,
    ) -> Self {
        Self {
            tag: NodeTag::LeafNode.into(),
            owner_slot,
            time_in_force,
            padding: Default::default(),
            key,
            owner,
            quantity,
            timestamp,
            peg_limit,
            client_order_id,
        }
    }

    /// The order's price_data as stored in the key
    ///
    /// Needs to be unpacked differently for fixed and oracle pegged orders.
    #[inline(always)]
    pub fn price_data(&self) -> u64 {
        (self.key >> 64) as u64
    }

    /// Time at which this order will expire, u64::MAX if never
    #[inline(always)]
    pub fn expiry(&self) -> u64 {
        if self.time_in_force == 0 {
            u64::MAX
        } else {
            self.timestamp + self.time_in_force as u64
        }
    }

    /// Returns if the order is expired at `now_ts`
    #[inline(always)]
    pub fn is_expired(&self, now_ts: u64) -> bool {
        self.time_in_force > 0 && now_ts >= self.timestamp + self.time_in_force as u64
    }
}

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct FreeNode {
    pub(crate) tag: u8, // NodeTag
    pub(crate) padding: [u8; 3],
    pub(crate) next: NodeHandle,
    pub(crate) reserved: [u8; NODE_SIZE - 16],
    // essential to make AnyNode alignment the same as other node types
    pub(crate) force_align: u64,
}
const_assert_eq!(size_of::<FreeNode>(), NODE_SIZE);
const_assert_eq!(size_of::<FreeNode>() % 8, 0);

#[zero_copy]
pub struct AnyNode {
    pub tag: u8,
    pub data: [u8; 79],
    // essential to make AnyNode alignment the same as other node types
    pub force_align: u64,
}
const_assert_eq!(size_of::<AnyNode>(), NODE_SIZE);
const_assert_eq!(size_of::<AnyNode>() % 8, 0);
const_assert_eq!(align_of::<AnyNode>(), 8);
const_assert_eq!(size_of::<AnyNode>(), size_of::<InnerNode>());
const_assert_eq!(align_of::<AnyNode>(), align_of::<InnerNode>());
const_assert_eq!(size_of::<AnyNode>(), size_of::<LeafNode>());
const_assert_eq!(align_of::<AnyNode>(), align_of::<LeafNode>());
const_assert_eq!(size_of::<AnyNode>(), size_of::<FreeNode>());
const_assert_eq!(align_of::<AnyNode>(), align_of::<FreeNode>());

pub(crate) enum NodeRef<'a> {
    Inner(&'a InnerNode),
    Leaf(&'a LeafNode),
}

pub(crate) enum NodeRefMut<'a> {
    Inner(&'a mut InnerNode),
    Leaf(&'a mut LeafNode),
}

impl AnyNode {
    pub fn key(&self) -> Option<u128> {
        match self.case()? {
            NodeRef::Inner(inner) => Some(inner.key),
            NodeRef::Leaf(leaf) => Some(leaf.key),
        }
    }

    pub(crate) fn children(&self) -> Option<[NodeHandle; 2]> {
        match self.case().unwrap() {
            NodeRef::Inner(&InnerNode { children, .. }) => Some(children),
            NodeRef::Leaf(_) => None,
        }
    }

    pub(crate) fn case(&self) -> Option<NodeRef> {
        match NodeTag::try_from(self.tag) {
            Ok(NodeTag::InnerNode) => Some(NodeRef::Inner(cast_ref(self))),
            Ok(NodeTag::LeafNode) => Some(NodeRef::Leaf(cast_ref(self))),
            _ => None,
        }
    }

    fn case_mut(&mut self) -> Option<NodeRefMut> {
        match NodeTag::try_from(self.tag) {
            Ok(NodeTag::InnerNode) => Some(NodeRefMut::Inner(cast_mut(self))),
            Ok(NodeTag::LeafNode) => Some(NodeRefMut::Leaf(cast_mut(self))),
            _ => None,
        }
    }

    #[inline]
    pub fn as_leaf(&self) -> Option<&LeafNode> {
        match self.case() {
            Some(NodeRef::Leaf(leaf_ref)) => Some(leaf_ref),
            _ => None,
        }
    }

    #[inline]
    pub fn as_leaf_mut(&mut self) -> Option<&mut LeafNode> {
        match self.case_mut() {
            Some(NodeRefMut::Leaf(leaf_ref)) => Some(leaf_ref),
            _ => None,
        }
    }

    #[inline]
    pub fn as_inner(&self) -> Option<&InnerNode> {
        match self.case() {
            Some(NodeRef::Inner(inner_ref)) => Some(inner_ref),
            _ => None,
        }
    }

    #[inline]
    pub fn as_inner_mut(&mut self) -> Option<&mut InnerNode> {
        match self.case_mut() {
            Some(NodeRefMut::Inner(inner_ref)) => Some(inner_ref),
            _ => None,
        }
    }

    #[inline]
    pub fn earliest_expiry(&self) -> u64 {
        match self.case().unwrap() {
            NodeRef::Inner(inner) => inner.earliest_expiry(),
            NodeRef::Leaf(leaf) => leaf.expiry(),
        }
    }
}

impl AsRef<AnyNode> for InnerNode {
    fn as_ref(&self) -> &AnyNode {
        cast_ref(self)
    }
}

impl AsRef<AnyNode> for LeafNode {
    #[inline]
    fn as_ref(&self) -> &AnyNode {
        cast_ref(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    #[test]
    fn order_tree_price_data() {
        for price in [1, 42, i64::MAX] {
            assert_eq!(price, fixed_price_lots(fixed_price_data(price).unwrap()));
        }

        let seq = [-i64::MAX, -i64::MAX + 1, 0, i64::MAX - 1, i64::MAX];
        for price_offset in seq {
            assert_eq!(
                price_offset,
                oracle_pegged_price_offset(oracle_pegged_price_data(price_offset))
            );
        }
        for (lhs, rhs) in seq.iter().tuple_windows() {
            let l_price_data = oracle_pegged_price_data(*lhs);
            let r_price_data = oracle_pegged_price_data(*rhs);
            assert!(l_price_data < r_price_data);
        }

        assert_eq!(oracle_pegged_price_data(i64::MIN), 0);
        assert_eq!(oracle_pegged_price_data(i64::MAX), u64::MAX);
        assert_eq!(oracle_pegged_price_data(0), -(i64::MIN as i128) as u64); // remember -i64::MIN is not a valid i64
    }

    #[test]
    fn order_tree_key_ordering() {
        let bid_seq: Vec<(i64, u64)> = vec![
            (-5, 15),
            (-5, 10),
            (-4, 6),
            (-4, 5),
            (0, 20),
            (0, 1),
            (4, 6),
            (4, 5),
            (5, 3),
        ];
        for (lhs, rhs) in bid_seq.iter().tuple_windows() {
            let l_price_data = oracle_pegged_price_data(lhs.0);
            let r_price_data = oracle_pegged_price_data(rhs.0);
            let l_key = new_node_key(Side::Bid, l_price_data, lhs.1);
            let r_key = new_node_key(Side::Bid, r_price_data, rhs.1);
            assert!(l_key < r_key);
        }

        let ask_seq: Vec<(i64, u64)> = vec![
            (-5, 10),
            (-5, 15),
            (-4, 6),
            (-4, 7),
            (0, 1),
            (0, 20),
            (4, 5),
            (4, 6),
            (5, 3),
        ];
        for (lhs, rhs) in ask_seq.iter().tuple_windows() {
            let l_price_data = oracle_pegged_price_data(lhs.0);
            let r_price_data = oracle_pegged_price_data(rhs.0);
            let l_key = new_node_key(Side::Ask, l_price_data, lhs.1);
            let r_key = new_node_key(Side::Ask, r_price_data, rhs.1);
            assert!(l_key < r_key);
        }
    }
}
