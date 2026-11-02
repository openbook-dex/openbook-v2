#![allow(dead_code)]
#![allow(clippy::await_holding_refcell_ref)]

use std::cell::RefCell;
use std::sync::{Arc, RwLock};

use super::utils::TestKeypair;
use anchor_lang::AccountDeserialize;
use anchor_spl::token::TokenAccount;
use solana_program::{program_pack::Pack, rent::*, system_instruction};
use solana_program_test::*;
use solana_sdk::{
    account::ReadableAccount,
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_token::*;

pub struct SolanaCookie {
    pub context: RefCell<ProgramTestContext>,
    pub rent: Rent,
    pub logger_capture: Arc<RwLock<Vec<String>>>,
    pub logger_lock: Arc<RwLock<()>>,
    pub last_transaction_log: RefCell<Vec<String>>,
}

impl SolanaCookie {
    pub async fn process_transaction(
        &self,
        instructions: &[Instruction],
        signers: Option<&[TestKeypair]>,
    ) -> Result<(), BanksClientError> {
        // The locking in this function is convoluted:
        // We capture the program log output by overriding the global logger and capturing
        // messages there. This logger is potentially shared among multiple tests that run
        // concurrently.
        // To allow each independent SolanaCookie to capture only the logs from the transaction
        // passed to process_transaction, wo globally hold the "program_log_lock" for the
        // duration that the tx needs to process. So only a single one can run at a time.
        let tx_log_lock = Arc::new(self.logger_lock.write().unwrap());
        self.logger_capture.write().unwrap().clear();

        let mut context = self.context.borrow_mut();

        let mut transaction =
            Transaction::new_with_payer(instructions, Some(&context.payer.pubkey()));

        let mut all_signers = vec![&context.payer];
        let signer_keypairs =
            signers.map(|signers| signers.iter().map(|s| s.into()).collect::<Vec<Keypair>>());
        let signer_keypair_refs = signer_keypairs
            .as_ref()
            .map(|kps| kps.iter().collect::<Vec<&Keypair>>());

        if let Some(signer_keypair_refs) = signer_keypair_refs {
            all_signers.extend(signer_keypair_refs.iter());
        }

        // This fails when warping is involved - https://gitmemory.com/issue/solana-labs/solana/18201/868325078
        // let recent_blockhash = self.context.banks_client.get_recent_blockhash().await.unwrap();

        transaction.sign(&all_signers, context.last_blockhash);

        let result = context
            .banks_client
            .process_transaction_with_commitment(
                transaction,
                solana_sdk::commitment_config::CommitmentLevel::Processed,
            )
            .await;

        *self.last_transaction_log.borrow_mut() = self.logger_capture.read().unwrap().clone();

        drop(tx_log_lock);
        drop(context);

        // This makes sure every transaction gets a new blockhash, avoiding issues where sending
        // the same transaction again would lead to it being skipped.
        self.advance_by_slots(1).await;

        result
    }

    pub async fn get_clock(&self) -> solana_program::clock::Clock {
        self.context
            .borrow_mut()
            .banks_client
            .get_sysvar::<solana_program::clock::Clock>()
            .await
            .unwrap()
    }

    /// Compute units the transaction consumes, simulated against the current bank.
    pub async fn simulate_compute_units(
        &self,
        instructions: &[Instruction],
        signers: &[TestKeypair],
    ) -> u64 {
        let mut context = self.context.borrow_mut();
        let mut transaction =
            Transaction::new_with_payer(instructions, Some(&context.payer.pubkey()));
        let signer_keypairs: Vec<Keypair> = signers.iter().map(|s| s.into()).collect();
        let mut all_signers = vec![&context.payer];
        all_signers.extend(signer_keypairs.iter());
        transaction.sign(&all_signers, context.last_blockhash);

        let simulation = context
            .banks_client
            .simulate_transaction(transaction)
            .await
            .unwrap();
        assert_eq!(simulation.result, Some(Ok(())), "simulation failed");
        simulation.simulation_details.unwrap().units_consumed
    }

    /// Move the Clock sysvar to `unix_timestamp`. `epoch_start_timestamp` is shifted by the same
    /// amount, the bank derives the clock of the following slots from it.
    pub async fn set_clock_unix_timestamp(&self, unix_timestamp: i64) {
        let mut clock = self.get_clock().await;
        clock.epoch_start_timestamp += unix_timestamp - clock.unix_timestamp;
        clock.unix_timestamp = unix_timestamp;
        self.context.borrow_mut().set_sysvar(&clock);
    }

    pub async fn advance_by_slots(&self, slots: u64) {
        let clock = self.get_clock().await;
        self.context
            .borrow_mut()
            .warp_to_slot(clock.slot + slots + 1)
            .unwrap();
    }

    pub async fn advance_clock_to(&self, target: i64) {
        let mut clock = self.get_clock().await;

        // just advance enough to ensure we get changes over last_updated in various ix
        // if this gets too slow for our tests, remove and replace with manual time offset
        // which is configurable
        while clock.unix_timestamp <= target {
            self.context
                .borrow_mut()
                .warp_to_slot(clock.slot + 50)
                .unwrap();
            clock = self.get_clock().await;
        }
    }

    pub async fn advance_clock_to_next_multiple(&self, window: i64) {
        let ts = self.get_clock().await.unix_timestamp;
        self.advance_clock_to(ts / window * window + window).await
    }

    pub async fn advance_clock(&self, seconds: i64) {
        let clock = self.get_clock().await;
        self.advance_clock_to(clock.unix_timestamp + seconds).await
    }

    pub async fn get_newest_slot_from_history(&self) -> u64 {
        self.context
            .borrow_mut()
            .banks_client
            .get_sysvar::<solana_program::slot_history::SlotHistory>()
            .await
            .unwrap()
            .newest()
    }

    pub async fn create_account_from_len(&self, owner: &Pubkey, len: usize) -> Pubkey {
        let key = TestKeypair::new();
        let rent = self.rent.minimum_balance(len);
        let create_account_instr = solana_sdk::system_instruction::create_account(
            &self.context.borrow().payer.pubkey(),
            &key.pubkey(),
            rent,
            len as u64,
            owner,
        );
        self.process_transaction(&[create_account_instr], Some(&[key]))
            .await
            .unwrap();
        key.pubkey()
    }

    pub async fn create_account_for_type<T>(&self, owner: &Pubkey) -> Pubkey {
        let key = TestKeypair::new();
        let len = 8 + std::mem::size_of::<T>();
        let rent = self.rent.minimum_balance(len);
        let create_account_instr = solana_sdk::system_instruction::create_account(
            &self.context.borrow().payer.pubkey(),
            &key.pubkey(),
            rent,
            len as u64,
            owner,
        );
        self.process_transaction(&[create_account_instr], Some(&[key]))
            .await
            .unwrap();
        key.pubkey()
    }

    pub async fn create_token_account(&self, owner: &Pubkey, mint: Pubkey) -> Pubkey {
        let keypair = TestKeypair::new();
        let rent = self.rent.minimum_balance(spl_token::state::Account::LEN);

        let instructions = [
            system_instruction::create_account(
                &self.context.borrow().payer.pubkey(),
                &keypair.pubkey(),
                rent,
                spl_token::state::Account::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &keypair.pubkey(),
                &mint,
                owner,
            )
            .unwrap(),
        ];

        self.process_transaction(&instructions, Some(&[keypair]))
            .await
            .unwrap();
        keypair.pubkey()
    }

    pub async fn create_associated_token_account(&self, owner: &Pubkey, mint: Pubkey) -> Pubkey {
        let instruction =
            spl_associated_token_account::instruction::create_associated_token_account(
                &self.context.borrow().payer.pubkey(),
                owner,
                &mint,
                &spl_token::id(),
            );

        self.process_transaction(&[instruction], None)
            .await
            .unwrap();

        spl_associated_token_account::get_associated_token_address(owner, &mint)
    }

    // Note: Only one table can be created per authority per slot!
    // pub async fn create_address_lookup_table(
    //     &self,
    //     authority: TestKeypair,
    //     payer: TestKeypair,
    // ) -> Pubkey {
    //     let (instruction, alt_address) =
    //         solana_address_lookup_table_program::instruction::create_lookup_table(
    //             authority.pubkey(),
    //             payer.pubkey(),
    //             self.get_newest_slot_from_history().await,
    //         );
    //     self.process_transaction(&[instruction], Some(&[authority, payer]))
    //         .await
    //         .unwrap();
    //     alt_address
    // }

    pub async fn get_account_data(&self, address: Pubkey) -> Option<Vec<u8>> {
        Some(
            self.context
                .borrow_mut()
                .banks_client
                .get_account(address)
                .await
                .unwrap()?
                .data()
                .to_vec(),
        )
    }

    pub async fn get_account_opt<T: AccountDeserialize>(&self, address: Pubkey) -> Option<T> {
        let data = self.get_account_data(address).await?;
        let mut data_slice: &[u8] = &data;
        AccountDeserialize::try_deserialize(&mut data_slice).ok()
    }

    // Use when accounts are too big for the stack
    pub async fn get_account_boxed<T: AccountDeserialize>(&self, address: Pubkey) -> Box<T> {
        let data = self.get_account_data(address).await.unwrap();
        let mut data_slice: &[u8] = &data;
        Box::new(AccountDeserialize::try_deserialize(&mut data_slice).unwrap())
    }

    pub async fn get_account<T: AccountDeserialize>(&self, address: Pubkey) -> T {
        self.get_account_opt(address).await.unwrap()
    }

    /// Lamports of `address`, 0 if the account does not exist
    pub async fn get_lamports(&self, address: Pubkey) -> u64 {
        self.context
            .borrow_mut()
            .banks_client
            .get_balance(address)
            .await
            .unwrap()
    }

    pub async fn account_exists(&self, address: Pubkey) -> bool {
        self.context
            .borrow_mut()
            .banks_client
            .get_account(address)
            .await
            .unwrap()
            .is_some()
    }

    /// Overwrite the data of an existing account, keeping its size
    pub async fn set_account_data(&self, address: Pubkey, data: &[u8]) {
        let mut account = self
            .context
            .borrow_mut()
            .banks_client
            .get_account(address)
            .await
            .unwrap()
            .unwrap();
        account.data.copy_from_slice(data);
        self.context
            .borrow_mut()
            .set_account(&address, &account.into());
    }

    /// Overwrite the supply of a mint. Test mints start at 0 whatever the token accounts hold;
    /// a burn needs the supply to cover it like on a real chain.
    pub async fn set_mint_supply(&self, mint: Pubkey, supply: u64) {
        let mut account = self
            .context
            .borrow_mut()
            .banks_client
            .get_account(mint)
            .await
            .unwrap()
            .unwrap();
        let mut mint_data = spl_token::state::Mint::unpack(&account.data).unwrap();
        mint_data.supply = supply;
        spl_token::state::Mint::pack(mint_data, &mut account.data).unwrap();
        self.context
            .borrow_mut()
            .set_account(&mint, &account.into());
    }

    /// Overwrite the lamports of an existing account. The difference is taken from / given to
    /// the payer, the bank checks that the capitalization does not change.
    pub async fn set_lamports(&self, address: Pubkey, lamports: u64) {
        let payer = self.context.borrow().payer.pubkey();
        let mut account = self
            .context
            .borrow_mut()
            .banks_client
            .get_account(address)
            .await
            .unwrap()
            .unwrap();
        let mut payer_account = self
            .context
            .borrow_mut()
            .banks_client
            .get_account(payer)
            .await
            .unwrap()
            .unwrap();
        payer_account.lamports = payer_account.lamports + account.lamports - lamports;
        account.lamports = lamports;
        let mut context = self.context.borrow_mut();
        context.set_account(&address, &account.into());
        context.set_account(&payer, &payer_account.into());
    }

    /// Remove an account as if it had been closed, its lamports go to the payer
    pub async fn remove_account(&self, address: Pubkey) {
        self.set_lamports(address, 0).await;
        self.context
            .borrow_mut()
            .set_account(&address, &solana_sdk::account::AccountSharedData::default());
    }

    pub async fn token_account_balance(&self, address: Pubkey) -> u64 {
        self.get_account::<TokenAccount>(address).await.amount
    }

    pub async fn set_account_balance(&self, address: Pubkey, amount: u64) {
        let mut account = self
            .context
            .borrow_mut()
            .banks_client
            .get_account(address)
            .await
            .unwrap()
            .unwrap();

        let mut account_data = spl_token::state::Account::unpack(&account.data).unwrap();
        account_data.amount = amount;
        spl_token::state::Account::pack(account_data, &mut account.data).unwrap();

        self.context
            .borrow_mut()
            .set_account(&address, &account.into());
    }

    pub fn program_log(&self) -> Vec<String> {
        self.last_transaction_log.borrow().clone()
    }

    pub fn program_log_events<T: anchor_lang::Event + anchor_lang::AnchorDeserialize>(
        &self,
    ) -> Vec<T> {
        self.program_log()
            .iter()
            .filter_map(|data| {
                let bytes = base64::decode(data).ok()?;
                if bytes[0..8] != T::discriminator() {
                    return None;
                }
                T::try_from_slice(&bytes[8..]).ok()
            })
            .collect()
    }
}
