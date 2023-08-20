use anchor_lang::prelude::*;
use static_assertions::const_assert_eq;
use std::mem::size_of;

#[account]
#[derive(Default)]
pub struct OpenOrdersIndexer {
    pub bump: u8,
    pub total_accounts: u32,
    pub addresses: Vec<Pubkey>,
}

const_assert_eq!(size_of::<OpenOrdersIndexer>(), 32);
const_assert_eq!(size_of::<OpenOrdersIndexer>() % 8, 0);

impl OpenOrdersIndexer {
    pub fn space(len: usize) -> usize {
        8 + (4 + ((len + 1) * 32)) + 8
    }

    pub fn has_active_open_orders_accounts(&self) -> bool {
        !self.addresses.is_empty()
    }
}
