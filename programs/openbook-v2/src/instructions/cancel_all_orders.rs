use anchor_lang::prelude::*;

use crate::accounts_ix::*;
use crate::error::OpenBookError;
use crate::state::*;

pub fn cancel_all_orders(ctx: Context<CancelAllOrders>, limit: u8) -> Result<()> {
    let mut account = ctx.accounts.open_orders_account.load_full_mut()?;
    // account constraint #1
    require!(
        account.fixed.is_owner_or_delegate(ctx.accounts.owner.key()),
        OpenBookError::NoOwnerOrDelegate
    );

    let market = ctx.accounts.market.load()?;
    let mut book = Orderbook {
        bids: ctx.accounts.bids.load_mut()?,
        asks: ctx.accounts.asks.load_mut()?,
    };

    book.cancel_all_orders(&mut account.borrow_mut(), *market, limit, None)?;

    Ok(())
}
