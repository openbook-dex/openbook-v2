use bumpalo::Bump;
use openbook_v2::state::*;
use solana_program::{
    account_info::AccountInfo, bpf_loader, clock::Epoch, program_pack::Pack, pubkey::Pubkey,
    rent::Rent, system_program,
};
use spl_token::state::{Account, Mint};

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

    pub fn new_ata(&self, owner: &Pubkey, mint: &Pubkey) -> AccountInfo {
        let bump = &self.0;
        let data = bump.alloc_slice_fill_copy(Account::LEN, 0u8);
        let mut account = Account::default();
        account.state = spl_token::state::AccountState::Initialized;
        account.mint = *mint;
        account.owner = *owner;
        Account::pack(account, data).unwrap();

        let pubkey = spl_associated_token_account::get_associated_token_address(owner, mint);
        AccountInfo::new(
            bump.alloc(pubkey),
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

    pub fn new_market(&self) -> AccountInfo {
        let market_index: MarketIndex = 0;
        let (pubkey, _bump) = Pubkey::find_program_address(
            &[b"Market".as_ref(), market_index.to_le_bytes().as_ref()],
            &openbook_v2::ID,
        );

        let len = 8 + std::mem::size_of::<Market>();
        let bump = &self.0;
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

    pub fn new_bookside(&self) -> AccountInfo {
        let bump = &self.0;
        let len = 8 + std::mem::size_of::<BookSide>();
        AccountInfo::new(
            bump.alloc(Pubkey::new_unique()),
            false,
            true,
            bump.alloc(Rent::default().minimum_balance(len)),
            bump.alloc_slice_fill_copy(len, 0u8),
            &openbook_v2::ID,
            false,
            Epoch::default(),
        )
    }

    pub fn new_event_queue(&self) -> AccountInfo {
        let bump = &self.0;
        let len = 8 + std::mem::size_of::<EventQueue>();
        AccountInfo::new(
            bump.alloc(Pubkey::new_unique()),
            false,
            true,
            bump.alloc(Rent::default().minimum_balance(len)),
            bump.alloc_slice_fill_copy(len, 0u8),
            &openbook_v2::ID,
            false,
            Epoch::default(),
        )
    }

    pub fn new_stub_oracle(&self, mint: &Pubkey) -> AccountInfo {
        let (pubkey, _bump) = Pubkey::find_program_address(
            &[b"StubOracle".as_ref(), mint.as_ref()],
            &openbook_v2::ID,
        );

        let len = 8 + std::mem::size_of::<StubOracle>();
        let bump = &self.0;
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
