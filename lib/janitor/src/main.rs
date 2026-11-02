//! Winds down the sunset OpenBook v2 program.
//!
//! Order of operations: crank -> close-open-orders -> close-indexers -> sweep-fees ->
//! close-markets, or all of them with `wind-down`. Every step re-reads the chain and only acts on
//! what is still there, so it can be re-run at any time.

use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

use anchor_lang::{Owner, ZeroCopy};
use anchor_spl::token::spl_token;
use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use itertools::Itertools;
use openbook_v2::state::{
    janitor_market_empty, orphan_fee_roundoff, BookSide, EventHeap, Market, OpenOrdersAccount,
    OpenOrdersIndexer, JANITOR,
};
use openbook_v2_client::gpa::fetch_anchor_accounts;
use openbook_v2_client::janitor::*;
use serde_json::json;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::instruction::Instruction;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{read_keypair_file, Keypair, Signer};
use solana_sdk::transaction::Transaction;
use solana_transaction_status::option_serializer::OptionSerializer;
use solana_transaction_status::UiTransactionEncoding;

/// Events consumed per consume_events instruction
const EVENTS_PER_IX: usize = 8;
/// consume_events instructions per transaction
const CRANK_IXS_PER_TX: usize = 3;
/// Limit for the simulation that measures a transaction, the most a transaction may use
const MAX_COMPUTE_UNIT_LIMIT: u32 = 1_400_000;

/// Compute unit limit for a transaction that consumed `units` in simulation, with headroom for
/// state that changes before it lands (an ATA created in between, another penalty branch)
fn compute_unit_limit(units: u64) -> u32 {
    (units + units / 10 + 1_000).min(MAX_COMPUTE_UNIT_LIMIT as u64) as u32
}

#[derive(Parser)]
struct Cli {
    #[clap(long, env = "RPC_URL", default_value = "http://127.0.0.1:8899")]
    rpc_url: String,
    #[clap(long, default_value = "janitor.json")]
    keypair: PathBuf,
    /// Simulate the transactions instead of sending them
    #[clap(long)]
    dry_run: bool,
    /// Priority fee in micro lamports per compute unit
    #[clap(long, default_value_t = 0)]
    priority_fee: u64,
    /// Open orders accounts closed per transaction
    #[clap(long, default_value_t = 1)]
    batch: usize,
    /// RPC with the full transaction history, to find the owners of empty indexers (default
    /// --rpc-url)
    #[clap(long, env = "HISTORY_RPC_URL")]
    history_rpc_url: Option<String>,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Count what is left and write it as JSON
    Survey {
        #[clap(long, default_value = "survey.json")]
        out: PathBuf,
    },
    /// Consume the events of every market with a non-empty event heap
    Crank,
    /// Cancel, settle and close every open orders account
    CloseOpenOrders,
    /// Close every empty open orders indexer into its owner
    CloseIndexers,
    /// Sweep fees into the collect_fee_admin's associated token account
    SweepFees,
    /// Close every empty market into the escrow
    CloseMarkets,
    /// Survey again and compare with an earlier survey
    Report {
        #[clap(long, default_value = "survey.json")]
        previous: PathBuf,
        #[clap(long, default_value = "report.json")]
        out: PathBuf,
    },
    /// All steps in order
    WindDown,
}

struct Janitor {
    rpc: RpcClient,
    history_rpc: RpcClient,
    keypair: Keypair,
    dry_run: bool,
    priority_fee: u64,
    batch: usize,
}

struct MarketState {
    market: Market,
    event_heap: Box<EventHeap>,
    book_empty: bool,
    base_vault_amount: u64,
    quote_vault_amount: u64,
    lamports: u64,
}

impl MarketState {
    /// Open orders accounts still have claims to settle (beyond the orphan fee roundoff)
    fn has_deposits(&self) -> bool {
        self.market.base_deposit_total != 0
            || self.market.quote_deposit_total
                > self.market.fees_available + orphan_fee_roundoff(&self.market)
    }

    fn closable(&self) -> bool {
        self.event_heap.is_empty() && self.book_empty && janitor_market_empty(&self.market)
    }

    /// Tokens in the vaults beyond the deposit totals (dust, donations), burned on close
    fn vault_leftovers(&self) -> (u64, u64) {
        (
            self.base_vault_amount
                .saturating_sub(self.market.base_deposit_total),
            self.quote_vault_amount
                .saturating_sub(self.market.quote_deposit_total),
        )
    }
}

struct State {
    markets: BTreeMap<Pubkey, MarketState>,
    /// Markets whose heap, book or vaults could not be read, with the reason. Nothing is done for
    /// them or for their open orders accounts; they are not orphaned, the market account exists.
    unreadable_markets: BTreeMap<Pubkey, String>,
    open_orders_accounts: Vec<(Pubkey, OpenOrdersAccount, u64)>,
    indexers: Vec<(Pubkey, OpenOrdersIndexer, u64)>,
}

impl State {
    fn market(&self, key: &Pubkey) -> Option<&Market> {
        self.markets.get(key).map(|m| &m.market)
    }

    fn open_orders_account_keys(&self) -> HashSet<Pubkey> {
        self.open_orders_accounts.iter().map(|(k, ..)| *k).collect()
    }
}

impl Janitor {
    async fn fetch_raw(
        &self,
        keys: &[Pubkey],
    ) -> Result<Vec<Option<solana_sdk::account::Account>>> {
        let mut accounts = vec![];
        for chunk in keys.chunks(100) {
            accounts.extend(self.rpc.get_multiple_accounts(chunk).await?);
        }
        Ok(accounts)
    }

    async fn fetch_state(&self) -> Result<State> {
        let program = openbook_v2::id();
        let markets = fetch_anchor_accounts::<Market>(&self.rpc, program).await?;
        let open_orders_accounts =
            fetch_anchor_accounts::<OpenOrdersAccount>(&self.rpc, program).await?;
        let indexers = fetch_anchor_accounts::<OpenOrdersIndexer>(&self.rpc, program).await?;

        let mut keys = vec![];
        for (key, market, _) in &markets {
            keys.extend([
                *key,
                market.event_heap,
                market.bids,
                market.asks,
                market.market_base_vault,
                market.market_quote_vault,
            ]);
        }
        let raw = self.fetch_raw(&keys).await?;
        let mut market_states = BTreeMap::new();
        let mut unreadable_markets = BTreeMap::new();
        for ((key, market, _), raw) in markets.into_iter().zip(raw.chunks(6)) {
            // one market with a damaged heap, book or vault must not hold up the others
            match market_state(market, raw) {
                Ok(state) => {
                    market_states.insert(key, state);
                }
                Err(err) => {
                    println!("market {key}: {err:#}, skipping it and its open orders accounts");
                    unreadable_markets.insert(key, format!("{err:#}"));
                }
            }
        }

        Ok(State {
            markets: market_states,
            unreadable_markets,
            open_orders_accounts,
            indexers,
        })
    }

    fn transaction(
        &self,
        compute_unit_limit: u32,
        instructions: &[Instruction],
        blockhash: solana_sdk::hash::Hash,
    ) -> Transaction {
        let mut all = vec![ComputeBudgetInstruction::set_compute_unit_limit(
            compute_unit_limit,
        )];
        if self.priority_fee > 0 {
            all.push(ComputeBudgetInstruction::set_compute_unit_price(
                self.priority_fee,
            ));
        }
        all.extend_from_slice(instructions);
        Transaction::new_signed_with_payer(
            &all,
            Some(&self.keypair.pubkey()),
            &[&self.keypair],
            blockhash,
        )
    }

    /// Simulate one transaction, paid and signed by the janitor, and send it with the compute
    /// unit limit set to what the simulation consumed plus headroom
    async fn send(&self, label: &str, instructions: Vec<Instruction>) -> Result<bool> {
        let blockhash = self.rpc.get_latest_blockhash().await?;
        let tx = self.transaction(MAX_COMPUTE_UNIT_LIMIT, &instructions, blockhash);
        let result = self.rpc.simulate_transaction(&tx).await?.value;
        if let Some(err) = result.err {
            println!("{label}: {err}");
            for log in result.logs.unwrap_or_default() {
                println!("    {log}");
            }
            return Ok(false);
        }
        let units = result
            .units_consumed
            .context("simulation did not report compute units")?;

        if self.dry_run {
            println!("{label}: ok ({units} CU)");
            return Ok(true);
        }

        let tx = self.transaction(compute_unit_limit(units), &instructions, blockhash);
        match self.rpc.send_and_confirm_transaction(&tx).await {
            Ok(signature) => {
                println!("{label}: {signature} ({units} CU)");
                Ok(true)
            }
            Err(err) => {
                println!("{label}: {err}");
                Ok(false)
            }
        }
    }

    /// Consume all events. Events of open orders accounts that no longer exist are dropped by the
    /// program when the janitor passes the closed account.
    async fn crank(&self, state: &State) -> Result<()> {
        let janitor = self.keypair.pubkey();
        let existing = state.open_orders_account_keys();
        for (key, m) in &state.markets {
            if m.event_heap.is_empty() {
                continue;
            }
            let events = m.event_heap.iter().map(|(event, _)| event).collect_vec();
            let closed = events
                .iter()
                .filter(|event| !existing.contains(&event_open_orders_account(event)))
                .count();
            if closed > 0 {
                println!("crank {key}: dropping {closed} events of closed open orders accounts");
            }
            let mut ixs = vec![];
            for chunk in events.chunks(EVENTS_PER_IX) {
                let accounts = chunk
                    .iter()
                    .map(|event| event_open_orders_account(event))
                    .unique()
                    .collect_vec();
                ixs.push(consume_events_ix(
                    janitor,
                    *key,
                    &m.market,
                    &accounts,
                    chunk.len(),
                ));
            }
            for (i, chunk) in ixs.chunks(CRANK_IXS_PER_TX).enumerate() {
                if !self
                    .send(&format!("crank {key} #{i}"), chunk.to_vec())
                    .await?
                {
                    break;
                }
            }
        }
        Ok(())
    }

    async fn close_open_orders(&self, state: &State) -> Result<()> {
        let janitor = self.keypair.pubkey();
        let with_events: HashSet<Pubkey> = state
            .markets
            .values()
            .flat_map(|m| event_heap_open_orders_accounts(&m.event_heap))
            .collect();
        let mut accounts = state.open_orders_accounts.iter().collect::<Vec<_>>();
        // shared vaults and mints keep batched transactions small
        accounts.sort_by_key(|(_, a, _)| a.market);
        for batch in accounts.chunks(self.batch) {
            let mut closes = vec![];
            for (key, account, _) in batch {
                if with_events.contains(key) {
                    println!("close {key}: unconsumed events, crank first");
                    continue;
                }
                if let Some(reason) = state.unreadable_markets.get(&account.market) {
                    println!(
                        "close {key}: market {} unreadable ({reason})",
                        account.market
                    );
                    continue;
                }
                let market = state.market(&account.market);
                let position = &account.position;
                let mut ixs = vec![];
                if let Some(market) = market {
                    if position.bids_base_lots != 0 || position.asks_base_lots != 0 {
                        ixs.push(cancel_all_orders_ix(
                            janitor,
                            account.market,
                            market,
                            *key,
                            None,
                            u8::MAX,
                        ));
                    }
                }
                ixs.push(close_open_orders_account_ix(janitor, *key, account, market));
                closes.push((*key, ixs));
            }
            if closes.is_empty() {
                continue;
            }
            let label = format!("close open orders {}", closes[0].0);
            let ixs = closes.iter().flat_map(|(_, ixs)| ixs.clone()).collect();
            // one account that can not be closed (e.g. its owner's ATA is unusable) must not
            // hold up the rest of the batch
            if !self.send(&label, ixs).await? && closes.len() > 1 {
                for (key, ixs) in closes {
                    self.send(&format!("close open orders {key}"), ixs).await?;
                }
            }
        }
        Ok(())
    }

    /// Close the indexers that were already empty into their owners. Indexers emptied by
    /// close-open-orders were closed there.
    async fn close_indexers(&self, state: &State) -> Result<()> {
        let janitor = self.keypair.pubkey();
        for (key, indexer, _) in &state.indexers {
            if !indexer.addresses.is_empty() {
                continue;
            }
            let owner = match self.indexer_owner(key).await {
                Ok(Some(owner)) => owner,
                Ok(None) => {
                    println!("close indexer {key}: owner not found, try a --history-rpc-url");
                    continue;
                }
                Err(err) => {
                    println!("close indexer {key}: looking up the owner: {err}");
                    continue;
                }
            };
            self.send(
                &format!("close indexer {key} into {owner}"),
                vec![close_open_orders_indexer_ix(janitor, owner)],
            )
            .await?;
        }
        Ok(())
    }

    /// An indexer does not store its owner. Every transaction that touched it has the owner
    /// among its accounts: the one whose indexer address it is.
    async fn indexer_owner(&self, indexer: &Pubkey) -> Result<Option<Pubkey>> {
        let mut before = None;
        loop {
            let page = self
                .history_rpc
                .get_signatures_for_address_with_config(
                    indexer,
                    GetConfirmedSignaturesForAddress2Config {
                        before,
                        until: None,
                        limit: Some(1000),
                        commitment: Some(CommitmentConfig::confirmed()),
                    },
                )
                .await?;
            for signature in &page {
                let tx = self
                    .history_rpc
                    .get_transaction_with_config(
                        &signature.signature.parse()?,
                        RpcTransactionConfig {
                            encoding: Some(UiTransactionEncoding::Base64),
                            commitment: Some(CommitmentConfig::confirmed()),
                            max_supported_transaction_version: Some(0),
                        },
                    )
                    .await?;
                let Some(transaction) = tx.transaction.transaction.decode() else {
                    continue;
                };
                let mut keys = transaction.message.static_account_keys().to_vec();
                if let Some(OptionSerializer::Some(loaded)) =
                    tx.transaction.meta.map(|meta| meta.loaded_addresses)
                {
                    for key in loaded.writable.iter().chain(&loaded.readonly) {
                        keys.push(key.parse()?);
                    }
                }
                if let Some(owner) = keys
                    .into_iter()
                    .find(|key| open_orders_indexer_address(key) == *indexer)
                {
                    return Ok(Some(owner));
                }
            }
            match page.last() {
                Some(last) if page.len() == 1000 => before = Some(last.signature.parse()?),
                _ => return Ok(None),
            }
        }
    }

    async fn sweep_fees(&self, state: &State) -> Result<()> {
        let janitor = self.keypair.pubkey();
        for (key, m) in &state.markets {
            if m.market.fees_available == 0 || m.has_deposits() {
                continue;
            }
            self.send(
                &format!("sweep fees {key}"),
                vec![
                    create_ata_idempotent_ix(
                        janitor,
                        m.market.collect_fee_admin,
                        m.market.quote_mint,
                    ),
                    sweep_fees_ix(janitor, *key, &m.market),
                ],
            )
            .await?;
        }
        Ok(())
    }

    async fn close_markets(&self, state: &State) -> Result<()> {
        let janitor = self.keypair.pubkey();
        for (key, m) in &state.markets {
            if !m.closable() {
                continue;
            }
            let (base, quote) = m.vault_leftovers();
            if base > 0 || quote > 0 {
                println!(
                    "close market {key}: burning {base} base and {quote} quote left in the vaults"
                );
            }
            self.send(
                &format!("close market {key}"),
                vec![close_market_ix(janitor, *key, &m.market)],
            )
            .await?;
        }
        Ok(())
    }

    async fn survey(&self) -> Result<serde_json::Value> {
        let state = self.fetch_state().await?;
        let markets = &state.markets;
        let ooas = &state.open_orders_accounts;
        let existing = state.open_orders_account_keys();
        let closed_account_events: usize = markets
            .values()
            .flat_map(|m| m.event_heap.iter())
            .filter(|(event, _)| !existing.contains(&event_open_orders_account(event)))
            .count();
        let count = |f: &dyn Fn(&MarketState) -> bool| markets.values().filter(|m| f(m)).count();
        let count_ooa =
            |f: &dyn Fn(&OpenOrdersAccount) -> bool| ooas.iter().filter(|(_, a, _)| f(a)).count();

        let janitor_lamports = self.rpc.get_balance(&self.keypair.pubkey()).await?;
        Ok(json!({
            "janitor_lamports": janitor_lamports,
            "rent": {
                "token_account": self.rpc
                    .get_minimum_balance_for_rent_exemption(spl_token::state::Account::LEN)
                    .await?,
                "empty_account": self.rpc.get_minimum_balance_for_rent_exemption(0).await?,
            },
            "markets": {
                "total": markets.len() + state.unreadable_markets.len(),
                "unreadable": state.unreadable_markets.len(),
                "closable": count(&|m| m.closable()),
                "event_heap_not_empty": count(&|m| !m.event_heap.is_empty()),
                "events": markets.values().map(|m| m.event_heap.len()).sum::<usize>(),
                "events_of_closed_accounts": closed_account_events,
                "orphaned_events": markets.values().map(|m| m.market.orphaned_events).sum::<u64>(),
                "book_not_empty": count(&|m| !m.book_empty),
                "with_deposits": count(&|m| m.has_deposits()),
                "fees_available": count(&|m| m.market.fees_available > 0),
                "vaults_with_leftovers": count(&|m| m.vault_leftovers() != (0, 0)),
                "lamports": markets.values().map(|m| m.lamports).sum::<u64>(),
            },
            "open_orders_accounts": {
                "total": ooas.len(),
                "with_orders": count_ooa(&|a| a.position.bids_base_lots != 0 || a.position.asks_base_lots != 0),
                "with_free_funds": count_ooa(&|a| a.position.base_free_native != 0 || a.position.quote_free_native != 0),
                "with_penalty": count_ooa(&|a| a.position.penalty_heap_count != 0),
                "orphaned": ooas.iter().filter(|(_, a, _)| !markets.contains_key(&a.market) && !state.unreadable_markets.contains_key(&a.market)).count(),
                "owners": ooas.iter().map(|(_, a, _)| a.owner).collect::<HashSet<_>>().len(),
                "lamports": ooas.iter().map(|(.., l)| l).sum::<u64>(),
            },
            "indexers": {
                "total": state.indexers.len(),
                "empty": state.indexers.iter().filter(|(_, i, _)| i.addresses.is_empty()).count(),
                "lamports": state.indexers.iter().map(|(.., l)| l).sum::<u64>(),
            },
        }))
    }
}

/// Like the program: some heaps and books on mainnet carry spare bytes after the struct, anchor's
/// `try_deserialize` rejects those
fn load_zero_copy<T: ZeroCopy + Owner>(data: &[u8]) -> Result<Box<T>> {
    let size = std::mem::size_of::<T>();
    if data.len() < 8 + size || data[..8] != T::discriminator() {
        bail!("not a {}", std::any::type_name::<T>());
    }
    Ok(Box::new(bytemuck::pod_read_unaligned(&data[8..8 + size])))
}

/// A market's heap, book and vaults decoded from `raw`: market, heap, bids, asks, base vault and
/// quote vault, in the order `fetch_state` requests them
fn market_state(
    market: Market,
    raw: &[Option<solana_sdk::account::Account>],
) -> Result<MarketState> {
    let [Some(market_account), Some(heap), Some(bids), Some(asks), base_vault, quote_vault] = raw
    else {
        bail!("heap or book account missing");
    };
    let bookside_empty =
        |data: &[u8]| -> Result<bool> { Ok(load_zero_copy::<BookSide>(data)?.is_empty()) };
    Ok(MarketState {
        market,
        event_heap: load_zero_copy::<EventHeap>(&heap.data).context("event heap")?,
        book_empty: bookside_empty(&bids.data).context("bids")?
            && bookside_empty(&asks.data).context("asks")?,
        base_vault_amount: token_amount(base_vault).context("base vault")?,
        quote_vault_amount: token_amount(quote_vault).context("quote vault")?,
        lamports: market_account.lamports
            + heap.lamports
            + bids.lamports
            + asks.lamports
            + base_vault.as_ref().map_or(0, |a| a.lamports)
            + quote_vault.as_ref().map_or(0, |a| a.lamports),
    })
}

/// Balance of a token account; a vault that is missing or no longer a token account is an error
fn token_amount(account: &Option<solana_sdk::account::Account>) -> Result<u64> {
    let account = account.as_ref().context("account missing")?;
    Ok(spl_token::state::Account::unpack(&account.data)?.amount)
}

/// Union of two surveys, numbers as (previous, now, difference)
fn diff(previous: &serde_json::Value, now: &serde_json::Value) -> serde_json::Value {
    match (previous, now) {
        (serde_json::Value::Object(p), serde_json::Value::Object(n)) => serde_json::Value::Object(
            n.iter()
                .map(|(k, v)| (k.clone(), diff(p.get(k).unwrap_or(&json!(null)), v)))
                .collect(),
        ),
        (p, n) if n.is_number() => {
            let (p, n) = (p.as_i64().unwrap_or(0), n.as_i64().unwrap_or(0));
            json!({ "previous": p, "now": n, "difference": n - p })
        }
        (_, n) => n.clone(),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let keypair = read_keypair_file(&cli.keypair)
        .map_err(|e| anyhow::anyhow!("reading {}: {e}", cli.keypair.display()))?;
    if keypair.pubkey() != JANITOR {
        bail!("{} is not the janitor {JANITOR}", cli.keypair.display());
    }
    let history_rpc_url = cli.history_rpc_url.unwrap_or_else(|| cli.rpc_url.clone());
    let janitor = Janitor {
        rpc: RpcClient::new_with_commitment(cli.rpc_url, CommitmentConfig::confirmed()),
        history_rpc: RpcClient::new_with_commitment(history_rpc_url, CommitmentConfig::confirmed()),
        keypair,
        dry_run: cli.dry_run,
        priority_fee: cli.priority_fee,
        batch: cli.batch.max(1),
    };

    match cli.command {
        Command::Survey { out } => {
            let survey = janitor.survey().await?;
            println!("{}", serde_json::to_string_pretty(&survey)?);
            std::fs::write(out, serde_json::to_string_pretty(&survey)?)?;
        }
        Command::Report { previous, out } => {
            let previous: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&previous).context("previous survey")?)?;
            let report = diff(&previous, &janitor.survey().await?);
            println!("{}", serde_json::to_string_pretty(&report)?);
            std::fs::write(out, serde_json::to_string_pretty(&report)?)?;
        }
        Command::Crank => janitor.crank(&janitor.fetch_state().await?).await?,
        Command::CloseOpenOrders => {
            janitor
                .close_open_orders(&janitor.fetch_state().await?)
                .await?
        }
        Command::CloseIndexers => {
            janitor
                .close_indexers(&janitor.fetch_state().await?)
                .await?
        }
        Command::SweepFees => janitor.sweep_fees(&janitor.fetch_state().await?).await?,
        Command::CloseMarkets => janitor.close_markets(&janitor.fetch_state().await?).await?,
        Command::WindDown => {
            janitor.crank(&janitor.fetch_state().await?).await?;
            janitor
                .close_open_orders(&janitor.fetch_state().await?)
                .await?;
            janitor
                .close_indexers(&janitor.fetch_state().await?)
                .await?;
            janitor.sweep_fees(&janitor.fetch_state().await?).await?;
            janitor.close_markets(&janitor.fetch_state().await?).await?;
        }
    }
    Ok(())
}
