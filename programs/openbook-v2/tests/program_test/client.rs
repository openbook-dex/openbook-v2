#![allow(dead_code)]

use anchor_lang::prelude::*;
// use anchor_spl::{associated_token::AssociatedToken, token::Token, token_2022::Token2022};
use anchor_spl::{associated_token::AssociatedToken, token_2022::Token2022};
use solana_program::instruction::Instruction;
use solana_program_test::BanksClientError;
use solana_sdk::instruction;
use solana_sdk::transport::TransportError;
use std::sync::Arc;

use super::solana::SolanaCookie;
use super::utils::TestKeypair;
use openbook_v2::{state::*, PlaceOrderArgs, PlaceOrderPeggedArgs, PlaceTakeOrderArgs};

#[async_trait::async_trait(?Send)]
pub trait ClientAccountLoader {
    async fn load_bytes(&self, pubkey: &Pubkey) -> Option<Vec<u8>>;
    async fn load<T: AccountDeserialize>(&self, pubkey: &Pubkey) -> Option<T> {
        let bytes = self.load_bytes(pubkey).await?;
        AccountDeserialize::try_deserialize(&mut &bytes[..]).ok()
    }
}

#[async_trait::async_trait(?Send)]
impl ClientAccountLoader for &SolanaCookie {
    async fn load_bytes(&self, pubkey: &Pubkey) -> Option<Vec<u8>> {
        self.get_account_data(*pubkey).await
    }
}

// TODO: report error outwards etc
pub async fn send_tx<CI: ClientInstruction>(
    solana: &SolanaCookie,
    ix: CI,
) -> std::result::Result<CI::Accounts, TransportError> {
    let (accounts, instruction) = ix.to_instruction(solana).await;
    let signers = ix.signers();
    let instructions = vec![instruction];
    solana
        .process_transaction(&instructions, Some(&signers[..]))
        .await?;
    Ok(accounts)
}

/// Build a transaction from multiple instructions
pub struct ClientTransaction {
    solana: Arc<SolanaCookie>,
    instructions: Vec<instruction::Instruction>,
    signers: Vec<TestKeypair>,
}

impl<'a> ClientTransaction {
    pub fn new(solana: &Arc<SolanaCookie>) -> Self {
        Self {
            solana: solana.clone(),
            instructions: vec![],
            signers: vec![],
        }
    }

    pub async fn add_instruction<CI: ClientInstruction>(&mut self, ix: CI) -> CI::Accounts {
        let solana: &SolanaCookie = &self.solana;
        let (accounts, instruction) = ix.to_instruction(solana).await;
        self.instructions.push(instruction);
        self.signers.extend(ix.signers());
        accounts
    }

    pub fn add_instruction_direct(&mut self, ix: instruction::Instruction) {
        self.instructions.push(ix);
    }

    pub fn add_signer(&mut self, keypair: TestKeypair) {
        self.signers.push(keypair);
    }

    pub async fn send(&self) -> std::result::Result<(), BanksClientError> {
        self.solana
            .process_transaction(&self.instructions, Some(&self.signers))
            .await
    }
}

#[async_trait::async_trait(?Send)]
pub trait ClientInstruction {
    type Accounts: anchor_lang::ToAccountMetas;
    type Instruction: anchor_lang::InstructionData;

    async fn to_instruction(
        &self,
        loader: impl ClientAccountLoader + 'async_trait,
    ) -> (Self::Accounts, instruction::Instruction);
    fn signers(&self) -> Vec<TestKeypair>;
}

fn make_instruction(
    program_id: Pubkey,
    accounts: &impl anchor_lang::ToAccountMetas,
    data: impl anchor_lang::InstructionData,
) -> instruction::Instruction {
    instruction::Instruction {
        program_id,
        accounts: anchor_lang::ToAccountMetas::to_account_metas(accounts, None),
        data: anchor_lang::InstructionData::data(&data),
    }
}

pub fn get_market_address(market: TestKeypair) -> Pubkey {
    Pubkey::find_program_address(
        &[b"Market".as_ref(), market.pubkey().to_bytes().as_ref()],
        &openbook_v2::id(),
    )
    .0
}
pub async fn set_stub_oracle_price(
    solana: &SolanaCookie,
    token: &super::setup::Token,
    owner: TestKeypair,
    price: f64,
) {
    send_tx(
        solana,
        StubOracleSetInstruction {
            owner,
            mint: token.mint.pubkey,
            price,
        },
    )
    .await
    .unwrap();
}

pub struct CreateOpenOrdersIndexerInstruction {
    pub market: Pubkey,
    pub owner: TestKeypair,
    pub payer: TestKeypair,
}
#[async_trait::async_trait(?Send)]
impl ClientInstruction for CreateOpenOrdersIndexerInstruction {
    type Accounts = openbook_v2::accounts::CreateOpenOrdersIndexer;
    type Instruction = openbook_v2::instruction::CreateOpenOrdersIndexer;
    async fn to_instruction(
        &self,
        _account_loader: impl ClientAccountLoader + 'async_trait,
    ) -> (Self::Accounts, instruction::Instruction) {
        let program_id = openbook_v2::id();
        let instruction = openbook_v2::instruction::CreateOpenOrdersIndexer {};

        let open_orders_indexer = Pubkey::find_program_address(
            &[b"OpenOrdersIndexer".as_ref(), self.owner.pubkey().as_ref()],
            &program_id,
        )
        .0;

        let accounts = openbook_v2::accounts::CreateOpenOrdersIndexer {
            payer: self.payer.pubkey(),
            owner: self.owner.pubkey(),
            open_orders_indexer,
            system_program: System::id(),
        };

        let instruction = make_instruction(program_id, &accounts, instruction);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.owner, self.payer]
    }
}

pub struct CreateOpenOrdersAccountInstruction {
    pub account_num: u32,
    pub market: Pubkey,
    pub owner: TestKeypair,
    pub payer: TestKeypair,
    pub delegate: Option<Pubkey>,
}
#[async_trait::async_trait(?Send)]
impl ClientInstruction for CreateOpenOrdersAccountInstruction {
    type Accounts = openbook_v2::accounts::CreateOpenOrdersAccount;
    type Instruction = openbook_v2::instruction::CreateOpenOrdersAccount;
    async fn to_instruction(
        &self,
        _account_loader: impl ClientAccountLoader + 'async_trait,
    ) -> (Self::Accounts, instruction::Instruction) {
        let program_id = openbook_v2::id();
        let instruction = openbook_v2::instruction::CreateOpenOrdersAccount {
            name: "test".to_string(),
        };

        let open_orders_indexer = Pubkey::find_program_address(
            &[b"OpenOrdersIndexer".as_ref(), self.owner.pubkey().as_ref()],
            &program_id,
        )
        .0;

        let open_orders_account = Pubkey::find_program_address(
            &[
                b"OpenOrders".as_ref(),
                self.owner.pubkey().as_ref(),
                &self.account_num.to_le_bytes(),
            ],
            &program_id,
        )
        .0;

        let accounts = openbook_v2::accounts::CreateOpenOrdersAccount {
            owner: self.owner.pubkey(),
            open_orders_indexer,
            open_orders_account,
            market: self.market,
            payer: self.payer.pubkey(),
            delegate_account: self.delegate,
            system_program: System::id(),
        };

        let instruction = make_instruction(program_id, &accounts, instruction);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.owner, self.payer]
    }
}

pub struct CloseOpenOrdersAccountInstruction {
    pub account_num: u32,
    pub market: Pubkey,
    pub owner: TestKeypair,
    pub payer: TestKeypair,
    pub sol_destination: Pubkey,
}
#[async_trait::async_trait(?Send)]
impl ClientInstruction for CloseOpenOrdersAccountInstruction {
    type Accounts = openbook_v2::accounts::CloseOpenOrdersAccount;
    type Instruction = openbook_v2::instruction::CloseOpenOrdersAccount;
    async fn to_instruction(
        &self,
        _account_loader: impl ClientAccountLoader + 'async_trait,
    ) -> (Self::Accounts, instruction::Instruction) {
        let program_id = openbook_v2::id();
        let instruction = openbook_v2::instruction::CloseOpenOrdersAccount {};

        let open_orders_indexer = Pubkey::find_program_address(
            &[b"OpenOrdersIndexer".as_ref(), self.owner.pubkey().as_ref()],
            &program_id,
        )
        .0;

        let open_orders_account = Pubkey::find_program_address(
            &[
                b"OpenOrders".as_ref(),
                self.owner.pubkey().as_ref(),
                &self.account_num.to_le_bytes(),
            ],
            &program_id,
        )
        .0;

        let accounts = openbook_v2::accounts::CloseOpenOrdersAccount {
            owner: self.owner.pubkey(),
            payer: self.payer.pubkey(),
            open_orders_indexer,
            open_orders_account,
            sol_destination: self.sol_destination,
            system_program: System::id(),
        };

        let instruction = make_instruction(program_id, &accounts, instruction);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.owner, self.payer]
    }
}

#[derive(Default)]
pub struct CreateMarketInstruction {
    pub collect_fee_admin: Pubkey,
    pub open_orders_admin: Option<Pubkey>,
    pub consume_events_admin: Option<Pubkey>,
    pub close_market_admin: Option<Pubkey>,
    pub oracle_a: Option<Pubkey>,
    pub oracle_b: Option<Pubkey>,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub name: String,
    pub bids: Pubkey,
    pub asks: Pubkey,
    pub event_heap: Pubkey,
    pub market: TestKeypair,
    pub payer: TestKeypair,
    pub quote_lot_size: i64,
    pub base_lot_size: i64,
    pub maker_fee: i64,
    pub taker_fee: i64,
    pub fee_penalty: u64,
    pub settle_fee_flat: f32,
    pub settle_fee_amount_threshold: f32,
    pub time_expiry: i64,
}
impl CreateMarketInstruction {
    pub async fn with_new_book_and_heap(
        solana: &SolanaCookie,
        oracle_a: Option<Pubkey>,
        oracle_b: Option<Pubkey>,
    ) -> Self {
        CreateMarketInstruction {
            bids: solana
                .create_account_for_type::<BookSide>(&openbook_v2::id())
                .await,
            asks: solana
                .create_account_for_type::<BookSide>(&openbook_v2::id())
                .await,
            event_heap: solana
                .create_account_for_type::<EventHeap>(&openbook_v2::id())
                .await,
            oracle_a,
            oracle_b,
            ..CreateMarketInstruction::default()
        }
    }
}

#[async_trait::async_trait(?Send)]
impl ClientInstruction for CreateMarketInstruction {
    type Accounts = openbook_v2::accounts::CreateMarket;
    type Instruction = openbook_v2::instruction::CreateMarket;
    async fn to_instruction(
        &self,
        _loader: impl ClientAccountLoader + 'async_trait,
    ) -> (Self::Accounts, instruction::Instruction) {
        let program_id = openbook_v2::id();
        let instruction = Self::Instruction {
            name: "ONE-TWO".to_string(),
            oracle_config: OracleConfigParams {
                conf_filter: 0.1,
                max_staleness_slots: Some(100),
            },
            quote_lot_size: self.quote_lot_size,
            base_lot_size: self.base_lot_size,
            maker_fee: self.maker_fee,
            taker_fee: self.taker_fee,
            time_expiry: self.time_expiry,
        };

        let event_authority =
            Pubkey::find_program_address(&[b"__event_authority".as_ref()], &openbook_v2::id()).0;

        let market_authority = Pubkey::find_program_address(
            &[b"Market".as_ref(), self.market.pubkey().to_bytes().as_ref()],
            &openbook_v2::id(),
        )
        .0;

        let market_base_vault = spl_associated_token_account::get_associated_token_address_with_program_id(
            &market_authority,
            &self.base_mint,
            &Token2022::id(),
        );
        let market_quote_vault = spl_associated_token_account::get_associated_token_address_with_program_id(
            &market_authority,
            &self.quote_mint,
            &Token2022::id(),
        );

        let accounts = Self::Accounts {
            market: self.market.pubkey(),
            market_authority,
            bids: self.bids,
            asks: self.asks,
            event_heap: self.event_heap,
            payer: self.payer.pubkey(),
            market_base_vault,
            market_quote_vault,
            quote_mint: self.quote_mint,
            base_mint: self.base_mint,
            system_program: System::id(),
            token_program: Token2022::id(),
            associated_token_program: AssociatedToken::id(),
            collect_fee_admin: self.collect_fee_admin,
            open_orders_admin: self.open_orders_admin,
            consume_events_admin: self.consume_events_admin,
            close_market_admin: self.close_market_admin,
            oracle_a: self.oracle_a,
            oracle_b: self.oracle_b,
            event_authority,
            program: openbook_v2::id(),
        };

        let instruction = make_instruction(program_id, &accounts, instruction);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.payer, self.market]
    }
}

#[derive(Clone)]
pub struct PlaceOrderInstruction {
    pub open_orders_account: Pubkey,
    pub open_orders_admin: Option<TestKeypair>,
    pub market: Pubkey,
    pub signer: TestKeypair,
    pub market_vault: Pubkey,
    pub user_token_account: Pubkey,
    pub mint: Pubkey,
    pub side: Side,
    pub price_lots: i64,
    pub max_base_lots: i64,
    pub max_quote_lots_including_fees: i64,
    pub client_order_id: u64,
    pub expiry_timestamp: u64,
    pub order_type: PlaceOrderType,
    pub self_trade_behavior: SelfTradeBehavior,
    pub remainings: Vec<Pubkey>,
}

#[async_trait::async_trait(?Send)]
impl ClientInstruction for PlaceOrderInstruction {
    type Accounts = openbook_v2::accounts::PlaceOrder;
    type Instruction = openbook_v2::instruction::PlaceOrder;
    async fn to_instruction(
        &self,
        account_loader: impl ClientAccountLoader + 'async_trait,
    ) -> (Self::Accounts, instruction::Instruction) {
        let program_id = openbook_v2::id();
        let instruction = Self::Instruction {
            args: PlaceOrderArgs {
                side: self.side,
                price_lots: self.price_lots,
                max_base_lots: self.max_base_lots,
                max_quote_lots_including_fees: self.max_quote_lots_including_fees,
                client_order_id: self.client_order_id,
                order_type: self.order_type,
                expiry_timestamp: self.expiry_timestamp,
                self_trade_behavior: self.self_trade_behavior,
                limit: 10,
            },
        };

        let market: Market = account_loader.load(&self.market).await.unwrap();

        let accounts = Self::Accounts {
            open_orders_account: self.open_orders_account,
            open_orders_admin: self.open_orders_admin.map(|kp| kp.pubkey()),
            market: self.market,
            bids: market.bids,
            asks: market.asks,
            event_heap: market.event_heap,
            oracle_a: market.oracle_a.into(),
            oracle_b: market.oracle_b.into(),
            signer: self.signer.pubkey(),
            user_token_account: self.user_token_account,
            market_vault: self.market_vault,
            mint: Some(self.mint),
            token_program: Token2022::id(),
        };
        let mut instruction = make_instruction(program_id, &accounts, instruction);
        let mut vec_remainings: Vec<AccountMeta> = Vec::new();
        for remaining in &self.remainings {
            vec_remainings.push(AccountMeta {
                pubkey: *remaining,
                is_signer: false,
                is_writable: true,
            })
        }
        instruction.accounts.append(&mut vec_remainings);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        let mut signers = vec![self.signer];
        if let Some(open_orders_admin) = self.open_orders_admin {
            signers.push(open_orders_admin);
        }

        signers
    }
}

#[derive(Clone)]
pub struct PlaceOrderPeggedInstruction {
    pub open_orders_account: Pubkey,
    pub market: Pubkey,
    pub signer: TestKeypair,
    pub user_token_account: Pubkey,
    pub market_vault: Pubkey,
    pub mint: Pubkey,
    pub side: Side,
    pub price_offset: i64,
    pub max_base_lots: i64,
    pub max_quote_lots_including_fees: i64,
    pub client_order_id: u64,
    pub peg_limit: i64,
    pub remainings: Vec<Pubkey>,
}
#[async_trait::async_trait(?Send)]
impl ClientInstruction for PlaceOrderPeggedInstruction {
    type Accounts = openbook_v2::accounts::PlaceOrder;
    type Instruction = openbook_v2::instruction::PlaceOrderPegged;
    async fn to_instruction(
        &self,
        account_loader: impl ClientAccountLoader + 'async_trait,
    ) -> (Self::Accounts, instruction::Instruction) {
        let program_id = openbook_v2::id();
        let instruction = Self::Instruction {
            args: PlaceOrderPeggedArgs {
                side: self.side,
                price_offset_lots: self.price_offset,
                peg_limit: self.peg_limit,
                max_base_lots: self.max_base_lots,
                max_quote_lots_including_fees: self.max_quote_lots_including_fees,
                client_order_id: self.client_order_id,
                order_type: PlaceOrderType::Limit,
                expiry_timestamp: 0,
                self_trade_behavior: SelfTradeBehavior::default(),
                limit: 10,
            },
        };

        let market: Market = account_loader.load(&self.market).await.unwrap();

        let accounts = Self::Accounts {
            open_orders_account: self.open_orders_account,
            open_orders_admin: None,
            market: self.market,
            bids: market.bids,
            asks: market.asks,
            event_heap: market.event_heap,
            oracle_a: market.oracle_a.into(),
            oracle_b: market.oracle_b.into(),
            signer: self.signer.pubkey(),
            user_token_account: self.user_token_account,
            market_vault: self.market_vault,
            mint: Some(self.mint),
            token_program: Token2022::id(),
        };
        let mut instruction = make_instruction(program_id, &accounts, instruction);
        let mut vec_remainings: Vec<AccountMeta> = Vec::new();
        for remaining in &self.remainings {
            vec_remainings.push(AccountMeta {
                pubkey: *remaining,
                is_signer: false,
                is_writable: true,
            })
        }
        instruction.accounts.append(&mut vec_remainings);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.signer]
    }
}

pub struct PlaceTakeOrderInstruction {
    pub open_orders_admin: Option<TestKeypair>,
    pub market: Pubkey,
    pub signer: TestKeypair,
    pub market_base_vault: Pubkey,
    pub market_quote_vault: Pubkey,
    pub user_base_account: Pubkey,
    pub user_quote_account: Pubkey,
    pub deposit_mint: Pubkey,
    pub withdraw_mint: Pubkey,
    pub side: Side,
    pub price_lots: i64,
    pub max_base_lots: i64,
    pub max_quote_lots_including_fees: i64,
    pub referrer_account: Option<Pubkey>,
    pub remainings: Vec<Pubkey>,
}
#[async_trait::async_trait(?Send)]
impl ClientInstruction for PlaceTakeOrderInstruction {
    type Accounts = openbook_v2::accounts::PlaceTakeOrder;
    type Instruction = openbook_v2::instruction::PlaceTakeOrder;
    async fn to_instruction(
        &self,
        account_loader: impl ClientAccountLoader + 'async_trait,
    ) -> (Self::Accounts, instruction::Instruction) {
        let program_id = openbook_v2::id();
        let instruction = Self::Instruction {
            args: PlaceTakeOrderArgs {
                side: self.side,
                price_lots: self.price_lots,
                max_base_lots: self.max_base_lots,
                max_quote_lots_including_fees: self.max_quote_lots_including_fees,
                order_type: PlaceOrderType::ImmediateOrCancel,
                limit: 10,
            },
        };

        let market: Market = account_loader.load(&self.market).await.unwrap();

        let accounts = Self::Accounts {
            open_orders_admin: self.open_orders_admin.map(|kp| kp.pubkey()),
            market: self.market,
            market_authority: market.market_authority,
            bids: market.bids,
            asks: market.asks,
            event_heap: market.event_heap,
            oracle_a: market.oracle_a.into(),
            oracle_b: market.oracle_b.into(),
            signer: self.signer.pubkey(),
            penalty_payer: self.signer.pubkey(),
            user_base_account: self.user_base_account,
            user_quote_account: self.user_quote_account,
            market_base_vault: self.market_base_vault,
            market_quote_vault: self.market_quote_vault,
            deposit_mint: Some(self.deposit_mint),
            withdraw_mint: Some(self.withdraw_mint),
            referrer_account: self.referrer_account,
            token_program: Token2022::id(),
            system_program: System::id(),
        };

        let mut instruction = make_instruction(program_id, &accounts, instruction);

        let mut vec_remainings: Vec<AccountMeta> = Vec::new();
        for remaining in &self.remainings {
            vec_remainings.push(AccountMeta {
                pubkey: *remaining,
                is_signer: false,
                is_writable: true,
            })
        }
        instruction.accounts.append(&mut vec_remainings);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        let mut signers = vec![self.signer];
        if let Some(open_orders_admin) = self.open_orders_admin {
            signers.push(open_orders_admin);
        }

        signers
    }
}

pub struct CancelOrderInstruction {
    pub open_orders_account: Pubkey,
    pub market: Pubkey,
    pub signer: TestKeypair,
    pub order_id: u128,
}
#[async_trait::async_trait(?Send)]
impl ClientInstruction for CancelOrderInstruction {
    type Accounts = openbook_v2::accounts::CancelOrder;
    type Instruction = openbook_v2::instruction::CancelOrder;
    async fn to_instruction(
        &self,
        account_loader: impl ClientAccountLoader + 'async_trait,
    ) -> (Self::Accounts, instruction::Instruction) {
        let program_id = openbook_v2::id();
        let instruction = Self::Instruction {
            order_id: self.order_id,
        };
        let market: Market = account_loader.load(&self.market).await.unwrap();
        let accounts = Self::Accounts {
            open_orders_account: self.open_orders_account,
            market: self.market,
            bids: market.bids,
            asks: market.asks,
            signer: self.signer.pubkey(),
        };

        let instruction = make_instruction(program_id, &accounts, instruction);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.signer]
    }
}

pub struct CancelOrderByClientOrderIdInstruction {
    pub open_orders_account: Pubkey,
    pub market: Pubkey,
    pub signer: TestKeypair,
    pub client_order_id: u64,
}
#[async_trait::async_trait(?Send)]
impl ClientInstruction for CancelOrderByClientOrderIdInstruction {
    type Accounts = openbook_v2::accounts::CancelOrder;
    type Instruction = openbook_v2::instruction::CancelOrderByClientOrderId;
    async fn to_instruction(
        &self,
        account_loader: impl ClientAccountLoader + 'async_trait,
    ) -> (Self::Accounts, instruction::Instruction) {
        let program_id = openbook_v2::id();
        let instruction = Self::Instruction {
            client_order_id: self.client_order_id,
        };
        let market: Market = account_loader.load(&self.market).await.unwrap();
        let accounts = Self::Accounts {
            open_orders_account: self.open_orders_account,
            market: self.market,
            bids: market.bids,
            asks: market.asks,
            signer: self.signer.pubkey(),
        };

        let instruction = make_instruction(program_id, &accounts, instruction);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.signer]
    }
}

#[derive(Clone)]
pub struct CancelAllOrdersInstruction {
    pub open_orders_account: Pubkey,
    pub market: Pubkey,
    pub signer: TestKeypair,
}
#[async_trait::async_trait(?Send)]
impl ClientInstruction for CancelAllOrdersInstruction {
    type Accounts = openbook_v2::accounts::CancelOrder;
    type Instruction = openbook_v2::instruction::CancelAllOrders;
    async fn to_instruction(
        &self,
        account_loader: impl ClientAccountLoader + 'async_trait,
    ) -> (Self::Accounts, instruction::Instruction) {
        let program_id = openbook_v2::id();
        let instruction = Self::Instruction {
            side_option: None,
            limit: 5,
        };
        let market: Market = account_loader.load(&self.market).await.unwrap();
        let accounts = Self::Accounts {
            open_orders_account: self.open_orders_account,
            market: self.market,
            bids: market.bids,
            asks: market.asks,
            signer: self.signer.pubkey(),
        };

        let instruction = make_instruction(program_id, &accounts, instruction);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.signer]
    }
}

#[derive(Clone)]
pub struct ConsumeEventsInstruction {
    pub consume_events_admin: Option<TestKeypair>,
    pub market: Pubkey,
    pub open_orders_accounts: Vec<Pubkey>,
}
#[async_trait::async_trait(?Send)]
impl ClientInstruction for ConsumeEventsInstruction {
    type Accounts = openbook_v2::accounts::ConsumeEvents;
    type Instruction = openbook_v2::instruction::ConsumeEvents;
    async fn to_instruction(
        &self,
        account_loader: impl ClientAccountLoader + 'async_trait,
    ) -> (Self::Accounts, instruction::Instruction) {
        let program_id = openbook_v2::id();
        let instruction = Self::Instruction { limit: 10 };

        let market: Market = account_loader.load(&self.market).await.unwrap();
        let accounts = Self::Accounts {
            consume_events_admin: self.consume_events_admin.map(|kp| kp.pubkey()),
            market: self.market,
            event_heap: market.event_heap,
        };

        let mut instruction = make_instruction(program_id, &accounts, instruction);
        instruction
            .accounts
            .extend(self.open_orders_accounts.iter().map(|ma| AccountMeta {
                pubkey: *ma,
                is_signer: false,
                is_writable: true,
            }));
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        match self.consume_events_admin {
            Some(consume_events_admin) => vec![consume_events_admin],
            None => vec![],
        }
    }
}

pub struct ConsumeGivenEventsInstruction {
    pub consume_events_admin: Option<TestKeypair>,
    pub market: Pubkey,
    pub open_orders_accounts: Vec<Pubkey>,
    pub slots: Vec<usize>,
}
#[async_trait::async_trait(?Send)]
impl ClientInstruction for ConsumeGivenEventsInstruction {
    type Accounts = openbook_v2::accounts::ConsumeEvents;
    type Instruction = openbook_v2::instruction::ConsumeGivenEvents;
    async fn to_instruction(
        &self,
        account_loader: impl ClientAccountLoader + 'async_trait,
    ) -> (Self::Accounts, instruction::Instruction) {
        let program_id = openbook_v2::id();
        let instruction = Self::Instruction {
            slots: self.slots.clone(),
        };

        let market: Market = account_loader.load(&self.market).await.unwrap();
        let accounts = Self::Accounts {
            consume_events_admin: self.consume_events_admin.map(|kp| kp.pubkey()),
            market: self.market,
            event_heap: market.event_heap,
        };

        let mut instruction = make_instruction(program_id, &accounts, instruction);
        instruction
            .accounts
            .extend(self.open_orders_accounts.iter().map(|ma| AccountMeta {
                pubkey: *ma,
                is_signer: false,
                is_writable: true,
            }));
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        match self.consume_events_admin {
            Some(consume_events_admin) => vec![consume_events_admin],
            None => vec![],
        }
    }
}

#[derive(Clone)]
pub struct SettleFundsInstruction {
    pub owner: TestKeypair,
    pub open_orders_account: Pubkey,
    pub market: Pubkey,
    pub market_base_vault: Pubkey,
    pub market_quote_vault: Pubkey,
    pub user_base_account: Pubkey,
    pub user_quote_account: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub referrer_account: Option<Pubkey>,
    pub remainings: Vec<Pubkey>,
}
#[async_trait::async_trait(?Send)]
impl ClientInstruction for SettleFundsInstruction {
    type Accounts = openbook_v2::accounts::SettleFunds;
    type Instruction = openbook_v2::instruction::SettleFunds;
    async fn to_instruction(
        &self,
        account_loader: impl ClientAccountLoader + 'async_trait,
    ) -> (Self::Accounts, instruction::Instruction) {
        let program_id = openbook_v2::id();
        let instruction = Self::Instruction {};
        let market: Market = account_loader.load(&self.market).await.unwrap();
        let accounts = Self::Accounts {
            owner: self.owner.pubkey(),
            penalty_payer: self.owner.pubkey(),
            open_orders_account: self.open_orders_account,
            market: self.market,
            market_authority: market.market_authority,
            market_base_vault: self.market_base_vault,
            market_quote_vault: self.market_quote_vault,
            user_base_account: self.user_base_account,
            user_quote_account: self.user_quote_account,
            referrer_account: self.referrer_account,
            base_mint: Some(self.base_mint),
            quote_mint: Some(self.quote_mint),
            base_token_program: Token2022::id(),
            quote_token_program: Token2022::id(),
            system_program: System::id(),
        };

        let mut instruction = make_instruction(program_id, &accounts, instruction);

        let mut vec_remainings: Vec<AccountMeta> = Vec::new();
        for remaining in &self.remainings {
            vec_remainings.push(AccountMeta {
                pubkey: *remaining,
                is_signer: false,
                is_writable: true,
            })
        }
        instruction.accounts.append(&mut vec_remainings);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.owner]
    }
}

#[derive(Clone)]
pub struct SettleFundsExpiredInstruction {
    pub close_market_admin: TestKeypair,
    pub owner: TestKeypair,
    pub open_orders_account: Pubkey,
    pub market: Pubkey,
    pub market_base_vault: Pubkey,
    pub market_quote_vault: Pubkey,
    pub user_base_account: Pubkey,
    pub user_quote_account: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub referrer_account: Option<Pubkey>,
    pub remainings: Vec<Pubkey>,
}
#[async_trait::async_trait(?Send)]
impl ClientInstruction for SettleFundsExpiredInstruction {
    type Accounts = openbook_v2::accounts::SettleFundsExpired;
    type Instruction = openbook_v2::instruction::SettleFundsExpired;
    async fn to_instruction(
        &self,
        account_loader: impl ClientAccountLoader + 'async_trait,
    ) -> (Self::Accounts, instruction::Instruction) {
        let program_id = openbook_v2::id();
        let instruction = Self::Instruction {};
        let market: Market = account_loader.load(&self.market).await.unwrap();
        let accounts = Self::Accounts {
            close_market_admin: self.close_market_admin.pubkey(),
            owner: self.owner.pubkey(),
            penalty_payer: self.owner.pubkey(),
            open_orders_account: self.open_orders_account,
            market: self.market,
            market_authority: market.market_authority,
            market_base_vault: self.market_base_vault,
            market_quote_vault: self.market_quote_vault,
            user_base_account: self.user_base_account,
            user_quote_account: self.user_quote_account,
            referrer_account: self.referrer_account,
            base_mint: Some(self.base_mint),
            quote_mint: Some(self.quote_mint),
            base_token_program: Token2022::id(),
            quote_token_program: Token2022::id(),
            system_program: System::id(),
        };

        let mut instruction = make_instruction(program_id, &accounts, instruction);

        let mut vec_remainings: Vec<AccountMeta> = Vec::new();
        for remaining in &self.remainings {
            vec_remainings.push(AccountMeta {
                pubkey: *remaining,
                is_signer: false,
                is_writable: true,
            })
        }
        instruction.accounts.append(&mut vec_remainings);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.close_market_admin, self.owner]
    }
}

pub struct SweepFeesInstruction {
    pub collect_fee_admin: TestKeypair,
    pub market: Pubkey,
    pub market_quote_vault: Pubkey,
    pub token_receiver_account: Pubkey,
    pub mint: Pubkey,
    pub remainings: Vec<Pubkey>,
}
#[async_trait::async_trait(?Send)]
impl ClientInstruction for SweepFeesInstruction {
    type Accounts = openbook_v2::accounts::SweepFees;
    type Instruction = openbook_v2::instruction::SweepFees;
    async fn to_instruction(
        &self,
        account_loader: impl ClientAccountLoader + 'async_trait,
    ) -> (Self::Accounts, instruction::Instruction) {
        let program_id = openbook_v2::id();
        let instruction = Self::Instruction {};
        let market: Market = account_loader.load(&self.market).await.unwrap();

        let accounts = Self::Accounts {
            collect_fee_admin: self.collect_fee_admin.pubkey(),
            market: self.market,
            market_authority: market.market_authority,
            market_quote_vault: self.market_quote_vault,
            token_receiver_account: self.token_receiver_account,
            mint: Some(self.mint),
            token_program: Token2022::id(),
        };
        let mut instruction = make_instruction(program_id, &accounts, instruction);

        let mut vec_remainings: Vec<AccountMeta> = Vec::new();
        for remaining in &self.remainings {
            vec_remainings.push(AccountMeta {
                pubkey: *remaining,
                is_signer: false,
                is_writable: true,
            })
        }
        instruction.accounts.append(&mut vec_remainings);
        // println!("side sweep");
        // println!("side sweep");
        // println!("side sweep");
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.collect_fee_admin]
    }
}

pub struct DepositInstruction {
    pub open_orders_account: Pubkey,
    pub market: Pubkey,
    pub market_base_vault: Pubkey,
    pub market_quote_vault: Pubkey,
    pub user_base_account: Pubkey,
    pub user_quote_account: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub owner: TestKeypair,
    pub base_amount: u64,
    pub quote_amount: u64,
    pub remainings: Vec<Pubkey>,
}
#[async_trait::async_trait(?Send)]
impl ClientInstruction for DepositInstruction {
    type Accounts = openbook_v2::accounts::Deposit;
    type Instruction = openbook_v2::instruction::Deposit;
    async fn to_instruction(
        &self,
        _account_loader: impl ClientAccountLoader + 'async_trait,
    ) -> (Self::Accounts, instruction::Instruction) {
        let program_id = openbook_v2::id();
        let instruction = Self::Instruction {
            base_amount: self.base_amount,
            quote_amount: self.quote_amount,
        };

        let accounts = Self::Accounts {
            owner: self.owner.pubkey(),
            open_orders_account: self.open_orders_account,
            market: self.market,
            market_base_vault: self.market_base_vault,
            market_quote_vault: self.market_quote_vault,
            user_base_account: self.user_base_account,
            user_quote_account: self.user_quote_account,
            base_mint: Some(self.base_mint),
            quote_mint: Some(self.quote_mint),
            base_token_program: Token2022::id(),
            quote_token_program: Token2022::id(),
        };
        let mut instruction = make_instruction(program_id, &accounts, instruction);

        let mut vec_remainings: Vec<AccountMeta> = Vec::new();
        for remaining in &self.remainings {
            vec_remainings.push(AccountMeta {
                pubkey: *remaining,
                is_signer: false,
                is_writable: true,
            })
        }
        instruction.accounts.append(&mut vec_remainings);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.owner]
    }
}

pub struct StubOracleSetInstruction {
    pub mint: Pubkey,
    pub owner: TestKeypair,
    pub price: f64,
}
#[async_trait::async_trait(?Send)]
impl ClientInstruction for StubOracleSetInstruction {
    type Accounts = openbook_v2::accounts::StubOracleSet;
    type Instruction = openbook_v2::instruction::StubOracleSet;

    async fn to_instruction(
        &self,
        _loader: impl ClientAccountLoader + 'async_trait,
    ) -> (Self::Accounts, Instruction) {
        let program_id = openbook_v2::id();
        let instruction = Self::Instruction { price: self.price };

        let oracle = Pubkey::find_program_address(
            &[
                b"StubOracle".as_ref(),
                self.owner.pubkey().as_ref(),
                self.mint.as_ref(),
            ],
            &program_id,
        )
        .0;

        let accounts = Self::Accounts {
            oracle,
            owner: self.owner.pubkey(),
        };

        let instruction = make_instruction(program_id, &accounts, instruction);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.owner]
    }
}

pub struct StubOracleCreate {
    pub mint: Pubkey,
    pub owner: TestKeypair,
    pub payer: TestKeypair,
}
#[async_trait::async_trait(?Send)]
impl ClientInstruction for StubOracleCreate {
    type Accounts = openbook_v2::accounts::StubOracleCreate;
    type Instruction = openbook_v2::instruction::StubOracleCreate;

    async fn to_instruction(
        &self,
        _loader: impl ClientAccountLoader + 'async_trait,
    ) -> (Self::Accounts, Instruction) {
        let program_id = openbook_v2::id();
        let instruction = Self::Instruction { price: 1.0 };

        let oracle = Pubkey::find_program_address(
            &[
                b"StubOracle".as_ref(),
                self.owner.pubkey().as_ref(),
                self.mint.as_ref(),
            ],
            &program_id,
        )
        .0;

        let accounts = Self::Accounts {
            oracle,
            mint: self.mint,
            owner: self.owner.pubkey(),
            payer: self.payer.pubkey(),
            system_program: System::id(),
        };

        let instruction = make_instruction(program_id, &accounts, instruction);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.payer, self.owner]
    }
}

pub struct StubOracleCloseInstruction {
    pub mint: Pubkey,
    pub owner: TestKeypair,
    pub sol_destination: Pubkey,
}
#[async_trait::async_trait(?Send)]
impl ClientInstruction for StubOracleCloseInstruction {
    type Accounts = openbook_v2::accounts::StubOracleClose;
    type Instruction = openbook_v2::instruction::StubOracleClose;

    async fn to_instruction(
        &self,
        _loader: impl ClientAccountLoader + 'async_trait,
    ) -> (Self::Accounts, Instruction) {
        let program_id = openbook_v2::id();
        let instruction = Self::Instruction {};

        let oracle = Pubkey::find_program_address(
            &[
                b"StubOracle".as_ref(),
                self.owner.pubkey().as_ref(),
                self.mint.as_ref(),
            ],
            &program_id,
        )
        .0;

        let accounts = Self::Accounts {
            owner: self.owner.pubkey(),
            oracle,
            sol_destination: self.sol_destination,
            token_program: Token2022::id(),
        };

        let instruction = make_instruction(program_id, &accounts, instruction);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.owner]
    }
}

#[derive(Clone)]
pub struct CloseMarketInstruction {
    pub close_market_admin: TestKeypair,
    pub market: Pubkey,
    pub sol_destination: Pubkey,
}
#[async_trait::async_trait(?Send)]
impl ClientInstruction for CloseMarketInstruction {
    type Accounts = openbook_v2::accounts::CloseMarket;
    type Instruction = openbook_v2::instruction::CloseMarket;
    async fn to_instruction(
        &self,
        account_loader: impl ClientAccountLoader + 'async_trait,
    ) -> (Self::Accounts, instruction::Instruction) {
        let program_id = openbook_v2::id();
        let instruction = Self::Instruction {};
        let market: Market = account_loader.load(&self.market).await.unwrap();

        let accounts = Self::Accounts {
            close_market_admin: self.close_market_admin.pubkey(),
            market: self.market,
            bids: market.bids,
            asks: market.asks,
            event_heap: market.event_heap,
            token_program: Token2022::id(),
            sol_destination: self.sol_destination,
        };

        let instruction = make_instruction(program_id, &accounts, instruction);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.close_market_admin]
    }
}

pub struct SetMarketExpiredInstruction {
    pub close_market_admin: TestKeypair,
    pub market: Pubkey,
}
#[async_trait::async_trait(?Send)]
impl ClientInstruction for SetMarketExpiredInstruction {
    type Accounts = openbook_v2::accounts::SetMarketExpired;
    type Instruction = openbook_v2::instruction::SetMarketExpired;
    async fn to_instruction(
        &self,
        _account_loader: impl ClientAccountLoader + 'async_trait,
    ) -> (Self::Accounts, instruction::Instruction) {
        let program_id = openbook_v2::id();
        let instruction = Self::Instruction {};

        let accounts = Self::Accounts {
            close_market_admin: self.close_market_admin.pubkey(),
            market: self.market,
        };

        let instruction = make_instruction(program_id, &accounts, instruction);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.close_market_admin]
    }
}

pub struct PruneOrdersInstruction {
    pub close_market_admin: TestKeypair,
    pub market: Pubkey,
    pub open_orders_account: Pubkey,
}
#[async_trait::async_trait(?Send)]
impl ClientInstruction for PruneOrdersInstruction {
    type Accounts = openbook_v2::accounts::PruneOrders;
    type Instruction = openbook_v2::instruction::PruneOrders;
    async fn to_instruction(
        &self,
        account_loader: impl ClientAccountLoader + 'async_trait,
    ) -> (Self::Accounts, instruction::Instruction) {
        let program_id = openbook_v2::id();
        let instruction = Self::Instruction { limit: 5 };
        let market: Market = account_loader.load(&self.market).await.unwrap();

        let accounts = Self::Accounts {
            close_market_admin: self.close_market_admin.pubkey(),
            market: self.market,
            open_orders_account: self.open_orders_account,
            bids: market.bids,
            asks: market.asks,
        };

        let instruction = make_instruction(program_id, &accounts, instruction);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.close_market_admin]
    }
}

pub struct SetDelegateInstruction {
    pub delegate_account: Option<Pubkey>,
    pub owner: TestKeypair,
    pub open_orders_account: Pubkey,
}
#[async_trait::async_trait(?Send)]
impl ClientInstruction for SetDelegateInstruction {
    type Accounts = openbook_v2::accounts::SetDelegate;
    type Instruction = openbook_v2::instruction::SetDelegate;
    async fn to_instruction(
        &self,
        _account_loader: impl ClientAccountLoader + 'async_trait,
    ) -> (Self::Accounts, instruction::Instruction) {
        let program_id = openbook_v2::id();
        let instruction = Self::Instruction {};

        let accounts = Self::Accounts {
            owner: self.owner.pubkey(),
            open_orders_account: self.open_orders_account,
            delegate_account: self.delegate_account,
        };

        let instruction = make_instruction(program_id, &accounts, instruction);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.owner]
    }
}

#[derive(Clone)]
pub struct EditOrderInstruction {
    pub open_orders_account: Pubkey,
    pub open_orders_admin: Option<TestKeypair>,
    pub market: Pubkey,
    pub signer: TestKeypair,
    pub market_vault: Pubkey,
    pub user_token_account: Pubkey,
    pub mint: Pubkey,
    pub side: Side,
    pub price_lots: i64,
    pub max_base_lots: i64,
    pub max_quote_lots_including_fees: i64,
    pub client_order_id: u64,
    pub expiry_timestamp: u64,
    pub order_type: PlaceOrderType,
    pub self_trade_behavior: SelfTradeBehavior,
    pub remainings: Vec<Pubkey>,
    pub expected_cancel_size: i64,
}

#[async_trait::async_trait(?Send)]
impl ClientInstruction for EditOrderInstruction {
    type Accounts = openbook_v2::accounts::PlaceOrder;
    type Instruction = openbook_v2::instruction::EditOrder;
    async fn to_instruction(
        &self,
        account_loader: impl ClientAccountLoader + 'async_trait,
    ) -> (Self::Accounts, instruction::Instruction) {
        let program_id = openbook_v2::id();
        let instruction = Self::Instruction {
            expected_cancel_size: self.expected_cancel_size,
            client_order_id: self.client_order_id,
            place_order: PlaceOrderArgs {
                side: self.side,
                price_lots: self.price_lots,
                max_base_lots: self.max_base_lots,
                max_quote_lots_including_fees: self.max_quote_lots_including_fees,
                client_order_id: self.client_order_id,
                order_type: self.order_type,
                expiry_timestamp: self.expiry_timestamp,
                self_trade_behavior: self.self_trade_behavior,
                limit: 10,
            },
        };

        let market: Market = account_loader.load(&self.market).await.unwrap();

        let accounts = Self::Accounts {
            open_orders_account: self.open_orders_account,
            open_orders_admin: self.open_orders_admin.map(|kp| kp.pubkey()),
            market: self.market,
            bids: market.bids,
            asks: market.asks,
            event_heap: market.event_heap,
            oracle_a: market.oracle_a.into(),
            oracle_b: market.oracle_b.into(),
            signer: self.signer.pubkey(),
            user_token_account: self.user_token_account,
            market_vault: self.market_vault,
            mint: Some(self.mint),
            token_program: Token2022::id(),
        };
        let mut instruction = make_instruction(program_id, &accounts, instruction);
        let mut vec_remainings: Vec<AccountMeta> = Vec::new();
        for remaining in &self.remainings {
            vec_remainings.push(AccountMeta {
                pubkey: *remaining,
                is_signer: false,
                is_writable: true,
            })
        }
        instruction.accounts.append(&mut vec_remainings);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        let mut signers = vec![self.signer];
        if let Some(open_orders_admin) = self.open_orders_admin {
            signers.push(open_orders_admin);
        }

        signers
    }
}
