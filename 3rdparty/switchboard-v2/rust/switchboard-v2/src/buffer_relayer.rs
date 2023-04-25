use super::error::SwitchboardError;
use super::SWITCHBOARD_PROGRAM_ID;
use anchor_lang::prelude::*;
use anchor_lang::{Discriminator, Owner};

#[derive(AnchorDeserialize, Default, Debug)]
pub struct BufferRelayerAccountData {
    /// Name of the buffer account to store on-chain.
    pub name: [u8; 32],
    /// Public key of the OracleQueueAccountData that is currently assigned to fulfill buffer relayer update request.
    pub queue_pubkey: Pubkey,
    /// Token account to reward oracles for completing update request.
    pub escrow: Pubkey,
    /// The account delegated as the authority for making account changes.
    pub authority: Pubkey,
    /// Public key of the JobAccountData that defines how the buffer relayer is updated.
    pub job_pubkey: Pubkey,
    /// Used to protect against malicious RPC nodes providing incorrect task definitions to oracles before fulfillment
    pub job_hash: [u8; 32],
    /// Minimum delay between update request.
    pub min_update_delay_seconds: u32,
    /// Whether buffer relayer config is locked for further changes.
    pub is_locked: bool,
    /// The current buffer relayer update round that is yet to be confirmed.
    pub current_round: BufferRelayerRound,
    /// The latest confirmed buffer relayer update round.
    pub latest_confirmed_round: BufferRelayerRound,
    /// The buffer holding the latest confirmed result.
    pub result: Vec<u8>,
}

#[zero_copy]
#[derive(Default, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct BufferRelayerRound {
    /// Number of successful responses.
    pub num_success: u32,
    /// Number of error responses.
    pub num_error: u32,
    /// Slot when the buffer relayer round was opened.
    pub round_open_slot: u64,
    /// Timestamp when the buffer relayer round was opened.
    pub round_open_timestamp: i64,
    /// The public key of the oracle fulfilling the buffer relayer update request.
    pub oracle_pubkey: Pubkey,
}

impl BufferRelayerAccountData {
    /// Returns the deserialized Switchboard Buffer Relayer account
    ///
    /// # Arguments
    ///
    /// * `switchboard_buffer` - A Solana AccountInfo referencing an existing Switchboard BufferRelayer
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use switchboard_v2::BufferRelayerAccountData;
    ///
    /// let buffer_account = BufferRelayerAccountData::new(buffer_account_info)?;
    /// ```
    pub fn new<'a>(
        switchboard_buffer: &'a AccountInfo,
    ) -> anchor_lang::Result<Box<BufferRelayerAccountData>> {
        let data = switchboard_buffer.try_borrow_data()?;

        let mut disc_bytes = [0u8; 8];
        disc_bytes.copy_from_slice(&data[..8]);
        if disc_bytes != BufferRelayerAccountData::discriminator() {
            return Err(SwitchboardError::AccountDiscriminatorMismatch.into());
        }

        let mut v_mut = &data[8..];
        Ok(Box::new(BufferRelayerAccountData::deserialize(&mut v_mut)?))
    }

    pub fn get_result(&self) -> &Vec<u8> {
        return &self.result;
    }

    /// Check whether the buffer relayer has been updated in the last max_staleness seconds
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use switchboard_v2::BufferRelayerAccountData;
    ///
    /// let buffer = BufferRelayerAccountData::new(buffer_account_info)?;
    /// buffer.check_staleness(clock::Clock::get().unwrap().unix_timestamp, 300)?;
    /// ```
    pub fn check_staleness(
        &self,
        unix_timestamp: i64,
        max_staleness: i64,
    ) -> anchor_lang::Result<()> {
        let staleness = unix_timestamp - self.latest_confirmed_round.round_open_timestamp;
        if staleness > max_staleness {
            msg!("Feed has not been updated in {} seconds!", staleness);
            return Err(SwitchboardError::StaleFeed.into());
        }
        Ok(())
    }
}
impl Discriminator for BufferRelayerAccountData {
    // const DISCRIMINATOR: [u8; 8] = [50, 35, 51, 115, 169, 219, 158, 52];
    fn discriminator() -> [u8; 8] {
        [50, 35, 51, 115, 169, 219, 158, 52]
    }
}
impl Owner for BufferRelayerAccountData {
    fn owner() -> solana_program::pubkey::Pubkey {
        SWITCHBOARD_PROGRAM_ID
    }
}
