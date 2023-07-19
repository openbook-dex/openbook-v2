use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

use crate::accounts_ix::*;
use crate::accounts_zerocopy::*;
use crate::error::*;
use crate::state::*;

#[allow(clippy::too_many_arguments)]
pub fn place_take_order<'info>(
    ctx: Context<'_, '_, '_, 'info, PlaceTakeOrder<'info>>,
    order: Order,
    limit: u8,
) -> Result<Option<u128>> {
    require_gte!(order.max_base_lots, 0, OpenBookError::InvalidInputLots);
    require_gte!(
        order.max_quote_lots_including_fees,
        0,
        OpenBookError::InvalidInputLots
    );

    let mut market = ctx.accounts.market.load_mut()?;
    require!(
        market.time_expiry == 0 || market.time_expiry > Clock::get()?.unix_timestamp,
        OpenBookError::MarketHasExpired
    );
    if let Some(open_orders_admin) = Option::<Pubkey>::from(market.open_orders_admin) {
        let open_orders_admin_signer = ctx
            .accounts
            .open_orders_admin
            .as_ref()
            .map(|signer| signer.key())
            .ok_or(OpenBookError::MissingOpenOrdersAdmin)?;
        require_eq!(
            open_orders_admin,
            open_orders_admin_signer,
            OpenBookError::InvalidOpenOrdersAdmin
        );
    }

    let mut book = Orderbook {
        bids: ctx.accounts.bids.load_mut()?,
        asks: ctx.accounts.asks.load_mut()?,
    };

    let mut event_queue = ctx.accounts.event_queue.load_mut()?;

    let now_ts: u64 = Clock::get()?.unix_timestamp.try_into().unwrap();
    let oracle_price = if let Some(oracle_acc) = &ctx.accounts.oracle {
        market.oracle_price(&AccountInfoRef::borrow(oracle_acc)?, Clock::get()?.slot)?
    } else {
        fixed::types::I80F48::ZERO
    };

    let side = order.side;

    let OrderWithAmounts {
        order_id,
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
                ctx.accounts.base_vault.to_account_info(),
                ctx.accounts.quote_vault.to_account_info(),
                total_quote_including_fees,
                total_base_taken_native,
            )
        }
        Side::Ask => {
            let total_quote_discounting_fees = total_quote_taken_native - taker_fees;
            market.base_deposit_total += total_base_taken_native;
            market.quote_deposit_total -= total_quote_discounting_fees;
            (
                ctx.accounts.quote_vault.to_account_info(),
                ctx.accounts.base_vault.to_account_info(),
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
    let signer = &[&seeds[..]];
    drop(market);

    // Transfer funds from token_deposit_account to vault
    if deposit_amount > 0 {
        let cpi_context = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.token_deposit_account.to_account_info(),
                to: to_vault,
                authority: ctx.accounts.signer.to_account_info(),
            },
        );
        token::transfer(cpi_context, deposit_amount)?;
    }

    if withdraw_amount > 0 {
        let cpi_context = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: from_vault,
                to: ctx.accounts.token_receiver_account.to_account_info(),
                authority: ctx.accounts.market.to_account_info(),
            },
        );
        token::transfer(cpi_context.with_signer(signer), withdraw_amount)?;
    }

    // Transfer to referrer
    if let Some(referrer) = &ctx.accounts.referrer {
        if referrer_amount > 0 {
            let cpi_context = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.quote_vault.to_account_info(),
                    to: referrer.to_account_info(),
                    authority: ctx.accounts.market.to_account_info(),
                },
            );
            token::transfer(cpi_context.with_signer(signer), referrer_amount)?;
        }
    }

    Ok(order_id)
}
