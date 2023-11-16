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

    let mint_acc: Option<AccountInfo<'_>>;

    let mint_decimals: Option<u8>;

    if let Some(mint) = &ctx.accounts.mint {
        mint_acc = Some(mint.to_account_info());

        mint_decimals = Some(mint.decimals);
    } else {
        mint_acc = None;

        mint_decimals = None;
    }


    token_transfer_signed(
        amount,
        &ctx.accounts.token_program,
        &ctx.accounts.market_quote_vault,
        &ctx.accounts.token_receiver_account,
        &ctx.accounts.market_authority,
        seeds,
        &mint_acc,
        mint_decimals,
    )?;

    emit!(SweepFeesLog {
        market: ctx.accounts.market.key(),
        amount,
        receiver: ctx.accounts.token_receiver_account.key(),
    });

    Ok(())
}
