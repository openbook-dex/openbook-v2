use anchor_lang::{prelude::*, Discriminator};
use arrayref::array_ref;
use solana_program::program_memory::sol_memmove;
use static_assertions::const_assert_eq;
use std::cell::{Ref, RefMut};
use std::mem::size_of;

use crate::error::*;
use crate::logs::FillLog;
use crate::pod_option::PodOption;
use crate::state::FEES_UNIT;

use super::FillEvent;
use super::LeafNode;
use super::Market;
use super::OpenOrder;
use super::Side;
use super::{dynamic_account::*, SideAndOrderTree};
use super::{BookSideOrderTree, Position};

type BorshVecLength = u32;
const BORSH_VEC_PADDING_BYTES: usize = 4;
const BORSH_VEC_SIZE_BYTES: usize = 4;
const DEFAULT_OPEN_ORDERS_ACCOUNT_VERSION: u8 = 1;

// OpenOrdersAccount
// This struct definition is only for clients e.g. typescript, so that they can easily use out of the box
// deserialization and not have to do custom deserialization
// On chain, we would prefer zero-copying to optimize for compute
#[account]
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
    pub reserved: [u8; 208],

    // dynamic
    pub header_version: u8,
    pub padding3: [u8; 7],
    pub padding4: u32, // for open_orders to be aligned
    pub open_orders: Vec<OpenOrder>,
}

impl OpenOrdersAccount {
    pub fn default_for_tests() -> Self {
        Self {
            name: Default::default(),
            owner: Pubkey::default(),
            market: Pubkey::default(),
            delegate: PodOption::default(),
            account_num: 0,
            bump: 0,

            padding: Default::default(),
            reserved: [0; 208],
            header_version: DEFAULT_OPEN_ORDERS_ACCOUNT_VERSION,
            padding3: Default::default(),
            padding4: Default::default(),
            position: Position::default(),
            open_orders: vec![OpenOrder::default(); 6],
        }
    }

    /// Number of bytes needed for the OpenOrdersAccount, including the discriminator
    pub fn space(oo_count: u8) -> Result<usize> {
        require_gte!(64, oo_count);

        Ok(8 + size_of::<OpenOrdersAccountFixed>() + Self::dynamic_size(oo_count))
    }

    pub fn dynamic_oo_vec_offset() -> usize {
        8 // header version + padding
          + BORSH_VEC_PADDING_BYTES
    }

    pub fn dynamic_size(oo_count: u8) -> usize {
        Self::dynamic_oo_vec_offset()
            + BORSH_VEC_SIZE_BYTES
            + size_of::<OpenOrder>() * usize::from(oo_count)
    }
}

// OpenOrders Account fixed part for easy zero copy deserialization
#[zero_copy]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
pub struct OpenOrdersAccountFixed {
    pub owner: Pubkey,
    pub market: Pubkey,
    pub name: [u8; 32],
    pub delegate: PodOption<Pubkey>,
    pub account_num: u32,
    pub bump: u8,
    pub padding: [u8; 3],
    pub position: Position,
    pub reserved: [u8; 208],
}

const_assert_eq!(
    size_of::<Position>(),
    size_of::<OpenOrdersAccountFixed>()
        - size_of::<Pubkey>() * 3
        - 40
        - size_of::<u32>()
        - size_of::<u8>()
        - size_of::<[u8; 3]>()
        - size_of::<[u8; 208]>()
);
const_assert_eq!(size_of::<OpenOrdersAccountFixed>(), 504);
const_assert_eq!(size_of::<OpenOrdersAccountFixed>() % 8, 0);

impl OpenOrdersAccountFixed {
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
}

impl Owner for OpenOrdersAccountFixed {
    fn owner() -> Pubkey {
        OpenOrdersAccount::owner()
    }
}

impl Discriminator for OpenOrdersAccountFixed {
    const DISCRIMINATOR: [u8; 8] = OpenOrdersAccount::DISCRIMINATOR;
}

impl anchor_lang::ZeroCopy for OpenOrdersAccountFixed {}

#[derive(Clone)]
pub struct OpenOrdersAccountDynamicHeader {
    pub oo_count: u8,
}

impl DynamicHeader for OpenOrdersAccountDynamicHeader {
    fn from_bytes(dynamic_data: &[u8]) -> Result<Self> {
        let header_version = u8::from_le_bytes(*array_ref![dynamic_data, 0, size_of::<u8>()]);

        match header_version {
            1 => {
                let oo_count = u8::try_from(BorshVecLength::from_le_bytes(*array_ref![
                    dynamic_data,
                    OpenOrdersAccount::dynamic_oo_vec_offset(),
                    BORSH_VEC_SIZE_BYTES
                ]))
                .unwrap();

                Ok(Self { oo_count })
            }
            _ => err!(OpenBookError::HeaderVersionNotKnown)
                .context("unexpected header version number"),
        }
    }

    fn initialize(dynamic_data: &mut [u8]) -> Result<()> {
        let dst: &mut [u8] = &mut dynamic_data[0..1];
        dst.copy_from_slice(&DEFAULT_OPEN_ORDERS_ACCOUNT_VERSION.to_le_bytes());
        Ok(())
    }
}

fn get_helper<T: bytemuck::Pod>(data: &[u8], index: usize) -> &T {
    bytemuck::from_bytes(&data[index..index + size_of::<T>()])
}

fn get_helper_mut<T: bytemuck::Pod>(data: &mut [u8], index: usize) -> &mut T {
    bytemuck::from_bytes_mut(&mut data[index..index + size_of::<T>()])
}

impl OpenOrdersAccountDynamicHeader {
    fn oo_offset(&self, raw_index: usize) -> usize {
        OpenOrdersAccount::dynamic_oo_vec_offset()
            + BORSH_VEC_SIZE_BYTES
            + raw_index * size_of::<OpenOrder>()
    }

    pub fn oo_count(&self) -> usize {
        self.oo_count.into()
    }
}

/// Fully owned OpenOrdersAccount, useful for tests
pub type OpenOrdersAccountValue =
    DynamicAccount<OpenOrdersAccountDynamicHeader, OpenOrdersAccountFixed, Vec<u8>>;

/// Full reference type, useful for borrows
pub type OpenOrdersAccountRef<'a> =
    DynamicAccount<&'a OpenOrdersAccountDynamicHeader, &'a OpenOrdersAccountFixed, &'a [u8]>;
/// Full reference type, useful for borrows
pub type OpenOrdersAccountRefMut<'a> = DynamicAccount<
    &'a mut OpenOrdersAccountDynamicHeader,
    &'a mut OpenOrdersAccountFixed,
    &'a mut [u8],
>;

/// Useful when loading from bytes
pub type OpenOrdersAccountLoadedRef<'a> =
    DynamicAccount<OpenOrdersAccountDynamicHeader, &'a OpenOrdersAccountFixed, &'a [u8]>;
/// Useful when loading from RefCell, like from AccountInfo
pub type OpenOrdersAccountLoadedRefCell<'a> =
    DynamicAccount<OpenOrdersAccountDynamicHeader, Ref<'a, OpenOrdersAccountFixed>, Ref<'a, [u8]>>;
/// Useful when loading from RefCell, like from AccountInfo
pub type OpenOrdersAccountLoadedRefCellMut<'a> = DynamicAccount<
    OpenOrdersAccountDynamicHeader,
    RefMut<'a, OpenOrdersAccountFixed>,
    RefMut<'a, [u8]>,
>;

impl OpenOrdersAccountValue {
    // bytes without discriminator
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let (fixed, dynamic) = bytes.split_at(size_of::<OpenOrdersAccountFixed>());
        Ok(Self {
            fixed: *bytemuck::from_bytes(fixed),
            header: OpenOrdersAccountDynamicHeader::from_bytes(dynamic)?,
            dynamic: dynamic.to_vec(),
        })
    }
}

impl<'a> OpenOrdersAccountLoadedRef<'a> {
    // bytes without discriminator
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self> {
        let (fixed, dynamic) = bytes.split_at(size_of::<OpenOrdersAccountFixed>());
        Ok(Self {
            fixed: bytemuck::from_bytes(fixed),
            header: OpenOrdersAccountDynamicHeader::from_bytes(dynamic)?,
            dynamic,
        })
    }
}

// This generic impl covers OpenOrdersAccountRef, OpenOrdersAccountRefMut and other
// DynamicAccount variants that allow read access.
impl<
        Header: DerefOrBorrow<OpenOrdersAccountDynamicHeader>,
        Fixed: DerefOrBorrow<OpenOrdersAccountFixed>,
        Dynamic: DerefOrBorrow<[u8]>,
    > DynamicAccount<Header, Fixed, Dynamic>
{
    fn header(&self) -> &OpenOrdersAccountDynamicHeader {
        self.header.deref_or_borrow()
    }

    pub fn header_version(&self) -> &u8 {
        get_helper(self.dynamic(), 0)
    }

    pub fn fixed(&self) -> &OpenOrdersAccountFixed {
        self.fixed.deref_or_borrow()
    }

    fn dynamic(&self) -> &[u8] {
        self.dynamic.deref_or_borrow()
    }

    pub fn order_by_raw_index(&self, raw_index: usize) -> &OpenOrder {
        get_helper(self.dynamic(), self.header().oo_offset(raw_index))
    }

    pub fn all_orders(&self) -> impl Iterator<Item = &OpenOrder> {
        (0..self.header().oo_count()).map(|i| self.order_by_raw_index(i))
    }

    pub fn next_order_slot(&self) -> Result<usize> {
        self.all_orders()
            .position(|&oo| oo.id == 0)
            .ok_or_else(|| error!(OpenBookError::OpenOrdersFull))
    }

    pub fn find_order_with_client_order_id(&self, client_order_id: u64) -> Option<&OpenOrder> {
        self.all_orders()
            .find(|&oo| oo.client_id == client_order_id)
    }

    pub fn find_order_with_order_id(&self, order_id: u128) -> Option<&OpenOrder> {
        self.all_orders().find(|&oo| oo.id == order_id)
    }

    pub fn borrow(&self) -> OpenOrdersAccountRef {
        OpenOrdersAccountRef {
            header: self.header(),
            fixed: self.fixed(),
            dynamic: self.dynamic(),
        }
    }
}

impl<
        Header: DerefOrBorrowMut<OpenOrdersAccountDynamicHeader>
            + DerefOrBorrow<OpenOrdersAccountDynamicHeader>,
        Fixed: DerefOrBorrowMut<OpenOrdersAccountFixed> + DerefOrBorrow<OpenOrdersAccountFixed>,
        Dynamic: DerefOrBorrowMut<[u8]> + DerefOrBorrow<[u8]>,
    > DynamicAccount<Header, Fixed, Dynamic>
{
    fn header_mut(&mut self) -> &mut OpenOrdersAccountDynamicHeader {
        self.header.deref_or_borrow_mut()
    }
    pub fn fixed_mut(&mut self) -> &mut OpenOrdersAccountFixed {
        self.fixed.deref_or_borrow_mut()
    }
    fn dynamic_mut(&mut self) -> &mut [u8] {
        self.dynamic.deref_or_borrow_mut()
    }

    pub fn borrow_mut(&mut self) -> OpenOrdersAccountRefMut {
        OpenOrdersAccountRefMut {
            header: self.header.deref_or_borrow_mut(),
            fixed: self.fixed.deref_or_borrow_mut(),
            dynamic: self.dynamic.deref_or_borrow_mut(),
        }
    }

    pub fn open_order_mut_by_raw_index(&mut self, raw_index: usize) -> &mut OpenOrder {
        let offset = self.header().oo_offset(raw_index);
        get_helper_mut(self.dynamic_mut(), offset)
    }

    pub fn execute_maker(&mut self, market: &mut Market, fill: &FillEvent) -> Result<()> {
        let is_self_trade = fill.maker == fill.taker;

        let side = fill.taker_side().invert_side();
        let (base_change, quote_change) = fill.base_quote_change(side);
        let quote_native_abs = (market.quote_lot_size * quote_change).unsigned_abs();
        let fees = if is_self_trade || market.maker_fee.is_positive() {
            // Maker pays fee. Fees already subtracted before sending to the book
            0
        } else {
            ((quote_native_abs as i128) * (market.maker_fee as i128) + (FEES_UNIT - 1i128))
                .checked_div(FEES_UNIT)
                .unwrap() as u64
        };

        let price = self
            .order_by_raw_index(fill.maker_slot as usize)
            .locked_price;

        let pa = &mut self.fixed_mut().position;
        pa.maker_volume += quote_native_abs;

        msg!(
            " maker price {}, quantity {}, base_change {}, quote_change {}",
            fill.price,
            fill.quantity,
            base_change,
            quote_change,
        );

        // Update free_lots
        {
            let (base_locked_change, quote_locked_change): (i64, i64) = match side {
                Side::Bid => (fill.quantity, -price * fill.quantity),
                Side::Ask => (-fill.quantity, price * fill.quantity),
            };

            let base_to_free = (market.base_lot_size * base_locked_change.abs()) as u64;
            let quote_to_free = (market.quote_lot_size * quote_locked_change.abs()) as u64;

            match side {
                Side::Bid => {
                    pa.base_free_native += base_to_free;
                    pa.quote_free_native += fees;
                }
                Side::Ask => {
                    let maker_fees = if market.maker_fee.is_positive() {
                        ((quote_locked_change as i128)
                            * (market.quote_lot_size as i128)
                            * (market.maker_fee as i128)
                            + (FEES_UNIT - 1i128))
                            .checked_div(FEES_UNIT)
                            .unwrap() as u64
                    } else {
                        0
                    };
                    pa.quote_free_native += quote_to_free + fees - maker_fees;
                }
            };

            if !is_self_trade && market.maker_fee.is_positive() {
                // Apply rebates
                let maker_fees = ((quote_to_free as i128) * (market.maker_fee as i128)
                    + (FEES_UNIT - 1i128))
                    .checked_div(FEES_UNIT)
                    .unwrap() as u64;

                pa.referrer_rebates_accrued += maker_fees;
                market.referrer_rebates_accrued += maker_fees;
            }
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
            let fee_amount: i64 = {
                let amount = (quote_native_abs as i128) * (market.maker_fee as i128);
                if market.maker_fee.is_positive() {
                    (amount + (FEES_UNIT - 1i128))
                        .checked_div(FEES_UNIT)
                        .unwrap() as i64
                } else {
                    (amount - (FEES_UNIT - 1i128))
                        .checked_div(FEES_UNIT)
                        .unwrap() as i64
                }
            };
            market.fees_accrued += fee_amount;
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
    pub fn release_funds_apply_fees(
        &mut self,
        taker_side: Side,
        market: &mut Market,
        base_native: u64,
        quote_native: u64,
        taker_fees: u64,
    ) -> Result<()> {
        let pa = &mut self.fixed_mut().position;
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

        // Referrer rebates
        pa.referrer_rebates_accrued += market.referrer_taker_rebate(quote_native);
        market.referrer_rebates_accrued += market.referrer_taker_rebate(quote_native);

        Ok(())
    }

    fn write_oo_length(&mut self) {
        let oo_offset = self.header().oo_offset(0);

        let count = self.header().oo_count;
        let dst: &mut [u8] = &mut self.dynamic_mut()[oo_offset - BORSH_VEC_SIZE_BYTES..oo_offset];
        dst.copy_from_slice(&BorshVecLength::from(count).to_le_bytes());
    }

    pub fn expand_dynamic_content(&mut self, new_oo_count: u8) -> Result<()> {
        require_gte!(new_oo_count, self.header().oo_count);

        // create a temp copy to compute new starting offsets
        let new_header = OpenOrdersAccountDynamicHeader {
            oo_count: new_oo_count,
        };
        let old_header = self.header().clone();
        let dynamic = self.dynamic_mut();

        // expand dynamic components by first moving existing positions, and then setting new ones to defaults

        // perp oo
        if old_header.oo_count() > 0 {
            unsafe {
                sol_memmove(
                    &mut dynamic[new_header.oo_offset(0)],
                    &mut dynamic[old_header.oo_offset(0)],
                    size_of::<OpenOrder>() * old_header.oo_count(),
                );
            }
        }
        for i in old_header.oo_count..new_oo_count {
            *get_helper_mut(dynamic, new_header.oo_offset(i.into())) = OpenOrder::default();
        }

        // update the already-parsed header
        *self.header_mut() = new_header;

        // write new lengths to the dynamic data (uses header)
        self.write_oo_length();

        Ok(())
    }

    pub fn add_order(
        &mut self,
        side: Side,
        order_tree: BookSideOrderTree,
        order: &LeafNode,
        client_order_id: u64,
        locked_price: i64,
    ) -> Result<()> {
        let position = &mut self.fixed_mut().position;
        match side {
            Side::Bid => position.bids_base_lots += order.quantity,
            Side::Ask => position.asks_base_lots += order.quantity,
        };
        let slot = order.owner_slot as usize;

        let oo = self.open_order_mut_by_raw_index(slot);
        oo.side_and_tree = SideAndOrderTree::new(side, order_tree).into();
        oo.id = order.key;
        oo.client_id = client_order_id;
        oo.locked_price = locked_price;
        Ok(())
    }

    pub fn remove_order(&mut self, slot: usize, base_quantity: i64) -> Result<()> {
        {
            let oo = self.open_order_mut_by_raw_index(slot);
            require_neq!(oo.id, 0);

            let order_side = oo.side_and_tree().side();
            let position = &mut self.fixed_mut().position;

            // accounting
            match order_side {
                Side::Bid => position.bids_base_lots -= base_quantity,
                Side::Ask => position.asks_base_lots -= base_quantity,
            }
        }

        // release space
        let oo = self.open_order_mut_by_raw_index(slot);
        oo.side_and_tree = SideAndOrderTree::BidFixed.into();
        oo.id = 0;
        oo.client_id = 0;
        Ok(())
    }

    pub fn cancel_order(&mut self, slot: usize, base_quantity: i64, market: Market) -> Result<()> {
        {
            let oo = self.open_order_mut_by_raw_index(slot);

            let price = oo.locked_price;
            let order_side = oo.side_and_tree().side();

            let mut base_quantity_native = (base_quantity * market.base_lot_size) as u64;
            let mut quote_quantity_native =
                (base_quantity.checked_mul(price).unwrap() * market.quote_lot_size) as u64;

            let position = &mut self.fixed_mut().position;

            // If maker fees, give back fees to user
            if market.maker_fee.is_positive() {
                let fees = ((quote_quantity_native as i128) * (market.maker_fee as i128)
                    + (FEES_UNIT - 1i128))
                    .checked_div(FEES_UNIT)
                    .unwrap() as u64;
                quote_quantity_native += fees;
                base_quantity_native += fees / (price as u64);
            }

            // accounting
            match order_side {
                Side::Bid => position.quote_free_native += quote_quantity_native,
                Side::Ask => position.base_free_native += base_quantity_native,
            }
        }

        self.remove_order(slot, base_quantity)
    }
}

/// Trait to allow a AccountLoader<OpenOrdersAccountFixed> to create an accessor for the full account.
pub trait OpenOrdersLoader<'a> {
    fn load_full(self) -> Result<OpenOrdersAccountLoadedRefCell<'a>>;
    fn load_full_mut(self) -> Result<OpenOrdersAccountLoadedRefCellMut<'a>>;
    fn load_full_init(self) -> Result<OpenOrdersAccountLoadedRefCellMut<'a>>;
}

impl<'a, 'info: 'a> OpenOrdersLoader<'a> for &'a AccountLoader<'info, OpenOrdersAccountFixed> {
    fn load_full(self) -> Result<OpenOrdersAccountLoadedRefCell<'a>> {
        // Error checking
        self.load()?;

        let data = self.as_ref().try_borrow_data()?;
        let header = OpenOrdersAccountDynamicHeader::from_bytes(
            &data[8 + size_of::<OpenOrdersAccountFixed>()..],
        )?;
        let (_, data) = Ref::map_split(data, |d| d.split_at(8));
        let (fixed_bytes, dynamic) =
            Ref::map_split(data, |d| d.split_at(size_of::<OpenOrdersAccountFixed>()));
        Ok(OpenOrdersAccountLoadedRefCell {
            header,
            fixed: Ref::map(fixed_bytes, |b| bytemuck::from_bytes(b)),
            dynamic,
        })
    }

    fn load_full_mut(self) -> Result<OpenOrdersAccountLoadedRefCellMut<'a>> {
        // Error checking
        self.load_mut()?;

        let data = self.as_ref().try_borrow_mut_data()?;
        let header = OpenOrdersAccountDynamicHeader::from_bytes(
            &data[8 + size_of::<OpenOrdersAccountFixed>()..],
        )?;
        let (_, data) = RefMut::map_split(data, |d| d.split_at_mut(8));
        let (fixed_bytes, dynamic) = RefMut::map_split(data, |d| {
            d.split_at_mut(size_of::<OpenOrdersAccountFixed>())
        });
        Ok(OpenOrdersAccountLoadedRefCellMut {
            header,
            fixed: RefMut::map(fixed_bytes, |b| bytemuck::from_bytes_mut(b)),
            dynamic,
        })
    }

    fn load_full_init(self) -> Result<OpenOrdersAccountLoadedRefCellMut<'a>> {
        // Error checking
        self.load_init()?;

        {
            let mut data = self.as_ref().try_borrow_mut_data()?;

            let disc_bytes: &mut [u8] = &mut data[0..8];
            disc_bytes.copy_from_slice(bytemuck::bytes_of(&(OpenOrdersAccount::discriminator())));

            OpenOrdersAccountDynamicHeader::initialize(
                &mut data[8 + size_of::<OpenOrdersAccountFixed>()..],
            )?;
        }

        self.load_full_mut()
    }
}
