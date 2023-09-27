use crate::accounts_ix::Deposit;
use crate::error::*;
use crate::logs::DepositLog;
use crate::token_utils::*;
use anchor_lang::prelude::*;
// use anchor_spl::token_2022::{Token, TokenAccount, Mint};
use anchor_spl::token_interface::Mint;

// Try converting ther mint to TokenInterface??

pub fn deposit<'info>(ctx: Context<'_, '_, '_, 'info, Deposit<'info>>, base_amount: u64, quote_amount: u64) -> Result<()> {
    let mut open_orders_account = ctx.accounts.open_orders_account.load_mut()?;
    let mut market = ctx.accounts.market.load_mut()?;
    require!(
        !market.is_expired(Clock::get()?.unix_timestamp),
        OpenBookError::MarketHasExpired
    );

    let remaining_accounts = ctx.remaining_accounts;

    // Getting base transfer details
    let base_token_fee_wrapped = {
        get_token_fee(remaining_accounts[0].to_account_info(), ctx.accounts.base_token_program.to_account_info(), base_amount)
    };

    let base_token_fee = base_token_fee_wrapped.unwrap().unwrap();

    let base_actual_amount = base_amount + base_token_fee;

    let base_data = {
        &mut &**remaining_accounts[0].try_borrow_data()?
    };

    let base_mint = Mint::try_deserialize(base_data).unwrap();
    let base_decimals = base_mint.decimals;

    // Getting quote transfer details
    let quote_token_fee_wrapped = {
        get_token_fee(remaining_accounts[1].to_account_info(), ctx.accounts.quote_token_program.to_account_info(), quote_amount)
    };

    let quote_token_fee = quote_token_fee_wrapped.unwrap().unwrap();

    let quote_actual_amount = quote_amount + quote_token_fee;

    let quote_data = {
        &mut &**remaining_accounts[1].try_borrow_data()?
    };

    let quote_mint = Mint::try_deserialize(quote_data).unwrap();
    let quote_decimals = quote_mint.decimals;


    // Should open_orders_account and market be editted with base amount or actual amount excluding fees 
    token_transfer(
        base_actual_amount,
        &ctx.accounts.base_token_program,
        &ctx.accounts.user_base_account,
        &ctx.accounts.market_base_vault,
        &ctx.accounts.owner,
        remaining_accounts[0].to_account_info(),
        base_decimals,
    )?;
    open_orders_account.position.base_free_native += base_amount;
    market.base_deposit_total += base_amount;

    token_transfer(
        quote_actual_amount,
        &ctx.accounts.quote_token_program,
        &ctx.accounts.user_quote_account,
        &ctx.accounts.market_quote_vault,
        &ctx.accounts.owner,
        remaining_accounts[1].to_account_info(),
        quote_decimals,
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
