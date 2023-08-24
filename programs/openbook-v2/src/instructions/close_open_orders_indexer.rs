use crate::accounts_ix::CloseOpenOrdersIndexer;
use crate::error::OpenBookError;
use anchor_lang::prelude::*;

pub fn close_open_orders_indexer(ctx: Context<CloseOpenOrdersIndexer>) -> Result<()> {
    require!(
        !ctx.accounts
            .open_orders_indexer
            .has_active_open_orders_accounts(),
        OpenBookError::IndexerActiveOO
    );

    Ok(())
}
