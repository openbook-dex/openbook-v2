use anchor_lang::prelude::*;
use bytemuck::cast_ref;
use itertools::Itertools;

use crate::error::OpenBookError;
use crate::state::*;

use crate::accounts_ix::*;

// Max events to consume per ix.
pub const MAX_EVENTS_CONSUME: usize = 8;

/// Load a open_orders account by key from the list of account infos.
///
/// Message and return Ok() if it's missing, to lock in successful processing
/// of previous events.
macro_rules! load_open_orders_account {
    ($name:ident, $key:expr, $ais:expr) => {
        let loader = match $ais.iter().find(|ai| ai.key == &$key) {
            None => {
                msg!(
                    "Unable to find {} account {}, skipping",
                    stringify!($name),
                    $key.to_string()
                );
                continue;
            }

            Some(ai) => {
                let ooa: AccountLoader<OpenOrdersAccount> = AccountLoader::try_from(ai)?;
                ooa
            }
        };
        let mut $name = loader.load_mut()?;
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
                load_open_orders_account!(maker, fill.maker, remaining_accs);
                maker.execute_maker(&mut market, fill);
            }
            EventType::Out => {
                let out: &OutEvent = cast_ref(event);
                load_open_orders_account!(owner, out.owner, remaining_accs);
                owner.cancel_order(out.owner_slot as usize, out.quantity, *market);
            }
        }

        // consume this event
        event_heap.delete_slot(slot)?;
    }

    Ok(())
}
