use crate::accounts_ix::*;
use anchor_lang::prelude::*;
use crate::error::OpenBookError;

pub fn close_market(ctx: Context<CloseMarket>) -> Result<()> {
    let mut event_queue = ctx.accounts.event_queue.load()?;
    require!(event_queue.header.count == 0, OpenBookError::EventQueueContainsElements);
    Ok(())
}
