use anchor_lang::prelude::*;

use anchor_spl::token::{self, Transfer};

use crate::accounts_ix::*;
use crate::accounts_zerocopy::*;
use crate::error::*;
use crate::state::*;

// TODO
#[allow(clippy::too_many_arguments)]
pub fn place_take_order(
    ctx: Context<PlaceTakeOrder>,
    order: Order,
    limit: u8,
) -> Result<Option<u128>> {
    require_gte!(order.max_base_lots, 0);
    require_gte!(order.max_quote_lots_including_fees, 0);
    require!(order.is_post_only(), OpenBookError::InvalidFeesError);

    let _now_ts: u64 = Clock::get()?.unix_timestamp.try_into().unwrap();
    let oracle_price;
    {
        let market = ctx.accounts.market.load_mut()?;
        oracle_price = market.oracle_price(
            &AccountInfoRef::borrow(ctx.accounts.oracle.as_ref())?,
            None, // staleness checked in health
        )?;
    }

    let mut market = ctx.accounts.market.load_mut()?;
    let mut book = Orderbook {
        bids: ctx.accounts.bids.load_mut()?,
        asks: ctx.accounts.asks.load_mut()?,
    };

    let mut event_queue = ctx.accounts.event_queue.load_mut()?;

    let now_ts: u64 = Clock::get()?.unix_timestamp.try_into().unwrap();

    let side = order.side;

    let TakenQuantitiesIncludingFees {
        order_id,
        total_base_lots_taken,
        total_quote_lots_taken_native,
    } = book.new_order(
        order,
        &mut market,
        &mut event_queue,
        oracle_price,
        None,
        &ctx.accounts.owner.key(),
        now_ts,
        limit,
    )?;

    let (to_vault, deposit_amount) = match side {
        Side::Bid => (
            ctx.accounts.quote_vault.to_account_info(),
            total_quote_lots_taken_native,
        ),

        Side::Ask => (
            ctx.accounts.base_vault.to_account_info(),
            total_base_lots_taken,
        ),
    };

    // Transfer funds
    if let Some(amount) = deposit_amount {
        if amount > 0 {
            let cpi_context = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.payer.to_account_info(),
                    to: to_vault,
                    authority: ctx.accounts.owner.to_account_info(),
                },
            );
            token::transfer(cpi_context, amount.to_num())?;
        }
    }
    Ok(order_id)
}
