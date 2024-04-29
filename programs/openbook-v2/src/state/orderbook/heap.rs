use crate::error::OpenBookError;
use anchor_lang::prelude::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use static_assertions::const_assert_eq;
use std::mem::size_of;

use super::Side;

pub const MAX_NUM_EVENTS: u16 = 600;
pub const NO_NODE: u16 = u16::MAX;

/// Container for the different EventTypes.
///
/// Events are stored in a fixed-array of nodes. Free nodes are connected by a single-linked list
/// starting at free_head while used nodes form a circular doubly-linked list starting at
/// used_head.
#[account(zero_copy)]
pub struct EventHeap {
    pub header: EventHeapHeader,
    pub nodes: [EventNode; MAX_NUM_EVENTS as usize],
    pub reserved: [u8; 64],
}
const_assert_eq!(
    std::mem::size_of::<EventHeap>(),
    16 + MAX_NUM_EVENTS as usize * (EVENT_SIZE + 8) + 64
);
// Costs 0.636 SOL to create this account
const_assert_eq!(std::mem::size_of::<EventHeap>(), 91280);
const_assert_eq!(std::mem::size_of::<EventHeap>() % 8, 0);

impl EventHeap {
    pub fn init(&mut self) {
        self.header = EventHeapHeader {
            free_head: 0,
            used_head: NO_NODE,
            count: 0,
            seq_num: 0,
            _padd: Default::default(),
        };

        for i in 0..MAX_NUM_EVENTS {
            self.nodes[i as usize].next = i + 1;
            self.nodes[i as usize].prev = NO_NODE;
        }
        self.nodes[MAX_NUM_EVENTS as usize - 1].next = NO_NODE;
    }

    pub fn len(&self) -> usize {
        self.header.count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_full(&self) -> bool {
        self.len() == self.nodes.len()
    }

    pub fn front(&self) -> Option<&AnyEvent> {
        if self.is_empty() {
            None
        } else {
            Some(&self.nodes[self.header.used_head()].event)
        }
    }

    pub fn at_slot(&self, slot: usize) -> Option<&AnyEvent> {
        if slot >= self.nodes.len() || self.nodes[slot].is_free() {
            None
        } else {
            Some(&self.nodes[slot].event)
        }
    }

    pub fn push_back(&mut self, value: AnyEvent) {
        assert!(!self.is_full());

        let slot = self.header.free_head;
        self.header.free_head = self.nodes[slot as usize].next;

        let new_next: u16;
        let new_prev: u16;

        if self.is_empty() {
            new_next = slot;
            new_prev = slot;

            self.header.used_head = slot;
        } else {
            new_next = self.header.used_head;
            new_prev = self.nodes[new_next as usize].prev;

            self.nodes[new_prev as usize].next = slot;
            self.nodes[new_next as usize].prev = slot;
        }

        self.header.incr_count();
        self.header.incr_event_id();
        self.nodes[slot as usize].event = value;
        self.nodes[slot as usize].next = new_next;
        self.nodes[slot as usize].prev = new_prev;
    }

    pub fn pop_front(&mut self) -> Result<AnyEvent> {
        self.delete_slot(self.header.used_head())
    }

    pub fn delete_slot(&mut self, slot: usize) -> Result<AnyEvent> {
        if slot >= self.nodes.len() || self.is_empty() || self.nodes[slot].is_free() {
            return Err(OpenBookError::SomeError.into());
        }

        let prev_slot = self.nodes[slot].prev;
        let next_slot = self.nodes[slot].next;
        let next_free = self.header.free_head;

        self.nodes[prev_slot as usize].next = next_slot;
        self.nodes[next_slot as usize].prev = prev_slot;

        if self.header.count() == 1 {
            self.header.used_head = NO_NODE;
        } else if self.header.used_head() == slot {
            self.header.used_head = next_slot;
        };

        self.header.decr_count();
        self.header.free_head = slot.try_into().unwrap();
        self.nodes[slot].next = next_free;
        self.nodes[slot].prev = NO_NODE;

        Ok(self.nodes[slot].event)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&AnyEvent, usize)> {
        EventHeapIterator {
            heap: self,
            index: 0,
            slot: self.header.used_head(),
        }
    }
}

struct EventHeapIterator<'a> {
    heap: &'a EventHeap,
    index: usize,
    slot: usize,
}

impl<'a> Iterator for EventHeapIterator<'a> {
    type Item = (&'a AnyEvent, usize);
    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.heap.len() {
            None
        } else {
            let current_slot = self.slot;
            self.slot = self.heap.nodes[current_slot].next as usize;
            self.index += 1;
            Some((&self.heap.nodes[current_slot].event, current_slot))
        }
    }
}

#[zero_copy]
pub struct EventHeapHeader {
    free_head: u16,
    used_head: u16,
    count: u16,
    _padd: u16,
    pub seq_num: u64,
}
const_assert_eq!(std::mem::size_of::<EventHeapHeader>(), 16);
const_assert_eq!(std::mem::size_of::<EventHeapHeader>() % 8, 0);

impl EventHeapHeader {
    pub fn count(&self) -> usize {
        self.count as usize
    }

    pub fn free_head(&self) -> usize {
        self.free_head as usize
    }

    pub fn used_head(&self) -> usize {
        self.used_head as usize
    }

    fn incr_count(&mut self) {
        self.count += 1;
    }

    fn decr_count(&mut self) {
        self.count -= 1;
    }

    fn incr_event_id(&mut self) {
        self.seq_num += 1;
    }
}

#[zero_copy]
#[derive(Debug)]
pub struct EventNode {
    next: u16,
    prev: u16,
    _pad: [u8; 4],
    pub event: AnyEvent,
}
const_assert_eq!(std::mem::size_of::<EventNode>(), 8 + EVENT_SIZE);
const_assert_eq!(std::mem::size_of::<EventNode>() % 8, 0);

impl EventNode {
    pub fn is_free(&self) -> bool {
        self.prev == NO_NODE
    }
}

const EVENT_SIZE: usize = 144;
#[zero_copy]
#[derive(Debug)]
pub struct AnyEvent {
    pub event_type: u8,
    pub padding: [u8; 143],
}

const_assert_eq!(size_of::<AnyEvent>(), EVENT_SIZE);

#[derive(Copy, Clone, IntoPrimitive, TryFromPrimitive, Eq, PartialEq)]
#[repr(u8)]
pub enum EventType {
    Fill,
    Out,
}

#[derive(
    Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, AnchorSerialize, AnchorDeserialize,
)]
#[repr(C)]
pub struct FillEvent {
    pub event_type: u8,
    pub taker_side: u8, // Side, from the taker's POV
    pub maker_out: u8,  // 1 if maker order quantity == 0
    pub maker_slot: u8,
    pub padding: [u8; 4],
    pub timestamp: u64,
    pub market_seq_num: u64,

    pub maker: Pubkey,

    // Timestamp of when the maker order was placed; copied over from the LeafNode
    pub maker_timestamp: u64,

    pub taker: Pubkey,
    pub taker_client_order_id: u64,

    pub price: i64,
    pub peg_limit: i64,
    pub quantity: i64, // number of base lots
    pub maker_client_order_id: u64,
    pub reserved: [u8; 8],
}
const_assert_eq!(size_of::<FillEvent>() % 8, 0);
const_assert_eq!(size_of::<FillEvent>(), EVENT_SIZE);

impl FillEvent {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        taker_side: Side,
        maker_out: bool,
        maker_slot: u8,
        timestamp: u64,
        market_seq_num: u64,
        maker: Pubkey,
        maker_client_order_id: u64,
        maker_timestamp: u64,
        taker: Pubkey,
        taker_client_order_id: u64,
        price: i64,
        peg_limit: i64,
        quantity: i64,
    ) -> FillEvent {
        Self {
            event_type: EventType::Fill as u8,
            taker_side: taker_side.into(),
            maker_out: maker_out.into(),
            maker_slot,
            timestamp,
            market_seq_num,
            maker,
            maker_client_order_id,
            maker_timestamp,
            taker,
            taker_client_order_id,
            price,
            peg_limit,
            quantity,
            padding: Default::default(),
            reserved: [0; 8],
        }
    }

    pub fn taker_side(&self) -> Side {
        self.taker_side.try_into().unwrap()
    }
    pub fn maker_out(&self) -> bool {
        self.maker_out == 1
    }
}

#[derive(
    Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, AnchorSerialize, AnchorDeserialize,
)]
#[repr(C)]
pub struct OutEvent {
    pub event_type: u8,
    pub side: u8, // Side
    pub owner_slot: u8,
    padding0: [u8; 5],
    pub timestamp: u64,
    pub seq_num: u64,
    pub owner: Pubkey,
    pub quantity: i64,
    padding1: [u8; 80],
}
const_assert_eq!(size_of::<OutEvent>() % 8, 0);
const_assert_eq!(size_of::<OutEvent>(), EVENT_SIZE);

impl OutEvent {
    pub fn new(
        side: Side,
        owner_slot: u8,
        timestamp: u64,
        seq_num: u64,
        owner: Pubkey,
        quantity: i64,
    ) -> Self {
        Self {
            event_type: EventType::Out.into(),
            side: side.into(),
            owner_slot,
            padding0: [0; 5],
            timestamp,
            seq_num,
            owner,
            quantity,
            padding1: [0; EVENT_SIZE - 64],
        }
    }

    pub fn side(&self) -> Side {
        self.side.try_into().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck::Zeroable;

    const LAST_SLOT: u16 = MAX_NUM_EVENTS - 1;

    fn count_free_nodes(event_heap: &EventHeap) -> usize {
        event_heap.nodes.iter().filter(|n| n.is_free()).count()
    }

    fn dummy_event_with_number(number: u8) -> AnyEvent {
        let mut dummy_event = AnyEvent::zeroed();
        dummy_event.event_type = number;
        dummy_event
    }

    #[test]
    fn init() {
        let mut eq = EventHeap::zeroed();
        eq.init();

        assert_eq!(eq.header.count(), 0);
        assert_eq!(eq.header.free_head(), 0);
        assert_eq!(eq.header.used_head(), NO_NODE as usize);
        assert_eq!(count_free_nodes(&eq), MAX_NUM_EVENTS as usize);
    }

    #[test]
    #[should_panic]
    fn cannot_insert_if_full() {
        let mut eq = EventHeap::zeroed();
        eq.init();
        for _ in 0..MAX_NUM_EVENTS + 1 {
            eq.push_back(AnyEvent::zeroed());
        }
    }

    #[test]
    #[should_panic]
    fn cannot_delete_if_empty() {
        let mut eq = EventHeap::zeroed();
        eq.init();
        eq.pop_front().unwrap();
    }

    #[test]
    fn insert_until_full() {
        let mut eq = EventHeap::zeroed();
        eq.init();

        // insert one event in the first slot; the single used node should point to himself
        eq.push_back(AnyEvent::zeroed());
        assert_eq!(eq.header.used_head, 0);
        assert_eq!(eq.header.free_head, 1);
        assert_eq!(eq.nodes[0].prev, 0);
        assert_eq!(eq.nodes[0].next, 0);
        assert_eq!(eq.nodes[1].next, 2);

        for i in 1..MAX_NUM_EVENTS - 2 {
            eq.push_back(AnyEvent::zeroed());
            assert_eq!(eq.header.used_head, 0);
            assert_eq!(eq.header.free_head, i + 1);
            assert_eq!(eq.nodes[0].prev, i);
            assert_eq!(eq.nodes[0].next, 1);
            assert_eq!(eq.nodes[i as usize + 1].next, i + 2);
        }

        // insert another one, afterwards only one free node pointing to null should be left
        eq.push_back(AnyEvent::zeroed());
        assert_eq!(eq.header.used_head, 0);
        assert_eq!(eq.header.free_head, LAST_SLOT);
        assert_eq!(eq.nodes[0].prev, LAST_SLOT - 1);
        assert_eq!(eq.nodes[0].next, 1);
        assert_eq!(eq.nodes[LAST_SLOT as usize].next, NO_NODE);

        // insert last available event
        eq.push_back(AnyEvent::zeroed());
        assert_eq!(eq.header.used_head, 0);
        assert_eq!(eq.header.free_head, NO_NODE);
        assert_eq!(eq.nodes[0].prev, LAST_SLOT);
        assert_eq!(eq.nodes[0].next, 1);
    }

    #[test]
    fn delete_full() {
        let mut eq = EventHeap::zeroed();
        eq.init();
        for _ in 0..MAX_NUM_EVENTS {
            eq.push_back(AnyEvent::zeroed());
        }

        eq.pop_front().unwrap();
        assert_eq!(eq.header.free_head, 0);
        assert_eq!(eq.header.used_head, 1);
        assert_eq!(eq.nodes[0].next, NO_NODE);
        assert_eq!(eq.nodes[1].prev, LAST_SLOT);
        assert_eq!(eq.nodes[1].next, 2);

        for i in 1..MAX_NUM_EVENTS - 2 {
            eq.pop_front().unwrap();
            assert_eq!(eq.header.free_head, i);
            assert_eq!(eq.header.used_head, i + 1);
            assert_eq!(eq.nodes[i as usize].next, i - 1);
            assert_eq!(eq.nodes[i as usize + 1].prev, LAST_SLOT);
            assert_eq!(eq.nodes[i as usize + 1].next, i + 2);
        }

        eq.pop_front().unwrap();
        assert_eq!(eq.header.free_head, LAST_SLOT - 1);
        assert_eq!(eq.header.used_head, LAST_SLOT);
        assert_eq!(eq.nodes[LAST_SLOT as usize - 1].next, LAST_SLOT - 2);
        assert_eq!(eq.nodes[LAST_SLOT as usize].prev, LAST_SLOT);
        assert_eq!(eq.nodes[LAST_SLOT as usize].next, LAST_SLOT);

        eq.pop_front().unwrap();
        assert_eq!(eq.header.used_head, NO_NODE);
        assert_eq!(eq.header.free_head, LAST_SLOT);
        assert_eq!(eq.nodes[LAST_SLOT as usize].next, LAST_SLOT - 1);

        assert_eq!(eq.header.count(), 0);
        assert_eq!(count_free_nodes(&eq), MAX_NUM_EVENTS as usize);
    }

    #[test]
    fn delete_at_given_position() {
        let mut eq = EventHeap::zeroed();
        eq.init();
        for _ in 0..5 {
            eq.push_back(AnyEvent::zeroed());
        }
        eq.delete_slot(2).unwrap();
        assert_eq!(eq.header.free_head(), 2);
        assert_eq!(eq.header.used_head(), 0);
    }

    #[test]
    #[should_panic]
    fn cannot_delete_twice_same() {
        let mut eq = EventHeap::zeroed();
        eq.init();
        for _ in 0..5 {
            eq.push_back(AnyEvent::zeroed());
        }
        eq.delete_slot(2).unwrap();
        eq.delete_slot(2).unwrap();
    }

    #[test]
    fn read_front() {
        let mut eq = EventHeap::zeroed();
        eq.init();
        eq.push_back(dummy_event_with_number(1));
        eq.push_back(AnyEvent::zeroed());
        assert_eq!(eq.front().unwrap().event_type, 1);
    }

    #[test]
    fn read_at_slot() {
        let mut eq = EventHeap::zeroed();
        eq.init();
        eq.push_back(AnyEvent::zeroed());
        eq.push_back(AnyEvent::zeroed());
        eq.push_back(dummy_event_with_number(1));
        assert_eq!(eq.at_slot(2).unwrap().event_type, 1);
    }

    #[test]
    fn fifo_event_processing() {
        // [ | | | | ] init
        // [1| | | | ] push_back
        // [1|2| | | ] push_back
        // [ |2| | | ] pop_front
        // [3|2| | | ] push_back
        // [3| | | | ] pop_front

        let mut eq = EventHeap::zeroed();
        eq.init();
        assert!(eq.nodes[0].is_free());
        assert!(eq.nodes[1].is_free());
        assert!(eq.nodes[2].is_free());

        eq.push_back(dummy_event_with_number(1));
        assert_eq!(eq.nodes[0].event.event_type, 1);
        assert!(eq.nodes[1].is_free());
        assert!(eq.nodes[2].is_free());

        eq.push_back(dummy_event_with_number(2));
        assert_eq!(eq.nodes[0].event.event_type, 1);
        assert_eq!(eq.nodes[1].event.event_type, 2);
        assert!(eq.nodes[2].is_free());

        eq.pop_front().unwrap();
        assert!(eq.nodes[0].is_free());
        assert_eq!(eq.nodes[1].event.event_type, 2);
        assert!(eq.nodes[2].is_free());

        eq.push_back(dummy_event_with_number(3));
        assert_eq!(eq.nodes[0].event.event_type, 3);
        assert_eq!(eq.nodes[1].event.event_type, 2);
        assert!(eq.nodes[2].is_free());

        eq.pop_front().unwrap();
        assert_eq!(eq.nodes[0].event.event_type, 3);
        assert!(eq.nodes[1].is_free());
        assert!(eq.nodes[2].is_free());
    }

    #[test]
    fn lifo_free_available_slots() {
        // [0|1|2|3|4] init
        // [ |0|1|2|3] push_back
        // [ | |0|1|2] push_back
        // [0| |1|2|3] pop_front
        // [1|0|2|3|4] pop_front
        // [0| |1|2|3] push_back
        // [ | |0|1|2] push_back

        let mut eq = EventHeap::zeroed();
        eq.init();
        assert_eq!(eq.header.free_head(), 0);
        assert_eq!(eq.nodes[0].next, 1);

        eq.push_back(AnyEvent::zeroed());
        assert_eq!(eq.header.free_head(), 1);
        assert_eq!(eq.nodes[1].next, 2);

        eq.push_back(AnyEvent::zeroed());
        assert_eq!(eq.header.free_head(), 2);
        assert_eq!(eq.nodes[2].next, 3);

        eq.pop_front().unwrap();
        assert_eq!(eq.header.free_head(), 0);
        assert_eq!(eq.nodes[0].next, 2);

        eq.pop_front().unwrap();
        assert_eq!(eq.header.free_head(), 1);
        assert_eq!(eq.nodes[1].next, 0);

        eq.push_back(AnyEvent::zeroed());
        assert_eq!(eq.header.free_head(), 0);
        assert_eq!(eq.nodes[0].next, 2);

        eq.push_back(AnyEvent::zeroed());
        assert_eq!(eq.header.free_head(), 2);
        assert_eq!(eq.nodes[2].next, 3);
    }
}
