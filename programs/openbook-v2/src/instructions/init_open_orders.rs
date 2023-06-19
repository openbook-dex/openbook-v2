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
    account.fixed.market = ctx.accounts.market.key();
    account.fixed.bump = *ctx.bumps.get("open_orders_account").unwrap();
    account.fixed.owner = ctx.accounts.owner.key();
    if let Some(delegate) = &ctx.accounts.delegate_account {
        account.fixed.delegate = delegate.key();
    } else {
        account.fixed.delegate = ctx.accounts.owner.key()
    };

    account.expand_dynamic_content(open_orders_count)?;

    Ok(())
}
