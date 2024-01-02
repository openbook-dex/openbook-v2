pub mod accounts_state;
pub mod processor;

use accounts_state::*;
use anchor_spl::token::spl_token;
use arbitrary::{Arbitrary, Unstructured};
use num_enum::IntoPrimitive;
use openbook_v2::state::*;
use processor::*;
use solana_program::{
    entrypoint::ProgramResult, instruction::AccountMeta, pubkey::Pubkey, system_program,
};
use spl_associated_token_account::get_associated_token_address;
use std::collections::{HashMap, HashSet};

pub const NUM_USERS: u8 = 8;
pub const INITIAL_BALANCE: u64 = 1_000_000_000;

#[derive(Debug, Clone, IntoPrimitive, Arbitrary)]
#[repr(u8)]
pub enum OracleId {
    OracleA = 1,
    OracleB = 2,
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct UserId(u8);

impl Arbitrary<'_> for UserId {
    fn arbitrary(u: &mut Unstructured<'_>) -> arbitrary::Result<Self> {
        let i: u8 = u.arbitrary()?;
        Ok(Self(i % NUM_USERS))
    }

    fn size_hint(_: usize) -> (usize, Option<usize>) {
        (1, Some(1))
    }
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub struct ReferrerId(u8);

impl Arbitrary<'_> for ReferrerId {
    fn arbitrary(u: &mut Unstructured<'_>) -> arbitrary::Result<Self> {
        let i: u8 = u.arbitrary()?;
        Ok(Self(i % NUM_USERS))
    }

    fn size_hint(_: usize) -> (usize, Option<usize>) {
        (1, Some(1))
    }
}

pub struct FuzzContext {
    pub payer: Pubkey,
    pub admin: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub market: Pubkey,
    pub market_authority: Pubkey,
    pub event_authority: Pubkey,
    pub bids: Pubkey,
    pub asks: Pubkey,
    pub event_heap: Pubkey,
    pub market_base_vault: Pubkey,
    pub market_quote_vault: Pubkey,
    pub oracle_a: Option<Pubkey>,
    pub oracle_b: Option<Pubkey>,
    pub collect_fee_admin: Pubkey,
    pub collect_fee_admin_quote_vault: Pubkey,
    pub users: HashMap<UserId, UserAccounts>,
    pub referrers: HashMap<ReferrerId, Pubkey>,
    pub state: AccountsState,
}

impl FuzzContext {
    pub fn new(oracles: Option<OracleId>) -> Self {
        let payer = Pubkey::new_unique();
        let admin = Pubkey::new_unique();
        let market = Pubkey::new_unique();
        let base_mint = Pubkey::new_unique();
        let quote_mint = Pubkey::new_unique();

        let (event_authority, _bump) =
            Pubkey::find_program_address(&[b"__event_authority".as_ref()], &openbook_v2::ID);

        let (market_authority, _bump) =
            Pubkey::find_program_address(&[b"Market".as_ref(), market.as_ref()], &openbook_v2::ID);

        let (oracle_a, oracle_b) = if let Some(oracles) = oracles {
            let seeds_a = &[b"StubOracle".as_ref(), admin.as_ref(), base_mint.as_ref()];
            let seeds_b = &[b"StubOracle".as_ref(), admin.as_ref(), quote_mint.as_ref()];
            match oracles {
                OracleId::OracleA => (
                    Some(Pubkey::find_program_address(seeds_a, &openbook_v2::ID).0),
                    None,
                ),
                OracleId::OracleB => (
                    Some(Pubkey::find_program_address(seeds_a, &openbook_v2::ID).0),
                    Some(Pubkey::find_program_address(seeds_b, &openbook_v2::ID).0),
                ),
            }
        } else {
            (None, None)
        };

        let bids = Pubkey::new_unique();
        let asks = Pubkey::new_unique();
        let event_heap = Pubkey::new_unique();

        let market_base_vault = get_associated_token_address(&market_authority, &base_mint);
        let market_quote_vault = get_associated_token_address(&market_authority, &quote_mint);

        let collect_fee_admin = Pubkey::new_unique();
        let collect_fee_admin_quote_vault =
            get_associated_token_address(&collect_fee_admin, &quote_mint);

        Self {
            payer,
            admin,
            base_mint,
            quote_mint,
            market,
            market_authority,
            event_authority,
            bids,
            asks,
            event_heap,
            market_base_vault,
            market_quote_vault,
            oracle_a,
            oracle_b,
            collect_fee_admin,
            collect_fee_admin_quote_vault,
            users: HashMap::new(),
            referrers: HashMap::new(),
            state: AccountsState::new(),
        }
    }

    pub fn initialize(&mut self) -> &mut Self {
        self.state
            .add_account_with_lamports(self.admin, INITIAL_BALANCE)
            .add_account_with_lamports(self.collect_fee_admin, 0)
            .add_account_with_lamports(self.payer, INITIAL_BALANCE)
            .add_mint(self.base_mint)
            .add_mint(self.quote_mint)
            .add_openbook_account::<BookSide>(self.asks)
            .add_openbook_account::<BookSide>(self.bids)
            .add_openbook_account::<EventHeap>(self.event_heap)
            .add_openbook_account::<Market>(self.market)
            .add_empty_system_account(self.market_authority)
            .add_empty_system_account(self.event_authority)
            .add_program(openbook_v2::ID) // optional accounts use this pubkey
            .add_program(spl_associated_token_account::ID)
            .add_program(spl_token::ID)
            .add_program(system_program::ID)
            .add_token_account_with_lamports(
                self.market_base_vault,
                self.market_authority,
                self.base_mint,
                0,
            )
            .add_token_account_with_lamports(
                self.market_quote_vault,
                self.market_authority,
                self.quote_mint,
                0,
            )
            .add_token_account_with_lamports(
                self.collect_fee_admin_quote_vault,
                self.collect_fee_admin,
                self.quote_mint,
                0,
            );

        if let Some(oracle_a) = self.oracle_a {
            self.state.add_openbook_account::<StubOracle>(oracle_a);
            self.stub_oracle_create(OracleId::OracleA).unwrap();
        }

        if let Some(oracle_b) = self.oracle_b {
            self.state.add_openbook_account::<StubOracle>(oracle_b);
            self.stub_oracle_create(OracleId::OracleB).unwrap();
        }

        self
    }

    fn get_or_create_new_user(&mut self, user_id: &UserId) -> &UserAccounts {
        let create_new_user = || -> UserAccounts {
            let owner = Pubkey::new_unique();
            let base_vault = Pubkey::new_unique();
            let quote_vault = Pubkey::new_unique();

            let indexer = Pubkey::find_program_address(
                &[b"OpenOrdersIndexer".as_ref(), owner.as_ref()],
                &openbook_v2::ID,
            )
            .0;

            let open_orders = Pubkey::find_program_address(
                &[b"OpenOrders".as_ref(), owner.as_ref(), &1_u32.to_le_bytes()],
                &openbook_v2::ID,
            )
            .0;

            self.state
                .add_account_with_lamports(owner, INITIAL_BALANCE)
                .add_account_with_lamports(owner, INITIAL_BALANCE)
                .add_token_account_with_lamports(base_vault, owner, self.base_mint, INITIAL_BALANCE)
                .add_token_account_with_lamports(
                    quote_vault,
                    owner,
                    self.quote_mint,
                    INITIAL_BALANCE,
                )
                .add_open_orders_indexer::<OpenOrdersIndexer>(indexer)
                .add_openbook_account::<OpenOrdersAccount>(open_orders);

            let accounts = openbook_v2::accounts::CreateOpenOrdersIndexer {
                open_orders_indexer: indexer,
                owner,
                payer: self.payer,
                system_program: system_program::ID,
            };
            let data = openbook_v2::instruction::CreateOpenOrdersIndexer {};
            process_instruction(&mut self.state, &data, &accounts, &[]).unwrap();

            let accounts = openbook_v2::accounts::CreateOpenOrdersAccount {
                open_orders_indexer: indexer,
                open_orders_account: open_orders,
                owner,
                delegate_account: None,
                payer: self.payer,
                market: self.market,
                system_program: system_program::ID,
            };
            let data = openbook_v2::instruction::CreateOpenOrdersAccount {
                name: "fuzz test".to_string(),
            };
            process_instruction(&mut self.state, &data, &accounts, &[]).unwrap();

            UserAccounts {
                owner,
                open_orders,
                base_vault,
                quote_vault,
            }
        };

        self.users.entry(*user_id).or_insert_with(create_new_user)
    }

    fn get_or_create_new_referrer(&mut self, referrer_id: &ReferrerId) -> &Pubkey {
        let create_new_referrer = || -> Pubkey {
            let quote_vault = Pubkey::new_unique();

            self.state.add_token_account_with_lamports(
                quote_vault,
                Pubkey::new_unique(),
                self.quote_mint,
                0,
            );

            quote_vault
        };

        self.referrers
            .entry(*referrer_id)
            .or_insert_with(create_new_referrer)
    }

    fn stub_oracle_create(&mut self, oracle_id: OracleId) -> ProgramResult {
        let (oracle, mint) = match oracle_id {
            OracleId::OracleA => (self.oracle_a.unwrap(), self.base_mint),
            OracleId::OracleB => (self.oracle_b.unwrap(), self.quote_mint),
        };

        let accounts = openbook_v2::accounts::StubOracleCreate {
            oracle,
            mint,
            owner: self.admin,
            payer: self.payer,
            system_program: system_program::ID,
        };
        let data = openbook_v2::instruction::StubOracleCreate { price: 1. };
        process_instruction(&mut self.state, &data, &accounts, &[])
    }

    pub fn create_market(&mut self, data: openbook_v2::instruction::CreateMarket) -> ProgramResult {
        let accounts = openbook_v2::accounts::CreateMarket {
            market: self.market,
            market_authority: self.market_authority,
            bids: self.bids,
            asks: self.asks,
            event_heap: self.event_heap,
            payer: self.payer,
            market_base_vault: self.market_base_vault,
            market_quote_vault: self.market_quote_vault,
            base_mint: self.base_mint,
            quote_mint: self.quote_mint,
            oracle_a: self.oracle_a,
            oracle_b: self.oracle_b,
            system_program: system_program::ID,
            token_program: spl_token::ID,
            associated_token_program: spl_associated_token_account::ID,
            collect_fee_admin: self.collect_fee_admin,
            open_orders_admin: None,
            consume_events_admin: None,
            close_market_admin: None,
            event_authority: self.event_authority,
            program: openbook_v2::ID,
        };
        process_instruction(&mut self.state, &data, &accounts, &[])
    }

    pub fn deposit(
        &mut self,
        user_id: &UserId,
        data: &openbook_v2::instruction::Deposit,
    ) -> ProgramResult {
        let user = self.get_or_create_new_user(user_id);

        let accounts = openbook_v2::accounts::Deposit {
            owner: user.owner,
            user_base_account: user.base_vault,
            user_quote_account: user.quote_vault,
            open_orders_account: user.open_orders,
            market: self.market,
            market_base_vault: self.market_base_vault,
            market_quote_vault: self.market_quote_vault,
            token_program: spl_token::ID,
        };

        process_instruction(&mut self.state, data, &accounts, &[])
    }

    pub fn refill(
        &mut self,
        user_id: &UserId,
        data: &openbook_v2::instruction::Refill,
    ) -> ProgramResult {
        let user = self.get_or_create_new_user(user_id);

        let accounts = openbook_v2::accounts::Deposit {
            owner: user.owner,
            user_base_account: user.base_vault,
            user_quote_account: user.quote_vault,
            open_orders_account: user.open_orders,
            market: self.market,
            market_base_vault: self.market_base_vault,
            market_quote_vault: self.market_quote_vault,
            token_program: spl_token::ID,
        };

        process_instruction(&mut self.state, data, &accounts, &[])
    }

    pub fn place_order(
        &mut self,
        user_id: &UserId,
        data: &openbook_v2::instruction::PlaceOrder,
        makers: Option<&HashSet<UserId>>,
    ) -> ProgramResult {
        let market_vault = match data.args.side {
            Side::Ask => self.market_base_vault,
            Side::Bid => self.market_quote_vault,
        };

        let user = self.get_or_create_new_user(user_id);
        let user_token_account = match data.args.side {
            Side::Ask => user.base_vault,
            Side::Bid => user.quote_vault,
        };

        let accounts = openbook_v2::accounts::PlaceOrder {
            open_orders_account: user.open_orders,
            signer: user.owner,
            user_token_account,
            open_orders_admin: None,
            market: self.market,
            bids: self.bids,
            asks: self.asks,
            event_heap: self.event_heap,
            market_vault,
            oracle_a: self.oracle_a,
            oracle_b: self.oracle_b,
            token_program: spl_token::ID,
        };

        let remaining = makers.map_or_else(Vec::new, |makers| {
            makers
                .iter()
                .filter(|id| id != &user_id)
                .filter_map(|id| self.users.get(id))
                .map(|user| AccountMeta {
                    pubkey: user.open_orders,
                    is_signer: false,
                    is_writable: true,
                })
                .collect::<Vec<_>>()
        });

        process_instruction(&mut self.state, data, &accounts, &remaining)
    }

    pub fn place_order_pegged(
        &mut self,
        user_id: &UserId,
        data: &openbook_v2::instruction::PlaceOrderPegged,
        makers: Option<&HashSet<UserId>>,
    ) -> ProgramResult {
        if self.oracle_a.is_none() {
            return Ok(());
        }

        let market_vault = match data.args.side {
            Side::Ask => self.market_base_vault,
            Side::Bid => self.market_quote_vault,
        };

        let user = self.get_or_create_new_user(user_id);
        let user_token_account = match data.args.side {
            Side::Ask => user.base_vault,
            Side::Bid => user.quote_vault,
        };

        let accounts = openbook_v2::accounts::PlaceOrder {
            open_orders_account: user.open_orders,
            signer: user.owner,
            user_token_account,
            open_orders_admin: None,
            market: self.market,
            bids: self.bids,
            asks: self.asks,
            event_heap: self.event_heap,
            market_vault,
            oracle_a: self.oracle_a,
            oracle_b: self.oracle_b,
            token_program: spl_token::ID,
        };

        let remaining = makers.map_or_else(Vec::new, |makers| {
            makers
                .iter()
                .filter(|id| id != &user_id)
                .filter_map(|id| self.users.get(id))
                .map(|user| AccountMeta {
                    pubkey: user.open_orders,
                    is_signer: false,
                    is_writable: true,
                })
                .collect::<Vec<_>>()
        });

        process_instruction(&mut self.state, data, &accounts, &remaining)
    }

    pub fn place_take_order(
        &mut self,
        user_id: &UserId,
        data: &openbook_v2::instruction::PlaceTakeOrder,
        makers: Option<&HashSet<UserId>>,
    ) -> ProgramResult {
        let user = self.get_or_create_new_user(user_id);

        let accounts = openbook_v2::accounts::PlaceTakeOrder {
            signer: user.owner,
            penalty_payer: user.owner,
            user_base_account: user.base_vault,
            user_quote_account: user.quote_vault,
            market: self.market,
            market_authority: self.market_authority,
            bids: self.bids,
            asks: self.asks,
            market_base_vault: self.market_base_vault,
            market_quote_vault: self.market_quote_vault,
            event_heap: self.event_heap,
            oracle_a: self.oracle_a,
            oracle_b: self.oracle_b,
            token_program: spl_token::ID,
            system_program: system_program::ID,
            open_orders_admin: None,
        };

        let remaining = makers.map_or_else(Vec::new, |makers| {
            makers
                .iter()
                .filter(|id| id != &user_id)
                .filter_map(|id| self.users.get(id))
                .map(|user| AccountMeta {
                    pubkey: user.open_orders,
                    is_signer: false,
                    is_writable: true,
                })
                .collect::<Vec<_>>()
        });

        process_instruction(&mut self.state, data, &accounts, &remaining)
    }

    pub fn edit_order(
        &mut self,
        user_id: &UserId,
        data: &openbook_v2::instruction::EditOrder,
        makers: Option<&HashSet<UserId>>,
    ) -> ProgramResult {
        let market_vault = match data.place_order.side {
            Side::Ask => self.market_base_vault,
            Side::Bid => self.market_quote_vault,
        };

        let user = self.get_or_create_new_user(user_id);
        let user_token_account = match data.place_order.side {
            Side::Ask => user.base_vault,
            Side::Bid => user.quote_vault,
        };

        let accounts = openbook_v2::accounts::PlaceOrder {
            open_orders_account: user.open_orders,
            signer: user.owner,
            user_token_account,
            open_orders_admin: None,
            market: self.market,
            bids: self.bids,
            asks: self.asks,
            event_heap: self.event_heap,
            market_vault,
            oracle_a: self.oracle_a,
            oracle_b: self.oracle_b,
            token_program: spl_token::ID,
        };

        let remaining = makers.map_or_else(Vec::new, |makers| {
            makers
                .iter()
                .filter(|id| id != &user_id)
                .filter_map(|id| self.users.get(id))
                .map(|user| AccountMeta {
                    pubkey: user.open_orders,
                    is_signer: false,
                    is_writable: true,
                })
                .collect::<Vec<_>>()
        });

        process_instruction(&mut self.state, data, &accounts, &remaining)
    }

    pub fn edit_order_pegged(
        &mut self,
        user_id: &UserId,
        data: &openbook_v2::instruction::EditOrderPegged,
        makers: Option<&HashSet<UserId>>,
    ) -> ProgramResult {
        if self.oracle_a.is_none() {
            return Ok(());
        }

        let market_vault = match data.place_order.side {
            Side::Ask => self.market_base_vault,
            Side::Bid => self.market_quote_vault,
        };

        let user = self.get_or_create_new_user(user_id);
        let user_token_account = match data.place_order.side {
            Side::Ask => user.base_vault,
            Side::Bid => user.quote_vault,
        };

        let accounts = openbook_v2::accounts::PlaceOrder {
            open_orders_account: user.open_orders,
            signer: user.owner,
            user_token_account,
            open_orders_admin: None,
            market: self.market,
            bids: self.bids,
            asks: self.asks,
            event_heap: self.event_heap,
            market_vault,
            oracle_a: self.oracle_a,
            oracle_b: self.oracle_b,
            token_program: spl_token::ID,
        };

        let remaining = makers.map_or_else(Vec::new, |makers| {
            makers
                .iter()
                .filter(|id| id != &user_id)
                .filter_map(|id| self.users.get(id))
                .map(|user| AccountMeta {
                    pubkey: user.open_orders,
                    is_signer: false,
                    is_writable: true,
                })
                .collect::<Vec<_>>()
        });

        process_instruction(&mut self.state, data, &accounts, &remaining)
    }

    pub fn cancel_all_and_place_orders(
        &mut self,
        user_id: &UserId,
        data: &openbook_v2::instruction::CancelAllAndPlaceOrders,
        makers: Option<&HashSet<UserId>>,
    ) -> ProgramResult {
        let user = self.get_or_create_new_user(user_id);

        let accounts = openbook_v2::accounts::CancelAllAndPlaceOrders {
            open_orders_account: user.open_orders,
            signer: user.owner,
            user_base_account: user.base_vault,
            user_quote_account: user.quote_vault,
            open_orders_admin: None,
            market: self.market,
            bids: self.bids,
            asks: self.asks,
            event_heap: self.event_heap,
            market_base_vault: self.market_base_vault,
            market_quote_vault: self.market_quote_vault,
            oracle_a: self.oracle_a,
            oracle_b: self.oracle_b,
            token_program: spl_token::ID,
        };

        let remaining = makers.map_or_else(Vec::new, |makers| {
            makers
                .iter()
                .filter(|id| id != &user_id)
                .filter_map(|id| self.users.get(id))
                .map(|user| AccountMeta {
                    pubkey: user.open_orders,
                    is_signer: false,
                    is_writable: true,
                })
                .collect::<Vec<_>>()
        });

        process_instruction(&mut self.state, data, &accounts, &remaining)
    }

    pub fn cancel_order(
        &mut self,
        user_id: &UserId,
        data: &openbook_v2::instruction::CancelOrder,
    ) -> ProgramResult {
        let Some(user) = self.users.get(user_id) else {
            return Ok(());
        };

        let accounts = openbook_v2::accounts::CancelOrder {
            signer: user.owner,
            open_orders_account: user.open_orders,
            market: self.market,
            asks: self.asks,
            bids: self.bids,
        };

        process_instruction(&mut self.state, data, &accounts, &[])
    }

    pub fn cancel_order_by_client_order_id(
        &mut self,
        user_id: &UserId,
        data: &openbook_v2::instruction::CancelOrderByClientOrderId,
    ) -> ProgramResult {
        let Some(user) = self.users.get(user_id) else {
            return Ok(());
        };

        let accounts = openbook_v2::accounts::CancelOrder {
            signer: user.owner,
            open_orders_account: user.open_orders,
            market: self.market,
            asks: self.asks,
            bids: self.bids,
        };

        process_instruction(&mut self.state, data, &accounts, &[])
    }

    pub fn cancel_all_orders(
        &mut self,
        user_id: &UserId,
        data: &openbook_v2::instruction::CancelAllOrders,
    ) -> ProgramResult {
        let Some(user) = self.users.get(user_id) else {
            return Ok(());
        };

        let accounts = openbook_v2::accounts::CancelOrder {
            signer: user.owner,
            open_orders_account: user.open_orders,
            market: self.market,
            asks: self.asks,
            bids: self.bids,
        };

        process_instruction(&mut self.state, data, &accounts, &[])
    }

    pub fn consume_events(
        &mut self,
        user_ids: &HashSet<UserId>,
        data: &openbook_v2::instruction::ConsumeEvents,
    ) -> ProgramResult {
        let accounts = openbook_v2::accounts::ConsumeEvents {
            consume_events_admin: None,
            market: self.market,
            event_heap: self.event_heap,
        };

        let remaining = user_ids
            .iter()
            .filter_map(|user_id| self.users.get(user_id))
            .map(|user| AccountMeta {
                pubkey: user.open_orders,
                is_signer: false,
                is_writable: true,
            })
            .collect::<Vec<_>>();

        process_instruction(&mut self.state, data, &accounts, &remaining)
    }

    pub fn consume_given_events(
        &mut self,
        user_ids: &HashSet<UserId>,
        data: &openbook_v2::instruction::ConsumeGivenEvents,
    ) -> ProgramResult {
        let accounts = openbook_v2::accounts::ConsumeEvents {
            consume_events_admin: None,
            market: self.market,
            event_heap: self.event_heap,
        };

        let remaining = user_ids
            .iter()
            .filter_map(|user_id| self.users.get(user_id))
            .map(|user| AccountMeta {
                pubkey: user.open_orders,
                is_signer: false,
                is_writable: true,
            })
            .collect::<Vec<_>>();

        process_instruction(&mut self.state, data, &accounts, &remaining)
    }

    pub fn settle_funds(
        &mut self,
        user_id: &UserId,
        data: &openbook_v2::instruction::SettleFunds,
        referrer_id: Option<&ReferrerId>,
    ) -> ProgramResult {
        let referrer_account = referrer_id.map(|id| *self.get_or_create_new_referrer(id));
        let Some(user) = self.users.get(user_id) else {
            return Ok(());
        };

        let accounts = openbook_v2::accounts::SettleFunds {
            owner: user.owner,
            penalty_payer: user.owner,
            open_orders_account: user.open_orders,
            user_base_account: user.base_vault,
            user_quote_account: user.quote_vault,
            market: self.market,
            market_authority: self.market_authority,
            market_base_vault: self.market_base_vault,
            market_quote_vault: self.market_quote_vault,
            token_program: spl_token::ID,
            system_program: system_program::ID,
            referrer_account,
        };

        process_instruction(&mut self.state, data, &accounts, &[])
    }

    pub fn sweep_fees(&mut self, data: &openbook_v2::instruction::SweepFees) -> ProgramResult {
        let accounts = openbook_v2::accounts::SweepFees {
            collect_fee_admin: self.collect_fee_admin,
            token_receiver_account: self.collect_fee_admin_quote_vault,
            market: self.market,
            market_authority: self.market_authority,
            market_quote_vault: self.market_quote_vault,
            token_program: spl_token::ID,
        };

        process_instruction(&mut self.state, data, &accounts, &[])
    }

    pub fn stub_oracle_set(
        &mut self,
        oracle_id: &OracleId,
        data: &openbook_v2::instruction::StubOracleSet,
    ) -> ProgramResult {
        let oracle = match oracle_id {
            OracleId::OracleA => self.oracle_a,
            OracleId::OracleB => self.oracle_b,
        };

        let Some(oracle) = oracle else {
            return Ok(());
        };

        let accounts = openbook_v2::accounts::StubOracleSet {
            oracle,
            owner: self.admin,
        };

        process_instruction(&mut self.state, data, &accounts, &[])
    }
}
