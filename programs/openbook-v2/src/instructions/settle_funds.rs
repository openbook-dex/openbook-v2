use anchor_lang::prelude::*;

use crate::accounts_ix::*;
use crate::logs::SettleFundsLog;
use crate::state::*;
use crate::token_utils::*;

pub fn settle_funds<'info>(ctx: Context<'_, '_, '_, 'info, SettleFunds<'info>>) -> Result<()> {
    let mut open_orders_account = ctx.accounts.open_orders_account.load_mut()?;
    let mut market = ctx.accounts.market.load_mut()?;

    let mut roundoff_maker_fees = 0;

    if market.maker_fee.is_positive() && open_orders_account.position.bids_base_lots == 0 {
        roundoff_maker_fees = open_orders_account.position.locked_maker_fees;
        open_orders_account.position.locked_maker_fees = 0;
    }

    let pa = &mut open_orders_account.position;
    let referrer_rebate = pa.referrer_rebates_available + roundoff_maker_fees;

    if ctx.accounts.referrer_account.is_some() {
        market.fees_to_referrers += referrer_rebate as u128;
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
            &ctx.accounts.penalty_payer,
            &ctx.accounts.market,
        )?;
        pa.penalty_heap_count = 0;
    }

    let base_mint_acc: Option<AccountInfo<'_>>;
    let quote_mint_acc: Option<AccountInfo<'_>>;

    let base_decimals: Option<u8>;
    let quote_decimals: Option<u8>;

    if let Some(base_mint) = &ctx.accounts.base_mint {
        base_mint_acc = Some(base_mint.to_account_info());

        base_decimals = Some(base_mint.decimals);
    } else {
        base_mint_acc = None;

        base_decimals = None;
    }

    if let Some(quote_mint) = &ctx.accounts.quote_mint {
        quote_mint_acc = Some(quote_mint.to_account_info());

        quote_decimals = Some(quote_mint.decimals)
    } else {
        quote_mint_acc = None;

        quote_decimals = None;
    }

    if let Some(referrer_account) = &ctx.accounts.referrer_account {
        token_transfer_signed(
            referrer_rebate,
            &ctx.accounts.quote_token_program,
            &ctx.accounts.market_quote_vault,
            referrer_account,
            &ctx.accounts.market_authority,
            seeds,
            &quote_mint_acc,
            quote_decimals,
        )?;
    }

    token_transfer_signed(
        pa.base_free_native,
        &ctx.accounts.base_token_program,
        &ctx.accounts.market_base_vault,
        &ctx.accounts.user_base_account,
        &ctx.accounts.market_authority,
        seeds,
        &base_mint_acc,
        base_decimals,
    )?;

    token_transfer_signed(
        pa.quote_free_native,
        &ctx.accounts.quote_token_program,
        &ctx.accounts.market_quote_vault,
        &ctx.accounts.user_quote_account,
        &ctx.accounts.market_authority,
        seeds,
        &quote_mint_acc,
        quote_decimals,
    )?;

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
