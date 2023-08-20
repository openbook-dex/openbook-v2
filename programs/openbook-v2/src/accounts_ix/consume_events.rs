use crate::error::OpenBookError;
use crate::pubkey_option::NonZeroKey;
use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct ConsumeEvents<'info> {
    pub consume_events_admin: Option<Signer<'info>>,
    #[account(
        mut,
        has_one = event_heap,
        constraint = market.load()?.consume_events_admin == consume_events_admin.non_zero_key() @ OpenBookError::InvalidConsumeEventsAdmin
    )]
    pub market: AccountLoader<'info, Market>,
    #[account(mut)]
    pub event_heap: AccountLoader<'info, EventHeap>,
}
