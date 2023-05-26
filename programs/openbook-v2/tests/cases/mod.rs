pub use anchor_lang::prelude::Pubkey;
pub use anchor_spl::token::TokenAccount;
pub use fixed::types::I80F48;
pub use solana_program_test::*;
pub use solana_sdk::transport::TransportError;
use std::sync::Arc;

pub use openbook_v2::{error::OpenBookError, state::*};
pub use program_test::*;
pub use setup::*;

pub use super::program_test;

pub use utils::assert_equal_fixed_f64 as assert_equal;

mod test;
mod test_fees;
mod test_oracle_peg;
mod test_order_types;
mod test_permissioned;
mod test_place_order_remaining;
mod test_self_trade;
mod test_take_order;

pub struct TestInitialize<'a> {
    pub context: &'a TestContext,
    pub solana: &'a Arc<SolanaCookie>,
    pub collect_fee_admin: TestKeypair,
    pub open_orders_admin: TestKeypair,
    pub close_market_admin: TestKeypair,
    pub owner: TestKeypair,
    pub payer: TestKeypair,
    pub mints: Vec<MintCookie>,
    pub owner_token_0: Pubkey,
    pub owner_token_1: Pubkey,
    pub market: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub tokens: Vec<Token>,
    pub account_0: Pubkey,
    pub account_1: Pubkey,
}

pub async fn initialize_test_market(
    fee_penalty: u64,
    quote_lot_size: i64,
    base_lot_size: i64,
    maker_fee: f32,
    taker_fee: f32,
    open_orders_admin_bool: bool,
    close_market_admin_bool: bool,
) -> Result<TestInitialize<'static>, TransportError> {
    let context = TestContext::new().await;
    let solana = &context.solana.clone();

    let collect_fee_admin_acc = TestKeypair::new();
    let open_orders_admin_acc = TestKeypair::new();
    let open_orders_admin = if open_orders_admin_bool {
        Some(open_orders_admin_acc.pubkey())
    } else {
        None
    };
    let close_market_admin_acc = TestKeypair::new();
    let close_market_admin = if close_market_admin_bool {
        Some(close_market_admin_acc.pubkey())
    } else {
        None
    };

    let owner = context.users[0].key;
    let payer = context.users[1].key;
    let mints = &context.mints[0..=2];

    let owner_token_0 = context.users[0].token_accounts[0];
    let owner_token_1 = context.users[0].token_accounts[1];

    let tokens = Token::create(mints.to_vec(), solana, collect_fee_admin_acc, payer).await;

    // Create a market

    let market = get_market_address(1);
    let base_vault = solana
        .create_associated_token_account(&market, mints[0].pubkey)
        .await;
    let quote_vault = solana
        .create_associated_token_account(&market, mints[1].pubkey)
        .await;

    let openbook_v2::accounts::CreateMarket {
        market,
        base_vault,
        quote_vault,
        ..
    } = send_tx(
        solana,
        CreateMarketInstruction {
            collect_fee_admin: collect_fee_admin_acc.pubkey(),
            open_orders_admin,
            close_market_admin,
            payer,
            market_index: 1,
            quote_lot_size,
            base_lot_size,
            maker_fee,
            taker_fee,
            base_mint: mints[0].pubkey,
            quote_mint: mints[1].pubkey,
            base_vault,
            quote_vault,
            fee_penalty,
            ..CreateMarketInstruction::with_new_book_and_queue(solana, &tokens[1]).await
        },
    )
    .await
    .unwrap();

    let account_0 = create_open_orders_account(solana, owner, market, 0, &context.users[1]).await;
    let account_1 = create_open_orders_account(solana, owner, market, 1, &context.users[1]).await;

    Ok(TestInitialize {
        context: &context,
        solana,
        collect_fee_admin: collect_fee_admin_acc,
        open_orders_admin: open_orders_admin_acc,
        close_market_admin: close_market_admin_acc,
        owner,
        payer,
        mints: mints.to_vec(),
        owner_token_0,
        owner_token_1,
        market,
        base_vault,
        quote_vault,
        tokens,
        account_0,
        account_1,
    })
}
