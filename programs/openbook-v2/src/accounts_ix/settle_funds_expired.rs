use crate::accounts_ix::SettleFunds;
use crate::error::OpenBookError;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

#[derive(Accounts)]
pub struct SettleFundsExpired<'info> {
    pub close_market_admin: Signer<'info>,
    #[account(
        mut,
        has_one = market,
    )]
    pub open_orders_account: AccountLoader<'info, OpenOrdersAccount>,
    #[account(
        mut,
        has_one = base_vault,
        has_one = quote_vault,
        has_one = market_authority,
        constraint = market.load()?.close_market_admin.is_some() @ OpenBookError::NoCloseMarketAdmin,
        constraint = market.load()?.close_market_admin == close_market_admin.key() @ OpenBookError::InvalidOpenOrdersAdmin
    )]
    pub market: AccountLoader<'info, Market>,
    /// CHECK: checked on has_one in market
    pub market_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub base_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub quote_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = base_vault.mint,
        constraint = token_base_account.owner == open_orders_account.load()?.owner
    )]
    pub token_base_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = quote_vault.mint,
        constraint = token_base_account.owner == open_orders_account.load()?.owner
    )]
    pub token_quote_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::mint = quote_vault.mint
    )]
    pub referrer: Option<Box<Account<'info, TokenAccount>>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> SettleFundsExpired<'info> {
    pub fn to_settle_funds_accounts(&self) -> SettleFunds<'info> {
        SettleFunds {
            owner: self.close_market_admin.clone(),
            open_orders_account: self.open_orders_account.clone(),
            market: self.market.clone(),
            market_authority: self.market_authority.clone(),
            base_vault: self.base_vault.clone(),
            quote_vault: self.quote_vault.clone(),
            token_base_account: self.token_base_account.clone(),
            token_quote_account: self.token_quote_account.clone(),
            referrer: self.referrer.clone(),
            token_program: self.token_program.clone(),
            system_program: self.system_program.clone(),
        }
    }
}
