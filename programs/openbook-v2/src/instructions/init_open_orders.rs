use crate::accounts_ix::InitOpenOrders;
use crate::pubkey_option::NonZeroKey;
use crate::state::*;
use anchor_lang::prelude::*;

pub fn init_open_orders(ctx: Context<InitOpenOrders>, account_num: u32) -> Result<()> {
    let mut account = ctx.accounts.open_orders_account.load_init()?;

    account.account_num = account_num;
    account.market = ctx.accounts.market.key();
    account.bump = *ctx.bumps.get("open_orders_account").unwrap();
    account.owner = ctx.accounts.owner.key();
    account.delegate = ctx.accounts.delegate_account.non_zero_key();
    account.open_orders = [OpenOrder::default(); MAX_OPEN_ORDERS];

    Ok(())
}
