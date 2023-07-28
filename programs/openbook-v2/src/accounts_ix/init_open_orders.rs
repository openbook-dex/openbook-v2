use crate::state::{Market, OpenOrdersAccount, OpenOrdersIndexer};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(account_num: u32)]
pub struct InitOpenOrders<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub owner: Signer<'info>,
    /// CHECK:
    pub delegate_account: Option<UncheckedAccount<'info>>,
    #[account(
        mut,
        has_one = owner,
        has_one = market
    )]
    pub open_orders_indexer: AccountLoader<'info, OpenOrdersIndexer>,
    #[account(
        init,
        seeds = [b"OpenOrders".as_ref(), owner.key().as_ref(), market.key().as_ref(), &(open_orders_indexer.load()?.created_counter + 1).to_le_bytes()],
        bump,
        payer = payer,
        space = OpenOrdersAccount::space()?,
    )]
    pub open_orders_account: AccountLoader<'info, OpenOrdersAccount>,
    pub market: AccountLoader<'info, Market>,
    pub system_program: Program<'info, System>,
}
