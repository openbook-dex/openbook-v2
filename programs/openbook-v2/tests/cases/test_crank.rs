use super::*;
use bytemuck::cast_ref;

#[tokio::test]
async fn test_skip_missing_accounts() -> Result<(), TransportError> {
    let context = TestContext::new().await;
    let solana = &context.solana.clone();

    let collect_fee_admin = TestKeypair::new();
    let close_market_admin = TestKeypair::new();
    let owner = context.users[0].key;
    let payer = context.users[1].key;
    let mints = &context.mints[0..=2];

    let owner_token_0 = context.users[0].token_accounts[0];
    let owner_token_1 = context.users[0].token_accounts[1];

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
            quote_lot_size: 10,
            base_lot_size: 100,
            maker_fee: -200,
            taker_fee: 400,
            base_mint: mints[0].pubkey,
            quote_mint: mints[1].pubkey,
            ..CreateMarketInstruction::with_new_book_and_heap(solana, Some(tokens[1].oracle), None)
                .await
        },
    )
    .await
    .unwrap();

    let price_lots = {
        let market = solana.get_account::<Market>(market).await;
        market.native_price_to_lot(I80F48::ONE).unwrap()
    };

    let _indexer = create_open_orders_indexer(solana, &context.users[1], owner, market).await;
    let (maker_1, maker_2, maker_3) = {
        (
            create_open_orders_account(solana, owner, market, 1, &context.users[1], None).await,
            create_open_orders_account(solana, owner, market, 2, &context.users[1], None).await,
            create_open_orders_account(solana, owner, market, 3, &context.users[1], None).await,
        )
    };
    let taker = create_open_orders_account(solana, owner, market, 4, &context.users[1], None).await;

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: maker_1,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_token_1,
            market_vault: market_quote_vault,
            side: Side::Bid,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10000,
            client_order_id: 1,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: maker_2,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_token_1,
            market_vault: market_quote_vault,
            side: Side::Bid,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10000,
            client_order_id: 2,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: maker_3,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_token_1,
            market_vault: market_quote_vault,
            side: Side::Bid,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10000,
            client_order_id: 3,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: taker,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_token_0,
            market_vault: market_base_vault,
            side: Side::Ask,
            price_lots,
            max_base_lots: 3,
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

    {
        let event_heap = solana.get_account_boxed::<EventHeap>(event_heap).await;
        assert_eq!(event_heap.header.count(), 3);
        assert_eq!(fill_maker(event_heap.at_slot(0).unwrap()), maker_1);
        assert_eq!(fill_maker(event_heap.at_slot(1).unwrap()), maker_2);
        assert_eq!(fill_maker(event_heap.at_slot(2).unwrap()), maker_3);
    }

    send_tx(
        solana,
        ConsumeEventsInstruction {
            consume_events_admin: None,
            market,
            open_orders_accounts: vec![maker_2, maker_3],
        },
    )
    .await
    .unwrap();

    {
        let event_heap = solana.get_account_boxed::<EventHeap>(event_heap).await;
        assert_eq!(event_heap.header.count(), 1);
        assert_eq!(fill_maker(event_heap.front().unwrap()), maker_1);
    }

    Ok(())
}

#[tokio::test]
async fn test_crank_given_events() -> Result<(), TransportError> {
    let context = TestContext::new().await;
    let solana = &context.solana.clone();

    let collect_fee_admin = TestKeypair::new();
    let close_market_admin = TestKeypair::new();
    let owner = context.users[0].key;
    let payer = context.users[1].key;
    let mints = &context.mints[0..=2];

    let owner_token_0 = context.users[0].token_accounts[0];
    let owner_token_1 = context.users[0].token_accounts[1];

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
            quote_lot_size: 10,
            base_lot_size: 100,
            maker_fee: -200,
            taker_fee: 400,
            base_mint: mints[0].pubkey,
            quote_mint: mints[1].pubkey,
            ..CreateMarketInstruction::with_new_book_and_heap(solana, Some(tokens[0].oracle), None)
                .await
        },
    )
    .await
    .unwrap();

    let price_lots = {
        let market = solana.get_account::<Market>(market).await;
        market.native_price_to_lot(I80F48::ONE).unwrap()
    };

    let _indexer = create_open_orders_indexer(solana, &context.users[1], owner, market).await;
    let (maker_1, maker_2, maker_3) = {
        (
            create_open_orders_account(solana, owner, market, 1, &context.users[1], None).await,
            create_open_orders_account(solana, owner, market, 2, &context.users[1], None).await,
            create_open_orders_account(solana, owner, market, 3, &context.users[1], None).await,
        )
    };
    let taker = create_open_orders_account(solana, owner, market, 4, &context.users[1], None).await;

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: maker_1,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_token_1,
            market_vault: market_quote_vault,
            side: Side::Bid,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10000,
            client_order_id: 1,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: maker_2,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_token_1,
            market_vault: market_quote_vault,
            side: Side::Bid,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10000,
            client_order_id: 2,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: maker_3,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_token_1,
            market_vault: market_quote_vault,
            side: Side::Bid,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10000,
            client_order_id: 3,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: taker,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_token_0,
            market_vault: market_base_vault,
            side: Side::Ask,
            price_lots,
            max_base_lots: 3,
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

    {
        let event_heap = solana.get_account_boxed::<EventHeap>(event_heap).await;
        assert_eq!(event_heap.header.count(), 3);
        assert_eq!(fill_maker(event_heap.at_slot(0).unwrap()), maker_1);
        assert_eq!(fill_maker(event_heap.at_slot(1).unwrap()), maker_2);
        assert_eq!(fill_maker(event_heap.at_slot(2).unwrap()), maker_3);
    }

    send_tx(
        solana,
        ConsumeGivenEventsInstruction {
            consume_events_admin: None,
            market,
            slots: vec![2, 0],
            open_orders_accounts: vec![maker_1, maker_3],
        },
    )
    .await
    .unwrap();

    {
        let event_heap = solana.get_account_boxed::<EventHeap>(event_heap).await;
        assert_eq!(event_heap.header.count(), 1);
        assert_eq!(fill_maker(event_heap.front().unwrap()), maker_2);
    }

    // is not possible to process slots > limit
    assert!(send_tx(
        solana,
        ConsumeGivenEventsInstruction {
            consume_events_admin: None,
            market,
            slots: vec![openbook_v2::state::MAX_NUM_EVENTS.into()],
            open_orders_accounts: vec![maker_2],
        },
    )
    .await
    .is_err());

    // but if non-valid free slots are sent, the crank is performed from the front
    send_tx(
        solana,
        ConsumeGivenEventsInstruction {
            consume_events_admin: None,
            market,
            slots: vec![100, 100, 200],
            open_orders_accounts: vec![maker_2],
        },
    )
    .await
    .unwrap();

    {
        let event_heap = solana.get_account_boxed::<EventHeap>(event_heap).await;
        assert_eq!(event_heap.header.count(), 0);
    }

    Ok(())
}

fn fill_maker(anyevent: &AnyEvent) -> Pubkey {
    let event: &FillEvent = cast_ref(anyevent);
    event.maker
}
