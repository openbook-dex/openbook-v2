pub mod account_allocator;
pub mod processor;

use account_allocator::*;
use fixed::types::I80F48;
use openbook_v2::state::OracleConfigParams;
use processor::*;
use solana_program::{instruction::Instruction, pubkey::Pubkey, system_program};

pub struct FuzzContext {}

impl FuzzContext {
    pub fn new() -> Self {
        let allocator = AccountAllocator::new();

        let payer = allocator.new_signer(1_000_000);
        let admin = allocator.new_signer(1_000_000);

        let system_program = allocator.new_program(system_program::ID);

        let market = allocator.new_market();
        let base_mint = allocator.new_mint();
        let quote_mint = allocator.new_mint();

        let oracle = allocator.new_stub_oracle(base_mint.key);

        let bids = allocator.new_bookside();
        let asks = allocator.new_bookside();
        let event_queue = allocator.new_event_queue();

        let base_vault = allocator.new_ata(market.key, base_mint.key);
        let quote_vault = allocator.new_ata(market.key, quote_mint.key);

        let instruction = {
            let accounts = openbook_v2::accounts::StubOracleCreate {
                oracle: *oracle.key,
                admin: *admin.key,
                mint: *base_mint.key,
                payer: *payer.key,
                system_program: *system_program.key,
            };
            let data = openbook_v2::instruction::StubOracleCreate { price: I80F48::ONE };
            make_instruction(&accounts, data)
        };
        let accounts = vec![
            oracle.clone(),
            admin.clone(),
            base_mint.clone(),
            payer.clone(),
            system_program.clone(),
        ];
        do_process_instruction(instruction, &accounts).unwrap();

        let instruction = {
            let accounts = openbook_v2::accounts::CreateMarket {
                market: *market.key,
                bids: *bids.key,
                asks: *asks.key,
                event_queue: *event_queue.key,
                payer: *payer.key,
                base_vault: *base_vault.key,
                quote_vault: *quote_vault.key,
                base_mint: *base_mint.key,
                quote_mint: *quote_mint.key,
                system_program: *system_program.key,
                oracle: *oracle.key,
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
            make_instruction(&accounts, data)
        };
        let accounts = vec![
            market.clone(),
            bids.clone(),
            asks.clone(),
            event_queue.clone(),
            payer.clone(),
            base_vault.clone(),
            quote_vault.clone(),
            base_mint.clone(),
            quote_mint.clone(),
            system_program.clone(),
            oracle.clone(),
        ];
        do_process_instruction(instruction, &accounts).unwrap();

        Self {}
    }
}

fn make_instruction(
    accounts: &impl anchor_lang::ToAccountMetas,
    data: impl anchor_lang::InstructionData,
) -> Instruction {
    Instruction {
        program_id: openbook_v2::ID,
        accounts: anchor_lang::ToAccountMetas::to_account_metas(accounts, None),
        data: anchor_lang::InstructionData::data(&data),
    }
}
