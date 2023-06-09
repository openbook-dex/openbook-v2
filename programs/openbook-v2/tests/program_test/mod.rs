#![allow(dead_code)]

use std::cell::RefCell;
use std::{sync::Arc, sync::RwLock};

use fixed::types::I80F48;
use log::*;
use openbook_v2::state::Market;
use solana_program::{program_option::COption, program_pack::Pack};
use solana_program_test::*;
use solana_sdk::pubkey::Pubkey;
pub use solana_sdk::transport::TransportError;
use spl_token::{state::*, *};

use crate::program_test::setup::create_open_orders_account;
use crate::program_test::setup::Token;
pub use client::*;
pub use cookies::*;
pub use solana::*;
pub use utils::*;

pub mod client;
pub mod cookies;
pub mod setup;
pub mod solana;
pub mod utils;

pub struct TestInitialize {
    pub context: TestContext,
    pub collect_fee_admin: TestKeypair,
    pub open_orders_admin: TestKeypair,
    pub close_market_admin: TestKeypair,
    pub consume_events_admin: TestKeypair,
    pub owner: TestKeypair,
    pub payer: TestKeypair,
    pub mints: Vec<MintCookie>,
    pub owner_token_0: Pubkey,
    pub owner_token_1: Pubkey,
    pub market: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub price_lots: i64,
    pub tokens: Vec<Token>,
    pub account_0: Pubkey,
    pub account_1: Pubkey,
    pub bids: Pubkey,
}

trait AddPacked {
    fn add_packable_account<T: Pack>(
        &mut self,
        pubkey: Pubkey,
        amount: u64,
        data: &T,
        owner: &Pubkey,
    );
}

impl AddPacked for ProgramTest {
    fn add_packable_account<T: Pack>(
        &mut self,
        pubkey: Pubkey,
        amount: u64,
        data: &T,
        owner: &Pubkey,
    ) {
        let mut account = solana_sdk::account::Account::new(amount, T::get_packed_len(), owner);
        data.pack_into_slice(&mut account.data);
        self.add_account(pubkey, account);
    }
}

struct LoggerWrapper {
    inner: env_logger::Logger,
    capture: Arc<RwLock<Vec<String>>>,
}

impl Log for LoggerWrapper {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.inner.enabled(metadata)
    }

    fn log(&self, record: &log::Record) {
        if record
            .target()
            .starts_with("solana_runtime::message_processor")
        {
            let msg = record.args().to_string();
            if let Some(data) = msg.strip_prefix("Program log: ") {
                self.capture.write().unwrap().push(data.into());
            } else if let Some(data) = msg.strip_prefix("Program data: ") {
                self.capture.write().unwrap().push(data.into());
            }
        }
        self.inner.log(record);
    }

    fn flush(&self) {}
}

#[derive(Default)]
pub struct TestContextBuilder {
    test: ProgramTest,
    logger_capture: Arc<RwLock<Vec<String>>>,
    mint0: Pubkey,
}

lazy_static::lazy_static! {
    static ref LOGGER_CAPTURE: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::new(vec![]));
    static ref LOGGER_LOCK: Arc<RwLock<()>> = Arc::new(RwLock::new(()));
}

impl TestContextBuilder {
    pub fn new() -> Self {
        // We need to intercept logs to capture program log output
        let log_filter = "solana_rbpf=trace,\
                    solana_runtime::message_processor=debug,\
                    solana_runtime::system_instruction_processor=trace,\
                    solana_program_test=info";
        let env_logger =
            env_logger::Builder::from_env(env_logger::Env::new().default_filter_or(log_filter))
                .format_timestamp_nanos()
                .build();
        let _ = log::set_boxed_logger(Box::new(LoggerWrapper {
            inner: env_logger,
            capture: LOGGER_CAPTURE.clone(),
        }));

        let mut test = ProgramTest::new(
            "openbook_v2",
            openbook_v2::id(),
            processor!(openbook_v2::entry),
        );

        // intentionally set to as tight as possible, to catch potential problems early
        test.set_compute_max_units(75000);

        Self {
            test,
            logger_capture: LOGGER_CAPTURE.clone(),
            mint0: Pubkey::new_unique(),
        }
    }

    pub fn test(&mut self) -> &mut ProgramTest {
        &mut self.test
    }

    pub fn create_mints(&mut self) -> Vec<MintCookie> {
        let mut mints: Vec<MintCookie> = vec![
            MintCookie {
                index: 0,
                decimals: 6,
                unit: 10u64.pow(6) as f64,
                base_lot: 100_f64,
                quote_lot: 10_f64,
                pubkey: self.mint0,
                authority: TestKeypair::new(),
            }, // symbol: "MNGO".to_string()
        ];
        for i in 1..10 {
            mints.push(MintCookie {
                index: i,
                decimals: 6,
                unit: 10u64.pow(6) as f64,
                base_lot: 100_f64,
                quote_lot: 10_f64,
                pubkey: Pubkey::default(),
                authority: TestKeypair::new(),
            });
        }
        // Add mints in loop
        for mint in &mut mints {
            let mint_pk = if mint.pubkey == Pubkey::default() {
                Pubkey::new_unique()
            } else {
                mint.pubkey
            };
            mint.pubkey = mint_pk;

            self.test.add_packable_account(
                mint_pk,
                u32::MAX as u64,
                &Mint {
                    is_initialized: true,
                    mint_authority: COption::Some(mint.authority.pubkey()),
                    decimals: mint.decimals,
                    ..Mint::default()
                },
                &spl_token::id(),
            );
        }

        mints
    }

    pub fn create_users(&mut self, mints: &[MintCookie]) -> Vec<UserCookie> {
        let num_users = 4;
        let mut users = Vec::new();
        for _ in 0..num_users {
            let user_key = TestKeypair::new();
            self.test.add_account(
                user_key.pubkey(),
                solana_sdk::account::Account::new(
                    u32::MAX as u64,
                    0,
                    &solana_sdk::system_program::id(),
                ),
            );

            // give every user 10^18 (< 2^60) of every token
            // ~~ 1 trillion in case of 6 decimals
            let mut token_accounts = Vec::new();
            for mint in mints {
                let token_key = Pubkey::new_unique();
                self.test.add_packable_account(
                    token_key,
                    u32::MAX as u64,
                    &spl_token::state::Account {
                        mint: mint.pubkey,
                        owner: user_key.pubkey(),
                        amount: 1_000_000_000_000_000_000,
                        state: spl_token::state::AccountState::Initialized,
                        ..spl_token::state::Account::default()
                    },
                    &spl_token::id(),
                );

                token_accounts.push(token_key);
            }
            users.push(UserCookie {
                key: user_key,
                token_accounts,
            });
        }

        users
    }

    pub async fn start_default(mut self) -> TestContext {
        let mints = self.create_mints();
        let users = self.create_users(&mints);

        let solana = self.start().await;

        TestContext {
            solana,
            mints,
            users,
        }
    }

    pub async fn start(self) -> Arc<SolanaCookie> {
        let mut context = self.test.start_with_context().await;
        let rent = context.banks_client.get_rent().await.unwrap();

        Arc::new(SolanaCookie {
            context: RefCell::new(context),
            rent,
            logger_capture: self.logger_capture.clone(),
            logger_lock: LOGGER_LOCK.clone(),
            last_transaction_log: RefCell::new(vec![]),
        })
    }
}

pub struct TestContext {
    pub solana: Arc<SolanaCookie>,
    pub mints: Vec<MintCookie>,
    pub users: Vec<UserCookie>,
}

pub struct TestNewMarketInitialize {
    pub fee_penalty: u64,
    pub quote_lot_size: i64,
    pub base_lot_size: i64,
    pub maker_fee: f32,
    pub taker_fee: f32,
    pub open_orders_admin_bool: bool,
    pub close_market_admin_bool: bool,
    pub consume_events_admin_bool: bool,
    pub time_expiry: i64,
}

impl Default for TestNewMarketInitialize {
    fn default() -> TestNewMarketInitialize {
        TestNewMarketInitialize {
            fee_penalty: 0,
            quote_lot_size: 10,
            base_lot_size: 100,
            maker_fee: -0.0002,
            taker_fee: 0.0004,
            open_orders_admin_bool: false,
            close_market_admin_bool: false,
            consume_events_admin_bool: false,
            time_expiry: 0,
        }
    }
}
impl TestContext {
    pub async fn new() -> Self {
        TestContextBuilder::new().start_default().await
    }

    pub async fn new_with_market(
        args: TestNewMarketInitialize,
    ) -> Result<TestInitialize, TransportError> {
        let context = TestContextBuilder::new().start_default().await;
        let solana = &context.solana.clone();

        let collect_fee_admin_acc = TestKeypair::new();
        let open_orders_admin_acc = TestKeypair::new();
        let open_orders_admin = if args.open_orders_admin_bool {
            Some(open_orders_admin_acc.pubkey())
        } else {
            None
        };
        let close_market_admin_acc = TestKeypair::new();
        let close_market_admin = if args.close_market_admin_bool {
            Some(close_market_admin_acc.pubkey())
        } else {
            None
        };
        let consume_events_admin_acc = TestKeypair::new();
        let consume_events_admin = if args.consume_events_admin_bool {
            Some(consume_events_admin_acc.pubkey())
        } else {
            None
        };

        let owner = context.users[0].key;
        let payer = context.users[1].key;
        let mints = &context.mints[0..=2];

        let owner_token_0 = context.users[0].token_accounts[0];
        let owner_token_1 = context.users[0].token_accounts[1];

        let tokens = Token::create(mints.to_vec(), solana, collect_fee_admin_acc, payer).await;

        // Create a market

        let market = get_market_address(1);
        let base_vault = solana
            .create_associated_token_account(&market, mints[0].pubkey)
            .await;
        let quote_vault = solana
            .create_associated_token_account(&market, mints[1].pubkey)
            .await;

        let openbook_v2::accounts::CreateMarket {
            market,
            base_vault,
            quote_vault,
            bids,
            ..
        } = send_tx(
            solana,
            CreateMarketInstruction {
                collect_fee_admin: collect_fee_admin_acc.pubkey(),
                open_orders_admin,
                close_market_admin,
                consume_events_admin,
                payer,
                market_index: 1,
                quote_lot_size: args.quote_lot_size,
                base_lot_size: args.base_lot_size,
                maker_fee: args.maker_fee,
                taker_fee: args.taker_fee,
                base_mint: mints[0].pubkey,
                quote_mint: mints[1].pubkey,
                base_vault,
                quote_vault,
                fee_penalty: args.fee_penalty,
                time_expiry: args.time_expiry,
                ..CreateMarketInstruction::with_new_book_and_queue(solana, &tokens[0]).await
            },
        )
        .await
        .unwrap();

        let account_0 =
            create_open_orders_account(solana, owner, market, 0, &context.users[1]).await;
        let account_1 =
            create_open_orders_account(solana, owner, market, 1, &context.users[1]).await;

        let price_lots = {
            let market = solana.get_account::<Market>(market).await;
            market.native_price_to_lot(I80F48::from(1000))
        };

        let mints = mints.to_vec();

        Ok(TestInitialize {
            context,
            collect_fee_admin: collect_fee_admin_acc,
            open_orders_admin: open_orders_admin_acc,
            close_market_admin: close_market_admin_acc,
            consume_events_admin: consume_events_admin_acc,
            owner,
            payer,
            mints,
            owner_token_0,
            owner_token_1,
            market,
            base_vault,
            quote_vault,
            price_lots,
            tokens,
            account_0,
            account_1,
            bids,
        })
    }
}
