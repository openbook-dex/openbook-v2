use anchor_lang::prelude::*;

#[account(zero_copy)]
#[repr(packed)]
pub struct SbState {
    /// The account authority permitted to make account changes.
    pub authority: Pubkey,
    /// The token mint used for oracle rewards, aggregator leases, and other reward incentives.
    pub token_mint: Pubkey,
    /// Token vault used by the program to receive kickbacks.
    pub token_vault: Pubkey,
    /// The token mint used by the DAO.
    pub dao_mint: Pubkey,
    /// Reserved for future info.
    pub _ebuf: [u8; 992],
}

impl SbState {}
