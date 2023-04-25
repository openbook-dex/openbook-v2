use anchor_lang::prelude::*;
use fixed::types::I80F48;

use crate::health::HealthCache;
use crate::state::{MarketIndex, TokenIndex};

#[event]
pub struct OpenOrdersAccountData {
    pub health_cache: HealthCache,
    pub init_health: I80F48,
    pub maint_health: I80F48,
    pub equity: Equity,
}

#[derive(AnchorDeserialize, AnchorSerialize, Debug)]
pub struct Equity {
    pub tokens: Vec<TokenEquity>,
    pub perps: Vec<Equity>,
}

#[derive(AnchorDeserialize, AnchorSerialize, Debug)]
pub struct TokenEquity {
    pub token_index: TokenIndex,
    pub value: I80F48, // in native quote
}

#[derive(AnchorDeserialize, AnchorSerialize, Debug)]
pub struct Equity {
    pub market_index: MarketIndex,
    value: I80F48, // in native quote
}
