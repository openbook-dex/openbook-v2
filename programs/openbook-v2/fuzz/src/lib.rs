pub mod account_allocator;
use account_allocator::*;
use fixed::types::I80F48;
use solana_program::{instruction::Instruction, system_program};

pub struct FuzzContext {}

impl FuzzContext {
    pub fn new() -> Self {
        let allocator = AccountAllocator::new();

        let payer = allocator.new_signer(1_000_000);
        let admin = allocator.new_signer(1_000_000);
        let mint = allocator.new_mint();
        let oracle = allocator.new_stub_oracle(mint.key);
        let system_program = allocator.new_program(system_program::ID);

        let instruction = {
            let accounts = openbook_v2::accounts::StubOracleCreate {
                admin: *admin.key,
                oracle: *oracle.key,
                payer: *payer.key,
                mint: *mint.key,
                system_program: *system_program.key,
            };
            let data = openbook_v2::instruction::StubOracleCreate { price: I80F48::ONE };
            make_instruction(&accounts, data)
        };

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
