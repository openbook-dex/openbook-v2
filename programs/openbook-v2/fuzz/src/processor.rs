use crate::accounts_state::AccountsState;
use bumpalo::Bump;
use log::debug;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, instruction::Instruction,
    program_error::ProgramError, program_stubs, pubkey::Pubkey, rent::Rent, system_program,
};

pub struct TestSyscallStubs {}
impl program_stubs::SyscallStubs for TestSyscallStubs {
    fn sol_log(&self, message: &str) {
        debug!("Program log: {}", message);
    }

    fn sol_invoke_signed(
        &self,
        instruction: &Instruction,
        account_infos: &[AccountInfo],
        signers_seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        let mut new_account_infos = vec![];

        for meta in instruction.accounts.iter() {
            for account_info in account_infos.iter() {
                if meta.pubkey == *account_info.key {
                    let mut new_account_info = account_info.clone();
                    for seeds in signers_seeds.iter() {
                        let signer =
                            Pubkey::create_program_address(seeds, &openbook_v2::id()).unwrap();
                        if *account_info.key == signer {
                            new_account_info.is_signer = true;
                        }
                    }
                    new_account_infos.push(new_account_info);
                }
            }
        }

        match instruction.program_id {
            // accounts should already be created
            id if id == system_program::ID => Ok(()),
            id if id == spl_token::ID => spl_token::processor::Processor::process(
                &instruction.program_id,
                &new_account_infos,
                &instruction.data,
            ),
            _ => Err(ProgramError::IncorrectProgramId),
        }
    }

    fn sol_get_clock_sysvar(&self, var_addr: *mut u8) -> u64 {
        unsafe {
            *(var_addr as *mut _ as *mut Clock) = Clock::default();
        }
        solana_program::entrypoint::SUCCESS
    }

    fn sol_get_rent_sysvar(&self, var_addr: *mut u8) -> u64 {
        unsafe {
            *(var_addr as *mut _ as *mut Rent) = Rent::default();
        }
        solana_program::entrypoint::SUCCESS
    }
}

pub fn process_instruction(
    state: &mut AccountsState,
    accounts: &impl anchor_lang::ToAccountMetas,
    data: &impl anchor_lang::InstructionData,
) -> ProgramResult {
    let bump = Bump::new();
    let metas = anchor_lang::ToAccountMetas::to_account_metas(accounts, None);
    let account_infos = state.account_infos(&bump, metas);

    let res = openbook_v2::entry(
        &openbook_v2::ID,
        &account_infos,
        &anchor_lang::InstructionData::data(data),
    );

    if res.is_ok() {
        state.update(account_infos);
    }

    res
}
