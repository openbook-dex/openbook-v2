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

    // Getting base transfer details
    let base_token_fee_wrapped = {
        get_token_fee(ctx.accounts.base_mint.to_account_info(), ctx.accounts.base_token_program.to_account_info(), base_amount)
    };

    let base_token_fee = base_token_fee_wrapped.unwrap().unwrap();

    let base_actual_amount = base_amount + base_token_fee;

    // Getting quote transfer details
    let quote_token_fee_wrapped = {
        get_token_fee(ctx.accounts.quote_mint.to_account_info(), ctx.accounts.quote_token_program.to_account_info(), quote_amount)
    };

    let quote_token_fee = quote_token_fee_wrapped.unwrap().unwrap();

    let quote_actual_amount = quote_amount + quote_token_fee;


    // Should open_orders_account and market be editted with base amount or actual amount excluding fees 
    token_transfer(
        base_actual_amount,
        &ctx.accounts.base_token_program,
        &ctx.accounts.user_base_account,
        &ctx.accounts.market_base_vault,
        &ctx.accounts.owner,
        ctx.accounts.base_mint.to_account_info(),
        market.base_decimals,
    )?;
    open_orders_account.position.base_free_native += base_amount;
    market.base_deposit_total += base_amount;

    token_transfer(
        quote_actual_amount,
        &ctx.accounts.quote_token_program,
        &ctx.accounts.user_quote_account,
        &ctx.accounts.market_quote_vault,
        &ctx.accounts.owner,
        ctx.accounts.quote_mint.to_account_info(),
        market.quote_decimals,
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
