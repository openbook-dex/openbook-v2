use std::collections::HashMap;

use anchor_lang::prelude::Pubkey;
use anyhow::Result;
use fixed::types::I80F48;
use openbook_v2::state::{
    Market, Orderbook, Side, DROP_EXPIRED_ORDER_LIMIT, FILL_EVENT_REMAINING_LIMIT,
};

// TODO Adjust this number after doing some calculations
const MAXIMUM_TAKEN_ORDERS: u8 = 8;
const MAXIMUM_REMAINING_ACCOUNTS: usize = 4;

pub struct Amounts {
    pub total_base_taken_native: u64,
    pub total_quote_taken_native: u64,
    pub fee: u64,
    pub not_enough_liquidity: bool,
}

pub fn remaining_accounts_to_crank(
    book: Orderbook,
    side: Side,
    max_base_lots: i64,
    max_quote_lots_including_fees: i64,
    market: &Market,
    oracle_price: I80F48,
    now_ts: u64,
) -> Result<Vec<Pubkey>> {
    let oracle_price_lots = market.native_price_to_lot(oracle_price)?;
    let mut accounts = Vec::new();

    iterate_book(
        book,
        side,
        max_base_lots,
        max_quote_lots_including_fees,
        market,
        oracle_price_lots,
        now_ts,
        &mut accounts,
    );

    // Get most occurred Pubkey
    let mut frequency_map: HashMap<Pubkey, usize> = HashMap::new();
    for &value in &accounts {
        *frequency_map.entry(value).or_insert(0) += 1;
    }
    // Sort by occurrences in descending order
    let mut sorted_pairs: Vec<(Pubkey, usize)> = frequency_map.into_iter().collect();
    sorted_pairs.sort_by(|a, b| b.1.cmp(&a.1));

    let common_accounts: Vec<Pubkey> = sorted_pairs
        .iter()
        .take(MAXIMUM_REMAINING_ACCOUNTS)
        .map(|(value, _)| *value)
        .collect();

    Ok(common_accounts)
}

pub fn amounts_from_book(
    book: Orderbook,
    side: Side,
    max_base_lots: i64,
    max_quote_lots_including_fees: i64,
    market: &Market,
    oracle_price: I80F48,
    now_ts: u64,
) -> Result<Amounts> {
    let oracle_price_lots = market.native_price_to_lot(oracle_price)?;
    let mut accounts = Vec::new();
    let (total_base_lots_taken, total_quote_lots_taken, not_enough_liquidity) = iterate_book(
        book,
        side,
        max_base_lots,
        max_quote_lots_including_fees,
        market,
        oracle_price_lots,
        now_ts,
        &mut accounts,
    );

    let total_base_taken_native = (total_base_lots_taken * market.base_lot_size) as u64;
    let total_quote_taken_native = (total_quote_lots_taken * market.quote_lot_size) as u64;

    Ok(Amounts {
        total_base_taken_native,
        total_quote_taken_native,
        fee: market.taker_fees_ceil(total_quote_taken_native),
        not_enough_liquidity,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn iterate_book(
    book: Orderbook,
    side: Side,
    max_base_lots: i64,
    max_quote_lots_including_fees: i64,
    market: &Market,
    oracle_price_lots: i64,
    now_ts: u64,
    accounts: &mut Vec<Pubkey>,
) -> (i64, i64, bool) {
    let mut limit = MAXIMUM_TAKEN_ORDERS;
    let mut number_of_processed_fill_events = 0;
    let mut number_of_dropped_expired_orders = 0;

    let order_max_base_lots = max_base_lots;
    let order_max_quote_lots = match side {
        Side::Bid => market.subtract_taker_fees(max_quote_lots_including_fees),
        Side::Ask => max_quote_lots_including_fees,
    };

    let mut remaining_base_lots = order_max_base_lots;
    let mut remaining_quote_lots = order_max_quote_lots;

    let opposing_bookside = book.bookside(side.invert_side());
    for best_opposing in opposing_bookside.iter_all_including_invalid(now_ts, oracle_price_lots) {
        if !best_opposing.is_valid() {
            // Remove the order from the book unless we've done that enough
            if number_of_dropped_expired_orders < DROP_EXPIRED_ORDER_LIMIT {
                accounts.push(best_opposing.node.owner);
                number_of_dropped_expired_orders += 1;
            }
        }

        if remaining_base_lots == 0 || remaining_quote_lots == 0 || limit == 0 {
            break;
        }

        let best_opposing_price = best_opposing.price_lots;
        let max_match_by_quote = remaining_quote_lots / best_opposing_price;
        if max_match_by_quote == 0 {
            break;
        }

        let match_base_lots = remaining_base_lots
            .min(best_opposing.node.quantity)
            .min(max_match_by_quote);
        let match_quote_lots = match_base_lots * best_opposing_price;

        remaining_base_lots -= match_base_lots;
        remaining_quote_lots -= match_quote_lots;

        limit -= 1;

        if number_of_processed_fill_events < FILL_EVENT_REMAINING_LIMIT {
            accounts.push(best_opposing.node.owner);
            number_of_processed_fill_events += 1;
        }
    }

    let total_base_lots_taken = order_max_base_lots - remaining_base_lots;
    let total_quote_lots_taken = order_max_quote_lots - remaining_quote_lots;
    let not_enough_liquidity = match side {
        Side::Ask => remaining_base_lots != 0,
        Side::Bid => remaining_quote_lots != 0,
    };

    (
        total_base_lots_taken,
        total_quote_lots_taken,
        not_enough_liquidity,
    )
}
