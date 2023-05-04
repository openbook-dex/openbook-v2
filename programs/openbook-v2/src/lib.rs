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
pub mod state;

use fixed::types::I80F48;
use state::{MarketIndex, OracleConfigParams, PlaceOrderType, Side};

#[cfg(feature = "enable-gpl")]
pub mod instructions;

#[cfg(all(not(feature = "no-entrypoint"), not(feature = "enable-gpl")))]
compile_error!("compiling the program entrypoint without 'enable-gpl' makes no sense, enable it or use the 'cpi' or 'client' features");

#[program]
pub mod openbook_v2 {
    use super::*;

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
        fee_penalty: f32,
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

    #[allow(clippy::too_many_arguments)]
    pub fn place_order(
        ctx: Context<PlaceOrder>,
        side: Side,

        // The price in lots (quote lots per base lots)
        // - fill orders on the book up to this price or
        // - place an order on the book at this price.
        // - ignored for Market orders and potentially adjusted for PostOnlySlide orders.
        price_lots: i64,

        max_base_lots: i64,
        max_quote_lots_including_fees: i64,
        client_order_id: u64,
        order_type: PlaceOrderType,
        reduce_only: bool,

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
    ) -> Result<Option<u128>> {
        require_gte!(price_lots, 0);

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
            reduce_only,
            time_in_force,
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
        // May be -1 to denote no peg limit.
        //
        // Example: An bid pegged to -20 with peg_limit 100 would expire if the oracle hits 121.
        peg_limit: i64,

        max_base_lots: i64,
        max_quote_lots: i64,
        client_order_id: u64,
        order_type: PlaceOrderType,
        reduce_only: bool,

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
        require_gte!(peg_limit, -1);
        require_eq!(max_oracle_staleness_slots, -1); // unimplemented

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
            max_quote_lots,
            client_order_id,
            reduce_only,
            time_in_force,
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

    pub fn consume_events(ctx: Context<ConsumeEvents>, limit: usize) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::consume_events(ctx, limit)?;
        Ok(())
    }

    pub fn cancel_order(ctx: Context<CancelOrder>, order_id: u128) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::cancel_order(ctx, order_id)?;
        Ok(())
    }

    pub fn cancel_order_by_client_order_id(
        ctx: Context<CancelOrderByClientOrderId>,
        client_order_id: u64,
    ) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::cancel_order_by_client_order_id(ctx, client_order_id)?;
        Ok(())
    }

    pub fn cancel_all_orders(ctx: Context<CancelAllOrders>, limit: u8) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::cancel_all_orders(ctx, limit)?;
        Ok(())
    }

    pub fn cancel_all_orders_by_side(
        ctx: Context<CancelAllOrdersBySide>,
        side_option: Option<Side>,
        limit: u8,
    ) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::cancel_all_orders_by_side(ctx, side_option, limit)?;
        Ok(())
    }

    pub fn deposit(
        ctx: Context<Deposit>,
        base_amount_lots: u64,
        quote_amount_lots: u64,
    ) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::deposit(ctx, base_amount_lots, quote_amount_lots)?;
        Ok(())
    }

    pub fn settle_funds<'info>(ctx: Context<'_, '_, '_, 'info, SettleFunds<'info>>) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::settle_funds(ctx)?;
        Ok(())
    }

    pub fn sweep_fees(ctx: Context<SweepFees>) -> Result<()> {
        #[cfg(feature = "enable-gpl")]
        instructions::sweep_fees(ctx)?;
        Ok(())
    }

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
