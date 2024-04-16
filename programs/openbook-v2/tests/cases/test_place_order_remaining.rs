use super::*;

#[tokio::test]
async fn test_place_cancel_order_remaining() -> Result<(), TransportError> {
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
        bids,
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

    {
        let bids_data = solana.get_account_boxed::<BookSide>(bids).await;
        assert_eq!(bids_data.roots[0].leaf_count, 1);
    }

    // Add remainings, no event on event_heap
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_2,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_token_0,
            market_vault: market_base_vault,
            side: Side::Ask,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10004,

            client_order_id: 0,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![account_1],
        },
    )
    .await
    .unwrap();

    {
        let bids_data = solana.get_account_boxed::<BookSide>(bids).await;
        assert_eq!(bids_data.roots[0].leaf_count, 0);
    }

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
        assert_eq!(open_orders_account_2.position.quote_free_native, 99960);
    }

    // No events on event_heap
    {
        let market_acc = solana.get_account::<Market>(market).await;
        let event_heap = solana.get_account::<EventHeap>(market_acc.event_heap).await;

        assert_eq!(event_heap.header.count(), 0);
    }

    // Order with expiry time of 10s
    let now_ts: u64 = solana.get_clock().await.unix_timestamp as u64;
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

            client_order_id: 35,
            expiry_timestamp: now_ts + 10,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();
    {
        let bids_data = solana.get_account_boxed::<BookSide>(bids).await;
        assert_eq!(bids_data.roots[0].leaf_count, 1);
    }

    // Advance clock
    solana.advance_clock(11).await;

    {
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;
        assert_eq!(open_orders_account_1.position.bids_base_lots, 1);
    }

    // Add remainings, no event on event_heap. previous order is canceled
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_2,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_token_0,
            market_vault: market_base_vault,
            side: Side::Ask,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10000,

            client_order_id: 36,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![account_1],
        },
    )
    .await
    .unwrap();
    // bid has been canceled
    {
        let bids_data = solana.get_account_boxed::<BookSide>(bids).await;
        assert_eq!(bids_data.roots[0].leaf_count, 0);
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;
        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
    }

    // No events on event_heap
    {
        let market_acc = solana.get_account_boxed::<Market>(market).await;
        let event_heap = solana
            .get_account_boxed::<EventHeap>(market_acc.event_heap)
            .await;

        assert_eq!(event_heap.header.count(), 0);
    }

    Ok(())
}

#[tokio::test]
async fn test_cancel_order_yourself() -> Result<(), TransportError> {
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
        bids,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize::default()).await?;
    let solana = &context.solana.clone();

    // Set the initial oracle price
    set_stub_oracle_price(solana, &tokens[1], collect_fee_admin, 1000.0).await;

    // Order with expiry time of 10s
    let now_ts: u64 = solana.get_clock().await.unix_timestamp as u64;
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
            expiry_timestamp: now_ts + 10,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    {
        let bids_data = solana.get_account_boxed::<BookSide>(bids).await;
        assert_eq!(bids_data.roots[0].leaf_count, 1);
    }

    // Advance clock
    solana.advance_clock(11).await;

    // No remainings, same account, previos bid is canceled
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_2,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_token_0,
            market_vault: market_base_vault,
            side: Side::Ask,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10004,

            client_order_id: 0,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![account_1],
        },
    )
    .await
    .unwrap();

    {
        let bids_data = solana.get_account_boxed::<BookSide>(bids).await;
        assert_eq!(bids_data.roots[0].leaf_count, 0);
    }

    Ok(())
}

#[tokio::test]
async fn test_place_order_taker_fees() -> Result<(), TransportError> {
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
        bids,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize {
        taker_fee: 11000, // 1.1%
        maker_fee: 0,
        quote_lot_size: 1000,
        base_lot_size: 1,
        ..Default::default()
    })
    .await?;
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
            max_base_lots: 500,
            max_quote_lots_including_fees: 500,
            client_order_id: 0,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    // Now place place a bid that fills the ask fully and has some remainder go to the book
    let before_quote_balance = solana.token_account_balance(owner_token_1).await;
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_2,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_token_1,
            market_vault: market_quote_vault,
            side: Side::Bid,
            price_lots,
            max_base_lots: 9999999, // unlimited
            max_quote_lots_including_fees: 1000,
            client_order_id: 0,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();
    let after_quote_balance = solana.token_account_balance(owner_token_1).await;

    // What should have happened is:
    // - match against the ask, paying 500 quote lots for 500 base lots
    // - taker fee native is 1.1% * 500 * 1000 = 5500 native
    // - which is 5.5 quote lots, so only 500 - 6 = 494 quote lots can be placed on the book

    let open_orders_account_2 = solana.get_account::<OpenOrdersAccount>(account_2).await;
    assert_eq!(open_orders_account_2.position.bids_quote_lots, 494);
    assert_eq!(open_orders_account_2.position.base_free_native, 500);

    assert_eq!(
        before_quote_balance - after_quote_balance,
        // cost of buying 500 base lots
        500 * 1000
        // taker fee
        + 5500
        // order on the book
        + 494 * 1000
    );

    Ok(())
}
