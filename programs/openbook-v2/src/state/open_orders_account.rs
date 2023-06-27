use anchor_lang::prelude::*;
use static_assertions::const_assert_eq;
use std::mem::size_of;

use crate::error::*;
use crate::logs::FillLog;
use crate::pod_option::PodOption;

use super::FillEvent;
use super::LeafNode;
use super::Market;
use super::OpenOrder;
use super::Side;
use super::SideAndOrderTree;
use super::{BookSideOrderTree, Position};

pub const MAX_OPEN_ORDERS: usize = 128;

#[account(zero_copy)]
#[derive(Debug)]
pub struct OpenOrdersAccount {
    // ABI: Clients rely on this being at offset 40
    pub owner: Pubkey,
    pub market: Pubkey,

    pub name: [u8; 32],

    // Alternative authority/signer of transactions for a openbook account
    pub delegate: PodOption<Pubkey>,

    pub account_num: u32,

    pub bump: u8,

    pub padding: [u8; 3],

    pub position: Position,

    pub open_orders: [OpenOrder; MAX_OPEN_ORDERS],
}

const_assert_eq!(
    size_of::<OpenOrdersAccount>(),
    size_of::<Pubkey>() * 2
        + 32
        + 40
        + 4
        + 1
        + 3
        + size_of::<Position>()
        + MAX_OPEN_ORDERS * size_of::<OpenOrder>()
);
const_assert_eq!(size_of::<OpenOrdersAccount>(), 9512);
const_assert_eq!(size_of::<OpenOrdersAccount>() % 8, 0);

impl OpenOrdersAccount {
    /// Number of bytes needed for the OpenOrdersAccount, including the discriminator
    pub fn space() -> Result<usize> {
        Ok(8 + size_of::<OpenOrdersAccount>())
    }

    pub fn name(&self) -> &str {
        std::str::from_utf8(&self.name)
            .unwrap()
            .trim_matches(char::from(0))
    }

    pub fn is_owner_or_delegate(&self, ix_signer: Pubkey) -> bool {
        let delegate_option: Option<Pubkey> = Option::from(self.delegate);
        if let Some(delegate) = delegate_option {
            return self.owner == ix_signer || delegate == ix_signer;
        }
        self.owner == ix_signer
    }

    pub fn order_by_raw_index(&self, raw_index: usize) -> &OpenOrder {
        &self.open_orders[raw_index]
    }

    pub fn all_orders(&self) -> impl Iterator<Item = &OpenOrder> {
        self.open_orders.iter()
    }

    pub fn all_orders_in_use(&self) -> impl Iterator<Item = &OpenOrder> {
        self.all_orders().filter(|oo| !oo.is_free())
    }

    pub fn next_order_slot(&self) -> Result<usize> {
        self.all_orders()
            .position(|&oo| oo.is_free())
            .ok_or_else(|| error!(OpenBookError::OpenOrdersFull))
    }

    pub fn find_order_with_client_order_id(&self, client_order_id: u64) -> Option<&OpenOrder> {
        self.all_orders_in_use()
            .find(|&oo| oo.client_id == client_order_id)
    }

    pub fn find_order_with_order_id(&self, order_id: u128) -> Option<&OpenOrder> {
        self.all_orders_in_use().find(|&oo| oo.id == order_id)
    }

    pub fn open_order_mut_by_raw_index(&mut self, raw_index: usize) -> &mut OpenOrder {
        &mut self.open_orders[raw_index]
    }

    pub fn execute_maker(&mut self, market: &mut Market, fill: &FillEvent) -> Result<()> {
        let is_self_trade = fill.maker == fill.taker;

        let side = fill.taker_side().invert_side();
        let (base_change, quote_change) = fill.base_quote_change(side);

        msg!(
            "maker price {}, quantity {}, base_change {}, quote_change {}",
            fill.price,
            fill.quantity,
            base_change,
            quote_change,
        );

        let base_native = (base_change * market.base_lot_size).unsigned_abs();
        let quote_native = (quote_change * market.quote_lot_size).unsigned_abs();
        let maker_fees = market.maker_fees_ceil(quote_native);

        let (taker_fees_to_receive, maker_fees_to_pay) = if is_self_trade {
            (0, 0)
        } else if market.maker_fee.is_positive() {
            (0, maker_fees)
        } else {
            (maker_fees, 0)
        };

        let locked_price = self
            .order_by_raw_index(fill.maker_slot as usize)
            .locked_price;

        let pa = &mut self.position;
        pa.maker_volume += quote_native;

        // Update free_lots
        {
            let quote_at_lock_price = (fill.quantity * locked_price * market.quote_lot_size) as u64;
            let quote_to_free = quote_at_lock_price - quote_native;

            let maker_fees_to_free = if market.maker_fee.is_positive() {
                let fees_at_lock_price = market.maker_fees_ceil(quote_at_lock_price);
                let fees_at_fill_price = maker_fees_to_pay;
                fees_at_lock_price - fees_at_fill_price
            } else {
                0_u64
            };

            match side {
                Side::Bid => {
                    pa.base_free_native += base_native;
                    pa.quote_free_native +=
                        quote_to_free + taker_fees_to_receive + maker_fees_to_free;
                }
                Side::Ask => {
                    pa.quote_free_native +=
                        quote_native + taker_fees_to_receive - maker_fees_to_pay;
                }
            };

            // Apply rebates
            pa.referrer_rebates_accrued += maker_fees_to_pay;
            market.referrer_rebates_accrued += maker_fees_to_pay;
        }

        if fill.maker_out() {
            self.remove_order(fill.maker_slot as usize, base_change.abs())?;
        } else {
            match side {
                Side::Bid => pa.bids_base_lots -= base_change.abs(),
                Side::Ask => pa.asks_base_lots -= base_change.abs(),
            };
        }

        // Update market fees
        if !is_self_trade {
            let fee_amount: i64 = maker_fees.try_into().unwrap();
            if market.maker_fee.is_positive() {
                market.fees_accrued += fee_amount
            } else {
                market.fees_accrued -= fee_amount
            }
        }

        //Emit event
        emit!(FillLog {
            taker_side: fill.taker_side,
            maker_slot: fill.maker_slot,
            maker_out: fill.maker_out(),
            timestamp: fill.timestamp,
            seq_num: fill.seq_num,
            maker: fill.maker,
            maker_client_order_id: fill.maker_client_order_id,
            maker_fee: market.maker_fee,
            maker_timestamp: fill.maker_timestamp,
            taker: fill.taker,
            taker_client_order_id: fill.taker_client_order_id,
            taker_fee: market.taker_fee,
            price: fill.price,
            quantity: fill.quantity,
        });
        Ok(())
    }

    /// Release funds and apply taker fees to the taker account. Account fees for referrer
    pub fn execute_taker(
        &mut self,
        market: &mut Market,
        taker_side: Side,
        base_native: u64,
        quote_native: u64,
        taker_fees: u64,
        referrer_amount: u64,
    ) {
        let pa = &mut self.position;
        match taker_side {
            Side::Bid => {
                pa.base_free_native += base_native;
                pa.taker_volume += quote_native + taker_fees;
            }
            Side::Ask => {
                pa.quote_free_native += quote_native - taker_fees;
                pa.taker_volume += quote_native;
            }
        };

        pa.referrer_rebates_accrued += referrer_amount;
        market.referrer_rebates_accrued += referrer_amount;
    }

    pub fn add_order(
        &mut self,
        side: Side,
        order_tree: BookSideOrderTree,
        order: &LeafNode,
        client_order_id: u64,
        locked_price: i64,
    ) -> Result<()> {
        let position = &mut self.position;
        match side {
            Side::Bid => position.bids_base_lots += order.quantity,
            Side::Ask => position.asks_base_lots += order.quantity,
        };
        let slot = order.owner_slot as usize;

        let oo = self.open_order_mut_by_raw_index(slot);
        oo.is_free = false.into();
        oo.side_and_tree = SideAndOrderTree::new(side, order_tree).into();
        oo.id = order.key;
        oo.client_id = client_order_id;
        oo.locked_price = locked_price;
        Ok(())
    }

    pub fn remove_order(&mut self, slot: usize, base_quantity: i64) -> Result<()> {
        {
            let oo = self.open_order_mut_by_raw_index(slot);
            assert!(!oo.is_free());

            let order_side = oo.side_and_tree().side();
            let position = &mut self.position;

            // accounting
            match order_side {
                Side::Bid => position.bids_base_lots -= base_quantity,
                Side::Ask => position.asks_base_lots -= base_quantity,
            }
        }

        // release space
        *self.open_order_mut_by_raw_index(slot) = OpenOrder::default();

        Ok(())
    }

    pub fn cancel_order(&mut self, slot: usize, base_quantity: i64, market: Market) -> Result<()> {
        {
            let oo = self.open_order_mut_by_raw_index(slot);
            let price = oo.locked_price;
            let order_side = oo.side_and_tree().side();

            let base_quantity_native = (base_quantity * market.base_lot_size) as u64;
            let mut quote_quantity_native = (base_quantity * price * market.quote_lot_size) as u64;

            if market.maker_fee.is_positive() {
                quote_quantity_native += market.maker_fees_ceil(quote_quantity_native);
            }

            let position = &mut self.position;
            match order_side {
                Side::Bid => position.quote_free_native += quote_quantity_native,
                Side::Ask => position.base_free_native += base_quantity_native,
            }
        }

        self.remove_order(slot, base_quantity)
    }
}
