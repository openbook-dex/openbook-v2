use crate::accounts_ix::CloseOpenOrdersIndexer;
use crate::error::OpenBookError;
use anchor_lang::prelude::*;

pub fn close_open_orders_indexer(ctx: Context<CloseOpenOrdersIndexer>) -> Result<()> {
        // Zero discriminator to prevent account revival
        let data = account.try_borrow_mut_data()?;
        data[..8].fill(0);
    require!(
        !ctx.accounts
            .open_orders_indexer
            .has_active_open_orders_accounts(),
        OpenBookError::IndexerActiveOO
    );

    Ok(())
}
