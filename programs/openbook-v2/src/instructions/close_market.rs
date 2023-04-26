use crate::accounts_ix::*;
use crate::error::OpenBookError;
use anchor_lang::prelude::*;

pub fn close_market(ctx: Context<CloseMarket>) -> Result<()> {
    let event_queue = ctx.accounts.event_queue.load()?;
    require!(
        event_queue.is_empty(),
        OpenBookError::EventQueueContainsElements
    );
    Ok(())
}
