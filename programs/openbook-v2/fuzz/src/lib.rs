pub mod accounts_state;
pub mod processor;

use accounts_state::*;
use fixed::types::I80F48;
use openbook_v2::state::OracleConfigParams;
use openbook_v2::state::*;
use processor::*;
use solana_program::{entrypoint::ProgramResult, pubkey::Pubkey, system_program};
use spl_associated_token_account::get_associated_token_address;

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

        let mut state = AccountsState::new();
        state
            .add_account_with_lamports(admin, 1_000_000)
            .add_account_with_lamports(payer, 1_000_000)
            .add_mint(base_mint)
            .add_mint(quote_mint)
            .add_openbook_account::<BookSide>(asks)
            .add_openbook_account::<BookSide>(bids)
            .add_openbook_account::<EventQueue>(event_queue)
            .add_openbook_account::<Market>(market)
            .add_openbook_account::<StubOracle>(oracle)
            .add_program(system_program::ID)
            .add_token_account(base_vault, market, base_mint)
            .add_token_account(quote_vault, market, quote_mint);

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
            state,
        }
    }

    pub fn stub_oracle_create(&mut self) -> ProgramResult {
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

    pub fn create_market(&mut self) -> ProgramResult {
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
}
