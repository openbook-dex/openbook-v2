use crate::error::OpenBookError;
use crate::state::{BookSide, Market, OpenOrdersAccount};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CancelOrder<'info> {
    pub signer: Signer<'info>,
    #[account(
        mut,
        has_one = market,
        constraint = open_orders_account.load()?.is_owner_or_delegate(signer.key()) @ OpenBookError::NoOwnerOrDelegate
    )]
    pub open_orders_account: AccountLoader<'info, OpenOrdersAccount>,
    #[account(
        has_one = bids,
        has_one = asks,
    )]
    pub market: AccountLoader<'info, Market>,
    #[account(mut)]
    pub bids: AccountLoader<'info, BookSide>,
    #[account(mut)]
    pub asks: AccountLoader<'info, BookSide>,
}
