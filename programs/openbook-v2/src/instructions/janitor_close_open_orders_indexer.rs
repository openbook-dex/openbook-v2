use crate::accounts_ix::JanitorCloseOpenOrdersIndexer;
use crate::error::OpenBookError;
use crate::state::close_with_reward;
use anchor_lang::prelude::*;

pub fn janitor_close_open_orders_indexer(
    ctx: Context<JanitorCloseOpenOrdersIndexer>,
) -> Result<()> {
    require!(
        !ctx.accounts
            .open_orders_indexer
            .has_active_open_orders_accounts(),
        OpenBookError::IndexerActiveOO
    );

    // no fee, the whole balance goes to the owner unless it can not create the owner's wallet
    close_with_reward(
        &ctx.accounts.open_orders_indexer.to_account_info(),
        None,
        &ctx.accounts.owner,
        &ctx.accounts.janitor,
        0,
        &Rent::get()?,
    )?;
    Ok(())
}
