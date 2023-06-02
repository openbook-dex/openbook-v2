use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, instruction::Instruction,
    program_error::ProgramError, program_stubs, pubkey::Pubkey, rent::Rent, system_program,
};

struct TestSyscallStubs {}
impl program_stubs::SyscallStubs for TestSyscallStubs {
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

fn test_syscall_stubs() {
    use std::sync::Once;
    static ONCE: Once = Once::new();

    ONCE.call_once(|| {
        program_stubs::set_syscall_stubs(Box::new(TestSyscallStubs {}));
    });
}

pub fn do_process_instruction(instruction: Instruction, accounts: &[AccountInfo]) -> ProgramResult {
    test_syscall_stubs();

    // approximate the logic in the actual runtime which runs the instruction
    // and only updates accounts if the instruction is successful
    let account_infos = accounts.clone();

    let res = if instruction.program_id == openbook_v2::id() {
        openbook_v2::entry(&instruction.program_id, &account_infos, &instruction.data)
    } else {
        spl_token::processor::Processor::process(
            &instruction.program_id,
            &account_infos,
            &instruction.data,
        )
    };

    if res.is_ok() {
        let mut account_metas = instruction
            .accounts
            .iter()
            .zip(accounts)
            .map(|(account_meta, account)| (&account_meta.pubkey, account))
            .collect::<Vec<_>>();
        for account_info in account_infos.iter() {
            for account_meta in account_metas.iter_mut() {
                if account_info.key == account_meta.0 {
                    let account = &mut account_meta.1;
                    let mut lamports = account.lamports.borrow_mut();
                    **lamports = **account_info.lamports.borrow();
                    let mut data = account.data.borrow_mut();
                    data.clone_from_slice(*account_info.data.borrow());
                }
            }
        }
    }
    res
}
