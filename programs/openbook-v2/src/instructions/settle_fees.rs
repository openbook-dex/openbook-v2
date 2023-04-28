use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};
use fixed::types::I80F48;

use crate::accounts_ix::*;

pub fn sweep_fees(ctx: Context<SettleFees>) -> Result<()> {
    let mut market = ctx.accounts.market.load_mut()?;

    let fees_amount = market.fees_accrued;

    let seeds = [
        b"Market".as_ref(),
        &market.market_index.to_le_bytes(),
        &[market.bump],
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
    token::transfer(cpi_context.with_signer(signer), fees_amount.to_num())?;

    market.fees_settled += fees_amount;
    market.fees_accrued = I80F48::ZERO;

    Ok(())
}
