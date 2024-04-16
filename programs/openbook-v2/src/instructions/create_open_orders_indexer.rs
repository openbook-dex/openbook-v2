use crate::accounts_ix::CreateOpenOrdersIndexer;
use anchor_lang::prelude::*;

pub fn create_open_orders_indexer(ctx: Context<CreateOpenOrdersIndexer>) -> Result<()> {
    let indexer = &mut ctx.accounts.open_orders_indexer;

    indexer.bump = ctx.bumps.open_orders_indexer;
    indexer.created_counter = 0;

    Ok(())
}
