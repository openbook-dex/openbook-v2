use crate::state::market_seeds;
use anchor_lang::prelude::*;

use crate::accounts_ix::*;
use crate::logs::SweepFeesLog;
use crate::token_utils::*;

pub fn sweep_fees(ctx: Context<SweepFees>) -> Result<()> {
    let mut market = ctx.accounts.market.load_mut()?;

    let amount = market.quote_fees_available;
    market.quote_fees_available = 0;
    market.quote_deposit_total -= amount;

    let seeds = market_seeds!(market);
    drop(market);

    token_transfer_signed(
        amount,
        &ctx.accounts.token_program,
        &ctx.accounts.quote_vault,
        &ctx.accounts.token_receiver_account,
        &ctx.accounts.market,
        seeds,
    )?;

    emit!(SweepFeesLog {
        market: ctx.accounts.market.key(),
        amount,
        receiver: ctx.accounts.token_receiver_account.key(),
    });

    Ok(())
}
