use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{TokenInterface, self};

#[derive(Accounts)]
pub struct Deposit<'info> {
    pub owner: Signer<'info>,
    #[account(
        mut,
        token::mint = market_base_vault.mint
    )]
    pub user_base_account: InterfaceAccount<'info, token_interface::TokenAccount>,
    #[account(
        mut,
        token::mint = market_quote_vault.mint
    )]
    pub user_quote_account: InterfaceAccount<'info, token_interface::TokenAccount>,
    #[account(
        mut,
        has_one = market,
    )]
    pub open_orders_account: AccountLoader<'info, OpenOrdersAccount>,

    #[account(
        mut,
        has_one = market_base_vault,
        has_one = market_quote_vault,
    )]
    pub market: AccountLoader<'info, Market>,
    #[account(mut)]
    pub market_base_vault: InterfaceAccount<'info, token_interface::TokenAccount>,
    #[account(mut)]
    pub market_quote_vault: InterfaceAccount<'info, token_interface::TokenAccount>,

    #[account(mut)]
    pub base_mint: Box<InterfaceAccount<'info, token_interface::Mint>>,
    #[account(mut)]
    pub quote_mint: Box<InterfaceAccount<'info, token_interface::Mint>>,

    pub base_token_program: Interface<'info, TokenInterface>,

    pub quote_token_program: Interface<'info, TokenInterface>,
}
