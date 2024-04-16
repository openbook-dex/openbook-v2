use anchor_lang::prelude::*;

use crate::accounts_ix::*;
use crate::accounts_zerocopy::AccountInfoRef;
use crate::error::*;
use crate::state::*;
use crate::token_utils::*;

#[allow(clippy::too_many_arguments)]
pub fn place_take_order<'c: 'info, 'info>(
    ctx: Context<'_, '_, 'c, 'info, PlaceTakeOrder<'info>>,
    order: Order,
    limit: u8,
) -> Result<()> {
    require_gte!(order.max_base_lots, 0, OpenBookError::InvalidInputLots);
    require_gte!(
        order.max_quote_lots_including_fees,
        0,
        OpenBookError::InvalidInputLots
    );

    let clock = Clock::get()?;

    let mut market = ctx.accounts.market.load_mut()?;
    require!(
        !market.is_expired(clock.unix_timestamp),
        OpenBookError::MarketHasExpired
    );

    let mut book = Orderbook {
        bids: ctx.accounts.bids.load_mut()?,
        asks: ctx.accounts.asks.load_mut()?,
    };

    let mut event_heap = ctx.accounts.event_heap.load_mut()?;
    let event_heap_size_before = event_heap.len();

    let now_ts: u64 = clock.unix_timestamp.try_into().unwrap();

    let oracle_price_lots = market.oracle_price_lots(
        AccountInfoRef::borrow_some(ctx.accounts.oracle_a.as_ref())?.as_ref(),
        AccountInfoRef::borrow_some(ctx.accounts.oracle_b.as_ref())?.as_ref(),
        clock.slot,
    )?;

    let side = order.side;

    let OrderWithAmounts {
        total_base_taken_native,
        total_quote_taken_native,
        referrer_amount,
        taker_fees,
        ..
    } = book.new_order(
        &order,
        &mut market,
        &ctx.accounts.market.key(),
        &mut event_heap,
        oracle_price_lots,
        None,
        &ctx.accounts.signer.key(),
        now_ts,
        limit,
        ctx.remaining_accounts,
    )?;

    // place_take_orders doesnt pay to referrers
    let makers_rebates = taker_fees - referrer_amount;

    let (deposit_amount, withdraw_amount) = match side {
        Side::Bid => {
            let total_quote_including_fees = total_quote_taken_native + makers_rebates;
            market.base_deposit_total -= total_base_taken_native;
            market.quote_deposit_total += total_quote_including_fees;
            (total_quote_including_fees, total_base_taken_native)
        }
        Side::Ask => {
            let total_quote_discounting_fees = total_quote_taken_native - makers_rebates;
            market.base_deposit_total += total_base_taken_native;
            market.quote_deposit_total -= total_quote_discounting_fees;
            (total_base_taken_native, total_quote_discounting_fees)
        }
    };

    let seeds = market_seeds!(market, ctx.accounts.market.key());

    drop(market);

    if event_heap.len() > event_heap_size_before {
        system_program_transfer(
            PENALTY_EVENT_HEAP,
            &ctx.accounts.system_program,
            &ctx.accounts.penalty_payer,
            &ctx.accounts.market,
        )?;
    }

    let (user_deposit_acc, user_withdraw_acc, market_deposit_acc, market_withdraw_acc) = match side
    {
        Side::Bid => (
            &ctx.accounts.user_quote_account,
            &ctx.accounts.user_base_account,
            &ctx.accounts.market_quote_vault,
            &ctx.accounts.market_base_vault,
        ),
        Side::Ask => (
            &ctx.accounts.user_base_account,
            &ctx.accounts.user_quote_account,
            &ctx.accounts.market_base_vault,
            &ctx.accounts.market_quote_vault,
        ),
    };

    token_transfer(
        deposit_amount,
        &ctx.accounts.token_program,
        user_deposit_acc.as_ref(),
        market_deposit_acc,
        &ctx.accounts.signer,
    )?;

    token_transfer_signed(
        withdraw_amount,
        &ctx.accounts.token_program,
        market_withdraw_acc,
        user_withdraw_acc.as_ref(),
        &ctx.accounts.market_authority,
        seeds,
    )?;

    Ok(())
}
