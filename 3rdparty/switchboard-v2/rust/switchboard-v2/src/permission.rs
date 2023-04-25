use super::error::SwitchboardError;
use anchor_lang::prelude::*;
// use bytemuck::{Pod, Zeroable};
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::Instruction;
use solana_program::program::{invoke, invoke_signed};
// use std::cell::Ref;

#[derive(Copy, Clone, AnchorSerialize, AnchorDeserialize, Eq, PartialEq)]
pub enum SwitchboardPermission {
    /// queue authority has permitted an Oracle Account to heartbeat on it's queue and receive update requests. Oracles always need permissions to join a queue.
    PermitOracleHeartbeat = 1 << 0,
    /// queue authority has permitted an Aggregator Account to request updates from it's oracles or join an existing crank. Note: Not required if a queue has unpermissionedFeedsEnabled.
    PermitOracleQueueUsage = 1 << 1, // TODO: rename
    /// queue authority has permitted a VRF Account to request randomness from it's oracles. Note: Not required if a queue has unpermissionedVrfEnabled.
    PermitVrfRequests = 1 << 2,
}

#[account(zero_copy)]
#[repr(packed)]
pub struct PermissionAccountData {
    /// The authority that is allowed to set permissions for this account.
    pub authority: Pubkey,
    /// The SwitchboardPermission enumeration assigned by the granter to the grantee.
    pub permissions: u32,
    /// Public key of account that is granting permissions to use its resources.
    pub granter: Pubkey,
    /// Public key of account that is being assigned permissions to use a granters resources.
    pub grantee: Pubkey,
    /// unused currently. may want permission PDA per permission for
    /// unique expiration periods, BUT currently only one permission
    /// per account makes sense for the infra. Dont over engineer.
    pub expiration: i64,
    /// Reserved for future info.
    pub _ebuf: [u8; 256],
}

impl PermissionAccountData {}

#[derive(Accounts)]
#[instruction(params: PermissionSetParams)] // rpc parameters hint
pub struct PermissionSet<'info> {
    #[account(mut, has_one = authority @ SwitchboardError::InvalidAuthority )]
    pub permission: AccountLoader<'info, PermissionAccountData>,
    #[account(signer)]
    pub authority: AccountInfo<'info>,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct PermissionSetParams {
    pub permission: SwitchboardPermission,
    pub enable: bool,
}

impl<'info> PermissionSet<'info> {
    fn discriminator() -> [u8; 8] {
        [211, 122, 185, 120, 129, 182, 55, 103]
    }

    pub fn get_instruction(
        &self,
        program_id: Pubkey,
        params: PermissionSetParams,
    ) -> anchor_lang::Result<Instruction> {
        let accounts = self.to_account_metas(None);

        let mut data: Vec<u8> = PermissionSet::discriminator().try_to_vec()?;
        let mut param_vec: Vec<u8> = params.try_to_vec()?;
        data.append(&mut param_vec);

        let instruction = Instruction::new_with_bytes(program_id, &data, accounts);
        Ok(instruction)
    }

    pub fn invoke(
        &self,
        program: AccountInfo<'info>,
        permission: SwitchboardPermission,
        enable: bool,
    ) -> ProgramResult {
        let cpi_params = PermissionSetParams { permission, enable };
        let instruction = self.get_instruction(program.key.clone(), cpi_params)?;
        let account_infos = self.to_account_infos();

        invoke(&instruction, &account_infos[..])
        // .map_err(|_| error!(SwitchboardError::VrfCpiError))
    }

    pub fn invoke_signed(
        &self,
        program: AccountInfo<'info>,
        permission: SwitchboardPermission,
        enable: bool,
        signer_seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        let cpi_params = PermissionSetParams { permission, enable };
        let instruction = self.get_instruction(program.key.clone(), cpi_params)?;
        let account_infos = self.to_account_infos();

        invoke_signed(&instruction, &account_infos[..], signer_seeds)
        // .map_err(|_| error!(SwitchboardError::VrfCpiSignedError))
    }

    fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        return vec![
            self.permission.to_account_info().clone(),
            self.authority.clone(),
        ];
    }

    #[allow(unused_variables)]
    fn to_account_metas(&self, is_signer: Option<bool>) -> Vec<AccountMeta> {
        return vec![
            AccountMeta {
                pubkey: self.permission.key(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.authority.key.clone(),
                is_signer: true,
                is_writable: false,
            },
        ];
    }
}
