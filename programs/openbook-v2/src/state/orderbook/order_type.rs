use anchor_lang::prelude::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use super::*;
use crate::error::*;
use crate::error_msg;

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
pub enum PlaceOrderType {
    /// Take existing orders up to price, max_base_quantity and max_quote_quantity.
    /// If any base_quantity or quote_quantity remains, place an order on the book
    Limit = 0,

    /// Take existing orders up to price, max_base_quantity and max_quote_quantity.
    /// Never place an order on the book.
    ImmediateOrCancel = 1,

    /// Never take any existing orders, post the order on the book if possible.
    /// If existing orders can match with this order, do nothing.
    PostOnly = 2,

    /// Ignore price and take orders up to max_base_quantity and max_quote_quantity.
    /// Never place an order on the book.
    ///
    /// Equivalent to ImmediateOrCancel with price=i64::MAX.
    Market = 3,

    /// If existing orders match with this order, adjust the price to just barely
    /// not match. Always places an order on the book.
    PostOnlySlide = 4,
}

impl PlaceOrderType {
    pub fn to_post_order_type(&self) -> Result<PostOrderType> {
        match *self {
            Self::Market => Err(error_msg!("Market is not a PostOrderType")),
            Self::ImmediateOrCancel => Err(error_msg!("ImmediateOrCancel is not a PostOrderType")),
            Self::Limit => Ok(PostOrderType::Limit),
            Self::PostOnly => Ok(PostOrderType::PostOnly),
            Self::PostOnlySlide => Ok(PostOrderType::PostOnlySlide),
        }
    }
}

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
pub enum PostOrderType {
    /// Take existing orders up to price, max_base_quantity and max_quote_quantity.
    /// If any base_quantity or quote_quantity remains, place an order on the book
    Limit = 0,

    /// Never take any existing orders, post the order on the book if possible.
    /// If existing orders can match with this order, do nothing.
    PostOnly = 2,

    /// If existing orders match with this order, adjust the price to just barely
    /// not match. Always places an order on the book.
    PostOnlySlide = 4,
}

#[derive(
    Eq,
    PartialEq,
    Copy,
    Clone,
    Default,
    TryFromPrimitive,
    IntoPrimitive,
    Debug,
    AnchorSerialize,
    AnchorDeserialize,
)]
#[repr(u8)]
/// Self trade behavior controls how taker orders interact with resting limit orders of the same account.
/// This setting has no influence on placing a resting or oracle pegged limit order that does not match
/// immediately, instead it's the responsibility of the user to correctly configure his taker orders.
pub enum SelfTradeBehavior {
    /// Both the maker and taker sides of the matched orders are decremented.
    /// This is equivalent to a normal order match, except for the fact that no fees are applied.
    #[default]
    DecrementTake = 0,

    /// Cancels the maker side of the trade, the taker side gets matched with other maker's orders.
    CancelProvide = 1,

    /// Cancels the whole transaction as soon as a self-matching scenario is encountered.
    AbortTransaction = 2,
}

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
pub enum Side {
    Bid = 0,
    Ask = 1,
}

impl Side {
    pub fn invert_side(self: &Side) -> Side {
        match self {
            Side::Bid => Side::Ask,
            Side::Ask => Side::Bid,
        }
    }

    /// Is `lhs` is a better order for `side` than `rhs`?
    pub fn is_price_data_better(self: &Side, lhs: u64, rhs: u64) -> bool {
        match self {
            Side::Bid => lhs > rhs,
            Side::Ask => lhs < rhs,
        }
    }

    /// Is `lhs` is a better order for `side` than `rhs`?
    pub fn is_price_better(self: &Side, lhs: i64, rhs: i64) -> bool {
        match self {
            Side::Bid => lhs > rhs,
            Side::Ask => lhs < rhs,
        }
    }

    /// Is `price` acceptable for a `limit` order on `side`?
    pub fn is_price_within_limit(self: &Side, price: i64, limit: i64) -> bool {
        match self {
            Side::Bid => price <= limit,
            Side::Ask => price >= limit,
        }
    }
}

/// SideAndOrderTree is a storage optimization, so we don't need two bytes for the data
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
pub enum SideAndOrderTree {
    BidFixed = 0,
    AskFixed = 1,
    BidOraclePegged = 2,
    AskOraclePegged = 3,
}

impl SideAndOrderTree {
    pub fn new(side: Side, order_tree: BookSideOrderTree) -> Self {
        match (side, order_tree) {
            (Side::Bid, BookSideOrderTree::Fixed) => Self::BidFixed,
            (Side::Ask, BookSideOrderTree::Fixed) => Self::AskFixed,
            (Side::Bid, BookSideOrderTree::OraclePegged) => Self::BidOraclePegged,
            (Side::Ask, BookSideOrderTree::OraclePegged) => Self::AskOraclePegged,
        }
    }

    pub fn side(&self) -> Side {
        match self {
            Self::BidFixed | Self::BidOraclePegged => Side::Bid,
            Self::AskFixed | Self::AskOraclePegged => Side::Ask,
        }
    }

    pub fn order_tree(&self) -> BookSideOrderTree {
        match self {
            Self::BidFixed | Self::AskFixed => BookSideOrderTree::Fixed,
            Self::BidOraclePegged | Self::AskOraclePegged => BookSideOrderTree::OraclePegged,
        }
    }
}
