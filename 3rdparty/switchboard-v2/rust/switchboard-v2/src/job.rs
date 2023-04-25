use anchor_lang::prelude::*;

#[account]
pub struct JobAccountData {
    /// Name of the job to store on-chain.
    pub name: [u8; 32],
    /// Metadata of the job to store on-chain.
    pub metadata: [u8; 64],
    /// The account delegated as the authority for making account changes.
    pub authority: Pubkey,
    /// Unix timestamp when the job is considered invalid
    pub expiration: i64,
    /// Hash of the serialized data to prevent tampering.
    pub hash: [u8; 32],
    /// Serialized protobuf containing the collection of task to retrieve data off-chain.
    pub data: Vec<u8>,
    /// The number of data feeds referencing the job account..
    pub reference_count: u32,
    /// The token amount funded into a feed that contains this job account.
    pub total_spent: u64,
    /// Unix timestamp when the job was created on-chain.
    pub created_at: i64,
}

impl JobAccountData {}
