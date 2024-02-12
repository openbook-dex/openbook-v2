use crate::accounts_ix::SettleFunds;
use crate::error::OpenBookError;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, TokenInterface};

#[derive(Accounts)]
pub struct SettleFundsExpired<'info> {
    pub close_market_admin: Signer<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub penalty_payer: Signer<'info>,
    #[account(
        mut,
        has_one = market,
    )]
    pub open_orders_account: AccountLoader<'info, OpenOrdersAccount>,
    #[account(
        mut,
        has_one = market_base_vault,
        has_one = market_quote_vault,
        has_one = market_authority,
        constraint = market.load()?.close_market_admin.is_some() @ OpenBookError::NoCloseMarketAdmin,
        constraint = market.load()?.close_market_admin == close_market_admin.key() @ OpenBookError::InvalidCloseMarketAdmin
    )]
    pub market: AccountLoader<'info, Market>,
    /// CHECK: checked on has_one in market
    pub market_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub market_base_vault: InterfaceAccount<'info, token_interface::TokenAccount>,
    #[account(mut)]
    pub market_quote_vault: InterfaceAccount<'info, token_interface::TokenAccount>,
    #[account(
        mut,
        token::mint = market_base_vault.mint,
        constraint = user_base_account.owner == open_orders_account.load()?.owner
    )]
    pub user_base_account: InterfaceAccount<'info, token_interface::TokenAccount>,
    #[account(
        mut,
        token::mint = market_quote_vault.mint,
        constraint = user_quote_account.owner == open_orders_account.load()?.owner
    )]
    pub user_quote_account: InterfaceAccount<'info, token_interface::TokenAccount>,
    #[account(
        mut,
        token::mint = market_quote_vault.mint
    )]
    pub referrer_account: Option<Box<InterfaceAccount<'info, token_interface::TokenAccount>>>,

    #[account(mut)]
    pub base_mint: Option<Box<InterfaceAccount<'info, token_interface::Mint>>>,
    #[account(mut)]
    pub quote_mint: Option<Box<InterfaceAccount<'info, token_interface::Mint>>>,

    pub base_token_program: Interface<'info, TokenInterface>,
    pub quote_token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> SettleFundsExpired<'info> {
    pub fn to_settle_funds_accounts(&self) -> SettleFunds<'info> {
        SettleFunds {
            owner: self.owner.clone(),
            penalty_payer: self.penalty_payer.clone(),
            open_orders_account: self.open_orders_account.clone(),
            market: self.market.clone(),
            market_authority: self.market_authority.clone(),
            market_base_vault: self.market_base_vault.clone(),
            market_quote_vault: self.market_quote_vault.clone(),
            user_base_account: self.user_base_account.clone(),
            user_quote_account: self.user_quote_account.clone(),
            referrer_account: self.referrer_account.clone(),
            base_mint: self.base_mint.clone(),
            quote_mint: self.quote_mint.clone(),
            base_token_program: self.base_token_program.clone(),
            quote_token_program: self.quote_token_program.clone(),
            system_program: self.system_program.clone(),
        }
    }
}
