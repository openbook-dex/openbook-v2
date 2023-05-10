use anchor_lang::prelude::*;

use super::*;

///  order parameters
pub struct Order {
    pub side: Side,

    /// Max base lots to buy/sell.
    pub max_base_lots: u64,

    /// Max quote lots to pay/receive including fees.
    /// TODO Binye Use native instead?
    pub max_quote_lots_including_fees: u64,

    /// Arbitrary user-controlled order id.
    pub client_order_id: u64,

    /// Reduce only
    pub reduce_only: bool,

    /// Number of seconds the order shall live, 0 meaning forever
    pub time_in_force: u16,

    /// Order type specific params
    pub params: OrderParams,
}

pub enum OrderParams {
    Market,
    ImmediateOrCancel {
        price_lots: u64,
    },
    Fixed {
        price_lots: u64,
        order_type: PostOrderType,
    },
    OraclePegged {
        price_offset_lots: i64,
        order_type: PostOrderType,
        peg_limit: u64,
        max_oracle_staleness_slots: i32,
    },
}

macro_rules! add_i64 {
    ($a:expr,$b:expr) => {{
        if $a > 0 {
            $a + ($b.abs() as u64)
        } else {
            $a - ($b.abs() as u64)
        }
    }};
}

impl Order {
    /// Convert an input expiry timestamp to a time_in_force value
    pub fn tif_from_expiry(expiry_timestamp: u64) -> Option<u16> {
        let now_ts: u64 = Clock::get().unwrap().unix_timestamp.try_into().unwrap();
        if expiry_timestamp != 0 {
            // If expiry is far in the future, clamp to u16::MAX seconds
            let tif = expiry_timestamp.saturating_sub(now_ts).min(u16::MAX.into());
            if tif == 0 {
                // If expiry is in the past, ignore the order
                return None;
            }
            Some(tif as u16)
        } else {
            // Never expire
            Some(0)
        }
    }

    /// Should this order be penalized with an extra fee?
    ///
    /// Some programs opportunistically call ioc orders, wasting lots of compute. This
    /// is intended to encourage people to be smarter about it.
    pub fn needs_penalty_fee(&self) -> bool {
        matches!(self.params, OrderParams::ImmediateOrCancel { .. })
    }

    /// Is this order required to be posted to the orderbook? It will fail if it would take.
    pub fn is_post_only(&self) -> bool {
        let order_type = match self.params {
            OrderParams::Fixed { order_type, .. } => order_type,
            OrderParams::OraclePegged { order_type, .. } => order_type,
            _ => return false,
        };
        order_type == PostOrderType::PostOnly || order_type == PostOrderType::PostOnlySlide
    }

    /// Order tree that this order should be added to
    pub fn post_target(&self) -> Option<BookSideOrderTree> {
        match self.params {
            OrderParams::Fixed { .. } => Some(BookSideOrderTree::Fixed),
            OrderParams::OraclePegged { .. } => Some(BookSideOrderTree::OraclePegged),
            _ => None,
        }
    }

    /// Some order types (PostOnlySlide) may override the price that is passed in,
    /// this function computes the order-type-adjusted price.
    fn price_for_order_type(
        &self,
        now_ts: u64,
        oracle_price_lots: u64,
        price_lots: u64,
        order_type: PostOrderType,
        order_book: &Orderbook,
    ) -> u64 {
        if order_type == PostOrderType::PostOnlySlide {
            if let Some(best_other_price) = order_book
                .bookside(self.side.invert_side())
                .best_price(now_ts, oracle_price_lots)
            {
                post_only_slide_limit(self.side, best_other_price, price_lots)
            } else {
                price_lots
            }
        } else {
            price_lots
        }
    }

    /// Compute the price_lots this order is currently at, as well as the price_data that
    /// would be stored in its OrderTree node if the order is posted to the orderbook.
    pub fn price(
        &self,
        now_ts: u64,
        oracle_price_lots: u64,
        order_book: &Orderbook,
    ) -> Result<(u64, u64)> {
        let price_lots = match self.params {
            OrderParams::Market => market_order_limit_for_side(self.side),
            OrderParams::ImmediateOrCancel { price_lots } => price_lots,
            OrderParams::Fixed {
                price_lots,
                order_type,
            } => self.price_for_order_type(
                now_ts,
                oracle_price_lots,
                price_lots,
                order_type,
                order_book,
            ),
            OrderParams::OraclePegged {
                price_offset_lots,
                order_type,
                ..
            } => {
                let price_lots = add_i64!(oracle_price_lots, price_offset_lots);
                self.price_for_order_type(
                    now_ts,
                    oracle_price_lots,
                    price_lots,
                    order_type,
                    order_book,
                )
            }
        };
        let price_data = match self.params {
            OrderParams::OraclePegged { .. } => {
                oracle_pegged_price_data(price_lots - oracle_price_lots)
            }
            _ => fixed_price_data(price_lots)?,
        };
        require_gte!(price_lots, 1);
        Ok((price_lots, price_data))
    }

    /// pegging limit for oracle peg orders, otherwise 0
    pub fn peg_limit(&self) -> u64 {
        match self.params {
            OrderParams::OraclePegged { peg_limit, .. } => peg_limit,
            _ => 0,
        }
    }
}

/// The implicit limit price to use for market orders
fn market_order_limit_for_side(side: Side) -> u64 {
    match side {
        Side::Bid => u64::MAX,
        Side::Ask => 1,
    }
}

/// The limit to use for PostOnlySlide orders: the tinyest bit better than
/// the best opposing order
fn post_only_slide_limit(side: Side, best_other_side: u64, limit: u64) -> u64 {
    match side {
        Side::Bid => limit.min(best_other_side - 1),
        Side::Ask => limit.max(best_other_side + 1),
    }
}
