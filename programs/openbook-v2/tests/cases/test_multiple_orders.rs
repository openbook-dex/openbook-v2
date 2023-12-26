use super::*;

#[tokio::test]
async fn insufficient_funds() -> Result<(), TransportError> {
    let TestInitialize {
        context,
        owner,
        owner_token_0,
        owner_token_1,
        account_1,
        account_2,
        market,
        market_base_vault,
        market_quote_vault,
        ..
    } = TestContext::new_with_market(TestNewMarketInitialize::default()).await?;

    let solana = &context.solana.clone();

    let max_quote_lots_including_fees = 104;

    // there's an ask on the book
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
            price_lots: 1,
            max_base_lots: i64::MAX / 1_000,
            max_quote_lots_including_fees,
            client_order_id: 0,
            expiry_timestamp: 0,
            order_type: PlaceOrderType::Limit,
            self_trade_behavior: SelfTradeBehavior::default(),
            remainings: vec![],
        },
    )
    .await
    .unwrap();

    solana.set_account_balance(owner_token_0, 2_500).await;
    solana.set_account_balance(owner_token_1, 110).await;

    // some lamports are already deposited
    send_tx(
        solana,
        DepositInstruction {
            owner,
            market,
            open_orders_account: account_1,
            market_base_vault,
            market_quote_vault,
            user_base_account: owner_token_0,
            user_quote_account: owner_token_1,
            base_amount: 1_200,
            quote_amount: 0,
        },
    )
    .await
    .unwrap();

    // note that a priori, we only have enough lamports to place 2.5 Ask. But as the bid will be
    // filled & the taker executed immediately, we will have 10 extra base lots available
    let order = openbook_v2::PlaceMultipleOrdersArgs {
        price_lots: 1,
        max_quote_lots_including_fees,
        expiry_timestamp: 0,
    };

    let bids = vec![order];
    let asks = vec![order; 4];

    send_tx(
        solana,
        CancelAllAndPlaceOrdersInstruction {
            open_orders_account: account_1,
            open_orders_admin: None,
            market,
            signer: owner,
            orders_type: PlaceOrderType::Limit,
            user_base_account: owner_token_0,
            user_quote_account: owner_token_1,
            bids,
            asks,
        },
    )
    .await
    .unwrap();

    let position = solana
        .get_account::<OpenOrdersAccount>(account_1)
        .await
        .position;

    assert_eq!(position.asks_base_lots, 35);
    assert_eq!(position.bids_base_lots, 0);

    assert_eq!(position.base_free_native, 0);
    assert_eq!(position.quote_free_native, 0);

    assert_eq!(position.referrer_rebates_available, 1);
    assert_eq!(solana.token_account_balance(owner_token_1).await, 9);
    assert_eq!(solana.token_account_balance(owner_token_0).await, 0);

    Ok(())
}
