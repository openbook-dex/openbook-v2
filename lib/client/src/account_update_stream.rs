use solana_client::rpc_response::{Response, RpcKeyedAccount};
use solana_sdk::{account::AccountSharedData, pubkey::Pubkey};

use log::*;
use std::{str::FromStr, sync::Arc};

use crate::chain_data;

#[derive(Clone)]
pub struct AccountUpdate {
    pub pubkey: Pubkey,
    pub slot: u64,
    pub account: AccountSharedData,
}

impl AccountUpdate {
    pub fn from_rpc(rpc: Response<RpcKeyedAccount>) -> anyhow::Result<Self> {
        let pubkey = Pubkey::from_str(&rpc.value.pubkey)?;
        let account = rpc
            .value
            .account
            .decode()
            .ok_or_else(|| anyhow::anyhow!("could not decode account"))?;
        Ok(AccountUpdate {
            pubkey,
            slot: rpc.context.slot,
            account,
        })
    }
}

#[derive(Clone)]
pub enum Message {
    Account(AccountUpdate),
    Snapshot(Vec<AccountUpdate>),
    Slot(Arc<solana_client::rpc_response::SlotUpdate>),
}

impl Message {
    pub fn update_chain_data(&self, chain: &mut chain_data::ChainData) {
        use chain_data::*;
        match self {
            Message::Account(account_write) => {
                trace!("websocket account message");
                chain.update_account(
                    account_write.pubkey,
                    AccountAndSlot {
                        slot: account_write.slot,
                        account: account_write.account.clone(),
                    },
                );
            }
            Message::Snapshot(snapshot) => {
                for account_update in snapshot {
                    chain.update_account(
                        account_update.pubkey,
                        chain_data::AccountAndSlot {
                            slot: account_update.slot,
                            account: account_update.account.clone(),
                        },
                    );
                }
            }
            Message::Slot(slot_update) => {
                trace!("websocket slot message");
                let slot_update = match **slot_update {
                    solana_client::rpc_response::SlotUpdate::CreatedBank {
                        slot, parent, ..
                    } => Some(SlotData {
                        slot,
                        parent: Some(parent),
                        status: SlotStatus::Processed,
                        chain: 0,
                    }),
                    solana_client::rpc_response::SlotUpdate::OptimisticConfirmation {
                        slot,
                        ..
                    } => Some(SlotData {
                        slot,
                        parent: None,
                        status: SlotStatus::Confirmed,
                        chain: 0,
                    }),
                    solana_client::rpc_response::SlotUpdate::Root { slot, .. } => Some(SlotData {
                        slot,
                        parent: None,
                        status: SlotStatus::Rooted,
                        chain: 0,
                    }),
                    _ => None,
                };
                if let Some(update) = slot_update {
                    chain.update_slot(update);
                }
            }
        }
    }
}
