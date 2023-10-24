use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct OpenOrdersIndexer {
    pub bump: u8,
    pub created_counter: u32,
    pub addresses: Vec<Pubkey>,
}

impl OpenOrdersIndexer {
    pub fn space(len: usize) -> usize {
        8 + 1 + 4 + (4 + (len * 32))
    }

    pub fn has_active_open_orders_accounts(&self) -> bool {
        !self.addresses.is_empty()
    }
}
