use super::*;

#[tokio::test]
async fn test_with_single_oracle() -> Result<(), TransportError> {
    let context = TestContextBuilder::new().start_default().await;
    let solana = &context.solana.clone();

    let payer = context.users[0].key;
    let mints = &context.mints[0..=2];
    let tokens = Token::create(mints.to_vec(), solana, payer, payer).await;

    let market_a = TestKeypair::new();
    let market_b = TestKeypair::new();

    assert!(send_tx(
        solana,
        CreateMarketInstruction {
            payer,
            market: market_a,
            quote_lot_size: 100,
            base_lot_size: 100,
            base_mint: mints[0].pubkey,
            quote_mint: mints[1].pubkey,
            ..CreateMarketInstruction::with_new_book_and_heap(solana, Some(tokens[0].oracle), None,)
                .await
        },
    )
    .await
    .is_ok());

    assert_eq!(
        send_tx_and_get_ix_custom_error(
            solana,
            CreateMarketInstruction {
                payer,
                market: market_b,
                quote_lot_size: 100,
                base_lot_size: 100,
                base_mint: mints[0].pubkey,
                quote_mint: mints[1].pubkey,
                ..CreateMarketInstruction::with_new_book_and_heap(
                    solana,
                    None,
                    Some(tokens[1].oracle)
                )
                .await
            },
        )
        .await,
        Some(openbook_v2::error::OpenBookError::InvalidSecondOracle.into())
    );

    Ok(())
}

#[tokio::test]
async fn test_with_same_oracles() -> Result<(), TransportError> {
    let context = TestContextBuilder::new().start_default().await;
    let solana = &context.solana.clone();

    let payer = context.users[0].key;
    let mints = &context.mints[0..=2];

    let market = TestKeypair::new();
    let fake_oracle_a = solana.create_account_from_len(&payer.pubkey(), 100).await;

    assert_eq!(
        send_tx_and_get_ix_custom_error(
            solana,
            CreateMarketInstruction {
                payer,
                market,
                quote_lot_size: 100,
                base_lot_size: 100,
                base_mint: mints[0].pubkey,
                quote_mint: mints[1].pubkey,
                ..CreateMarketInstruction::with_new_book_and_heap(
                    solana,
                    Some(fake_oracle_a),
                    Some(fake_oracle_a),
                )
                .await
            },
        )
        .await,
        Some(anchor_lang::error::ErrorCode::RequireKeysNeqViolated.into())
    );

    Ok(())
}

#[tokio::test]
async fn test_with_wrong_oracle_types() -> Result<(), TransportError> {
    let context = TestContextBuilder::new().start_default().await;
    let solana = &context.solana.clone();

    let payer = context.users[0].key;
    let mints = &context.mints[0..=2];

    let market_a = TestKeypair::new();
    let market_ab = TestKeypair::new();

    let fake_oracle_a = solana.create_account_from_len(&payer.pubkey(), 100).await;
    let fake_oracle_b = solana.create_account_from_len(&payer.pubkey(), 100).await;

    assert_eq!(
        send_tx_and_get_ix_custom_error(
            solana,
            CreateMarketInstruction {
                payer,
                market: market_a,
                quote_lot_size: 100,
                base_lot_size: 100,
                base_mint: mints[0].pubkey,
                quote_mint: mints[1].pubkey,
                ..CreateMarketInstruction::with_new_book_and_heap(solana, Some(fake_oracle_a), None)
                    .await
            },
        )
        .await,
        Some(openbook_v2::error::OpenBookError::UnknownOracleType.into())
    );

    assert_eq!(
        send_tx_and_get_ix_custom_error(
            solana,
            CreateMarketInstruction {
                payer,
                market: market_ab,
                quote_lot_size: 100,
                base_lot_size: 100,
                base_mint: mints[0].pubkey,
                quote_mint: mints[1].pubkey,
                ..CreateMarketInstruction::with_new_book_and_heap(
                    solana,
                    Some(fake_oracle_a),
                    Some(fake_oracle_b)
                )
                .await
            },
        )
        .await,
        Some(openbook_v2::error::OpenBookError::UnknownOracleType.into())
    );

    Ok(())
}
