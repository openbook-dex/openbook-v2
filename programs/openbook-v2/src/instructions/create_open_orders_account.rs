use crate::accounts_ix::CreateOpenOrdersAccount;
use crate::pubkey_option::NonZeroKey;
use crate::state::*;
use anchor_lang::prelude::*;

pub fn create_open_orders_account(ctx: Context<CreateOpenOrdersAccount>) -> Result<()> {
    let mut account = ctx.accounts.open_orders_account.load_init()?;
    let indexer = &mut ctx.accounts.open_orders_indexer;
    indexer
        .addresses
        .push(ctx.accounts.open_orders_account.key());
    indexer.total_accounts += 1;

    account.account_num = indexer.total_accounts;
    account.market = ctx.accounts.market.key();
    account.bump = *ctx.bumps.get("open_orders_account").unwrap();
    account.owner = ctx.accounts.owner.key();
    account.delegate = ctx.accounts.delegate_account.non_zero_key();
    account.open_orders = [OpenOrder::default(); MAX_OPEN_ORDERS];

    Ok(())
}
