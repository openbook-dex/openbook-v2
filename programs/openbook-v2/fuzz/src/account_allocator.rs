use bumpalo::Bump;
use openbook_v2::state::*;
use solana_program::{
    account_info::AccountInfo, bpf_loader, clock::Epoch, program_pack::Pack, pubkey::Pubkey,
    rent::Rent, system_program,
};
use spl_token::state::Mint;

pub struct AccountAllocator(Bump);

impl AccountAllocator {
    pub fn new() -> Self {
        Self(Bump::new())
    }

    pub fn new_signer(&self, lamports: u64) -> AccountInfo {
        let bump = &self.0;
        AccountInfo::new(
            bump.alloc(Pubkey::new_unique()),
            true,
            true,
            bump.alloc(lamports),
            &mut [],
            &system_program::ID,
            false,
            Epoch::default(),
        )
    }

    pub fn new_mint(&self) -> AccountInfo {
        let bump = &self.0;
        let data = bump.alloc_slice_fill_copy(Mint::LEN, 0u8);
        let mut mint = Mint::default();
        mint.is_initialized = true;
        Mint::pack(mint, data).unwrap();
        AccountInfo::new(
            bump.alloc(Pubkey::new_unique()),
            false,
            true,
            bump.alloc(Rent::default().minimum_balance(data.len())),
            data,
            &spl_token::ID,
            false,
            Epoch::default(),
        )
    }

    pub fn new_program(&self, pubkey: Pubkey) -> AccountInfo {
        let bump = &self.0;
        AccountInfo::new(
            bump.alloc(pubkey),
            false,
            false,
            bump.alloc(0),
            &mut [],
            &bpf_loader::ID,
            true,
            Epoch::default(),
        )
    }

    pub fn new_stub_oracle(&self, mint: &Pubkey) -> AccountInfo {
        let bump = &self.0;
        let (pubkey, _bump) = Pubkey::find_program_address(
            &[b"StubOracle".as_ref(), mint.as_ref()],
            &openbook_v2::ID,
        );

        let len = 8 + std::mem::size_of::<StubOracle>();
        AccountInfo::new(
            bump.alloc(pubkey),
            false,
            true,
            bump.alloc(Rent::default().minimum_balance(len)),
            bump.alloc_slice_fill_copy(len, 0u8),
            &openbook_v2::ID,
            false,
            Epoch::default(),
        )
    }
}
