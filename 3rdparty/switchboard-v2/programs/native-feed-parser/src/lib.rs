pub use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};
use std::convert::TryInto;
pub use switchboard_v2::{AggregatorAccountData, SWITCHBOARD_PROGRAM_ID};

entrypoint!(process_instruction);

fn process_instruction<'a>(
    _program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    _instruction_data: &'a [u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let aggregator = next_account_info(accounts_iter)?;

    let clock = Clock::get()?;

    // check feed owner
    let owner = *aggregator.owner;
    if owner != SWITCHBOARD_PROGRAM_ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    // load and deserialize feed
    let feed = AggregatorAccountData::new(aggregator)?;

    // check if feed has updated in the last 5 minutes
    let staleness = clock.unix_timestamp - feed.latest_confirmed_round.round_open_timestamp;
    if staleness > 300 {
        msg!("Feed has not been updated in {} seconds!", staleness);
        return Err(ProgramError::InvalidAccountData);
    }

    // get result
    let val: f64 = feed.get_result()?.try_into()?;
    msg!("Current feed result is {}!", val);

    Ok(())
}
