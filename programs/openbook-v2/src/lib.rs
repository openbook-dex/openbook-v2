//! A central-limit order book (CLOB) program that targets the Sealevel runtime.

use anchor_lang::prelude::*;

declare_id!("BfxZj7ckfRGHxByn7aHgH2puyXhfjAUvULtRjJo4rd8X");

#[macro_use]
pub mod util;

use accounts_ix::*;

pub mod accounts_ix;
pub mod accounts_zerocopy;
pub mod error;
pub mod i80f48;
pub mod logs;
pub mod pod_option;
pub mod state;
pub mod types;

use error::*;
use fixed::types::I80F48;
use state::{MarketIndex, OracleConfigParams, PlaceOrderType, SelfTradeBehavior, Side};

#[cfg(feature = "enable-gpl")]
pub mod instructions;

#[cfg(all(not(feature = "no-entrypoint"), not(feature = "enable-gpl")))]
compile_error!("compiling the program entrypoint without 'enable-gpl' makes no sense, enable it or use the 'cpi' or 'client' features");

#[program]
pub mod openbook_v2 {
    use super::*;

    /// Create a [`Market`](crate::state::Market) for a given token pair.
    #[allow(clippy::too_many_arguments)]
    pub fn create_market(
        ctx: Context<CreateMarket>,
        market_index: MarketIndex,
        name: String,
        oracle_config: OracleConfigParams,
        quote_lot_size: i64,
        base_lot_size: i64,
        maker_fee: f32,
        taker_fee: f32,
        fee_penalty: u64,
        time_expiry: i64,
    ) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::create_market(
            ctx,
            market_index,
            name,
            oracle_config,
            quote_lot_size,
            base_lot_size,
            maker_fee,
            taker_fee,
            fee_penalty,
            time_expiry,
        )?;
        Ok(())
    }

    pub fn init_open_orders(
        ctx: Context<InitOpenOrders>,
        account_num: u32,
        open_orders_count: u8,
    ) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::init_open_orders(ctx, account_num, open_orders_count)?;
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
    #[allow(clippy::too_many_arguments)]
    pub fn place_order(
        ctx: Context<PlaceOrder>,
        side: Side,
        price_lots: i64,
        max_base_lots: i64,
        max_quote_lots_including_fees: i64,
        client_order_id: u64,
        order_type: PlaceOrderType,
        self_trade_behavior: SelfTradeBehavior,
        expiry_timestamp: u64,
        limit: u8,
    ) -> Result<Option<u128>> {
        require_gte!(price_lots, 1, OpenBookError::InvalidPriceLots);

        use crate::state::{Order, OrderParams};
        let time_in_force = match Order::tif_from_expiry(expiry_timestamp) {
            Some(t) => t,
            None => {
                msg!("Order is already expired");
                return Ok(None);
            }
        };
        let order = Order {
            side,
            max_base_lots,
            max_quote_lots_including_fees,
            client_order_id,
            time_in_force,
            self_trade_behavior,
            params: match order_type {
                PlaceOrderType::Market => OrderParams::Market,
                PlaceOrderType::ImmediateOrCancel => OrderParams::ImmediateOrCancel { price_lots },
                _ => OrderParams::Fixed {
                    price_lots,
                    order_type: order_type.to_post_order_type()?,
                },
            },
        };
        #[cfg(feature = "enable-gpl")]
        return instructions::place_order(ctx, order, limit);

        #[cfg(not(feature = "enable-gpl"))]
        Ok(None)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn place_order_pegged(
        ctx: Context<PlaceOrder>,
        side: Side,

        // The adjustment from the oracle price, in lots (quote lots per base lots).
        // Orders on the book may be filled at oracle + adjustment (depends on order type).
        price_offset_lots: i64,

        // The limit at which the pegged order shall expire.
        //
        // Example: An bid pegged to -20 with peg_limit 100 would expire if the oracle hits 121.
        peg_limit: i64,

        max_base_lots: i64,
        max_quote_lots_including_fees: i64,
        client_order_id: u64,
        order_type: PlaceOrderType,
        self_trade_behavior: SelfTradeBehavior,

        // Timestamp of when order expires
        //
        // Send 0 if you want the order to never expire.
        // Timestamps in the past mean the instruction is skipped.
        // Timestamps in the future are reduced to now + 65535s.
        expiry_timestamp: u64,

        // Maximum number of orders from the book to fill.
        //
        // Use this to limit compute used during order matching.
        // When the limit is reached, processing stops and the instruction succeeds.
        limit: u8,

        // Oracle staleness limit, in slots. Set to -1 to disable.
        //
        // WARNING: Not currently implemented.
        max_oracle_staleness_slots: i32,
    ) -> Result<Option<u128>> {
        require_gt!(peg_limit, 0, OpenBookError::InvalidPegLimit);
        require_eq!(
            max_oracle_staleness_slots,
            -1,
            OpenBookError::UnimplementedStaleness
        );

        use crate::state::{Order, OrderParams};
        let time_in_force = match Order::tif_from_expiry(expiry_timestamp) {
            Some(t) => t,
            None => {
                msg!("Order is already expired");
                return Ok(None);
            }
        };

        let order = Order {
            side,
            max_base_lots,
            max_quote_lots_including_fees,
            client_order_id,
            time_in_force,
            self_trade_behavior,
            params: OrderParams::OraclePegged {
                price_offset_lots,
                order_type: order_type.to_post_order_type()?,
                peg_limit,
                max_oracle_staleness_slots,
            },
        };
        #[cfg(feature = "enable-gpl")]
        return instructions::place_order(ctx, order, limit);

        #[cfg(not(feature = "enable-gpl"))]
        Ok(None)
    }

    /// Place an order that shall take existing liquidity off of the book, not
    /// add a new order off the book.
    ///
    /// This type of order allows for instant token settlement for the taker.
    #[allow(clippy::too_many_arguments)]
    pub fn place_take_order<'info>(
        ctx: Context<'_, '_, '_, 'info, PlaceTakeOrder<'info>>,
        side: Side,
        price_lots: i64,
        max_base_lots: i64,
        max_quote_lots_including_fees: i64,
        client_order_id: u64,
        order_type: PlaceOrderType,
        self_trade_behavior: SelfTradeBehavior,
        limit: u8,
    ) -> Result<Option<u128>> {
        require_gte!(price_lots, 1, OpenBookError::InvalidPriceLots);

        use crate::state::{Order, OrderParams};
        require!(
            order_type == PlaceOrderType::Market || order_type == PlaceOrderType::ImmediateOrCancel,
            OpenBookError::InvalidOrderType
        );
        let order = Order {
            side,
            max_base_lots,
            max_quote_lots_including_fees,
            client_order_id,
            time_in_force: 0,
            self_trade_behavior,
            params: match order_type {
                PlaceOrderType::Market => OrderParams::Market,
                PlaceOrderType::ImmediateOrCancel => OrderParams::ImmediateOrCancel { price_lots },
                // never goes here, order type already checked
                _ => panic!(),
            },
        };
        #[cfg(feature = "enable-gpl")]
        return instructions::place_take_order(ctx, order, limit);

        #[cfg(not(feature = "enable-gpl"))]
        Ok(None)
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
    pub fn consume_events(ctx: Context<ConsumeEvents>, limit: usize) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::consume_events(ctx, limit, None)?;
        Ok(())
    }

    /// Process the [events](crate::state::AnyEvent) at the given positions.
    pub fn consume_given_events(ctx: Context<ConsumeEvents>, slots: Vec<usize>) -> Result<()> {
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
        ctx: Context<CancelOrderByClientOrderId>,
        client_order_id: u64,
    ) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::cancel_order_by_client_order_id(ctx, client_order_id)?;
        Ok(())
    }

    /// Cancel up to `limit` orders.
    pub fn cancel_all_orders(ctx: Context<CancelAllOrders>, limit: u8) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::cancel_all_orders(ctx, limit)?;
        Ok(())
    }

    /// Cancel up to `limit` orders on a single side of the book.
    pub fn cancel_all_orders_by_side(
        ctx: Context<CancelAllOrdersBySide>,
        side_option: Option<Side>,
        limit: u8,
    ) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::cancel_all_orders_by_side(ctx, side_option, limit)?;
        Ok(())
    }

    /// Desposit a certain amount of `base_amount_lots` and `quote_amount_lots`
    /// into one's [`Position`](crate::state::Position).
    ///
    /// Makers might wish to `deposit`, rather than have actual tokens moved for
    /// each trade, in order to reduce CUs.
    pub fn deposit(
        ctx: Context<Deposit>,
        base_amount_lots: u64,
        quote_amount_lots: u64,
    ) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::deposit(ctx, base_amount_lots, quote_amount_lots)?;
        Ok(())
    }

    /// Withdraw any available tokens.
    pub fn settle_funds<'info>(ctx: Context<'_, '_, '_, 'info, SettleFunds<'info>>) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::settle_funds(ctx)?;
        Ok(())
    }

    /// Sweep fees, as a [`Market`](crate::state::Market)'s admin.
    pub fn sweep_fees(ctx: Context<SweepFees>) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::sweep_fees(ctx)?;
        Ok(())
    }

    pub fn set_delegate(ctx: Context<SetDelegate>) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::set_delegate(ctx)?;
        Ok(())
    }

    /// Set market to expired before pruning orders and closing the market
    pub fn set_market_expired(ctx: Context<SetMarketExpired>) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::set_market_expired(ctx)?;
        Ok(())
    }

    pub fn prune_orders(ctx: Context<PruneOrders>, limit: u8) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::prune_orders(ctx, limit)?;
        Ok(())
    }

    /// Close a [`Market`](crate::state::Market).
    pub fn close_market(ctx: Context<CloseMarket>) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::close_market(ctx)?;
        Ok(())
    }

    // todo:
    // ckamm: generally, using an I80F48 arg will make it harder to call
    // because generic anchor clients won't know how to deal with it
    // and it's tricky to use in typescript generally
    // lets do an interface pass later
    pub fn stub_oracle_create(ctx: Context<StubOracleCreate>, price: I80F48) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::stub_oracle_create(ctx, price)?;
        Ok(())
    }

    pub fn stub_oracle_close(ctx: Context<StubOracleClose>) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::stub_oracle_close(ctx)?;
        Ok(())
    }

    pub fn stub_oracle_set(ctx: Context<StubOracleSet>, price: I80F48) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::stub_oracle_set(ctx, price)?;
        Ok(())
    }
}
