use crate::accounts_state::AccountsState;
use anchor_spl::token::spl_token;
use base64::{prelude::BASE64_STANDARD, Engine};
use bumpalo::Bump;
use itertools::Itertools;
use log::debug;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, instruction::AccountMeta,
    instruction::Instruction, program_error::ProgramError, program_stubs, pubkey::Pubkey,
    rent::Rent, system_program,
};

pub struct TestSyscallStubs {}
impl program_stubs::SyscallStubs for TestSyscallStubs {
    fn sol_log(&self, message: &str) {
        debug!("Program log: {}", message);
    }

    fn sol_log_data(&self, fields: &[&[u8]]) {
        debug!(
            "Program data: {}",
            fields.iter().map(|b| BASE64_STANDARD.encode(b)).join(" ")
        );
    }

    fn sol_invoke_signed(
        &self,
        instruction: &Instruction,
        account_infos: &[AccountInfo],
        signers_seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        let mut new_account_infos = vec![];

        let pdas = signers_seeds
            .iter()
            .map(|seeds| Pubkey::create_program_address(seeds, &openbook_v2::id()).unwrap())
            .collect::<Vec<_>>();

        for meta in instruction.accounts.iter() {
            for account_info in account_infos.iter() {
                if meta.pubkey == *account_info.key {
                    let mut new_account_info = account_info.clone();
                    if pdas.iter().any(|pda| pda == account_info.key) {
                        new_account_info.is_signer = true;
                    }
                    new_account_infos.push(new_account_info);
                }
            }
        }

        match instruction.program_id {
            // accounts should already be created & reallocated
            id if id == system_program::ID => Ok(()),
            id if id == spl_associated_token_account::ID => Ok(()),
            id if id == spl_token::ID => spl_token::processor::Processor::process(
                &instruction.program_id,
                &new_account_infos,
                &instruction.data,
            ),
            id if id == openbook_v2::ID => {
                let extended_lifetime_accs = unsafe {
                    core::mem::transmute::<&[AccountInfo], &[AccountInfo<'_>]>(
                        new_account_infos.as_ref(),
                    )
                };
                openbook_v2::entry(
                    &instruction.program_id,
                    &extended_lifetime_accs,
                    &instruction.data,
                )
            }
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
    data: &impl anchor_lang::InstructionData,
    accounts: &impl anchor_lang::ToAccountMetas,
    remaining_accounts: &[AccountMeta],
) -> ProgramResult {
    let bump = Bump::new();
    let mut metas = anchor_lang::ToAccountMetas::to_account_metas(accounts, None);
    metas.extend_from_slice(remaining_accounts);
    let account_infos = state.account_infos(&bump, metas);

    let res = openbook_v2::entry(
        &openbook_v2::ID,
        &account_infos,
        &anchor_lang::InstructionData::data(data),
    );

    if res.is_ok() {
        state.update(&account_infos);
    }

    res
}
