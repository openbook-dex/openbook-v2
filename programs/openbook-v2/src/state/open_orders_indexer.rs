use anchor_lang::prelude::*;
use static_assertions::const_assert_eq;
use std::mem::size_of;

#[account(zero_copy)]
#[derive(Debug)]
pub struct OpenOrdersIndexer {
    pub owner: Pubkey,
    pub market: Pubkey,
    pub bump: u8,
    pub padding: [u8; 3],
    pub created_counter: u32,
    pub closed_counter: u32,
    pub reserved: [u8; 20],
}

const_assert_eq!(
    size_of::<OpenOrdersIndexer>(),
    size_of::<Pubkey>() * 2 + 1 + 3 + 4 + 4 + 20
);
const_assert_eq!(size_of::<OpenOrdersIndexer>(), 96);
const_assert_eq!(size_of::<OpenOrdersIndexer>() % 8, 0);

impl OpenOrdersIndexer {
    /// Number of bytes needed for the account, including the discriminator
    pub fn space() -> usize {
        8 + size_of::<OpenOrdersIndexer>()
    }
}
