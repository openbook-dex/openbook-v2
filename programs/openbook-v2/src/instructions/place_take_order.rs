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

    let mut event_queue = ctx.accounts.event_queue.load_mut()?;

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
        &mut event_queue,
        oracle_price,
        None,
        &ctx.accounts.signer.key(),
        now_ts,
        limit,
        ctx.remaining_accounts,
    )?;

    let (from_vault, to_vault, deposit_amount, withdraw_amount) = match side {
        Side::Bid => {
            let total_quote_including_fees = total_quote_taken_native + taker_fees;
            market.base_deposit_total -= total_base_taken_native;
            market.quote_deposit_total += total_quote_including_fees;
            (
                &ctx.accounts.base_vault,
                &ctx.accounts.quote_vault,
                total_quote_including_fees,
                total_base_taken_native,
            )
        }
        Side::Ask => {
            let total_quote_discounting_fees = total_quote_taken_native - taker_fees;
            market.base_deposit_total += total_base_taken_native;
            market.quote_deposit_total -= total_quote_discounting_fees;
            (
                &ctx.accounts.quote_vault,
                &ctx.accounts.base_vault,
                total_base_taken_native,
                total_quote_discounting_fees,
            )
        }
    };

    if ctx.accounts.referrer.is_some() {
        market.fees_to_referrers += referrer_amount;
        market.quote_deposit_total -= referrer_amount;
    } else {
        market.quote_fees_accrued += referrer_amount;
    }

    let seeds = market_seeds!(market);
    drop(market);

    // Transfer funds from token_deposit_account to vault
    token_transfer(
        deposit_amount,
        &ctx.accounts.token_program,
        ctx.accounts.token_deposit_account.as_ref(),
        to_vault,
        &ctx.accounts.signer,
    )?;

    token_transfer_signed(
        withdraw_amount,
        &ctx.accounts.token_program,
        from_vault,
        ctx.accounts.token_receiver_account.as_ref(),
        &ctx.accounts.market,
        seeds,
    )?;

    // Transfer to referrer
    if let Some(referrer) = &ctx.accounts.referrer {
        token_transfer_signed(
            referrer_amount,
            &ctx.accounts.token_program,
            &ctx.accounts.quote_vault,
            referrer,
            &ctx.accounts.market,
            seeds,
        )?;
    }

    Ok(())
}
