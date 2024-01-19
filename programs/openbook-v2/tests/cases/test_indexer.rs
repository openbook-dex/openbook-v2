use super::*;

#[tokio::test]
async fn test_indexer() -> Result<(), TransportError> {
    let context = TestContext::new().await;
    let solana = &context.solana.clone();

    let collect_fee_admin = TestKeypair::new();
    let close_market_admin = TestKeypair::new();
    let owner = context.users[0].key;
    let payer = context.users[1].key;
    let mints = &context.mints[0..=2];

    let tokens = Token::create(mints.to_vec(), solana, collect_fee_admin, payer).await;

    let market = TestKeypair::new();

    let openbook_v2::accounts::CreateMarket { market, .. } = send_tx(
        solana,
        CreateMarketInstruction {
            collect_fee_admin: collect_fee_admin.pubkey(),
            open_orders_admin: None,
            close_market_admin: Some(close_market_admin.pubkey()),
            payer,
            market,
            quote_lot_size: 10,
            base_lot_size: 100,
            maker_fee: -200,
            taker_fee: 400,
            base_mint: mints[0].pubkey,
            quote_mint: mints[1].pubkey,
            ..CreateMarketInstruction::with_new_book_and_heap(solana, Some(tokens[1].oracle), None)
                .await
        },
    )
    .await
    .unwrap();

    let indexer = create_open_orders_indexer(solana, &context.users[1], owner, market).await;
    let maker_1 =
        create_open_orders_account(solana, owner, market, 1, &context.users[1], None).await;

    {
        let indexer = solana.get_account::<OpenOrdersIndexer>(indexer).await;
        assert_eq!(indexer.created_counter, 1);
        assert!(indexer.addresses.contains(&maker_1));
    }

    let (maker_2, maker_3) = {
        (
            create_open_orders_account(solana, owner, market, 2, &context.users[1], None).await,
            create_open_orders_account(solana, owner, market, 3, &context.users[1], None).await,
        )
    };

    {
        let indexer = solana.get_account::<OpenOrdersIndexer>(indexer).await;

        assert_eq!(indexer.created_counter, 3);
        assert_eq!(indexer.addresses.len(), 3);
        assert!(indexer.addresses.contains(&maker_1));
        assert!(indexer.addresses.contains(&maker_2));
        assert!(indexer.addresses.contains(&maker_3));
    }

    send_tx(
        solana,
        CloseOpenOrdersAccountInstruction {
            account_num: 2,
            market,
            owner,
            sol_destination: owner.pubkey(),
        },
    )
    .await
    .unwrap();

    {
        let indexer = solana.get_account::<OpenOrdersIndexer>(indexer).await;

        assert_eq!(indexer.created_counter, 3);
        assert_eq!(indexer.addresses.len(), 2);
        assert!(indexer.addresses.contains(&maker_1));
        assert!(indexer.addresses.contains(&maker_3));
    }

    let maker_4 =
        create_open_orders_account(solana, owner, market, 4, &context.users[1], None).await;

    {
        let indexer = solana.get_account::<OpenOrdersIndexer>(indexer).await;
        assert_eq!(indexer.created_counter, 4);
        assert_eq!(indexer.addresses.len(), 3);
        assert!(indexer.addresses.contains(&maker_1));
        assert!(indexer.addresses.contains(&maker_3));
        assert!(indexer.addresses.contains(&maker_4));
    }

    Ok(())
}

#[tokio::test]
async fn test_size_vector() -> Result<(), TransportError> {
    let context = TestContext::new().await;
    let solana = &context.solana.clone();

    let collect_fee_admin = TestKeypair::new();
    let close_market_admin = TestKeypair::new();
    let owner = context.users[0].key;
    let payer = context.users[1].key;
    let mints = &context.mints[0..=2];

    let tokens = Token::create(mints.to_vec(), solana, collect_fee_admin, payer).await;

    let market = TestKeypair::new();

    let openbook_v2::accounts::CreateMarket { market, .. } = send_tx(
        solana,
        CreateMarketInstruction {
            collect_fee_admin: collect_fee_admin.pubkey(),
            open_orders_admin: None,
            close_market_admin: Some(close_market_admin.pubkey()),
            payer,
            market,
            quote_lot_size: 10,
            base_lot_size: 100,
            maker_fee: -200,
            taker_fee: 400,
            base_mint: mints[0].pubkey,
            quote_mint: mints[1].pubkey,
            ..CreateMarketInstruction::with_new_book_and_heap(solana, Some(tokens[1].oracle), None)
                .await
        },
    )
    .await
    .unwrap();

    let indexer = create_open_orders_indexer(solana, &context.users[1], owner, market).await;

    let mut makers = vec![];
    let max = 256;
    for n in 0..max {
        makers.push(
            create_open_orders_account(solana, owner, market, n + 1, &context.users[1], None).await,
        )
    }

    {
        let indexer = solana.get_account::<OpenOrdersIndexer>(indexer).await;

        assert_eq!(indexer.created_counter, max);
        assert_eq!(indexer.addresses.len(), max as usize);
        assert!(indexer.addresses.contains(&makers[(max - 1) as usize]));
        assert!(indexer.addresses.contains(&makers[(max / 2) as usize]));
        assert!(indexer.addresses.contains(&makers[1]));
    }

    // Can't create more than 256
    assert!(send_tx(
        solana,
        CreateOpenOrdersAccountInstruction {
            account_num: 257,
            market,
            owner,
            payer: context.users[1].key,
            delegate: None,
        },
    )
    .await
    .is_err());

    Ok(())
}
