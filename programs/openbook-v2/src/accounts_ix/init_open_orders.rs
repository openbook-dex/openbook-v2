use anchor_lang::prelude::*;

use crate::state::{Market, OpenOrdersAccount};

#[derive(Accounts)]
#[instruction(account_num: u32)]
pub struct InitOpenOrders<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
    /// CHECK:
    pub delegate_account: Option<UncheckedAccount<'info>>,
    #[account(
        init,
        seeds = [b"OpenOrders".as_ref(), owner.key().as_ref(), market.key().as_ref(), &account_num.to_le_bytes()],
        bump,
        payer = owner,
        space = OpenOrdersAccount::space()?,
    )]
    pub open_orders_account: AccountLoader<'info, OpenOrdersAccount>,
    pub market: AccountLoader<'info, Market>,
    pub system_program: Program<'info, System>,
}
