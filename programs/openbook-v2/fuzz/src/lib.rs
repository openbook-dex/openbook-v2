pub mod accounts_state;
pub mod processor;

use accounts_state::*;
use fixed::types::I80F48;
use openbook_v2::state::OracleConfigParams;
use openbook_v2::state::*;
use processor::*;
use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey, system_program};
use spl_associated_token_account::get_associated_token_address;

pub const MAX_NUM_USERS: usize = 8;

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
    users: Vec<UserAccounts>,
    state: AccountsState,
}

impl FuzzContext {
    pub fn new(n_users: u8) -> Self {
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

        let users: Vec<UserAccounts> = (0..n_users)
            .map(|_| {
                let account_num = 0_u32;
                let owner = Pubkey::new_unique();
                let open_orders = Pubkey::find_program_address(
                    &[
                        b"OpenOrders".as_ref(),
                        owner.as_ref(),
                        market.as_ref(),
                        &account_num.to_le_bytes(),
                    ],
                    &openbook_v2::ID,
                )
                .0;

                UserAccounts {
                    owner,
                    open_orders,
                    base_vault: Pubkey::new_unique(),
                    quote_vault: Pubkey::new_unique(),
                }
            })
            .collect();

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
            users,
            state: AccountsState::new(),
        }
    }

    pub fn initialize(&mut self) {
        self.state
            .add_account_with_lamports(self.admin, 1_000_000)
            .add_account_with_lamports(self.payer, 1_000_000)
            .add_mint(self.base_mint)
            .add_mint(self.quote_mint)
            .add_openbook_account::<BookSide>(self.asks)
            .add_openbook_account::<BookSide>(self.bids)
            .add_openbook_account::<EventQueue>(self.event_queue)
            .add_openbook_account::<Market>(self.market)
            .add_openbook_account::<StubOracle>(self.oracle)
            .add_program(system_program::ID)
            .add_token_account_with_lamports(self.base_vault, self.market, self.base_mint, 0)
            .add_token_account_with_lamports(self.quote_vault, self.market, self.quote_mint, 0);

        self.users.iter().for_each(|u| {
            self.state
                .add_account_with_lamports(u.owner, 1_000_000)
                .add_account_with_lamports(u.owner, 1_000_000)
                .add_token_account_with_lamports(u.base_vault, u.owner, self.base_mint, 1_000_000)
                .add_token_account_with_lamports(u.quote_vault, u.owner, self.quote_mint, 1_000_000)
                .add_open_orders_account(u.open_orders, 8);
        });

        self.stub_oracle_create().unwrap();
        self.create_market().unwrap();
        self.init_open_orders().unwrap();
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
            collect_fee_admin: Pubkey::new_unique(),
            open_orders_admin: None,
            consume_events_admin: None,
            close_market_admin: None,
        };
        process_instruction(&mut self.state, &accounts, &data)
    }

    fn init_open_orders(&mut self) -> ProgramResult {
        self.users.iter().try_for_each(|user| {
            let accounts = openbook_v2::accounts::InitOpenOrders {
                open_orders_account: user.open_orders,
                owner: user.owner,
                payer: self.payer,
                market: self.market,
                system_program: system_program::ID,
            };
            let data = openbook_v2::instruction::InitOpenOrders {
                account_num: 0,
                open_orders_count: 8,
            };
            process_instruction(&mut self.state, &accounts, &data)
        })
    }
}
