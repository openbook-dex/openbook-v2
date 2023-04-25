use solana_program::pubkey::*;

use super::utils::*;

#[derive(Debug, Clone, Copy)]
pub struct MintCookie {
    pub index: usize,
    pub decimals: u8,
    pub unit: f64,
    pub base_lot: f64,
    pub quote_lot: f64,
    pub pubkey: Pubkey,
    pub authority: TestKeypair,
}

#[derive(Debug, Clone)]
pub struct UserCookie {
    pub key: TestKeypair,
    pub token_accounts: Vec<Pubkey>,
}
