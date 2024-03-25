use anchor_lang::AccountDeserialize;
use anchor_spl::token::spl_token::{
    self,
    state::{Account as TokenAccount, AccountState, Mint},
};
use bumpalo::Bump;
use solana_program::{
    account_info::AccountInfo, bpf_loader, clock::Epoch, instruction::AccountMeta,
    program_pack::Pack, pubkey::Pubkey, rent::Rent, system_program,
};
use solana_sdk::account::{Account, WritableAccount};
use std::collections::HashMap;

pub struct UserAccounts {
    pub owner: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub open_orders: Pubkey,
}

pub struct AccountsState(HashMap<Pubkey, Account>);

impl Default for AccountsState {
    fn default() -> Self {
        Self::new()
    }
}

impl AccountsState {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert(&mut self, pubkey: Pubkey, account: Account) {
        self.0.insert(pubkey, account);
    }

    pub fn get_account<T: AccountDeserialize>(&self, pubkey: &Pubkey) -> Option<T> {
        self.0
            .get(pubkey)
            .and_then(|acc| AccountDeserialize::try_deserialize(&mut &acc.data[..]).ok())
    }

    pub fn get_balance(&self, pubkey: &Pubkey) -> u64 {
        self.get_account::<anchor_spl::token::TokenAccount>(pubkey)
            .unwrap()
            .amount
    }

    pub fn account_infos<'a, 'b: 'a>(
        &'a self,
        bump: &'b Bump,
        metas: Vec<AccountMeta>,
    ) -> Vec<AccountInfo<'b>> {
        let mut infos: Vec<AccountInfo> = vec![];

        metas.iter().for_each(|meta| {
            if let Some(info) = infos.iter().find(|info| info.key == &meta.pubkey) {
                infos.push(info.clone());
            } else {
                let account = self.0.get(&meta.pubkey).unwrap();
                infos.push(AccountInfo::new(
                    bump.alloc(meta.pubkey),
                    meta.is_signer,
                    meta.is_writable,
                    bump.alloc(account.lamports),
                    bump.alloc_slice_copy(&account.data),
                    bump.alloc(account.owner),
                    account.executable,
                    account.rent_epoch,
                ));
            }
        });

        infos
    }

    pub fn update(&mut self, infos: &[AccountInfo]) {
        infos.iter().for_each(|info| {
            let account = self.0.get_mut(info.key).unwrap();
            let new_data = info.data.borrow();
            let new_lamports = **info.lamports.borrow();
            if new_lamports != account.lamports || *new_data != account.data {
                account.data.copy_from_slice(*new_data);
                account.lamports = new_lamports;
            }
        });
    }

    pub fn add_program(&mut self, pubkey: Pubkey) -> &mut Self {
        self.insert(
            pubkey,
            Account::create(0, vec![], bpf_loader::ID, true, Epoch::default()),
        );
        self
    }

    pub fn add_account_with_lamports(&mut self, pubkey: Pubkey, lamports: u64) -> &mut Self {
        self.insert(
            pubkey,
            Account::create(
                lamports,
                vec![],
                system_program::ID,
                false,
                Epoch::default(),
            ),
        );
        self
    }

    pub fn add_token_account_with_lamports(
        &mut self,
        pubkey: Pubkey,
        owner: Pubkey,
        mint: Pubkey,
        amount: u64,
    ) -> &mut Self {
        let mut data = vec![0_u8; TokenAccount::LEN];
        let account = TokenAccount {
            state: AccountState::Initialized,
            mint,
            owner,
            amount,
            ..TokenAccount::default()
        };
        TokenAccount::pack(account, &mut data).unwrap();
        self.insert(
            pubkey,
            Account::create(
                Rent::default().minimum_balance(data.len()),
                data,
                spl_token::ID,
                false,
                Epoch::default(),
            ),
        );
        self
    }

    pub fn add_mint(&mut self, pubkey: Pubkey) -> &mut Self {
        let mut data = vec![0_u8; Mint::LEN];
        let mint = Mint {
            is_initialized: true,
            ..Mint::default()
        };
        Mint::pack(mint, &mut data).unwrap();
        self.insert(
            pubkey,
            Account::create(
                Rent::default().minimum_balance(data.len()),
                data,
                spl_token::ID,
                false,
                Epoch::default(),
            ),
        );
        self
    }

    pub fn add_empty_system_account(&mut self, pubkey: Pubkey) -> &mut Self {
        self.insert(pubkey, Account::new(0, 0, &system_program::ID));
        self
    }

    pub fn add_openbook_account<T>(&mut self, pubkey: Pubkey) -> &mut Self {
        let len = 8 + std::mem::size_of::<T>();
        self.insert(pubkey, zero_account(len));
        self
    }

    pub fn add_open_orders_indexer<T>(&mut self, pubkey: Pubkey) -> &mut Self {
        let len = openbook_v2::state::OpenOrdersIndexer::space(1);
        self.insert(pubkey, zero_account(len));
        self
    }
}

fn zero_account(len: usize) -> Account {
    Account::create(
        Rent::default().minimum_balance(len),
        vec![0; len],
        openbook_v2::ID,
        false,
        Epoch::default(),
    )
}
