use crate::accounts_ix::CreateOpenOrdersIndexer;
use anchor_lang::prelude::*;

pub fn create_open_orders_indexer(ctx: Context<CreateOpenOrdersIndexer>) -> Result<()> {
    let mut indexer = ctx.accounts.open_orders_indexer.load_init()?;

    indexer.owner = ctx.accounts.owner.key();
    indexer.market = ctx.accounts.market.key();
    indexer.bump = *ctx.bumps.get("open_orders_indexer").unwrap();

    indexer.created_counter = 0;
    indexer.closed_counter = 0;

    Ok(())
}
