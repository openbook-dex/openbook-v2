use crate::accounts_ix::InitOpenOrders;
use crate::state::*;
use anchor_lang::prelude::*;

pub fn init_open_orders(
    ctx: Context<InitOpenOrders>,
    account_num: u32,
    open_orders_count: u8,
) -> Result<()> {
    let mut account = ctx.accounts.open_orders_account.load_full_init()?;

    account.fixed.account_num = account_num;
    account.fixed.bump = *ctx.bumps.get("open_orders_account").unwrap();
    account.fixed.owner = ctx.accounts.owner.key();
    account.fixed.delegate = Pubkey::default();

    account.expand_dynamic_content(open_orders_count)?;

    Ok(())
}
