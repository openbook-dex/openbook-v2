use crate::accounts_ix::CreateOpenOrdersAccount;
use crate::pubkey_option::NonZeroKey;
use crate::state::*;
use crate::util::fill_from_str;
use anchor_lang::prelude::*;

pub fn create_open_orders_account(
    ctx: Context<CreateOpenOrdersAccount>,
    name: String,
) -> Result<()> {
    let mut account = ctx.accounts.open_orders_account.load_init()?;
    let indexer = &mut ctx.accounts.open_orders_indexer;
    indexer
        .addresses
        .push(ctx.accounts.open_orders_account.key());
    indexer.created_counter += 1;

    account.name = fill_from_str(&name)?;
    account.account_num = indexer.created_counter;
    account.market = ctx.accounts.market.key();
    account.bump = ctx.bumps.open_orders_account;
    account.owner = ctx.accounts.owner.key();
    account.delegate = ctx.accounts.delegate_account.non_zero_key();
    account.version = 1;
    account.open_orders = [OpenOrder::default(); MAX_OPEN_ORDERS];

    Ok(())
}
