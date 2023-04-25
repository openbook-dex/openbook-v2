use super::*;

/// Iterate over orders in order (bids=descending, asks=ascending)
pub struct OrderTreeIter<'a> {
    order_tree: &'a OrderTreeNodes,
    /// InnerNodes where the right side still needs to be iterated on
    stack: Vec<&'a InnerNode>,
    /// To be returned on `next()`
    next_leaf: Option<(NodeHandle, &'a LeafNode)>,

    /// either 0, 1 to iterate low-to-high, or 1, 0 to iterate high-to-low
    left: usize,
    right: usize,
}

impl<'a> OrderTreeIter<'a> {
    pub fn new(order_tree: &'a OrderTreeNodes, root: &OrderTreeRoot) -> Self {
        let (left, right) = if order_tree.order_tree_type() == OrderTreeType::Bids {
            (1, 0)
        } else {
            (0, 1)
        };
        let stack = vec![];

        let mut iter = Self {
            order_tree,
            stack,
            next_leaf: None,
            left,
            right,
        };
        if let Some(r) = root.node() {
            iter.next_leaf = iter.find_leftmost_leaf(r);
        }
        iter
    }

    pub fn side(&self) -> Side {
        if self.left == 1 {
            Side::Bid
        } else {
            Side::Ask
        }
    }

    pub fn peek(&self) -> Option<(NodeHandle, &'a LeafNode)> {
        self.next_leaf
    }

    fn find_leftmost_leaf(&mut self, start: NodeHandle) -> Option<(NodeHandle, &'a LeafNode)> {
        let mut current = start;
        loop {
            match self.order_tree.node(current).unwrap().case().unwrap() {
                NodeRef::Inner(inner) => {
                    self.stack.push(inner);
                    current = inner.children[self.left];
                }
                NodeRef::Leaf(leaf) => {
                    return Some((current, leaf));
                }
            }
        }
    }
}

impl<'a> Iterator for OrderTreeIter<'a> {
    type Item = (NodeHandle, &'a LeafNode);

    fn next(&mut self) -> Option<Self::Item> {
        // no next leaf? done
        self.next_leaf?;

        // start popping from stack and get the other child
        let current_leaf = self.next_leaf;
        self.next_leaf = match self.stack.pop() {
            None => None,
            Some(inner) => {
                let start = inner.children[self.right];
                // go down the left branch as much as possible until reaching a leaf
                self.find_leftmost_leaf(start)
            }
        };

        current_leaf
    }
}
