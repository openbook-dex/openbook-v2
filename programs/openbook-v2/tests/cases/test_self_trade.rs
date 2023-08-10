use super::*;

#[tokio::test]
async fn test_self_trade_decrement_take() -> Result<(), TransportError> {
    let TestInitialize {
        context,
        owner,
        owner_token_0,
        owner_token_1,
        market,

        market_base_vault,
        market_quote_vault,
        account_1,
        account_2,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize::default()).await?;
    let solana = &context.solana.clone();
    let owner_quote_ata = context.users[0].token_accounts[1];
    let owner_base_ata = context.users[0].token_accounts[0];

    // maker (which will be the taker) limit order
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_base_ata,
            market_vault: market_base_vault,
            side: Side::Ask,
            price_lots: 1000,
            max_base_lots: 2,
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

    // maker limit order
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_2,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_base_ata,
            market_vault: market_base_vault,
            side: Side::Ask,
            price_lots: 1000,
            max_base_lots: 2,
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

    // taker full self-trade IOC
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_quote_ata,
            market_vault: market_quote_vault,
            side: Side::Bid,
            price_lots: 1000,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10000,
            client_order_id: 3,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::ImmediateOrCancel,
            self_trade_behavior: SelfTradeBehavior::DecrementTake,
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    {
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;
        let open_orders_account_2 = solana.get_account::<OpenOrdersAccount>(account_2).await;

        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 2);
        assert_eq!(open_orders_account_1.position.base_free_native, 100);
        assert_eq!(open_orders_account_1.position.quote_free_native, 0);

        assert_eq!(open_orders_account_2.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_2.position.asks_base_lots, 2);
        assert_eq!(open_orders_account_2.position.base_free_native, 0);
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

        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 2);
        assert_eq!(open_orders_account_1.position.base_free_native, 0);
        assert_eq!(open_orders_account_1.position.quote_free_native, 0);
    }

    // taker partial self-trade limit
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_quote_ata,
            market_vault: market_quote_vault,
            side: Side::Bid,
            price_lots: 1000,
            max_base_lots: 2,
            max_quote_lots_including_fees: 10002,
            client_order_id: 4,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::DecrementTake,
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    {
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;
        let open_orders_account_2 = solana.get_account::<OpenOrdersAccount>(account_2).await;

        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 2);
        assert_eq!(open_orders_account_1.position.base_free_native, 200);
        assert_eq!(open_orders_account_1.position.quote_free_native, 0);

        assert_eq!(open_orders_account_2.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_2.position.asks_base_lots, 2);
        assert_eq!(open_orders_account_2.position.base_free_native, 0);
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
        assert_eq!(open_orders_account_1.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.base_free_native, 200);
        assert_eq!(open_orders_account_1.position.quote_free_native, 20000);

        assert_eq!(open_orders_account_2.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_2.position.asks_base_lots, 1);
        assert_eq!(open_orders_account_2.position.base_free_native, 0);
        assert_eq!(open_orders_account_2.position.quote_free_native, 10002);
    }

    Ok(())
}

#[tokio::test]
async fn test_self_trade_cancel_provide() -> Result<(), TransportError> {
    let TestInitialize {
        context,
        owner,
        market,
        market_base_vault,
        market_quote_vault,
        account_1,
        account_2,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize::default()).await?;
    let solana = &context.solana.clone();
    let owner_quote_ata = context.users[0].token_accounts[1];
    let owner_base_ata = context.users[0].token_accounts[0];

    // maker (which will be the taker) limit order
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_base_ata,
            market_vault: market_base_vault,
            side: Side::Ask,
            price_lots: 1000,
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

    // maker limit order
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_2,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_base_ata,
            market_vault: market_base_vault,
            side: Side::Ask,
            price_lots: 1000,
            max_base_lots: 2,
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

    {
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;
        let open_orders_account_2 = solana.get_account::<OpenOrdersAccount>(account_2).await;

        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 1);
        assert_eq!(open_orders_account_1.position.base_free_native, 0);
        assert_eq!(open_orders_account_1.position.quote_free_native, 0);

        assert_eq!(open_orders_account_2.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_2.position.asks_base_lots, 2);
        assert_eq!(open_orders_account_2.position.base_free_native, 0);
        assert_eq!(open_orders_account_2.position.quote_free_native, 0);
    }

    // taker partial self-trade
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_quote_ata,
            market_vault: market_quote_vault,
            side: Side::Bid,
            price_lots: 1000,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10000,
            client_order_id: 3,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::ImmediateOrCancel,
            self_trade_behavior: SelfTradeBehavior::CancelProvide,
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    {
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;
        let open_orders_account_2 = solana.get_account::<OpenOrdersAccount>(account_2).await;

        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.base_free_native, 200);
        assert_eq!(open_orders_account_1.position.quote_free_native, 0);

        assert_eq!(open_orders_account_2.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_2.position.asks_base_lots, 2);
        assert_eq!(open_orders_account_2.position.base_free_native, 0);
        assert_eq!(open_orders_account_2.position.quote_free_native, 0);
    }

    // taker with another maker
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_quote_ata,
            market_vault: market_quote_vault,
            side: Side::Bid,
            price_lots: 1000,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10004,
            client_order_id: 4,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::DecrementTake,
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    {
        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;
        let open_orders_account_2 = solana.get_account::<OpenOrdersAccount>(account_2).await;

        assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_1.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.base_free_native, 300);
        assert_eq!(open_orders_account_1.position.quote_free_native, 0);

        assert_eq!(open_orders_account_2.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_2.position.asks_base_lots, 2);
        assert_eq!(open_orders_account_2.position.base_free_native, 0);
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
        assert_eq!(open_orders_account_1.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_1.position.base_free_native, 300);
        assert_eq!(open_orders_account_1.position.quote_free_native, 0);

        assert_eq!(open_orders_account_2.position.bids_base_lots, 0);
        assert_eq!(open_orders_account_2.position.asks_base_lots, 0);
        assert_eq!(open_orders_account_2.position.base_free_native, 0);
        assert_eq!(open_orders_account_2.position.quote_free_native, 20004);
    }

    Ok(())
}

#[tokio::test]
async fn test_self_abort_transaction() -> Result<(), TransportError> {
    let TestInitialize {
        context,
        owner,
        market,
        market_base_vault,
        market_quote_vault,
        account_1,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize::default()).await?;
    let solana = &context.solana.clone();
    let owner_quote_ata = context.users[0].token_accounts[1];
    let owner_base_ata = context.users[0].token_accounts[0];

    // taker limit order
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_base_ata,
            market_vault: market_base_vault,
            side: Side::Ask,
            price_lots: 1000,
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

    // taker failing self-trade
    assert!(send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            signer: owner,
            user_token_account: owner_quote_ata,
            market_vault: market_quote_vault,
            side: Side::Bid,
            price_lots: 1000,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10000,
            client_order_id: 2,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::AbortTransaction,
            remainings: vec![],
        },
    )
    .await
    .is_err());

    Ok(())
}

#[tokio::test]
async fn test_self_trade_no_fees() -> Result<(), TransportError> {
    let TestInitialize {
        context,
        owner,
        owner_token_0: owner_base_ata,
        owner_token_1: owner_quote_ata,
        market,

        market_base_vault,
        market_quote_vault,
        account_1: open_orders_account,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize::default()).await?;
    let solana = &context.solana.clone();

    let place_bid_ix = PlaceOrderInstruction {
        open_orders_account,
        open_orders_admin: None,
        market,
        signer: owner,
        user_token_account: owner_quote_ata,
        market_vault: market_quote_vault,
        side: Side::Bid,
        price_lots: 1000,
        max_base_lots: 1,
        max_quote_lots_including_fees: 10000,
        client_order_id: 1,
        expiry_timestamp: 0,
        order_type: PlaceOrderType::Limit,
        self_trade_behavior: SelfTradeBehavior::default(),
        remainings: vec![],
    };

    let place_ask_ix = PlaceOrderInstruction {
        side: Side::Ask,
        market_vault: market_base_vault,
        user_token_account: owner_base_ata,
        ..place_bid_ix.clone()
    };

    let consume_events_ix = ConsumeEventsInstruction {
        consume_events_admin: None,
        market,
        open_orders_accounts: vec![open_orders_account],
    };

    let settle_funds_ix = SettleFundsInstruction {
        owner,
        market,
        open_orders_account,
        market_base_vault,
        market_quote_vault,
        user_base_account: owner_base_ata,
        user_quote_account: owner_quote_ata,
        referrer_account: None,
    };

    let balances_before = (
        solana.token_account_balance(owner_base_ata).await,
        solana.token_account_balance(owner_quote_ata).await,
    );

    send_tx(solana, place_bid_ix.clone()).await.unwrap();
    send_tx(solana, place_ask_ix.clone()).await.unwrap();
    send_tx(solana, consume_events_ix.clone()).await.unwrap();
    send_tx(solana, settle_funds_ix.clone()).await.unwrap();

    let balances_after = (
        solana.token_account_balance(owner_base_ata).await,
        solana.token_account_balance(owner_quote_ata).await,
    );

    assert_eq!(balances_before, balances_after);

    send_tx(solana, place_ask_ix).await.unwrap();
    send_tx(solana, place_bid_ix).await.unwrap();
    send_tx(solana, consume_events_ix).await.unwrap();
    send_tx(solana, settle_funds_ix).await.unwrap();

    let balances_after = (
        solana.token_account_balance(owner_base_ata).await,
        solana.token_account_balance(owner_quote_ata).await,
    );

    assert_eq!(balances_before, balances_after);

    Ok(())
}
