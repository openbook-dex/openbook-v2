use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

use crate::accounts_ix::*;
use crate::state::*;

pub fn settle_funds<'info>(ctx: Context<'_, '_, '_, 'info, SettleFunds<'info>>) -> Result<()> {
    let pa = &mut ctx.accounts.open_orders_account.load_mut()?.position;
    let mut market = ctx.accounts.market.load_mut()?;

    if ctx.remaining_accounts.is_empty() {
        market.quote_fees_accrued += pa.referrer_rebates_accrued;
    } else {
        market.fees_to_referrers += pa.referrer_rebates_accrued;
    }

    market.referrer_rebates_accrued -= pa.referrer_rebates_accrued;
    market.base_deposit_total -= pa.base_free_native;
    market.quote_deposit_total -= pa.quote_free_native;

    let seeds = market_seeds!(market);
    let signer = &[&seeds[..]];

    drop(market);

    if !ctx.remaining_accounts.is_empty() && pa.referrer_rebates_accrued > 0 {
        let referrer = ctx.remaining_accounts[0].to_account_info();
        let cpi_context = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.quote_vault.to_account_info(),
                to: referrer,
                authority: ctx.accounts.market.to_account_info(),
            },
        );
        token::transfer(cpi_context.with_signer(signer), pa.referrer_rebates_accrued)?;
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

    // Set to 0 after transfer
    pa.base_free_native = 0;
    pa.quote_free_native = 0;
    pa.referrer_rebates_accrued = 0;

    Ok(())
}
