use jsonrpc_core_client::transports::http;

use openbook_v2::accounts_zerocopy::*;
use openbook_v2::state::{OpenOrdersAccountFixed, OpenOrdersAccountLoadedRef};
use solana_account_decoder::{UiAccount, UiAccountEncoding};
use solana_client::{
    rpc_config::{RpcAccountInfoConfig, RpcContextConfig, RpcProgramAccountsConfig},
    rpc_response::{OptionalContext, Response, RpcKeyedAccount},
};
use solana_rpc::rpc::{rpc_accounts::AccountsDataClient, rpc_minimal::MinimalClient};
use solana_sdk::{account::AccountSharedData, commitment_config::CommitmentConfig, pubkey::Pubkey};

use anyhow::Context;
use futures::{stream, StreamExt};
use log::*;
use std::str::FromStr;
use std::time::Duration;
use tokio::time;

use crate::account_update_stream::{AccountUpdate, Message};
use crate::AnyhowWrap;

pub fn is_openbook_account<'a>(
    account: &'a AccountSharedData,
    group_id: &Pubkey,
) -> Option<OpenOrdersAccountLoadedRef<'a>> {
    // check owner, discriminator
    let fixed = account.load::<OpenOrdersAccountFixed>().ok()?;

    let data = account.data();
    OpenOrdersAccountLoadedRef::from_bytes(&data[8..]).ok()
}

#[derive(Default)]
struct AccountSnapshot {
    accounts: Vec<AccountUpdate>,
}

impl AccountSnapshot {
    pub fn extend_from_gpa_rpc(
        &mut self,
        rpc: Response<Vec<RpcKeyedAccount>>,
    ) -> anyhow::Result<()> {
        self.accounts.reserve(rpc.value.len());
        for a in rpc.value {
            self.accounts.push(AccountUpdate {
                slot: rpc.context.slot,
                pubkey: Pubkey::from_str(&a.pubkey).unwrap(),
                account: a
                    .account
                    .decode()
                    .ok_or_else(|| anyhow::anyhow!("could not decode account"))?,
            });
        }
        Ok(())
    }

    pub fn extend_from_gma_rpc(
        &mut self,
        keys: &[Pubkey],
        rpc: Response<Vec<Option<UiAccount>>>,
    ) -> anyhow::Result<()> {
        self.accounts.reserve(rpc.value.len());
        for (&pubkey, a) in keys.iter().zip(rpc.value.iter()) {
            if let Some(ui_account) = a {
                self.accounts.push(AccountUpdate {
                    slot: rpc.context.slot,
                    pubkey,
                    account: ui_account
                        .decode()
                        .ok_or_else(|| anyhow::anyhow!("could not decode account"))?,
                });
            }
        }
        Ok(())
    }
}

pub struct Config {
    pub rpc_http_url: String,
    pub openbook_group: Pubkey,
    pub get_multiple_accounts_count: usize,
    pub parallel_rpc_requests: usize,
    pub snapshot_interval: Duration,
    pub min_slot: u64,
}

async fn feed_snapshots(
    config: &Config,
    openbook_oracles: Vec<Pubkey>,
    sender: &async_channel::Sender<Message>,
) -> anyhow::Result<()> {
    let rpc_client = http::connect_with_options::<AccountsDataClient>(&config.rpc_http_url, true)
        .await
        .map_err_anyhow()?;

    let account_info_config = RpcAccountInfoConfig {
        encoding: Some(UiAccountEncoding::Base64),
        commitment: Some(CommitmentConfig::finalized()),
        data_slice: None,
        min_context_slot: Some(config.min_slot),
    };
    let all_accounts_config = RpcProgramAccountsConfig {
        filters: None,
        with_context: Some(true),
        account_config: account_info_config.clone(),
    };

    // TODO: This way the snapshots are done sequentially, and a failing snapshot prohibits the second one to be attempted

    let mut snapshot = AccountSnapshot::default();

    // Get all accounts of the openorders program
    let response = rpc_client
        .get_program_accounts(
            openbook_v2::id().to_string(),
            Some(all_accounts_config.clone()),
        )
        .await
        .map_err_anyhow()
        .context("error during getProgamAccounts for openbook program")?;
    if let OptionalContext::Context(account_snapshot_response) = response {
        snapshot.extend_from_gpa_rpc(account_snapshot_response)?;
    } else {
        anyhow::bail!("did not receive context");
    }

    // Get all the pyth oracles
    let results: Vec<(
        Vec<Pubkey>,
        Result<Response<Vec<Option<UiAccount>>>, jsonrpc_core_client::RpcError>,
    )> = stream::iter(openbook_oracles)
        .chunks(config.get_multiple_accounts_count)
        .map(|keys| {
            let rpc_client = &rpc_client;
            let account_info_config = account_info_config.clone();
            async move {
                let string_keys = keys.iter().map(|k| k.to_string()).collect::<Vec<_>>();
                (
                    keys,
                    rpc_client
                        .get_multiple_accounts(string_keys, Some(account_info_config))
                        .await,
                )
            }
        })
        .buffer_unordered(config.parallel_rpc_requests)
        .collect::<Vec<_>>()
        .await;
    for (keys, result) in results {
        snapshot.extend_from_gma_rpc(
            &keys,
            result
                .map_err_anyhow()
                .context("error during getMultipleAccounts for Pyth Oracles")?,
        )?;
    }

    // Get all the active open orders account keys
    let oo_account_pubkeys = snapshot
        .accounts
        .iter()
        .filter_map(|update| is_openbook_account(&update.account, &config.openbook_group))
        .collect::<Vec<Pubkey>>();

    // Retrieve all the open orders accounts
    let results: Vec<(
        Vec<Pubkey>,
        Result<Response<Vec<Option<UiAccount>>>, jsonrpc_core_client::RpcError>,
    )> = stream::iter(oo_account_pubkeys)
        .chunks(config.get_multiple_accounts_count)
        .map(|keys| {
            let rpc_client = &rpc_client;
            let account_info_config = account_info_config.clone();
            async move {
                let string_keys = keys.iter().map(|k| k.to_string()).collect::<Vec<_>>();
                (
                    keys,
                    rpc_client
                        .get_multiple_accounts(string_keys, Some(account_info_config))
                        .await,
                )
            }
        })
        .buffer_unordered(config.parallel_rpc_requests)
        .collect::<Vec<_>>()
        .await;
    for (keys, result) in results {
        snapshot.extend_from_gma_rpc(
            &keys,
            result
                .map_err_anyhow()
                .context("error during getMultipleAccounts for OpenOrders accounts")?,
        )?;
    }

    sender
        .send(Message::Snapshot(snapshot.accounts))
        .await
        .expect("sending must succeed");
    Ok(())
}

pub fn start(
    config: Config,
    openbook_oracles: Vec<Pubkey>,
    sender: async_channel::Sender<Message>,
) {
    let mut poll_wait_first_snapshot = time::interval(time::Duration::from_secs(2));
    let mut interval_between_snapshots = time::interval(config.snapshot_interval);

    tokio::spawn(async move {
        let rpc_client = http::connect_with_options::<MinimalClient>(&config.rpc_http_url, true)
            .await
            .expect("always Ok");

        // Wait for slot to exceed min_slot
        loop {
            poll_wait_first_snapshot.tick().await;

            let epoch_info = rpc_client
                .get_epoch_info(Some(RpcContextConfig {
                    commitment: Some(CommitmentConfig::finalized()),
                    min_context_slot: None,
                }))
                .await
                .expect("always Ok");
            log::debug!("latest slot for snapshot {}", epoch_info.absolute_slot);

            if epoch_info.absolute_slot > config.min_slot {
                log::debug!("continuing to fetch snapshot now, min_slot {} is older than latest epoch slot {}", config.min_slot, epoch_info.absolute_slot);
                break;
            }
        }

        loop {
            interval_between_snapshots.tick().await;
            if let Err(err) = feed_snapshots(&config, openbook_oracles.clone(), &sender).await {
                warn!("snapshot error: {:?}", err);
            } else {
                info!("snapshot success");
            };
        }
    });
}
