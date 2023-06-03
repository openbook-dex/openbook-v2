use crate::accounts_ix::Deposit;
use crate::logs::DepositLog;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

pub fn deposit(ctx: Context<Deposit>, base_amount_lots: u64, quote_amount_lots: u64) -> Result<()> {
    let mut open_orders_account = ctx.accounts.open_orders_account.load_full_mut()?;
    let mut market = ctx.accounts.market.load_mut()?;

    if base_amount_lots != 0 {
        let base_amount_native = base_amount_lots * (market.base_lot_size as u64);
        let cpi_context = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.token_base_account.to_account_info(),
                to: ctx.accounts.base_vault.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        );
        token::transfer(cpi_context, base_amount_native)?;
        open_orders_account.fixed.position.base_free_native += base_amount_native;
        market.base_deposit_total += base_amount_native;

        emit!(DepositLog {
            open_orders_acc: ctx.accounts.open_orders_account.key(),
            signer: ctx.accounts.owner.key(),
            quantity: base_amount_native,
        });
    }

    if quote_amount_lots != 0 {
        let quote_amount_native = quote_amount_lots * (market.quote_lot_size as u64);
        let cpi_context = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.token_quote_account.to_account_info(),
                to: ctx.accounts.quote_vault.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        );
        token::transfer(cpi_context, quote_amount_native)?;

        open_orders_account.fixed.position.quote_free_native += quote_amount_native;
        market.quote_deposit_total += quote_amount_native;

        emit!(DepositLog {
            open_orders_acc: ctx.accounts.open_orders_account.key(),
            signer: ctx.accounts.owner.key(),
            quantity: quote_amount_native,
        });
    }

    Ok(())
}
