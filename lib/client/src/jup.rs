use anyhow::Result;
use openbook_v2::state::{SelfTradeBehavior::DecrementTake, Order, OrderWithAmounts, Orderbook, PlaceOrderType, Side, EventQueue, Market};

/// An abstraction in order to share reserve mints and necessary data
use solana_sdk::{account::Account, instruction::AccountMeta, pubkey::Pubkey};
use std::collections::HashMap;
use std::str;

pub struct QuoteParams {
    pub max_base_lots: u64,
    pub max_quote_lots_including_fees: u64,
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Quote {
    pub not_enough_liquidity: bool,
    pub min_in_amount: Option<u64>,
    pub min_out_amount: Option<u64>,
    pub in_amount: u64,
    pub out_amount: u64,
    pub fee_amount: u64,
    pub fee_mint: Pubkey,
    pub fee_pct: Decimal,
    pub price_impact_pct: Decimal,
}

pub trait Amm {
    // Label of your Amm
    fn label(&self) -> String;
    // Identifier for your amm, should be your pool id
    fn key(&self) -> Pubkey;
    // The token mints that the pool support for swapping
    fn get_reserve_mints(&self) -> Vec<Pubkey>;
    // Related accounts to get the quote for swapping and creating ix
    fn get_accounts_to_update(&self) -> Vec<Pubkey>;
    // Picks data necessary to update it's internal state
    fn update(&mut self, accounts_map: &HashMap<Pubkey, Vec<u8>>) -> Result<()>;
    // Compute the quote from internal state
    fn quote(&self, quote_params: &QuoteParams) -> Result<Quote>;
}

pub struct OpenBookMarket {
    market: Market,
    key: Pubkey,
    label: String,
    related_accounts: [Pubkey; 6],
    reserve_mints: [Pubkey; 2],
    reserves: [u128; 2],
    program_id: Pubkey,
}

impl OpenBookMarket {
    pub fn new_from_market(key: Pubkey, market: Market) -> OpenBookMarket {
        OpenBookMarket {
            market,
            key,
            label: str::from_utf8(&market.name).unwrap_or_else(|d| "").to_string(),
            related_accounts: [
                market.bids,
                market.asks,
                market.event_queue,
                market.oracle,
                market.base_vault,
                market.quote_vault,
            ],
            reserve_mints: [market.base_mint, market.quote_mint],
            reserves: [0, 0],
            program_id: openbook_v2::ID,
        }
    }
}

impl Amm for OpenBookMarket {
    fn label(&self) -> String {
        self.label.clone()
    }

    fn key(&self) -> Pubkey {
        self.key
    }

    fn get_reserve_mints(&self) -> Vec<Pubkey> {
        self.reserve_mints.to_vec()
    }

    fn get_accounts_to_update(&self) -> Vec<Pubkey> {
        self.related_accounts.to_vec()
    }

    fn update(&mut self, accounts_map: &HashMap<Pubkey, Vec<u8>>) -> Result<()> {
        Ok(())
    }

    fn quote(&self, quote_params: &QuoteParams) -> Result<Quote> {
        let mut book = Orderbook {
            bids: self.market.bids,
            asks: self.market.asks,
        };
        let side = if quote_params.input_mint == self.market.quote_mint {
            Side::Bid
        } else {
            Side::Ask
        };

        let order = &Order {
            side,
            max_base_lots: quote_params.max_base_lots,
            max_quote_lots_including_fees: quote_params.max_quote_lots_including_fees,
            client_order_id: 0,
            time_in_force: 0,
            self_trade_behavior: DecrementTake,
            params: match order_type {
                PlaceOrderType::Market => OrderParams::Market,
                _ => unreachable!(),
            },
        };
        let owner = &Pubkey::default();
        let mut event_queue = &EventQueue{header: , nodes: , reserved:};
        event_queue.init();
        let order_amounts: OrderWithAmounts = book.new_order(
            order,
            &mut self.market,
            event_queue,
            0,
            None,
            owner,
            0,
            8,
            None,
            [],
        )?;
        let out_amount = if side == Side::Bid {
            order_amounts.total_base_taken_native
        } else {
            order_amounts.total_quote_taken_native
        };

        Ok(Quote {
            out_amount,
            ..Quote::default()
        })
    }
}
