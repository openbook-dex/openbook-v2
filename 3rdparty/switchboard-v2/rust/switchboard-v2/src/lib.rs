use anchor_lang::prelude::*;
use solana_program::pubkey;

pub mod aggregator;
pub mod buffer_relayer;
pub mod crank;
pub mod decimal;
pub mod error;
pub mod history_buffer;
pub mod job;
pub mod oracle;
pub mod permission;
pub mod queue;
pub mod sb_state;
pub mod vrf;

pub use aggregator::{AggregatorAccountData, AggregatorRound};
pub use buffer_relayer::{BufferRelayerAccountData, BufferRelayerRound};
pub use crank::{CrankAccountData, CrankRow};
pub use decimal::SwitchboardDecimal;
pub use error::SwitchboardError;
pub use history_buffer::{AggregatorHistoryBuffer, AggregatorHistoryRow};
pub use job::JobAccountData;
pub use oracle::{OracleAccountData, OracleMetrics};
pub use permission::{PermissionAccountData, PermissionSet, SwitchboardPermission};
pub use queue::OracleQueueAccountData;
pub use sb_state::SbState;
pub use vrf::{VrfAccountData, VrfRequestRandomness, VrfRound, VrfStatus};

/// Seed used to derive the SbState PDA.
pub const STATE_SEED: &[u8] = b"STATE";
/// Seed used to derive the PermissionAccountData PDA.
pub const PERMISSION_SEED: &[u8] = b"PermissionAccountData";
/// Seed used to derive the LeaseAccountData PDA.
pub const LEASE_SEED: &[u8] = b"LeaseAccountData";
/// Seed used to derive the OracleAccountData PDA.
pub const ORACLE_SEED: &[u8] = b"OracleAccountData";
/// Discriminator used for Switchboard buffer accounts.
pub const BUFFER_DISCRIMINATOR: &[u8] = b"BUFFERxx";
/// Seed used to derive the SlidingWindow PDA.
//const SLIDING_RESULT_SEED: &[u8] = b"SlidingResultAccountData";

/// Mainnet program id for Switchboard v2
pub const SWITCHBOARD_V2_MAINNET: Pubkey = pubkey!("SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f");

/// Devnet program id for Switchboard v2
pub const SWITCHBOARD_V2_DEVNET: Pubkey = pubkey!("2TfB33aLaneQb5TNVwyDz3jSZXS6jdW2ARw1Dgf84XCG");

#[cfg(feature = "devnet")]
/// Switchboard Program ID.
pub const SWITCHBOARD_PROGRAM_ID: Pubkey = SWITCHBOARD_V2_DEVNET;
#[cfg(not(feature = "devnet"))]
/// Switchboard Program ID.
pub const SWITCHBOARD_PROGRAM_ID: Pubkey = SWITCHBOARD_V2_MAINNET;

#[cfg(feature = "devnet")]
declare_id!(SWITCHBOARD_V2_DEVNET);
#[cfg(not(feature = "devnet"))]
declare_id!(SWITCHBOARD_V2_MAINNET);
