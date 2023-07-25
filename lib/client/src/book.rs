use anyhow::Result;
use fixed::types::I80F48;
use openbook_v2::state::{Market, Orderbook, Side};

// TODO Adjust this number after doing some calculations
const MAXIUM_TAKEN_ORDERS: u8 = 8;

pub struct Amounts {
    pub total_base_taken_native: u64,
    pub total_quote_taken_native: u64,
    pub fee: u64,
    pub price_impact: i64,
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

    let other_side = side.invert_side();
    let oracle_price_lots = market.native_price_to_lot(oracle_price)?;

    let (order_max_quote_lots, order_max_base_lots) = match side {
        Side::Bid => (
            market.subtract_taker_fees(max_quote_lots_including_fees),
            market.max_base_lots(),
        ),
        Side::Ask => (market.max_quote_lots(), max_base_lots),
    };

    let mut remaining_base_lots = order_max_base_lots;
    let mut remaining_quote_lots = order_max_quote_lots;

    let mut first_price = 0_i64;
    let mut last_price = 0_i64;

    let opposing_bookside = book.bookside(other_side);
    for (index, best_opposing) in opposing_bookside
        .iter_valid(now_ts, oracle_price_lots)
        .enumerate()
    {
        if remaining_base_lots == 0 || remaining_quote_lots == 0 {
            break;
        }

        if index == 0 {
            first_price = best_opposing.price_lots;
        } else {
            last_price = best_opposing.price_lots;
        }

        let best_opposing_price = best_opposing.price_lots;

        if limit == 0 {
            break;
        }

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

    let total_base_taken_native = (total_base_lots_taken * market.base_lot_size) as u64;

    let mut total_quote_taken_native = (total_quote_lots_taken * market.quote_lot_size) as u64;

    let mut taker_fees = 0_u64;
    let mut not_enough_liquidity = false;
    if total_quote_lots_taken > 0 || total_base_lots_taken > 0 {
        taker_fees = market.taker_fees_ceil(total_quote_taken_native);

        match side {
            Side::Bid => {
                total_quote_taken_native += taker_fees;
                if total_quote_taken_native < max_quote_lots_including_fees as u64 {
                    not_enough_liquidity = true
                }
            }
            Side::Ask => {
                total_quote_taken_native -= taker_fees;
                if total_base_taken_native < max_quote_lots_including_fees as u64 {
                    not_enough_liquidity = true
                }
            }
        };
    } else {
        not_enough_liquidity = true;
    }

    Ok(Amounts {
        total_base_taken_native,
        total_quote_taken_native,
        fee: taker_fees,
        price_impact: (first_price - last_price).abs(),
        not_enough_liquidity,
    })
}
