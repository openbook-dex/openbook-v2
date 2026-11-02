use anchor_lang::prelude::*;

use crate::accounts_ix::*;
use crate::accounts_zerocopy::LoadMutZeroCopyRef;
use crate::logs::emit_stack;
use crate::logs::SettleFundsLog;
use crate::state::*;
use crate::token_utils::*;

pub fn settle_funds<'info>(ctx: Context<'_, '_, '_, 'info, SettleFunds<'info>>) -> Result<()> {
    let mut open_orders_account = ctx.accounts.open_orders_account.load_mut()?;
    let referrer_account = ctx
        .accounts
        .referrer_account
        .as_ref()
        .map(|acc| acc.to_account_info());

    let (base_native, quote_native, referrer_rebate) = settle_funds_inner(
        &mut open_orders_account,
        &ctx.accounts.market.to_account_info(),
        &ctx.accounts.market_authority,
        &ctx.accounts.market_base_vault.to_account_info(),
        &ctx.accounts.market_quote_vault.to_account_info(),
        &ctx.accounts.user_base_account.to_account_info(),
        &ctx.accounts.user_quote_account.to_account_info(),
        referrer_account.as_ref(),
        &ctx.accounts.penalty_payer,
        &ctx.accounts.token_program,
        &ctx.accounts.system_program,
    )?;

    emit_stack(SettleFundsLog {
        open_orders_account: ctx.accounts.open_orders_account.key(),
        base_native,
        quote_native,
        referrer_rebate,
        referrer: ctx.accounts.referrer_account.as_ref().map(|acc| acc.key()),
    });

    Ok(())
}

/// Move the free balances of `open_orders_account` to the user token accounts, shared by
/// `settle_funds` and `janitor_close_open_orders_account`. Without a referrer account the
/// referrer rebate goes to the market's `fees_available`.
///
/// Returns the settled `(base_native, quote_native, referrer_rebate)`.
#[allow(clippy::too_many_arguments)]
pub fn settle_funds_inner<'info>(
    open_orders_account: &mut OpenOrdersAccount,
    market_info: &AccountInfo<'info>,
    market_authority: &AccountInfo<'info>,
    market_base_vault: &AccountInfo<'info>,
    market_quote_vault: &AccountInfo<'info>,
    user_base_account: &AccountInfo<'info>,
    user_quote_account: &AccountInfo<'info>,
    referrer_account: Option<&AccountInfo<'info>>,
    penalty_payer: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
) -> Result<(u64, u64, u64)> {
    let mut market = market_info.load_mut::<Market>()?;

    let mut roundoff_maker_fees = 0;

    if market.maker_fee.is_positive() && open_orders_account.position.bids_base_lots == 0 {
        roundoff_maker_fees = open_orders_account.position.locked_maker_fees;
        open_orders_account.position.locked_maker_fees = 0;
    }

    let pa = &mut open_orders_account.position;
    let referrer_rebate = pa.referrer_rebates_available + roundoff_maker_fees;

    if referrer_account.is_some() {
        market.fees_to_referrers += referrer_rebate as u128;
        market.quote_deposit_total -= referrer_rebate;
    } else {
        market.fees_available += referrer_rebate;
    }

    market.base_deposit_total -= pa.base_free_native;
    market.quote_deposit_total -= pa.quote_free_native;
    market.referrer_rebates_accrued -= pa.referrer_rebates_available;

    let seeds = market_seeds!(market, market_info.key());

    drop(market);

    if pa.penalty_heap_count > 0 {
        system_program_transfer(
            pa.penalty_heap_count * PENALTY_EVENT_HEAP,
            system_program,
            penalty_payer,
            market_info,
        )?;
        pa.penalty_heap_count = 0;
    }

    if let Some(referrer_account) = referrer_account {
        token_transfer_signed(
            referrer_rebate,
            token_program,
            market_quote_vault,
            referrer_account,
            market_authority,
            seeds,
        )?;
    }

    token_transfer_signed(
        pa.base_free_native,
        token_program,
        market_base_vault,
        user_base_account,
        market_authority,
        seeds,
    )?;

    token_transfer_signed(
        pa.quote_free_native,
        token_program,
        market_quote_vault,
        user_quote_account,
        market_authority,
        seeds,
    )?;

    let settled = (pa.base_free_native, pa.quote_free_native, referrer_rebate);

    pa.base_free_native = 0;
    pa.quote_free_native = 0;
    pa.referrer_rebates_available = 0;

    Ok(settled)
}
