mod crank;
mod taker;

use std::sync::Arc;
use std::time::Duration;

use anchor_client::Cluster;

use clap::{Parser, Subcommand};
use openbook_v2_client::{keypair_from_cli, Client, OpenBookClient, TransactionBuilderConfig};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use tokio::time;

// TODO
// - may be nice to have one-shot cranking as well as the interval cranking
// - doing a gPA for all banks call every 10millis may be too often,
// might make sense that we maintain a service when users should query group for changes
// - I'm really annoyed about Keypair not being clonable. Seems everyone works around that manually. Should make a PR to solana to newtype it and provide that function.
// keypair_from_arg_or_env could be a function

#[derive(Parser, Debug)]
#[clap()]
struct CliDotenv {
    // When --dotenv <file> is passed, read the specified dotenv file before parsing args
    #[clap(long)]
    dotenv: std::path::PathBuf,

    remaining_args: Vec<std::ffi::OsString>,
}

#[derive(Parser, Debug, Clone)]
#[clap()]
struct Cli {
    #[clap(short, long, env)]
    rpc_url: String,

    #[clap(short, long, env)]
    openbook_account: Pubkey,

    #[clap(short, long, env)]
    owner: String,

    #[clap(subcommand)]
    command: Command,

    #[clap(long, env, default_value_t = 60)]
    // TODO: use duration type from rust instead of u64 for all these below intervals
    interval_update_banks: u64,

    #[clap(long, env, default_value_t = 5)]
    interval_consume_events: u64,

    #[clap(long, env, default_value_t = 5)]
    interval_update_funding: u64,

    #[clap(long, env, default_value_t = 120)]
    interval_check_new_listings_and_abort: u64,

    #[clap(long, env, default_value_t = 10)]
    timeout: u64,

    /// prioritize each transaction with this many microlamports/cu
    #[clap(long, env, default_value = "0")]
    prioritization_micro_lamports: u64,
}

#[derive(Subcommand, Debug, Clone)]
enum Command {
    Crank {},
    Taker {},
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let args = if let Ok(cli_dotenv) = CliDotenv::try_parse() {
        dotenv::from_path(cli_dotenv.dotenv)?;
        cli_dotenv.remaining_args
    } else {
        dotenv::dotenv().ok();
        std::env::args_os().collect()
    };
    let cli = Cli::parse_from(args);

    let owner = Arc::new(keypair_from_cli(&cli.owner));

    let rpc_url = cli.rpc_url;
    let ws_url = rpc_url.replace("https", "wss");

    let cluster = Cluster::Custom(rpc_url, ws_url);
    let commitment = match cli.command {
        Command::Crank { .. } => CommitmentConfig::confirmed(),
        Command::Taker { .. } => CommitmentConfig::confirmed(),
    };

    let openbook_client = Arc::new(
        OpenBookClient::new_for_existing_account(
            Client::new(
                cluster,
                commitment,
                owner.clone(),
                Some(Duration::from_secs(cli.timeout)),
                TransactionBuilderConfig {
                    prioritization_micro_lamports: (cli.prioritization_micro_lamports > 0)
                        .then_some(cli.prioritization_micro_lamports),
                },
            ),
            cli.openbook_account,
            owner.clone(),
        )
        .await?,
    );

    let debugging_handle = async {
        let mut interval = time::interval(time::Duration::from_secs(5));
        loop {
            interval.tick().await;
            let client = openbook_client.clone();
            tokio::task::spawn_blocking(move || {
                log::info!(
                    "Arc<OpenBookClient>::strong_count() {}",
                    Arc::<OpenBookClient>::strong_count(&client)
                )
            });
        }
    };

    match cli.command {
        Command::Crank { .. } => {
            let client = openbook_client.clone();
            crank::runner(
                client,
                debugging_handle,
                cli.interval_update_banks,
                cli.interval_consume_events,
                cli.interval_update_funding,
                cli.interval_check_new_listings_and_abort,
            )
            .await
        }
        Command::Taker { .. } => {
            let client = openbook_client.clone();
            taker::runner(client, debugging_handle).await
        }
    }
}
