use crate::state::market_seeds;
use anchor_lang::prelude::*;

use crate::accounts_ix::*;
use crate::logs::SweepFeesLog;
use crate::token_utils::*;

pub fn sweep_fees<'info>(ctx: Context<'_, '_, '_, 'info, SweepFees<'info>>) -> Result<()> {
    let mut market = ctx.accounts.market.load_mut()?;

    let amount = market.fees_available;
    market.fees_available = 0;
    market.quote_deposit_total -= amount;

    let seeds = market_seeds!(market, ctx.accounts.market.key());
    drop(market);

    // Getting actual base token amount to be deposited
    let token_fee_wrapped = {
        get_token_fee(ctx.accounts.mint.to_account_info(), ctx.accounts.token_program.to_account_info(), amount)
    };
    let token_fee = token_fee_wrapped.unwrap().unwrap();

    let actual_amount = amount - token_fee;

    token_transfer_signed(
        actual_amount,
        &ctx.accounts.token_program,
        &ctx.accounts.market_quote_vault,
        &ctx.accounts.token_receiver_account,
        &ctx.accounts.market_authority,
        seeds,
        ctx.accounts.mint.to_account_info(),
        ctx.accounts.mint.decimals,
    )?;

    emit!(SweepFeesLog {
        market: ctx.accounts.market.key(),
        amount,
        receiver: ctx.accounts.token_receiver_account.key(),
    });

    Ok(())
}
