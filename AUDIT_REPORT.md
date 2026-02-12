# OpenBook V2 Security Audit Report

**Auditor:** Security Engineer  
**Date:** 2026-02-12  
**Commit:** HEAD of `openbook-dex/openbook-v2` main branch  
**Scope:** Full program source (`programs/openbook-v2/src/`)

---

## Finding 1: Fees from `place_take_order` Permanently Locked in Vault

**Severity:** Medium  
**File:** `programs/openbook-v2/src/instructions/place_take_order.rs` (lines 51-67)  
**Related:** `programs/openbook-v2/src/state/orderbook/book.rs` (line ~189)

### Description

When `place_take_order` is used (no OpenOrdersAccount), taker fees are collected and `market.fees_accrued` is incremented inside `book.rs::new_order()` (line ~189: `market.fees_accrued += referrer_amount as u128`). However, `place_take_order` has no settlement step and no path to convert `fees_accrued` into `fees_available`.

The only place `fees_available` is incremented is in `settle_funds.rs` line 27:
```rust
market.fees_available += referrer_rebate;
```

This only runs when an OOA holder settles. Since `place_take_order` creates no OOA and has no settle step, the portion of fees that would normally go to referrers (`referrer_amount`) is deposited into the quote vault but never becomes withdrawable via `sweep_fees`.

### Attack Scenario

Over time, significant quote tokens accumulate in the vault that can never be swept by `collect_fee_admin`. This is a permanent loss of protocol revenue proportional to `place_take_order` volume.

### Fix

In `place_take_order.rs`, after computing `referrer_amount`, add it to `fees_available`:

```rust
market.fees_available += referrer_amount;
```

---

## Finding 2: Missing Owner Check on `Deposit` — Anyone Can Deposit into Any OOA

**Severity:** Medium  
**File:** `programs/openbook-v2/src/accounts_ix/deposit.rs` (full file)

### Description

The `Deposit` accounts struct has no `is_owner_or_delegate` check on the `open_orders_account`. The `owner` signer authorizes the token transfer but is NOT verified to be the OOA owner. Any user can deposit their own tokens into someone else's OpenOrdersAccount.

### Attack Scenario

1. Attacker deposits dust amounts (e.g., 1 lamport of base and quote) into a victim's OOA
2. This prevents `Position::is_empty()` from returning true (since `base_free_native` or `quote_free_native` > 0)
3. The victim cannot close their OpenOrdersAccount until they settle and withdraw these extra funds
4. This is a griefing attack that prevents account closure, locking rent SOL

More critically, if an attacker deposits a large amount of base tokens into a victim's OOA, the victim's next ask order could be placed using those tokens as collateral (since `place_order` uses `position.base_free_native` to reduce deposit requirements). The victim then has a larger position than intended.

### Fix

Add owner validation to the Deposit struct:

```rust
#[account(
    mut,
    has_one = market,
    constraint = open_orders_account.load()?.is_owner_or_delegate(owner.key()) @ OpenBookError::NoOwnerOrDelegate
)]
pub open_orders_account: AccountLoader<'info, OpenOrdersAccount>,
```

---

## Finding 3: Self-Trade `DecrementTake` Does Not Fully Skip Matching

**Severity:** Medium  
**File:** `programs/openbook-v2/src/state/orderbook/book.rs` (lines ~141-155)

### Description

When `SelfTradeBehavior::DecrementTake` is selected and a self-trade occurs, the code records `decremented_quote_lots` for fee exemption but **still processes the match**:
- `remaining_base_lots` and `remaining_quote_lots` are decremented (lines ~165-166)
- A `FillEvent` is created and pushed to the event heap
- The maker order is partially/fully consumed

The fill event in `execute_maker` then checks `is_self_trade` to skip fees, but the order is still matched. This means `DecrementTake` self-trade behavior allows a user to consume their own orders and generate fill events, which wastes event heap space (each event costs PENALTY_EVENT_HEAP = 500 lamports from OTHER takers who fill the heap).

Additionally, the matched quantities are moved between the maker/taker sides of the same account, but the `fill event` is processed via `process_fill_event` which looks for the maker's OOA in `remaining_accounts`. If the maker OOA isn't passed, the event goes to the heap, causing unnecessary heap congestion.

### Attack Scenario

An attacker with a single OOA places asks, then immediately takes them with bids using `DecrementTake`. Each self-trade pushes a fill event to the heap, polluting it. Other users placing orders that add events to the heap pay the 500 lamport penalty. The attacker cycles through this to fill the event heap (600 slots), blocking legitimate trades until events are consumed.

### Fix

For `DecrementTake`, skip `remaining_base_lots/remaining_quote_lots` decrements and fill event generation on self-trades (similar to `CancelProvide`'s `continue`):

```rust
SelfTradeBehavior::DecrementTake => {
    // Only track for fee calculation, don't actually match
    continue;
}
```

---

## Finding 4: Event Heap Full Causes Silent Order Degradation

**Severity:** Medium  
**File:** `programs/openbook-v2/src/state/orderbook/book.rs` (line ~206, `process_fill_event`)

### Description

In `process_fill_event`, if the event heap is full (`push_back` asserts `!self.is_full()`), the transaction will panic/abort. This means if the event heap has 600 events and a new taker order matches, the entire transaction fails.

While `FILL_EVENT_REMAINING_LIMIT` (15) and passing maker OOAs in remaining accounts helps, there's no graceful degradation. An attacker can fill the event heap with 600 events (via dust trades or self-trades), then **all new taker orders fail** until someone calls `consume_events`.

The heap is limited to 600 events. At `MAX_EVENTS_CONSUME = 8` per crank, clearing a full heap requires 75 crank transactions.

### Attack Scenario

1. Attacker creates many OOAs, places orders from each, matches them to generate fill events
2. Once heap is full (600 events), no new matching orders can be placed
3. Market is effectively frozen until crankers process events
4. During this freeze, the attacker can front-run the unfreeze by having consume_events + new_order in the same transaction

### Fix

Add a check before matching to stop taking if the event heap is near-full, rather than panicking:

```rust
if event_heap.len() >= event_heap.nodes.len() - 1 {
    post_target = None;
    break;
}
```

---

## Finding 5: `cancel_order` in `book.rs` Does Not Verify Owner Slot Consistency

**Severity:** Medium  
**File:** `programs/openbook-v2/src/state/orderbook/book.rs` (line ~294, `cancel_order`)  
**Related:** `programs/openbook-v2/src/state/open_orders_account.rs` (line ~279, `cancel_order`)

### Description

When an out event or cancel is processed, `open_orders_account.cancel_order(slot, quantity, market)` is called where `slot` comes from the event's `owner_slot`. The `cancel_order` function trusts this slot index without verifying that the order at that slot matches the expected order_id.

In `cancel_order` (OOA), the slot is used directly:
```rust
let oo = self.open_order_by_raw_index(slot);
let price = oo.locked_price;
let order_side = oo.side_and_tree().side();
```

If an event is replayed or if slots are reused after an order is filled and a new order placed in the same slot, the wrong order's accounting could be affected.

Specifically, in `consume_events.rs`, the event is read from the heap and the slot is trusted:
```rust
let fill: &FillEvent = cast_ref(event);
load_open_orders_account!(maker, fill.maker, remaining_accs);
maker.execute_maker(&mut market, fill);
```

The `execute_maker` calls `remove_order(fill.maker_slot, ...)` which frees the slot. If the fill event references a slot that was already freed and reused by a new order, the new order gets incorrectly cancelled.

### Attack Scenario

1. User A places order in slot 0, gets filled (fill event in heap)
2. Before the fill event is cranked, User A places new order which lands in slot 0
3. Crank processes the old fill event → calls `execute_maker` → removes order at slot 0
4. User A's NEW order at slot 0 is incorrectly freed, and its locked funds are released with wrong accounting

### Fix

In `execute_maker`, verify the order_id at the slot matches the fill event before processing:

```rust
// In execute_maker, after getting the slot:
if fill.maker_out() {
    let oo = self.open_order_by_raw_index(fill.maker_slot as usize);
    // Verify the order at this slot is the one being filled
    if oo.is_free() {
        // Already processed, skip
        return;
    }
    self.remove_order(fill.maker_slot as usize, fill.quantity, locked_price);
}
```

---

## Summary

| # | Finding | Severity | Impact |
|---|---------|----------|--------|
| 1 | Fees from `place_take_order` permanently locked | Medium | Protocol revenue loss |
| 2 | Missing owner check on Deposit | Medium | Griefing, prevents OOA closure |
| 3 | `DecrementTake` self-trade pollutes event heap | Medium | Market DoS via heap flooding |
| 4 | Full event heap panics all taker orders | Medium | Market freeze / DoS |
| 5 | `cancel_order`/`execute_maker` slot reuse race | Medium | Incorrect order accounting |

---

## Notes on Areas Reviewed Without Findings

- **Market creation**: Proper validation of lot sizes, fees, oracles, mint uniqueness. `has_one` constraints link vaults to market authority PDA. No fake market attack vector found.
- **Settlement logic**: `SettleFunds` properly validates owner/delegate, vault matches, and token mint matches. Funds always flow to authorized destinations.
- **Account validation**: All critical instructions (place_order, cancel_order, settle_funds) have proper signer/owner checks via Anchor constraints. Exception: Deposit (Finding 2).
- **Fee overflow**: Fee calculations use `i128` intermediate values with `FEES_SCALE_FACTOR = 1_000_000`, safely within bounds. `ceil_fee_division` correctly rounds up.
- **Oracle handling**: Stale/invalid oracles return `None`, which blocks oracle-pegged orders. Confidence bands are validated.
- **Front-running**: Inherent to Solana's leader-based transaction ordering; no protocol-level mitigation possible. PostOnlySlide correctly adjusts prices.
