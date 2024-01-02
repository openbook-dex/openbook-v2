use super::*;

#[tokio::test]
async fn test_fees_accrued() -> Result<(), TransportError> {
    let TestInitialize {
        context,
        collect_fee_admin,
        owner,
        mints,
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
        assert_eq!(open_orders_account_2.position.quote_free_native, 99980);
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
        assert_eq!(open_orders_account_2.position.quote_free_native, 99980);
    }

    let admin_token_1 = solana
        .create_associated_token_account(&collect_fee_admin.pubkey(), mints[1].pubkey)
        .await;

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

    {
        let market = solana.get_account::<Market>(market).await;
        assert_eq!(market.fees_available, 10);
        assert_eq!(market.fees_accrued, 10);
        assert_eq!(market.fees_to_referrers, 0);
    }

    send_tx(
        solana,
        SweepFeesInstruction {
            collect_fee_admin,
            market,
            market_quote_vault,
            token_receiver_account: admin_token_1,
        },
    )
    .await
    .unwrap();

    {
        let market = solana.get_account::<Market>(market).await;
        assert_eq!(market.fees_available, 0);
        assert_eq!(market.fees_accrued, 10);
        assert_eq!(market.fees_to_referrers, 0);
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

        market_base_vault,
        market_quote_vault,
        price_lots,
        tokens,
        account_1,
        account_2,
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
            signer: owner,
            user_token_account: owner_token_1,
            market_vault: market_quote_vault,
            side: Side::Bid,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10020,
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
            open_orders_account: account_1,
            market,
            signer: owner,
            client_order_id: 30,
        },
    )
    .await
    .unwrap();

    {
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;

        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.base_free_native, 0);
        assert_eq!(open_orders_account_1.position.quote_free_native, 100020);
    }

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
            max_quote_lots_including_fees: 10020,
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
        assert_eq!(open_orders_account_1.position.base_free_native, 0);
        assert_eq!(open_orders_account_1.position.quote_free_native, 0);
    }

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
        assert_eq!(open_orders_account_1.position.quote_free_native, 0);
        assert_eq!(open_orders_account_2.position.quote_free_native, 99960);
    }

    let admin_token_1 = solana
        .create_associated_token_account(&collect_fee_admin.pubkey(), mints[1].pubkey)
        .await;

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

    {
        let market = solana.get_account::<Market>(market).await;
        assert_eq!(market.fees_available, 40);
        assert_eq!(market.fees_accrued, 60);
        assert_eq!(market.fees_to_referrers, 0);
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
            referrer_account: Some(owner_token_1),
        },
    )
    .await
    .unwrap();

    {
        let market = solana.get_account::<Market>(market).await;
        assert_eq!(market.fees_available, 40);
        assert_eq!(market.fees_accrued, 60);
        assert_eq!(market.fees_to_referrers, 20);
    }

    send_tx(
        solana,
        SweepFeesInstruction {
            collect_fee_admin,
            market,
            market_quote_vault,
            token_receiver_account: admin_token_1,
        },
    )
    .await
    .unwrap();

    {
        let market = solana.get_account::<Market>(market).await;
        assert_eq!(market.fees_available, 0);
    }

    Ok(())
}

// Real simulation. Market maker pays fees but get them back on referral. Jupiter users don't pay fees.
// Only users paying fees are users using a UI limit orders.
#[tokio::test]
async fn test_market_maker_fees() -> Result<(), TransportError> {
    let market_base_lot_size = 100;
    let market_quote_lot_size = 10;
    let TestInitialize {
        context,
        owner,
        owner_token_0,
        owner_token_1,
        market,
        market_base_vault,
        market_quote_vault,
        price_lots,
        account_1,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize {
        quote_lot_size: market_quote_lot_size,
        base_lot_size: market_base_lot_size,
        maker_fee: 400,
        taker_fee: 400,
        ..TestNewMarketInitialize::default()
    })
    .await?;
    let solana = &context.solana.clone();

    let balance_owner_0_before = solana.token_account_balance(owner_token_0).await;
    let balance_owner_1_before = solana.token_account_balance(owner_token_1).await;

    // Account_1 is a MM and pays fee which will get back later
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
            max_quote_lots_including_fees: 10040,
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
        let balance_owner_0_after = solana.token_account_balance(owner_token_0).await;
        let balance_owner_1_after = solana.token_account_balance(owner_token_1).await;
        assert_eq!(balance_owner_0_before, balance_owner_0_after);
        assert_eq!(
            balance_owner_1_before,
            balance_owner_1_after + 10000 * (market_quote_lot_size as u64) + 40
        );
    }

    let jup_user = context.users[2].key;
    let jup_user_token_0 = context.users[2].token_accounts[0];
    let jup_user_token_1 = context.users[2].token_accounts[1];

    let balance_jup_0_before = solana.token_account_balance(jup_user_token_0).await;
    let balance_jup_1_before = solana.token_account_balance(jup_user_token_1).await;

    // Jupiter user, takes and don't pay fees
    send_tx(
        solana,
        PlaceTakeOrderInstruction {
            market,
            signer: jup_user,
            user_base_account: jup_user_token_0,
            user_quote_account: jup_user_token_1,
            market_base_vault,
            market_quote_vault,
            side: Side::Ask,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10040,
            open_orders_admin: None,
        },
    )
    .await
    .unwrap();

    {
        let balance_jup_0_after = solana.token_account_balance(jup_user_token_0).await;
        let balance_jup_1_after = solana.token_account_balance(jup_user_token_1).await;
        assert_eq!(
            balance_jup_0_before,
            balance_jup_0_after + (market_base_lot_size as u64)
        );
        assert_eq!(
            balance_jup_1_before + 10000 * (market_quote_lot_size as u64),
            balance_jup_1_after
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
            referrer_account: Some(owner_token_1),
        },
    )
    .await
    .unwrap();

    // MM gets back the fee
    {
        let balance_owner_0_after = solana.token_account_balance(owner_token_0).await;
        let balance_owner_1_after = solana.token_account_balance(owner_token_1).await;
        assert_eq!(
            balance_owner_0_before + (market_base_lot_size as u64),
            balance_owner_0_after
        );
        assert_eq!(
            balance_owner_1_before,
            balance_owner_1_after + 10000 * (market_quote_lot_size as u64)
        );
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

        market_base_vault,
        market_quote_vault,
        price_lots,
        tokens,
        account_1,
        account_2,
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

    {
        let open_orders_account_2 = solana.get_account::<OpenOrdersAccount>(account_2).await;

        assert_eq!(open_orders_account_2.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_2.position.asks_base_lots, 1);
        assert_eq!(open_orders_account_2.position.base_free_native, 0);
        assert_eq!(open_orders_account_2.position.quote_free_native, 0);
    }

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
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;
        let open_orders_account_2 = solana.get_account::<OpenOrdersAccount>(account_2).await;

        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_2.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_2.position.asks_base_lots, 1);
        assert_eq!(open_orders_account_1.position.base_free_native, 100);
        assert_eq!(open_orders_account_2.position.base_free_native, 0);
        assert_eq!(open_orders_account_1.position.quote_free_native, 0);
        assert_eq!(open_orders_account_2.position.quote_free_native, 0);
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
        assert_eq!(open_orders_account_1.position.quote_free_native, 0);
        assert_eq!(open_orders_account_2.position.quote_free_native, 100020);
    }

    let admin_token_1 = solana
        .create_associated_token_account(&collect_fee_admin.pubkey(), mints[1].pubkey)
        .await;

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

    {
        let market = solana.get_account::<Market>(market).await;
        assert_eq!(market.fees_available, 0);
        assert_eq!(market.fees_accrued, 20);
        assert_eq!(market.fees_to_referrers, 0);
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
            referrer_account: Some(owner_token_1),
        },
    )
    .await
    .unwrap();

    {
        let market = solana.get_account::<Market>(market).await;
        assert_eq!(market.fees_available, 0);
        assert_eq!(market.fees_accrued, 20);
        assert_eq!(market.fees_to_referrers, 20);
    }

    send_tx(
        solana,
        SweepFeesInstruction {
            collect_fee_admin,
            market,
            market_quote_vault,
            token_receiver_account: admin_token_1,
        },
    )
    .await
    .unwrap();

    {
        let market = solana.get_account::<Market>(market).await;
        assert_eq!(market.fees_available, 0);
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

        market_base_vault,
        market_quote_vault,
        price_lots,
        tokens,
        account_1,
        account_2,
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
            open_orders_account: account_2,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_token_0,
            market_vault: market_base_vault,
            side: Side::Ask,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10020,
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
        let open_orders_account_2 = solana.get_account::<OpenOrdersAccount>(account_2).await;

        assert_eq!(open_orders_account_2.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_2.position.asks_base_lots, 1);
        assert_eq!(open_orders_account_2.position.base_free_native, 0);
        assert_eq!(open_orders_account_2.position.quote_free_native, 0);
    }

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
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;
        let open_orders_account_2 = solana.get_account::<OpenOrdersAccount>(account_2).await;

        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_2.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_2.position.asks_base_lots, 1);
        assert_eq!(open_orders_account_1.position.base_free_native, 100);
        assert_eq!(open_orders_account_2.position.base_free_native, 0);
        assert_eq!(open_orders_account_1.position.quote_free_native, 0);
        assert_eq!(open_orders_account_2.position.quote_free_native, 0);
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
        assert_eq!(open_orders_account_1.position.quote_free_native, 0);
        assert_eq!(open_orders_account_2.position.quote_free_native, 99980);
    }

    let admin_token_1 = solana
        .create_associated_token_account(&collect_fee_admin.pubkey(), mints[1].pubkey)
        .await;

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

    {
        let market = solana.get_account::<Market>(market).await;
        assert_eq!(market.fees_available, 20);
        assert_eq!(market.fees_accrued, 60);
        assert_eq!(market.fees_to_referrers, 0);
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
            referrer_account: Some(owner_token_1),
        },
    )
    .await
    .unwrap();

    {
        let market = solana.get_account::<Market>(market).await;
        assert_eq!(market.fees_available, 20);
        assert_eq!(market.fees_accrued, 60);
        assert_eq!(market.fees_to_referrers, 40);
    }

    send_tx(
        solana,
        SweepFeesInstruction {
            collect_fee_admin,
            market,
            market_quote_vault,
            token_receiver_account: admin_token_1,
        },
    )
    .await
    .unwrap();

    {
        let market = solana.get_account::<Market>(market).await;
        assert_eq!(market.fees_available, 0);
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

        market_base_vault,
        market_quote_vault,
        price_lots,
        tokens,
        account_1,
        account_2,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize {
        maker_fee: -3700,
        taker_fee: 7400,
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
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_token_1,
            market_vault: market_quote_vault,
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
            open_orders_account: account_2,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_token_0,
            market_vault: market_base_vault,
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
            open_orders_accounts: vec![account_1, account_2],
        },
    )
    .await
    .unwrap();

    {
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;
        let open_orders_account_2 = solana.get_account::<OpenOrdersAccount>(account_2).await;
        let market = solana.get_account::<Market>(market).await;

        assert_eq!(open_orders_account_1.position.quote_free_native, 370);
        assert_eq!(open_orders_account_2.position.quote_free_native, 99260);
        assert_eq!(market.referrer_rebates_accrued, 370);

        assert_eq!(
            (market.referrer_rebates_accrued
                + open_orders_account_2.position.quote_free_native
                + open_orders_account_1.position.quote_free_native) as i64,
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
        let balance_quote = solana.token_account_balance(market_quote_vault).await;
        assert_eq!(balance_quote, 370);
    }

    send_tx(
        solana,
        SweepFeesInstruction {
            collect_fee_admin,
            market,
            market_quote_vault,
            token_receiver_account: admin_token_1,
        },
    )
    .await
    .unwrap();

    {
        let balance_quote = solana.token_account_balance(market_quote_vault).await;
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

        market_base_vault,
        market_quote_vault,
        price_lots,
        tokens,
        account_1,
        account_2,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize {
        maker_fee: -3200,
        taker_fee: 6400,
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
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_token_1,
            market_vault: market_quote_vault,
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
            open_orders_account: account_2,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_token_0,
            market_vault: market_base_vault,
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
            open_orders_accounts: vec![account_1, account_2],
        },
    )
    .await
    .unwrap();

    {
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;
        let open_orders_account_2 = solana.get_account::<OpenOrdersAccount>(account_2).await;
        let market = solana.get_account::<Market>(market).await;

        assert_eq!(open_orders_account_1.position.quote_free_native, 320);
        assert_eq!(open_orders_account_2.position.quote_free_native, 99360);
        assert_eq!(market.referrer_rebates_accrued, 320);

        assert_eq!(
            (market.referrer_rebates_accrued
                + open_orders_account_2.position.quote_free_native
                + open_orders_account_1.position.quote_free_native) as i64,
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
        let balance_quote = solana.token_account_balance(market_quote_vault).await;
        assert_eq!(balance_quote, 320);
    }

    send_tx(
        solana,
        SweepFeesInstruction {
            collect_fee_admin,
            market,
            market_quote_vault,
            token_receiver_account: admin_token_1,
        },
    )
    .await
    .unwrap();

    {
        let balance_quote = solana.token_account_balance(market_quote_vault).await;
        assert_eq!(balance_quote, 0);
    }

    Ok(())
}

#[tokio::test]
async fn test_locked_maker_fees() -> Result<(), TransportError> {
    let maker_fee = 350;
    let taker_fee = 0;

    let TestInitialize {
        context,
        owner,
        owner_token_0: owner_base_ata,
        owner_token_1: owner_quote_ata,
        market,

        market_base_vault,
        market_quote_vault,
        account_1: maker,
        account_2: taker,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize {
        maker_fee,
        taker_fee,
        ..TestNewMarketInitialize::default()
    })
    .await?;
    let solana = &context.solana.clone();

    let place_maker_bid = PlaceOrderInstruction {
        open_orders_account: maker,
        open_orders_admin: None,
        market,
        signer: owner,
        user_token_account: owner_quote_ata,
        market_vault: market_quote_vault,
        side: Side::Bid,
        price_lots: 1_000,
        max_base_lots: 5,
        max_quote_lots_including_fees: 1_000_000_000,
        client_order_id: 0,
        expiry_timestamp: 0,
        order_type: PlaceOrderType::Limit,
        self_trade_behavior: SelfTradeBehavior::default(),
        remainings: vec![],
    };

    let place_taker_ask = PlaceOrderInstruction {
        side: Side::Ask,
        market_vault: market_base_vault,
        open_orders_account: taker,
        user_token_account: owner_base_ata,
        max_base_lots: 3,
        ..place_maker_bid.clone()
    };

    let cancel_maker_orders_ix = CancelAllOrdersInstruction {
        open_orders_account: maker,
        signer: owner,
        market,
    };

    let settle_maker_funds_ix = SettleFundsInstruction {
        owner,
        market,
        open_orders_account: maker,
        market_base_vault,
        market_quote_vault,
        user_base_account: owner_base_ata,
        user_quote_account: owner_quote_ata,
        referrer_account: None,
    };

    send_tx(solana, place_maker_bid.clone()).await.unwrap();
    {
        let oo = solana.get_account::<OpenOrdersAccount>(maker).await;
        assert_eq!(oo.position.locked_maker_fees, 18);
    }

    send_tx(solana, place_taker_ask).await.unwrap();
    send_tx(
        solana,
        ConsumeEventsInstruction {
            consume_events_admin: None,
            market,
            open_orders_accounts: vec![maker],
        },
    )
    .await
    .unwrap();

    {
        let oo = solana.get_account::<OpenOrdersAccount>(maker).await;
        assert_eq!(oo.position.locked_maker_fees, 8);
    }

    send_tx(solana, cancel_maker_orders_ix.clone())
        .await
        .unwrap();

    // one lamport is still locked due rounding
    {
        let oo = solana.get_account::<OpenOrdersAccount>(maker).await;
        assert_eq!(oo.position.locked_maker_fees, 1);
    }

    send_tx(solana, place_maker_bid.clone()).await.unwrap();
    send_tx(solana, settle_maker_funds_ix.clone())
        .await
        .unwrap();

    // which cannot be claimed yet because there're still bids on the book
    {
        let oo = solana.get_account::<OpenOrdersAccount>(maker).await;
        assert_eq!(oo.position.locked_maker_fees, 1 + 18);
    }

    // but now if we don't have any pending bid order
    send_tx(solana, cancel_maker_orders_ix.clone())
        .await
        .unwrap();

    {
        let oo = solana.get_account::<OpenOrdersAccount>(maker).await;
        assert_eq!(oo.position.locked_maker_fees, 1);
    }

    // it's gone!
    send_tx(solana, settle_maker_funds_ix.clone())
        .await
        .unwrap();
    {
        let oo = solana.get_account::<OpenOrdersAccount>(maker).await;
        assert_eq!(oo.position.locked_maker_fees, 0);
    }

    Ok(())
}
