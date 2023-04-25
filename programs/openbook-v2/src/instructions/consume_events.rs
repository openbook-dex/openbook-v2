use anchor_lang::prelude::*;
use bytemuck::cast_ref;

use crate::error::OpenBookError;
use crate::state::*;

use crate::accounts_ix::*;
use crate::logs::{emit_balances, FillLogV2};

/// Load a open_orders account by key from the list of account infos.
///
/// Message and return Ok() if it's missing, to lock in successful processing
/// of previous events.
///
/// Special handling for testing, where events for accounts with bad
/// owners (most likely due to force closure of the account) are being skipped.
macro_rules! load_open_orders_acc {
    ($name:ident, $key:expr, $ais:expr, $event_queue:expr) => {
        let loader = match $ais.iter().find(|ai| ai.key == &$key) {
            None => {
                msg!(
                    "Unable to find {} account {}",
                    stringify!($name),
                    $key.to_string()
                );
                return Ok(());
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

                let mal: AccountLoader<OpenOrdersAccountFixed> = AccountLoader::try_from(ai)?;
                mal
            }
        };
        let mut $name = loader.load_full_mut()?;
    };
}

pub fn consume_events(ctx: Context<ConsumeEvents>, limit: usize) -> Result<()> {
    let limit = std::cmp::min(limit, 8);

    let mut market = ctx.accounts.market.load_mut()?;
    let mut event_queue = ctx.accounts.event_queue.load_mut()?;
    let remaining_accs = &ctx.remaining_accounts;

    // Iterate over event_queue
    for _ in 0..limit {
        let event = match event_queue.peek_front() {
            None => break,
            Some(e) => e,
        };

        match EventType::try_from(event.event_type).map_err(|_| error!(OpenBookError::SomeError))? {
            EventType::Fill => {
                let fill: &FillEvent = cast_ref(event);

                // handle self trade separately because of rust borrow checker
                if fill.maker == fill.taker {
                    load_open_orders_acc!(maker_taker, fill.maker, remaining_accs, event_queue);
                    maker_taker.execute_maker(&mut market, fill)?;
                    maker_taker.execute_taker(&mut market, fill)?;
                    emit_balances(fill.maker, &maker_taker.fixed.position, &market);
                } else {
                    load_open_orders_acc!(maker, fill.maker, remaining_accs, event_queue);
                    load_open_orders_acc!(taker, fill.taker, remaining_accs, event_queue);

                    maker.execute_maker(&mut market, fill)?;
                    taker.execute_taker(&mut market, fill)?;
                    emit_balances(fill.maker, &maker.fixed.position, &market);
                    emit_balances(fill.taker, &taker.fixed.position, &market);
                }
                emit!(FillLogV2 {
                    taker_side: fill.taker_side,
                    maker_slot: fill.maker_slot,
                    maker_out: fill.maker_out(),
                    timestamp: fill.timestamp,
                    seq_num: fill.seq_num,
                    maker: fill.maker,
                    maker_client_order_id: fill.maker_client_order_id,
                    maker_fee: fill.maker_fee,
                    maker_timestamp: fill.maker_timestamp,
                    taker: fill.taker,
                    taker_client_order_id: fill.taker_client_order_id,
                    taker_fee: fill.taker_fee,
                    price: fill.price,
                    quantity: fill.quantity,
                });
            }
            EventType::Out => {
                let out: &OutEvent = cast_ref(event);
                load_open_orders_acc!(owner, out.owner, remaining_accs, event_queue);
                owner.remove_order(out.owner_slot as usize, out.quantity, true)?;
            }
        }

        // consume this event
        event_queue.pop_front()?;
    }
    Ok(())
}
