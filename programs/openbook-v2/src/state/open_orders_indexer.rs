use anchor_lang::prelude::*;
use static_assertions::const_assert_eq;
use std::mem::size_of;

#[account(zero_copy)]
#[derive(Debug)]
pub struct OpenOrdersIndexer {
    pub owner: Pubkey,
    pub created_counter: u32,
    pub closed_counter: u32,
    pub reserved: [u8; 24],
}

const_assert_eq!(
    size_of::<OpenOrdersIndexer>(),
    size_of::<Pubkey>() + 4 + 4 + 24
);
const_assert_eq!(size_of::<OpenOrdersIndexer>(), 64);
const_assert_eq!(size_of::<OpenOrdersIndexer>() % 8, 0);
