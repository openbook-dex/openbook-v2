use anchor_lang::prelude::*;

use crate::accounts_ix::*;
use crate::accounts_zerocopy::*;
use crate::error::*;
use crate::state::*;
use crate::token_utils::*;

#[allow(clippy::too_many_arguments)]
pub fn place_take_order<'info>(
    ctx: Context<'_, '_, '_, 'info, PlaceTakeOrder<'info>>,
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
    let oracle_price = if market.oracle_a.is_some() && market.oracle_b.is_some() {
        Some(market.oracle_price_from_a_and_b(
            &AccountInfoRef::borrow(ctx.accounts.oracle_a.as_ref().unwrap())?,
            &AccountInfoRef::borrow(ctx.accounts.oracle_b.as_ref().unwrap())?,
            clock.slot,
        )?)
    } else if market.oracle_a.is_some() {
        Some(market.oracle_price_from_a(
            &AccountInfoRef::borrow(ctx.accounts.oracle_a.as_ref().unwrap())?,
            clock.slot,
        )?)
    } else {
        None
    };

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
        &mut event_heap,
        oracle_price,
        None,
        &ctx.accounts.signer.key(),
        now_ts,
        limit,
        ctx.remaining_accounts,
    )?;

    let (deposit_amount, withdraw_amount) = match side {
        Side::Bid => {
            let total_quote_including_fees = total_quote_taken_native + taker_fees;
            market.base_deposit_total -= total_base_taken_native;
            market.quote_deposit_total += total_quote_including_fees;
            (total_quote_including_fees, total_base_taken_native)
        }
        Side::Ask => {
            let total_quote_discounting_fees = total_quote_taken_native - taker_fees;
            market.base_deposit_total += total_base_taken_native;
            market.quote_deposit_total -= total_quote_discounting_fees;
            (total_base_taken_native, total_quote_discounting_fees)
        }
    };

    if ctx.accounts.referrer_account.is_some() {
        market.fees_to_referrers += referrer_amount;
        market.quote_deposit_total -= referrer_amount;
    } else {
        market.fees_available += referrer_amount;
    }

    let seeds = market_seeds!(market, ctx.accounts.market.key());

    if event_heap.len() > event_heap_size_before {
        system_program_transfer(
            PENALTY_EVENT_HEAP,
            &ctx.accounts.system_program,
            &ctx.accounts.signer,
            &ctx.accounts.market,
        )?;
    }
    drop(market);

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

    if let Some(referrer_account) = &ctx.accounts.referrer_account {
        token_transfer_signed(
            referrer_amount,
            &ctx.accounts.token_program,
            &ctx.accounts.market_quote_vault,
            referrer_account,
            &ctx.accounts.market_authority,
            seeds,
        )?;
    }

    Ok(())
}
