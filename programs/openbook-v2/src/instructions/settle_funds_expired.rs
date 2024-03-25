use crate::accounts_ix::*;
use crate::error::OpenBookError;
use anchor_lang::prelude::*;

pub fn settle_funds_expired<'info>(
    ctx: Context<'_, '_, '_, 'info, SettleFundsExpired<'info>>,
) -> Result<()> {
    {
        let market = ctx.accounts.market.load()?;
        require!(
            market.is_expired(Clock::get()?.unix_timestamp),
            OpenBookError::MarketHasNotExpired
        );
    }

    crate::instructions::settle_funds(Context::new(
        ctx.program_id,
        &mut ctx.accounts.to_settle_funds(),
        ctx.remaining_accounts,
        ctx.bumps.to_settle_funds(),
    ))
}
