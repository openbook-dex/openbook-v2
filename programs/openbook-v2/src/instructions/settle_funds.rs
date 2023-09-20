use anchor_lang::prelude::*;

use crate::accounts_ix::*;
use crate::logs::SettleFundsLog;
use crate::state::*;
use crate::token_utils::*;
use anchor_spl::token_interface::{TokenInterface, self, Mint, TokenAccount};

pub fn settle_funds<'info>(ctx: Context<'_, '_, '_, 'info, SettleFunds<'info>>) -> Result<()> {
    let mut open_orders_account = ctx.accounts.open_orders_account.load_mut()?;
    let mut market = ctx.accounts.market.load_mut()?;

    let remaining_accounts = ctx.remaining_accounts;

    let mut roundoff_maker_fees = 0;

    if market.maker_fee.is_positive() && open_orders_account.position.bids_base_lots == 0 {
        roundoff_maker_fees = open_orders_account.position.locked_maker_fees;
        open_orders_account.position.locked_maker_fees = 0;
    }

    let pa = &mut open_orders_account.position;
    let referrer_rebate = pa.referrer_rebates_available + roundoff_maker_fees;

    if ctx.accounts.referrer_account.is_some() {
        market.fees_to_referrers += referrer_rebate;
        market.quote_deposit_total -= referrer_rebate;
    } else {
        market.fees_available += referrer_rebate;
    }

    market.base_deposit_total -= pa.base_free_native;
    market.quote_deposit_total -= pa.quote_free_native;
    market.referrer_rebates_accrued -= pa.referrer_rebates_available;

    let seeds = market_seeds!(market, ctx.accounts.market.key());

    drop(market);

    if pa.penalty_heap_count > 0 {
        system_program_transfer(
            pa.penalty_heap_count * PENALTY_EVENT_HEAP,
            &ctx.accounts.system_program,
            &ctx.accounts.owner,
            &ctx.accounts.market,
        )?;
        pa.penalty_heap_count = 0;
    }

    if let Some(referrer_account) = &ctx.accounts.referrer_account {

        // Getting actual referrer amount to be paid
        // let referrer_token_account_info = remaining_accounts[1];

        let referrer_amount_wrapped = {
            get_token_fee(remaining_accounts[1].to_account_info(), ctx.accounts.quote_token_program.to_account_info(), referrer_rebate)
        };
        let referrer_amount_fee = referrer_amount_wrapped.unwrap().unwrap();

        let referrer_amount = referrer_rebate - referrer_amount_fee;

        let referrer_data = &mut &**remaining_accounts[1].try_borrow_data()?;
        let referrer_mint = Mint::try_deserialize(referrer_data).unwrap();
        let referrer_decimals = referrer_mint.decimals;

        token_transfer_signed(
            referrer_amount,
            &ctx.accounts.quote_token_program,
            &ctx.accounts.market_quote_vault,
            referrer_account,
            &ctx.accounts.market_authority,
            seeds,
            remaining_accounts[1].to_account_info(),
            referrer_decimals,
        )?;
    }


    // Getting actual base amount to be paid
    // let base_token_account_info = remaining_accounts[0];

    let base_token_fee_wrapped = {
        get_token_fee(remaining_accounts[0].to_account_info(), ctx.accounts.base_token_program.to_account_info(), pa.base_free_native)
    };
    let base_token_fee = base_token_fee_wrapped.unwrap().unwrap();

    let base_amount = pa.base_free_native - base_token_fee;

    let base_data = &mut &**remaining_accounts[0].try_borrow_data()?;
    let base_mint = Mint::try_deserialize(base_data).unwrap();
    let base_decimals = base_mint.decimals;

    // Getting actual quote native amount to be paid
    // let quote_token_account_info = remaining_accounts[1];

    let quote_token_fee_wrapped = {
        get_token_fee(remaining_accounts[1].to_account_info(), ctx.accounts.quote_token_program.to_account_info(), pa.quote_free_native)
    };
    let quote_token_fee = quote_token_fee_wrapped.unwrap().unwrap();

    let quote_amount = pa.quote_free_native - quote_token_fee;

    let quote_data = &mut &**remaining_accounts[1].try_borrow_data()?;
    let quote_mint = Mint::try_deserialize(quote_data).unwrap();
    let quote_decimals = quote_mint.decimals;

    token_transfer_signed(
        base_amount,
        &ctx.accounts.base_token_program,
        &ctx.accounts.market_base_vault,
        &ctx.accounts.user_base_account,
        &ctx.accounts.market_authority,
        seeds,
        remaining_accounts[0].to_account_info(),
        base_decimals
    )?;

    token_transfer_signed(
        quote_amount,
        &ctx.accounts.quote_token_program,
        &ctx.accounts.market_quote_vault,
        &ctx.accounts.user_quote_account,
        &ctx.accounts.market_authority,
        seeds,
        remaining_accounts[1].to_account_info(),
        quote_decimals,
    )?;

    // Should settle funds have total amount, or the actual amount paid out excluding fees ??
    emit!(SettleFundsLog {
        open_orders_account: ctx.accounts.open_orders_account.key(),
        base_native: pa.base_free_native,
        quote_native: pa.quote_free_native,
        referrer_rebate,
        referrer: ctx.accounts.referrer_account.as_ref().map(|acc| acc.key())
    });

    pa.base_free_native = 0;
    pa.quote_free_native = 0;
    pa.referrer_rebates_available = 0;

    Ok(())
}
