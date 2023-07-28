use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CreateOpenOrdersIndexer<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub owner: Signer<'info>,
}
