use super::*;

#[tokio::test]
async fn test_take_ask_order() -> Result<(), TransportError> {
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
    } = TestContext::new_with_market(TestNewMarketInitialize::default()).await?;
    let solana = &context.solana.clone();

    // Set the initial oracle price
    set_stub_oracle_price(solana, &tokens[1], collect_fee_admin, 1000.0).await;

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_token_1,
            market_vault: market_quote_vault,
            side: Side::Bid,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10000,

            client_order_id: 0,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    let balance_base = solana.token_account_balance(owner_token_0).await;
    let balance_quote = solana.token_account_balance(owner_token_1).await;

    send_tx(
        solana,
        PlaceTakeOrderInstruction {
            market,
            signer: owner,
            user_base_account: owner_token_0,
            user_quote_account: owner_token_1,
            market_base_vault,
            market_quote_vault,
            side: Side::Ask,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10000,
            open_orders_admin: None,
        },
    )
    .await
    .unwrap();

    {
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;
        let open_orders_account_2 = solana.get_account::<OpenOrdersAccount>(account_2).await;

        assert_eq!(open_orders_account_1.position.bids_base_lots, 1);
        assert_eq!(open_orders_account_2.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_2.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.base_free_native, 0);
        assert_eq!(open_orders_account_2.position.base_free_native, 0);
        assert_eq!(open_orders_account_1.position.quote_free_native, 0);
        assert_eq!(open_orders_account_2.position.quote_free_native, 0);
        assert_eq!(
            balance_base - 100,
            solana.token_account_balance(owner_token_0).await
        );
        assert_eq!(
            balance_quote + 99980,
            solana.token_account_balance(owner_token_1).await
        );
    }

    send_tx(
        solana,
        ConsumeEventsInstruction {
            consume_events_admin: None,
            market,
            open_orders_accounts: vec![account_1],
        },
    )
    .await
    .unwrap();

    {
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;
        let open_orders_account_2 = solana.get_account::<OpenOrdersAccount>(account_2).await;

        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_2.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_2.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.base_free_native, 100);
        assert_eq!(open_orders_account_2.position.base_free_native, 0);
        assert_eq!(open_orders_account_1.position.quote_free_native, 20);
        assert_eq!(open_orders_account_2.position.quote_free_native, 0);
    }

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

    {
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;
        let open_orders_account_2 = solana.get_account::<OpenOrdersAccount>(account_2).await;

        assert_eq!(open_orders_account_1.position.base_free_native, 0);
        assert_eq!(open_orders_account_2.position.base_free_native, 0);
        assert_eq!(open_orders_account_1.position.quote_free_native, 0);
        assert_eq!(open_orders_account_2.position.quote_free_native, 0);
    }

    Ok(())
}

#[tokio::test]
async fn test_take_bid_order() -> Result<(), TransportError> {
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
    } = TestContext::new_with_market(TestNewMarketInitialize::default()).await?;
    let solana = &context.solana.clone();

    // Set the initial oracle price
    set_stub_oracle_price(solana, &tokens[1], collect_fee_admin, 1000.0).await;

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_token_0,
            market_vault: market_base_vault,
            side: Side::Ask,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10000,

            client_order_id: 0,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    let balance_base = solana.token_account_balance(owner_token_0).await;
    let balance_quote = solana.token_account_balance(owner_token_1).await;

    send_tx(
        solana,
        PlaceTakeOrderInstruction {
            market,
            signer: owner,
            user_base_account: owner_token_0,
            user_quote_account: owner_token_1,
            market_base_vault,
            market_quote_vault,
            side: Side::Bid,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10040,
            open_orders_admin: None,
        },
    )
    .await
    .unwrap();

    {
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;
        let open_orders_account_2 = solana.get_account::<OpenOrdersAccount>(account_2).await;

        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_2.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 1);
        assert_eq!(open_orders_account_2.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.base_free_native, 0);
        assert_eq!(open_orders_account_2.position.base_free_native, 0);
        assert_eq!(open_orders_account_1.position.quote_free_native, 0);
        assert_eq!(open_orders_account_2.position.quote_free_native, 0);
        assert_eq!(
            balance_base + 100,
            solana.token_account_balance(owner_token_0).await
        );
        assert_eq!(
            balance_quote - 100020,
            solana.token_account_balance(owner_token_1).await
        );
    }

    send_tx(
        solana,
        ConsumeEventsInstruction {
            consume_events_admin: None,
            market,
            open_orders_accounts: vec![account_1],
        },
    )
    .await
    .unwrap();

    {
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;
        let open_orders_account_2 = solana.get_account::<OpenOrdersAccount>(account_2).await;

        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_2.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_2.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.base_free_native, 0);
        assert_eq!(open_orders_account_2.position.base_free_native, 0);
        assert_eq!(open_orders_account_1.position.quote_free_native, 100020);
        assert_eq!(open_orders_account_2.position.quote_free_native, 0);
    }

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

    {
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;
        let open_orders_account_2 = solana.get_account::<OpenOrdersAccount>(account_2).await;

        assert_eq!(open_orders_account_1.position.base_free_native, 0);
        assert_eq!(open_orders_account_2.position.base_free_native, 0);
        assert_eq!(open_orders_account_1.position.quote_free_native, 0);
        assert_eq!(open_orders_account_2.position.quote_free_native, 0);
    }

    Ok(())
}

#[tokio::test]
async fn test_negative_spread_ask() -> Result<(), TransportError> {
    let context = TestContext::new().await;
    let solana = &context.solana.clone();

    let collect_fee_admin = TestKeypair::new();
    let close_market_admin = TestKeypair::new();
    let owner = context.users[0].key;
    let payer = context.users[1].key;
    let mints = &context.mints[0..=2];

    let owner_token_0 = context.users[0].token_accounts[0];

    let meta_owner_token_2 = context.users[1].token_accounts[2];

    let tokens = Token::create(mints.to_vec(), solana, collect_fee_admin, payer).await;

    let market = TestKeypair::new();

    let openbook_v2::accounts::CreateMarket {
        market,
        market_base_vault,
        market_quote_vault,
        event_heap,
        ..
    } = send_tx(
        solana,
        CreateMarketInstruction {
            collect_fee_admin: collect_fee_admin.pubkey(),
            open_orders_admin: None,
            close_market_admin: Some(close_market_admin.pubkey()),
            payer,
            market,
            quote_lot_size: 100,
            base_lot_size: 1_000_000_000,
            maker_fee: 0,
            taker_fee: 400,
            base_mint: mints[2].pubkey,
            quote_mint: mints[0].pubkey,
            ..CreateMarketInstruction::with_new_book_and_heap(solana, Some(tokens[1].oracle), None)
                .await
        },
    )
    .await
    .unwrap();

    let _indexer = create_open_orders_indexer(solana, &context.users[0], owner, market).await;
    let _indexer_2 = create_open_orders_indexer(solana, &context.users[1], payer, market).await;
    let maker_1 =
        create_open_orders_account(solana, owner, market, 1, &context.users[0], None).await;

    let maker_2 =
        create_open_orders_account(solana, payer, market, 1, &context.users[1], None).await;

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: maker_1,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_token_0,
            market_vault: market_quote_vault,
            side: Side::Bid,
            price_lots: 10_000,     // $1
            max_base_lots: 1000000, // wahtever
            max_quote_lots_including_fees: 10_000,
            client_order_id: 1,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    // This order doesn't take any due max_quote_lots_including_fees but it's also don't post in on the book
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: maker_2,
            open_orders_admin: None,
            market,
            signer: payer,
            user_token_account: meta_owner_token_2,
            market_vault: market_base_vault,
            side: Side::Ask,
            price_lots: 7_500,
            max_base_lots: 1,
            max_quote_lots_including_fees: 7_500,
            client_order_id: 25,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::AbortTransaction,
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    let position = solana
        .get_account::<OpenOrdersAccount>(maker_2)
        .await
        .position;

    assert_eq!(position.asks_base_lots, 0);
    assert_eq!(position.bids_base_lots, 0);

    {
        let event_heap = solana.get_account_boxed::<EventHeap>(event_heap).await;
        assert_eq!(event_heap.header.count(), 0);
    }

    Ok(())
}

#[tokio::test]
async fn test_negative_spread_bid() -> Result<(), TransportError> {
    let context = TestContext::new().await;
    let solana = &context.solana.clone();

    let collect_fee_admin = TestKeypair::new();
    let close_market_admin = TestKeypair::new();
    let owner = context.users[0].key;
    let payer = context.users[1].key;
    let mints = &context.mints[0..=2];

    let owner_token_1 = context.users[0].token_accounts[2];

    let meta_owner_token_1 = context.users[1].token_accounts[0];

    let tokens = Token::create(mints.to_vec(), solana, collect_fee_admin, payer).await;

    let market = TestKeypair::new();

    let openbook_v2::accounts::CreateMarket {
        market,
        market_base_vault,
        market_quote_vault,
        event_heap,
        ..
    } = send_tx(
        solana,
        CreateMarketInstruction {
            collect_fee_admin: collect_fee_admin.pubkey(),
            open_orders_admin: None,
            close_market_admin: Some(close_market_admin.pubkey()),
            payer,
            market,
            quote_lot_size: 1_000_000_000,
            base_lot_size: 100,
            maker_fee: 0,
            taker_fee: 400,
            base_mint: mints[2].pubkey,
            quote_mint: mints[0].pubkey,
            ..CreateMarketInstruction::with_new_book_and_heap(solana, Some(tokens[1].oracle), None)
                .await
        },
    )
    .await
    .unwrap();

    let _indexer = create_open_orders_indexer(solana, &context.users[0], owner, market).await;
    let _indexer_2 = create_open_orders_indexer(solana, &context.users[1], payer, market).await;
    let maker_1 =
        create_open_orders_account(solana, owner, market, 1, &context.users[0], None).await;

    let maker_2 =
        create_open_orders_account(solana, payer, market, 1, &context.users[1], None).await;

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: maker_1,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_token_1,
            market_vault: market_base_vault,
            side: Side::Ask,
            price_lots: 1000000,   // whatever
            max_base_lots: 10_000, // $1
            max_quote_lots_including_fees: 10_000,
            client_order_id: 1,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    // This order doesn't take any due max_base_lots but it's also don't post in on the book
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: maker_2,
            open_orders_admin: None,
            market,
            signer: payer,
            user_token_account: meta_owner_token_1,
            market_vault: market_quote_vault,
            side: Side::Bid,
            price_lots: 7_500,
            max_base_lots: 7_500,
            max_quote_lots_including_fees: 1,
            client_order_id: 25,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::AbortTransaction,
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    let position = solana
        .get_account::<OpenOrdersAccount>(maker_2)
        .await
        .position;

    assert_eq!(position.asks_base_lots, 0);
    assert_eq!(position.bids_base_lots, 0);

    {
        let event_heap = solana.get_account_boxed::<EventHeap>(event_heap).await;
        assert_eq!(event_heap.header.count(), 0);
    }

    Ok(())
}
