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
///
/// Special handling for testing, where events for accounts with bad
/// owners (most likely due to force closure of the account) are being skipped.
macro_rules! load_open_orders_account {
    ($name:ident, $key:expr, $ais:expr, $event_queue:expr) => {
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
                if ai.owner != &crate::id() {
                    msg!(
                        "OpenOrdersAccount ({}) not owned by openbook program",
                        stringify!($name)
                    );
                    $event_queue.pop_front()?;
                    continue;
                }

                let mal: AccountLoader<OpenOrdersAccount> = AccountLoader::try_from(ai)?;
                mal
            }
        };
        let mut $name = loader.load_mut()?;
    };
}

pub fn consume_events(
    ctx: Context<ConsumeEvents>,
    limit: usize,
    slots: Option<Vec<usize>>,
) -> Result<()> {
    let limit = std::cmp::min(limit, MAX_EVENTS_CONSUME);

    let mut market = ctx.accounts.market.load_mut()?;
    let mut event_queue = ctx.accounts.event_queue.load_mut()?;
    let remaining_accs = &ctx.remaining_accounts;

    let slots_to_consume = slots
        .unwrap_or_default()
        .into_iter()
        .filter(|slot| !event_queue.nodes[*slot].is_free())
        .chain(event_queue.iter().map(|(_event, slot)| slot))
        .unique()
        .take(limit)
        .collect_vec();

    for slot in slots_to_consume {
        let event = match event_queue.at_slot(slot) {
            Some(e) => e,
            None => continue,
        };

        match EventType::try_from(event.event_type).map_err(|_| error!(OpenBookError::SomeError))? {
            EventType::Fill => {
                let fill: &FillEvent = cast_ref(event);
                load_open_orders_account!(maker, fill.maker, remaining_accs, event_queue);
                maker.execute_maker(&mut market, fill);
            }
            EventType::Out => {
                let out: &OutEvent = cast_ref(event);
                load_open_orders_account!(owner, out.owner, remaining_accs, event_queue);
                owner.cancel_order(out.owner_slot as usize, out.quantity, *market);
            }
        }

        // consume this event
        event_queue.delete_slot(slot)?;
    }

    Ok(())
}
