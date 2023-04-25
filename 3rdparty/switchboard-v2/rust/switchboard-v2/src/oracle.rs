use anchor_lang::prelude::*;

#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize)]
pub enum OracleResponseType {
    TypeSuccess,
    TypeError,
    TypeDisagreement,
    TypeNoResponse,
}
#[zero_copy]
#[derive(Default)]
#[repr(packed)]
pub struct OracleMetrics {
    /// Number of consecutive successful update request.
    pub consecutive_success: u64,
    /// Number of consecutive update request that resulted in an error.
    pub consecutive_error: u64,
    /// Number of consecutive update request that resulted in a disagreement with the accepted median result.
    pub consecutive_disagreement: u64,
    /// Number of consecutive update request that were posted on-chain late and not included in an accepted result.
    pub consecutive_late_response: u64,
    /// Number of consecutive update request that resulted in a failure.
    pub consecutive_failure: u64,
    /// Total number of successful update request.
    pub total_success: u128,
    /// Total number of update request that resulted in an error.
    pub total_error: u128,
    /// Total number of update request that resulted in a disagreement with the accepted median result.
    pub total_disagreement: u128,
    /// Total number of update request that were posted on-chain late and not included in an accepted result.
    pub total_late_response: u128,
}

#[account(zero_copy)]
#[repr(packed)]
pub struct OracleAccountData {
    /// Name of the oracle to store on-chain.
    pub name: [u8; 32],
    /// Metadata of the oracle to store on-chain.
    pub metadata: [u8; 128],
    /// The account delegated as the authority for making account changes or withdrawing funds from a staking wallet.
    pub oracle_authority: Pubkey,
    /// Unix timestamp when the oracle last heartbeated
    pub last_heartbeat: i64,
    /// Flag dictating if an oracle is active and has heartbeated before the queue's oracle timeout parameter.
    pub num_in_use: u32,
    // Must be unique per oracle account and authority should be a pda
    /// Stake account and reward/slashing wallet.
    pub token_account: Pubkey,
    /// Public key of the oracle queue who has granted it permission to use its resources.
    pub queue_pubkey: Pubkey,
    /// Oracle track record.
    pub metrics: OracleMetrics,
    /// Reserved for future info.
    pub _ebuf: [u8; 256],
}

impl OracleAccountData {}
