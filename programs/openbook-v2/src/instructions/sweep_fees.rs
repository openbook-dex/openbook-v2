use crate::state::market_seeds;
use anchor_lang::prelude::*;

use crate::accounts_ix::*;
use crate::logs::SweepFeesLog;
use crate::token_utils::*;

pub fn sweep_fees(ctx: Context<SweepFees>) -> Result<()> {
    let mut market = ctx.accounts.market.load_mut()?;

    let fee_amount = market.fees_available;
    market.fees_available = 0;
    market.quote_deposit_total -= fee_amount;

    let seeds = market_seeds!(market, ctx.accounts.market.key());
    drop(market);

    token_transfer_signed(
        fee_amount,
        &ctx.accounts.token_program,
        &ctx.accounts.market_quote_vault,
        &ctx.accounts.token_receiver_account,
        &ctx.accounts.market_authority,
        seeds,
    )?;

    let rent = Rent::get()?;
    let rent_exempt_amount = rent.minimum_balance(ctx.accounts.market.to_account_info().data_len());
    let penalty_amount = ctx.accounts.market.to_account_info().lamports().saturating_sub(rent_exempt_amount);
    ctx.accounts.market.sub_lamports(penalty_amount)?;
    ctx.accounts.collect_fee_admin.add_lamports(penalty_amount)?;

    emit!(SweepFeesLog {
        market: ctx.accounts.market.key(),
        fee_amount,
        fee_receiver: ctx.accounts.token_receiver_account.key(),
        penalty_amount,
        penalty_receiver: ctx.accounts.collect_fee_admin.key(),
    });

    Ok(())
}
