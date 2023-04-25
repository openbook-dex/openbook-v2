use std::collections::HashMap;

use anchor_client::ClientError;

use anchor_lang::__private::bytemuck;

use openbook_v2::state::{Market, MarketIndex, OpenOrdersAccountValue, TokenIndex};

use fixed::types::I80F48;
use futures::{stream, StreamExt, TryStreamExt};
use itertools::Itertools;

use crate::gpa::*;

use solana_client::nonblocking::rpc_client::RpcClient as RpcClientAsync;
use solana_sdk::account::Account;
use solana_sdk::instruction::AccountMeta;
use solana_sdk::pubkey::Pubkey;

#[derive(Clone)]
pub struct TokenContext {
    pub token_index: TokenIndex,
    pub name: String,
    pub mint_info: MintInfo,
    pub mint_info_address: Pubkey,
    pub decimals: u8,
}

impl TokenContext {
    pub fn native_to_ui(&self, native: I80F48) -> f64 {
        (native / I80F48::from(10u64.pow(self.decimals.into()))).to_num()
    }
}

pub struct MarketContext {
    pub address: Pubkey,
    pub market: Market,
}

pub struct OpenBookContext {
    pub group: Pubkey,

    pub tokens: HashMap<TokenIndex, TokenContext>,
    pub token_indexes_by_name: HashMap<String, TokenIndex>,

    pub markets: HashMap<MarketIndex, MarketContext>,
    pub market_indexes_by_name: HashMap<String, MarketIndex>,

    pub address_lookup_tables: Vec<Pubkey>,
}

impl OpenBookContext {
    pub fn mint_info_address(&self, token_index: TokenIndex) -> Pubkey {
        self.token(token_index).mint_info_address
    }

    pub fn mint_info(&self, token_index: TokenIndex) -> MintInfo {
        self.token(token_index).mint_info
    }

    pub fn token(&self, token_index: TokenIndex) -> &TokenContext {
        self.tokens.get(&token_index).unwrap()
    }

    pub fn context(&self, market_index: MarketIndex) -> &MarketContext {
        self.markets.get(&market_index).unwrap()
    }

    pub fn token_by_mint(&self, mint: &Pubkey) -> anyhow::Result<&TokenContext> {
        self.tokens
            .iter()
            .find_map(|(_, tc)| (tc.mint_info.mint == *mint).then(|| tc))
            .ok_or_else(|| anyhow::anyhow!("no token for mint {}", mint))
    }

    pub fn market_address(&self, market_index: MarketIndex) -> Pubkey {
        self.context(market_index).address
    }

    pub async fn new_from_rpc(rpc: &RpcClientAsync) -> anyhow::Result<Self> {
        let program = openbook_v2::ID;

        // tokens
        let mint_info_tuples = fetch_mint_infos(rpc, program, group).await?;
        let mut tokens = mint_info_tuples
            .iter()
            .map(|(pk, mi)| {
                (
                    mi.token_index,
                    TokenContext {
                        token_index: mi.token_index,
                        name: String::new(),
                        mint_info: *mi,
                        mint_info_address: *pk,
                        decimals: u8::MAX,
                    },
                )
            })
            .collect::<HashMap<_, _>>();

        // markets
        let market_tuples = fetch_markets(rpc, program, group).await?;
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
        let token_indexes_by_name = tokens
            .iter()
            .map(|(i, t)| (t.name.clone(), *i))
            .collect::<HashMap<_, _>>();
        let market_indexes_by_name = markets
            .iter()
            .map(|(i, p)| (p.market.name().to_string(), *i))
            .collect::<HashMap<_, _>>();

        let group_data = fetch_anchor_account::<Group>(rpc, &group).await?;
        let address_lookup_tables = group_data
            .address_lookup_tables
            .iter()
            .filter(|&&k| k != Pubkey::default())
            .cloned()
            .collect::<Vec<Pubkey>>();

        Ok(OpenBookContext {
            group,
            tokens,
            token_indexes_by_name,
            markets,
            market_indexes_by_name,
            address_lookup_tables,
        })
    }

    pub async fn new_tokens_listed(&self, rpc: &RpcClientAsync) -> anyhow::Result<bool> {
        let mint_infos = fetch_mint_infos(rpc, openbook_v2::id(), self.group).await?;
        Ok(mint_infos.len() > self.tokens.len())
    }

    pub async fn new_markets_listed(&self, rpc: &RpcClientAsync) -> anyhow::Result<bool> {
        let new_markets = fetch_markets(rpc, openbook_v2::id(), self.group).await?;
        Ok(new_markets.len() > self.markets.len())
    }
}

fn from_serum_style_pubkey(d: [u64; 4]) -> Pubkey {
    Pubkey::new(bytemuck::cast_slice(&d as &[_]))
}

async fn fetch_raw_account(rpc: &RpcClientAsync, address: Pubkey) -> Result<Account, ClientError> {
    rpc.get_account_with_commitment(&address, rpc.commitment())
        .await?
        .value
        .ok_or(ClientError::AccountNotFound)
}
