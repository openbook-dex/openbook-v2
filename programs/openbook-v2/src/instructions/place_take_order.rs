use anchor_lang::prelude::*;

use anchor_spl::token::{self, Transfer};

use crate::accounts_ix::*;
use crate::accounts_zerocopy::*;
use crate::state::*;

// TODO
#[allow(clippy::too_many_arguments)]
pub fn place_take_order<'info>(
    ctx: Context<'_, '_, '_, 'info, PlaceTakeOrder<'info>>,
    order: Order,
    limit: u8,
) -> Result<Option<u128>> {
    require_gte!(order.max_base_lots, 0);
    require_gte!(order.max_quote_lots_including_fees, 0);

    let mut market = ctx.accounts.market.load_mut()?;
    let mut book = Orderbook {
        bids: ctx.accounts.bids.load_mut()?,
        asks: ctx.accounts.asks.load_mut()?,
    };

    let mut event_queue = ctx.accounts.event_queue.load_mut()?;

    let now_ts: u64 = Clock::get()?.unix_timestamp.try_into().unwrap();
    let oracle_price = market.oracle_price(
        &AccountInfoRef::borrow(ctx.accounts.oracle.as_ref())?,
        Clock::get()?.slot,
    )?;

    let side = order.side;

    let OrderWithAmounts {
        order_id,
        total_base_taken_native,
        total_quote_taken_native,
        referrer_amount,
        ..
    } = book.new_order(
        &order,
        &mut market,
        &mut event_queue,
        oracle_price,
        &mut None,
        &ctx.accounts.owner.key(),
        now_ts,
        limit,
        ctx.accounts
            .open_orders_admin
            .as_ref()
            .map(|signer| signer.key()),
        ctx.remaining_accounts,
    )?;

    let (from_vault, to_vault, deposit_amount, withdraw_amount) = match side {
        Side::Bid => {
            // Update market deposit total
            market.quote_deposit_total += total_quote_taken_native;
            (
                ctx.accounts.base_vault.to_account_info(),
                ctx.accounts.quote_vault.to_account_info(),
                total_quote_taken_native,
                total_base_taken_native,
            )
        }

        Side::Ask => {
            // Update market deposit total
            market.base_deposit_total += total_base_taken_native;
            (
                ctx.accounts.quote_vault.to_account_info(),
                ctx.accounts.base_vault.to_account_info(),
                total_base_taken_native,
                total_quote_taken_native,
            )
        }
    };

    // Transfer funds from token_deposit_account to vault
    if deposit_amount > 0 {
        let cpi_context = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.token_deposit_account.to_account_info(),
                to: to_vault,
                authority: ctx.accounts.owner.to_account_info(),
            },
        );
        token::transfer(cpi_context, deposit_amount)?;
    }

    let seeds = market_seeds!(market);
    let signer = &[&seeds[..]];

    drop(market);

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
    if !ctx.remaining_accounts.is_empty() && referrer_amount > 0 {
        let referrer = ctx.remaining_accounts[0].to_account_info();
        let cpi_context = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.quote_vault.to_account_info(),
                to: referrer,
                authority: ctx.accounts.market.to_account_info(),
            },
        );
        token::transfer(cpi_context.with_signer(signer), referrer_amount)?;
    }

    Ok(order_id)
}
