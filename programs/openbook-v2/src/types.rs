use anchor_lang::prelude::*;
use static_assertions::const_assert_eq;
use std::mem::size_of;

/// Nothing in Rust shall use these types. They only exist so that the Anchor IDL
/// knows about them and typescript can deserialize it.

#[derive(AnchorSerialize, AnchorDeserialize, Default)]
pub struct MarketIndex {
    val: u32,
}
const_assert_eq!(
    size_of::<MarketIndex>(),
    size_of::<crate::state::MarketIndex>(),
);

#[derive(AnchorSerialize, AnchorDeserialize, Default)]
pub struct I80F48 {
    val: i128,
}
