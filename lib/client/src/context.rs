use openbook_v2::state::{Market, MarketIndex, FEES_SCALE_FACTOR};
use std::collections::HashMap;
use std::convert::TryInto;

use crate::gpa::*;

use solana_client::nonblocking::rpc_client::RpcClient as RpcClientAsync;
use solana_sdk::pubkey::Pubkey;

pub struct MarketContext {
    pub address: Pubkey,
    pub market: Market,
}

impl MarketContext {
    pub fn max_quote_lots_including_taker_fees_from_usd(&self, quote_size_usd: u64) -> u64 {
        self.max_quote_lots_including_taker_fees(quote_size_usd * 10u64.pow(6))
    }
    pub fn max_quote_lots_including_maker_fees_from_usd(&self, quote_size_usd: u64) -> u64 {
        self.max_quote_lots_including_maker_fees(quote_size_usd * 10u64.pow(6))
    }
    pub fn max_base_lots_from_usd(&self, base_size: u64) -> u64 {
        self.max_base_lots(base_size * self.market.base_decimals as u64)
    }

    // For orders where matching is possible, therefore taker fees can apply. Assume order incurs in max fees.
    pub fn max_quote_lots_including_taker_fees(&self, quote_size: u64) -> u64 {
        let quote_lots: u64 = quote_size / (self.market.quote_lot_size as u64);
        let fees: u64 = ((quote_size as i128 * self.market.taker_fee as i128)
            / (FEES_SCALE_FACTOR + self.market.taker_fee as i128))
            .try_into()
            .unwrap();
        quote_lots + fees
    }
    // For PostOnly or PostOnlySlide orders.
    pub fn max_quote_lots_including_maker_fees(&self, quote_size: u64) -> u64 {
        let quote_lots: u64 = quote_size / (self.market.quote_lot_size as u64);
        let fees: u64 = self.market.maker_fees_floor(quote_size);
        quote_lots + fees
    }

    pub fn max_base_lots(&self, base_size: u64) -> u64 {
        base_size / (self.market.base_lot_size as u64)
    }
}

pub struct OpenBookContext {
    pub markets: HashMap<MarketIndex, MarketContext>,
    pub market_indexes_by_name: HashMap<String, MarketIndex>,
}

impl OpenBookContext {
    pub fn context(&self, market_index: MarketIndex) -> &MarketContext {
        self.markets.get(&market_index).unwrap()
    }

    pub fn market_address(&self, market_index: MarketIndex) -> Pubkey {
        self.context(market_index).address
    }

    pub async fn new_from_rpc(rpc: &RpcClientAsync) -> anyhow::Result<Self> {
        let program = openbook_v2::ID;

        // markets
        let market_tuples = fetch_markets(rpc).await?;
        let markets = market_tuples
            .iter()
            .map(|(pk, pm)| {
                (
                    pm.market_index,
                    MarketContext {
                        address: *pk,
                        market: *pm,
                    },
                )
            })
            .collect::<HashMap<_, _>>();

        // Name lookup tables
        let market_indexes_by_name = markets
            .iter()
            .map(|(i, p)| (p.market.name().to_string(), *i))
            .collect::<HashMap<_, _>>();

        Ok(OpenBookContext {
            markets,
            market_indexes_by_name,
        })
    }

    pub async fn new_markets_listed(&self, rpc: &RpcClientAsync) -> anyhow::Result<bool> {
        let new_markets = fetch_markets(rpc).await?;
        Ok(new_markets.len() > self.markets.len())
    }
}
