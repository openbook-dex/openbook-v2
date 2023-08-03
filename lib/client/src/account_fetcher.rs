use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use async_once_cell::unpin::Lazy;

use anyhow::Context;

use anchor_client::ClientError;
use anchor_lang::AccountDeserialize;

use solana_client::nonblocking::rpc_client::RpcClient as RpcClientAsync;
use solana_sdk::account::{AccountSharedData, ReadableAccount};
use solana_sdk::pubkey::Pubkey;

use openbook_v2::state::OpenOrdersAccount;

#[async_trait::async_trait]
pub trait AccountFetcher: Sync + Send {
    async fn fetch_raw_account(&self, address: &Pubkey) -> anyhow::Result<AccountSharedData>;
    async fn fetch_raw_account_lookup_table(
        &self,
        address: &Pubkey,
    ) -> anyhow::Result<AccountSharedData> {
        self.fetch_raw_account(address).await
    }
    async fn fetch_program_accounts(
        &self,
        program: &Pubkey,
        discriminator: [u8; 8],
    ) -> anyhow::Result<Vec<(Pubkey, AccountSharedData)>>;
}

// Can't be in the trait, since then it would no longer be object-safe...
pub async fn account_fetcher_fetch_anchor_account<T: AccountDeserialize>(
    fetcher: &dyn AccountFetcher,
    address: &Pubkey,
) -> anyhow::Result<T> {
    let account = fetcher.fetch_raw_account(address).await?;
    let mut data: &[u8] = account.data();
    T::try_deserialize(&mut data)
        .with_context(|| format!("deserializing anchor account {}", address))
}

// Can't be in the trait, since then it would no longer be object-safe...
pub async fn account_fetcher_fetch_openorders_account(
    fetcher: &dyn AccountFetcher,
    address: &Pubkey,
) -> anyhow::Result<OpenOrdersAccount> {
    let account = fetcher.fetch_raw_account(address).await?;
    let mut data: &[u8] = account.data();
    OpenOrdersAccount::try_deserialize(&mut data)
        .with_context(|| format!("deserializing open orders account {}", address))
}

pub struct RpcAccountFetcher {
    pub rpc: RpcClientAsync,
}

#[async_trait::async_trait]
impl AccountFetcher for RpcAccountFetcher {
    async fn fetch_raw_account(&self, address: &Pubkey) -> anyhow::Result<AccountSharedData> {
        self.rpc
            .get_account_with_commitment(address, self.rpc.commitment())
            .await
            .with_context(|| format!("fetch account {}", *address))?
            .value
            .ok_or(ClientError::AccountNotFound)
            .with_context(|| format!("fetch account {}", *address))
            .map(Into::into)
    }

    async fn fetch_program_accounts(
        &self,
        program: &Pubkey,
        discriminator: [u8; 8],
    ) -> anyhow::Result<Vec<(Pubkey, AccountSharedData)>> {
        use solana_account_decoder::UiAccountEncoding;
        use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
        use solana_client::rpc_filter::{Memcmp, RpcFilterType};
        let config = RpcProgramAccountsConfig {
            filters: Some(vec![RpcFilterType::Memcmp(Memcmp::new_raw_bytes(
                0,
                discriminator.to_vec(),
            ))]),
            account_config: RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::Base64),
                commitment: Some(self.rpc.commitment()),
                ..RpcAccountInfoConfig::default()
            },
            with_context: Some(true),
        };
        let accs = self
            .rpc
            .get_program_accounts_with_config(program, config)
            .await?;
        // convert Account -> AccountSharedData
        Ok(accs
            .into_iter()
            .map(|(pk, acc)| (pk, acc.into()))
            .collect::<Vec<_>>())
    }
}

struct CoalescedAsyncJob<Key, Output> {
    jobs: HashMap<Key, Arc<Lazy<Output>>>,
}

impl<Key, Output> Default for CoalescedAsyncJob<Key, Output> {
    fn default() -> Self {
        Self {
            jobs: Default::default(),
        }
    }
}

impl<Key: std::cmp::Eq + std::hash::Hash, Output: 'static> CoalescedAsyncJob<Key, Output> {
    /// Either returns the job for `key` or registers a new job for it
    fn run_coalesced<F: std::future::Future<Output = Output> + Send + 'static>(
        &mut self,
        key: Key,
        fut: F,
    ) -> Arc<Lazy<Output>> {
        self.jobs
            .entry(key)
            .or_insert_with(|| Arc::new(Lazy::new(Box::pin(fut))))
            .clone()
    }

    fn remove(&mut self, key: &Key) {
        self.jobs.remove(key);
    }
}

#[derive(Default)]
#[allow(clippy::type_complexity)]
struct AccountCache {
    accounts: HashMap<Pubkey, AccountSharedData>,
    keys_for_program_and_discriminator: HashMap<(Pubkey, [u8; 8]), Vec<Pubkey>>,

    account_jobs: CoalescedAsyncJob<Pubkey, anyhow::Result<AccountSharedData>>,
    program_accounts_jobs:
        CoalescedAsyncJob<(Pubkey, [u8; 8]), anyhow::Result<Vec<(Pubkey, AccountSharedData)>>>,
}

impl AccountCache {
    fn clear(&mut self) {
        self.accounts.clear();
        self.keys_for_program_and_discriminator.clear();
    }
}

pub struct CachedAccountFetcher<T: AccountFetcher> {
    fetcher: Arc<T>,
    cache: Arc<Mutex<AccountCache>>,
}

impl<T: AccountFetcher> Clone for CachedAccountFetcher<T> {
    fn clone(&self) -> Self {
        Self {
            fetcher: self.fetcher.clone(),
            cache: self.cache.clone(),
        }
    }
}

impl<T: AccountFetcher> CachedAccountFetcher<T> {
    pub fn new(fetcher: Arc<T>) -> Self {
        Self {
            fetcher,
            cache: Arc::new(Mutex::new(AccountCache::default())),
        }
    }

    pub fn clear_cache(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }
}

#[async_trait::async_trait]
impl<T: AccountFetcher + 'static> AccountFetcher for CachedAccountFetcher<T> {
    #[allow(clippy::clone_on_copy)]
    async fn fetch_raw_account(&self, address: &Pubkey) -> anyhow::Result<AccountSharedData> {
        let fetch_job = {
            let mut cache = self.cache.lock().unwrap();
            if let Some(acc) = cache.accounts.get(address) {
                return Ok(acc.clone());
            }

            // Start or fetch a reference to the fetch + cache update job
            let self_copy = self.clone();
            let address_copy = address.clone();
            cache.account_jobs.run_coalesced(*address, async move {
                let result = self_copy.fetcher.fetch_raw_account(&address_copy).await;
                let mut cache = self_copy.cache.lock().unwrap();

                // remove the job from the job list, so it can be redone if it errored
                cache.account_jobs.remove(&address_copy);

                // store a successful fetch
                if let Ok(account) = result.as_ref() {
                    cache.accounts.insert(address_copy, account.clone());
                }
                result
            })
        };

        match fetch_job.get().await {
            Ok(v) => Ok(v.clone()),
            // Can't clone the stored error, so need to stringize it
            Err(err) => Err(anyhow::format_err!(
                "fetch error in CachedAccountFetcher: {:?}",
                err
            )),
        }
    }

    #[allow(clippy::clone_on_copy)]
    async fn fetch_program_accounts(
        &self,
        program: &Pubkey,
        discriminator: [u8; 8],
    ) -> anyhow::Result<Vec<(Pubkey, AccountSharedData)>> {
        let cache_key = (*program, discriminator);
        let fetch_job = {
            let mut cache = self.cache.lock().unwrap();
            if let Some(accounts) = cache.keys_for_program_and_discriminator.get(&cache_key) {
                return Ok(accounts
                    .iter()
                    .map(|pk| (*pk, cache.accounts.get(pk).unwrap().clone()))
                    .collect::<Vec<_>>());
            }

            let self_copy = self.clone();
            let program_copy = program.clone();
            cache
                .program_accounts_jobs
                .run_coalesced(cache_key, async move {
                    let result = self_copy
                        .fetcher
                        .fetch_program_accounts(&program_copy, discriminator)
                        .await;
                    let mut cache = self_copy.cache.lock().unwrap();
                    cache.program_accounts_jobs.remove(&cache_key);
                    if let Ok(accounts) = result.as_ref() {
                        cache
                            .keys_for_program_and_discriminator
                            .insert(cache_key, accounts.iter().map(|(pk, _)| *pk).collect());
                        for (pk, acc) in accounts.iter() {
                            cache.accounts.insert(*pk, acc.clone());
                        }
                    }
                    result
                })
        };

        match fetch_job.get().await {
            Ok(v) => Ok(v.clone()),
            // Can't clone the stored error, so need to stringize it
            Err(err) => Err(anyhow::format_err!(
                "fetch error in CachedAccountFetcher: {:?}",
                err
            )),
        }
    }
}
