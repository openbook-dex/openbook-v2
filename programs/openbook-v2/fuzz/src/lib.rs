pub mod accounts_state;
pub mod processor;

use accounts_state::*;
use arbitrary::{Arbitrary, Unstructured};
use fixed::types::I80F48;
use openbook_v2::{error::OpenBookError, state::*};
use processor::*;
use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey, system_program};
use spl_associated_token_account::get_associated_token_address;
use std::collections::HashMap;

pub const NUM_USERS: u8 = 8;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct UserId(u8);

impl Arbitrary<'_> for UserId {
    fn arbitrary(u: &mut Unstructured<'_>) -> arbitrary::Result<Self> {
        let i: u8 = u.arbitrary()?;
        Ok(Self(i % NUM_USERS))
    }

    fn size_hint(_: usize) -> (usize, Option<usize>) {
        (1, Some(1))
    }
}

pub struct FuzzContext {
    payer: Pubkey,
    admin: Pubkey,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    market: Pubkey,
    oracle: Pubkey,
    bids: Pubkey,
    asks: Pubkey,
    event_queue: Pubkey,
    base_vault: Pubkey,
    quote_vault: Pubkey,
    collect_fee_admin: Pubkey,
    users: HashMap<UserId, UserAccounts>,
    state: AccountsState,
}

impl FuzzContext {
    pub fn new() -> Self {
        let payer = Pubkey::new_unique();
        let admin = Pubkey::new_unique();
        let base_mint = Pubkey::new_unique();
        let quote_mint = Pubkey::new_unique();

        let market_index: MarketIndex = 0;
        let market = Pubkey::find_program_address(
            &[b"Market".as_ref(), market_index.to_le_bytes().as_ref()],
            &openbook_v2::ID,
        )
        .0;

        let oracle = Pubkey::find_program_address(
            &[b"StubOracle".as_ref(), base_mint.as_ref()],
            &openbook_v2::ID,
        )
        .0;

        let bids = Pubkey::new_unique();
        let asks = Pubkey::new_unique();
        let event_queue = Pubkey::new_unique();

        let base_vault = get_associated_token_address(&market, &base_mint);
        let quote_vault = get_associated_token_address(&market, &quote_mint);

        let collect_fee_admin = Pubkey::new_unique();

        Self {
            payer,
            admin,
            base_mint,
            quote_mint,
            market,
            oracle,
            bids,
            asks,
            event_queue,
            base_vault,
            quote_vault,
            collect_fee_admin,
            users: HashMap::new(),
            state: AccountsState::new(),
        }
    }

    pub fn initialize(&mut self) -> &mut Self {
        self.state
            .add_account_with_lamports(self.admin, 1_000_000)
            .add_account_with_lamports(self.collect_fee_admin, 0)
            .add_account_with_lamports(self.payer, 1_000_000)
            .add_mint(self.base_mint)
            .add_mint(self.quote_mint)
            .add_openbook_account::<BookSide>(self.asks)
            .add_openbook_account::<BookSide>(self.bids)
            .add_openbook_account::<EventQueue>(self.event_queue)
            .add_openbook_account::<Market>(self.market)
            .add_openbook_account::<StubOracle>(self.oracle)
            .add_program(openbook_v2::ID) // optional accounts use this pubkey
            .add_program(spl_token::ID)
            .add_program(system_program::ID)
            .add_token_account_with_lamports(self.base_vault, self.market, self.base_mint, 0)
            .add_token_account_with_lamports(self.quote_vault, self.market, self.quote_mint, 0);

        self.stub_oracle_create().unwrap();
        self.create_market().unwrap();

        self
    }

    fn user(&mut self, user_id: UserId) -> &UserAccounts {
        let create_new_user = || -> UserAccounts {
            let account_num = 0_u32;

            let owner = Pubkey::new_unique();
            let base_vault = Pubkey::new_unique();
            let quote_vault = Pubkey::new_unique();
            let open_orders = Pubkey::find_program_address(
                &[
                    b"OpenOrders".as_ref(),
                    owner.as_ref(),
                    self.market.as_ref(),
                    &account_num.to_le_bytes(),
                ],
                &openbook_v2::ID,
            )
            .0;

            self.state
                .add_account_with_lamports(owner, 1_000_000)
                .add_account_with_lamports(owner, 1_000_000)
                .add_token_account_with_lamports(base_vault, owner, self.base_mint, 1_000_000)
                .add_token_account_with_lamports(quote_vault, owner, self.quote_mint, 1_000_000)
                .add_open_orders_account(open_orders, 8);

            let accounts = openbook_v2::accounts::InitOpenOrders {
                open_orders_account: open_orders,
                owner,
                payer: self.payer,
                market: self.market,
                system_program: system_program::ID,
            };
            let data = openbook_v2::instruction::InitOpenOrders {
                account_num,
                open_orders_count: 8,
            };
            process_instruction(&mut self.state, &accounts, &data).unwrap();

            UserAccounts {
                owner,
                open_orders,
                base_vault,
                quote_vault,
            }
        };

        self.users.entry(user_id).or_insert_with(create_new_user)
    }

    fn stub_oracle_create(&mut self) -> ProgramResult {
        let accounts = openbook_v2::accounts::StubOracleCreate {
            oracle: self.oracle,
            admin: self.admin,
            mint: self.base_mint,
            payer: self.payer,
            system_program: system_program::ID,
        };
        let data = openbook_v2::instruction::StubOracleCreate { price: I80F48::ONE };
        process_instruction(&mut self.state, &accounts, &data)
    }

    fn create_market(&mut self) -> ProgramResult {
        let accounts = openbook_v2::accounts::CreateMarket {
            market: self.market,
            bids: self.bids,
            asks: self.asks,
            event_queue: self.event_queue,
            payer: self.payer,
            base_vault: self.base_vault,
            quote_vault: self.quote_vault,
            base_mint: self.base_mint,
            quote_mint: self.quote_mint,
            oracle: self.oracle,
            system_program: system_program::ID,
            collect_fee_admin: self.collect_fee_admin,
            open_orders_admin: None,
            consume_events_admin: None,
            close_market_admin: None,
        };
        let data = openbook_v2::instruction::CreateMarket {
            market_index: 0,
            name: "fuzz_market".to_string(),
            oracle_config: OracleConfigParams {
                conf_filter: 0.1,
                max_staleness_slots: None,
            },
            quote_lot_size: 10,
            base_lot_size: 100,
            maker_fee: -0.0002,
            taker_fee: 0.0004,
            fee_penalty: 0,
        };
        process_instruction(&mut self.state, &accounts, &data)
    }

    pub fn place_order(
        &mut self,
        user_id: UserId,
        data: openbook_v2::instruction::PlaceOrder,
    ) -> ProgramResult {
        let user = self.user(user_id);

        let token_deposit_account = match data.side {
            Side::Ask => user.base_vault,
            Side::Bid => user.quote_vault,
        };

        let accounts = openbook_v2::accounts::PlaceOrder {
            open_orders_account: user.open_orders,
            owner: user.owner,
            open_orders_admin: None,
            market: self.market,
            bids: self.bids,
            asks: self.asks,
            token_deposit_account,
            base_vault: self.base_vault,
            quote_vault: self.quote_vault,
            event_queue: self.event_queue,
            oracle: self.oracle,
            token_program: spl_token::ID,
            system_program: system_program::ID,
        };

        process_instruction(&mut self.state, &accounts, &data).or_else(|err| match err {
            e if e == OpenBookError::InvalidOrderSize.into() => Ok(()),
            e if e == OpenBookError::InvalidPriceLots.into() => Ok(()),
            e if e == OpenBookError::NegativeLots.into() => Ok(()),
            e if e == OpenBookError::WouldSelfTrade.into() => Ok(()),
            e if e == spl_token::error::TokenError::InsufficientFunds.into() => Ok(()),
            _ => Err(err),
        })
    }
}