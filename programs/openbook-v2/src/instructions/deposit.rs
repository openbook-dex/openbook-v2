use crate::accounts_ix::Deposit;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

pub fn deposit(ctx: Context<Deposit>, base_amount_lots: u64, quote_amount_lots: u64) -> Result<()> {
    let mut open_orders_account = ctx.accounts.open_orders_account.load_full_mut()?;
    let market = ctx.accounts.market.load()?;

    if base_amount_lots != 0 {
        let cpi_context = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.payer_base.to_account_info(),
                to: ctx.accounts.base_vault.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        );
        token::transfer(
            cpi_context,
            base_amount_lots * (market.base_lot_size as u64),
        )?;
        open_orders_account.fixed.position.base_free_lots += base_amount_lots as i64;
    }

    if quote_amount_lots != 0 {
        let cpi_context = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.payer_quote.to_account_info(),
                to: ctx.accounts.quote_vault.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        );
        token::transfer(
            cpi_context,
            quote_amount_lots * (market.quote_lot_size as u64),
        )?;

        open_orders_account.fixed.position.quote_free_lots += quote_amount_lots as i64;
    }

    Ok(())
}
