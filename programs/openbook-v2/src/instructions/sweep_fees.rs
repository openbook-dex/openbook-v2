use crate::error::OpenBookError;
use crate::state::market_seeds;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

use crate::accounts_ix::*;

pub fn sweep_fees(ctx: Context<SweepFees>) -> Result<()> {
    let mut market = ctx.accounts.market.load_mut()?;
    // Enforce only admin can withdraw fees
    require!(
        market.admin == ctx.accounts.receiver.owner,
        OpenBookError::InvalidFundsReceiver
    );

    let amount = market.quote_fees_accrued;
    market.quote_fees_accrued = 0;

    let seeds = market_seeds!(market);
    let signer = &[&seeds[..]];

    drop(market);

    let cpi_context = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.quote_vault.to_account_info(),
            to: ctx.accounts.receiver.to_account_info(),
            authority: ctx.accounts.market.to_account_info(),
        },
    );
    token::transfer(cpi_context.with_signer(signer), amount)?;

    Ok(())
}
