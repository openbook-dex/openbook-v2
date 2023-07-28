use crate::accounts_ix::CreateOpenOrdersIndexer;
use anchor_lang::prelude::*;

pub fn create_open_orders_indexer(ctx: Context<CreateOpenOrdersIndexer>) -> Result<()> {
    let mut account = ctx.accounts.open_orders_indexer.load_init()?;

    account.owner = ctx.accounts.owner.key();
    account.market = ctx.accounts.market.key();
    account.bump = *ctx.bumps.get("open_orders_indexer").unwrap();

    account.created_counter = 0;
    account.closed_counter = 0;

    Ok(())
}
