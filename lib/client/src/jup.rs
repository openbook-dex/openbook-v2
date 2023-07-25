use anchor_lang::AccountDeserialize;
use anchor_lang::__private::bytemuck::Zeroable;
use anchor_lang::prelude::Clock;
use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anyhow::Result;
use fixed::types::I80F48;
use openbook_v2::accounts_zerocopy::{AccountReader, KeyedAccountReader};
use openbook_v2::state::{BookSide, OrderParams};
use openbook_v2::state::{
    EventQueue, Market, Order, Orderbook, SelfTradeBehavior::DecrementTake, Side,
};

use crate::book::{iterate_book, Amounts};
use jupiter_amm_interface::{
    AccountMap, Amm, KeyedAccount, Quote, QuoteParams, Side as JupiterSide, Swap,
    SwapAndAccountMetas, SwapParams,
};
/// An abstraction in order to share reserve mints and necessary data
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey, sysvar::clock};
use std::cell::RefCell;
use std::str;

#[derive(Clone)]
pub struct OpenBookMarket {
    market: Market,
    event_queue: EventQueue,
    bids: BookSide,
    asks: BookSide,
    timestamp: u64,
    key: Pubkey,
    label: String,
    related_accounts: Vec<Pubkey>,
    reserve_mints: [Pubkey; 2],
    oracle_price: I80F48,
}

impl Amm for OpenBookMarket {
    fn label(&self) -> String {
        self.label.clone()
    }

    fn key(&self) -> Pubkey {
        self.key
    }

    fn program_id(&self) -> Pubkey {
        openbook_v2::id()
    }

    fn get_reserve_mints(&self) -> Vec<Pubkey> {
        self.reserve_mints.to_vec()
    }

    fn get_accounts_to_update(&self) -> Vec<Pubkey> {
        self.related_accounts.to_vec()
    }

    fn from_keyed_account(keyed_account: &KeyedAccount) -> Result<Self> {
        let market = Market::try_deserialize(&mut keyed_account.account.data.as_slice())?;
        let mut related_accounts = vec![
            market.bids,
            market.asks,
            market.event_queue,
            market.base_vault,
            market.quote_vault,
            clock::ID,
        ];

        let oracles = if market.oracle_a.is_some() && market.oracle_b.is_some() {
            vec![market.oracle_a.key, market.oracle_b.key]
        } else if market.oracle_a.is_some() {
            vec![market.oracle_a.key]
        } else {
            vec![]
        };
        related_accounts.extend(oracles);

        Ok(OpenBookMarket {
            market,
            key: keyed_account.key,
            label: str::from_utf8(&market.name).unwrap().to_string(),
            related_accounts,
            reserve_mints: [market.base_mint, market.quote_mint],
            event_queue: EventQueue::zeroed(),
            bids: BookSide::zeroed(),
            asks: BookSide::zeroed(),
            oracle_price: I80F48::ZERO,
            timestamp: 0,
        })
    }

    fn update(&mut self, account_map: &AccountMap) -> Result<()> {
        let bids_data = account_map.get(&self.market.bids).unwrap();
        self.bids = BookSide::try_deserialize(&mut bids_data.data.as_slice()).unwrap();

        let asks_data = account_map.get(&self.market.asks).unwrap();
        self.asks = BookSide::try_deserialize(&mut asks_data.data.as_slice()).unwrap();

        let event_queue_data = account_map.get(&self.market.event_queue).unwrap();
        self.event_queue =
            EventQueue::try_deserialize(&mut event_queue_data.data.as_slice()).unwrap();

        let clock_data = account_map.get(&clock::ID).unwrap();
        let clock: Clock = bincode::deserialize(&clock_data.data.as_slice())?;
        self.timestamp = clock.unix_timestamp as u64;

        if self.market.oracle_a.is_some() && self.market.oracle_b.is_some() {
            let oracle_a_data = account_map.get(&self.market.oracle_a.key).unwrap();
            let oracle_a_acc = &AccountOracle {
                data: &oracle_a_data.data,
                key: self.market.oracle_a.key,
            };
            let oracle_b_data = account_map.get(&self.market.oracle_b.key).unwrap();
            let oracle_b_acc = &AccountOracle {
                data: &oracle_b_data.data,
                key: self.market.oracle_b.key,
            };

            self.oracle_price = self.market.oracle_price_from_a_and_b(
                oracle_a_acc,
                oracle_b_acc,
                self.timestamp,
            )?;
        } else if self.market.oracle_a.is_some() {
            let oracle_a_data = account_map.get(&self.market.oracle_a.key).unwrap();
            let oracle_a_acc = &AccountOracle {
                data: &oracle_a_data.data,
                key: self.market.oracle_a.key,
            };

            self.oracle_price = self
                .market
                .oracle_price_from_a(oracle_a_acc, self.timestamp)?;
        };

        Ok(())
    }

    fn quote(&self, quote_params: &QuoteParams) -> Result<Quote> {
        let side = if quote_params.input_mint == self.market.quote_mint {
            Side::Bid
        } else {
            Side::Ask
        };
        // quote params can have exact in (which is implemented here) and exact out which is not implemented
        // check with jupiter to add to their API exact_out support
        let (max_base_lots, max_quote_lots_including_fees) = match side {
            Side::Bid => (
                0,
                TryInto::<i64>::try_into(quote_params.in_amount).unwrap()
                    / self.market.quote_lot_size,
            ),

            Side::Ask => (
                TryInto::<i64>::try_into(quote_params.in_amount).unwrap()
                    / self.market.base_lot_size,
                0,
            ),
        };

        let bids_ref = RefCell::new(self.bids);
        let asks_ref = RefCell::new(self.asks);
        let book = Orderbook {
            bids: bids_ref.borrow_mut(),
            asks: asks_ref.borrow_mut(),
        };

        let order_amounts: Amounts = iterate_book(
            book,
            side,
            max_base_lots,
            max_quote_lots_including_fees,
            &self.market,
            self.oracle_price,
            self.timestamp,
        )?;
        let (in_amount, out_amount) = match side {
            Side::Bid => (
                order_amounts.total_quote_taken_native,
                order_amounts.total_base_taken_native,
            ),
            Side::Ask => (
                order_amounts.total_base_taken_native,
                order_amounts.total_quote_taken_native,
            ),
        };

        Ok(Quote {
            in_amount,
            out_amount,
            fee_mint: self.market.quote_mint,
            fee_amount: order_amounts.fee,
            ..Quote::default()
        })
    }

    fn get_swap_and_account_metas(&self, swap_params: &SwapParams) -> Result<SwapAndAccountMetas> {
        let SwapParams {
            destination_mint,
            source_mint,
            user_destination_token_account,
            user_source_token_account,
            user_transfer_authority,
            ..
        } = swap_params;

        let (side, base_account, quote_account) = if source_mint == &self.market.quote_mint {
            (
                JupiterSide::Bid,
                user_destination_token_account,
                user_source_token_account,
            )
        } else {
            (
                JupiterSide::Ask,
                user_source_token_account,
                user_destination_token_account,
            )
        };

        let mut account_metas = vec![
            AccountMeta::new(*user_transfer_authority, true),
            AccountMeta::new(self.key, false),
            AccountMeta::new(self.market.bids, false),
            AccountMeta::new(self.market.asks, false),
            AccountMeta::new(self.market.event_queue, false),
            AccountMeta::new(self.market.base_vault, false),
            AccountMeta::new(self.market.quote_vault, false),
            AccountMeta::new(*base_account, false),
            AccountMeta::new(*quote_account, false),
            AccountMeta::new_readonly(Token::id(), false),
            AccountMeta::new_readonly(System::id(), false),
        ];
        if self.market.oracle_a.is_some() && self.market.oracle_b.is_some() {
            account_metas.extend(vec![
                AccountMeta::new_readonly(self.market.oracle_a.key, false),
                AccountMeta::new_readonly(self.market.oracle_b.key, false),
            ]);
        } else if self.market.oracle_a.is_some() {
            account_metas.extend(vec![AccountMeta::new_readonly(
                self.market.oracle_a.key,
                false,
            )]);
        };

        Ok(SwapAndAccountMetas {
            swap: Swap::Openbook { side: { side } },
            account_metas,
        })
    }

    fn clone_amm(&self) -> Box<dyn Amm + Send + Sync> {
        Box::new(self.clone())
    }
}

pub struct AccountOracle<'a> {
    data: &'a Vec<u8>,
    key: Pubkey,
}

impl AccountReader for AccountOracle<'_> {
    fn owner(&self) -> &Pubkey {
        return &self.key;
    }

    fn data(&self) -> &[u8] {
        return &self.data;
    }
}

impl KeyedAccountReader for AccountOracle<'_> {
    fn key(&self) -> &Pubkey {
        return &self.key;
    }
}
