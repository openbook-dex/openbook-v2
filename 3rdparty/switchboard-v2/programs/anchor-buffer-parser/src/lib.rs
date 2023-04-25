use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock;
pub use switchboard_v2::{BufferRelayerAccountData, SWITCHBOARD_PROGRAM_ID};

declare_id!("96punQGZDShZGkzsBa3SsfTxfUnwu4XGpzXbhF7NTgcP");

#[derive(Accounts)]
#[instruction(params: ReadResultParams)]
pub struct ReadResult<'info> {
    /// CHECK
    #[account(mut,
        constraint = 
            *buffer.owner == SWITCHBOARD_PROGRAM_ID @ BufferErrorCode::InvalidSwitchboardAccount
    )]
    pub buffer: AccountInfo<'info>,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct ReadResultParams {
    pub expected_result: Option<Vec<u8>>,
}

#[program]
pub mod anchor_buffer_parser {
    use super::*;

    pub fn read_result(
        ctx: Context<ReadResult>,
        params: ReadResultParams,
    ) -> anchor_lang::Result<()> {
        // load and deserialize buffer
        let buffer = BufferRelayerAccountData::new(&ctx.accounts.buffer)?;

        msg!("Buffer account loaded!");

        let buffer_result = buffer.get_result();

        // get result
        msg!("Current buffer result is {:?}!", buffer_result);

        // check whether the buffer has been updated in the last 300 seconds
        buffer
            .check_staleness(clock::Clock::get().unwrap().unix_timestamp, 300)
            .map_err(|_| error!(BufferErrorCode::StaleBuffer))?;

        // compare buffer with expected result
        if let Some(expected_result) = params.expected_result {
            if expected_result != *buffer_result {
                msg!(
                    "Buffer mismatch, expected {:?}, actual {:?}",
                    expected_result,
                    buffer_result
                );
                return Err(error!(BufferErrorCode::ExpectedResultMismatch));
            }
        }

        let result_string = String::from_utf8(buffer.result)
            .map_err(|_| error!(BufferErrorCode::StringConversionFailed))?;

        msg!("Buffer string {:?}!", result_string);

        Ok(())
    }
}

#[error_code]
#[derive(Eq, PartialEq)]
pub enum BufferErrorCode {
    #[msg("Not a valid Switchboard account")]
    InvalidSwitchboardAccount,
    #[msg("Switchboard buffer does not match provided expected_result!")]
    ExpectedResultMismatch,
    #[msg("Switchboard buffer has not been updated in the last 5 minutes!")]
    StaleBuffer,
    #[msg("Failed to convert the buffer to a string!")]
    StringConversionFailed,
}
