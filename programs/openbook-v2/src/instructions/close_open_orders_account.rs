use crate::accounts_ix::*;
use crate::error::OpenBookError;
use anchor_lang::prelude::*;

pub fn close_open_orders_account(ctx: Context<CloseOpenOrdersAccount>) -> Result<()> {
    let open_orders_account = ctx.accounts.open_orders_account.load()?;

    require!(
        open_orders_account
            .position
            .is_empty(open_orders_account.version),
        OpenBookError::NonEmptyOpenOrdersPosition
    );

    let indexer = &mut ctx.accounts.open_orders_indexer;
    let index = indexer
        .addresses
        .iter()
        .position(|x| *x == ctx.accounts.open_orders_account.key())
        .unwrap();
    indexer.addresses.remove(index);

    Ok(())
}
