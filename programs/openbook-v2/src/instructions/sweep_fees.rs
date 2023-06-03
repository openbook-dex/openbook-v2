use crate::state::market_seeds;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

use crate::accounts_ix::*;

pub fn sweep_fees(ctx: Context<SweepFees>) -> Result<()> {
    let mut market = ctx.accounts.market.load_mut()?;

    // get/update all values from market and drop reference to it before cpi
    let amount = market.quote_fees_accrued;
    market.quote_fees_accrued = 0;

    let seeds = market_seeds!(market);
    let signer = &[&seeds[..]];

    drop(market);

    let cpi_context = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.quote_vault.to_account_info(),
            to: ctx.accounts.token_receiver_account.to_account_info(),
            authority: ctx.accounts.market.to_account_info(),
        },
    );
    token::transfer(cpi_context.with_signer(signer), amount)?;

    Ok(())
}
