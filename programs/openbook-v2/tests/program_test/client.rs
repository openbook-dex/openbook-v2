#![allow(dead_code)]

use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use fixed::types::I80F48;
use openbook_v2::state::OpenOrdersAccountValue;
use solana_program::instruction::Instruction;
use solana_program_test::BanksClientError;
use solana_sdk::instruction;
use solana_sdk::transport::TransportError;
use std::sync::Arc;

use super::solana::SolanaCookie;
use super::utils::TestKeypair;
use openbook_v2::state::*;

#[async_trait::async_trait(?Send)]
pub trait ClientAccountLoader {
    async fn load_bytes(&self, pubkey: &Pubkey) -> Option<Vec<u8>>;
    async fn load<T: AccountDeserialize>(&self, pubkey: &Pubkey) -> Option<T> {
        let bytes = self.load_bytes(pubkey).await?;
        AccountDeserialize::try_deserialize(&mut &bytes[..]).ok()
    }
    async fn load_open_orders_account(&self, pubkey: &Pubkey) -> Option<OpenOrdersAccountValue> {
        self.load_bytes(pubkey)
            .await
            .map(|v| OpenOrdersAccountValue::from_bytes(&v[8..]).unwrap())
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

pub fn get_market_address_by_index(market_index: MarketIndex) -> Pubkey {
    Pubkey::find_program_address(
        &[b"Market".as_ref(), &market_index.to_le_bytes()],
        &openbook_v2::id(),
    )
    .0
}

async fn get_oracle_address_from_market_address(
    account_loader: &impl ClientAccountLoader,
    market_address: &Pubkey,
) -> Pubkey {
    let market: Market = account_loader.load(market_address).await.unwrap();
    market.oracle
}

pub async fn get_open_orders_account(
    solana: &SolanaCookie,
    account: Pubkey,
) -> OpenOrdersAccountValue {
    let bytes = solana.get_account_data(account).await.unwrap();
    OpenOrdersAccountValue::from_bytes(&bytes[8..]).unwrap()
}

pub async fn set_stub_oracle_price(
    solana: &SolanaCookie,
    token: &super::setup::Token,
    admin: TestKeypair,
    price: f64,
) {
    send_tx(
        solana,
        StubOracleSetInstruction {
            admin,
            mint: token.mint.pubkey,
            price,
        },
    )
    .await
    .unwrap();
}

pub struct InitOpenOrdersInstruction {
    pub account_num: u32,
    pub open_orders_count: u8,
    pub market: Pubkey,
    pub owner: TestKeypair,
    pub payer: TestKeypair,
}
#[async_trait::async_trait(?Send)]
impl ClientInstruction for InitOpenOrdersInstruction {
    type Accounts = openbook_v2::accounts::InitOpenOrders;
    type Instruction = openbook_v2::instruction::InitOpenOrders;
    async fn to_instruction(
        &self,
        _account_loader: impl ClientAccountLoader + 'async_trait,
    ) -> (Self::Accounts, instruction::Instruction) {
        let program_id = openbook_v2::id();
        let instruction = openbook_v2::instruction::InitOpenOrders {
            account_num: self.account_num,
            open_orders_count: self.open_orders_count,
        };

        let open_orders_account = Pubkey::find_program_address(
            &[
                b"OpenOrders".as_ref(),
                self.owner.pubkey().as_ref(),
                self.market.as_ref(),
                &self.account_num.to_le_bytes(),
            ],
            &program_id,
        )
        .0;

        let accounts = openbook_v2::accounts::InitOpenOrders {
            owner: self.owner.pubkey(),
            open_orders_account,
            market: self.market,
            payer: self.payer.pubkey(),
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
    pub admin: TestKeypair,
    pub oracle: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub market_index: MarketIndex,
    pub name: String,
    pub bids: Pubkey,
    pub asks: Pubkey,
    pub event_queue: Pubkey,
    pub payer: TestKeypair,
    pub quote_lot_size: i64,
    pub base_lot_size: i64,
    pub maker_fee: f32,
    pub taker_fee: f32,
    pub fee_penalty: f32,
    pub settle_fee_flat: f32,
    pub settle_fee_amount_threshold: f32,
}
impl CreateMarketInstruction {
    pub async fn with_new_book_and_queue(
        solana: &SolanaCookie,
        base: &super::setup::Token,
    ) -> Self {
        CreateMarketInstruction {
            bids: solana
                .create_account_for_type::<BookSide>(&openbook_v2::id())
                .await,
            asks: solana
                .create_account_for_type::<BookSide>(&openbook_v2::id())
                .await,
            event_queue: solana
                .create_account_for_type::<EventQueue>(&openbook_v2::id())
                .await,
            oracle: base.oracle,
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
            market_index: self.market_index,
            oracle_config: OracleConfigParams {
                conf_filter: 0.1,
                max_staleness_slots: None,
            },
            quote_lot_size: self.quote_lot_size,
            base_lot_size: self.base_lot_size,
            maker_fee: self.maker_fee,
            taker_fee: self.taker_fee,
            fee_penalty: self.fee_penalty,
        };

        let market = Pubkey::find_program_address(
            &[b"Market".as_ref(), self.market_index.to_le_bytes().as_ref()],
            &program_id,
        )
        .0;

        let base_vault =
            spl_associated_token_account::get_associated_token_address(&market, &self.base_mint);
        let quote_vault =
            spl_associated_token_account::get_associated_token_address(&market, &self.quote_mint);

        let accounts = Self::Accounts {
            admin: self.admin.pubkey(),
            oracle: self.oracle,
            market,
            bids: self.bids,
            asks: self.asks,
            event_queue: self.event_queue,
            payer: self.payer.pubkey(),
            base_vault,
            quote_vault,
            quote_mint: self.quote_mint,
            base_mint: self.base_mint,
            system_program: System::id(),
        };

        let instruction = make_instruction(program_id, &accounts, instruction);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.admin, self.payer]
    }
}

pub struct PlaceOrderInstruction {
    pub open_orders_account: Pubkey,
    pub market: Pubkey,
    pub owner: TestKeypair,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub payer: Pubkey,
    pub side: Side,
    pub price_lots: i64,
    pub max_base_lots: i64,
    pub max_quote_lots_including_fees: i64,
    pub reduce_only: bool,
    pub client_order_id: u64,
    pub expiry_timestamp: u64,
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
            side: self.side,
            price_lots: self.price_lots,
            max_base_lots: self.max_base_lots,
            max_quote_lots_including_fees: self.max_quote_lots_including_fees,
            client_order_id: self.client_order_id,
            order_type: PlaceOrderType::Limit,
            reduce_only: self.reduce_only,
            expiry_timestamp: self.expiry_timestamp,
            limit: 10,
        };

        let market: Market = account_loader.load(&self.market).await.unwrap();

        let accounts = Self::Accounts {
            open_orders_account: self.open_orders_account,
            market: self.market,
            bids: market.bids,
            asks: market.asks,
            event_queue: market.event_queue,
            oracle: market.oracle,
            owner: self.owner.pubkey(),
            payer: self.payer,
            base_vault: self.base_vault,
            quote_vault: self.quote_vault,
            token_program: Token::id(),
            system_program: System::id(),
        };
        let instruction = make_instruction(program_id, &accounts, instruction);

        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.owner]
    }
}

pub struct CancelOrderInstruction {
    pub open_orders_account: Pubkey,
    pub market: Pubkey,
    pub owner: TestKeypair,
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
            owner: self.owner.pubkey(),
        };

        let instruction = make_instruction(program_id, &accounts, instruction);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.owner]
    }
}

pub struct CancelOrderByClientOrderIdInstruction {
    pub open_orders_account: Pubkey,
    pub market: Pubkey,
    pub owner: TestKeypair,
    pub client_order_id: u64,
}
#[async_trait::async_trait(?Send)]
impl ClientInstruction for CancelOrderByClientOrderIdInstruction {
    type Accounts = openbook_v2::accounts::CancelOrderByClientOrderId;
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
            owner: self.owner.pubkey(),
        };

        let instruction = make_instruction(program_id, &accounts, instruction);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.owner]
    }
}

pub struct CancelAllOrdersInstruction {
    pub open_orders_account: Pubkey,
    pub market: Pubkey,
    pub owner: TestKeypair,
}
#[async_trait::async_trait(?Send)]
impl ClientInstruction for CancelAllOrdersInstruction {
    type Accounts = openbook_v2::accounts::CancelAllOrders;
    type Instruction = openbook_v2::instruction::CancelAllOrders;
    async fn to_instruction(
        &self,
        account_loader: impl ClientAccountLoader + 'async_trait,
    ) -> (Self::Accounts, instruction::Instruction) {
        let program_id = openbook_v2::id();
        let instruction = Self::Instruction { limit: 5 };
        let market: Market = account_loader.load(&self.market).await.unwrap();
        let accounts = Self::Accounts {
            open_orders_account: self.open_orders_account,
            market: self.market,
            bids: market.bids,
            asks: market.asks,
            owner: self.owner.pubkey(),
        };

        let instruction = make_instruction(program_id, &accounts, instruction);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.owner]
    }
}

pub struct ConsumeEventsInstruction {
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
            market: self.market,
            event_queue: market.event_queue,
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
        vec![]
    }
}

pub struct SettleFundsInstruction {
    pub open_orders_account: Pubkey,
    pub market: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub payer_base: Pubkey,
    pub payer_quote: Pubkey,
}
#[async_trait::async_trait(?Send)]
impl ClientInstruction for SettleFundsInstruction {
    type Accounts = openbook_v2::accounts::SettleFunds;
    type Instruction = openbook_v2::instruction::SettleFunds;
    async fn to_instruction(
        &self,
        _account_loader: impl ClientAccountLoader + 'async_trait,
    ) -> (Self::Accounts, instruction::Instruction) {
        let program_id = openbook_v2::id();
        let instruction = Self::Instruction {};

        let accounts = Self::Accounts {
            open_orders_account: self.open_orders_account,
            market: self.market,
            base_vault: self.base_vault,
            quote_vault: self.quote_vault,
            payer_base: self.payer_base,
            payer_quote: self.payer_quote,
            token_program: Token::id(),
            system_program: System::id(),
        };
        let instruction = make_instruction(program_id, &accounts, instruction);

        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![]
    }
}

pub struct DepositInstruction {
    pub open_orders_account: Pubkey,
    pub market: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub payer_base: Pubkey,
    pub payer_quote: Pubkey,
    pub owner: TestKeypair,
    pub base_amount_lots: u64,
    pub quote_amount_lots: u64,
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
            base_amount_lots: self.base_amount_lots,
            quote_amount_lots: self.quote_amount_lots,
        };

        let accounts = Self::Accounts {
            owner: self.owner.pubkey(),
            open_orders_account: self.open_orders_account,
            market: self.market,
            base_vault: self.base_vault,
            quote_vault: self.quote_vault,
            payer_base: self.payer_base,
            payer_quote: self.payer_quote,
            token_program: Token::id(),
            system_program: System::id(),
        };
        let instruction = make_instruction(program_id, &accounts, instruction);

        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.owner]
    }
}

pub struct StubOracleSetInstruction {
    pub mint: Pubkey,
    pub admin: TestKeypair,
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
        let instruction = Self::Instruction {
            price: I80F48::from_num(self.price),
        };
        // TODO: remove copy pasta of pda derivation, use reference
        let oracle = Pubkey::find_program_address(
            &[b"StubOracle".as_ref(), self.mint.as_ref()],
            &program_id,
        )
        .0;

        let accounts = Self::Accounts {
            oracle,
            admin: self.admin.pubkey(),
        };

        let instruction = make_instruction(program_id, &accounts, instruction);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.admin]
    }
}

pub struct StubOracleCreate {
    pub mint: Pubkey,
    pub admin: TestKeypair,
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
        let instruction = Self::Instruction {
            price: I80F48::from_num(1.0),
        };

        let oracle = Pubkey::find_program_address(
            &[b"StubOracle".as_ref(), self.mint.as_ref()],
            &program_id,
        )
        .0;

        let accounts = Self::Accounts {
            oracle,
            mint: self.mint,
            admin: self.admin.pubkey(),
            payer: self.payer.pubkey(),
            system_program: System::id(),
        };

        let instruction = make_instruction(program_id, &accounts, instruction);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.payer, self.admin]
    }
}

pub struct StubOracleCloseInstruction {
    pub mint: Pubkey,
    pub admin: TestKeypair,
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
            &[b"StubOracle".as_ref(), self.mint.as_ref()],
            &program_id,
        )
        .0;

        let accounts = Self::Accounts {
            admin: self.admin.pubkey(),
            oracle,
            sol_destination: self.sol_destination,
            token_program: Token::id(),
        };

        let instruction = make_instruction(program_id, &accounts, instruction);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.admin]
    }
}

pub struct CloseMarketInstruction {
    pub admin: TestKeypair,
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
            admin: self.admin.pubkey(),
            market: self.market,
            bids: market.bids,
            asks: market.asks,
            event_queue: market.event_queue,
            token_program: Token::id(),
            sol_destination: self.sol_destination,
        };

        let instruction = make_instruction(program_id, &accounts, instruction);
        (accounts, instruction)
    }

    fn signers(&self) -> Vec<TestKeypair> {
        vec![self.admin]
    }
}
