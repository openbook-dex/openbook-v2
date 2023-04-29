use anchor_lang::prelude::*;

use derivative::Derivative;
use fixed::types::I80F48;
use static_assertions::const_assert_eq;
use std::mem::size_of;

use crate::state::*;

pub const FREE_ORDER_SLOT: MarketIndex = MarketIndex::MAX;

#[zero_copy]
#[derive(AnchorSerialize, AnchorDeserialize, Derivative)]
#[derivative(Debug)]
pub struct Position {
    /// Active position size, measured in base lots
    pub base_position_lots: i64,
    /// Active position in quote (conversation rate is that of the time the order was settled)
    /// measured in native quote
    pub quote_position_native: I80F48,

    /// Tracks what the position is to calculate average entry & break even price
    pub quote_running_native: i64,

    /// Base lots in open bids
    pub bids_base_lots: i64,
    /// Base lots in open asks
    pub asks_base_lots: i64,

    /// Amount of base lots on the EventQueue waiting to be processed
    pub taker_base_lots: i64,
    /// Amount of quote lots on the EventQueue waiting to be processed
    pub taker_quote_lots: i64,

    pub base_free_lots: i64,
    pub quote_free_lots: i64,

    pub referrer_rebates_accrued: u64,

    /// Cumulative maker volume in quote native units
    ///
    /// (Display only)
    pub maker_volume: u64,
    /// Cumulative taker volume in quote native units
    ///
    /// (Display only)
    pub taker_volume: u64,

    /// The native average entry price for the base lots of the current position.
    /// Reset to 0 when the base position reaches or crosses 0.
    pub avg_entry_price_per_base_lot: f64,

    #[derivative(Debug = "ignore")]
    pub reserved: [u8; 88],
}

const_assert_eq!(
    size_of::<Position>(),
    8 + 16 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 88
);
const_assert_eq!(size_of::<Position>(), 200);
const_assert_eq!(size_of::<Position>() % 8, 0);

impl Default for Position {
    fn default() -> Self {
        Self {
            base_position_lots: 0,
            quote_position_native: I80F48::ZERO,
            quote_running_native: 0,
            bids_base_lots: 0,
            asks_base_lots: 0,
            taker_base_lots: 0,
            taker_quote_lots: 0,
            base_free_lots: 0,
            quote_free_lots: 0,
            referrer_rebates_accrued: 0,
            maker_volume: 0,
            taker_volume: 0,
            avg_entry_price_per_base_lot: 0.0,
            reserved: [0; 88],
        }
    }
}

impl Position {
    /// Add taker trade after it has been matched but before it has been process on EventQueue
    pub fn add_taker_trade(&mut self, side: Side, base_lots: i64, quote_lots: i64) {
        match side {
            Side::Bid => {
                self.taker_base_lots += base_lots;
                self.taker_quote_lots -= quote_lots;
            }
            Side::Ask => {
                self.taker_base_lots -= base_lots;
                self.taker_quote_lots += quote_lots;
            }
        }
    }

    /// Remove taker trade after it has been processed on EventQueue
    pub fn remove_taker_trade(&mut self, base_change: i64, quote_change: i64) {
        if base_change > 0 {
            self.base_free_lots += base_change;
        }
        if quote_change > 0 {
            self.quote_free_lots += quote_change;
        }

        self.taker_base_lots -= base_change;
        self.taker_quote_lots -= quote_change;
    }

    // Return base position in native units for a perp market
    pub fn base_position_native(&self, market: &Market) -> I80F48 {
        I80F48::from(self.base_position_lots * market.base_lot_size)
    }

    pub fn base_position_lots(&self) -> i64 {
        self.base_position_lots
    }

    // This takes into account base lots from unprocessed events, but not anything from open orders
    pub fn effective_base_position_lots(&self) -> i64 {
        self.base_position_lots + self.taker_base_lots
    }

    pub fn quote_position_native(&self) -> I80F48 {
        self.quote_position_native
    }

    /// This assumes settle_funding was already called
    pub fn change_base_position(&mut self, base_change: i64) {
        self.base_position_lots += base_change;
    }

    /// Updates avg entry price, breakeven price, realized pnl, realized pnl limit
    fn update_trade_stats(&mut self, base_change: i64, quote_change_native: I80F48) {
        if base_change == 0 {
            return;
        }

        let old_position = self.base_position_lots;
        let new_position = old_position + base_change;

        // amount of lots that were reduced (so going from -5 to 10 lots is a reduction of 5)
        let _reduced_lots;
        // amount of pnl that was realized by the reduction (signed)
        let _newly_realized_pnl;

        if new_position == 0 {
            _reduced_lots = -old_position;

            // clear out display fields that live only while the position lasts
            self.avg_entry_price_per_base_lot = 0.0;
            self.quote_running_native = 0;
        } else if old_position.signum() != new_position.signum() {
            // If the base position changes sign, we've crossed base_pos == 0 (or old_position == 0)
            _reduced_lots = -old_position;
            let _old_position = old_position as f64;
            let _new_position = new_position as f64;
            let base_change = base_change as f64;
            let _old_avg_entry = self.avg_entry_price_per_base_lot;
            let new_avg_entry = (quote_change_native.to_num::<f64>() / base_change).abs();

            // Set entry and break-even based on the new_position entered
            self.avg_entry_price_per_base_lot = new_avg_entry;
        } else {
            // The old and new position have the same sign

            self.quote_running_native += quote_change_native.round_to_zero().to_num::<i64>();

            let is_increasing = old_position.signum() == base_change.signum();
            if is_increasing {
                // Increasing position: avg entry price updates, no new realized pnl
                _reduced_lots = 0;
                _newly_realized_pnl = I80F48::ZERO;
                let old_position_abs = old_position.abs() as f64;
                let new_position_abs = new_position.abs() as f64;
                let old_avg_entry = self.avg_entry_price_per_base_lot;
                let new_position_quote_value =
                    old_position_abs * old_avg_entry + quote_change_native.to_num::<f64>().abs();
                self.avg_entry_price_per_base_lot = new_position_quote_value / new_position_abs;
            } else {
                // Decreasing position: pnl is realized, avg entry price does not change
                _reduced_lots = base_change;
                let _avg_entry = I80F48::from_num(self.avg_entry_price_per_base_lot);
            }
        }
    }

    /// Change the base and quote positions as the result of a trade
    pub fn record_trade(
        &mut self,
        _market: &mut Market,
        base_change: i64,
        quote_change_native: I80F48,
    ) {
        self.update_trade_stats(base_change, quote_change_native);
        self.change_base_position(base_change);
        self.change_quote_position(quote_change_native);
    }

    pub fn change_quote_position(&mut self, quote_change_native: I80F48) {
        self.quote_position_native += quote_change_native;
    }

    /// Does the user have any orders on the book?
    ///
    /// Note that it's possible they were matched already: This only becomes
    /// false when the fill event is processed or the orders are cancelled.
    pub fn has_open_orders(&self) -> bool {
        self.asks_base_lots != 0 || self.bids_base_lots != 0
    }

    // Did the user take orders and hasn't been filled yet?
    pub fn has_open_taker_fills(&self) -> bool {
        self.taker_base_lots != 0 || self.taker_quote_lots != 0
    }

    /// Are there any open orders or fills that haven't been processed yet?
    pub fn has_open_orders_or_fills(&self) -> bool {
        self.has_open_orders() || self.has_open_taker_fills()
    }

    /// Calculate the average entry price of the position, in native/native units
    pub fn avg_entry_price(&self, market: &Market) -> f64 {
        self.avg_entry_price_per_base_lot / (market.base_lot_size as f64)
    }

    /// Calculate the break even price of the position, in native/native units
    pub fn break_even_price(&self, market: &Market) -> f64 {
        if self.base_position_lots == 0 {
            return 0.0;
        }
        -(self.quote_running_native as f64)
            / ((self.base_position_lots * market.base_lot_size) as f64)
    }

    /// Update position for a maker/taker fee payment
    pub fn record_trading_fee(&mut self, fee: I80F48) {
        self.change_quote_position(-fee);
    }
}

#[zero_copy]
#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct OpenOrder {
    pub side_and_tree: u8, // SideAndOrderTree -- enums aren't POD
    pub padding1: [u8; 7],
    pub client_id: u64,
    pub id: u128,
    pub reserved: [u8; 64],
}
const_assert_eq!(size_of::<OpenOrder>(), 1 + 1 + 2 + 4 + 8 + 16 + 64);
const_assert_eq!(size_of::<OpenOrder>(), 96);
const_assert_eq!(size_of::<OpenOrder>() % 8, 0);

impl Default for OpenOrder {
    fn default() -> Self {
        Self {
            side_and_tree: SideAndOrderTree::BidFixed.into(),
            padding1: Default::default(),
            client_id: 0,
            id: 0,
            reserved: [0; 64],
        }
    }
}

impl OpenOrder {
    pub fn side_and_tree(&self) -> SideAndOrderTree {
        SideAndOrderTree::try_from(self.side_and_tree).unwrap()
    }
}

#[macro_export]
macro_rules! account_seeds {
    ( $account:expr ) => {
        &[
            b"OpenOrdersAccount".as_ref(),
            $account.group.as_ref(),
            $account.owner.as_ref(),
            &$account.account_num.to_le_bytes(),
            &[$account.bump],
        ]
    };
}

pub use account_seeds;

// #[cfg(test)]
// mod tests {
//     use crate::state::Market;
//     use fixed::types::I80F48;
//     use rand::Rng;

//     use super::Position;

//     fn create_position(
//         market: &Market,
//         base_pos: i64,
//         entry_price_per_lot: i64,
//     ) -> Position {
//         let mut pos = Position::default();
//         pos.market_index = market.market_index;
//         pos.base_position_lots = base_pos;
//         pos.quote_position_native = I80F48::from(-base_pos * entry_price_per_lot);
//         pos.quote_running_native = -base_pos * entry_price_per_lot;
//         pos.avg_entry_price_per_base_lot = entry_price_per_lot as f64;
//         pos
//     }

//     fn test_market(stable_price: f64) -> Market {
//         let mut m = Market::default_for_tests();
//         m.stable_price_model.stable_price = stable_price;
//         m
//     }

//     #[test]
//     fn test_quote_entry_long_increasing_from_zero() {
//         let mut market = test_market(10.0);
//         let mut pos = create_position(&market, 0, 0);
//         // Go long 10 @ 10
//         pos.record_trade(&mut market, 10, I80F48::from(-100));
//         assert_eq!(pos.quote_running_native, -100);
//         assert_eq!(pos.avg_entry_price(&market), 10.0);
//         assert_eq!(pos.break_even_price(&market), 10.0);
//         assert_eq!(pos.realized_trade_pnl_native, I80F48::from(0));
//         assert_eq!(pos.realized_pnl_for_position_native, I80F48::from(0));
//         assert_eq!(pos.settle_pnl_limit_realized_trade, 0);
//     }

//     #[test]
//     fn test_quote_entry_short_increasing_from_zero() {
//         let mut market = test_market(10.0);
//         let mut pos = create_position(&market, 0, 0);
//         // Go short 10 @ 10
//         pos.record_trade(&mut market, -10, I80F48::from(100));
//         assert_eq!(pos.quote_running_native, 100);
//         assert_eq!(pos.avg_entry_price(&market), 10.0);
//         assert_eq!(pos.break_even_price(&market), 10.0);
//     }

//     #[test]
//     fn test_quote_entry_long_increasing_from_long() {
//         let mut market = test_market(10.0);
//         let mut pos = create_position(&market, 10, 10);
//         // Go long 10 @ 30
//         pos.record_trade(&mut market, 10, I80F48::from(-300));
//         assert_eq!(pos.quote_running_native, -400);
//         assert_eq!(pos.avg_entry_price(&market), 20.0);
//         assert_eq!(pos.break_even_price(&market), 20.0);
//     }

//     #[test]
//     fn test_quote_entry_short_increasing_from_short() {
//         let mut market = test_market(10.0);
//         let mut pos = create_position(&market, -10, 10);
//         // Go short 10 @ 30
//         pos.record_trade(&mut market, -10, I80F48::from(300));
//         assert_eq!(pos.quote_running_native, 400);
//         assert_eq!(pos.avg_entry_price(&market), 20.0);
//         assert_eq!(pos.break_even_price(&market), 20.0);
//     }

//     #[test]
//     fn test_quote_entry_long_decreasing_from_short() {
//         let mut market = test_market(10.0);
//         let mut pos = create_position(&market, -10, 10);
//         // Go long 5 @ 50
//         pos.record_trade(&mut market, 5, I80F48::from(-250));
//         assert_eq!(pos.quote_running_native, -150);
//         assert_eq!(pos.avg_entry_price(&market), 10.0); // Entry price remains the same when decreasing
//         assert_eq!(pos.break_even_price(&market), -30.0); // The short can't break even anymore
//     }

//     #[test]
//     fn test_quote_entry_short_decreasing_from_long() {
//         let mut market = test_market(10.0);
//         let mut pos = create_position(&market, 10, 10);
//         // Go short 5 @ 50
//         pos.record_trade(&mut market, -5, I80F48::from(250));
//         assert_eq!(pos.quote_running_native, 150);
//         assert_eq!(pos.avg_entry_price(&market), 10.0); // Entry price remains the same when decreasing
//         assert_eq!(pos.break_even_price(&market), -30.0); // Already broke even
//     }

//     #[test]
//     fn test_quote_entry_long_close_with_short() {
//         let mut market = test_market(10.0);
//         let mut pos = create_position(&market, 10, 10);
//         // Go short 10 @ 25
//         pos.record_trade(&mut market, -10, I80F48::from(250));
//         assert_eq!(pos.quote_running_native, 0);
//         assert_eq!(pos.avg_entry_price(&market), 0.0); // Entry price zero when no position
//         assert_eq!(pos.break_even_price(&market), 0.0);
//     }

//     #[test]
//     fn test_quote_entry_short_close_with_long() {
//         let mut market = test_market(10.0);
//         let mut pos = create_position(&market, -10, 10);
//         // Go long 10 @ 25
//         pos.record_trade(&mut market, 10, I80F48::from(-250));
//         assert_eq!(pos.quote_running_native, 0);
//         assert_eq!(pos.avg_entry_price(&market), 0.0); // Entry price zero when no position
//         assert_eq!(pos.break_even_price(&market), 0.0);
//     }

//     #[test]
//     fn test_quote_entry_long_close_short_with_overflow() {
//         let mut market = test_market(10.0);
//         let mut pos = create_position(&market, 10, 10);
//         // Go short 15 @ 20
//         pos.record_trade(&mut market, -15, I80F48::from(300));
//         assert_eq!(pos.quote_running_native, 100);
//         assert_eq!(pos.avg_entry_price(&market), 20.0);
//         assert_eq!(pos.break_even_price(&market), 20.0);
//     }

//     #[test]
//     fn test_quote_entry_short_close_long_with_overflow() {
//         let mut market = test_market(10.0);
//         let mut pos = create_position(&market, -10, 10);
//         // Go long 15 @ 20
//         pos.record_trade(&mut market, 15, I80F48::from(-300));
//         assert_eq!(pos.quote_running_native, -100);
//         assert_eq!(pos.avg_entry_price(&market), 20.0);
//         assert_eq!(pos.break_even_price(&market), 20.0);
//     }

//     #[test]
//     fn test_quote_entry_break_even_price() {
//         let mut market = test_market(10.0);
//         let mut pos = create_position(&market, 0, 0);
//         // Buy 11 @ 10,000
//         pos.record_trade(&mut market, 11, I80F48::from(-11 * 10_000));
//         // Sell 1 @ 12,000
//         pos.record_trade(&mut market, -1, I80F48::from(12_000));
//         assert_eq!(pos.quote_running_native, -98_000);
//         assert_eq!(pos.base_position_lots, 10);
//         assert_eq!(pos.break_even_price(&market), 9_800.0); // We made 2k on the trade, so we can sell our contract up to a loss of 200 each
//     }

//     #[test]
//     fn test_entry_and_break_even_prices_with_lots() {
//         let mut market = test_market(10.0);
//         market.base_lot_size = 10;

//         let mut pos = create_position(&market, 0, 0);
//         // Buy 110 @ 10,000
//         pos.record_trade(&mut market, 11, I80F48::from(-11 * 10 * 10_000));
//         // Sell 10 @ 12,000
//         pos.record_trade(&mut market, -1, I80F48::from(1 * 10 * 12_000));
//         assert_eq!(pos.quote_running_native, -980_000);
//         assert_eq!(pos.base_position_lots, 10);
//         assert_eq!(pos.avg_entry_price_per_base_lot, 100_000.0);
//         assert_eq!(pos.avg_entry_price(&market), 10_000.0);
//         assert_eq!(pos.break_even_price(&market), 9_800.0);
//         assert_eq!(pos.realized_trade_pnl_native, I80F48::from(20_000));
//         assert_eq!(pos.realized_pnl_for_position_native, I80F48::from(20_000));
//     }

//     #[test]
//     fn test_realized_settle_limit_no_reduction() {
//         let mut market = test_market(10.0);
//         let mut pos = create_position(&market, 0, 0);
//         // Buy 11 @ 10,000
//         pos.record_trade(&mut market, 11, I80F48::from(-11 * 10_000));

//         // Sell 1 @ 11,000
//         pos.record_trade(&mut market, -1, I80F48::from(11_000));
//         assert_eq!(pos.realized_trade_pnl_native, I80F48::from(1_000));
//         assert_eq!(pos.realized_pnl_for_position_native, I80F48::from(1_000));
//         assert_eq!(pos.settle_pnl_limit_realized_trade, 1 * 10 / 5 + 1);

//         // Sell 1 @ 11,000 -- increases limit
//         pos.record_trade(&mut market, -1, I80F48::from(11_000));
//         assert_eq!(pos.realized_trade_pnl_native, I80F48::from(2_000));
//         assert_eq!(pos.realized_pnl_for_position_native, I80F48::from(2_000));
//         assert_eq!(pos.settle_pnl_limit_realized_trade, 2 * (10 / 5 + 1));

//         // Sell 1 @ 9,000 -- a loss, but doesn't flip realized_trade_pnl_native sign, no change to limit
//         pos.record_trade(&mut market, -1, I80F48::from(9_000));
//         assert_eq!(pos.realized_trade_pnl_native, I80F48::from(1_000));
//         assert_eq!(pos.realized_pnl_for_position_native, I80F48::from(1_000));
//         assert_eq!(pos.settle_pnl_limit_realized_trade, 2 * (10 / 5 + 1));

//         // Sell 1 @ 8,000 -- flips sign, changes pnl limit
//         pos.record_trade(&mut market, -1, I80F48::from(8_000));
//         assert_eq!(pos.realized_trade_pnl_native, I80F48::from(-1_000));
//         assert_eq!(pos.realized_pnl_for_position_native, I80F48::from(-1_000));
//         assert_eq!(pos.settle_pnl_limit_realized_trade, -(1 * 10 / 5 + 1));
//     }

//     #[test]
//     fn test_trade_without_realized_pnl() {
//         let mut market = test_market(10.0);
//         let mut pos = create_position(&market, 0, 0);

//         // Buy 11 @ 10,000
//         pos.record_trade(&mut market, 11, I80F48::from(-11 * 10_000));

//         // Sell 1 @ 10,000
//         pos.record_trade(&mut market, -1, I80F48::from(10_000));
//         assert_eq!(pos.realized_trade_pnl_native, I80F48::from(0));
//         assert_eq!(pos.realized_pnl_for_position_native, I80F48::from(0));
//         assert_eq!(pos.settle_pnl_limit_realized_trade, 0);

//         // Sell 10 @ 10,000
//         pos.record_trade(&mut market, -10, I80F48::from(10 * 10_000));
//         assert_eq!(pos.realized_trade_pnl_native, I80F48::from(0));
//         assert_eq!(pos.realized_pnl_for_position_native, I80F48::from(0));
//         assert_eq!(pos.settle_pnl_limit_realized_trade, 0);

//         assert_eq!(pos.base_position_lots, 0);
//         assert_eq!(pos.quote_position_native, I80F48::ZERO);
//     }

//     #[test]
//     fn test_realized_pnl_trade_other_separation() {
//         let mut market = test_market(10.0);
//         let mut pos = create_position(&market, 0, 0);

//         pos.record_trading_fee(I80F48::from(-70));
//         assert_eq!(pos.realized_other_pnl_native, I80F48::from(70));

//         pos.record_liquidation_quote_change(I80F48::from(30));
//         assert_eq!(pos.realized_other_pnl_native, I80F48::from(100));

//         // Buy 1 @ 10,000
//         pos.record_trade(&mut market, 1, I80F48::from(-1 * 10_000));

//         // Sell 1 @ 11,000
//         pos.record_trade(&mut market, -1, I80F48::from(11_000));

//         assert_eq!(pos.realized_other_pnl_native, I80F48::from(100));
//         assert_eq!(pos.realized_trade_pnl_native, I80F48::from(1_000));
//         assert_eq!(pos.realized_pnl_for_position_native, I80F48::from(0));
//         assert_eq!(pos.settle_pnl_limit_realized_trade, 1 * 10 / 5 + 1);
//     }

//     #[test]
//     fn test_realized_pnl_fractional() {
//         let mut market = test_market(10.0);
//         let mut pos = create_position(&market, 0, 0);
//         pos.quote_position_native += I80F48::from_num(0.1);

//         // Buy 1 @ 1
//         pos.record_trade(&mut market, 1, I80F48::from(-1));
//         // Buy 2 @ 2
//         pos.record_trade(&mut market, 2, I80F48::from(-2 * 2));

//         assert!((pos.avg_entry_price(&market) - 1.66666).abs() < 0.001);
//         assert_eq!(pos.realized_trade_pnl_native, I80F48::from(0));

//         // Sell 2 @ 4
//         pos.record_trade(&mut market, -2, I80F48::from(2 * 4));

//         assert!((pos.avg_entry_price(&market) - 1.66666).abs() < 0.001);
//         assert!((pos.realized_trade_pnl_native.to_num::<f64>() - 4.6666).abs() < 0.01);

//         // Sell 1 @ 2
//         pos.record_trade(&mut market, -1, I80F48::from(2));

//         assert_eq!(pos.avg_entry_price(&market), 0.0);
//         assert!((pos.quote_position_native.to_num::<f64>() - 5.1).abs() < 0.001);
//         assert!((pos.realized_trade_pnl_native.to_num::<f64>() - 5.1).abs() < 0.01);
//     }

//     #[test]
//     fn test_entry_multiple_random_long() {
//         let mut market = test_market(10.0);
//         let mut pos = create_position(&market, 0, 0);

//         // Generate array of random trades
//         let mut rng = rand::thread_rng();
//         let mut trades: Vec<[i64; 2]> = Vec::with_capacity(500);
//         for _ in 0..trades.capacity() {
//             let qty: i64 = rng.gen_range(1..=1000);
//             let px: f64 = rng.gen_range(0.1..=100.0);
//             let quote: i64 = (-qty as f64 * px).round() as i64;
//             trades.push([qty, quote]);
//         }
//         // Apply all of the trades going forward
//         let mut total_qty = 0;
//         let mut total_quote = 0;
//         trades.iter().for_each(|[qty, quote]| {
//             pos.record_trade(&mut market, *qty, I80F48::from(*quote));
//             total_qty += qty.abs();
//             total_quote += quote.abs();
//             let entry_actual = pos.avg_entry_price(&market);
//             let entry_expected = total_quote as f64 / total_qty as f64;
//             assert!(((entry_actual - entry_expected) / entry_expected).abs() < 10.0 * f64::EPSILON);
//         });
//         // base_position should be sum of all base quantities
//         assert_eq!(pos.base_position_lots, total_qty);
//         // Reverse out all the trades
//         trades.iter().for_each(|[qty, quote]| {
//             pos.record_trade(&mut market, -*qty, I80F48::from(-*quote));
//         });
//         assert_eq!(pos.base_position_lots, 0);
//         assert_eq!(pos.quote_running_native, 0);
//         assert_eq!(pos.avg_entry_price_per_base_lot, 0.0);
//     }

//     #[test]
//     fn test_position_pnl_returns_correct_pnl_for_oracle_price() {
//         let mut market = test_market(10.0);
//         market.base_lot_size = 10;

//         let long_pos = create_position(&market, 50, 100);
//         let pnl = long_pos.unsettled_pnl(&market, I80F48::from(11)).unwrap();
//         assert_eq!(pnl, I80F48::from(50 * 10 * 1), "long profitable");
//         let pnl = long_pos.unsettled_pnl(&market, I80F48::from(9)).unwrap();
//         assert_eq!(pnl, I80F48::from(50 * 10 * -1), "long unprofitable");

//         let short_pos = create_position(&market, -50, 100);
//         let pnl = short_pos.unsettled_pnl(&market, I80F48::from(11)).unwrap();
//         assert_eq!(pnl, I80F48::from(50 * 10 * -1), "short unprofitable");
//         let pnl = short_pos.unsettled_pnl(&market, I80F48::from(9)).unwrap();
//         assert_eq!(pnl, I80F48::from(50 * 10 * 1), "short profitable");
//     }

//     #[test]
//     fn test_realized_pnl_consumption() {
//         let market = test_market(10.0);

//         let mut pos = create_position(&market, 0, 0);
//         assert_eq!(pos.realized_trade_pnl_native, I80F48::from(0));

//         pos.settle_pnl_limit_realized_trade = 1000;
//         pos.realized_trade_pnl_native = I80F48::from(1500);
//         pos.record_settle(I80F48::from(10));
//         assert_eq!(pos.realized_trade_pnl_native, I80F48::from(1490));
//         assert_eq!(pos.settle_pnl_limit_realized_trade, 1000);
//         assert_eq!(pos.settle_pnl_limit_settled_in_current_window_native, 10);

//         pos.record_settle(I80F48::from(-2));
//         assert_eq!(pos.realized_trade_pnl_native, I80F48::from(1490));
//         assert_eq!(pos.settle_pnl_limit_realized_trade, 1000);
//         assert_eq!(pos.settle_pnl_limit_settled_in_current_window_native, 8);

//         pos.record_settle(I80F48::from(1100));
//         assert_eq!(pos.realized_trade_pnl_native, I80F48::from(390));
//         assert_eq!(pos.settle_pnl_limit_realized_trade, 390);
//         assert_eq!(
//             pos.settle_pnl_limit_settled_in_current_window_native,
//             8 + 1100 - (1000 - 390)
//         );

//         pos.settle_pnl_limit_realized_trade = 4;
//         pos.settle_pnl_limit_settled_in_current_window_native = 0;
//         pos.realized_trade_pnl_native = I80F48::from(5);
//         assert_eq!(pos.available_settle_limit(&market), (0, 4));
//         pos.record_settle(I80F48::from(-20));
//         assert_eq!(pos.realized_trade_pnl_native, I80F48::from(5));
//         assert_eq!(pos.settle_pnl_limit_realized_trade, 4);
//         assert_eq!(pos.settle_pnl_limit_settled_in_current_window_native, -20);
//         assert_eq!(pos.available_settle_limit(&market), (0, 24));

//         pos.record_settle(I80F48::from(2));
//         assert_eq!(pos.realized_trade_pnl_native, I80F48::from(3));
//         assert_eq!(pos.settle_pnl_limit_realized_trade, 3);
//         assert_eq!(pos.settle_pnl_limit_settled_in_current_window_native, -19);
//         assert_eq!(pos.available_settle_limit(&market), (0, 22));

//         pos.record_settle(I80F48::from(10));
//         assert_eq!(pos.realized_trade_pnl_native, I80F48::from(0));
//         assert_eq!(pos.settle_pnl_limit_realized_trade, 0);
//         assert_eq!(pos.settle_pnl_limit_settled_in_current_window_native, -12);
//         assert_eq!(pos.available_settle_limit(&market), (0, 12));

//         pos.realized_trade_pnl_native = I80F48::from(-5);
//         pos.settle_pnl_limit_realized_trade = -4;
//         pos.settle_pnl_limit_settled_in_current_window_native = 0;
//         pos.record_settle(I80F48::from(20));
//         assert_eq!(pos.realized_trade_pnl_native, I80F48::from(-5));
//         assert_eq!(pos.settle_pnl_limit_realized_trade, -4);
//         assert_eq!(pos.settle_pnl_limit_settled_in_current_window_native, 20);

//         pos.record_settle(I80F48::from(-2));
//         assert_eq!(pos.realized_trade_pnl_native, I80F48::from(-3));
//         assert_eq!(pos.settle_pnl_limit_realized_trade, -3);
//         assert_eq!(pos.settle_pnl_limit_settled_in_current_window_native, 19);

//         pos.record_settle(I80F48::from(-10));
//         assert_eq!(pos.realized_trade_pnl_native, I80F48::from(0));
//         assert_eq!(pos.settle_pnl_limit_realized_trade, 0);
//         assert_eq!(pos.settle_pnl_limit_settled_in_current_window_native, 12);

//         pos.realized_other_pnl_native = I80F48::from(10);
//         pos.realized_trade_pnl_native = I80F48::from(25);
//         pos.settle_pnl_limit_realized_trade = 20;
//         pos.record_settle(I80F48::from(1));
//         assert_eq!(pos.realized_other_pnl_native, I80F48::from(9));
//         assert_eq!(pos.realized_trade_pnl_native, I80F48::from(25));
//         assert_eq!(pos.settle_pnl_limit_realized_trade, 20);
//         assert_eq!(pos.settle_pnl_limit_settled_in_current_window_native, 12);

//         pos.record_settle(I80F48::from(10));
//         assert_eq!(pos.realized_other_pnl_native, I80F48::from(0));
//         assert_eq!(pos.realized_trade_pnl_native, I80F48::from(24));
//         assert_eq!(pos.settle_pnl_limit_realized_trade, 20);
//         assert_eq!(pos.settle_pnl_limit_settled_in_current_window_native, 13);

//         pos.realized_other_pnl_native = I80F48::from(-10);
//         pos.realized_trade_pnl_native = I80F48::from(-25);
//         pos.settle_pnl_limit_realized_trade = -20;
//         pos.record_settle(I80F48::from(-1));
//         assert_eq!(pos.realized_other_pnl_native, I80F48::from(-9));
//         assert_eq!(pos.realized_trade_pnl_native, I80F48::from(-25));
//         assert_eq!(pos.settle_pnl_limit_realized_trade, -20);
//     }

//     #[test]
//     fn test_settle_limit_window() {
//         let mut market = Market::default_for_tests();
//         let mut pos = create_position(&market, 100, -50);

//         market.settle_pnl_limit_window_size_ts = 100;
//         pos.settle_pnl_limit_settled_in_current_window_native = 10;

//         pos.update_settle_limit(&market, 505);
//         assert_eq!(pos.settle_pnl_limit_settled_in_current_window_native, 0);
//         assert_eq!(pos.settle_pnl_limit_window, 5);

//         pos.settle_pnl_limit_settled_in_current_window_native = 10;
//         pos.update_settle_limit(&market, 550);
//         assert_eq!(pos.settle_pnl_limit_settled_in_current_window_native, 10);
//         assert_eq!(pos.settle_pnl_limit_window, 5);

//         pos.settle_pnl_limit_settled_in_current_window_native = 10;
//         pos.update_settle_limit(&market, 600);
//         assert_eq!(pos.settle_pnl_limit_settled_in_current_window_native, 0);
//         assert_eq!(pos.settle_pnl_limit_window, 6);

//         market.settle_pnl_limit_window_size_ts = 400;
//         pos.update_settle_limit(&market, 605);
//         assert_eq!(pos.settle_pnl_limit_settled_in_current_window_native, 0);
//         assert_eq!(pos.settle_pnl_limit_window, 1);
//     }

//     #[test]
//     fn test_settle_limit() {
//         let mut market = test_market(0.5);

//         let mut pos = create_position(&market, 100, 1);
//         pos.realized_trade_pnl_native = I80F48::from(60); // no effect

//         let limited_pnl = |pos: &Position, market: &Market, pnl: i64| {
//             pos.apply_pnl_settle_limit(market, I80F48::from(pnl))
//                 .to_num::<f64>()
//         };

//         pos.settle_pnl_limit_realized_trade = 5;
//         assert_eq!(pos.available_settle_limit(&market), (-10, 15)); // 0.2 factor * 0.5 stable price * 100 lots + 5 realized
//         assert_eq!(limited_pnl(&pos, &market, 100), 15.0);
//         assert_eq!(limited_pnl(&pos, &market, -100), -10.0);

//         pos.settle_pnl_limit_settled_in_current_window_native = 2;
//         assert_eq!(pos.available_settle_limit(&market), (-12, 13));
//         assert_eq!(limited_pnl(&pos, &market, 100), 13.0);
//         assert_eq!(limited_pnl(&pos, &market, -100), -12.0);

//         pos.settle_pnl_limit_settled_in_current_window_native = 16;
//         assert_eq!(pos.available_settle_limit(&market), (-26, 0));

//         pos.settle_pnl_limit_settled_in_current_window_native = -16;
//         assert_eq!(pos.available_settle_limit(&market), (0, 31));

//         pos.settle_pnl_limit_realized_trade = 0;
//         pos.settle_pnl_limit_settled_in_current_window_native = 2;
//         assert_eq!(pos.available_settle_limit(&market), (-12, 8));

//         pos.settle_pnl_limit_settled_in_current_window_native = -2;
//         assert_eq!(pos.available_settle_limit(&market), (-8, 12));

//         market.stable_price_model.stable_price = 1.0;
//         assert_eq!(pos.available_settle_limit(&market), (-18, 22));

//         pos.settle_pnl_limit_realized_trade = 1000;
//         pos.settle_pnl_limit_settled_in_current_window_native = 2;
//         assert_eq!(pos.available_settle_limit(&market), (-22, 1018));

//         pos.realized_other_pnl_native = I80F48::from(5);
//         assert_eq!(pos.available_settle_limit(&market), (-22, 1023));

//         pos.realized_other_pnl_native = I80F48::from(-5);
//         assert_eq!(pos.available_settle_limit(&market), (-27, 1018));
//     }

//     #[test]
//     fn test_reduced_realized_pnl_settle_limit() {
//         let market = test_market(0.5);
//         let mut pos = create_position(&market, 100, 1);

//         let cases = vec![
//             // No change if realized > limit
//             (0, (100, 50, 70, -200), (50, 70)),
//             // No change if realized > limit
//             (1, (100, 50, 70, 200), (50, 70)),
//             // No change if abs(realized) > abs(limit)
//             (2, (-100, -50, 70, -200), (-50, 70)),
//             // No change if abs(realized) > abs(limit)
//             (3, (-100, -50, 70, 200), (-50, 70)),
//             // reduction limited by realized change
//             (4, (40, 50, 70, -5), (40, 65)),
//             // reduction max
//             (5, (40, 50, 70, -15), (40, 60)),
//             // reduction, with realized change wrong direction
//             (6, (40, 50, 70, 15), (40, 70)),
//             // reduction limited by realized change
//             (7, (-40, -50, -70, 5), (-40, -65)),
//             // reduction max
//             (8, (-40, -50, -70, 15), (-40, -60)),
//             // reduction, with realized change wrong direction
//             (9, (-40, -50, -70, -15), (-40, -70)),
//             // reduction when used amount is opposite sign
//             (10, (-40, -50, 70, -15), (-40, 70)),
//             // reduction when used amount is opposite sign
//             (11, (-40, -50, 70, 15), (-40, 80)),
//         ];

//         for (i, (realized, realized_limit, used, change), (expected_limit, expected_used)) in cases
//         {
//             println!("test case {i}");
//             pos.realized_trade_pnl_native = I80F48::from(realized);
//             pos.settle_pnl_limit_realized_trade = realized_limit;
//             pos.settle_pnl_limit_settled_in_current_window_native = used;
//             pos.apply_realized_trade_pnl_settle_limit_constraint(I80F48::from(change));
//             assert_eq!(pos.settle_pnl_limit_realized_trade, expected_limit);
//             assert_eq!(
//                 pos.settle_pnl_limit_settled_in_current_window_native,
//                 expected_used
//             );
//         }
//     }
// }
