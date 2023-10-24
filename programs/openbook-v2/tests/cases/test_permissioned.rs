use super::*;

#[tokio::test]
async fn test_permissioned_open_order() -> Result<(), TransportError> {
    let TestInitialize {
        context,
        collect_fee_admin,
        owner,
        owner_token_1,
        market,
        market_quote_vault,
        tokens,
        account_1,
        open_orders_admin,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize {
        open_orders_admin_bool: true,
        ..TestNewMarketInitialize::default()
    })
    .await?;
    let solana = &context.solana.clone();

    let price_lots = {
        let market = solana.get_account::<Market>(market).await;
        market.native_price_to_lot(I80F48::from(1000)).unwrap()
    };

    // Set the initial oracle price
    set_stub_oracle_price(solana, &tokens[1], collect_fee_admin, 1000.0).await;

    // First, send in an order w/o the signature of the open order authority, expect failure
    let result = send_tx(
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
    .await;

    assert!(result.is_err());

    // Second, send in an order w/ the signature of the open order authority, expect success
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            open_orders_admin: Some(open_orders_admin),
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

    Ok(())
}

#[tokio::test]
async fn test_permissioned_open_take_order() -> Result<(), TransportError> {
    let TestInitialize {
        context,
        collect_fee_admin,
        open_orders_admin,
        owner,
        owner_token_1,
        market,
        market_quote_vault,
        price_lots,
        tokens,
        account_1,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize {
        open_orders_admin_bool: true,
        ..TestNewMarketInitialize::default()
    })
    .await?;
    let solana = &context.solana.clone();

    // Set the initial oracle price
    set_stub_oracle_price(solana, &tokens[1], collect_fee_admin, 1000.0).await;

    let result = send_tx(
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
    .await;

    assert!(result.is_err());

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            open_orders_admin: Some(open_orders_admin),
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

    Ok(())
}

#[tokio::test]
async fn test_consume_events_admin() -> Result<(), TransportError> {
    let TestInitialize {
        context,
        collect_fee_admin,
        consume_events_admin,
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
        consume_events_admin_bool: true,
        ..TestNewMarketInitialize::default()
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

            client_order_id: 0,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    let result = send_tx(
        solana,
        ConsumeEventsInstruction {
            consume_events_admin: None,
            market,
            open_orders_accounts: vec![account_1, account_2],
        },
    )
    .await;

    assert!(result.is_err());

    send_tx(
        solana,
        ConsumeEventsInstruction {
            consume_events_admin: Some(consume_events_admin),
            market,
            open_orders_accounts: vec![account_1, account_2],
        },
    )
    .await
    .unwrap();

    Ok(())
}

#[tokio::test]
async fn test_close_market_admin() -> Result<(), TransportError> {
    let TestInitialize {
        context,
        close_market_admin,
        collect_fee_admin,
        owner,
        mints,
        owner_token_0,
        owner_token_1,
        market,
        market_base_vault,
        market_quote_vault,
        price_lots,
        account_1,
        account_2,
        payer,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize {
        close_market_admin_bool: true,
        ..TestNewMarketInitialize::default()
    })
    .await?;
    let solana = &context.solana.clone();

    let fee_admin_ata = solana
        .create_associated_token_account(&collect_fee_admin.pubkey(), mints[1].pubkey)
        .await;

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

    // Place an order that matches
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
            client_order_id: 0,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    // Place another order
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

    let close_ix = CloseMarketInstruction {
        close_market_admin,
        market,
        sol_destination: owner.pubkey(),
    };

    let settle_funds_expired_ix = SettleFundsExpiredInstruction {
        close_market_admin,
        market,
        owner: payer,
        open_orders_account: account_2,
        market_base_vault,
        market_quote_vault,
        user_base_account: owner_token_0,
        user_quote_account: owner_token_1,
        referrer_account: None,
    };

    // Can't close yet, market not market as expired
    assert!(send_tx(solana, close_ix.clone()).await.is_err());

    // also not possible to settle in behalf of the users
    assert!(send_tx(solana, settle_funds_expired_ix.clone())
        .await
        .is_err());

    send_tx(
        solana,
        SetMarketExpiredInstruction {
            close_market_admin,
            market,
        },
    )
    .await
    .unwrap();

    {
        let market = solana.get_account::<Market>(market).await;
        assert_eq!(market.time_expiry, -1);
    }

    // Can't post orders anymore
    let result = send_tx(
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
    .await;
    assert!(result.is_err());

    // Consume events
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

    // Can't close, have to prune orders first
    assert!(send_tx(solana, close_ix.clone()).await.is_err());

    send_tx(
        solana,
        PruneOrdersInstruction {
            close_market_admin,
            market,
            open_orders_account: account_1,
        },
    )
    .await
    .unwrap();

    // and wait until users settle funds
    {
        let market = solana.get_account::<Market>(market).await;
        assert!(market.base_deposit_total != 0);
        assert!(market.quote_deposit_total != 0);
    }
    assert!(send_tx(solana, close_ix.clone()).await.is_err());

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

    // which can be even be called by the close_market_admin once the market is expired so it
    // doesn't have to wait for the users!
    send_tx(solana, settle_funds_expired_ix).await.unwrap();

    // but wait! the're still pending fees
    {
        let market = solana.get_account::<Market>(market).await;
        assert!(market.fees_available != 0);
    }
    assert!(send_tx(solana, close_ix.clone()).await.is_err());

    send_tx(
        solana,
        SweepFeesInstruction {
            collect_fee_admin,
            market,
            market_quote_vault,
            token_receiver_account: fee_admin_ata,
        },
    )
    .await
    .unwrap();

    // Boom
    send_tx(solana, close_ix.clone()).await.unwrap();

    Ok(())
}

#[tokio::test]
async fn test_delegate() -> Result<(), TransportError> {
    let TestInitialize {
        context,
        collect_fee_admin,
        owner,
        market,
        market_quote_vault,
        price_lots,
        tokens,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize {
        ..TestNewMarketInitialize::default()
    })
    .await?;
    let solana = &context.solana.clone();

    let account_3_delegate = context.users[2].key;
    let account_3 = create_open_orders_account(
        solana,
        owner,
        market,
        3,
        &context.users[0],
        Some(account_3_delegate.pubkey()),
    )
    .await;
    let delegate_token_1 = context.users[2].token_accounts[1];

    // Set the initial oracle price
    set_stub_oracle_price(solana, &tokens[1], collect_fee_admin, 1000.0).await;

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_3,
            open_orders_admin: None,
            market,
            signer: account_3_delegate,
            user_token_account: delegate_token_1,
            market_vault: market_quote_vault,
            side: Side::Bid,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10000,

            client_order_id: 23,
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
        CancelOrderByClientOrderIdInstruction {
            signer: account_3_delegate,
            market,
            open_orders_account: account_3,
            client_order_id: 23,
        },
    )
    .await
    .unwrap();

    send_tx(
        solana,
        SetDelegateInstruction {
            owner,
            open_orders_account: account_3,
            delegate_account: None,
        },
    )
    .await
    .unwrap();

    // No delegate anymore
    let result = send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_3,
            open_orders_admin: None,
            market,
            signer: account_3_delegate,
            user_token_account: delegate_token_1,
            market_vault: market_quote_vault,
            side: Side::Bid,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10000,

            client_order_id: 23,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await;
    assert!(result.is_err());

    Ok(())
}
