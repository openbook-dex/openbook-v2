use super::decimal::SwitchboardDecimal;
use super::error::SwitchboardError;
use anchor_lang::prelude::*;
use rust_decimal::Decimal;
use std::cell::Ref;

#[zero_copy]
#[repr(packed)]
#[derive(Default, Debug, PartialEq, Eq)]
pub struct Hash {
    /// The bytes used to derive the hash.
    pub data: [u8; 32],
}

#[zero_copy]
#[repr(packed)]
#[derive(Default, Debug, PartialEq, Eq)]
pub struct AggregatorRound {
    /// Maintains the number of successful responses received from nodes.
    /// Nodes can submit one successful response per round.
    pub num_success: u32,
    /// Number of error responses.
    pub num_error: u32,
    /// Whether an update request round has ended.
    pub is_closed: bool,
    /// Maintains the `solana_program::clock::Slot` that the round was opened at.
    pub round_open_slot: u64,
    /// Maintains the `solana_program::clock::UnixTimestamp;` the round was opened at.
    pub round_open_timestamp: i64,
    /// Maintains the current median of all successful round responses.
    pub result: SwitchboardDecimal,
    /// Standard deviation of the accepted results in the round.
    pub std_deviation: SwitchboardDecimal,
    /// Maintains the minimum node response this round.
    pub min_response: SwitchboardDecimal,
    /// Maintains the maximum node response this round.
    pub max_response: SwitchboardDecimal,
    /// Pubkeys of the oracles fulfilling this round.
    pub oracle_pubkeys_data: [Pubkey; 16],
    /// Represents all successful node responses this round. `NaN` if empty.
    pub medians_data: [SwitchboardDecimal; 16],
    /// Current rewards/slashes oracles have received this round.
    pub current_payout: [i64; 16],
    /// Keep track of which responses are fulfilled here.
    pub medians_fulfilled: [bool; 16],
    /// Keeps track of which errors are fulfilled here.
    pub errors_fulfilled: [bool; 16],
}

#[derive(Copy, Clone, Debug, AnchorSerialize, AnchorDeserialize, Eq, PartialEq)]
#[repr(u8)]
pub enum AggregatorResolutionMode {
    ModeRoundResolution = 0,
    ModeSlidingResolution = 1,
}
#[account(zero_copy)]
#[repr(packed)]
pub struct SlidingResultAccountData {
    pub data: [SlidingWindowElement; 16],
    pub bump: u8,
    pub _ebuf: [u8; 512],
}
#[zero_copy]
#[derive(Default)]
#[repr(packed)]
pub struct SlidingWindowElement {
    pub oracle_key: Pubkey,
    pub value: SwitchboardDecimal,
    pub slot: u64,
    pub timestamp: i64,
}

// #[zero_copy]
#[account(zero_copy)]
#[repr(packed)]
#[derive(Debug, PartialEq)]
pub struct AggregatorAccountData {
    /// Name of the aggregator to store on-chain.
    pub name: [u8; 32],
    /// Metadata of the aggregator to store on-chain.
    pub metadata: [u8; 128],
    /// Reserved.
    pub _reserved1: [u8; 32],
    /// Pubkey of the queue the aggregator belongs to.
    pub queue_pubkey: Pubkey,
    /// CONFIGS
    /// Number of oracles assigned to an update request.
    pub oracle_request_batch_size: u32,
    /// Minimum number of oracle responses required before a round is validated.
    pub min_oracle_results: u32,
    /// Minimum number of job results before an oracle accepts a result.
    pub min_job_results: u32,
    /// Minimum number of seconds required between aggregator rounds.
    pub min_update_delay_seconds: u32,
    /// Unix timestamp for which no feed update will occur before.
    pub start_after: i64,
    /// Change percentage required between a previous round and the current round. If variance percentage is not met, reject new oracle responses.
    pub variance_threshold: SwitchboardDecimal,
    /// Number of seconds for which, even if the variance threshold is not passed, accept new responses from oracles.
    pub force_report_period: i64,
    /// Timestamp when the feed is no longer needed.
    pub expiration: i64,
    //
    /// Counter for the number of consecutive failures before a feed is removed from a queue. If set to 0, failed feeds will remain on the queue.
    pub consecutive_failure_count: u64,
    /// Timestamp when the next update request will be available.
    pub next_allowed_update_time: i64,
    /// Flag for whether an aggregators configuration is locked for editing.
    pub is_locked: bool,
    /// Optional, public key of the crank the aggregator is currently using. Event based feeds do not need a crank.
    pub crank_pubkey: Pubkey,
    /// Latest confirmed update request result that has been accepted as valid.
    pub latest_confirmed_round: AggregatorRound,
    /// Oracle results from the current round of update request that has not been accepted as valid yet.
    pub current_round: AggregatorRound,
    /// List of public keys containing the job definitions for how data is sourced off-chain by oracles.
    pub job_pubkeys_data: [Pubkey; 16],
    /// Used to protect against malicious RPC nodes providing incorrect task definitions to oracles before fulfillment.
    pub job_hashes: [Hash; 16],
    /// Number of jobs assigned to an oracle.
    pub job_pubkeys_size: u32,
    /// Used to protect against malicious RPC nodes providing incorrect task definitions to oracles before fulfillment.
    pub jobs_checksum: [u8; 32],
    //
    /// The account delegated as the authority for making account changes.
    pub authority: Pubkey,
    /// Optional, public key of a history buffer account storing the last N accepted results and their timestamps.
    pub history_buffer: Pubkey,
    /// The previous confirmed round result.
    pub previous_confirmed_round_result: SwitchboardDecimal,
    /// The slot when the previous confirmed round was opened.
    pub previous_confirmed_round_slot: u64,
    /// 	Whether an aggregator is permitted to join a crank.
    pub disable_crank: bool,
    /// Job weights used for the weighted median of the aggregator's assigned job accounts.
    pub job_weights: [u8; 16],
    /// Unix timestamp when the feed was created.
    pub creation_timestamp: i64,
    /// Use sliding windoe or round based resolution
    /// NOTE: This changes result propogation in latest_round_result
    pub resolution_mode: AggregatorResolutionMode,
    /// Reserved for future info.
    pub _ebuf: [u8; 138],
}

impl AggregatorAccountData {
    /// Returns the deserialized Switchboard Aggregator account
    ///
    /// # Arguments
    ///
    /// * `switchboard_feed` - A Solana AccountInfo referencing an existing Switchboard Aggregator
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use switchboard_v2::AggregatorAccountData;
    ///
    /// let data_feed = AggregatorAccountData::new(feed_account_info)?;
    /// ```
    pub fn new<'info>(
        switchboard_feed: &'info AccountInfo<'info>,
    ) -> anchor_lang::Result<Ref<'info, AggregatorAccountData>> {
        let data = switchboard_feed.try_borrow_data()?;
        if data.len() < AggregatorAccountData::discriminator().len() {
            return Err(ErrorCode::AccountDiscriminatorNotFound.into());
        }

        let mut disc_bytes = [0u8; 8];
        disc_bytes.copy_from_slice(&data[..8]);
        if disc_bytes != AggregatorAccountData::discriminator() {
            return Err(ErrorCode::AccountDiscriminatorMismatch.into());
        }

        Ok(Ref::map(data, |data| {
            bytemuck::from_bytes(&data[8..std::mem::size_of::<AggregatorAccountData>() + 8])
        }))
    }

    /// Returns the deserialized Switchboard Aggregator account
    ///
    /// # Arguments
    ///
    /// * `data` - A Solana AccountInfo's data buffer
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use switchboard_v2::AggregatorAccountData;
    ///
    /// let data_feed = AggregatorAccountData::new(feed_account_info.try_borrow_data()?)?;
    /// ```
    pub fn new_from_bytes(data: &[u8]) -> anchor_lang::Result<&AggregatorAccountData> {
        if data.len() < AggregatorAccountData::discriminator().len() {
            return Err(ErrorCode::AccountDiscriminatorNotFound.into());
        }

        let mut disc_bytes = [0u8; 8];
        disc_bytes.copy_from_slice(&data[..8]);
        if disc_bytes != AggregatorAccountData::discriminator() {
            return Err(ErrorCode::AccountDiscriminatorMismatch.into());
        }

        Ok(bytemuck::from_bytes(&data[8..std::mem::size_of::<AggregatorAccountData>() + 8]))
    }

    /// If sufficient oracle responses, returns the latest on-chain result in SwitchboardDecimal format
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use switchboard_v2::AggregatorAccountData;
    /// use std::convert::TryInto;
    ///
    /// let feed_result = AggregatorAccountData::new(feed_account_info)?.get_result()?;
    /// let decimal: f64 = feed_result.try_into()?;
    /// ```
    pub fn get_result(&self) -> anchor_lang::Result<SwitchboardDecimal> {
        if self.resolution_mode == AggregatorResolutionMode::ModeSlidingResolution {
            return Ok(self.latest_confirmed_round.result);
        }
        let min_oracle_results = self.min_oracle_results;
        let latest_confirmed_round_num_success = self.latest_confirmed_round.num_success;
        if min_oracle_results > latest_confirmed_round_num_success {
            return Err(SwitchboardError::InvalidAggregatorRound.into());
        }
        Ok(self.latest_confirmed_round.result)
    }

    /// Check whether the confidence interval exceeds a given threshold
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use switchboard_v2::{AggregatorAccountData, SwitchboardDecimal};
    ///
    /// let feed = AggregatorAccountData::new(feed_account_info)?;
    /// feed.check_confidence_interval(SwitchboardDecimal::from_f64(0.80))?;
    /// ```
    pub fn check_confidence_interval(
        &self,
        max_confidence_interval: SwitchboardDecimal,
    ) -> anchor_lang::Result<()> {
        if self.latest_confirmed_round.std_deviation > max_confidence_interval {
            return Err(SwitchboardError::ConfidenceIntervalExceeded.into());
        }
        Ok(())
    }

    /// Check the variance (as a percentage difference from the max delivered
    /// oracle value) from all oracles.
    pub fn check_variace(&self, max_variance: Decimal) -> anchor_lang::Result<()> {
        if max_variance > Decimal::ONE {
            return Err(SwitchboardError::InvalidFunctionInput.into());
        }
        let min: Decimal = self.latest_confirmed_round.min_response.try_into().unwrap();
        let max: Decimal = self.latest_confirmed_round.max_response.try_into().unwrap();

        if min < Decimal::ZERO || max < Decimal::ZERO || min > max {
            return Err(SwitchboardError::AllowedVarianceExceeded.into());
        }
        if min / max > max_variance {
            return Err(SwitchboardError::AllowedVarianceExceeded.into());
        }
        Ok(())
    }

    /// Check whether the feed has been updated in the last max_staleness seconds
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use switchboard_v2::AggregatorAccountData;
    ///
    /// let feed = AggregatorAccountData::new(feed_account_info)?;
    /// feed.check_staleness(clock::Clock::get().unwrap().unix_timestamp, 300)?;
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

    fn discriminator() -> [u8; 8] {
        [217, 230, 65, 101, 201, 162, 27, 125]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    impl<'info> Default for AggregatorAccountData {
        fn default() -> Self {
            unsafe { std::mem::zeroed() }
        }
    }

    fn create_aggregator(lastest_round: AggregatorRound) -> AggregatorAccountData {
        let mut aggregator = AggregatorAccountData::default();
        aggregator.min_update_delay_seconds = 10;
        aggregator.latest_confirmed_round = lastest_round;
        aggregator.min_job_results = 10;
        aggregator.min_oracle_results = 10;
        return aggregator;
    }

    fn create_round(value: f64, num_success: u32, num_error: u32) -> AggregatorRound {
        let mut result = AggregatorRound::default();
        result.num_success = num_success;
        result.num_error = num_error;
        result.result = SwitchboardDecimal::from_f64(value);
        return result;
    }

    #[test]
    fn test_accept_current_on_sucess_count() {
        let lastest_round = create_round(100.0, 30, 0); // num success 30 > 10 min oracle result

        let aggregator = create_aggregator(lastest_round.clone());
        assert_eq!(
            aggregator.get_result().unwrap(),
            lastest_round.result.clone()
        );
    }

    #[test]
    fn test_reject_current_on_sucess_count() {
        let lastest_round = create_round(100.0, 5, 0); // num success 30 < 10 min oracle result
        let aggregator = create_aggregator(lastest_round.clone());

        assert!(
            aggregator.get_result().is_err(),
            "Aggregator is not currently populated with a valid round."
        );
    }

    #[test]
    fn test_no_valid_aggregator_result() {
        let aggregator = create_aggregator(AggregatorRound::default());

        assert!(
            aggregator.get_result().is_err(),
            "Aggregator is not currently populated with a valid round."
        );
    }
}
