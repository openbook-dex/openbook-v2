use anchor_lang::__private::bytemuck::Zeroable;
use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anyhow::Result;
use fixed::types::I80F48;
use openbook_v2::{
    accounts::PlaceTakeOrder,
    accounts_zerocopy,
    pubkey_option::NonZeroPubkeyOption,
    state::{BookSide, EventHeap, Market, Orderbook, Side},
};

use crate::{
    book::{amounts_from_book, Amounts},
    remaining_accounts_to_crank,
    util::ZeroCopyDeserialize,
};
use jupiter_amm_interface::{
    AccountMap, Amm, KeyedAccount, Quote, QuoteParams, Side as JupiterSide, Swap,
    SwapAndAccountMetas, SwapParams,
};
/// An abstraction in order to share reserve mints and necessary data
use solana_sdk::{pubkey::Pubkey, sysvar::clock};
use std::cell::RefCell;

#[derive(Clone)]
pub struct OpenBookMarket {
    market: Market,
    event_heap: EventHeap,
    bids: BookSide,
    asks: BookSide,
    timestamp: u64,
    key: Pubkey,
    label: String,
    related_accounts: Vec<Pubkey>,
    reserve_mints: [Pubkey; 2],
    oracle_price: Option<I80F48>,
    is_permissioned: bool,
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
        let market =
            Market::try_deserialize_from_slice(&mut keyed_account.account.data.as_slice())?;

        let is_permissioned = market.open_orders_admin.is_some();
        let related_accounts = if is_permissioned {
            vec![]
        } else {
            let mut accs = vec![market.bids, market.asks, market.event_heap, clock::ID];

            accs.extend(
                [market.oracle_a, market.oracle_b]
                    .into_iter()
                    .filter_map(Option::<Pubkey>::from),
            );
            accs
        };

        Ok(OpenBookMarket {
            market,
            key: keyed_account.key,
            label: market.name().to_string(),
            related_accounts,
            reserve_mints: [market.base_mint, market.quote_mint],
            event_heap: EventHeap::zeroed(),
            bids: BookSide::zeroed(),
            asks: BookSide::zeroed(),
            oracle_price: None,
            timestamp: 0,
            is_permissioned,
        })
    }

    fn update(&mut self, account_map: &AccountMap) -> Result<()> {
        if self.is_permissioned {
            return Ok(());
        }

        let bids_data = account_map.get(&self.market.bids).unwrap();
        self.bids = BookSide::try_deserialize_from_slice(&mut bids_data.data.as_slice()).unwrap();

        let asks_data = account_map.get(&self.market.asks).unwrap();
        self.asks = BookSide::try_deserialize_from_slice(&mut asks_data.data.as_slice()).unwrap();

        let event_heap_data = account_map.get(&self.market.event_heap).unwrap();
        self.event_heap =
            EventHeap::try_deserialize_from_slice(&mut event_heap_data.data.as_slice()).unwrap();

        let clock_data = account_map.get(&clock::ID).unwrap();
        let clock: Clock = bincode::deserialize(clock_data.data.as_slice())?;

        let oracle_acc =
            |nonzero_pubkey: NonZeroPubkeyOption| -> Option<accounts_zerocopy::KeyedAccount> {
                let key = Option::from(nonzero_pubkey)?;
                let account = account_map.get(&key).unwrap().clone();
                Some(accounts_zerocopy::KeyedAccount { key, account })
            };

        self.oracle_price = self.market.oracle_price(
            oracle_acc(self.market.oracle_a).as_ref(),
            oracle_acc(self.market.oracle_b).as_ref(),
            clock.slot,
        )?;

        self.timestamp = clock.unix_timestamp.try_into().unwrap();

        Ok(())
    }

    fn quote(&self, quote_params: &QuoteParams) -> Result<Quote> {
        if self.is_permissioned {
            return Ok(Quote {
                not_enough_liquidity: true,
                ..Quote::default()
            });
        }

        let side = if quote_params.input_mint == self.market.quote_mint {
            Side::Bid
        } else {
            Side::Ask
        };

        let input_amount = i64::try_from(quote_params.in_amount)?;

        // quote params can have exact in (which is implemented here) and exact out which is not implemented
        // check with jupiter to add to their API exact_out support
        let (max_base_lots, max_quote_lots_including_fees) = match side {
            Side::Bid => (
                self.market.max_base_lots(),
                input_amount + (self.market.quote_lot_size - 1) / self.market.quote_lot_size,
            ),
            Side::Ask => (
                input_amount + (self.market.base_lot_size - 1) / self.market.base_lot_size,
                self.market.max_quote_lots(),
            ),
        };

        let bids_ref = RefCell::new(self.bids);
        let asks_ref = RefCell::new(self.asks);
        let book = Orderbook {
            bids: bids_ref.borrow_mut(),
            asks: asks_ref.borrow_mut(),
        };

        let order_amounts: Amounts = amounts_from_book(
            book,
            side,
            max_base_lots,
            max_quote_lots_including_fees,
            &self.market,
            self.oracle_price,
            0,
        )?;

        let (in_amount, out_amount) = match side {
            Side::Bid => (
                order_amounts.total_quote_taken_native - order_amounts.fee,
                order_amounts.total_base_taken_native,
            ),
            Side::Ask => (
                order_amounts.total_base_taken_native,
                order_amounts.total_quote_taken_native + order_amounts.fee,
            ),
        };

        Ok(Quote {
            in_amount,
            out_amount,
            fee_mint: self.market.quote_mint,
            fee_amount: order_amounts.fee,
            not_enough_liquidity: order_amounts.not_enough_liquidity,
            ..Quote::default()
        })
    }

    fn get_swap_and_account_metas(&self, swap_params: &SwapParams) -> Result<SwapAndAccountMetas> {
        let SwapParams {
            source_mint,
            user_destination_token_account,
            user_source_token_account,
            user_transfer_authority,
            ..
        } = swap_params;

        let source_is_quote = source_mint == &self.market.quote_mint;

        let (side, jup_side) = if source_is_quote {
            (Side::Bid, JupiterSide::Bid)
        } else {
            (Side::Ask, JupiterSide::Ask)
        };

        if self.is_permissioned {
            Ok(SwapAndAccountMetas {
                swap: Swap::Openbook { side: { jup_side } },
                account_metas: vec![],
            })
        } else {
            let (user_quote_account, user_base_account) = if source_is_quote {
                (*user_source_token_account, *user_destination_token_account)
            } else {
                (*user_destination_token_account, *user_source_token_account)
            };

            let accounts = PlaceTakeOrder {
                signer: *user_transfer_authority,
                penalty_payer: *user_transfer_authority,
                market: self.key,
                market_authority: self.market.market_authority,
                bids: self.market.bids,
                asks: self.market.asks,
                user_base_account,
                user_quote_account,
                market_base_vault: self.market.market_base_vault,
                market_quote_vault: self.market.market_quote_vault,
                event_heap: self.market.event_heap,
                oracle_a: Option::from(self.market.oracle_a),
                oracle_b: Option::from(self.market.oracle_b),
                token_program: Token::id(),
                system_program: System::id(),
                open_orders_admin: None,
            };

            let mut account_metas = accounts.to_account_metas(None);

            let bids_ref = RefCell::new(self.bids);
            let asks_ref = RefCell::new(self.asks);
            let book = Orderbook {
                bids: bids_ref.borrow_mut(),
                asks: asks_ref.borrow_mut(),
            };

            let remaining_accounts = remaining_accounts_to_crank(
                book,
                side,
                &self.market,
                self.oracle_price,
                self.timestamp,
            )?;

            let remaining_accounts: Vec<AccountMeta> = remaining_accounts
                .iter()
                .map(|&pubkey| AccountMeta::new(pubkey, false))
                .collect();
            account_metas.extend(remaining_accounts);

            Ok(SwapAndAccountMetas {
                swap: Swap::Openbook { side: { jup_side } },
                account_metas,
            })
        }
    }

    fn clone_amm(&self) -> Box<dyn Amm + Send + Sync> {
        Box::new(self.clone())
    }
}

#[cfg(all(test, feature = "enable-gpl"))]
mod test {
    use super::*;
    use crate::book::MAXIMUM_TAKEN_ORDERS;
    use anchor_spl::token::spl_token::{
        self,
        state::{Account as TokenAccount, AccountState},
    };
    use solana_client::nonblocking::rpc_client::RpcClient;
    use solana_program_test::{processor, ProgramTest};
    use solana_sdk::{
        account::{Account, WritableAccount},
        instruction::Instruction,
        program_pack::Pack,
        signature::Signer,
        signer::keypair::Keypair,
        stake_history::Epoch,
        transaction::Transaction,
    };
    use std::str::FromStr;

    #[tokio::test]
    // TODO replace with local accounts
    async fn test_jupiter_local() -> Result<()> {
        let market = match std::env::var("MARKET_PUBKEY") {
            Ok(key) => Pubkey::from_str(&key)?,
            Err(_) => {
                println!("missing MARKET_PUBKEY env with an existing market in the local validator, skipping test");
                return Ok(());
            }
        };

        let rpc = RpcClient::new("http://localhost:8899".to_string());
        let account = rpc.get_account(&market).await?;

        let market_account = KeyedAccount {
            key: market,
            account,
            params: None,
        };

        let mut openbook = OpenBookMarket::from_keyed_account(&market_account).unwrap();

        let pubkeys = openbook.get_accounts_to_update();
        let accounts_map: AccountMap = pubkeys
            .iter()
            .zip(rpc.get_multiple_accounts(&pubkeys).await?)
            .map(|(key, acc)| (*key, acc.unwrap()))
            .collect();

        openbook.update(&accounts_map)?;

        let (base_mint, quote_mint) = {
            let reserves = openbook.get_reserve_mints();
            (reserves[0], reserves[1])
        };

        for (side, in_amount) in [
            (openbook_v2::state::Side::Ask, 1_000_000_000),
            (openbook_v2::state::Side::Bid, 120_456_000),
        ] {
            let (input_mint, output_mint) = match side {
                openbook_v2::state::Side::Ask => (base_mint, quote_mint),
                openbook_v2::state::Side::Bid => (quote_mint, base_mint),
            };

            let quote_params = QuoteParams {
                in_amount,
                input_mint,
                output_mint,
            };

            let quote = openbook.quote(&quote_params)?;

            println!(
                "Market with base_lot_size = {}, quote_lot_size = {}, taker_fee = {}",
                openbook.market.base_lot_size,
                openbook.market.quote_lot_size,
                openbook.market.taker_fee
            );
            println!("{:#?}", quote_params);
            println!("{:#?}", quote);

            if openbook.market.open_orders_admin.is_some() {
                println!("Permissioned market");
                assert_eq!(quote.in_amount, 0);
                assert_eq!(quote.out_amount, 0);
                assert!(quote.not_enough_liquidity);
            } else {
                // hack to fix https://github.com/coral-xyz/anchor/issues/2738
                pub fn fixed_entry(
                    program_id: &Pubkey,
                    accounts: &[anchor_lang::prelude::AccountInfo],
                    data: &[u8],
                ) -> anchor_lang::solana_program::entrypoint::ProgramResult {
                    let extended_lifetime_accs = unsafe {
                        core::mem::transmute::<_, &[anchor_lang::prelude::AccountInfo<'_>]>(
                            accounts,
                        )
                    };
                    openbook_v2::entry(program_id, extended_lifetime_accs, data)
                }

                let mut pt =
                    ProgramTest::new("openbook_v2", openbook_v2::id(), processor!(fixed_entry));

                pt.add_account(market, market_account.account.clone());
                for (pubkey, account) in accounts_map.iter() {
                    pt.add_account(*pubkey, account.clone());
                }

                let initial_amount = 1_000_000_000_000_000;

                let mut add_token_account = |pubkey, owner, mint| {
                    let mut data = vec![0_u8; TokenAccount::LEN];
                    let account = TokenAccount {
                        state: AccountState::Initialized,
                        mint,
                        owner,
                        amount: initial_amount,
                        ..TokenAccount::default()
                    };
                    TokenAccount::pack(account, &mut data).unwrap();
                    pt.add_account(
                        pubkey,
                        Account::create(
                            Rent::default().minimum_balance(data.len()),
                            data,
                            spl_token::ID,
                            false,
                            Epoch::default(),
                        ),
                    );
                };

                let user = Keypair::new();
                let user_input_account = Pubkey::new_unique();
                let user_output_account = Pubkey::new_unique();

                let market_data = Market::try_deserialize_from_slice(
                    &mut market_account.account.data.as_slice(),
                )?;

                add_token_account(user_input_account, user.pubkey(), input_mint);
                add_token_account(user_output_account, user.pubkey(), output_mint);

                let (mut banks_client, payer, recent_blockhash) = pt.start().await;

                // This replicates the above logic in quote() so the asme amounts are used
                let input_amount = i64::try_from(in_amount).unwrap();
                let (max_base_lots, max_quote_lots_including_fees) = match side {
                    Side::Bid => (
                        market_data.max_base_lots(),
                        input_amount / market_data.quote_lot_size,
                    ),
                    Side::Ask => (
                        input_amount / market_data.base_lot_size,
                        market_data.max_quote_lots(),
                    ),
                };

                let (user_base_account, user_quote_account) = match side {
                    openbook_v2::state::Side::Ask => (user_input_account, user_output_account),
                    openbook_v2::state::Side::Bid => (user_output_account, user_input_account),
                };

                let ix = Instruction {
                    program_id: openbook_v2::id(),
                    accounts: anchor_lang::ToAccountMetas::to_account_metas(
                        &openbook_v2::accounts::PlaceTakeOrder {
                            signer: user.pubkey(),
                            penalty_payer: user.pubkey(),
                            market,
                            user_base_account,
                            user_quote_account,
                            market_authority: market_data.market_authority,
                            bids: market_data.bids,
                            asks: market_data.asks,
                            market_base_vault: market_data.market_base_vault,
                            market_quote_vault: market_data.market_quote_vault,
                            event_heap: market_data.event_heap,
                            oracle_a: Option::from(market_data.oracle_a),
                            oracle_b: Option::from(market_data.oracle_b),
                            token_program: Token::id(),
                            system_program: System::id(),
                            open_orders_admin: None,
                        },
                        None,
                    ),
                    data: anchor_lang::InstructionData::data(
                        &openbook_v2::instruction::PlaceTakeOrder {
                            args: openbook_v2::PlaceTakeOrderArgs {
                                side,
                                price_lots: i64::MAX,
                                max_base_lots,
                                max_quote_lots_including_fees,
                                order_type: openbook_v2::state::PlaceOrderType::Market,
                                limit: MAXIMUM_TAKEN_ORDERS,
                            },
                        },
                    ),
                };

                let tx = Transaction::new_signed_with_payer(
                    &[ix],
                    Some(&payer.pubkey()),
                    &[&payer, &user],
                    recent_blockhash,
                );
                banks_client.process_transaction(tx).await.unwrap();

                // let input_account = banks_client.get_account(user).await.unwrap().unwrap();

                let output_account = banks_client
                    .get_account(user_output_account)
                    .await
                    .unwrap()
                    .unwrap();

                let get_amount = |account: Account| -> u64 {
                    TokenAccount::unpack(&account.data).unwrap().amount
                };

                // let input_amount = get_amount(input_account);
                let output_amount = get_amount(output_account);
                // println!("{}", input_amount);
                println!("{}", output_amount);

                assert_eq!(output_amount - initial_amount, quote.out_amount);
            }
        }
        Ok(())
    }
}
