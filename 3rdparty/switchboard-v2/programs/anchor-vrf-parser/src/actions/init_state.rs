use crate::*;
use anchor_lang::prelude::*;
pub use switchboard_v2::VrfAccountData;
use std::mem;

#[derive(Accounts)]
#[instruction(params: InitStateParams)]
pub struct InitState<'info> {
    #[account(
        init,
        seeds = [
            STATE_SEED, 
            vrf.key().as_ref(),
            authority.key().as_ref(),
        ],
        payer = payer,
        space = 8 + mem::size_of::<VrfClient>(),
        bump,
    )]
    pub state: AccountLoader<'info, VrfClient>,
    /// CHECK:
    pub authority: AccountInfo<'info>,
    /// CHECK:
    #[account(mut, signer)]
    /// CHECK:
    pub payer: AccountInfo<'info>,
    #[account(
        constraint = 
            *vrf.to_account_info().owner == SWITCHBOARD_PROGRAM_ID @ VrfErrorCode::InvalidSwitchboardAccount
    )]
    pub vrf: AccountLoader<'info, VrfAccountData>,
    #[account(address = solana_program::system_program::ID)]
    pub system_program: Program<'info, System>,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct InitStateParams {
    pub max_result: u64,
}

impl InitState<'_> {
    pub fn validate(&self, _ctx: &Context<Self>, params: &InitStateParams) -> Result<()> {
        msg!("Validate init");
        if params.max_result > MAX_RESULT {
            return Err(error!(VrfErrorCode::MaxResultExceedsMaximum));
        }

        Ok(())
    }

    pub fn actuate(ctx: &Context<Self>, params: &InitStateParams) -> Result<()> {
        msg!("Actuate init");

        msg!("Checking VRF Account");
        let vrf = ctx.accounts.vrf.load()?;
        // client state needs to be authority in order to sign request randomness instruction
        if vrf.authority != ctx.accounts.state.key() {
            return Err(error!(VrfErrorCode::InvalidAuthorityError));
        }
        drop(vrf);

        msg!("Setting VrfClient state");
        let mut state = ctx.accounts.state.load_init()?;
        *state = VrfClient::default();
        state.bump = ctx.bumps.get("state").unwrap().clone();
        state.authority =  ctx.accounts.authority.key.clone();
        state.vrf = ctx.accounts.vrf.key();
        
        msg!("Setting VrfClient max_result");
        if params.max_result == 0 {
            state.max_result = MAX_RESULT;
        } else {
            state.max_result = params.max_result;
        }

        Ok(())
    }
}
