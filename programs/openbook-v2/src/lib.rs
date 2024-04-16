//! A central-limit order book (CLOB) program that targets the Sealevel runtime.

use anchor_lang::prelude::{
    borsh::{BorshDeserialize, BorshSerialize},
    *,
};

declare_id!("opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb");

#[macro_use]
pub mod util;

pub mod accounts_ix;
pub mod accounts_zerocopy;
pub mod error;
pub mod logs;
pub mod pubkey_option;
pub mod state;
pub mod token_utils;
pub mod types;

mod i80f48;

#[cfg(feature = "enable-gpl")]
pub mod instructions;

use accounts_ix::*;
use accounts_ix::{StubOracleCreate, StubOracleSet};
use error::*;
use state::{OracleConfigParams, Order, OrderParams, PlaceOrderType, SelfTradeBehavior, Side};
use std::cmp;

#[cfg(all(not(feature = "no-entrypoint"), not(feature = "enable-gpl")))]
compile_error!("compiling the program entrypoint without 'enable-gpl' makes no sense, enable it or use the 'cpi' or 'client' features");

#[program]
pub mod openbook_v2 {
    use super::*;

    /// Create a [`Market`](crate::state::Market) for a given token pair.
    #[allow(clippy::too_many_arguments)]
    pub fn create_market(
        ctx: Context<CreateMarket>,
        name: String,
        oracle_config: OracleConfigParams,
        quote_lot_size: i64,
        base_lot_size: i64,
        maker_fee: i64,
        taker_fee: i64,
        time_expiry: i64,
    ) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::create_market(
            ctx,
            name,
            oracle_config,
            quote_lot_size,
            base_lot_size,
            maker_fee,
            taker_fee,
            time_expiry,
        )?;
        Ok(())
    }

    /// Close a [`Market`](crate::state::Market) (only
    /// [`close_market_admin`](crate::state::Market::close_market_admin)).
    pub fn close_market(ctx: Context<CloseMarket>) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::close_market(ctx)?;
        Ok(())
    }

    /// Create an [`OpenOrdersIndexer`](crate::state::OpenOrdersIndexer) account.
    pub fn create_open_orders_indexer(ctx: Context<CreateOpenOrdersIndexer>) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::create_open_orders_indexer(ctx)?;
        Ok(())
    }

    /// Close an [`OpenOrdersIndexer`](crate::state::OpenOrdersIndexer) account.
    pub fn close_open_orders_indexer(ctx: Context<CloseOpenOrdersIndexer>) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::close_open_orders_indexer(ctx)?;
        Ok(())
    }

    /// Create an [`OpenOrdersAccount`](crate::state::OpenOrdersAccount).
    pub fn create_open_orders_account(
        ctx: Context<CreateOpenOrdersAccount>,
        name: String,
    ) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::create_open_orders_account(ctx, name)?;
        Ok(())
    }

    /// Close an [`OpenOrdersAccount`](crate::state::OpenOrdersAccount).
    pub fn close_open_orders_account(ctx: Context<CloseOpenOrdersAccount>) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::close_open_orders_account(ctx)?;
        Ok(())
    }

    /// Place an order.
    ///
    /// Different types of orders have different effects on the order book,
    /// as described in [`PlaceOrderType`](crate::state::PlaceOrderType).
    ///
    /// `price_lots` refers to the price in lots: the number of quote lots
    /// per base lot. It is ignored for `PlaceOrderType::Market` orders.
    ///
    /// `expiry_timestamp` is a unix timestamp for when this order should
    /// expire. If 0 is passed in, the order will never expire. If the time
    /// is in the past, the instruction is skipped. Timestamps in the future
    /// are reduced to now + 65,535s.
    ///
    /// `limit` determines the maximum number of orders from the book to fill,
    /// and can be used to limit CU spent. When the limit is reached, processing
    /// stops and the instruction succeeds.
    pub fn place_order<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, PlaceOrder<'info>>,
        args: PlaceOrderArgs,
    ) -> Result<Option<u128>> {
        require_gte!(args.price_lots, 1, OpenBookError::InvalidInputPriceLots);

        let time_in_force = match Order::tif_from_expiry(args.expiry_timestamp) {
            Some(t) => t,
            None => {
                msg!("Order is already expired");
                return Ok(None);
            }
        };
        let order = Order {
            side: args.side,
            max_base_lots: args.max_base_lots,
            max_quote_lots_including_fees: args.max_quote_lots_including_fees,
            client_order_id: args.client_order_id,
            time_in_force,
            self_trade_behavior: args.self_trade_behavior,
            params: match args.order_type {
                PlaceOrderType::Market => OrderParams::Market,
                PlaceOrderType::ImmediateOrCancel => OrderParams::ImmediateOrCancel {
                    price_lots: args.price_lots,
                },
                PlaceOrderType::FillOrKill => OrderParams::FillOrKill {
                    price_lots: args.price_lots,
                },
                _ => OrderParams::Fixed {
                    price_lots: args.price_lots,
                    order_type: args.order_type.to_post_order_type()?,
                },
            },
        };
        #[cfg(feature = "enable-gpl")]
        return instructions::place_order(ctx, order, args.limit);

        #[cfg(not(feature = "enable-gpl"))]
        Ok(None)
    }

    /// Edit an order.
    pub fn edit_order<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, PlaceOrder<'info>>,
        client_order_id: u64,
        expected_cancel_size: i64,
        place_order: PlaceOrderArgs,
    ) -> Result<Option<u128>> {
        require_gte!(
            place_order.price_lots,
            1,
            OpenBookError::InvalidInputPriceLots
        );

        let time_in_force = match Order::tif_from_expiry(place_order.expiry_timestamp) {
            Some(t) => t,
            None => {
                msg!("Order is already expired");
                return Ok(None);
            }
        };
        let order = Order {
            side: place_order.side,
            max_base_lots: place_order.max_base_lots,
            max_quote_lots_including_fees: place_order.max_quote_lots_including_fees,
            client_order_id: place_order.client_order_id,
            time_in_force,
            self_trade_behavior: place_order.self_trade_behavior,
            params: match place_order.order_type {
                PlaceOrderType::Market => OrderParams::Market,
                PlaceOrderType::ImmediateOrCancel => OrderParams::ImmediateOrCancel {
                    price_lots: place_order.price_lots,
                },
                PlaceOrderType::FillOrKill => OrderParams::FillOrKill {
                    price_lots: place_order.price_lots,
                },
                _ => OrderParams::Fixed {
                    price_lots: place_order.price_lots,
                    order_type: place_order.order_type.to_post_order_type()?,
                },
            },
        };
        #[cfg(feature = "enable-gpl")]
        return instructions::edit_order(
            ctx,
            client_order_id,
            expected_cancel_size,
            order,
            place_order.limit,
        );

        #[cfg(not(feature = "enable-gpl"))]
        Ok(None)
    }

    /// Edit an order pegged.
    pub fn edit_order_pegged<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, PlaceOrder<'info>>,
        client_order_id: u64,
        expected_cancel_size: i64,
        place_order: PlaceOrderPeggedArgs,
    ) -> Result<Option<u128>> {
        require!(
            ctx.accounts.oracle_a.is_some(),
            OpenBookError::DisabledOraclePeg
        );

        require_gt!(
            place_order.peg_limit,
            0,
            OpenBookError::InvalidInputPegLimit
        );

        let time_in_force = match Order::tif_from_expiry(place_order.expiry_timestamp) {
            Some(t) => t,
            None => {
                msg!("Order is already expired");
                return Ok(None);
            }
        };

        let order = Order {
            side: place_order.side,
            max_base_lots: place_order.max_base_lots,
            max_quote_lots_including_fees: place_order.max_quote_lots_including_fees,
            client_order_id: place_order.client_order_id,
            time_in_force,
            self_trade_behavior: place_order.self_trade_behavior,
            params: OrderParams::OraclePegged {
                price_offset_lots: place_order.price_offset_lots,
                order_type: place_order.order_type.to_post_order_type()?,
                peg_limit: place_order.peg_limit,
            },
        };
        #[cfg(feature = "enable-gpl")]
        return instructions::edit_order(
            ctx,
            client_order_id,
            expected_cancel_size,
            order,
            place_order.limit,
        );

        #[cfg(not(feature = "enable-gpl"))]
        Ok(None)
    }

    /// Place multiple orders
    pub fn place_orders<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, CancelAllAndPlaceOrders<'info>>,
        orders_type: PlaceOrderType,
        bids: Vec<PlaceMultipleOrdersArgs>,
        asks: Vec<PlaceMultipleOrdersArgs>,
        limit: u8,
    ) -> Result<Vec<Option<u128>>> {
        let n_bids = bids.len();

        let mut orders = vec![];
        for (i, order) in bids.into_iter().chain(asks).enumerate() {
            require_gte!(order.price_lots, 1, OpenBookError::InvalidInputPriceLots);

            let time_in_force = match Order::tif_from_expiry(order.expiry_timestamp) {
                Some(t) => t,
                None => {
                    msg!("Order is already expired");
                    continue;
                }
            };
            orders.push(Order {
                side: if i < n_bids { Side::Bid } else { Side::Ask },
                max_base_lots: i64::MIN, // this will be overriden to max_base_lots
                max_quote_lots_including_fees: order.max_quote_lots_including_fees,
                client_order_id: i as u64,
                time_in_force,
                self_trade_behavior: SelfTradeBehavior::CancelProvide,
                params: match orders_type {
                    PlaceOrderType::Market => OrderParams::Market,
                    PlaceOrderType::ImmediateOrCancel => OrderParams::ImmediateOrCancel {
                        price_lots: order.price_lots,
                    },
                    PlaceOrderType::FillOrKill => OrderParams::FillOrKill {
                        price_lots: order.price_lots,
                    },
                    _ => OrderParams::Fixed {
                        price_lots: order.price_lots,
                        order_type: orders_type.to_post_order_type()?,
                    },
                },
            });
        }

        #[cfg(feature = "enable-gpl")]
        return instructions::cancel_all_and_place_orders(ctx, false, orders, limit);

        #[cfg(not(feature = "enable-gpl"))]
        Ok(vec![])
    }

    /// Cancel orders and place multiple orders.
    pub fn cancel_all_and_place_orders<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, CancelAllAndPlaceOrders<'info>>,
        orders_type: PlaceOrderType,
        bids: Vec<PlaceMultipleOrdersArgs>,
        asks: Vec<PlaceMultipleOrdersArgs>,
        limit: u8,
    ) -> Result<Vec<Option<u128>>> {
        let n_bids = bids.len();

        let mut orders = vec![];
        for (i, order) in bids.into_iter().chain(asks).enumerate() {
            require_gte!(order.price_lots, 1, OpenBookError::InvalidInputPriceLots);

            let time_in_force = match Order::tif_from_expiry(order.expiry_timestamp) {
                Some(t) => t,
                None => {
                    msg!("Order is already expired");
                    continue;
                }
            };
            orders.push(Order {
                side: if i < n_bids { Side::Bid } else { Side::Ask },
                max_base_lots: i64::MIN, // this will be overriden to max_base_lots
                max_quote_lots_including_fees: order.max_quote_lots_including_fees,
                client_order_id: i as u64,
                time_in_force,
                self_trade_behavior: SelfTradeBehavior::CancelProvide,
                params: match orders_type {
                    PlaceOrderType::Market => OrderParams::Market,
                    PlaceOrderType::ImmediateOrCancel => OrderParams::ImmediateOrCancel {
                        price_lots: order.price_lots,
                    },
                    PlaceOrderType::FillOrKill => OrderParams::FillOrKill {
                        price_lots: order.price_lots,
                    },
                    _ => OrderParams::Fixed {
                        price_lots: order.price_lots,
                        order_type: orders_type.to_post_order_type()?,
                    },
                },
            });
        }

        #[cfg(feature = "enable-gpl")]
        return instructions::cancel_all_and_place_orders(ctx, true, orders, limit);

        #[cfg(not(feature = "enable-gpl"))]
        Ok(vec![])
    }

    /// Place an oracle-peg order.
    pub fn place_order_pegged<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, PlaceOrder<'info>>,
        args: PlaceOrderPeggedArgs,
    ) -> Result<Option<u128>> {
        require!(
            ctx.accounts.oracle_a.is_some(),
            OpenBookError::DisabledOraclePeg
        );

        require_gt!(args.peg_limit, 0, OpenBookError::InvalidInputPegLimit);

        let time_in_force = match Order::tif_from_expiry(args.expiry_timestamp) {
            Some(t) => t,
            None => {
                msg!("Order is already expired");
                return Ok(None);
            }
        };

        let order = Order {
            side: args.side,
            max_base_lots: args.max_base_lots,
            max_quote_lots_including_fees: args.max_quote_lots_including_fees,
            client_order_id: args.client_order_id,
            time_in_force,
            self_trade_behavior: args.self_trade_behavior,
            params: OrderParams::OraclePegged {
                price_offset_lots: args.price_offset_lots,
                order_type: args.order_type.to_post_order_type()?,
                peg_limit: args.peg_limit,
            },
        };
        #[cfg(feature = "enable-gpl")]
        return instructions::place_order(ctx, order, args.limit);

        #[cfg(not(feature = "enable-gpl"))]
        Ok(None)
    }

    /// Place an order that shall take existing liquidity off of the book, not
    /// add a new order off the book.
    ///
    /// This type of order allows for instant token settlement for the taker.
    pub fn place_take_order<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, PlaceTakeOrder<'info>>,
        args: PlaceTakeOrderArgs,
    ) -> Result<()> {
        require_gte!(args.price_lots, 1, OpenBookError::InvalidInputPriceLots);

        let order = Order {
            side: args.side,
            max_base_lots: args.max_base_lots,
            max_quote_lots_including_fees: args.max_quote_lots_including_fees,
            client_order_id: 0,
            time_in_force: 0,
            self_trade_behavior: SelfTradeBehavior::default(),
            params: match args.order_type {
                PlaceOrderType::Market => OrderParams::Market,
                PlaceOrderType::ImmediateOrCancel => OrderParams::ImmediateOrCancel {
                    price_lots: args.price_lots,
                },
                PlaceOrderType::FillOrKill => OrderParams::FillOrKill {
                    price_lots: args.price_lots,
                },
                _ => return Err(OpenBookError::InvalidInputOrderType.into()),
            },
        };

        #[cfg(feature = "enable-gpl")]
        instructions::place_take_order(ctx, order, args.limit)?;
        Ok(())
    }

    /// Process up to `limit` [events](crate::state::AnyEvent).
    ///
    /// When a user places a 'take' order, they do not know beforehand which
    /// market maker will have placed the 'make' order that they get executed
    /// against. This prevents them from passing in a market maker's
    /// [`OpenOrdersAccount`](crate::state::OpenOrdersAccount), which is needed
    /// to credit/debit the relevant tokens to/from the maker. As such, Openbook
    /// uses a 'crank' system, where `place_order` only emits events, and
    /// `consume_events` handles token settlement.
    ///
    /// Currently, there are two types of events: [`FillEvent`](crate::state::FillEvent)s
    /// and [`OutEvent`](crate::state::OutEvent)s.
    ///
    /// A `FillEvent` is emitted when an order is filled, and it is handled by
    /// debiting whatever the taker is selling from the taker and crediting
    /// it to the maker, and debiting whatever the taker is buying from the
    /// maker and crediting it to the taker. Note that *no tokens are moved*,
    /// these are just debits and credits to each party's [`Position`](crate::state::Position).
    ///
    /// An `OutEvent` is emitted when a limit order needs to be removed from
    /// the book during a `place_order` invocation, and it is handled by
    /// crediting whatever the maker would have sold (quote token in a bid,
    /// base token in an ask) back to the maker.
    pub fn consume_events<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, ConsumeEvents>,
        limit: usize,
    ) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::consume_events(ctx, limit, None)?;
        Ok(())
    }

    /// Process the [events](crate::state::AnyEvent) at the given positions.
    pub fn consume_given_events<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, ConsumeEvents>,
        slots: Vec<usize>,
    ) -> Result<()> {
        require!(
            slots
                .iter()
                .all(|slot| *slot < crate::state::MAX_NUM_EVENTS as usize),
            OpenBookError::InvalidInputHeapSlots
        );
        #[cfg(feature = "enable-gpl")]
        instructions::consume_events(ctx, slots.len(), Some(slots))?;
        Ok(())
    }

    /// Cancel an order by its `order_id`.
    ///
    /// Note that this doesn't emit an [`OutEvent`](crate::state::OutEvent) because a
    /// maker knows that they will be passing in their own [`OpenOrdersAccount`](crate::state::OpenOrdersAccount).
    pub fn cancel_order(ctx: Context<CancelOrder>, order_id: u128) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::cancel_order(ctx, order_id)?;
        Ok(())
    }

    /// Cancel an order by its `client_order_id`.
    ///
    /// Note that this doesn't emit an [`OutEvent`](crate::state::OutEvent) because a
    /// maker knows that they will be passing in their own [`OpenOrdersAccount`](crate::state::OpenOrdersAccount).
    pub fn cancel_order_by_client_order_id(
        ctx: Context<CancelOrder>,
        client_order_id: u64,
    ) -> Result<i64> {
        #[cfg(feature = "enable-gpl")]
        return instructions::cancel_order_by_client_order_id(ctx, client_order_id);

        #[cfg(not(feature = "enable-gpl"))]
        Ok(0)
    }

    /// Cancel up to `limit` orders, optionally filtering by side
    pub fn cancel_all_orders(
        ctx: Context<CancelOrder>,
        side_option: Option<Side>,
        limit: u8,
    ) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::cancel_all_orders(ctx, side_option, limit)?;
        Ok(())
    }

    /// Deposit a certain amount of `base` and `quote` lamports into one's
    /// [`Position`](crate::state::Position).
    ///
    /// Makers might wish to `deposit`, rather than have actual tokens moved for
    /// each trade, in order to reduce CUs.
    pub fn deposit(ctx: Context<Deposit>, base_amount: u64, quote_amount: u64) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::deposit(ctx, base_amount, quote_amount)?;
        Ok(())
    }

    /// Refill a certain amount of `base` and `quote` lamports. The amount being passed is the
    /// total lamports that the [`Position`](crate::state::Position) will have.
    ///
    /// Makers might wish to `refill`, rather than have actual tokens moved for
    /// each trade, in order to reduce CUs.
    pub fn refill(ctx: Context<Deposit>, base_amount: u64, quote_amount: u64) -> Result<()> {
        let (quote_amount, base_amount) = {
            let open_orders_account = ctx.accounts.open_orders_account.load()?;
            (
                quote_amount
                    - cmp::min(quote_amount, open_orders_account.position.quote_free_native),
                base_amount - cmp::min(base_amount, open_orders_account.position.base_free_native),
            )
        };
        #[cfg(feature = "enable-gpl")]
        instructions::deposit(ctx, base_amount, quote_amount)?;
        Ok(())
    }

    /// Withdraw any available tokens.
    pub fn settle_funds<'info>(ctx: Context<'_, '_, '_, 'info, SettleFunds<'info>>) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::settle_funds(ctx)?;
        Ok(())
    }

    /// Withdraw any available tokens when the market is expired (only
    /// [`close_market_admin`](crate::state::Market::close_market_admin)).
    pub fn settle_funds_expired<'info>(
        ctx: Context<'_, '_, '_, 'info, SettleFundsExpired<'info>>,
    ) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::settle_funds_expired(ctx)?;
        Ok(())
    }

    /// Sweep fees, as a [`Market`](crate::state::Market)'s admin.
    pub fn sweep_fees(ctx: Context<SweepFees>) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::sweep_fees(ctx)?;
        Ok(())
    }

    /// Update the [`delegate`](crate::state::OpenOrdersAccount::delegate) of an open orders account.
    pub fn set_delegate(ctx: Context<SetDelegate>) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::set_delegate(ctx)?;
        Ok(())
    }

    /// Set market to expired before pruning orders and closing the market (only
    /// [`close_market_admin`](crate::state::Market::close_market_admin)).
    pub fn set_market_expired(ctx: Context<SetMarketExpired>) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::set_market_expired(ctx)?;
        Ok(())
    }

    /// Remove orders from the book when the market is expired (only
    /// [`close_market_admin`](crate::state::Market::close_market_admin)).
    pub fn prune_orders(ctx: Context<PruneOrders>, limit: u8) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::prune_orders(ctx, limit)?;
        Ok(())
    }

    pub fn stub_oracle_create(ctx: Context<StubOracleCreate>, price: f64) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::stub_oracle_create(ctx, price)?;
        Ok(())
    }

    pub fn stub_oracle_close(ctx: Context<StubOracleClose>) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::stub_oracle_close(ctx)?;
        Ok(())
    }

    pub fn stub_oracle_set(ctx: Context<StubOracleSet>, price: f64) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::stub_oracle_set(ctx, price)?;
        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Copy, Clone)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct PlaceOrderArgs {
    pub side: Side,
    pub price_lots: i64,
    pub max_base_lots: i64,
    pub max_quote_lots_including_fees: i64,
    pub client_order_id: u64,
    pub order_type: PlaceOrderType,
    pub expiry_timestamp: u64,
    pub self_trade_behavior: SelfTradeBehavior,
    // Maximum number of orders from the book to fill.
    //
    // Use this to limit compute used during order matching.
    // When the limit is reached, processing stops and the instruction succeeds.
    pub limit: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Copy, Clone)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct PlaceMultipleOrdersArgs {
    pub price_lots: i64,
    pub max_quote_lots_including_fees: i64,
    pub expiry_timestamp: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Copy, Clone)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct PlaceOrderPeggedArgs {
    pub side: Side,

    // The adjustment from the oracle price, in lots (quote lots per base lots).
    // Orders on the book may be filled at oracle + adjustment (depends on order type).
    pub price_offset_lots: i64,

    // The limit at which the pegged order shall expire.
    //
    // Example: An bid pegged to -20 with peg_limit 100 would expire if the oracle hits 121.
    pub peg_limit: i64,

    pub max_base_lots: i64,
    pub max_quote_lots_including_fees: i64,
    pub client_order_id: u64,
    pub order_type: PlaceOrderType,

    // Timestamp of when order expires
    //
    // Send 0 if you want the order to never expire.
    // Timestamps in the past mean the instruction is skipped.
    // Timestamps in the future are reduced to now + 65535s.
    pub expiry_timestamp: u64,

    pub self_trade_behavior: SelfTradeBehavior,
    // Maximum number of orders from the book to fill.
    //
    // Use this to limit compute used during order matching.
    // When the limit is reached, processing stops and the instruction succeeds.
    pub limit: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Copy, Clone)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct PlaceTakeOrderArgs {
    pub side: Side,
    pub price_lots: i64,
    pub max_base_lots: i64,
    pub max_quote_lots_including_fees: i64,
    pub order_type: PlaceOrderType,
    // Maximum number of orders from the book to fill.
    //
    // Use this to limit compute used during order matching.
    // When the limit is reached, processing stops and the instruction succeeds.
    pub limit: u8,
}

// Add security details to explorer.solana.com
#[cfg(not(feature = "no-entrypoint"))]
use {default_env::default_env, solana_security_txt::security_txt};

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    name: "OpenBook V2",
    project_url: "https://www.openbook-solana.com/",
    contacts: "email:contact@openbook-solana.com,discord:https://discord.com/invite/pX3n5Sercb",
    policy: "https://github.com/openbook-dex/openbook-v2/blob/master/SECURITY.md",
    preferred_languages: "en",
    source_code: "https://github.com/openbook-dex/openbook-v2",
    auditors: "https://github.com/openbook-dex/openbook-v2/blob/master/audit/openbook_audit.pdf",
    source_revision: default_env!("GITHUB_SHA", "Unknown source revision"),
    source_release: default_env!("GITHUB_REF_NAME", "Unknown source release")
}
