use crate::error::OpenBookError;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

use crate::accounts_ix::*;

pub fn sweep_fees(ctx: Context<SweepFees>) -> Result<()> {
    let (market_index, market_bump, amount) = {
        let market = &mut ctx.accounts.market.load_mut()?;
        // Enforce only admin can withdraw fees
        require!(
            market.admin == ctx.accounts.receiver.owner,
            OpenBookError::InvalidFundsReceiver
        );

        let amount = market.quote_fees_accrued;
        market.quote_fees_accrued = 0;
        (market.market_index, market.bump, amount)
    };

    let seeds = [
        b"Market".as_ref(),
        &market_index.to_le_bytes(),
        &[market_bump],
    ];
    let signer = &[&seeds[..]];

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
