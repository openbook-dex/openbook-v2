use super::*;
use std::sync::Arc;

#[tokio::test]
async fn test_fill_or_kill_order() -> Result<(), TransportError> {
    let TestInitialize {
        context,
        collect_fee_admin,
        owner,
        owner_token_0,
        owner_token_1,
        market,

        market_base_vault,
        market_quote_vault,
        price_lots,
        tokens,
        account_1,
        account_2,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize {
        maker_fee: 0,
        taker_fee: 0,
        ..TestNewMarketInitialize::default()
    })
    .await?;
    let default = PlaceOrderInstruction {
        open_orders_account: Default::default(),
        open_orders_admin: None,
        market,
        signer: owner,
        market_vault: Default::default(),
        user_token_account: Default::default(),
        side: Side::Bid,
        price_lots,
        max_base_lots: 0,
        max_quote_lots_including_fees: 0,
        client_order_id: 0,
        expiry_timestamp: 0,
        order_type: PlaceOrderType::Limit,
        self_trade_behavior: Default::default(),
        remainings: vec![],
    };
    let solana = &context.solana.clone();

    // Set the initial oracle price
    set_stub_oracle_price(solana, &tokens[1], collect_fee_admin, 1000.0).await;

    let initial_balance_base = solana.token_account_balance(owner_token_0).await;
    let initial_balance_quote = solana.token_account_balance(owner_token_1).await;

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            user_token_account: owner_token_1,
            market_vault: market_quote_vault,
            side: Side::Bid,
            max_base_lots: 5,
            max_quote_lots_including_fees: 50_000,
            order_type: PlaceOrderType::Limit,
            ..default.clone()
        },
    )
    .await
    .unwrap();

    assert_open_orders(account_1, solana, 5, 0).await;
    assert_open_orders(account_2, solana, 0, 0).await;

    // small order -> no problem
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_2,
            user_token_account: owner_token_0,
            market_vault: market_base_vault,
            side: Side::Ask,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10_000,
            order_type: PlaceOrderType::FillOrKill,
            ..default.clone()
        },
    )
    .await
    .unwrap();

    consume_and_settle(
        owner,
        owner_token_0,
        owner_token_1,
        market,
        market_base_vault,
        market_quote_vault,
        account_1,
        account_2,
        solana,
    )
    .await;

    assert_open_orders(account_1, solana, 4, 0).await;
    assert_open_orders(account_2, solana, 0, 0).await;

    assert_balance(owner_token_0, solana, initial_balance_base).await;
    assert_balance(owner_token_1, solana, initial_balance_quote - 400_000).await;

    // big order -> should fail
    let result = send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_2,
            user_token_account: owner_token_0,
            market_vault: market_base_vault,
            side: Side::Ask,
            max_base_lots: 6,
            max_quote_lots_including_fees: 60_000,
            order_type: PlaceOrderType::FillOrKill,
            ..default.clone()
        },
    )
    .await;

    assert_openbook_error(
        &result,
        OpenBookError::WouldExecutePartially.error_code(),
        "Should kill order".into(),
    );
    Ok(())
}

async fn assert_open_orders(
    account: Pubkey,
    solana: &Arc<SolanaCookie>,
    bids_base_lots: i64,
    asks_base_lots: i64,
) {
    let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account).await;
    assert_eq!(
        open_orders_account_1.position.bids_base_lots,
        bids_base_lots
    );
    assert_eq!(
        open_orders_account_1.position.asks_base_lots,
        asks_base_lots
    );
}

async fn assert_balance(token: Pubkey, solana: &Arc<SolanaCookie>, expected_balance: u64) {
    let balance_base = solana.token_account_balance(token).await;
    assert_eq!(balance_base, expected_balance);
}

#[allow(clippy::too_many_arguments)]
async fn consume_and_settle(
    owner: TestKeypair,
    owner_token_0: Pubkey,
    owner_token_1: Pubkey,
    market: Pubkey,
    market_base_vault: Pubkey,
    market_quote_vault: Pubkey,
    account_1: Pubkey,
    account_2: Pubkey,
    solana: &Arc<SolanaCookie>,
) {
    send_tx(
        solana,
        ConsumeEventsInstruction {
            consume_events_admin: None,
            market,
            open_orders_accounts: vec![account_1, account_2],
        },
    )
    .await
    .unwrap();

    send_tx(
        solana,
        SettleFundsInstruction {
            owner,
            market,
            open_orders_account: account_1,
            market_base_vault,
            market_quote_vault,
            user_base_account: owner_token_0,
            user_quote_account: owner_token_1,
            referrer_account: None,
        },
    )
    .await
    .unwrap();

    send_tx(
        solana,
        SettleFundsInstruction {
            owner,
            market,
            open_orders_account: account_2,
            market_base_vault,
            market_quote_vault,
            user_base_account: owner_token_0,
            user_quote_account: owner_token_1,
            referrer_account: None,
        },
    )
    .await
    .unwrap();
}
