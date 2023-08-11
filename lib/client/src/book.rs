use anyhow::Result;
use fixed::types::I80F48;
use openbook_v2::state::{Market, Orderbook, Side};

// TODO Adjust this number after doing some calculations
const MAXIUM_TAKEN_ORDERS: u8 = 8;

pub struct Amounts {
    pub total_base_taken_native: u64,
    pub total_quote_taken_native: u64,
    pub fee: u64,
    pub not_enough_liquidity: bool,
}

pub fn iterate_book(
    book: Orderbook,
    side: Side,
    max_base_lots: i64,
    max_quote_lots_including_fees: i64,
    market: &Market,
    oracle_price: I80F48,
    now_ts: u64,
) -> Result<Amounts> {
    let mut limit = MAXIUM_TAKEN_ORDERS;

    let oracle_price_lots = market.native_price_to_lot(oracle_price)?;

    let order_max_base_lots = max_base_lots;
    let order_max_quote_lots = match side {
        Side::Bid => market.subtract_taker_fees(max_quote_lots_including_fees),
        Side::Ask => max_quote_lots_including_fees,
    };

    let mut remaining_base_lots = order_max_base_lots;
    let mut remaining_quote_lots = order_max_quote_lots;

    let opposing_bookside = book.bookside(side.invert_side());
    for best_opposing in opposing_bookside.iter_valid(now_ts, oracle_price_lots) {
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
    }

    let total_quote_lots_taken = order_max_quote_lots - remaining_quote_lots;
    let total_base_lots_taken = order_max_base_lots - remaining_base_lots;

    let not_enough_liquidity = match side {
        Side::Ask => remaining_base_lots != 0,
        Side::Bid => remaining_quote_lots != 0,
    };

    let total_base_taken_native = (total_base_lots_taken * market.base_lot_size) as u64;
    let total_quote_taken_native = (total_quote_lots_taken * market.quote_lot_size) as u64;

    Ok(Amounts {
        total_base_taken_native,
        total_quote_taken_native,
        fee: market.taker_fees_ceil(total_quote_taken_native),
        not_enough_liquidity,
    })
}
