use super::*;

#[tokio::test]
async fn test_place_cancel_order_remaining() -> Result<(), TransportError> {
    let context = TestContext::new().await;
    let solana = &context.solana.clone();

    let admin = TestKeypair::new();
    let owner = context.users[0].key;
    let payer = context.users[1].key;
    let mints = &context.mints[0..=2];

    let owner_token_0 = context.users[0].token_accounts[0];
    let owner_token_1 = context.users[0].token_accounts[1];

    let tokens = Token::create(mints.to_vec(), solana, admin, payer).await;

    //
    // TEST: Create a market
    //
    let market = get_market_address(admin.pubkey(), 1);
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
        bids,
        ..
    } = send_tx(
        solana,
        CreateMarketInstruction {
            admin,
            payer,
            market_index: 1,
            quote_lot_size: 10,
            base_lot_size: 100,
            maker_fee: -0.0002,
            taker_fee: 0.0004,
            base_mint: mints[0].pubkey,
            quote_mint: mints[1].pubkey,
            base_vault,
            quote_vault,
            ..CreateMarketInstruction::with_new_book_and_queue(solana, &tokens[1]).await
        },
    )
    .await
    .unwrap();

    let account_0 = create_open_orders_account(solana, owner, market, 0, &context.users[1]).await;
    let account_1 = create_open_orders_account(solana, owner, market, 1, &context.users[1]).await;

    let price_lots = {
        let market = solana.get_account::<Market>(market).await;
        market.native_price_to_lot(I80F48::from(1000))
    };

    // Set the initial oracle price
    set_stub_oracle_price(solana, &tokens[1], admin, 1000.0).await;

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_0,
            market,
            owner,
            payer: owner_token_1,
            base_vault,
            quote_vault,
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

    // Add remainings, no event on event_queue
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            market,
            owner,
            payer: owner_token_0,
            base_vault,
            quote_vault,
            side: Side::Ask,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10004,

            client_order_id: 0,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![account_0],
        },
    )
    .await
    .unwrap();

    {
        let bids_data = solana.get_account_boxed::<BookSide>(bids).await;
        assert_eq!(bids_data.roots[0].leaf_count, 0);
    }

    {
        let open_orders_account_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;

        assert_eq!(open_orders_account_0.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_0.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_0.position.base_free_native, 100);
        assert_eq!(open_orders_account_1.position.base_free_native, 0);
        assert_eq!(open_orders_account_0.position.quote_free_native.round(), 20);
        assert_eq!(
            open_orders_account_1.position.quote_free_native.round(),
            99960
        );
    }

    // No events on event_queue
    {
        let market_acc = solana.get_account::<Market>(market).await;
        let event_queue = solana
            .get_account::<EventQueue>(market_acc.event_queue)
            .await;

        assert_eq!(event_queue.header.count(), 0);
    }

    // Order with expiry time of 10s
    let now_ts: u64 = solana.get_clock().await.unix_timestamp as u64;
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_0,
            market,
            owner,
            payer: owner_token_1,
            base_vault,
            quote_vault,
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
        let open_orders_account_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;
        assert_eq!(open_orders_account_0.position.bids_base_lots, 1);
    }

    // Add remainings, no event on event_queue. previous order is canceled
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            market,
            owner,
            payer: owner_token_0,
            base_vault,
            quote_vault,
            side: Side::Ask,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10000,

            client_order_id: 36,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![account_0],
        },
    )
    .await
    .unwrap();
    // bid has been canceled
    {
        let bids_data = solana.get_account_boxed::<BookSide>(bids).await;
        assert_eq!(bids_data.roots[0].leaf_count, 0);
        let open_orders_account_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;
        assert_eq!(open_orders_account_0.position.bids_base_lots, 0);
    }

    // No events on event_queue
    // {
    //     let market_acc = solana.get_account::<Market>(market).await;
    //     let event_queue = solana
    //         .get_account::<EventQueue>(market_acc.event_queue)
    //         .await;

    //     assert_eq!(event_queue.header.count(), 0);
    // }

    Ok(())
}
