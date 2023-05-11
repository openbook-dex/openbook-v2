use std::cell::{Ref, RefMut};
use std::mem::size_of;

use anchor_lang::prelude::*;
use anchor_lang::Discriminator;
use arrayref::array_ref;

use fixed::types::I80F48;

use solana_program::program_memory::sol_memmove;
use static_assertions::const_assert_eq;

use crate::error::*;

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
const DEFAULT_MANGO_ACCOUNT_VERSION: u8 = 1;

// OpenOrdersAccount
// This struct definition is only for clients e.g. typescript, so that they can easily use out of the box
// deserialization and not have to do custom deserialization
// On chain, we would prefer zero-copying to optimize for compute
#[account]
pub struct OpenOrdersAccount {
    // ABI: Clients rely on this being at offset 40
    pub owner: Pubkey,

    pub name: [u8; 32],

    // Alternative authority/signer of transactions for a openbook account
    pub delegate: Pubkey,

    pub account_num: u32,

    pub bump: u8,

    pub padding: [u8; 3],

    /// Fees usable with the "fees buyback" feature.
    /// This tracks the ones that accrued in the current expiry interval.
    pub buyback_fees_accrued_current: u64,
    /// Fees buyback amount from the previous expiry interval.
    pub buyback_fees_accrued_previous: u64,
    /// End timestamp of the current expiry interval of the buyback fees amount.
    pub buyback_fees_expiry_timestamp: u64,

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
            delegate: Pubkey::default(),
            account_num: 0,
            bump: 0,

            padding: Default::default(),
            buyback_fees_accrued_current: 0,
            buyback_fees_accrued_previous: 0,
            buyback_fees_expiry_timestamp: 0,
            reserved: [0; 208],
            header_version: DEFAULT_MANGO_ACCOUNT_VERSION,
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
    pub name: [u8; 32],
    pub delegate: Pubkey,
    pub account_num: u32,
    pub bump: u8,
    pub padding: [u8; 3],
    pub buyback_fees_accrued_current: u64,
    pub buyback_fees_accrued_previous: u64,
    pub buyback_fees_expiry_timestamp: u64,
    pub position: Position,
    pub reserved: [u8; 208],
}

const_assert_eq!(
    size_of::<Position>(),
    size_of::<OpenOrdersAccountFixed>()
        - size_of::<Pubkey>() * 3
        - size_of::<u32>()
        - size_of::<u8>()
        - size_of::<[u8; 3]>()
        - size_of::<u64>() * 3
        - size_of::<[u8; 208]>()
);
const_assert_eq!(size_of::<OpenOrdersAccountFixed>(), 528);
const_assert_eq!(size_of::<OpenOrdersAccountFixed>() % 8, 0);

impl OpenOrdersAccountFixed {
    pub fn name(&self) -> &str {
        std::str::from_utf8(&self.name)
            .unwrap()
            .trim_matches(char::from(0))
    }

    pub fn is_owner_or_delegate(&self, ix_signer: Pubkey) -> bool {
        self.owner == ix_signer || self.delegate == ix_signer
    }

    pub fn is_delegate(&self, ix_signer: Pubkey) -> bool {
        self.delegate == ix_signer
    }

    // TODO Binye remove this code
    /// Updates the buyback_fees_* fields for staggered expiry of available amounts.
    pub fn expire_buyback_fees(&mut self, now_ts: u64, interval: u64) {
        if interval == 0 || now_ts < self.buyback_fees_expiry_timestamp {
            return;
        } else if now_ts < self.buyback_fees_expiry_timestamp + interval {
            self.buyback_fees_accrued_previous = self.buyback_fees_accrued_current;
        } else {
            self.buyback_fees_accrued_previous = 0;
        }
        self.buyback_fees_accrued_current = 0;
        self.buyback_fees_expiry_timestamp = (now_ts / interval + 1) * interval;
    }

    /// The total buyback fees amount that the account can make use of.
    pub fn buyback_fees_accrued(&self) -> u64 {
        self.buyback_fees_accrued_current
            .saturating_add(self.buyback_fees_accrued_previous)
    }

    /// Add new fees that are usable with the buyback fees feature.
    pub fn accrue_buyback_fees(&mut self, amount: u64) {
        self.buyback_fees_accrued_current =
            self.buyback_fees_accrued_current.saturating_add(amount);
    }

    /// Reduce the available buyback fees amount because it was used up.
    pub fn reduce_buyback_fees_accrued(&mut self, amount: u64) {
        if amount > self.buyback_fees_accrued_previous {
            self.buyback_fees_accrued_current = self
                .buyback_fees_accrued_current
                .saturating_sub(amount - self.buyback_fees_accrued_previous);
            self.buyback_fees_accrued_previous = 0;
        } else {
            self.buyback_fees_accrued_previous -= amount;
        }
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
            _ => {
                err!(OpenBookError::NotImplementedError).context("unexpected header version number")
            }
        }
    }

    fn initialize(dynamic_data: &mut [u8]) -> Result<()> {
        let dst: &mut [u8] = &mut dynamic_data[0..1];
        dst.copy_from_slice(&DEFAULT_MANGO_ACCOUNT_VERSION.to_le_bytes());
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
            .ok_or_else(|| error_msg!("no free perp order index"))
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
        let side = fill.taker_side().invert_side();
        let (base_change, quote_change) = fill.base_quote_change(side);
        let quote_native = I80F48::from(market.quote_lot_size) * I80F48::from(quote_change);
        let fees = quote_native.abs() * I80F48::from_num(market.maker_fee);
        if fees.is_positive() {
            self.fixed_mut()
                .accrue_buyback_fees(fees.floor().to_num::<u64>());
        }

        let locked_price = {
            let oo = self.order_by_raw_index(fill.maker_slot as usize);
            match oo.side_and_tree().order_tree() {
                BookSideOrderTree::Fixed => fill.price,
                BookSideOrderTree::OraclePegged => oo.peg_limit,
            }
        };

        let pa = &mut self.fixed_mut().position;
        pa.update_trade_stats(base_change, quote_native);
        pa.maker_volume += quote_native.abs().to_num::<u64>();

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
                Side::Bid => (fill.quantity, -locked_price * fill.quantity),
                Side::Ask => (-fill.quantity, locked_price * fill.quantity),
            };

            let base_to_free =
                I80F48::from(market.base_lot_size) * I80F48::from(base_locked_change);
            let quote_to_free =
                I80F48::from(market.quote_lot_size) * I80F48::from(quote_locked_change);

            match side {
                Side::Bid => {
                    pa.base_free_native += base_to_free.abs();
                    pa.quote_free_native += fees;
                }
                Side::Ask => {
                    pa.quote_free_native +=
                        quote_to_free.abs() + quote_native.abs() * market.maker_fee;
                }
            };
        }
        if fill.maker_out() {
            self.remove_order(fill.maker_slot as usize, base_change.abs())?;
        } else {
            match side {
                Side::Bid => {
                    pa.bids_base_lots -= base_change.abs();
                }
                Side::Ask => {
                    pa.asks_base_lots -= base_change.abs();
                }
            };
        }

        // Update market fees
        market.fees_accrued += market.maker_fee * quote_native.abs();

        Ok(())
    }

    pub fn execute_taker(&mut self, market: &mut Market, fill: &FillEvent) -> Result<()> {
        let mut pa = &mut self.fixed_mut().position;

        // Replicate the base_quote_change function but substracting the fees for an Ask
        // let (base_change, quote_change) = fill.base_quote_change(fill.taker_side());
        let base_change: i64;
        let quote_change: i64;
        match fill.taker_side() {
            Side::Bid => {
                base_change = fill.quantity;
                quote_change = -fill.price * fill.quantity;
                // TODO Binye: remove Already done on matching
                // pa.base_free_lots += base_change;
            }
            Side::Ask => {
                // remove fee from quote_change
                base_change = -fill.quantity;
                quote_change = fill.price * fill.quantity * (1 - market.taker_fee.to_num::<i64>());
                // TODO Binye remove: Already done on matching
                // pa.quote_free_lots += quote_change;
            }
        };

        pa.remove_taker_trade(base_change, quote_change);

        // fees are assessed at time of trade; no need to assess fees here
        let quote_change_native = I80F48::from(market.quote_lot_size) * I80F48::from(quote_change);

        pa.update_trade_stats(base_change, quote_change_native);

        pa.taker_volume += quote_change_native.abs().to_num::<u64>();

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
        peg_limit: i64,
    ) -> Result<()> {
        let mut position = &mut self.fixed_mut().position;
        match side {
            Side::Bid => {
                position.bids_base_lots += order.quantity;
            }
            Side::Ask => {
                position.asks_base_lots += order.quantity;
            }
        };
        let slot = order.owner_slot as usize;

        let mut oo = self.open_order_mut_by_raw_index(slot);
        oo.side_and_tree = SideAndOrderTree::new(side, order_tree).into();
        oo.id = order.key;
        oo.client_id = client_order_id;
        oo.peg_limit = peg_limit;
        Ok(())
    }

    pub fn remove_order(&mut self, slot: usize, base_quantity: i64) -> Result<()> {
        {
            let oo = self.open_order_mut_by_raw_index(slot);
            require_neq!(oo.id, 0);

            let order_side = oo.side_and_tree().side();
            let mut position = &mut self.fixed_mut().position;

            // accounting
            match order_side {
                Side::Bid => {
                    position.bids_base_lots -= base_quantity;
                }
                Side::Ask => {
                    position.asks_base_lots -= base_quantity;
                }
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

            let price = match oo.side_and_tree().order_tree() {
                BookSideOrderTree::Fixed => (oo.id >> 64) as i64,
                BookSideOrderTree::OraclePegged => oo.peg_limit,
            };

            let base_quantity_native = base_quantity * market.base_lot_size;
            let quote_quantity_native =
                base_quantity.checked_mul(price).unwrap() * market.quote_lot_size;
            let order_side = oo.side_and_tree().side();

            let position = &mut self.fixed_mut().position;

            // accounting
            match order_side {
                Side::Bid => {
                    position.quote_free_native += I80F48::from_num(quote_quantity_native);
                }
                Side::Ask => {
                    position.base_free_native += I80F48::from_num(base_quantity_native);
                }
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

// #[cfg(test)]
// mod tests {
//     use super::*;

//     fn make_test_account() -> OpenOrdersAccountValue {
//         let bytes = AnchorSerialize::try_to_vec(&OpenOrdersAccount::default_for_tests()).unwrap();
//         OpenOrdersAccountValue::from_bytes(&bytes).unwrap()
//     }

//     #[test]
//     fn test_serialization_match() {
//         let mut account = OpenOrdersAccount::default_for_tests();
//         account.group = Pubkey::new_unique();
//         account.owner = Pubkey::new_unique();
//         account.name = crate::util::fill_from_str("abcdef").unwrap();
//         account.delegate = Pubkey::new_unique();
//         account.account_num = 1;
//         account.being_liquidated = 2;
//         account.in_health_region = 3;
//         account.bump = 4;
//         account.net_deposits = 5;
//         account.health_region_begin_init_health = 7;
//         account.buyback_fees_accrued_current = 10;
//         account.buyback_fees_accrued_previous = 11;
//         account.buyback_fees_expiry_timestamp = 12;
//         account.tokens.resize(8, TokenPosition::default());
//         account.tokens[0].token_index = 8;
//         account.serum3.resize(8, Serum3Orders::default());
//         account.orderPosition.resize(8, Position::default());
//         account.orderPosition.market_index = 9;
//         account.open_orders.resize(8, OpenOrdersAccount::default());

//         let account_bytes = AnchorSerialize::try_to_vec(&account).unwrap();
//         assert_eq!(
//             8 + account_bytes.len(),
//             OpenOrdersAccount::space(8, 8, 8, 8).unwrap()
//         );

//         let account2 = OpenOrdersAccountValue::from_bytes(&account_bytes).unwrap();
//         assert_eq!(account.group, account2.fixed.group);
//         assert_eq!(account.owner, account2.fixed.owner);
//         assert_eq!(account.name, account2.fixed.name);
//         assert_eq!(account.delegate, account2.fixed.delegate);
//         assert_eq!(account.account_num, account2.fixed.account_num);
//         assert_eq!(account.being_liquidated, account2.fixed.being_liquidated);
//         assert_eq!(account.in_health_region, account2.fixed.in_health_region);
//         assert_eq!(account.bump, account2.fixed.bump);
//         assert_eq!(account.net_deposits, account2.fixed.net_deposits);
//         assert_eq!(
//             account.spot_transfers,
//             account2.fixed.spot_transfers
//         );
//         assert_eq!(
//             account.health_region_begin_init_health,
//             account2.fixed.health_region_begin_init_health
//         );
//         assert_eq!(
//             account.buyback_fees_accrued_current,
//             account2.fixed.buyback_fees_accrued_current
//         );
//         assert_eq!(
//             account.buyback_fees_accrued_previous,
//             account2.fixed.buyback_fees_accrued_previous
//         );
//         assert_eq!(
//             account.buyback_fees_expiry_timestamp,
//             account2.fixed.buyback_fees_expiry_timestamp
//         );
//         assert_eq!(
//             account.tokens[0].token_index,
//             account2.token_position_by_raw_index(0).token_index
//         );
//         assert_eq!(
//             account.serum3[0].open_orders,
//             account2.serum3_orders_by_raw_index(0).open_orders
//         );
//         assert_eq!(
//             account.orderPosition[0].market_index,
//             account2.position_by_raw_index(0).market_index
//         );
//     }

//     #[test]
//     fn test_token_positions() {
//         let mut account = make_test_account();
//         assert!(account.token_position(1).is_err());
//         assert!(account.token_position_and_raw_index(2).is_err());
//         assert!(account.token_position_mut(3).is_err());
//         assert_eq!(
//             account.token_position_by_raw_index(0).token_index,
//             TokenIndex::MAX
//         );

//         {
//             let (pos, raw, active) = account.ensure_token_position(1).unwrap();
//             assert_eq!(raw, 0);
//             assert_eq!(active, 0);
//             assert_eq!(pos.token_index, 1);
//         }
//         {
//             let (pos, raw, active) = account.ensure_token_position(7).unwrap();
//             assert_eq!(raw, 1);
//             assert_eq!(active, 1);
//             assert_eq!(pos.token_index, 7);
//         }
//         {
//             let (pos, raw, active) = account.ensure_token_position(42).unwrap();
//             assert_eq!(raw, 2);
//             assert_eq!(active, 2);
//             assert_eq!(pos.token_index, 42);
//         }

//         {
//             account.deactivate_token_position(1);

//             let (pos, raw, active) = account.ensure_token_position(42).unwrap();
//             assert_eq!(raw, 2);
//             assert_eq!(active, 1);
//             assert_eq!(pos.token_index, 42);

//             let (pos, raw, active) = account.ensure_token_position(8).unwrap();
//             assert_eq!(raw, 1);
//             assert_eq!(active, 1);
//             assert_eq!(pos.token_index, 8);
//         }

//         assert_eq!(account.active_token_positions().count(), 3);
//         account.deactivate_token_position(0);
//         assert_eq!(
//             account.token_position_by_raw_index(0).token_index,
//             TokenIndex::MAX
//         );
//         assert!(account.token_position(1).is_err());
//         assert!(account.token_position_mut(1).is_err());
//         assert!(account.token_position(8).is_ok());
//         assert!(account.token_position(42).is_ok());
//         assert_eq!(account.token_position_and_raw_index(42).unwrap().1, 2);
//         assert_eq!(account.active_token_positions().count(), 2);

//         {
//             let (pos, raw) = account.token_position_mut(42).unwrap();
//             assert_eq!(pos.token_index, 42);
//             assert_eq!(raw, 2);
//         }
//         {
//             let (pos, raw) = account.token_position_mut(8).unwrap();
//             assert_eq!(pos.token_index, 8);
//             assert_eq!(raw, 1);
//         }
//     }

//     #[test]
//     fn test_serum3_orders() {
//         let mut account = make_test_account();
//         assert!(account.serum3_orders(1).is_err());
//         assert!(account.serum3_orders_mut(3).is_err());
//         assert_eq!(
//             account.serum3_orders_by_raw_index(0).market_index,
//             Serum3MarketIndex::MAX
//         );

//         assert_eq!(account.create_serum3_orders(1).unwrap().market_index, 1);
//         assert_eq!(account.create_serum3_orders(7).unwrap().market_index, 7);
//         assert_eq!(account.create_serum3_orders(42).unwrap().market_index, 42);
//         assert!(account.create_serum3_orders(7).is_err());
//         assert_eq!(account.active_serum3_orders().count(), 3);

//         assert!(account.deactivate_serum3_orders(7).is_ok());
//         assert_eq!(
//             account.serum3_orders_by_raw_index(1).market_index,
//             Serum3MarketIndex::MAX
//         );
//         assert!(account.create_serum3_orders(8).is_ok());
//         assert_eq!(account.serum3_orders_by_raw_index(1).market_index, 8);

//         assert_eq!(account.active_serum3_orders().count(), 3);
//         assert!(account.deactivate_serum3_orders(1).is_ok());
//         assert!(account.serum3_orders(1).is_err());
//         assert!(account.serum3_orders_mut(1).is_err());
//         assert!(account.serum3_orders(8).is_ok());
//         assert!(account.serum3_orders(42).is_ok());
//         assert_eq!(account.active_serum3_orders().count(), 2);

//         assert_eq!(account.serum3_orders_mut(42).unwrap().market_index, 42);
//         assert_eq!(account.serum3_orders_mut(8).unwrap().market_index, 8);
//         assert!(account.serum3_orders_mut(7).is_err());
//     }

//     #[test]
//     fn test_positions() {
//         let mut account = make_test_account();
//         assert!(account.position(1).is_err());
//         assert!(account.position_mut(3).is_err());
//         assert_eq!(
//             account.position_by_raw_index(0).market_index,
//             MarketIndex::MAX
//         );

//         {
//             let (pos, raw) = account.ensure_position(1, 0).unwrap();
//             assert_eq!(raw, 0);
//             assert_eq!(pos.market_index, 1);
//             assert_eq!(account.token_position_mut(0).unwrap().0.in_use_count, 1);
//         }
//         {
//             let (pos, raw) = account.ensure_position(7, 0).unwrap();
//             assert_eq!(raw, 1);
//             assert_eq!(pos.market_index, 7);
//             assert_eq!(account.token_position_mut(0).unwrap().0.in_use_count, 2);
//         }
//         {
//             let (pos, raw) = account.ensure_position(42, 0).unwrap();
//             assert_eq!(raw, 2);
//             assert_eq!(pos.market_index, 42);
//             assert_eq!(account.token_position_mut(0).unwrap().0.in_use_count, 3);
//         }

//         {
//             let pos_res = account.position_mut(1);
//             assert!(pos_res.is_ok());
//             assert_eq!(pos_res.unwrap().market_index, 1)
//         }

//         {
//             let pos_res = account.position_mut(99);
//             assert!(pos_res.is_err());
//         }

//         {
//             assert!(account.deactivate_position(7, 0).is_ok());

//             let (pos, raw) = account.ensure_position(42, 0).unwrap();
//             assert_eq!(raw, 2);
//             assert_eq!(pos.market_index, 42);
//             assert_eq!(account.token_position_mut(0).unwrap().0.in_use_count, 2);

//             let (pos, raw) = account.ensure_position(8, 0).unwrap();
//             assert_eq!(raw, 1);
//             assert_eq!(pos.market_index, 8);
//             assert_eq!(account.token_position_mut(0).unwrap().0.in_use_count, 3);
//         }

//         assert_eq!(account.active_positions().count(), 3);
//         assert!(account.deactivate_position(1, 0).is_ok());
//         assert_eq!(
//             account.position_by_raw_index(0).market_index,
//             MarketIndex::MAX
//         );
//         assert!(account.position(1).is_err());
//         assert!(account.position_mut(1).is_err());
//         assert!(account.position(8).is_ok());
//         assert!(account.position(42).is_ok());
//         assert_eq!(account.active_positions().count(), 2);
//     }

//     #[test]
//     fn test_buyback_fees() {
//         let mut account = make_test_account();
//         let fixed = account.fixed_mut();
//         assert_eq!(fixed.buyback_fees_accrued(), 0);
//         fixed.expire_buyback_fees(1000, 10);
//         assert_eq!(fixed.buyback_fees_accrued(), 0);
//         assert_eq!(fixed.buyback_fees_expiry_timestamp, 1010);

//         fixed.accrue_buyback_fees(10);
//         fixed.accrue_buyback_fees(5);
//         assert_eq!(fixed.buyback_fees_accrued(), 15);
//         fixed.reduce_buyback_fees_accrued(2);
//         assert_eq!(fixed.buyback_fees_accrued(), 13);

//         fixed.expire_buyback_fees(1009, 10);
//         assert_eq!(fixed.buyback_fees_expiry_timestamp, 1010);
//         assert_eq!(fixed.buyback_fees_accrued(), 13);
//         assert_eq!(fixed.buyback_fees_accrued_current, 13);

//         fixed.expire_buyback_fees(1010, 10);
//         assert_eq!(fixed.buyback_fees_expiry_timestamp, 1020);
//         assert_eq!(fixed.buyback_fees_accrued(), 13);
//         assert_eq!(fixed.buyback_fees_accrued_previous, 13);
//         assert_eq!(fixed.buyback_fees_accrued_current, 0);

//         fixed.accrue_buyback_fees(5);
//         assert_eq!(fixed.buyback_fees_accrued(), 18);

//         fixed.reduce_buyback_fees_accrued(15);
//         assert_eq!(fixed.buyback_fees_accrued(), 3);
//         assert_eq!(fixed.buyback_fees_accrued_previous, 0);
//         assert_eq!(fixed.buyback_fees_accrued_current, 3);

//         fixed.expire_buyback_fees(1021, 10);
//         fixed.accrue_buyback_fees(1);
//         assert_eq!(fixed.buyback_fees_expiry_timestamp, 1030);
//         assert_eq!(fixed.buyback_fees_accrued_previous, 3);
//         assert_eq!(fixed.buyback_fees_accrued_current, 1);

//         fixed.expire_buyback_fees(1051, 10);
//         assert_eq!(fixed.buyback_fees_expiry_timestamp, 1060);
//         assert_eq!(fixed.buyback_fees_accrued_previous, 0);
//         assert_eq!(fixed.buyback_fees_accrued_current, 0);

//         fixed.accrue_buyback_fees(7);
//         fixed.expire_buyback_fees(1060, 10);
//         fixed.accrue_buyback_fees(5);
//         assert_eq!(fixed.buyback_fees_expiry_timestamp, 1070);
//         assert_eq!(fixed.buyback_fees_accrued(), 12);

//         fixed.reduce_buyback_fees_accrued(100);
//         assert_eq!(fixed.buyback_fees_accrued(), 0);
//     }
// }
