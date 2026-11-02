use anchor_lang::prelude::*;
use bytemuck::cast_ref;
use itertools::Itertools;

use crate::error::OpenBookError;
use crate::pubkey_option::NonZeroKey;
use crate::state::*;

use crate::accounts_ix::*;

// Max events to consume per ix.
pub const MAX_EVENTS_CONSUME: usize = 8;

/// Load a open_orders account by key from the list of account infos.
///
/// Message and return Ok() if it's missing, to lock in successful processing
/// of previous events.
///
/// `$name` is `None` for an account that is passed but closed (System owned, no data) when the
/// janitor consumes after the sunset (`$janitor`): the event is then written off against the
/// market. The key comes from the event, so it was an open orders account, and its address can
/// not be initialized again (the indexer never reuses an account number). Anyone else fails to
/// load the closed account as before.
macro_rules! load_open_orders_account {
    ($name:ident, $key:expr, $ais:expr, $janitor:expr) => {
        let ai = match $ais.iter().find(|ai| ai.key == &$key) {
            Some(ai) => ai,
            None => {
                msg!(
                    "Unable to find {} account {}, skipping",
                    stringify!($name),
                    $key.to_string()
                );
                continue;
            }
        };
        let closed = $janitor && ai.owner == &System::id() && ai.data_is_empty();
        let loader = if closed {
            msg!(
                "{} account {} is closed, writing the event off",
                stringify!($name),
                $key.to_string()
            );
            None
        } else {
            Some(AccountLoader::<OpenOrdersAccount>::try_from(ai)?)
        };
        let mut $name = match &loader {
            Some(loader) => Some(loader.load_mut()?),
            None => None,
        };
    };
}

pub fn consume_events<'c: 'info, 'info>(
    ctx: Context<'_, '_, 'c, 'info, ConsumeEvents>,
    limit: usize,
    slots: Option<Vec<usize>>,
) -> Result<()> {
    let limit = std::cmp::min(limit, MAX_EVENTS_CONSUME);

    let mut market = ctx.accounts.market.load_mut()?;
    let mut event_heap = ctx.accounts.event_heap.load_mut()?;
    let remaining_accs = &ctx.remaining_accounts;
    // After the sunset the janitor writes events of closed accounts off: whoever managed to
    // orphan an event forfeits its proceeds (burned when the market closes) instead of keeping
    // the market from closing.
    let janitor = ctx.accounts.consume_events_admin.non_zero_key() == JANITOR && janitor_active()?;

    let slots_to_consume = slots
        .unwrap_or_default()
        .into_iter()
        .filter(|slot| !event_heap.nodes[*slot].is_free())
        .chain(event_heap.iter().map(|(_event, slot)| slot))
        .unique()
        .take(limit)
        .collect_vec();

    for slot in slots_to_consume {
        let event = event_heap.at_slot(slot).unwrap();

        match EventType::try_from(event.event_type).map_err(|_| error!(OpenBookError::SomeError))? {
            EventType::Fill => {
                let fill: &FillEvent = cast_ref(event);
                load_open_orders_account!(maker, fill.maker, remaining_accs, janitor);
                match maker.as_deref_mut() {
                    Some(maker) => maker.execute_maker(&mut market, fill),
                    None => market.absorb_orphan_fill(fill),
                }
            }
            EventType::Out => {
                let out: &OutEvent = cast_ref(event);
                load_open_orders_account!(owner, out.owner, remaining_accs, janitor);
                match owner.as_deref_mut() {
                    Some(owner) => {
                        owner.cancel_order(out.owner_slot as usize, out.quantity, *market)
                    }
                    None => {
                        if !market.absorb_orphan_out(out) {
                            msg!(
                                "out event of {} carries no locked price, skipping",
                                out.owner.to_string()
                            );
                            continue;
                        }
                    }
                }
            }
        }

        // consume this event
        event_heap.delete_slot(slot)?;
    }

    Ok(())
}
