use crate::accounts_ix::Deposit;
use crate::error::*;
use crate::logs::DepositLog;
use crate::token_utils::*;
use anchor_lang::prelude::*;

pub fn deposit<'info>(ctx: Context<'_, '_, '_, 'info, Deposit<'info>>, base_amount: u64, quote_amount: u64) -> Result<()> {
    let mut open_orders_account = ctx.accounts.open_orders_account.load_mut()?;
    let mut market = ctx.accounts.market.load_mut()?;
    require!(
        !market.is_expired(Clock::get()?.unix_timestamp),
        OpenBookError::MarketHasExpired
    );

    let base_mint_acc: Option<AccountInfo<'_>>;
    let quote_mint_acc: Option<AccountInfo<'_>>;

    let base_actual_amount: u64;
    let quote_actual_amount: u64;

    if let Some(base_mint) = &ctx.accounts.base_mint {
        let base_amount_wrapped = {
            calculate_amount_with_fee(base_mint.to_account_info(), ctx.accounts.base_token_program.to_account_info(), base_amount)
        };

        base_actual_amount = base_amount_wrapped.unwrap().unwrap();

        base_mint_acc = Some(base_mint.to_account_info());
    } else {
        base_actual_amount = base_amount;

        base_mint_acc = None;
    }

    if let Some(quote_mint) = &ctx.accounts.quote_mint {
        let quote_amount_wrapped = {
            calculate_amount_with_fee(quote_mint.to_account_info(), ctx.accounts.quote_token_program.to_account_info(), quote_amount)
        };

        quote_actual_amount = quote_amount_wrapped.unwrap().unwrap();

        quote_mint_acc = Some(quote_mint.to_account_info());
    } else {
        quote_actual_amount = quote_amount;

        quote_mint_acc = None;
    }

    token_transfer(
        base_actual_amount,
        &ctx.accounts.base_token_program,
        &ctx.accounts.user_base_account,
        &ctx.accounts.market_base_vault,
        &ctx.accounts.owner,
        &base_mint_acc,
        Some(market.base_decimals),
    )?;
    open_orders_account.position.base_free_native += base_amount;
    market.base_deposit_total += base_amount;

    token_transfer(
        quote_actual_amount,
        &ctx.accounts.quote_token_program,
        &ctx.accounts.user_quote_account,
        &ctx.accounts.market_quote_vault,
        &ctx.accounts.owner,
        &quote_mint_acc,
        Some(market.quote_decimals),
    )?;
    open_orders_account.position.quote_free_native += quote_amount;
    market.quote_deposit_total += quote_amount;

    if base_amount > 0 || quote_amount > 0 {
        emit!(DepositLog {
            open_orders_account: ctx.accounts.open_orders_account.key(),
            signer: ctx.accounts.owner.key(),
            base_amount,
            quote_amount,
        });
    }

    Ok(())
}
