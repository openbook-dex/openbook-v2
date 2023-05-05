use super::*;

// TODO test quantities after fees implemented

#[tokio::test]
async fn test_oracle_peg() -> Result<(), TransportError> {
    let context = TestContext::new().await;
    let solana = &context.solana.clone();

    let admin = TestKeypair::new();
    let owner = context.users[0].key;
    let payer = context.users[1].key;
    let mints = &context.mints[0..2];

    let owner_token_0 = context.users[0].token_accounts[0];
    let owner_token_1 = context.users[0].token_accounts[1];
    let tokens = Token::create(mints.to_vec(), solana, admin, payer).await;

    // SETUP: Create a perp market
    let market = get_market_address_by_index(1);
    let base_vault = solana
        .create_associated_token_account(&market, mints[0].pubkey)
        .await;
    let quote_vault = solana
        .create_associated_token_account(&market, mints[1].pubkey)
        .await;

    let market_base_lot_size = 10000;
    let market_quote_lot_size = 10;

    let openbook_v2::accounts::CreateMarket { bids, .. } = send_tx(
        solana,
        CreateMarketInstruction {
            admin,
            payer,
            market_index: 1,
            base_lot_size: market_base_lot_size,
            quote_lot_size: market_quote_lot_size,
            maker_fee: 0.0,
            taker_fee: 0.0,
            base_mint: mints[0].pubkey,
            quote_mint: mints[1].pubkey,
            base_vault,
            quote_vault,
            ..CreateMarketInstruction::with_new_book_and_queue(solana, &tokens[0]).await
        },
    )
    .await
    .unwrap();

    let account_0 = create_funded_account(solana, owner, market, 0, &context.users[1]).await;
    let account_1 = create_funded_account(solana, owner, market, 1, &context.users[1]).await;

    let price_lots = {
        let market = solana.get_account::<Market>(market).await;
        market.native_price_to_lot(I80F48::ONE)
    };
    assert_eq!(price_lots, market_base_lot_size / market_quote_lot_size);


    // TEST: Place and cancel order with order_id
    send_tx(
        solana,
        PlaceOrderPeggedInstruction {
            open_orders_account: account_0,
            market,
            owner,
            payer: owner_token_1,
            base_vault,
            quote_vault,
            side: Side::Bid,
            price_offset: -1,
            peg_limit: -1,
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

    // TEST: Place a pegged bid, take it with a direct and pegged ask, and consume events
    send_tx(
        solana,
        PlaceOrderPeggedInstruction {
            open_orders_account: account_0,
            market,
            owner,
            payer: owner_token_1,
            base_vault,
            quote_vault,
            side: Side::Bid,
            price_offset: 0,
            peg_limit: -1,
            max_base_lots: 2,
            max_quote_lots_including_fees: 100_000,
            client_order_id: 5,
        },
    )
    .await
    .unwrap();

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
            max_quote_lots_including_fees: 100_000,
            reduce_only: false,
            client_order_id: 6,
            expiry_timestamp: 0,
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
            payer: owner_token_0,
            base_vault,
            quote_vault,
            side: Side::Ask,
            price_offset: 0,
            peg_limit: -1,
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
            market,
            open_orders_accounts: vec![account_0, account_1],
        },
    )
    .await
    .unwrap();

    {
        let open_orders_account_0 = solana.get_account::<OpenOrdersAccount>(account_0).await;
        assert_eq!(open_orders_account_0.position.base_position_lots(), 2);
        //assert!(assert_equal(
        //    open_orders_account_0.position.quote_position_native(),
        //    -19998.0,
        //    0.001
        //));

        let open_orders_account_1 = solana.get_account::<OpenOrdersAccount>(account_1).await;
        assert_eq!(open_orders_account_1.position.base_position_lots(), -2);
        //assert!(assert_equal(
        //    open_orders_account_1.position.quote_position_native(),
        //    19996.0,
        //    0.001
        //));
    }

    // TEST: Place a pegged order and check how it behaves with oracle changes
    send_tx(
        solana,
        PlaceOrderPeggedInstruction {
            open_orders_account: account_0,
            market,
            owner,
            payer: owner_token_1,
            base_vault,
            quote_vault,
            side: Side::Bid,
            price_offset: -1,
            peg_limit: -1,
            max_base_lots: 2,
            max_quote_lots_including_fees: 100_000,
            client_order_id: 5,
        },
    )
    .await
    .unwrap();

    panic!();

    // TEST: an ask at current oracle price does not match
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
            max_quote_lots_including_fees: 100_000,
            reduce_only: false,
            client_order_id: 60,
            expiry_timestamp: 0,
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
    set_stub_oracle_price(solana, &tokens[0], admin, 1.002).await;
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
            max_base_lots: 2,
            max_quote_lots_including_fees: 100_000,
            reduce_only: false,
            client_order_id: 61,
            expiry_timestamp: 0,
        },
    )
    .await
    .unwrap();
    send_tx(
        solana,
        ConsumeEventsInstruction {
            market,
            open_orders_accounts: vec![account_0, account_1],
        },
    )
    .await
    .unwrap();
    assert_no_orders(solana, account_0).await;

    // restore the oracle to default
    set_stub_oracle_price(solana, &tokens[0], admin, 1.0).await;

    // TEST: order is cancelled when the price exceeds the peg limit
    send_tx(
        solana,
        PlaceOrderPeggedInstruction {
            open_orders_account: account_0,
            market,
            owner,
            payer: owner_token_1,
            base_vault,
            quote_vault,
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
    set_stub_oracle_price(solana, &tokens[0], admin, 1.003).await;
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
            price_lots: price_lots + 2,
            max_base_lots: 1,
            max_quote_lots_including_fees: 100_000,
            reduce_only: false,
            client_order_id: 62,
            expiry_timestamp: 0,
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
    set_stub_oracle_price(solana, &tokens[0], admin, 1.004).await;
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
            price_lots: price_lots + 3,
            max_base_lots: 1,
            max_quote_lots_including_fees: 100_000,
            reduce_only: false,
            client_order_id: 63,
            expiry_timestamp: 0,
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
