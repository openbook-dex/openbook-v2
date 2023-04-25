pub mod actions;
pub use actions::*;

pub use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

pub use switchboard_v2::SWITCHBOARD_PROGRAM_ID;

declare_id!("HjjRFjCyQH3ne6Gg8Yn3TQafrrYecRrphwLwnh2A26vM");

const MAX_RESULT: u64 = u64::MAX;

const STATE_SEED: &[u8] = b"STATE";

#[program]
pub mod anchor_vrf_parser {
    use super::*;

    #[access_control(ctx.accounts.validate(&ctx, &params))]
    pub fn init_state(ctx: Context<InitState>, params: InitStateParams) -> Result<()> {
        InitState::actuate(&ctx, &params)
    }

    #[access_control(ctx.accounts.validate(&ctx))]
    pub fn update_result(ctx: Context<UpdateResult>) -> Result<()> {
        UpdateResult::actuate(&ctx)
    }

    #[access_control(ctx.accounts.validate(&ctx, &params))]
    pub fn request_result(ctx: Context<RequestResult>, params: RequestResultParams) -> Result<()> {
        RequestResult::actuate(&ctx, &params)
    }
}

#[repr(packed)]
#[account(zero_copy)]
pub struct VrfClient {
    pub bump: u8,
    pub max_result: u64,
    pub result_buffer: [u8; 32],
    pub result: u128,
    pub last_timestamp: i64,
    pub authority: Pubkey,
    pub vrf: Pubkey,
}
impl Default for VrfClient {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[event]
pub struct RequestingRandomness {
    pub vrf_client: Pubkey,
    pub max_result: u64,
    pub timestamp: i64,
}

#[event]
pub struct VrfClientInvoked {
    pub vrf_client: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct VrfClientResultUpdated {
    pub vrf_client: Pubkey,
    pub result: u128,
    pub result_buffer: [u8; 32],
    pub timestamp: i64,
}

#[error_code]
#[derive(Eq, PartialEq)]
pub enum VrfErrorCode {
    #[msg("Not a valid Switchboard account")]
    InvalidSwitchboardAccount,
    #[msg("The max result must not exceed u64")]
    MaxResultExceedsMaximum,
    #[msg("Current round result is empty")]
    EmptyCurrentRoundResult,
    #[msg("Invalid authority account provided.")]
    InvalidAuthorityError,
    #[msg("Invalid VRF account provided.")]
    InvalidVrfAccount,
}
