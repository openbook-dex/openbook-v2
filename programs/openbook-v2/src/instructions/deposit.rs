use crate::accounts_ix::Deposit;
use crate::error::*;
use crate::logs::DepositLog;
use crate::token_utils::*;
use anchor_lang::prelude::*;

pub fn deposit(ctx: Context<Deposit>, base_amount: u64, quote_amount: u64) -> Result<()> {
    let mut open_orders_account = ctx.accounts.open_orders_account.load_mut()?;
    let mut market = ctx.accounts.market.load_mut()?;
    require!(
        !market.is_expired(Clock::get()?.unix_timestamp),
        OpenBookError::MarketHasExpired
    );

    let remaining_accounts = ctx.remaining_accounts;

    // Getting actual base token amount to be deposited
    
    let base_token_account_info = remaining_accounts[0]; // base token mint

    let base_token_fee_wrapped = get_token_fee(base_token_account_info, ctx.accounts.v2_token_program.to_account_info(), base_amount);
    let base_token_fee = base_token_fee_wrapped.unwrap().unwrap();

    let quote_token_account_info = remaining_accounts[1]; // quote token mint

    let quote_token_fee_wrapped = get_token_fee(base_token_account_info, ctx.accounts.v2_token_program.to_account_info(), quote_amount);
    let quote_token_fee = quote_token_fee_wrapped.unwrap().unwrap();

    let base_actual_amount = base_amount + base_token_fee;

    let quote_actual_amount = quote_amount + quote_token_fee;


    token_transfer(
        base_actual_amount,
        &ctx.accounts.token_program,
        &ctx.accounts.user_base_account,
        &ctx.accounts.market_base_vault,
        &ctx.accounts.owner,
    )?;
    open_orders_account.position.base_free_native += base_amount;
    market.base_deposit_total += base_amount;

    token_transfer(
        quote_actual_amount,
        &ctx.accounts.token_program,
        &ctx.accounts.user_quote_account,
        &ctx.accounts.market_quote_vault,
        &ctx.accounts.owner,
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
