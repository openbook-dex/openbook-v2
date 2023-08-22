use crate::accounts_ix::CancelOrder;
use crate::accounts_ix::PlaceOrder;
use crate::error::OpenBookError;
use crate::pubkey_option::NonZeroKey;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

#[derive(Accounts)]
pub struct CancelAndPlaceOrders<'info> {
    pub signer: Signer<'info>,
    #[account(
        mut,
        has_one = market,
        constraint = open_orders_account.load()?.is_owner_or_delegate(signer.key()) @ OpenBookError::NoOwnerOrDelegate
    )]
    pub open_orders_account: AccountLoader<'info, OpenOrdersAccount>,
    pub open_orders_admin: Option<Signer<'info>>,

    #[account(
        mut,
        token::mint = market_quote_vault.mint
    )]
    pub user_quote_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = market_base_vault.mint
    )]
    pub user_base_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        has_one = bids,
        has_one = asks,
        has_one = event_heap,
        has_one = market_base_vault,
        has_one = market_quote_vault,
        constraint = market.load()?.oracle_a == oracle_a.non_zero_key(),
        constraint = market.load()?.oracle_b == oracle_b.non_zero_key(),
        constraint = market.load()?.open_orders_admin == open_orders_admin.non_zero_key() @ OpenBookError::InvalidOpenOrdersAdmin
    )]
    pub market: AccountLoader<'info, Market>,
    #[account(mut)]
    pub bids: AccountLoader<'info, BookSide>,
    #[account(mut)]
    pub asks: AccountLoader<'info, BookSide>,
    #[account(mut)]
    pub event_heap: AccountLoader<'info, EventHeap>,

    #[account(mut)]
    pub market_quote_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub market_base_vault: Box<Account<'info, TokenAccount>>,

    /// CHECK: The oracle can be one of several different account types and the pubkey is checked above
    pub oracle_a: Option<UncheckedAccount<'info>>,
    /// CHECK: The oracle can be one of several different account types and the pubkey is checked above
    pub oracle_b: Option<UncheckedAccount<'info>>,

    pub token_program: Program<'info, Token>,
}

impl<'info> CancelAndPlaceOrders<'info> {
    pub fn to_cancel_order(&self) -> CancelOrder<'info> {
        CancelOrder {
            signer: self.signer.clone(),
            bids: self.bids.clone(),
            asks: self.asks.clone(),
            open_orders_account: self.open_orders_account.clone(),
            market: self.market.clone(),
        }
    }
    pub fn to_place_order(&self, side: Side) -> PlaceOrder<'info> {
        let (user_token_account, market_vault) = match side {
            Side::Bid => (
                self.user_quote_account.clone(),
                *self.market_quote_vault.clone(),
            ),
            Side::Ask => (
                self.user_base_account.clone(),
                *self.market_base_vault.clone(),
            ),
        };
        PlaceOrder {
            signer: self.signer.clone(),
            open_orders_account: self.open_orders_account.clone(),
            market: self.market.clone(),
            bids: self.bids.clone(),
            asks: self.asks.clone(),
            token_program: self.token_program.clone(),
            system_program: self.system_program.clone(),
            open_orders_admin: self.open_orders_admin.clone(),
            user_token_account,
            market_vault,
            event_heap: self.event_heap.clone(),
            oracle_a: self.oracle_a.clone(),
            oracle_b: self.oracle_b.clone(),
        }
    }
}
