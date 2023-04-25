use crate::state::{BookSide, Market, OpenOrdersAccountFixed};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CancelOrder<'info> {
    #[account(mut)]
    pub open_orders_account: AccountLoader<'info, OpenOrdersAccountFixed>,
    pub owner: Signer<'info>,

    #[account(
        mut,
        has_one = bids,
        has_one = asks,
    )]
    pub market: AccountLoader<'info, Market>,
    #[account(mut)]
    pub bids: AccountLoader<'info, BookSide>,
    #[account(mut)]
    pub asks: AccountLoader<'info, BookSide>,
}
