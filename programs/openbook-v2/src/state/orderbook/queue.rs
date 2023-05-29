use crate::error::OpenBookError;
use anchor_lang::prelude::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use static_assertions::const_assert_eq;
use std::mem::size_of;

use super::Side;

pub const MAX_NUM_EVENTS: usize = 488;

pub const NULL: u16 = u16::MAX;
pub const LAST_SLOT: usize = MAX_NUM_EVENTS - 1;

#[account(zero_copy)]
pub struct EventQueue {
    pub header: EventQueueHeader,
    pub nodes: [EventNode; MAX_NUM_EVENTS],
    pub reserved: [u8; 64],
}
const_assert_eq!(std::mem::size_of::<EventQueue>(), 16 + 488 * 208 + 64);
const_assert_eq!(std::mem::size_of::<EventQueue>(), 101584);
const_assert_eq!(std::mem::size_of::<EventQueue>() % 8, 0);

impl EventQueue {
    pub fn init(&mut self) {
        self.header = EventQueueHeader {
            free_head: 0,
            used_head: NULL,
            count: 0,
            seq_num: 0,
            _padd: Default::default(),
        };

        for i in 0..MAX_NUM_EVENTS {
            self.nodes[i].set_next(i + 1);
            self.nodes[i].set_prev(NULL as usize);
            self.nodes[i].mark_as_free();
        }
        self.nodes[LAST_SLOT].set_next(NULL as usize);
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

    pub fn at(&self, slot: usize) -> Option<&AnyEvent> {
        if !self.nodes[slot].is_free() {
            None
        } else {
            Some(&self.nodes[slot].event)
        }
    }

    pub fn push_back(&mut self, value: AnyEvent) {
        assert!(!self.is_full());

        let slot = self.header.free_head();
        let new_next: usize;
        let new_prev: usize;

        if self.is_empty() {
            new_next = slot;
            new_prev = slot;

            self.header.set_free_head(self.nodes[slot].next() as u16);
            self.header.set_used_head(slot as u16);
        } else {
            new_next = self.header.used_head();
            new_prev = self.nodes[new_next].prev as usize;

            self.nodes[new_prev].set_next(slot);
            self.nodes[new_next].set_prev(slot);
            self.header.set_free_head(self.nodes[slot].next() as u16);
        }

        self.header.incr_count();
        self.header.incr_event_id();
        self.nodes[slot].event = value;
        self.nodes[slot].mark_as_used();
        self.nodes[slot].set_next(new_next);
        self.nodes[slot].set_prev(new_prev);
    }

    pub fn pop_front(&mut self) -> Result<AnyEvent> {
        self.delete_slot(self.header.used_head())
    }

    pub fn delete_slot(&mut self, slot: usize) -> Result<AnyEvent> {
        if self.is_empty() || self.nodes[slot].is_free() {
            return Err(OpenBookError::SomeError.into());
        }

        let prev_slot = self.nodes[slot].prev();
        let next_slot = self.nodes[slot].next();
        let next_free = self.header.free_head();

        self.nodes[prev_slot].set_next(next_slot);
        self.nodes[next_slot].set_prev(prev_slot);

        self.header.set_free_head(slot as u16);

        if self.header.count() == 1 {
            self.header.set_used_head(NULL);
        } else if self.header.used_head() == slot {
            self.header.set_used_head(next_slot as u16);
        };

        self.header.decr_count();
        self.nodes[slot].set_next(next_free);
        self.nodes[slot].mark_as_free();

        Ok(self.nodes[slot].event)
    }

    pub fn iter(&self) -> impl Iterator<Item = &AnyEvent> {
        EventQueueIterator {
            queue: self,
            index: 0,
            slot: self.header.free_head(),
        }
    }
}

struct EventQueueIterator<'a> {
    queue: &'a EventQueue,
    index: usize,
    slot: usize,
}

impl<'a> Iterator for EventQueueIterator<'a> {
    type Item = &'a AnyEvent;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.queue.len() {
            None
        } else {
            let item = &self.queue.nodes[self.slot].event;
            self.slot = self.queue.nodes[self.slot].next();
            self.index += 1;
            Some(item)
        }
    }
}

#[zero_copy]
pub struct EventQueueHeader {
    free_head: u16,
    used_head: u16,
    count: u16,
    _padd: u16,
    pub seq_num: u64,
}
const_assert_eq!(std::mem::size_of::<EventQueueHeader>(), 16);
const_assert_eq!(std::mem::size_of::<EventQueueHeader>() % 8, 0);

impl EventQueueHeader {
    pub fn count(&self) -> usize {
        self.count as usize
    }

    pub fn free_head(&self) -> usize {
        self.free_head as usize
    }

    pub fn used_head(&self) -> usize {
        self.used_head as usize
    }

    fn set_free_head(&mut self, value: u16) {
        self.free_head = value;
    }

    fn set_used_head(&mut self, value: u16) {
        self.used_head = value;
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
    status: u8, // NodeStatus,
    _pad: [u8; 3],
    pub event: AnyEvent,
}
const_assert_eq!(std::mem::size_of::<EventNode>(), 8 + 200);
const_assert_eq!(std::mem::size_of::<EventNode>() % 8, 0);

impl EventNode {
    pub fn status(&self) -> EventNodeStatus {
        EventNodeStatus::try_from(self.status).unwrap()
    }

    pub fn is_free(&self) -> bool {
        self.status == Into::<u8>::into(EventNodeStatus::Free)
    }

    fn mark_as_free(&mut self) {
        self.status = EventNodeStatus::Free.into();
    }

    fn mark_as_used(&mut self) {
        self.status = EventNodeStatus::InUse.into();
    }

    pub fn next(&self) -> usize {
        self.next as usize
    }

    pub fn prev(&self) -> usize {
        self.prev as usize
    }

    fn set_next(&mut self, next: usize) {
        self.next = next as u16;
    }

    fn set_prev(&mut self, prev: usize) {
        self.prev = prev as u16;
    }
}

#[derive(
    Eq,
    PartialEq,
    Copy,
    Clone,
    Debug,
    IntoPrimitive,
    TryFromPrimitive,
    AnchorSerialize,
    AnchorDeserialize,
)]
#[repr(u8)]
pub enum EventNodeStatus {
    Free = 0,
    InUse = 1,
}

const EVENT_SIZE: usize = 200;
#[zero_copy]
#[derive(Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AnyEvent {
    pub event_type: u8,
    pub padding: [u8; 199],
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
    pub seq_num: u64,

    pub maker: Pubkey,
    pub padding2: [u8; 32],

    // Timestamp of when the maker order was placed; copied over from the LeafNode
    pub maker_timestamp: u64,

    pub taker: Pubkey,
    pub padding3: [u8; 16],
    pub taker_client_order_id: u64,
    pub padding4: [u8; 16],

    pub price: i64,
    pub quantity: i64, // number of quote lots
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
        seq_num: u64,
        maker: Pubkey,
        maker_client_order_id: u64,
        maker_timestamp: u64,
        taker: Pubkey,
        taker_client_order_id: u64,
        price: i64,
        quantity: i64,
    ) -> FillEvent {
        Self {
            event_type: EventType::Fill as u8,
            taker_side: taker_side.into(),
            maker_out: maker_out.into(),
            maker_slot,
            timestamp,
            seq_num,
            maker,
            maker_client_order_id,
            maker_timestamp,
            taker,
            taker_client_order_id,
            price,
            quantity,
            padding: Default::default(),
            padding2: Default::default(),
            padding3: Default::default(),
            padding4: Default::default(),
            reserved: [0; 8],
        }
    }

    pub fn base_quote_change(&self, side: Side) -> (i64, i64) {
        match side {
            Side::Bid => (self.quantity, -self.price * self.quantity),
            Side::Ask => (-self.quantity, self.price * self.quantity),
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
    padding1: [u8; 136],
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

    const LAST_SLOT: usize = MAX_NUM_EVENTS - 1;

    fn count_free_nodes(event_queue: &EventQueue) -> usize {
        event_queue.nodes.iter().filter(|n| n.is_free()).count()
    }

    #[test]
    fn init() {
        let mut eq = EventQueue::zeroed();
        eq.init();

        assert_eq!(eq.header.count(), 0);
        assert_eq!(eq.header.free_head(), 0);
        assert_eq!(eq.header.used_head(), NULL as usize);
        assert_eq!(count_free_nodes(&eq), MAX_NUM_EVENTS);
    }

    #[test]
    #[should_panic]
    fn cannot_insert_if_full() {
        let mut eq = EventQueue::zeroed();
        eq.init();
        for _ in 0..MAX_NUM_EVENTS + 1 {
            eq.push_back(AnyEvent::zeroed());
        }
    }

    #[test]
    #[should_panic]
    fn cannot_delete_if_empty() {
        let mut eq = EventQueue::zeroed();
        eq.init();
        eq.pop_front().unwrap();
    }

    #[test]
    fn insert_until_full() {
        let mut eq = EventQueue::zeroed();
        eq.init();

        // insert one event in the first slot; the single used node should point to himself
        eq.push_back(AnyEvent::zeroed());
        assert_eq!(eq.header.used_head(), 0);
        assert_eq!(eq.header.free_head(), 1);
        assert_eq!(eq.nodes[0].prev(), 0);
        assert_eq!(eq.nodes[0].next(), 0);
        assert_eq!(eq.nodes[1].next(), 2);

        for i in 1..MAX_NUM_EVENTS - 2 {
            eq.push_back(AnyEvent::zeroed());
            assert_eq!(eq.header.used_head(), 0);
            assert_eq!(eq.header.free_head(), i + 1);
            assert_eq!(eq.nodes[0].prev(), i);
            assert_eq!(eq.nodes[0].next(), 1);
            assert_eq!(eq.nodes[i + 1].next(), i + 2);
        }

        // insert another one, afterwards only one free node pointing to null should be left
        eq.push_back(AnyEvent::zeroed());
        assert_eq!(eq.header.used_head(), 0);
        assert_eq!(eq.header.free_head(), LAST_SLOT);
        assert_eq!(eq.nodes[0].prev(), LAST_SLOT - 1);
        assert_eq!(eq.nodes[0].next(), 1);
        assert_eq!(eq.nodes[LAST_SLOT].next(), NULL as usize);

        // insert last available event
        eq.push_back(AnyEvent::zeroed());
        assert_eq!(eq.header.used_head(), 0);
        assert_eq!(eq.header.free_head(), NULL as usize);
        assert_eq!(eq.nodes[0].prev(), LAST_SLOT);
        assert_eq!(eq.nodes[0].next(), 1);
    }

    #[test]
    fn delete_full() {
        let mut eq = EventQueue::zeroed();
        eq.init();
        for _ in 0..MAX_NUM_EVENTS {
            eq.push_back(AnyEvent::zeroed());
        }

        eq.pop_front().unwrap();
        assert_eq!(eq.header.free_head(), 0);
        assert_eq!(eq.header.used_head(), 1);
        assert_eq!(eq.nodes[0].next(), NULL as usize);
        assert_eq!(eq.nodes[1].prev(), LAST_SLOT);
        assert_eq!(eq.nodes[1].next(), 2);

        for i in 1..MAX_NUM_EVENTS - 2 {
            eq.pop_front().unwrap();
            assert_eq!(eq.header.free_head(), i);
            assert_eq!(eq.header.used_head(), i + 1);
            assert_eq!(eq.nodes[i].next(), i - 1);
            assert_eq!(eq.nodes[i + 1].prev(), LAST_SLOT);
            assert_eq!(eq.nodes[i + 1].next(), i + 2);
        }

        eq.pop_front().unwrap();
        assert_eq!(eq.header.free_head(), LAST_SLOT - 1);
        assert_eq!(eq.header.used_head(), LAST_SLOT);
        assert_eq!(eq.nodes[LAST_SLOT - 1].next(), LAST_SLOT - 2);
        assert_eq!(eq.nodes[LAST_SLOT].prev(), LAST_SLOT);
        assert_eq!(eq.nodes[LAST_SLOT].next(), LAST_SLOT);

        eq.pop_front().unwrap();
        assert_eq!(eq.header.used_head(), NULL as usize);
        assert_eq!(eq.header.free_head(), LAST_SLOT);
        assert_eq!(eq.nodes[LAST_SLOT].next(), LAST_SLOT - 1);

        assert_eq!(eq.header.count(), 0);
        assert_eq!(count_free_nodes(&eq), MAX_NUM_EVENTS);
    }

    #[test]
    fn delete_at_given_position() {
        let mut eq = EventQueue::zeroed();
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
        let mut eq = EventQueue::zeroed();
        eq.init();
        for _ in 0..5 {
            eq.push_back(AnyEvent::zeroed());
        }
        eq.delete_slot(2).unwrap();
        eq.delete_slot(2).unwrap();
    }

    #[test]
    fn fifo_event_processing() {
        let event_1 = {
            let mut dummy_event = AnyEvent::zeroed();
            dummy_event.event_type = 1;
            dummy_event
        };

        let event_2 = {
            let mut dummy_event = AnyEvent::zeroed();
            dummy_event.event_type = 2;
            dummy_event
        };

        let event_3 = {
            let mut dummy_event = AnyEvent::zeroed();
            dummy_event.event_type = 3;
            dummy_event
        };

        // [ | | | | ] init
        // [1| | | | ] push_back
        // [1|2| | | ] push_back
        // [ |2| | | ] pop_front
        // [3|2| | | ] push_back
        // [3| | | | ] pop_front

        let mut eq = EventQueue::zeroed();
        eq.init();
        assert!(eq.nodes[0].is_free());
        assert!(eq.nodes[1].is_free());
        assert!(eq.nodes[2].is_free());

        eq.push_back(event_1);
        assert_eq!(eq.nodes[0].event.event_type, 1);
        assert!(eq.nodes[1].is_free());
        assert!(eq.nodes[2].is_free());

        eq.push_back(event_2);
        assert_eq!(eq.nodes[0].event.event_type, 1);
        assert_eq!(eq.nodes[1].event.event_type, 2);
        assert!(eq.nodes[2].is_free());

        eq.pop_front().unwrap();
        assert!(eq.nodes[0].is_free());
        assert_eq!(eq.nodes[1].event.event_type, 2);
        assert!(eq.nodes[2].is_free());

        eq.push_back(event_3);
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

        let mut eq = EventQueue::zeroed();
        eq.init();
        assert_eq!(eq.header.free_head(), 0);
        assert_eq!(eq.nodes[0].next(), 1);

        eq.push_back(AnyEvent::zeroed());
        assert_eq!(eq.header.free_head(), 1);
        assert_eq!(eq.nodes[1].next(), 2);

        eq.push_back(AnyEvent::zeroed());
        assert_eq!(eq.header.free_head(), 2);
        assert_eq!(eq.nodes[2].next(), 3);

        eq.pop_front().unwrap();
        assert_eq!(eq.header.free_head(), 0);
        assert_eq!(eq.nodes[0].next(), 2);

        eq.pop_front().unwrap();
        assert_eq!(eq.header.free_head(), 1);
        assert_eq!(eq.nodes[1].next(), 0);

        eq.push_back(AnyEvent::zeroed());
        assert_eq!(eq.header.free_head(), 0);
        assert_eq!(eq.nodes[0].next(), 2);

        eq.push_back(AnyEvent::zeroed());
        assert_eq!(eq.header.free_head(), 2);
        assert_eq!(eq.nodes[2].next(), 3);
    }
}
