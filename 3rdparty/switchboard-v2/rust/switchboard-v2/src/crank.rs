use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};

#[zero_copy]
#[derive(Default)]
#[repr(packed)]
pub struct CrankRow {
    /// The PublicKey of the AggregatorAccountData.
    pub pubkey: Pubkey,
    /// The aggregator's next available update time.
    pub next_timestamp: i64,
}
unsafe impl Pod for CrankRow {}
unsafe impl Zeroable for CrankRow {}

#[account(zero_copy)]
#[repr(packed)]
pub struct CrankAccountData {
    /// Name of the crank to store on-chain.
    pub name: [u8; 32],
    /// Metadata of the crank to store on-chain.
    pub metadata: [u8; 64],
    /// Public key of the oracle queue who owns the crank.
    pub queue_pubkey: Pubkey,
    /// Number of aggregators added to the crank.
    pub pq_size: u32,
    /// Maximum number of aggregators allowed to be added to a crank.
    pub max_rows: u32,
    /// Pseudorandom value added to next aggregator update time.
    pub jitter_modifier: u8,
    /// Reserved for future info.
    pub _ebuf: [u8; 255],
    /// The public key of the CrankBuffer account holding a collection of Aggregator pubkeys and their next allowed update time.
    pub data_buffer: Pubkey,
}

impl CrankAccountData {}
