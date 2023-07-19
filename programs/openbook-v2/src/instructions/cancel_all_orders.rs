use anchor_lang::prelude::*;

use crate::accounts_ix::*;
use crate::error::OpenBookError;
use crate::logs::CancelAllOrdersLog;
use crate::state::*;

pub fn cancel_all_orders(
    ctx: Context<CancelOrder>,
    side_option: Option<Side>,
    limit: u8,
) -> Result<()> {
    let mut account = ctx.accounts.open_orders_account.load_mut()?;
    require!(
        account.is_owner_or_delegate(ctx.accounts.owner.key()),
        OpenBookError::NoOwnerOrDelegate
    );

    let market = ctx.accounts.market.load()?;
    let mut book = Orderbook {
        bids: ctx.accounts.bids.load_mut()?,
        asks: ctx.accounts.asks.load_mut()?,
    };

    let quantity = book.cancel_all_orders(&mut account, *market, limit, side_option)?;

    emit!(CancelAllOrdersLog {
        open_orders_account: ctx.accounts.open_orders_account.key(),
        side: side_option.map(|side| side.into()),
        quantity,
        limit,
    });

    Ok(())
}
