use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use anchor_client::{ClientError, Cluster};

use anchor_lang::__private::bytemuck;
use anchor_lang::prelude::System;
use anchor_lang::{AccountDeserialize, Id};
use anchor_spl::associated_token::get_associated_token_address;
use anchor_spl::token::Token;

use bincode::Options;
use fixed::types::I80F48;
use futures::{stream, StreamExt, TryStreamExt};
use itertools::Itertools;

use openbook_v2::state::{MarketIndex, OpenOrdersAccountValue, PlaceOrderType, Side, TokenIndex};

use solana_address_lookup_table_program::state::AddressLookupTable;
use solana_client::nonblocking::rpc_client::RpcClient as RpcClientAsync;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::address_lookup_table_account::AddressLookupTableAccount;
use solana_sdk::commitment_config::CommitmentLevel;
use solana_sdk::hash::Hash;
use solana_sdk::signer::keypair;
use solana_sdk::transaction::TransactionError;

use crate::account_fetcher::*;
use crate::context::{OpenBookContext, TokenContext};
use crate::gpa::{fetch_anchor_account, fetch_openbook_accounts};
use crate::jupiter;

use anyhow::Context;
use solana_sdk::account::ReadableAccount;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::signature::{Keypair, Signature};
use solana_sdk::sysvar;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signer::Signer};

// very close to anchor_client::Client, which unfortunately has no accessors or Clone
#[derive(Clone, Debug)]
pub struct Client {
    pub cluster: Cluster,
    pub fee_payer: Arc<Keypair>,
    pub commitment: CommitmentConfig,
    pub timeout: Option<Duration>,
    pub transaction_builder_config: TransactionBuilderConfig,
    pub rpc_send_transaction_config: RpcSendTransactionConfig,
}

impl Client {
    pub fn new(
        cluster: Cluster,
        commitment: CommitmentConfig,
        fee_payer: Arc<Keypair>,
        timeout: Option<Duration>,
        transaction_builder_config: TransactionBuilderConfig,
    ) -> Self {
        Self {
            cluster,
            fee_payer,
            commitment,
            timeout,
            transaction_builder_config,
            rpc_send_transaction_config: RpcSendTransactionConfig {
                preflight_commitment: Some(CommitmentLevel::Processed),
                ..Default::default()
            },
        }
    }

    pub fn rpc_async(&self) -> RpcClientAsync {
        let url = self.cluster.url().to_string();
        if let Some(timeout) = self.timeout.as_ref() {
            RpcClientAsync::new_with_timeout_and_commitment(url, *timeout, self.commitment)
        } else {
            RpcClientAsync::new_with_commitment(url, self.commitment)
        }
    }

    // TODO: this function here is awkward, since it (intentionally) doesn't use OpenBookClient::account_fetcher
    pub async fn rpc_anchor_account<T: AccountDeserialize>(
        &self,
        address: &Pubkey,
    ) -> anyhow::Result<T> {
        fetch_anchor_account(&self.rpc_async(), address).await
    }
}

// todo: might want to integrate geyser, websockets, or simple http polling for keeping data fresh
pub struct OpenBookClient {
    pub client: Client,

    // todo: possibly this object should have cache-functions, so there can be one getMultipleAccounts
    // call to refresh banks etc -- if it's backed by websockets, these could just do nothing
    pub account_fetcher: Arc<dyn AccountFetcher>,

    pub owner: Arc<Keypair>,
    pub open_orders_account: Pubkey,

    pub context: OpenBookContext,

    pub http_client: reqwest::Client,
}

// TODO: add retry framework for sending tx and rpc calls
// 1/ this works right now, but I think mid-term the OpenBookClient will want to interact with multiple openorders accounts
// -- then we should probably specify accounts by owner+account_num / or pubkey
// 2/ pubkey, can be both owned, but also delegated accouns

impl OpenBookClient {
    pub fn group_for_admin(admin: Pubkey, num: u32) -> Pubkey {
        Pubkey::find_program_address(
            &["Group".as_ref(), admin.as_ref(), num.to_le_bytes().as_ref()],
            &openbook_v2::ID,
        )
        .0
    }

    pub async fn find_accounts(
        client: &Client,
        owner: &Keypair,
    ) -> anyhow::Result<Vec<(Pubkey, OpenOrdersAccountValue)>> {
        fetch_openbook_accounts(&client.rpc_async(), openbook_v2::ID, owner.pubkey()).await
    }

    pub async fn find_or_create_account(
        client: &Client,
        owner: &Keypair,
        payer: &Keypair, // pays the SOL for the new account
        market: Pubkey,
        openbook_account_name: &str,
    ) -> anyhow::Result<Pubkey> {
        let rpc = client.rpc_async();
        let program = openbook_v2::ID;

        let mut openbook_account_tuples =
            fetch_openbook_accounts(&rpc, program, owner.pubkey()).await?;
        let openbook_account_opt = openbook_account_tuples
            .iter()
            .find(|(_, account)| account.fixed.name() == openbook_account_name);
        if openbook_account_opt.is_none() {
            openbook_account_tuples.sort_by(|a, b| {
                a.1.fixed
                    .account_num
                    .partial_cmp(&b.1.fixed.account_num)
                    .unwrap()
            });
            let account_num = match openbook_account_tuples.last() {
                Some(tuple) => tuple.1.fixed.account_num + 1,
                None => 0u32,
            };
            Self::init_open_orders(client, market, owner, payer, account_num)
                .await
                .context("Failed to create account...")?;
        }
        let openbook_account_tuples =
            fetch_openbook_accounts(&rpc, program, owner.pubkey()).await?;
        let index = openbook_account_tuples
            .iter()
            .position(|tuple| tuple.1.fixed.name() == openbook_account_name)
            .unwrap();
        Ok(openbook_account_tuples[index].0)
    }

    pub async fn init_open_orders(
        client: &Client,
        market: Pubkey,
        owner: &Keypair,
        payer: &Keypair, // pays the SOL for the new account
        account_num: u32,
    ) -> anyhow::Result<(Pubkey, Signature)> {
        let account = Pubkey::find_program_address(
            &[
                b"OpenOrdersAccount".as_ref(),
                owner.pubkey().as_ref(),
                market.as_ref(),
                &account_num.to_le_bytes(),
            ],
            &openbook_v2::id(),
        )
        .0;
        let ix = Instruction {
            program_id: openbook_v2::id(),
            accounts: anchor_lang::ToAccountMetas::to_account_metas(
                &openbook_v2::accounts::InitOpenOrders {
                    owner: owner.pubkey(),
                    open_orders_account: account,
                    market,
                    system_program: System::id(),
                },
                None,
            ),
            data: anchor_lang::InstructionData::data(&openbook_v2::instruction::InitOpenOrders {
                account_num,
                open_orders_count: 8,
            }),
        };

        let txsig = TransactionBuilder {
            instructions: vec![ix],
            address_lookup_tables: vec![],
            payer: payer.pubkey(),
            signers: vec![owner, payer],
            config: client.transaction_builder_config,
        }
        .send_and_confirm(&client)
        .await?;

        Ok((account, txsig))
    }

    /// Conveniently creates a RPC based client
    pub async fn new_for_existing_account(
        client: Client,
        account: Pubkey,
        owner: Arc<Keypair>,
    ) -> anyhow::Result<Self> {
        let rpc = client.rpc_async();
        let account_fetcher = Arc::new(CachedAccountFetcher::new(Arc::new(RpcAccountFetcher {
            rpc,
        })));
        let openbook_account =
            account_fetcher_fetch_openbook_account(&*account_fetcher, &account).await?;
        if openbook_account.fixed.owner != owner.pubkey() {
            anyhow::bail!(
                "bad owner for account: expected {} got {}",
                openbook_account.fixed.owner,
                owner.pubkey()
            );
        }

        let rpc = client.rpc_async();
        let openbook_context = OpenBookContext::new_from_rpc(&rpc).await?;

        Self::new_detail(client, account, owner, openbook_context, account_fetcher)
    }

    /// Allows control of AccountFetcher and externally created MangoGroupContext
    pub fn new_detail(
        client: Client,
        account: Pubkey,
        owner: Arc<Keypair>,
        // future: maybe pass Arc<MangoGroupContext>, so it can be extenally updated?
        openbook_context: OpenBookContext,
        account_fetcher: Arc<dyn AccountFetcher>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            client,
            account_fetcher,
            owner,
            open_orders_account: account,
            context: openbook_context,
            http_client: reqwest::Client::new(),
        })
    }

    pub fn owner(&self) -> Pubkey {
        self.owner.pubkey()
    }

    pub async fn openbook_account(&self) -> anyhow::Result<OpenOrdersAccountValue> {
        account_fetcher_fetch_openbook_account(&*self.account_fetcher, &self.open_orders_account)
            .await
    }

    // TODO Token withdraw
    // pub async fn token_withdraw(
    //     &self,
    //     mint: Pubkey,
    //     amount: u64,
    //     allow_borrow: bool,
    // ) -> anyhow::Result<Signature> {
    //     let token = self.context.token_by_mint(&mint)?;
    //     let token_index = token.token_index;
    //     let mint_info = token.mint_info;

    //     let ixs = vec![
    //         spl_associated_token_account::instruction::create_associated_token_account_idempotent(
    //             &self.owner(),
    //             &self.owner(),
    //             &mint,
    //             &Token::id(),
    //         ),
    //         Instruction {
    //             program_id: openbook_v2::id(),
    //             accounts: {
    //                 let mut ams = anchor_lang::ToAccountMetas::to_account_metas(
    //                     &openbook_v2::accounts::TokenWithdraw {
    //                         account: self.open_orders_account,
    //                         owner: self.owner(),
    //                         vault: mint_info.first_vault(),
    //                         oracle: mint_info.oracle,
    //                         token_account: get_associated_token_address(
    //                             &self.owner(),
    //                             &mint_info.mint,
    //                         ),
    //                         token_program: Token::id(),
    //                     },
    //                     None,
    //                 );
    //                 // ams.extend(health_check_metas.into_iter());
    //                 ams
    //             },
    //             data: anchor_lang::InstructionData::data(&openbook_v2::instruction::TokenWithdraw {
    //                 amount,
    //                 allow_borrow,
    //             }),
    //         },
    //     ];
    //     self.send_and_confirm_owner_tx(ixs).await
    // }

    pub async fn get_oracle_price(
        &self,
        token_name: &str,
    ) -> Result<pyth_sdk_solana::Price, anyhow::Error> {
        let token_index = *self.context.token_indexes_by_name.get(token_name).unwrap();
        let mint_info = self.context.mint_info(token_index);
        let oracle_account = self
            .account_fetcher
            .fetch_raw_account(&mint_info.oracle)
            .await?;
        Ok(pyth_sdk_solana::load_price(&oracle_account.data()).unwrap())
    }

    pub async fn place_order(
        &self,
        market_index: MarketIndex,
        side: Side,
        price_lots: i64,
        max_base_lots: i64,
        max_quote_lots: i64,
        client_order_id: u64,
        order_type: PlaceOrderType,
        reduce_only: bool,
        expiry_timestamp: u64,
        limit: u8,
        payer_acc: Pubkey,
        base_vault: Pubkey,
        quote_vault: Pubkey,
    ) -> anyhow::Result<Signature> {
        let perp = self.context.context(market_index);

        let ix = Instruction {
            program_id: openbook_v2::id(),
            accounts: {
                let ams = anchor_lang::ToAccountMetas::to_account_metas(
                    &openbook_v2::accounts::PlaceOrder {
                        open_orders_acc: self.open_orders_account,
                        owner: self.owner(),
                        market: perp.address,
                        bids: perp.market.bids,
                        asks: perp.market.asks,
                        event_queue: perp.market.event_queue,
                        oracle: perp.market.oracle,
                        payer_acc,
                        base_vault,
                        quote_vault,
                        system_program: System::id(),
                    },
                    None,
                );
                ams
            },
            data: anchor_lang::InstructionData::data(&openbook_v2::instruction::PlaceOrder {
                side,
                price_lots,
                max_base_lots,
                max_quote_lots,
                client_order_id,
                order_type,
                reduce_only,
                expiry_timestamp,
                limit,
            }),
        };
        self.send_and_confirm_owner_tx(vec![ix]).await
    }

    // pub async fn deactivate_position(
    //     &self,
    //     market_index: MarketIndex,
    // ) -> anyhow::Result<Signature> {
    //     let perp = self.context.context(market_index);

    //     let ix = Instruction {
    //         program_id: openbook_v2::id(),
    //         accounts: {
    //             let mut ams = anchor_lang::ToAccountMetas::to_account_metas(
    //                 &openbook_v2::accounts::DeactivatePosition {
    //                     account: self.open_orders_account,
    //                     owner: self.owner(),
    //                     market: perp.address,
    //                 },
    //                 None,
    //             );
    //             ams
    //         },
    //         data: anchor_lang::InstructionData::data(
    //             &openbook_v2::instruction::DeactivatePosition {},
    //         ),
    //     };
    //     self.send_and_confirm_owner_tx(vec![ix]).await
    // }

    pub async fn jupiter_route(
        &self,
        input_mint: Pubkey,
        output_mint: Pubkey,
        amount: u64,
        slippage: u64,
        swap_mode: JupiterSwapMode,
    ) -> anyhow::Result<jupiter::QueryRoute> {
        let quote = self
            .http_client
            .get("https://quote-api.jup.ag/v4/quote")
            .query(&[
                ("inputMint", input_mint.to_string()),
                ("outputMint", output_mint.to_string()),
                ("amount", format!("{}", amount)),
                ("onlyDirectRoutes", "true".into()),
                ("enforceSingleTx", "true".into()),
                ("filterTopNResult", "10".into()),
                ("slippageBps", format!("{}", slippage)),
                (
                    "swapMode",
                    match swap_mode {
                        JupiterSwapMode::ExactIn => "ExactIn",
                        JupiterSwapMode::ExactOut => "ExactOut",
                    }
                    .into(),
                ),
            ])
            .send()
            .await
            .context("quote request to jupiter")?
            .json::<jupiter::QueryResult>()
            .await
            .context("receiving json response from jupiter quote request")?;

        // Find the top route that doesn't involve Raydium (that has too many accounts)
        let route = quote
            .data
            .iter()
            .find(|route| {
                !route
                    .market_infos
                    .iter()
                    .any(|mi| mi.label.contains("Raydium"))
            })
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "no route for swap. found {} routes, but none were usable",
                    quote.data.len()
                )
            })?;

        Ok(route.clone())
    }

    // TODO Binye. This isn't needed cause it's a flash loan?
    // pub async fn jupiter_swap(
    //     &self,
    //     input_mint: Pubkey,
    //     output_mint: Pubkey,
    //     amount: u64,
    //     slippage: u64,
    //     swap_mode: JupiterSwapMode,
    // ) -> anyhow::Result<Signature> {
    //     let source_token = self.context.token_by_mint(&input_mint)?;
    //     let target_token = self.context.token_by_mint(&output_mint)?;
    //     let route = self
    //         .jupiter_route(input_mint, output_mint, amount, slippage, swap_mode)
    //         .await?;

    //     let swap = self
    //         .http_client
    //         .post("https://quote-api.jup.ag/v4/swap")
    //         .json(&jupiter::SwapRequest {
    //             route: route.clone(),
    //             user_public_key: self.owner.pubkey().to_string(),
    //             wrap_unwrap_sol: false,
    //         })
    //         .send()
    //         .await
    //         .context("swap transaction request to jupiter")?
    //         .json::<jupiter::SwapResponse>()
    //         .await
    //         .context("receiving json response from jupiter swap transaction request")?;

    //     if swap.setup_transaction.is_some() || swap.cleanup_transaction.is_some() {
    //         anyhow::bail!(
    //             "chosen jupiter route requires setup or cleanup transactions, can't execute"
    //         );
    //     }

    //     let jup_tx = bincode::options()
    //         .with_fixint_encoding()
    //         .reject_trailing_bytes()
    //         .deserialize::<solana_sdk::transaction::VersionedTransaction>(
    //             &base64::decode(&swap.swap_transaction)
    //                 .context("base64 decoding jupiter transaction")?,
    //         )
    //         .context("parsing jupiter transaction")?;
    //     let ata_program = anchor_spl::associated_token::ID;
    //     let token_program = anchor_spl::token::ID;
    //     let is_setup_ix = |k: Pubkey| -> bool { k == ata_program || k == token_program };
    //     let (jup_ixs, jup_alts) = self
    //         .deserialize_instructions_and_alts(&jup_tx.message)
    //         .await?;
    //     let filtered_jup_ix = jup_ixs
    //         .into_iter()
    //         .filter(|ix| !is_setup_ix(ix.program_id))
    //         .collect::<Vec<_>>();

    //     let bank_ams = [
    //         source_token.mint_info.first_bank(),
    //         target_token.mint_info.first_bank(),
    //     ]
    //     .into_iter()
    //     .map(to_writable_account_meta)
    //     .collect::<Vec<_>>();

    //     let vault_ams = [
    //         source_token.mint_info.first_vault(),
    //         target_token.mint_info.first_vault(),
    //     ]
    //     .into_iter()
    //     .map(to_writable_account_meta)
    //     .collect::<Vec<_>>();

    //     let token_ams = [source_token.mint_info.mint, target_token.mint_info.mint]
    //         .into_iter()
    //         .map(|mint| {
    //             to_writable_account_meta(
    //                 anchor_spl::associated_token::get_associated_token_address(
    //                     &self.owner(),
    //                     &mint,
    //                 ),
    //             )
    //         })
    //         .collect::<Vec<_>>();

    //     let loan_amounts = vec![
    //         match swap_mode {
    //             JupiterSwapMode::ExactIn => amount,
    //             // in amount + slippage
    //             JupiterSwapMode::ExactOut => u64::from_str(&route.other_amount_threshold).unwrap(),
    //         },
    //         0u64,
    //     ];

    //     let mut instructions = Vec::new();

    //     instructions.push(
    //         spl_associated_token_account::instruction::create_associated_token_account_idempotent(
    //             &self.owner.pubkey(),
    //             &self.owner.pubkey(),
    //             &source_token.mint_info.mint,
    //             &Token::id(),
    //         ),
    //     );
    //     instructions.push(
    //         spl_associated_token_account::instruction::create_associated_token_account_idempotent(
    //             &self.owner.pubkey(),
    //             &self.owner.pubkey(),
    //             &target_token.mint_info.mint,
    //             &Token::id(),
    //         ),
    //     );
    //     instructions.push(Instruction {
    //         program_id: openbook_v2::id(),
    //         accounts: {
    //             let mut ams = anchor_lang::ToAccountMetas::to_account_metas(
    //                 &openbook_v2::accounts::FlashLoanBegin {
    //                     account: self.open_orders_account,
    //                     owner: self.owner(),
    //                     token_program: Token::id(),
    //                     instructions: solana_sdk::sysvar::instructions::id(),
    //                 },
    //                 None,
    //             );
    //             ams.extend(bank_ams);
    //             ams.extend(vault_ams.clone());
    //             ams.extend(token_ams.clone());
    //             ams.push(to_readonly_account_meta(self.group()));
    //             ams
    //         },
    //         data: anchor_lang::InstructionData::data(&openbook_v2::instruction::FlashLoanBegin {
    //             loan_amounts,
    //         }),
    //     });
    //     for ix in filtered_jup_ix {
    //         instructions.push(ix.clone());
    //     }
    //     instructions.push(Instruction {
    //         program_id: openbook_v2::id(),
    //         accounts: {
    //             let mut ams = anchor_lang::ToAccountMetas::to_account_metas(
    //                 &openbook_v2::accounts::FlashLoanEnd {
    //                     account: self.open_orders_account,
    //                     owner: self.owner(),
    //                     token_program: Token::id(),
    //                 },
    //                 None,
    //             );
    //             ams.extend(vault_ams);
    //             ams.extend(token_ams);
    //             ams.push(to_readonly_account_meta(self.group()));
    //             ams
    //         },
    //         data: anchor_lang::InstructionData::data(&openbook_v2::instruction::FlashLoanEnd {
    //             flash_loan_type: openbook_v2::accounts_ix::FlashLoanType::Swap,
    //         }),
    //     });

    //     let payer = self.owner.pubkey(); // maybe use fee_payer? but usually it's the same
    //     let mut address_lookup_tables = self.openbook_address_lookup_tables().await?;
    //     address_lookup_tables.extend(jup_alts.into_iter());

    //     TransactionBuilder {
    //         instructions,
    //         address_lookup_tables,
    //         payer,
    //         signers: vec![&*self.owner],
    //         config: self.client.transaction_builder_config,
    //     }
    //     .send_and_confirm(&self.client)
    //     .await
    // }

    async fn fetch_address_lookup_table(
        &self,
        address: Pubkey,
    ) -> anyhow::Result<AddressLookupTableAccount> {
        let raw = self
            .account_fetcher
            .fetch_raw_account_lookup_table(&address)
            .await?;
        let data = AddressLookupTable::deserialize(&raw.data())?;
        Ok(AddressLookupTableAccount {
            key: address,
            addresses: data.addresses.to_vec(),
        })
    }

    pub async fn openbook_address_lookup_tables(
        &self,
    ) -> anyhow::Result<Vec<AddressLookupTableAccount>> {
        stream::iter(self.context.address_lookup_tables.iter())
            .then(|&k| self.fetch_address_lookup_table(k))
            .try_collect::<Vec<_>>()
            .await
    }

    async fn deserialize_instructions_and_alts(
        &self,
        message: &solana_sdk::message::VersionedMessage,
    ) -> anyhow::Result<(Vec<Instruction>, Vec<AddressLookupTableAccount>)> {
        let lookups = message.address_table_lookups().unwrap_or_default();
        let address_lookup_tables = stream::iter(lookups)
            .then(|a| self.fetch_address_lookup_table(a.account_key))
            .try_collect::<Vec<_>>()
            .await?;

        let mut account_keys = message.static_account_keys().to_vec();
        for (lookups, table) in lookups.iter().zip(address_lookup_tables.iter()) {
            account_keys.extend(
                lookups
                    .writable_indexes
                    .iter()
                    .map(|&index| table.addresses[index as usize]),
            );
        }
        for (lookups, table) in lookups.iter().zip(address_lookup_tables.iter()) {
            account_keys.extend(
                lookups
                    .readonly_indexes
                    .iter()
                    .map(|&index| table.addresses[index as usize]),
            );
        }

        let compiled_ix = message
            .instructions()
            .iter()
            .map(|ci| solana_sdk::instruction::Instruction {
                program_id: *ci.program_id(&account_keys),
                accounts: ci
                    .accounts
                    .iter()
                    .map(|&index| AccountMeta {
                        pubkey: account_keys[index as usize],
                        is_signer: message.is_signer(index.into()),
                        is_writable: message.is_maybe_writable(index.into()),
                    })
                    .collect(),
                data: ci.data.clone(),
            })
            .collect();

        Ok((compiled_ix, address_lookup_tables))
    }

    pub async fn send_and_confirm_owner_tx(
        &self,
        instructions: Vec<Instruction>,
    ) -> anyhow::Result<Signature> {
        TransactionBuilder {
            instructions,
            address_lookup_tables: vec![],
            payer: self.client.fee_payer.pubkey(),
            signers: vec![&*self.owner, &*self.client.fee_payer],
            config: self.client.transaction_builder_config,
        }
        .send_and_confirm(&self.client)
        .await
    }

    pub async fn send_and_confirm_permissionless_tx(
        &self,
        instructions: Vec<Instruction>,
    ) -> anyhow::Result<Signature> {
        TransactionBuilder {
            instructions,
            address_lookup_tables: vec![],
            payer: self.client.fee_payer.pubkey(),
            signers: vec![&*self.client.fee_payer],
            config: self.client.transaction_builder_config,
        }
        .send_and_confirm(&self.client)
        .await
    }
}

#[derive(Debug, thiserror::Error)]
pub enum OpenBookClientError {
    #[error("Transaction simulation error. Error: {err:?}, Logs: {}",
        .logs.iter().join("; ")
    )]
    SendTransactionPreflightFailure {
        err: Option<TransactionError>,
        logs: Vec<String>,
    },
}

#[derive(Copy, Clone, Debug)]
pub struct TransactionBuilderConfig {
    // adds a SetComputeUnitPrice instruction in front
    pub prioritization_micro_lamports: Option<u64>,
}

pub struct TransactionBuilder<'a> {
    pub instructions: Vec<Instruction>,
    pub address_lookup_tables: Vec<AddressLookupTableAccount>,
    pub signers: Vec<&'a Keypair>,
    pub payer: Pubkey,
    pub config: TransactionBuilderConfig,
}

impl<'a> TransactionBuilder<'a> {
    pub async fn transaction(
        self,
        rpc: &RpcClientAsync,
    ) -> anyhow::Result<solana_sdk::transaction::VersionedTransaction> {
        let latest_blockhash = rpc.get_latest_blockhash().await?;
        self.transaction_with_blockhash(latest_blockhash)
    }

    pub fn transaction_with_blockhash(
        mut self,
        blockhash: Hash,
    ) -> anyhow::Result<solana_sdk::transaction::VersionedTransaction> {
        if let Some(prio_price) = self.config.prioritization_micro_lamports {
            self.instructions.insert(
                0,
                solana_sdk::compute_budget::ComputeBudgetInstruction::set_compute_unit_price(
                    prio_price,
                ),
            )
        }
        let v0_message = solana_sdk::message::v0::Message::try_compile(
            &self.payer,
            &self.instructions,
            &self.address_lookup_tables,
            blockhash,
        )?;
        let versioned_message = solana_sdk::message::VersionedMessage::V0(v0_message);
        let signers = self
            .signers
            .into_iter()
            .unique_by(|s| s.pubkey())
            .collect::<Vec<_>>();
        let tx =
            solana_sdk::transaction::VersionedTransaction::try_new(versioned_message, &signers)?;
        Ok(tx)
    }

    // These two send() functions don't really belong into the transaction builder!

    pub async fn send(self, client: &Client) -> anyhow::Result<Signature> {
        let rpc = client.rpc_async();
        let tx = self.transaction(&rpc).await?;
        rpc.send_transaction_with_config(&tx, client.rpc_send_transaction_config)
            .await
            .map_err(prettify_solana_client_error)
    }

    pub async fn send_and_confirm(self, client: &Client) -> anyhow::Result<Signature> {
        let rpc = client.rpc_async();
        let tx = self.transaction(&rpc).await?;
        // TODO: Wish we could use client.rpc_send_transaction_config here too!
        rpc.send_and_confirm_transaction(&tx)
            .await
            .map_err(prettify_solana_client_error)
    }
}

/// Do some manual unpacking on some ClientErrors
///
/// Unfortunately solana's RpcResponseError will very unhelpfully print [N log messages]
/// instead of showing the actual log messages. This unpacks the error to provide more useful
/// output.
pub fn prettify_client_error(err: anchor_client::ClientError) -> anyhow::Error {
    match err {
        anchor_client::ClientError::SolanaClientError(c) => prettify_solana_client_error(c),
        _ => err.into(),
    }
}

pub fn prettify_solana_client_error(
    err: solana_client::client_error::ClientError,
) -> anyhow::Error {
    use solana_client::client_error::ClientErrorKind;
    use solana_client::rpc_request::{RpcError, RpcResponseErrorData};
    match err.kind() {
        ClientErrorKind::RpcError(RpcError::RpcResponseError { data, .. }) => match data {
            RpcResponseErrorData::SendTransactionPreflightFailure(s) => {
                return OpenBookClientError::SendTransactionPreflightFailure {
                    err: s.err.clone(),
                    logs: s.logs.clone().unwrap_or_default(),
                }
                .into();
            }
            _ => {}
        },
        _ => {}
    };
    err.into()
}

#[derive(Clone, Copy)]
pub enum JupiterSwapMode {
    ExactIn,
    ExactOut,
}

pub fn keypair_from_cli(keypair: &str) -> Keypair {
    let maybe_keypair = keypair::read_keypair(&mut keypair.as_bytes());
    match maybe_keypair {
        Ok(keypair) => keypair,
        Err(_) => {
            let path = std::path::PathBuf::from_str(&*shellexpand::tilde(keypair)).unwrap();
            keypair::read_keypair_file(path)
                .unwrap_or_else(|_| panic!("Failed to read keypair from {}", keypair))
        }
    }
}

pub fn pubkey_from_cli(pubkey: &str) -> Pubkey {
    match Pubkey::from_str(pubkey) {
        Ok(p) => p,
        Err(_) => keypair_from_cli(pubkey).pubkey(),
    }
}

fn to_readonly_account_meta(pubkey: Pubkey) -> AccountMeta {
    AccountMeta {
        pubkey,
        is_writable: false,
        is_signer: false,
    }
}

fn to_writable_account_meta(pubkey: Pubkey) -> AccountMeta {
    AccountMeta {
        pubkey,
        is_writable: true,
        is_signer: false,
    }
}
