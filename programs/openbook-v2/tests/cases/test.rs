use super::*;

#[tokio::test]
async fn test_simple_settle() -> Result<(), TransportError> {
    let TestInitialize {
        context,
        collect_fee_admin,
        owner,
        payer,
        mints,
        owner_token_0,
        owner_token_1,
        market,

        base_vault,
        quote_vault,
        price_lots,
        tokens,
        account_1,
        account_2,
        close_market_admin,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize {
        close_market_admin_bool: true,
        ..TestNewMarketInitialize::default()
    })
    .await?;
    let solana = &context.solana.clone();

    //
    // TEST: Create another market
    //

    let market_2 = TestKeypair::new();
    let market_2_authority = Pubkey::find_program_address(
        &[b"Market".as_ref(), market_2.pubkey().to_bytes().as_ref()],
        &openbook_v2::id(),
    )
    .0;

    let base_vault_2 = solana
        .create_associated_token_account(&market_2_authority, mints[0].pubkey)
        .await;
    let quote_vault_2 = solana
        .create_associated_token_account(&market_2_authority, mints[1].pubkey)
        .await;

    send_tx(
        solana,
        CreateMarketInstruction {
            collect_fee_admin: collect_fee_admin.pubkey(),
            open_orders_admin: None,
            close_market_admin: None,
            payer,
            market: market_2,
            quote_lot_size: 10,
            base_lot_size: 100,
            maker_fee: -200,
            taker_fee: 400,
            base_mint: mints[0].pubkey,
            quote_mint: mints[1].pubkey,
            base_vault: base_vault_2,
            quote_vault: quote_vault_2,
            ..CreateMarketInstruction::with_new_book_and_queue(solana, None, None).await
        },
    )
    .await
    .unwrap();

    // Set the initial oracle price
    set_stub_oracle_price(solana, &tokens[1], collect_fee_admin, 1000.0).await;

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            signer: owner,
            token_deposit_account: owner_token_1,
            market_vault: quote_vault,
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

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_2,
            open_orders_admin: None,
            market,
            signer: owner,
            token_deposit_account: owner_token_0,
            market_vault: base_vault,
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
        assert_eq!(open_orders_account_2.position.quote_free_native, 99960);
    }

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

    send_tx(
        solana,
        SettleFundsInstruction {
            owner,
            market,
            open_orders_account: account_1,
            base_vault,
            quote_vault,
            token_base_account: owner_token_0,
            token_quote_account: owner_token_1,
            referrer: None,
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
        assert_eq!(open_orders_account_2.position.quote_free_native, 99960);
    }

    send_tx(
        solana,
        SettleFundsInstruction {
            owner,
            market,
            open_orders_account: account_2,
            base_vault,
            quote_vault,
            token_base_account: owner_token_0,
            token_quote_account: owner_token_1,
            referrer: None,
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

    send_tx(
        solana,
        SetMarketExpiredInstruction {
            close_market_admin,
            market,
        },
    )
    .await
    .unwrap();

    send_tx(
        solana,
        CloseMarketInstruction {
            close_market_admin,
            market,
            sol_destination: owner.pubkey(),
        },
    )
    .await
    .unwrap();

    Ok(())
}

#[tokio::test]
async fn test_cancel_orders() -> Result<(), TransportError> {
    let TestInitialize {
        context,
        owner,
        owner_token_0,
        owner_token_1,
        market,

        base_vault,
        quote_vault,
        price_lots,
        account_1,
        account_2,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize {
        maker_fee: -100,
        ..TestNewMarketInitialize::default()
    })
    .await?;
    let solana = &context.solana.clone();

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            signer: owner,
            token_deposit_account: owner_token_1,
            market_vault: quote_vault,
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

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_2,
            open_orders_admin: None,
            market,
            signer: owner,
            token_deposit_account: owner_token_0,
            market_vault: base_vault,
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
        assert_eq!(open_orders_account_2.position.quote_free_native, 99960);
    }

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

    {
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;
        let open_orders_account_2 = solana.get_account::<OpenOrdersAccount>(account_2).await;

        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_2.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_2.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.base_free_native, 100);
        assert_eq!(open_orders_account_2.position.base_free_native, 0);
        assert_eq!(open_orders_account_1.position.quote_free_native, 10);
        assert_eq!(open_orders_account_2.position.quote_free_native, 99960);
    }

    send_tx(
        solana,
        DepositInstruction {
            owner,
            market,
            open_orders_account: account_1,
            base_vault,
            quote_vault,
            token_base_account: owner_token_0,
            token_quote_account: owner_token_1,
            base_amount: 10000,
            quote_amount: 0,
        },
    )
    .await
    .unwrap();

    {
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;
        assert_eq!(open_orders_account_1.position.base_free_native, 10100);
        assert_eq!(open_orders_account_1.position.quote_free_native, 10);
    }

    let balance = solana.token_account_balance(owner_token_0).await;

    // Assets should be free, let's post the opposite orders
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            signer: owner,
            token_deposit_account: owner_token_0,
            market_vault: base_vault,
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

    {
        assert_eq!(balance, solana.token_account_balance(owner_token_0).await);
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;
        let open_orders_account_2 = solana.get_account::<OpenOrdersAccount>(account_2).await;

        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_2.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 1);
        assert_eq!(open_orders_account_2.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.base_free_native, 10000);
        assert_eq!(open_orders_account_2.position.base_free_native, 0);
        assert_eq!(open_orders_account_1.position.quote_free_native, 10);
        assert_eq!(open_orders_account_2.position.quote_free_native, 99960);
    }

    send_tx(
        solana,
        SettleFundsInstruction {
            owner,
            market,
            open_orders_account: account_1,
            base_vault,
            quote_vault,
            token_base_account: owner_token_0,
            token_quote_account: owner_token_1,
            referrer: None,
        },
    )
    .await
    .unwrap();

    {
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;
        let open_orders_account_2 = solana.get_account::<OpenOrdersAccount>(account_2).await;

        assert_eq!(open_orders_account_1.position.asks_base_lots, 1);
        assert_eq!(open_orders_account_1.position.base_free_native, 0);
        assert_eq!(open_orders_account_2.position.base_free_native, 0);
        assert_eq!(open_orders_account_1.position.quote_free_native, 0);
        assert_eq!(open_orders_account_2.position.quote_free_native, 99960);
    }

    let order_id_to_cancel = solana
        .get_account::<OpenOrdersAccount>(account_1)
        .await
        .open_orders[0]
        .id;

    send_tx(
        solana,
        CancelOrderInstruction {
            signer: owner,
            market,
            open_orders_account: account_1,
            order_id: order_id_to_cancel,
        },
    )
    .await
    .unwrap();

    {
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;

        assert_eq!(open_orders_account_1.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.base_free_native, 100);
        assert_eq!(open_orders_account_1.position.quote_free_native, 0);
    }

    // Post and cancel Bid
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            signer: owner,
            token_deposit_account: owner_token_1,
            market_vault: quote_vault,
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
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;

        assert_eq!(open_orders_account_1.position.bids_base_lots, 1);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.base_free_native, 100);
        assert_eq!(open_orders_account_1.position.quote_free_native, 0);
    }

    let order_id_to_cancel = solana
        .get_account::<OpenOrdersAccount>(account_1)
        .await
        .open_orders[0]
        .id;

    send_tx(
        solana,
        CancelOrderInstruction {
            signer: owner,
            market,
            open_orders_account: account_1,
            order_id: order_id_to_cancel,
        },
    )
    .await
    .unwrap();

    {
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;

        assert_eq!(open_orders_account_1.position.base_free_native, 100);
        assert_eq!(open_orders_account_1.position.quote_free_native, 100000);
    }

    Ok(())
}

#[tokio::test]
async fn test_expired_orders() -> Result<(), TransportError> {
    let TestInitialize {
        context,
        owner,
        owner_token_0,
        owner_token_1,
        market,
        base_vault,
        quote_vault,
        price_lots,
        account_1,
        account_2,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize::default()).await?;
    let solana = &context.solana.clone();

    // Order with expiry time of 2s
    let now_ts: u64 = solana.get_clock().await.unix_timestamp as u64;
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            signer: owner,
            token_deposit_account: owner_token_1,
            market_vault: quote_vault,
            side: Side::Bid,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10000,

            client_order_id: 0,
            expiry_timestamp: now_ts + 2,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    {
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;

        assert_eq!(open_orders_account_1.position.bids_base_lots, 1);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.base_free_native, 0);
        assert_eq!(open_orders_account_1.position.quote_free_native, 0);
    }

    // Advance clock
    solana.advance_clock(2).await;
    // Bid isn't available anymore, shouldn't be matched. Introduces event on the event_queue
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_2,
            open_orders_admin: None,
            market,
            signer: owner,
            token_deposit_account: owner_token_0,
            market_vault: base_vault,
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
    {
        let market_acc = solana.get_account_boxed::<Market>(market).await;
        let event_queue = solana
            .get_account_boxed::<EventQueue>(market_acc.event_queue)
            .await;
        assert_eq!(event_queue.header.count(), 1);
    }
    {
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;
        let open_orders_account_2 = solana.get_account::<OpenOrdersAccount>(account_2).await;

        assert_eq!(open_orders_account_1.position.bids_base_lots, 1);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.base_free_native, 0);
        assert_eq!(open_orders_account_1.position.quote_free_native, 0);
        assert_eq!(open_orders_account_2.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_2.position.asks_base_lots, 1);
        assert_eq!(open_orders_account_2.position.base_free_native, 0);
        assert_eq!(open_orders_account_2.position.quote_free_native, 0);
    }

    // ConsumeEvents removes the bids_base_lots in the Out event
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
    {
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;
        let open_orders_account_2 = solana.get_account::<OpenOrdersAccount>(account_2).await;

        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.base_free_native, 0);
        assert_eq!(open_orders_account_1.position.quote_free_native, 100000);
        assert_eq!(open_orders_account_2.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_2.position.asks_base_lots, 1);
        assert_eq!(open_orders_account_2.position.base_free_native, 0);
        assert_eq!(open_orders_account_2.position.quote_free_native, 0);
    }
    // No more events on event_queue
    {
        let market_acc = solana.get_account::<Market>(market).await;
        let event_queue = solana
            .get_account::<EventQueue>(market_acc.event_queue)
            .await;

        assert_eq!(event_queue.header.count(), 0);
    }

    Ok(())
}
