use super::*;

#[tokio::test]
async fn test_fees_accrued() -> Result<(), TransportError> {
    let fee_penalty = 1000;
    let TestInitialize {
        context,
        collect_fee_admin,
        owner,
        mints,
        owner_token_0,
        owner_token_1,
        market,
        base_vault,
        quote_vault,
        price_lots,
        tokens,
        account_0,
        account_1,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize {
        fee_penalty,
        maker_fee: -100,
        taker_fee: 200,
        ..TestNewMarketInitialize::default()
    })
    .await?;
    let solana = &context.solana.clone();

    // Set the initial oracle price
    set_stub_oracle_price(solana, &tokens[1], collect_fee_admin, 1000.0).await;

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_0,
            open_orders_admin: None,
            market,
            owner,
            token_deposit_account: owner_token_1,
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

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            owner,
            token_deposit_account: owner_token_0,
            base_vault,
            quote_vault,
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
        let open_orders_account_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;

        assert_eq!(open_orders_account_0.position.bids_base_lots, 1);
        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_0.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_0.position.base_free_native, 0);
        assert_eq!(open_orders_account_1.position.base_free_native, 0);
        assert_eq!(open_orders_account_0.position.quote_free_native, 0);
        assert_eq!(open_orders_account_1.position.quote_free_native, 99980);
    }

    send_tx(
        solana,
        ConsumeEventsInstruction {
            consume_events_admin: None,
            market,
            open_orders_accounts: vec![account_0, account_1],
        },
    )
    .await
    .unwrap();

    {
        let open_orders_account_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;

        assert_eq!(open_orders_account_0.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_0.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_0.position.base_free_native, 100);
        assert_eq!(open_orders_account_1.position.base_free_native, 0);
        assert_eq!(open_orders_account_0.position.quote_free_native, 10);
        assert_eq!(open_orders_account_1.position.quote_free_native, 99980);
    }

    let admin_token_1 = solana
        .create_associated_token_account(&collect_fee_admin.pubkey(), mints[1].pubkey)
        .await;

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
        let market = solana.get_account::<Market>(market).await;
        assert_eq!(market.quote_fees_accrued, 10);
        assert_eq!(market.fees_accrued, 10);
        assert_eq!(market.fees_to_referrers, 0);
    }

    send_tx(
        solana,
        SweepFeesInstruction {
            collect_fee_admin,
            market,
            quote_vault,
            token_receiver_account: admin_token_1,
        },
    )
    .await
    .unwrap();

    {
        let market = solana.get_account::<Market>(market).await;
        assert_eq!(market.quote_fees_accrued, 0);
        assert_eq!(market.fees_accrued, 10);
        assert_eq!(market.fees_to_referrers, 0);
    }

    let balance_quote = solana.token_account_balance(owner_token_1).await;

    // Order with penalty fees
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            owner,
            token_deposit_account: owner_token_1,
            base_vault,
            quote_vault,
            side: Side::Bid,
            price_lots: price_lots - 1000,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10000,
            client_order_id: 0,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::ImmediateOrCancel,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    {
        let market = solana.get_account::<Market>(market).await;
        assert_eq!(market.quote_fees_accrued, 1000);
        assert_eq!(market.fees_accrued, 10);
        assert_eq!(market.fees_to_referrers, 0);
        assert_eq!(
            balance_quote - fee_penalty,
            solana.token_account_balance(owner_token_1).await
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_maker_fees() -> Result<(), TransportError> {
    let TestInitialize {
        context,
        collect_fee_admin,
        owner,
        mints,
        owner_token_0,
        owner_token_1,
        market,
        base_vault,
        quote_vault,
        price_lots,
        tokens,
        account_0,
        account_1,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize {
        maker_fee: 200,
        taker_fee: 400,
        ..TestNewMarketInitialize::default()
    })
    .await?;
    let solana = &context.solana.clone();

    // Set the initial oracle price
    set_stub_oracle_price(solana, &tokens[1], collect_fee_admin, 1000.0).await;

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_0,
            open_orders_admin: None,
            market,
            owner,
            token_deposit_account: owner_token_1,
            base_vault,
            quote_vault,
            side: Side::Bid,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10002,
            client_order_id: 30,
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
            open_orders_account: account_0,
            market,
            owner,
            client_order_id: 30,
        },
    )
    .await
    .unwrap();

    {
        let open_orders_account_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;

        assert_eq!(open_orders_account_0.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_0.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_0.position.base_free_native, 0);
        assert_eq!(open_orders_account_0.position.quote_free_native, 100020);
    }

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_0,
            open_orders_admin: None,
            market,
            owner,
            token_deposit_account: owner_token_1,
            base_vault,
            quote_vault,
            side: Side::Bid,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10002,
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
        let open_orders_account_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;

        assert_eq!(open_orders_account_0.position.bids_base_lots, 1);
        assert_eq!(open_orders_account_0.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_0.position.base_free_native, 0);
        assert_eq!(open_orders_account_0.position.quote_free_native, 0);
    }

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            owner,
            token_deposit_account: owner_token_0,
            base_vault,
            quote_vault,
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
        let open_orders_account_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;

        assert_eq!(open_orders_account_0.position.bids_base_lots, 1);
        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_0.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_0.position.base_free_native, 0);
        assert_eq!(open_orders_account_1.position.base_free_native, 0);
        assert_eq!(open_orders_account_0.position.quote_free_native, 0);
        assert_eq!(open_orders_account_1.position.quote_free_native, 99960);
    }

    send_tx(
        solana,
        ConsumeEventsInstruction {
            consume_events_admin: None,
            market,
            open_orders_accounts: vec![account_0, account_1],
        },
    )
    .await
    .unwrap();

    {
        let open_orders_account_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;

        assert_eq!(open_orders_account_0.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_0.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_0.position.base_free_native, 100);
        assert_eq!(open_orders_account_1.position.base_free_native, 0);
        assert_eq!(open_orders_account_0.position.quote_free_native, 0);
        assert_eq!(open_orders_account_1.position.quote_free_native, 99960);
    }

    let admin_token_1 = solana
        .create_associated_token_account(&collect_fee_admin.pubkey(), mints[1].pubkey)
        .await;

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
        let market = solana.get_account::<Market>(market).await;
        assert_eq!(market.quote_fees_accrued, 40);
        assert_eq!(market.fees_accrued, 60);
        assert_eq!(market.fees_to_referrers, 0);
    }

    send_tx(
        solana,
        SettleFundsInstruction {
            owner,
            market,
            open_orders_account: account_0,
            base_vault,
            quote_vault,
            token_base_account: owner_token_0,
            token_quote_account: owner_token_1,
            referrer: Some(owner_token_1),
        },
    )
    .await
    .unwrap();

    {
        let market = solana.get_account::<Market>(market).await;
        assert_eq!(market.quote_fees_accrued, 40);
        assert_eq!(market.fees_accrued, 60);
        assert_eq!(market.fees_to_referrers, 20);
    }

    send_tx(
        solana,
        SweepFeesInstruction {
            collect_fee_admin,
            market,
            quote_vault,
            token_receiver_account: admin_token_1,
        },
    )
    .await
    .unwrap();

    {
        let market = solana.get_account::<Market>(market).await;
        assert_eq!(market.quote_fees_accrued, 0);
    }

    Ok(())
}

#[tokio::test]
async fn test_no_maker_fees_ask() -> Result<(), TransportError> {
    let TestInitialize {
        context,
        collect_fee_admin,
        owner,
        mints,
        owner_token_0,
        owner_token_1,
        market,
        base_vault,
        quote_vault,
        price_lots,
        tokens,
        account_0,
        account_1,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize {
        maker_fee: -200,
        taker_fee: 400,
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
            owner,
            token_deposit_account: owner_token_0,
            base_vault,
            quote_vault,
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

        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 1);
        assert_eq!(open_orders_account_1.position.base_free_native, 0);
        assert_eq!(open_orders_account_1.position.quote_free_native, 0);
    }

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_0,
            open_orders_admin: None,
            market,
            owner,
            token_deposit_account: owner_token_1,
            base_vault,
            quote_vault,
            side: Side::Bid,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10004,
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
        let open_orders_account_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;

        assert_eq!(open_orders_account_0.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_0.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 1);
        assert_eq!(open_orders_account_0.position.base_free_native, 100);
        assert_eq!(open_orders_account_1.position.base_free_native, 0);
        assert_eq!(open_orders_account_0.position.quote_free_native, 0);
        assert_eq!(open_orders_account_1.position.quote_free_native, 0);
    }

    send_tx(
        solana,
        ConsumeEventsInstruction {
            consume_events_admin: None,
            market,
            open_orders_accounts: vec![account_0, account_1],
        },
    )
    .await
    .unwrap();

    {
        let open_orders_account_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;

        assert_eq!(open_orders_account_0.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_0.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_0.position.base_free_native, 100);
        assert_eq!(open_orders_account_1.position.base_free_native, 0);
        assert_eq!(open_orders_account_0.position.quote_free_native, 0);
        assert_eq!(open_orders_account_1.position.quote_free_native, 100020);
    }

    let admin_token_1 = solana
        .create_associated_token_account(&collect_fee_admin.pubkey(), mints[1].pubkey)
        .await;

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
        let market = solana.get_account::<Market>(market).await;
        assert_eq!(market.quote_fees_accrued, 0);
        assert_eq!(market.fees_accrued, 20);
        assert_eq!(market.fees_to_referrers, 0);
    }

    send_tx(
        solana,
        SettleFundsInstruction {
            owner,
            market,
            open_orders_account: account_0,
            base_vault,
            quote_vault,
            token_base_account: owner_token_0,
            token_quote_account: owner_token_1,
            referrer: Some(owner_token_1),
        },
    )
    .await
    .unwrap();

    {
        let market = solana.get_account::<Market>(market).await;
        assert_eq!(market.quote_fees_accrued, 0);
        assert_eq!(market.fees_accrued, 20);
        assert_eq!(market.fees_to_referrers, 20);
    }

    send_tx(
        solana,
        SweepFeesInstruction {
            collect_fee_admin,
            market,
            quote_vault,
            token_receiver_account: admin_token_1,
        },
    )
    .await
    .unwrap();

    {
        let market = solana.get_account::<Market>(market).await;
        assert_eq!(market.quote_fees_accrued, 0);
    }

    Ok(())
}

#[tokio::test]
async fn test_maker_fees_ask() -> Result<(), TransportError> {
    let TestInitialize {
        context,
        collect_fee_admin,
        owner,
        mints,
        owner_token_0,
        owner_token_1,
        market,
        base_vault,
        quote_vault,
        price_lots,
        tokens,
        account_0,
        account_1,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize {
        maker_fee: 200,
        taker_fee: 400,
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
            owner,
            token_deposit_account: owner_token_0,
            base_vault,
            quote_vault,
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

        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 1);
        assert_eq!(open_orders_account_1.position.base_free_native, 0);
        assert_eq!(open_orders_account_1.position.quote_free_native, 0);
    }

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_0,
            open_orders_admin: None,
            market,
            owner,
            token_deposit_account: owner_token_1,
            base_vault,
            quote_vault,
            side: Side::Bid,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10004,
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
        let open_orders_account_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;

        assert_eq!(open_orders_account_0.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_0.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 1);
        assert_eq!(open_orders_account_0.position.base_free_native, 100);
        assert_eq!(open_orders_account_1.position.base_free_native, 0);
        assert_eq!(open_orders_account_0.position.quote_free_native, 0);
        assert_eq!(open_orders_account_1.position.quote_free_native, 0);
    }

    send_tx(
        solana,
        ConsumeEventsInstruction {
            consume_events_admin: None,
            market,
            open_orders_accounts: vec![account_0, account_1],
        },
    )
    .await
    .unwrap();

    {
        let open_orders_account_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;

        assert_eq!(open_orders_account_0.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_0.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_0.position.base_free_native, 100);
        assert_eq!(open_orders_account_1.position.base_free_native, 0);
        assert_eq!(open_orders_account_0.position.quote_free_native, 0);
        assert_eq!(open_orders_account_1.position.quote_free_native, 99980);
    }

    let admin_token_1 = solana
        .create_associated_token_account(&collect_fee_admin.pubkey(), mints[1].pubkey)
        .await;

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
        let market = solana.get_account::<Market>(market).await;
        assert_eq!(market.quote_fees_accrued, 20);
        assert_eq!(market.fees_accrued, 60);
        assert_eq!(market.fees_to_referrers, 0);
    }

    send_tx(
        solana,
        SettleFundsInstruction {
            owner,
            market,
            open_orders_account: account_0,
            base_vault,
            quote_vault,
            token_base_account: owner_token_0,
            token_quote_account: owner_token_1,
            referrer: Some(owner_token_1),
        },
    )
    .await
    .unwrap();

    {
        let market = solana.get_account::<Market>(market).await;
        assert_eq!(market.quote_fees_accrued, 20);
        assert_eq!(market.fees_accrued, 60);
        assert_eq!(market.fees_to_referrers, 40);
    }

    send_tx(
        solana,
        SweepFeesInstruction {
            collect_fee_admin,
            market,
            quote_vault,
            token_receiver_account: admin_token_1,
        },
    )
    .await
    .unwrap();

    {
        let market = solana.get_account::<Market>(market).await;
        assert_eq!(market.quote_fees_accrued, 0);
    }

    Ok(())
}

#[tokio::test]
async fn test_fees_half() -> Result<(), TransportError> {
    let TestInitialize {
        context,
        collect_fee_admin,
        owner,
        mints,
        owner_token_0,
        owner_token_1,
        market,
        base_vault,
        quote_vault,
        price_lots,
        tokens,
        account_0,
        account_1,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize {
        maker_fee: -370,
        taker_fee: 740,
        ..TestNewMarketInitialize::default()
    })
    .await?;
    let solana = &context.solana.clone();

    // Set the initial oracle price
    set_stub_oracle_price(solana, &tokens[1], collect_fee_admin, 1000.0).await;
    let initial_quote_amount = 10000;

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_0,
            open_orders_admin: None,
            market,
            owner,
            token_deposit_account: owner_token_1,
            base_vault,
            quote_vault,
            side: Side::Bid,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: initial_quote_amount,
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
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            owner,
            token_deposit_account: owner_token_0,
            base_vault,
            quote_vault,
            side: Side::Ask,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: initial_quote_amount,
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
        ConsumeEventsInstruction {
            consume_events_admin: None,
            market,
            open_orders_accounts: vec![account_0, account_1],
        },
    )
    .await
    .unwrap();

    {
        let open_orders_account_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;
        let market = solana.get_account::<Market>(market).await;

        assert_eq!(open_orders_account_0.position.quote_free_native, 371);
        assert_eq!(open_orders_account_1.position.quote_free_native, 99259);
        assert_eq!(market.referrer_rebates_accrued, 370);

        assert_eq!(
            (market.referrer_rebates_accrued
                + open_orders_account_1.position.quote_free_native
                + open_orders_account_0.position.quote_free_native) as i64,
            initial_quote_amount * 10 // as it's in native quote
        );
    }

    let admin_token_1 = solana
        .create_associated_token_account(&collect_fee_admin.pubkey(), mints[1].pubkey)
        .await;

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

    send_tx(
        solana,
        SettleFundsInstruction {
            owner,
            market,
            open_orders_account: account_0,
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
        let balance_quote = solana.token_account_balance(quote_vault).await;
        assert_eq!(balance_quote, 370);
    }

    send_tx(
        solana,
        SweepFeesInstruction {
            collect_fee_admin,
            market,
            quote_vault,
            token_receiver_account: admin_token_1,
        },
    )
    .await
    .unwrap();

    {
        let balance_quote = solana.token_account_balance(quote_vault).await;
        assert_eq!(balance_quote, 0);
    }

    // Different fees

    let TestInitialize {
        context,
        collect_fee_admin,
        owner,
        mints,
        owner_token_0,
        owner_token_1,
        market,
        base_vault,
        quote_vault,
        price_lots,
        tokens,
        account_0,
        account_1,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize {
        maker_fee: -320,
        taker_fee: 640,
        ..TestNewMarketInitialize::default()
    })
    .await?;
    let solana = &context.solana.clone();

    // Set the initial oracle price
    set_stub_oracle_price(solana, &tokens[1], collect_fee_admin, 1000.0).await;
    let initial_quote_amount = 10000;

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_0,
            open_orders_admin: None,
            market,
            owner,
            token_deposit_account: owner_token_1,
            base_vault,
            quote_vault,
            side: Side::Bid,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: initial_quote_amount,
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
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            owner,
            token_deposit_account: owner_token_0,
            base_vault,
            quote_vault,
            side: Side::Ask,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: initial_quote_amount,
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
        ConsumeEventsInstruction {
            consume_events_admin: None,
            market,
            open_orders_accounts: vec![account_0, account_1],
        },
    )
    .await
    .unwrap();

    {
        let open_orders_account_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;
        let market = solana.get_account::<Market>(market).await;

        assert_eq!(open_orders_account_0.position.quote_free_native, 320);
        assert_eq!(open_orders_account_1.position.quote_free_native, 99360);
        assert_eq!(market.referrer_rebates_accrued, 320);

        assert_eq!(
            (market.referrer_rebates_accrued
                + open_orders_account_1.position.quote_free_native
                + open_orders_account_0.position.quote_free_native) as i64,
            initial_quote_amount * 10 // as it's in native quote
        );
    }

    let admin_token_1 = solana
        .create_associated_token_account(&collect_fee_admin.pubkey(), mints[1].pubkey)
        .await;

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

    send_tx(
        solana,
        SettleFundsInstruction {
            owner,
            market,
            open_orders_account: account_0,
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
        let balance_quote = solana.token_account_balance(quote_vault).await;
        assert_eq!(balance_quote, 320);
    }

    send_tx(
        solana,
        SweepFeesInstruction {
            collect_fee_admin,
            market,
            quote_vault,
            token_receiver_account: admin_token_1,
        },
    )
    .await
    .unwrap();

    {
        let balance_quote = solana.token_account_balance(quote_vault).await;
        assert_eq!(balance_quote, 0);
    }

    Ok(())
}
