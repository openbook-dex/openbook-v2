//! Instructions for the janitor that winds down the sunset program.

use anchor_lang::prelude::System;
use anchor_lang::Id;
use anchor_spl::associated_token::{get_associated_token_address, AssociatedToken};
use anchor_spl::token::Token;
use itertools::Itertools;

use openbook_v2::state::{
    AnyEvent, EventHeap, EventType, FillEvent, Market, OpenOrdersAccount, OutEvent, Side,
};
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;

fn instruction(
    accounts: impl anchor_lang::ToAccountMetas,
    data: impl anchor_lang::InstructionData,
) -> Instruction {
    Instruction {
        program_id: openbook_v2::id(),
        accounts: accounts.to_account_metas(None),
        data: data.data(),
    }
}

pub fn open_orders_indexer_address(owner: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[b"OpenOrdersIndexer".as_ref(), owner.as_ref()],
        &openbook_v2::id(),
    )
    .0
}

/// Open orders accounts referenced by the events on the heap, in heap order
pub fn event_heap_open_orders_accounts(event_heap: &EventHeap) -> Vec<Pubkey> {
    event_heap
        .iter()
        .map(|(event, _)| event_open_orders_account(event))
        .unique()
        .collect()
}

/// The open orders account `consume_events` needs for `event`
pub fn event_open_orders_account(event: &AnyEvent) -> Pubkey {
    match EventType::try_from(event.event_type).expect("valid event type") {
        EventType::Fill => bytemuck::cast_ref::<AnyEvent, FillEvent>(event).maker,
        EventType::Out => bytemuck::cast_ref::<AnyEvent, OutEvent>(event).owner,
    }
}

/// Consume up to `limit` events with the janitor as `consume_events_admin`
pub fn consume_events_ix(
    janitor: Pubkey,
    market_address: Pubkey,
    market: &Market,
    open_orders_accounts: &[Pubkey],
    limit: usize,
) -> Instruction {
    let mut ix = instruction(
        openbook_v2::accounts::ConsumeEvents {
            consume_events_admin: Some(janitor),
            market: market_address,
            event_heap: market.event_heap,
        },
        openbook_v2::instruction::ConsumeEvents { limit },
    );
    ix.accounts.extend(
        open_orders_accounts
            .iter()
            .map(|key| solana_sdk::instruction::AccountMeta::new(*key, false)),
    );
    ix
}

pub fn cancel_all_orders_ix(
    janitor: Pubkey,
    market_address: Pubkey,
    market: &Market,
    open_orders_account: Pubkey,
    side: Option<Side>,
    limit: u8,
) -> Instruction {
    instruction(
        openbook_v2::accounts::CancelOrder {
            signer: janitor,
            open_orders_account,
            market: market_address,
            bids: market.bids,
            asks: market.asks,
        },
        openbook_v2::instruction::CancelAllOrders {
            side_option: side,
            limit,
        },
    )
}

/// Settle into the owner's associated token accounts and close. `market` is `None` for
/// accounts whose market was already closed.
pub fn close_open_orders_account_ix(
    janitor: Pubkey,
    open_orders_account_address: Pubkey,
    open_orders_account: &OpenOrdersAccount,
    market: Option<&Market>,
) -> Instruction {
    let owner = open_orders_account.owner;
    let market_address = open_orders_account.market;
    // the settlement accounts are not read for orphans
    let (market_authority, base_vault, quote_vault, base_mint, quote_mint) = match market {
        Some(market) => (
            market.market_authority,
            market.market_base_vault,
            market.market_quote_vault,
            market.base_mint,
            market.quote_mint,
        ),
        None => (
            market_address,
            market_address,
            market_address,
            market_address,
            market_address,
        ),
    };

    instruction(
        openbook_v2::accounts::JanitorCloseOpenOrdersAccount {
            janitor,
            owner,
            open_orders_indexer: open_orders_indexer_address(&owner),
            open_orders_account: open_orders_account_address,
            market: market_address,
            market_authority,
            market_base_vault: base_vault,
            market_quote_vault: quote_vault,
            owner_base_ata: get_associated_token_address(&owner, &base_mint),
            owner_quote_ata: get_associated_token_address(&owner, &quote_mint),
            base_mint,
            quote_mint,
            token_program: Token::id(),
            associated_token_program: AssociatedToken::id(),
            system_program: System::id(),
        },
        openbook_v2::instruction::JanitorCloseOpenOrdersAccount {},
    )
}

/// Close the empty indexer of `owner` into it
pub fn close_open_orders_indexer_ix(janitor: Pubkey, owner: Pubkey) -> Instruction {
    instruction(
        openbook_v2::accounts::JanitorCloseOpenOrdersIndexer {
            janitor,
            owner,
            open_orders_indexer: open_orders_indexer_address(&owner),
        },
        openbook_v2::instruction::JanitorCloseOpenOrdersIndexer {},
    )
}

/// Sweep fees into the `collect_fee_admin`'s associated quote token account
pub fn sweep_fees_ix(janitor: Pubkey, market_address: Pubkey, market: &Market) -> Instruction {
    instruction(
        openbook_v2::accounts::SweepFees {
            collect_fee_admin: janitor,
            market: market_address,
            market_authority: market.market_authority,
            market_quote_vault: market.market_quote_vault,
            token_receiver_account: get_associated_token_address(
                &market.collect_fee_admin,
                &market.quote_mint,
            ),
            token_program: Token::id(),
        },
        openbook_v2::instruction::SweepFees {},
    )
}

/// Close the market into the escrow. The escrow's associated token accounts receive tokens
/// left in the vaults, pass `true` for a vault that is not empty.
/// Close an empty market into the escrow, burning what is left in its vaults
pub fn close_market_ix(janitor: Pubkey, market_address: Pubkey, market: &Market) -> Instruction {
    instruction(
        openbook_v2::accounts::JanitorCloseMarket {
            janitor,
            market: market_address,
            bids: market.bids,
            asks: market.asks,
            event_heap: market.event_heap,
            market_base_vault: market.market_base_vault,
            market_quote_vault: market.market_quote_vault,
            market_authority: market.market_authority,
            base_mint: market.base_mint,
            quote_mint: market.quote_mint,
            escrow: openbook_v2::state::ESCROW,
            token_program: Token::id(),
        },
        openbook_v2::instruction::JanitorCloseMarket {},
    )
}

/// Create the associated token account of `owner` if it does not exist, paid by the janitor
pub fn create_ata_idempotent_ix(janitor: Pubkey, owner: Pubkey, mint: Pubkey) -> Instruction {
    spl_associated_token_account::instruction::create_associated_token_account_idempotent(
        &janitor,
        &owner,
        &mint,
        &Token::id(),
    )
}
