use anchor_lang::AccountDeserialize;
use anchor_lang::__private::bytemuck::Zeroable;
use anchor_lang::prelude::Clock;
use anyhow::Result;
use fixed::types::I80F48;
use openbook_v2::accounts_zerocopy::{AccountReader, KeyedAccountReader};
use openbook_v2::state::{BookSide, OrderParams};
use openbook_v2::state::{
    EventQueue, Market, Order, Orderbook, SelfTradeBehavior::DecrementTake, Side,
};

use jupiter_amm_interface::{
    AccountMap, Amm, KeyedAccount, Quote, QuoteParams, Side as JupiterSide, Swap,
    SwapAndAccountMetas, SwapParams,
};
/// An abstraction in order to share reserve mints and necessary data
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey, sysvar::clock};
use std::cell::RefCell;
use std::str;

// TODO Adjust this number after doing some calculations
const MAXIUM_TAKEN_ORDERS: u8 = 8;

#[derive(Clone)]
pub struct OpenBookMarket {
    market: Market,
    event_queue: EventQueue,
    bids: BookSide,
    asks: BookSide,
    timestamp: u64,
    key: Pubkey,
    label: String,
    related_accounts: [Pubkey; 7],
    reserve_mints: [Pubkey; 2],
    oracle_price: I80F48,
}

impl Amm for OpenBookMarket {
    fn label(&self) -> String {
        self.label.clone()
    }

    fn key(&self) -> Pubkey {
        self.key
    }

    fn program_id(&self) -> Pubkey {
        openbook_v2::id()
    }

    fn get_reserve_mints(&self) -> Vec<Pubkey> {
        self.reserve_mints.to_vec()
    }

    fn get_accounts_to_update(&self) -> Vec<Pubkey> {
        self.related_accounts.to_vec()
    }

    fn from_keyed_account(keyed_account: &KeyedAccount) -> Result<Self> {
        let market = Market::try_deserialize(&mut keyed_account.account.data.as_slice())?;

        Ok(OpenBookMarket {
            market,
            key: keyed_account.key,
            label: str::from_utf8(&market.name).unwrap().to_string(),
            related_accounts: [
                market.bids,
                market.asks,
                market.event_queue,
                market.oracle,
                market.base_vault,
                market.quote_vault,
                clock::ID,
            ],
            reserve_mints: [market.base_mint, market.quote_mint],
            event_queue: EventQueue::zeroed(),
            bids: BookSide::zeroed(),
            asks: BookSide::zeroed(),
            oracle_price: I80F48::ZERO,
            timestamp: 0,
        })
    }

    fn update(&mut self, account_map: &AccountMap) -> Result<()> {
        let bids_data = account_map.get(&self.market.bids).unwrap();
        self.bids = BookSide::try_deserialize(&mut bids_data.data.as_slice()).unwrap();

        let asks_data = account_map.get(&self.market.asks).unwrap();
        self.asks = BookSide::try_deserialize(&mut asks_data.data.as_slice()).unwrap();

        let event_queue_data = account_map.get(&self.market.event_queue).unwrap();
        self.event_queue =
            EventQueue::try_deserialize(&mut event_queue_data.data.as_slice()).unwrap();

        let oracle_data = account_map.get(&self.market.oracle).unwrap();
        let oracle_acc = &AccountOracle {
            data: &oracle_data.data,
            key: self.market.oracle,
        };

        let clock_data = account_map.get(&clock::ID).unwrap();
        let clock: Clock = bincode::deserialize(&clock_data.data.as_slice())?;
        self.timestamp = clock.unix_timestamp as u64;
        self.oracle_price = self.market.oracle_price(oracle_acc, self.timestamp)?;

        Ok(())
    }

    fn quote(&self, quote_params: &QuoteParams) -> Result<Quote> {
        let order = if quote_params.input_mint == self.market.quote_mint {
            Order {
                side: Side::Bid,
                max_base_lots: 0,
                max_quote_lots_including_fees: quote_params.in_amount.try_into().unwrap(),
                client_order_id: 0,
                time_in_force: 0,
                self_trade_behavior: DecrementTake,
                params: OrderParams::Market,
            }
        } else {
            Order {
                side: Side::Ask,
                max_base_lots: quote_params.in_amount.try_into().unwrap(),
                max_quote_lots_including_fees: 0,
                client_order_id: 0,
                time_in_force: 0,
                self_trade_behavior: DecrementTake,
                params: OrderParams::Market,
            }
        };

        let bids_ref = RefCell::new(self.bids);
        let asks_ref = RefCell::new(self.asks);
        let book = Orderbook {
            bids: bids_ref.borrow_mut(),
            asks: asks_ref.borrow_mut(),
        };

        let order_amounts: Amounts = iterate_book(
            book,
            &order,
            &self.market,
            self.oracle_price,
            self.timestamp,
        )?;
        let (in_amount, out_amount) = match order.side {
            Side::Bid => (
                order_amounts.total_quote_taken_native,
                order_amounts.total_base_taken_native,
            ),
            Side::Ask => (
                order_amounts.total_base_taken_native,
                order_amounts.total_quote_taken_native,
            ),
        };

        Ok(Quote {
            in_amount,
            out_amount,
            fee_mint: self.market.quote_mint,
            fee_amount: order_amounts.fee,
            // price_impact_pct: order_amounts.price_impact.into(),
            ..Quote::default()
        })
    }

    fn get_swap_and_account_metas(&self, swap_params: &SwapParams) -> Result<SwapAndAccountMetas> {
        let SwapParams {
            destination_mint,
            source_mint,
            user_destination_token_account,
            user_source_token_account,
            user_transfer_authority,
            ..
        } = swap_params;

        let (side, base_account, quote_account) = if source_mint == &self.market.quote_mint {
            (
                JupiterSide::Bid,
                user_destination_token_account,
                user_source_token_account,
            )
        } else {
            (
                JupiterSide::Ask,
                user_source_token_account,
                user_destination_token_account,
            )
        };

        let account_metas = vec![
            AccountMeta::new(self.key, true),
            AccountMeta::new(self.market.bids, true),
            AccountMeta::new(self.market.asks, true),
            AccountMeta::new(self.market.event_queue, true),
            AccountMeta::new(self.market.base_vault, true),
            AccountMeta::new(self.market.quote_vault, true),
            AccountMeta::new(*base_account, true),
            AccountMeta::new(*quote_account, true),
            AccountMeta::new(self.market.oracle, false),
        ];

        Ok(SwapAndAccountMetas {
            swap: Swap::Openbook { side: { side } },
            account_metas,
        })
    }

    fn clone_amm(&self) -> Box<dyn Amm + Send + Sync> {
        Box::new(self.clone())
    }
}

pub struct Amounts {
    pub total_base_taken_native: u64,
    pub total_quote_taken_native: u64,
    pub fee: u64,
    pub price_impact: i64,
    pub not_enough_liquidity: bool,
}

fn iterate_book(
    book: Orderbook,
    order: &Order,
    market: &Market,
    oracle_price: I80F48,
    now_ts: u64,
) -> Result<Amounts> {
    let side = order.side;
    let mut limit = MAXIUM_TAKEN_ORDERS;

    let other_side = side.invert_side();
    let oracle_price_lots = market.native_price_to_lot(oracle_price)?;
    let (price_lots, _) = order.price(now_ts, oracle_price_lots, &book)?;

    let order_max_quote_lots = match side {
        Side::Bid => market.subtract_taker_fees(order.max_quote_lots_including_fees),
        Side::Ask => order.max_quote_lots_including_fees,
    };

    let mut remaining_base_lots = order.max_base_lots;
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
        assert!(remaining_quote_lots >= 0);

        limit -= 1;
    }

    let total_quote_lots_taken = order_max_quote_lots - remaining_quote_lots;
    let total_base_lots_taken = order.max_base_lots - remaining_base_lots;

    let mut total_base_taken_native = (total_base_lots_taken * market.base_lot_size) as u64;

    let mut total_quote_taken_native = (total_quote_lots_taken * market.quote_lot_size) as u64;

    let mut taker_fees = 0_u64;
    let mut not_enough_liquidity = false;
    if total_quote_lots_taken > 0 || total_base_lots_taken > 0 {
        taker_fees = market.taker_fees_ceil(total_quote_taken_native);

        match side {
            Side::Bid => {
                total_quote_taken_native += taker_fees;
            }
            Side::Ask => {
                total_quote_taken_native -= taker_fees;
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

pub struct AccountOracle<'a> {
    data: &'a Vec<u8>,
    key: Pubkey,
}

impl AccountReader for AccountOracle<'_> {
    fn owner(&self) -> &Pubkey {
        return &self.key;
    }

    fn data(&self) -> &[u8] {
        return &self.data;
    }
}

impl KeyedAccountReader for AccountOracle<'_> {
    fn key(&self) -> &Pubkey {
        return &self.key;
    }
}
