use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

use crate::accounts_ix::*;
use crate::state::*;

pub fn settle_funds<'info>(ctx: Context<'_, '_, '_, 'info, SettleFunds<'info>>) -> Result<()> {
    let mut open_orders_account = ctx.accounts.open_orders_account.load_mut()?;
    let mut market = ctx.accounts.market.load_mut()?;

    let mut roundoff_maker_fees = 0;

    if market.maker_fee.is_positive()
        && !open_orders_account
            .all_orders_in_use()
            .any(|oo| oo.side_and_tree().side() == Side::Bid)
    {
        roundoff_maker_fees = open_orders_account.position.locked_maker_fees;
        open_orders_account.position.locked_maker_fees = 0;
    }

    let pa = &mut open_orders_account.position;
    let referrer_rebate = pa.referrer_rebates_accrued + roundoff_maker_fees;

    if ctx.remaining_accounts.is_empty() {
        market.quote_fees_accrued += referrer_rebate;
    } else {
        market.fees_to_referrers += referrer_rebate;
    }

    market.base_deposit_total -= pa.base_free_native;
    market.quote_deposit_total -= pa.quote_free_native;
    market.referrer_rebates_accrued -= pa.referrer_rebates_accrued;

    let seeds = market_seeds!(market);
    let signer = &[&seeds[..]];

    drop(market);

    if !ctx.remaining_accounts.is_empty() && referrer_rebate > 0 {
        let referrer = ctx.remaining_accounts[0].to_account_info();
        let cpi_context = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.quote_vault.to_account_info(),
                to: referrer,
                authority: ctx.accounts.market.to_account_info(),
            },
        );
        token::transfer(cpi_context.with_signer(signer), referrer_rebate)?;
    }

    if pa.base_free_native > 0 {
        let cpi_context = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.base_vault.to_account_info(),
                to: ctx.accounts.token_base_account.to_account_info(),
                authority: ctx.accounts.market.to_account_info(),
            },
        );
        token::transfer(cpi_context.with_signer(signer), pa.base_free_native)?;
    }

    if pa.quote_free_native > 0 {
        let cpi_context = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.quote_vault.to_account_info(),
                to: ctx.accounts.token_quote_account.to_account_info(),
                authority: ctx.accounts.market.to_account_info(),
            },
        );
        token::transfer(cpi_context.with_signer(signer), pa.quote_free_native)?;
    }

    pa.base_free_native = 0;
    pa.quote_free_native = 0;
    pa.referrer_rebates_accrued = 0;

    Ok(())
}
