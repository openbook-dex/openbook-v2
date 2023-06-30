use super::*;

async fn test_oracle_peg() -> Result<(), TransportError> {
    let market_base_lot_size = 10000;
    let market_quote_lot_size = 10;

    let TestInitialize {
        context,
        owner,
        owner_token_0,
        owner_token_1,
        market,
        base_vault,
        quote_vault,
        collect_fee_admin,
        account_0,
        account_1,
        tokens,
        bids,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize {
        quote_lot_size: market_quote_lot_size,
        base_lot_size: market_base_lot_size,
        maker_fee: -0,
        taker_fee: 0,
        ..TestNewMarketInitialize::default()
    })
    .await?;
    let solana = &context.solana.clone();

    let price_lots = {
        let market = solana.get_account::<Market>(market).await;
        market.native_price_to_lot(I80F48::ONE).unwrap()
    };
    assert_eq!(price_lots, market_base_lot_size / market_quote_lot_size);

    // TEST: Place and cancel order with order_id
    send_tx(
        solana,
        PlaceOrderPeggedInstruction {
            open_orders_account: account_0,
            market,
            owner,
            token_deposit_account: owner_token_1,
            market_vault: quote_vault,
            side: Side::Bid,
            price_offset: -1,
            peg_limit: 1,
            max_base_lots: 1,
            max_quote_lots_including_fees: 100_000,
            client_order_id: 0,
        },
    )
    .await
    .unwrap();

    let bids_data = solana.get_account_boxed::<BookSide>(bids).await;
    assert_eq!(bids_data.roots[1].leaf_count, 1);

    let order = solana
        .get_account::<OpenOrdersAccount>(account_0)
        .await
        .open_orders[0];
    assert_eq!(order.side_and_tree(), SideAndOrderTree::BidOraclePegged);

    send_tx(
        solana,
        CancelOrderInstruction {
            owner,
            market,
            open_orders_account: account_0,
            order_id: order.id,
        },
    )
    .await
    .unwrap();

    assert_no_orders(solana, account_0).await;

    let balance_before = solana.token_account_balance(owner_token_1).await;
    let max_quote_lots_including_fees = 100_000;

    // TEST: Place a pegged bid, take it with a direct and pegged ask, and consume events
    send_tx(
        solana,
        PlaceOrderPeggedInstruction {
            open_orders_account: account_0,
            market,
            owner,
            token_deposit_account: owner_token_1,
            market_vault: quote_vault,
            side: Side::Bid,
            price_offset: 0,
            peg_limit: price_lots,
            max_base_lots: 2,
            max_quote_lots_including_fees,
            client_order_id: 5,
        },
    )
    .await
    .unwrap();

    let balance_after = solana.token_account_balance(owner_token_1).await;

    // Max quantity being subtracted from owner is max_quote_lots_including_fees
    {
        assert!(
            balance_before
                - ((max_quote_lots_including_fees as u64) * (market_quote_lot_size as u64))
                <= balance_after
        );
    }

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            owner,
            token_deposit_account: owner_token_0,
            market_vault: base_vault,
            side: Side::Ask,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 100_000,
            client_order_id: 6,
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
        PlaceOrderPeggedInstruction {
            open_orders_account: account_1,
            market,
            owner,
            token_deposit_account: owner_token_0,
            market_vault: base_vault,
            side: Side::Ask,
            price_offset: 0,
            peg_limit: price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 100_000,
            client_order_id: 7,
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

    assert_no_orders(solana, account_0).await;

    // TEST: Place a pegged order and check how it behaves with oracle changes
    send_tx(
        solana,
        PlaceOrderPeggedInstruction {
            open_orders_account: account_0,
            market,
            owner,
            token_deposit_account: owner_token_1,
            market_vault: quote_vault,
            side: Side::Bid,
            price_offset: -1,
            peg_limit: 1,
            max_base_lots: 2,
            max_quote_lots_including_fees: 100_000,
            client_order_id: 5,
        },
    )
    .await
    .unwrap();

    // TEST: an ask at current oracle price does not match
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            owner,
            token_deposit_account: owner_token_0,
            market_vault: base_vault,
            side: Side::Ask,
            price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 100_000,

            client_order_id: 60,
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
            owner,
            client_order_id: 60,
        },
    )
    .await
    .unwrap();

    // TEST: Change the oracle, now the ask matches
    set_stub_oracle_price(solana, &tokens[0], collect_fee_admin, 1.002).await;
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            owner,
            token_deposit_account: owner_token_0,
            market_vault: base_vault,
            side: Side::Ask,
            price_lots,
            max_base_lots: 2,
            max_quote_lots_including_fees: 100_000,

            client_order_id: 61,
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
    assert_no_orders(solana, account_0).await;

    // restore the oracle to default
    set_stub_oracle_price(solana, &tokens[0], collect_fee_admin, 1.0).await;

    // TEST: order is cancelled when the price exceeds the peg limit
    send_tx(
        solana,
        PlaceOrderPeggedInstruction {
            open_orders_account: account_0,
            market,
            owner,
            token_deposit_account: owner_token_1,
            market_vault: quote_vault,
            side: Side::Bid,
            price_offset: -1,
            peg_limit: price_lots + 2,
            max_base_lots: 2,
            max_quote_lots_including_fees: 100_000,
            client_order_id: 5,
        },
    )
    .await
    .unwrap();

    // order is still matchable when exactly at the peg limit
    set_stub_oracle_price(solana, &tokens[0], collect_fee_admin, 1.003).await;
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            owner,
            token_deposit_account: owner_token_0,
            market_vault: base_vault,
            side: Side::Ask,
            price_lots: price_lots + 2,
            max_base_lots: 1,
            max_quote_lots_including_fees: 100_000,

            client_order_id: 62,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();
    assert!(send_tx(
        solana,
        CancelOrderByClientOrderIdInstruction {
            open_orders_account: account_1,
            market,
            owner,
            client_order_id: 62,
        },
    )
    .await
    .is_err());

    // but once the adjusted price is > the peg limit, it's gone
    set_stub_oracle_price(solana, &tokens[0], collect_fee_admin, 1.004).await;
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            owner,
            token_deposit_account: owner_token_0,
            market_vault: base_vault,
            side: Side::Ask,
            price_lots: price_lots + 3,
            max_base_lots: 1,
            max_quote_lots_including_fees: 100_000,

            client_order_id: 63,
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
            owner,
            client_order_id: 63,
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
    assert_no_orders(solana, account_0).await;

    Ok(())
}

async fn assert_no_orders(solana: &SolanaCookie, account_0: Pubkey) {
    let open_orders_account = solana.get_account::<OpenOrdersAccount>(account_0).await;

    for oo in open_orders_account.open_orders.iter() {
        assert!(oo.id == 0);
        assert!(oo.side_and_tree() == SideAndOrderTree::BidFixed);
        assert!(oo.client_id == 0);
    }
}

#[tokio::test]
async fn test_oracle_peg_limit() -> Result<(), TransportError> {
    let market_base_lot_size = 10000;
    let market_quote_lot_size = 10;

    let TestInitialize {
        context,
        owner,
        owner_token_1,
        market,
        quote_vault,
        account_0,
        bids,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize {
        quote_lot_size: market_quote_lot_size,
        base_lot_size: market_base_lot_size,
        maker_fee: -0,
        taker_fee: 0,
        ..TestNewMarketInitialize::default()
    })
    .await?;
    let solana = &context.solana.clone();

    let price_lots = {
        let market = solana.get_account::<Market>(market).await;
        market.native_price_to_lot(I80F48::ONE).unwrap()
    };
    assert_eq!(price_lots, market_base_lot_size / market_quote_lot_size);

    let balance_before = solana.token_account_balance(owner_token_1).await;
    let max_quote_lots_including_fees = 100_000;

    // TEST: Place a pegged bid, can't post in book due insufficient funds
    send_tx(
        solana,
        PlaceOrderPeggedInstruction {
            open_orders_account: account_0,
            market,
            owner,
            token_deposit_account: owner_token_1,
            market_vault: quote_vault,
            side: Side::Bid,
            price_offset: -100,
            peg_limit: price_lots + 100_000,
            max_base_lots: 2,
            max_quote_lots_including_fees,
            client_order_id: 5,
        },
    )
    .await
    .unwrap();
    assert_no_orders(solana, account_0).await;

    // Upgrade max quantity
    let max_quote_lots_including_fees = 101_000;

    send_tx(
        solana,
        PlaceOrderPeggedInstruction {
            open_orders_account: account_0,
            market,
            owner,
            token_deposit_account: owner_token_1,
            market_vault: quote_vault,
            side: Side::Bid,
            price_offset: -100,
            peg_limit: price_lots + 100_000,
            max_base_lots: 2,
            max_quote_lots_including_fees,
            client_order_id: 5,
        },
    )
    .await
    .unwrap();

    let bids_data = solana.get_account_boxed::<BookSide>(bids).await;
    assert_eq!(bids_data.roots[1].leaf_count, 1);

    let balance_after = solana.token_account_balance(owner_token_1).await;

    // Max quantity being subtracted from owner is max_quote_lots_including_fees
    {
        assert_eq!(
            balance_before
                - ((max_quote_lots_including_fees as u64) * (market_quote_lot_size as u64)),
            balance_after
        );
    }
    Ok(())
}

#[tokio::test]
async fn test_locked_amounts() -> Result<(), TransportError> {
    let quote_lot_size = 10;
    let base_lot_size = 100;
    let maker_fee = 200;
    let taker_fee = 400;

    let TestInitialize {
        context,
        owner,
        owner_token_0: owner_base_ata,
        owner_token_1: owner_quote_ata,
        market,
        base_vault,
        quote_vault,
        account_0,
        account_1,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize {
        quote_lot_size,
        base_lot_size,
        maker_fee,
        taker_fee,
        ..TestNewMarketInitialize::default()
    })
    .await?;
    let solana = &context.solana.clone();

    let place_bid_0_ix = PlaceOrderPeggedInstruction {
        open_orders_account: account_0,
        market,
        owner,
        token_deposit_account: owner_quote_ata,
        market_vault: quote_vault,
        side: Side::Bid,
        price_offset: 0,
        peg_limit: 30,
        max_base_lots: 1_000,
        max_quote_lots_including_fees: 100_000_000,
        client_order_id: 0,
    };

    let place_ask_1_ix = PlaceOrderPeggedInstruction {
        side: Side::Ask,
        market_vault: base_vault,
        token_deposit_account: owner_base_ata,
        open_orders_account: account_1,
        ..place_bid_0_ix.clone()
    };

    let settle_funds_0_ix = SettleFundsInstruction {
        owner,
        market,
        open_orders_account: account_0,
        base_vault,
        quote_vault,
        token_base_account: owner_base_ata,
        token_quote_account: owner_quote_ata,
        referrer: None,
    };

    let settle_funds_1_ix = SettleFundsInstruction {
        open_orders_account: account_1,
        ..settle_funds_0_ix.clone()
    };

    let consume_events_ix = ConsumeEventsInstruction {
        consume_events_admin: None,
        market,
        open_orders_accounts: vec![account_0, account_1],
    };

    let init_balances = (
        solana.token_account_balance(owner_base_ata).await,
        solana.token_account_balance(owner_quote_ata).await,
    );

    // Cancel bid order
    {
        send_tx(solana, place_bid_0_ix.clone()).await.unwrap();

        let balances = (
            solana.token_account_balance(owner_base_ata).await,
            solana.token_account_balance(owner_quote_ata).await + 300_000 + 60,
        );

        assert_eq!(init_balances, balances);

        send_tx(
            solana,
            CancelAllOrdersInstruction {
                open_orders_account: account_0,
                market,
                owner,
            },
        )
        .await
        .unwrap();
        send_tx(solana, settle_funds_0_ix.clone()).await.unwrap();

        let balances = (
            solana.token_account_balance(owner_base_ata).await,
            solana.token_account_balance(owner_quote_ata).await,
        );

        assert_eq!(init_balances, balances);
    }

    // Cancel ask order
    {
        send_tx(solana, place_ask_1_ix.clone()).await.unwrap();

        let balances = (
            solana.token_account_balance(owner_base_ata).await + 100_000,
            solana.token_account_balance(owner_quote_ata).await,
        );

        assert_eq!(init_balances, balances);

        send_tx(
            solana,
            CancelAllOrdersInstruction {
                open_orders_account: account_1,
                market,
                owner,
            },
        )
        .await
        .unwrap();
        send_tx(solana, settle_funds_1_ix.clone()).await.unwrap();

        let balances = (
            solana.token_account_balance(owner_base_ata).await,
            solana.token_account_balance(owner_quote_ata).await,
        );

        assert_eq!(init_balances, balances);
    }

    // Place & take a bid
    {
        send_tx(solana, place_bid_0_ix.clone()).await.unwrap();
        send_tx(solana, place_ask_1_ix.clone()).await.unwrap();
        send_tx(solana, consume_events_ix.clone()).await.unwrap();

        let (position_0, position_1) = {
            let oo_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;
            let oo_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;
            (oo_0.position, oo_1.position)
        };

        assert_eq!(position_0.quote_free_native, 200_000 + 40);
        assert_eq!(position_0.base_free_native, 100_000);

        assert_eq!(position_1.quote_free_native, 100_000 - 40);
        assert_eq!(position_1.base_free_native, 0);
    }

    Ok(())
}
