use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

#[derive(Accounts)]
#[instruction(market_index: MarketIndex)]
pub struct CreateMarket<'info> {
    pub admin: Signer<'info>,
    #[account(
        init,
        seeds = [b"Market".as_ref(), admin.key().as_ref(), &market_index.to_le_bytes()],
        bump,
        payer = payer,
        space = 8 + std::mem::size_of::<Market>(),
    )]
    pub market: AccountLoader<'info, Market>,

    /// Accounts are initialised by client,
    /// anchor discriminator is set first when ix exits,
    #[account(zero)]
    pub bids: AccountLoader<'info, BookSide>,
    #[account(zero)]
    pub asks: AccountLoader<'info, BookSide>,
    #[account(zero)]
    pub event_queue: AccountLoader<'info, EventQueue>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut, token::mint = base_mint, token::authority = market)]
    pub base_vault: Account<'info, TokenAccount>,
    #[account(mut, token::mint = quote_mint, token::authority = market)]
    pub quote_vault: Account<'info, TokenAccount>,

    pub base_mint: Account<'info, Mint>,
    pub quote_mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    /// CHECK: The oracle can be one of several different account types and the pubkey is checked above
    pub oracle: UncheckedAccount<'info>,
}
