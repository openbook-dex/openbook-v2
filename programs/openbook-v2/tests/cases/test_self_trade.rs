//#! just test test_self_trade
use super::*;

#[tokio::test]
async fn test_self_trade_decrement_take() -> Result<(), TransportError> {
    let context = TestContext::new().await;
    let solana = &context.solana.clone();

    let admin = TestKeypair::new();
    let payer = &context.users[0];
    let owner = context.users[1].key;
    let owner_base_ata = context.users[1].token_accounts[0];
    let owner_quote_ata = context.users[1].token_accounts[1];

    let mints = &context.mints[0..2];
    let tokens = Token::create(mints.to_vec(), solana, admin, payer.key).await;

    let base_mint = context.mints[0].pubkey;
    let quote_mint = context.mints[1].pubkey;

    // TEST: Create a market
    let market = get_market_address(admin.pubkey(), 1);
    let base_vault = solana
        .create_associated_token_account(&market, base_mint)
        .await;
    let quote_vault = solana
        .create_associated_token_account(&market, quote_mint)
        .await;

    let openbook_v2::accounts::CreateMarket {
        market,
        base_vault,
        quote_vault,
        ..
    } = send_tx(
        solana,
        CreateMarketInstruction {
            admin,
            payer: payer.key,
            market_index: 1,
            quote_lot_size: 10,
            base_lot_size: 100,
            maker_fee: -0.0002,
            taker_fee: 0.0004,
            base_mint,
            quote_mint,
            base_vault,
            quote_vault,
            ..CreateMarketInstruction::with_new_book_and_queue(solana, &tokens[1]).await
        },
    )
    .await
    .unwrap();

    let account_0 = create_open_orders_account(solana, owner, market, 0, &payer).await;
    let account_1 = create_open_orders_account(solana, owner, market, 1, &payer).await;

    println!("\n\nTAKER LIMIT ORDER\n\n");

    // taker limit order
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_0,
            market,
            owner,
            payer: owner_base_ata,
            base_vault,
            quote_vault,
            side: Side::Ask,
            price_lots: 1000,
            max_base_lots: 2,
            max_quote_lots_including_fees: 10000,
            client_order_id: 1,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
        },
    )
    .await
    .unwrap();

    // maker limit order
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_1,
            market,
            owner,
            payer: owner_base_ata,
            base_vault,
            quote_vault,
            side: Side::Ask,
            price_lots: 1000,
            max_base_lots: 2,
            max_quote_lots_including_fees: 10000,
            client_order_id: 2,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
        },
    )
    .await
    .unwrap();

    // taker full self-trade IOC
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_0,
            market,
            owner,
            payer: owner_quote_ata,
            base_vault,
            quote_vault,
            side: Side::Bid,
            price_lots: 1000,
            max_base_lots: 1,
            max_quote_lots_including_fees: 10000,
            client_order_id: 3,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::ImmediateOrCancel,
            self_trade_behavior: SelfTradeBehavior::DecrementTake,
        },
    )
    .await
    .unwrap();

    // taker partial self-trade limit
    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_account: account_0,
            market,
            owner,
            payer: owner_quote_ata,
            base_vault,
            quote_vault,
            side: Side::Bid,
            price_lots: 1000,
            max_base_lots: 2,
            max_quote_lots_including_fees: 10000,
            client_order_id: 4,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::DecrementTake,
        },
    )
    .await
    .unwrap();


    {
        //let open_orders_account_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;
        //let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;

        //assert_eq!(open_orders_account_0.position.bids_base_lots, 1);
        //assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
        //assert_eq!(open_orders_account_0.position.asks_base_lots, 0);
        //assert_eq!(open_orders_account_1.position.asks_base_lots, 0);
        //assert_eq!(open_orders_account_0.position.base_free_native, 0);
        //assert_eq!(open_orders_account_1.position.base_free_native, 0);
        //assert_eq!(open_orders_account_0.position.quote_free_native, 0);
        //assert_eq!(
        //    open_orders_account_1.position.quote_free_native.round(),
        //    99960
        //);
    }

    ////send_tx(
    ////    solana,
    ////    ConsumeEventsInstruction {
    ////        market,
    ////        open_orders_accounts: vec![account_0, account_1],
    ////    },
    ////)
    ////.await
    ////.unwrap();

    ////{
    ////    let open_orders_account_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;
    ////    let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;

    ////    assert_eq!(open_orders_account_0.position.bids_base_lots, 0);
    ////    assert_eq!(open_orders_account_1.position.bids_base_lots, 0);
    ////    assert_eq!(open_orders_account_0.position.asks_base_lots, 0);
    ////    assert_eq!(open_orders_account_1.position.asks_base_lots, 0);
    ////    assert_eq!(open_orders_account_0.position.base_free_native, 100);
    ////    assert_eq!(open_orders_account_1.position.base_free_native, 0);
    ////    assert_eq!(open_orders_account_0.position.quote_free_native.round(), 20);
    ////    assert_eq!(
    ////        open_orders_account_1.position.quote_free_native.round(),
    ////        99960
    ////    );
    ////}

    ////send_tx(
    ////    solana,
    ////    SettleFundsInstruction {
    ////        market,
    ////        open_orders_account: account_0,
    ////        base_vault,
    ////        quote_vault,
    ////        payer_base: owner_token_0,
    ////        payer_quote: owner_token_1,
    ////        referrer: None,
    ////    },
    ////)
    ////.await
    ////.unwrap();

    ////{
    ////    let open_orders_account_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;
    ////    let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;

    ////    assert_eq!(open_orders_account_0.position.base_free_native, 0);
    ////    assert_eq!(open_orders_account_1.position.base_free_native, 0);
    ////    assert_eq!(open_orders_account_0.position.quote_free_native, 0);
    ////    assert_eq!(
    ////        open_orders_account_1.position.quote_free_native.round(),
    ////        99960
    ////    );
    ////}

    ////send_tx(
    ////    solana,
    ////    SettleFundsInstruction {
    ////        market,
    ////        open_orders_account: account_1,
    ////        base_vault,
    ////        quote_vault,
    ////        payer_base: owner_token_0,
    ////        payer_quote: owner_token_1,
    ////        referrer: None,
    ////    },
    ////)
    ////.await
    ////.unwrap();

    ////{
    ////    let open_orders_account_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;
    ////    let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;

    ////    assert_eq!(open_orders_account_0.position.base_free_native, 0);
    ////    assert_eq!(open_orders_account_1.position.base_free_native, 0);
    ////    assert_eq!(open_orders_account_0.position.quote_free_native, 0);
    ////    assert_eq!(open_orders_account_1.position.quote_free_native, 0);
    ////}

    ////send_tx(
    ////    solana,
    ////    CloseMarketInstruction {
    ////        admin,
    ////        market,
    ////        sol_destination: owner.pubkey(),
    ////    },
    ////)
    ////.await
    ////.unwrap();

    Ok(())
}
