use super::*;
use anchor_lang::InstructionData;
use anchor_spl::associated_token::get_associated_token_address;
use openbook_v2::{PlaceMultipleOrdersArgs, PlaceOrderPeggedArgs};
use solana_program::program_pack::Pack;

const TOKEN_ACCOUNT_LEN: usize = spl_token::state::Account::LEN;
const ANCHOR_CONSTRAINT_HAS_ONE: u32 = 2001;

fn janitor_builder() -> TestContextBuilder {
    let mut builder = TestContextBuilder::new();
    // settling with two ATA creations does not fit the default 130k, see
    // test_janitor_close_open_orders_account for the measured figure
    builder.test().set_compute_max_units(1_400_000);
    builder
}

async fn new_market(args: TestNewMarketInitialize) -> TestInitialize {
    TestContext::new_with_market_and_builder(janitor_builder(), args)
        .await
        .unwrap()
}

async fn sunset(solana: &SolanaCookie) {
    solana.set_clock_unix_timestamp(SUNSET_TS + 1).await;
    // the clock stays past the sunset as slots advance
    solana.advance_by_slots(3).await;
    assert!(solana.get_clock().await.unix_timestamp > SUNSET_TS);
}

fn assert_error<T>(result: Result<T, TransportError>, error: OpenBookError, comment: &str) {
    assert_openbook_error(&result, error.into(), comment.to_string());
}

fn indexer_address(owner: Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[b"OpenOrdersIndexer".as_ref(), owner.as_ref()],
        &openbook_v2::id(),
    )
    .0
}

/// Send `ix` with its instruction data replaced, for instructions without a client builder
async fn send_with_data<CI: ClientInstruction>(
    solana: &SolanaCookie,
    ix: CI,
    data: impl InstructionData,
) -> Result<(), TransportError> {
    let (_, mut instruction) = ix.to_instruction(solana).await;
    instruction.data = data.data();
    solana
        .process_transaction(&[instruction], Some(&ix.signers()))
        .await
        .map_err(TransportError::from)
}

/// A limit order at the fixture price, funded from the given token accounts
fn limit_order(
    init: &TestInitialize,
    open_orders_account: Pubkey,
    signer: TestKeypair,
    user_tokens: (Pubkey, Pubkey),
    side: Side,
    max_base_lots: i64,
) -> PlaceOrderInstruction {
    let (user_token_account, market_vault) = match side {
        Side::Bid => (user_tokens.1, init.market_quote_vault),
        Side::Ask => (user_tokens.0, init.market_base_vault),
    };
    PlaceOrderInstruction {
        open_orders_account,
        open_orders_admin: None,
        market: init.market,
        signer,
        user_token_account,
        market_vault,
        side,
        price_lots: init.price_lots,
        max_base_lots,
        max_quote_lots_including_fees: 20_000 * max_base_lots,
        client_order_id: 0,
        expiry_timestamp: 0,
        order_type: PlaceOrderType::Limit,
        self_trade_behavior: SelfTradeBehavior::default(),
        remainings: vec![],
    }
}

/// Indexer + open orders account for `owner`, paid by user 1
async fn new_open_orders_account(
    solana: &SolanaCookie,
    init: &TestInitialize,
    market: Pubkey,
    owner: TestKeypair,
    account_num: u32,
) -> Pubkey {
    if !solana.account_exists(indexer_address(owner.pubkey())).await {
        create_open_orders_indexer(solana, &init.context.users[1], owner, market).await;
    }
    create_open_orders_account(
        solana,
        owner,
        market,
        account_num,
        &init.context.users[1],
        None,
    )
    .await
}

/// An owner wallet that does not exist on chain, with an open orders account holding free
/// `base` and `quote` deposited from non associated token accounts
async fn owner_without_wallet(
    solana: &SolanaCookie,
    init: &TestInitialize,
    base: u64,
    quote: u64,
) -> (TestKeypair, Pubkey) {
    let owner = TestKeypair::new();
    let open_orders_account = new_open_orders_account(solana, init, init.market, owner, 1).await;
    let user_base_account = solana
        .create_token_account(&owner.pubkey(), init.mints[0].pubkey)
        .await;
    let user_quote_account = solana
        .create_token_account(&owner.pubkey(), init.mints[1].pubkey)
        .await;
    solana.set_account_balance(user_base_account, base).await;
    solana.set_account_balance(user_quote_account, quote).await;
    send_tx(
        solana,
        DepositInstruction {
            open_orders_account,
            market: init.market,
            market_base_vault: init.market_base_vault,
            market_quote_vault: init.market_quote_vault,
            user_base_account,
            user_quote_account,
            owner,
            base_amount: base,
            quote_amount: quote,
        },
    )
    .await
    .unwrap();
    assert_eq!(solana.get_lamports(owner.pubkey()).await, 0);
    (owner, open_orders_account)
}

/// owner's account_1 bids 2 lots, user 2's account asks 1 lot twice: two fill events, the
/// taker carries two heap penalties
async fn make_fills(
    solana: &SolanaCookie,
    init: &TestInitialize,
    open_orders_admin: Option<TestKeypair>,
) -> Pubkey {
    let taker = init.context.users[2].key;
    let taker_tokens = (
        init.context.users[2].token_accounts[0],
        init.context.users[2].token_accounts[1],
    );
    let taker_account = new_open_orders_account(solana, init, init.market, taker, 1).await;

    send_tx(
        solana,
        PlaceOrderInstruction {
            open_orders_admin,
            ..limit_order(
                init,
                init.account_1,
                init.owner,
                (init.owner_token_0, init.owner_token_1),
                Side::Bid,
                2,
            )
        },
    )
    .await
    .unwrap();
    for _ in 0..2 {
        send_tx(
            solana,
            PlaceOrderInstruction {
                open_orders_admin,
                ..limit_order(init, taker_account, taker, taker_tokens, Side::Ask, 1)
            },
        )
        .await
        .unwrap();
    }
    taker_account
}

async fn event_heap_len(solana: &SolanaCookie, market: Pubkey) -> usize {
    let market = solana.get_account::<Market>(market).await;
    solana
        .get_account_boxed::<EventHeap>(market.event_heap)
        .await
        .len()
}

fn close_ooa(
    janitor: TestKeypair,
    open_orders_account: Pubkey,
) -> JanitorCloseOpenOrdersAccountInstruction {
    JanitorCloseOpenOrdersAccountInstruction {
        janitor,
        open_orders_account,
        owner_base_ata: None,
        owner_quote_ata: None,
    }
}

fn cancel_all(
    janitor: TestKeypair,
    market: Pubkey,
    open_orders_account: Pubkey,
) -> CancelAllOrdersInstruction {
    CancelAllOrdersInstruction {
        open_orders_account,
        market,
        signer: janitor,
    }
}

#[tokio::test]
async fn test_sunset_create_market() -> Result<(), TransportError> {
    let init = new_market(TestNewMarketInitialize::default()).await;
    let solana = &init.context.solana.clone();
    sunset(solana).await;

    let result = send_tx(
        solana,
        CreateMarketInstruction {
            collect_fee_admin: init.collect_fee_admin.pubkey(),
            payer: init.payer,
            market: TestKeypair::new(),
            quote_lot_size: 10,
            base_lot_size: 100,
            base_mint: init.mints[0].pubkey,
            quote_mint: init.mints[1].pubkey,
            ..CreateMarketInstruction::with_new_book_and_heap(
                solana,
                Some(init.tokens[0].oracle),
                None,
            )
            .await
        },
    )
    .await;
    assert_error(result, OpenBookError::ProgramSunset, "create_market");

    let result = send_tx(
        solana,
        StubOracleCreate {
            mint: init.mints[0].pubkey,
            owner: TestKeypair::new(),
            payer: init.payer,
        },
    )
    .await;
    assert_error(result, OpenBookError::ProgramSunset, "stub_oracle_create");

    Ok(())
}

#[tokio::test]
async fn test_sunset_create_open_orders_indexer() -> Result<(), TransportError> {
    let init = new_market(TestNewMarketInitialize::default()).await;
    let solana = &init.context.solana.clone();
    sunset(solana).await;

    let result = send_tx(
        solana,
        CreateOpenOrdersIndexerInstruction {
            market: init.market,
            owner: init.context.users[2].key,
            payer: init.payer,
        },
    )
    .await;
    assert_error(
        result,
        OpenBookError::ProgramSunset,
        "create_open_orders_indexer",
    );

    Ok(())
}

#[tokio::test]
async fn test_sunset_create_open_orders_account() -> Result<(), TransportError> {
    let init = new_market(TestNewMarketInitialize::default()).await;
    let solana = &init.context.solana.clone();
    sunset(solana).await;

    // the owner's indexer exists from before the sunset
    let result = send_tx(
        solana,
        CreateOpenOrdersAccountInstruction {
            account_num: 3,
            market: init.market,
            owner: init.owner,
            payer: init.payer,
            delegate: None,
        },
    )
    .await;
    assert_error(
        result,
        OpenBookError::ProgramSunset,
        "create_open_orders_account",
    );

    Ok(())
}

#[tokio::test]
async fn test_sunset_place_order() -> Result<(), TransportError> {
    let init = new_market(TestNewMarketInitialize::default()).await;
    let solana = &init.context.solana.clone();
    let owner_tokens = (init.owner_token_0, init.owner_token_1);

    // resting bid from before the sunset
    send_tx(
        solana,
        limit_order(
            &init,
            init.account_1,
            init.owner,
            owner_tokens,
            Side::Bid,
            1,
        ),
    )
    .await
    .unwrap();

    sunset(solana).await;

    let result = send_tx(
        solana,
        limit_order(
            &init,
            init.account_1,
            init.owner,
            owner_tokens,
            Side::Bid,
            1,
        ),
    )
    .await;
    assert_error(result, OpenBookError::ProgramSunset, "place_order");

    let result = send_tx(
        solana,
        PlaceOrderPeggedInstruction {
            open_orders_account: init.account_1,
            market: init.market,
            signer: init.owner,
            user_token_account: init.owner_token_1,
            market_vault: init.market_quote_vault,
            side: Side::Bid,
            price_offset: -1,
            max_base_lots: 1,
            max_quote_lots_including_fees: 20_000,
            client_order_id: 0,
            peg_limit: 100_000,
        },
    )
    .await;
    assert_error(result, OpenBookError::ProgramSunset, "place_order_pegged");

    let result = send_tx(
        solana,
        PlaceTakeOrderInstruction {
            open_orders_admin: None,
            market: init.market,
            signer: init.owner,
            market_base_vault: init.market_base_vault,
            market_quote_vault: init.market_quote_vault,
            user_base_account: init.owner_token_0,
            user_quote_account: init.owner_token_1,
            side: Side::Ask,
            price_lots: init.price_lots,
            max_base_lots: 1,
            max_quote_lots_including_fees: 20_000,
        },
    )
    .await;
    assert_error(result, OpenBookError::ProgramSunset, "place_take_order");

    let place_multiple = || CancelAllAndPlaceOrdersInstruction {
        open_orders_account: init.account_1,
        open_orders_admin: None,
        market: init.market,
        signer: init.owner,
        user_base_account: init.owner_token_0,
        user_quote_account: init.owner_token_1,
        orders_type: PlaceOrderType::Limit,
        bids: vec![PlaceMultipleOrdersArgs {
            price_lots: init.price_lots,
            max_quote_lots_including_fees: 20_000,
            expiry_timestamp: 0,
        }],
        asks: vec![],
    };
    let result = send_tx(solana, place_multiple()).await;
    assert_error(
        result,
        OpenBookError::ProgramSunset,
        "cancel_all_and_place_orders",
    );
    let result = send_with_data(
        solana,
        place_multiple(),
        openbook_v2::instruction::PlaceOrders {
            orders_type: PlaceOrderType::Limit,
            bids: place_multiple().bids,
            asks: vec![],
            limit: 10,
        },
    )
    .await;
    assert_error(result, OpenBookError::ProgramSunset, "place_orders");

    let edit_order = || EditOrderInstruction {
        open_orders_account: init.account_1,
        open_orders_admin: None,
        market: init.market,
        signer: init.owner,
        market_vault: init.market_quote_vault,
        user_token_account: init.owner_token_1,
        side: Side::Bid,
        price_lots: init.price_lots,
        max_base_lots: 1,
        max_quote_lots_including_fees: 20_000,
        client_order_id: 0,
        expiry_timestamp: 0,
        order_type: PlaceOrderType::Limit,
        self_trade_behavior: SelfTradeBehavior::default(),
        remainings: vec![],
        expected_cancel_size: 1,
    };
    let result = send_tx(solana, edit_order()).await;
    assert_error(result, OpenBookError::ProgramSunset, "edit_order");
    let result = send_with_data(
        solana,
        edit_order(),
        openbook_v2::instruction::EditOrderPegged {
            client_order_id: 0,
            expected_cancel_size: 1,
            place_order: PlaceOrderPeggedArgs {
                side: Side::Bid,
                price_offset_lots: -1,
                peg_limit: 100_000,
                max_base_lots: 1,
                max_quote_lots_including_fees: 20_000,
                client_order_id: 0,
                order_type: PlaceOrderType::Limit,
                expiry_timestamp: 0,
                self_trade_behavior: SelfTradeBehavior::default(),
                limit: 10,
            },
        },
    )
    .await;
    assert_error(result, OpenBookError::ProgramSunset, "edit_order_pegged");

    let deposit = || DepositInstruction {
        open_orders_account: init.account_1,
        market: init.market,
        market_base_vault: init.market_base_vault,
        market_quote_vault: init.market_quote_vault,
        user_base_account: init.owner_token_0,
        user_quote_account: init.owner_token_1,
        owner: init.owner,
        base_amount: 100,
        quote_amount: 100,
    };
    let result = send_tx(solana, deposit()).await;
    assert_error(result, OpenBookError::ProgramSunset, "deposit");
    let result = send_with_data(
        solana,
        deposit(),
        openbook_v2::instruction::Refill {
            base_amount: 100,
            quote_amount: 100,
        },
    )
    .await;
    assert_error(result, OpenBookError::ProgramSunset, "refill");

    // every exit path stays open
    send_tx(
        solana,
        CancelAllOrdersInstruction {
            open_orders_account: init.account_1,
            market: init.market,
            signer: init.owner,
        },
    )
    .await
    .unwrap();
    send_tx(
        solana,
        ConsumeEventsInstruction {
            consume_events_admin: None,
            market: init.market,
            open_orders_accounts: vec![],
        },
    )
    .await
    .unwrap();
    send_tx(
        solana,
        SettleFundsInstruction {
            owner: init.owner,
            open_orders_account: init.account_1,
            market: init.market,
            market_base_vault: init.market_base_vault,
            market_quote_vault: init.market_quote_vault,
            user_base_account: init.owner_token_0,
            user_quote_account: init.owner_token_1,
            referrer_account: None,
        },
    )
    .await
    .unwrap();
    for account_num in [1, 2] {
        send_tx(
            solana,
            CloseOpenOrdersAccountInstruction {
                account_num,
                market: init.market,
                owner: init.owner,
                sol_destination: init.owner.pubkey(),
            },
        )
        .await
        .unwrap();
    }
    assert!(!solana.account_exists(init.account_1).await);

    Ok(())
}

async fn test_janitor_crank(permissioned: bool) {
    let init = new_market(TestNewMarketInitialize {
        consume_events_admin_bool: permissioned,
        ..TestNewMarketInitialize::default()
    })
    .await;
    let solana = &init.context.solana.clone();
    let janitor = janitor_keypair();

    let taker_account = make_fills(solana, &init, None).await;
    assert_eq!(event_heap_len(solana, init.market).await, 2);

    sunset(solana).await;

    let accounts = vec![init.account_1, taker_account];
    let crank = |consume_events_admin| ConsumeEventsInstruction {
        consume_events_admin,
        market: init.market,
        open_orders_accounts: accounts.clone(),
    };

    let result = send_tx(solana, crank(Some(TestKeypair::new()))).await;
    assert_error(
        result,
        OpenBookError::InvalidConsumeEventsAdmin,
        "random signer",
    );

    // the regular cranker consumes the first event: the admin, or anyone without one
    let market = solana.get_account::<Market>(init.market).await;
    let slot = solana
        .get_account_boxed::<EventHeap>(market.event_heap)
        .await
        .iter()
        .next()
        .unwrap()
        .1;
    send_tx(
        solana,
        ConsumeGivenEventsInstruction {
            consume_events_admin: permissioned.then_some(init.consume_events_admin),
            market: init.market,
            open_orders_accounts: accounts.clone(),
            slots: vec![slot],
        },
    )
    .await
    .unwrap();
    assert_eq!(event_heap_len(solana, init.market).await, 1);

    send_tx(solana, crank(Some(janitor))).await.unwrap();
    assert_eq!(event_heap_len(solana, init.market).await, 0);

    let maker = solana
        .get_account::<OpenOrdersAccount>(init.account_1)
        .await;
    assert_eq!(maker.position.bids_base_lots, 0);
    assert_eq!(maker.position.base_free_native, 200);
}

#[tokio::test]
async fn test_janitor_crank_permissioned() -> Result<(), TransportError> {
    test_janitor_crank(true).await;
    Ok(())
}

#[tokio::test]
async fn test_janitor_crank_permissionless() -> Result<(), TransportError> {
    test_janitor_crank(false).await;
    Ok(())
}

async fn mint_supply(solana: &SolanaCookie, mint: Pubkey) -> u64 {
    let data = solana.get_account_data(mint).await.unwrap();
    spl_token::state::Mint::unpack(&data).unwrap().supply
}

#[tokio::test]
async fn test_janitor_crank_writes_off_fills_of_closed_account() -> Result<(), TransportError> {
    let init = new_market(TestNewMarketInitialize::default()).await;
    let solana = &init.context.solana.clone();
    let janitor = janitor_keypair();

    // two fills for a maker account that no longer exists (nothing in the program closes an
    // account with events in flight, the test removes it)
    let taker_account = make_fills(solana, &init, None).await;
    solana.remove_account(init.account_1).await;
    assert_eq!(event_heap_len(solana, init.market).await, 2);
    let before = solana.get_account::<Market>(init.market).await;

    let crank = |consume_events_admin, open_orders_accounts| ConsumeEventsInstruction {
        consume_events_admin,
        market: init.market,
        open_orders_accounts,
    };

    // before the sunset the events stay, nobody can write them off: a regular crank fails to
    // load the closed account
    let result = send_tx(solana, crank(None, vec![init.account_1, taker_account])).await;
    assert!(result.is_err());
    assert_eq!(event_heap_len(solana, init.market).await, 2);
    let result = send_tx(solana, crank(Some(janitor), vec![init.account_1])).await;
    assert_error(
        result,
        OpenBookError::InvalidConsumeEventsAdmin,
        "janitor before the sunset",
    );

    sunset(solana).await;

    // only the janitor, and only when the closed account is passed
    let result = send_tx(solana, crank(None, vec![init.account_1])).await;
    assert!(result.is_err());
    send_tx(solana, crank(Some(janitor), vec![taker_account]))
        .await
        .unwrap();
    assert_eq!(event_heap_len(solana, init.market).await, 2);

    send_tx(solana, crank(Some(janitor), vec![init.account_1]))
        .await
        .unwrap();
    assert_eq!(event_heap_len(solana, init.market).await, 0);

    // the maker's bid of 2 lots was filled twice: the base it bought and its rebates have no
    // claimant, they left the deposit totals; the negative maker fee leaves nothing for the fee
    // admin
    let market = solana.get_account::<Market>(init.market).await;
    let quote_native = (init.price_lots * market.quote_lot_size) as u64;
    let base_native = 2 * market.base_lot_size as u64;
    let rebate_native = 2 * market.maker_rebate_floor(quote_native);
    assert!(rebate_native > 0);
    assert_eq!(market.orphaned_events, 2);
    assert_eq!(market.orphaned_base_native, base_native);
    assert_eq!(market.orphaned_quote_native, rebate_native);
    assert_eq!(
        market.base_deposit_total,
        before.base_deposit_total - base_native
    );
    assert_eq!(
        market.quote_deposit_total,
        before.quote_deposit_total - rebate_native
    );
    assert_eq!(market.fees_available, before.fees_available);

    // the rest of the wind-down: the taker's account and the fees, then the market burns the
    // written off amounts and nothing is left for the escrow
    send_tx(solana, close_ooa(janitor, taker_account))
        .await
        .unwrap();
    let fee_admin_ata = solana
        .create_associated_token_account(&init.collect_fee_admin.pubkey(), init.mints[1].pubkey)
        .await;
    send_tx(
        solana,
        SweepFeesInstruction {
            collect_fee_admin: janitor,
            market: init.market,
            market_quote_vault: init.market_quote_vault,
            token_receiver_account: fee_admin_ata,
        },
    )
    .await
    .unwrap();
    let (base_supply, quote_supply) = (1_000_000_000_000, 2_000_000_000_000);
    solana
        .set_mint_supply(init.mints[0].pubkey, base_supply)
        .await;
    solana
        .set_mint_supply(init.mints[1].pubkey, quote_supply)
        .await;
    send_tx(
        solana,
        JanitorCloseMarketInstruction {
            janitor,
            market: init.market,
        },
    )
    .await
    .unwrap();
    assert!(!solana.account_exists(init.market).await);
    assert_eq!(
        mint_supply(solana, init.mints[0].pubkey).await,
        base_supply - base_native
    );
    assert_eq!(
        mint_supply(solana, init.mints[1].pubkey).await,
        quote_supply - rebate_native
    );

    Ok(())
}

#[tokio::test]
async fn test_janitor_crank_writes_off_out_event_of_closed_account() -> Result<(), TransportError> {
    let init = new_market(TestNewMarketInitialize::default()).await;
    let solana = &init.context.solana.clone();
    let janitor = janitor_keypair();
    let taker = init.context.users[2].key;
    let taker_tokens = (
        init.context.users[2].token_accounts[0],
        init.context.users[2].token_accounts[1],
    );
    let taker_account = new_open_orders_account(solana, &init, init.market, taker, 1).await;

    // a bid that expires before an ask would cross it: the matching engine removes it from the
    // book with an out event for its owner, which only the owner's account can consume
    let now = solana.get_clock().await.unix_timestamp;
    send_tx(
        solana,
        PlaceOrderInstruction {
            expiry_timestamp: (now + 100) as u64,
            ..limit_order(
                &init,
                init.account_1,
                init.owner,
                (init.owner_token_0, init.owner_token_1),
                Side::Bid,
                2,
            )
        },
    )
    .await
    .unwrap();
    let before = solana.get_account::<Market>(init.market).await;
    solana.advance_clock(200).await;
    send_tx(
        solana,
        limit_order(&init, taker_account, taker, taker_tokens, Side::Ask, 1),
    )
    .await
    .unwrap();
    assert_eq!(event_heap_len(solana, init.market).await, 1);

    solana.remove_account(init.account_1).await;
    sunset(solana).await;
    send_tx(
        solana,
        ConsumeEventsInstruction {
            consume_events_admin: Some(janitor),
            market: init.market,
            open_orders_accounts: vec![init.account_1],
        },
    )
    .await
    .unwrap();
    assert_eq!(event_heap_len(solana, init.market).await, 0);

    // the quote the bid had locked (no maker fees at a negative maker fee) was written off
    let market = solana.get_account::<Market>(init.market).await;
    let quote_native = (2 * init.price_lots * market.quote_lot_size) as u64;
    assert_eq!(market.orphaned_events, 1);
    assert_eq!(market.orphaned_base_native, 0);
    assert_eq!(market.orphaned_quote_native, quote_native);
    assert_eq!(
        market.quote_deposit_total,
        before.quote_deposit_total - quote_native
    );

    // the taker's resting ask, then the market burns the quote
    send_tx(solana, cancel_all(janitor, init.market, taker_account))
        .await
        .unwrap();
    send_tx(solana, close_ooa(janitor, taker_account))
        .await
        .unwrap();
    let quote_supply = 2_000_000_000_000;
    solana
        .set_mint_supply(init.mints[1].pubkey, quote_supply)
        .await;
    send_tx(
        solana,
        JanitorCloseMarketInstruction {
            janitor,
            market: init.market,
        },
    )
    .await
    .unwrap();
    assert!(!solana.account_exists(init.market).await);
    assert_eq!(
        mint_supply(solana, init.mints[1].pubkey).await,
        quote_supply - quote_native
    );

    Ok(())
}

#[tokio::test]
async fn test_janitor_cancel_all_orders() -> Result<(), TransportError> {
    let init = new_market(TestNewMarketInitialize {
        open_orders_admin_bool: true,
        ..TestNewMarketInitialize::default()
    })
    .await;
    let solana = &init.context.solana.clone();
    let janitor = janitor_keypair();

    let second_owner = init.context.users[2].key;
    let second_account = new_open_orders_account(solana, &init, init.market, second_owner, 1).await;
    let orders = [
        (init.account_1, init.owner, init.owner_token_1),
        (
            second_account,
            second_owner,
            init.context.users[2].token_accounts[1],
        ),
    ];
    for (account, owner, quote_account) in orders {
        send_tx(
            solana,
            PlaceOrderInstruction {
                open_orders_admin: Some(init.open_orders_admin),
                ..limit_order(
                    &init,
                    account,
                    owner,
                    (Pubkey::default(), quote_account),
                    Side::Bid,
                    1,
                )
            },
        )
        .await
        .unwrap();
    }
    let quote_free_before = solana
        .get_account::<OpenOrdersAccount>(init.account_1)
        .await
        .position
        .quote_free_native;

    sunset(solana).await;

    let result = send_tx(
        solana,
        cancel_all(TestKeypair::new(), init.market, init.account_1),
    )
    .await;
    assert_error(result, OpenBookError::NoOwnerOrDelegate, "random signer");

    for (account, ..) in orders {
        send_tx(solana, cancel_all(janitor, init.market, account))
            .await
            .unwrap();
        let position = solana
            .get_account::<OpenOrdersAccount>(account)
            .await
            .position;
        assert_eq!(position.bids_base_lots, 0);
        assert!(position.quote_free_native > quote_free_before);
    }
    let bids = solana.get_account_boxed::<BookSide>(init.bids).await;
    assert!(bids.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_janitor_close_open_orders_account_fails_with_open_orders(
) -> Result<(), TransportError> {
    let init = new_market(TestNewMarketInitialize::default()).await;
    let solana = &init.context.solana.clone();
    let janitor = janitor_keypair();

    send_tx(
        solana,
        limit_order(
            &init,
            init.account_1,
            init.owner,
            (init.owner_token_0, init.owner_token_1),
            Side::Ask,
            1,
        ),
    )
    .await
    .unwrap();

    sunset(solana).await;

    let result = send_tx(solana, close_ooa(janitor, init.account_1)).await;
    assert_error(result, OpenBookError::OpenOrdersNotCancelled, "resting ask");

    // cancel and close in one transaction
    let mut tx = ClientTransaction::new(&init.context.solana);
    tx.add_instruction(cancel_all(janitor, init.market, init.account_1))
        .await;
    tx.add_instruction(close_ooa(janitor, init.account_1)).await;
    tx.send().await.unwrap();
    assert!(!solana.account_exists(init.account_1).await);

    Ok(())
}

#[tokio::test]
async fn test_janitor_close_open_orders_account() -> Result<(), TransportError> {
    let init = new_market(TestNewMarketInitialize::default()).await;
    let solana = &init.context.solana.clone();
    let janitor = janitor_keypair();
    let (base, quote) = (12_300, 45_600);
    let (owner, open_orders_account) = owner_without_wallet(solana, &init, base, quote).await;

    sunset(solana).await;

    let base_ata = get_associated_token_address(&owner.pubkey(), &init.mints[0].pubkey);
    let quote_ata = get_associated_token_address(&owner.pubkey(), &init.mints[1].pubkey);
    assert!(!solana.account_exists(base_ata).await);
    assert!(!solana.account_exists(quote_ata).await);
    let market_before = solana.get_account::<Market>(init.market).await;
    let ooa_lamports = solana.get_lamports(open_orders_account).await;
    let indexer = indexer_address(owner.pubkey());
    let indexer_lamports = solana.get_lamports(indexer).await;
    let janitor_before = solana.get_lamports(JANITOR).await;

    let (_, instruction) = close_ooa(janitor, open_orders_account)
        .to_instruction(&**solana)
        .await;
    let compute_units = solana
        .simulate_compute_units(&[instruction], &[janitor])
        .await;
    println!("janitor_close_open_orders_account with two ATA creations: {compute_units} CU");
    assert!(compute_units < 200_000);

    send_tx(solana, close_ooa(janitor, open_orders_account))
        .await
        .unwrap();

    let rent = solana.rent;
    let token_rent = rent.minimum_balance(TOKEN_ACCOUNT_LEN);
    let reward = JANITOR_OPEN_ORDERS_ACCOUNT_REWARD;
    assert!(ooa_lamports - reward >= rent.minimum_balance(0));

    assert!(!solana.account_exists(open_orders_account).await);
    assert_eq!(solana.token_account_balance(base_ata).await, base);
    assert_eq!(solana.token_account_balance(quote_ata).await, quote);
    // the owner's last account: the indexer is closed into the owner as well
    assert!(!solana.account_exists(indexer).await);
    assert_eq!(
        solana.get_lamports(owner.pubkey()).await,
        ooa_lamports - reward + indexer_lamports
    );
    // the harness payer pays the fee
    assert_eq!(
        solana.get_lamports(JANITOR).await,
        janitor_before + reward - 2 * token_rent
    );

    let market = solana.get_account::<Market>(init.market).await;
    assert_eq!(
        market.base_deposit_total,
        market_before.base_deposit_total - base
    );
    assert_eq!(
        market.quote_deposit_total,
        market_before.quote_deposit_total - quote
    );

    Ok(())
}

#[tokio::test]
async fn test_janitor_close_open_orders_account_owner_topped_up() -> Result<(), TransportError> {
    let init = new_market(TestNewMarketInitialize::default()).await;
    let solana = &init.context.solana.clone();
    let janitor = janitor_keypair();
    let (owner, open_orders_account) = owner_without_wallet(solana, &init, 0, 0).await;

    // 1000 lamports above the reward and 1000 in the indexer: too little to create the owner
    // wallet, the reward pays for the rest
    let reward = JANITOR_OPEN_ORDERS_ACCOUNT_REWARD;
    let indexer = indexer_address(owner.pubkey());
    solana
        .set_lamports(open_orders_account, reward + 1_000)
        .await;
    solana.set_lamports(indexer, 1_000).await;

    sunset(solana).await;

    let janitor_before = solana.get_lamports(JANITOR).await;
    send_tx(solana, close_ooa(janitor, open_orders_account))
        .await
        .unwrap();

    assert!(!solana.account_exists(open_orders_account).await);
    assert!(!solana.account_exists(indexer).await);
    let minimum = solana.rent.minimum_balance(0);
    assert_eq!(solana.get_lamports(owner.pubkey()).await, minimum);
    assert_eq!(
        solana.get_lamports(JANITOR).await,
        janitor_before + reward + 2_000 - minimum
    );

    Ok(())
}

#[tokio::test]
async fn test_janitor_close_open_orders_account_with_penalty() -> Result<(), TransportError> {
    let init = new_market(TestNewMarketInitialize::default()).await;
    let solana = &init.context.solana.clone();
    let janitor = janitor_keypair();

    let taker_account = make_fills(solana, &init, None).await;
    let penalties = solana
        .get_account::<OpenOrdersAccount>(taker_account)
        .await
        .position
        .penalty_heap_count;
    assert_eq!(penalties, 2);

    sunset(solana).await;

    send_tx(
        solana,
        ConsumeEventsInstruction {
            consume_events_admin: Some(janitor),
            market: init.market,
            open_orders_accounts: vec![init.account_1, taker_account],
        },
    )
    .await
    .unwrap();

    let market_lamports = solana.get_lamports(init.market).await;
    let janitor_before = solana.get_lamports(JANITOR).await;
    let ooa_lamports = solana.get_lamports(taker_account).await;
    let taker = init.context.users[2].key.pubkey();
    let taker_base_ata = get_associated_token_address(&taker, &init.mints[0].pubkey);
    let taker_quote_ata = get_associated_token_address(&taker, &init.mints[1].pubkey);

    send_tx(solana, close_ooa(janitor, taker_account))
        .await
        .unwrap();

    let penalty = penalties * PENALTY_EVENT_HEAP;
    assert_eq!(
        solana.get_lamports(init.market).await,
        market_lamports + penalty
    );
    // the taker sold base, only a quote ATA is created
    assert!(!solana.account_exists(taker_base_ata).await);
    assert!(solana.token_account_balance(taker_quote_ata).await > 0);
    let token_rent = solana.rent.minimum_balance(TOKEN_ACCOUNT_LEN);
    let reward = JANITOR_OPEN_ORDERS_ACCOUNT_REWARD.min(ooa_lamports);
    assert_eq!(
        solana.get_lamports(JANITOR).await,
        janitor_before + reward - token_rent - penalty
    );

    Ok(())
}

#[tokio::test]
async fn test_janitor_close_open_orders_account_orphan() -> Result<(), TransportError> {
    let init = new_market(TestNewMarketInitialize {
        close_market_admin_bool: true,
        ..TestNewMarketInitialize::default()
    })
    .await;
    let solana = &init.context.solana.clone();
    let janitor = janitor_keypair();

    // the market is closed through the regular admin path
    send_tx(
        solana,
        SetMarketExpiredInstruction {
            close_market_admin: init.close_market_admin,
            market: init.market,
        },
    )
    .await
    .unwrap();
    send_tx(
        solana,
        CloseMarketInstruction {
            close_market_admin: init.close_market_admin,
            market: init.market,
            sol_destination: init.close_market_admin.pubkey(),
        },
    )
    .await
    .unwrap();
    assert!(!solana.account_exists(init.market).await);

    // Stale order counters on an orphan can not be cancelled any more, they do not block
    let mut data = solana.get_account_data(init.account_1).await.unwrap();
    let len = std::mem::size_of::<OpenOrdersAccount>();
    let mut open_orders_account: OpenOrdersAccount =
        bytemuck::pod_read_unaligned(&data[8..8 + len]);
    open_orders_account.position.bids_base_lots = 5;
    data[8..8 + len].copy_from_slice(bytemuck::bytes_of(&open_orders_account));
    solana.set_account_data(init.account_1, &data).await;

    sunset(solana).await;

    let owner_before = solana.get_lamports(init.owner.pubkey()).await;
    let ooa_lamports = solana.get_lamports(init.account_1).await;
    send_tx(solana, close_ooa(janitor, init.account_1))
        .await
        .unwrap();

    assert!(!solana.account_exists(init.account_1).await);
    let reward = JANITOR_OPEN_ORDERS_ACCOUNT_REWARD;
    assert_eq!(
        solana.get_lamports(init.owner.pubkey()).await,
        owner_before + ooa_lamports - reward
    );
    let indexer = solana
        .get_account::<OpenOrdersIndexer>(indexer_address(init.owner.pubkey()))
        .await;
    assert_eq!(indexer.addresses, vec![init.account_2]);

    Ok(())
}

#[tokio::test]
async fn test_janitor_close_open_orders_account_wrong_owner_ata() -> Result<(), TransportError> {
    let init = new_market(TestNewMarketInitialize::default()).await;
    let solana = &init.context.solana.clone();
    let janitor = janitor_keypair();

    send_tx(
        solana,
        DepositInstruction {
            open_orders_account: init.account_1,
            market: init.market,
            market_base_vault: init.market_base_vault,
            market_quote_vault: init.market_quote_vault,
            user_base_account: init.owner_token_0,
            user_quote_account: init.owner_token_1,
            owner: init.owner,
            base_amount: 1_000,
            quote_amount: 1_000,
        },
    )
    .await
    .unwrap();

    sunset(solana).await;

    // the owner's own, but not associated, token accounts
    let result = send_tx(
        solana,
        JanitorCloseOpenOrdersAccountInstruction {
            owner_base_ata: Some(init.owner_token_0),
            ..close_ooa(janitor, init.account_1)
        },
    )
    .await;
    assert_error(result, OpenBookError::InvalidOwnerTokenAccount, "base");
    let result = send_tx(
        solana,
        JanitorCloseOpenOrdersAccountInstruction {
            owner_quote_ata: Some(init.owner_token_1),
            ..close_ooa(janitor, init.account_1)
        },
    )
    .await;
    assert_error(result, OpenBookError::InvalidOwnerTokenAccount, "quote");

    // also with nothing to settle: the destination is checked whatever the amount
    let result = send_tx(
        solana,
        JanitorCloseOpenOrdersAccountInstruction {
            owner_base_ata: Some(init.owner_token_0),
            ..close_ooa(janitor, init.account_2)
        },
    )
    .await;
    assert_error(result, OpenBookError::InvalidOwnerTokenAccount, "empty");

    for account in [init.account_1, init.account_2] {
        send_tx(solana, close_ooa(janitor, account)).await.unwrap();
    }

    Ok(())
}

#[tokio::test]
async fn test_janitor_close_open_orders_account_closes_indexer() -> Result<(), TransportError> {
    let init = new_market(TestNewMarketInitialize::default()).await;
    let solana = &init.context.solana.clone();
    let janitor = janitor_keypair();
    let owner = init.owner.pubkey();
    let indexer = indexer_address(owner);

    sunset(solana).await;

    let result = send_tx(
        solana,
        JanitorCloseOpenOrdersIndexerInstruction {
            janitor,
            owner,
            open_orders_indexer: indexer,
        },
    )
    .await;
    assert_error(result, OpenBookError::IndexerActiveOO, "two accounts");

    let indexer_lamports = solana.get_lamports(indexer).await;
    send_tx(solana, close_ooa(janitor, init.account_1))
        .await
        .unwrap();
    // not resized, the indexer keeps its lamports until it is empty
    let indexer_account = solana.get_account::<OpenOrdersIndexer>(indexer).await;
    assert_eq!(indexer_account.addresses, vec![init.account_2]);
    assert_eq!(solana.get_lamports(indexer).await, indexer_lamports);

    let owner_before = solana.get_lamports(owner).await;
    let ooa_lamports = solana.get_lamports(init.account_2).await;
    let janitor_before = solana.get_lamports(JANITOR).await;
    send_tx(solana, close_ooa(janitor, init.account_2))
        .await
        .unwrap();

    // the last account: the indexer goes to the owner, the janitor only keeps the reward
    assert!(!solana.account_exists(indexer).await);
    let reward = JANITOR_OPEN_ORDERS_ACCOUNT_REWARD;
    assert_eq!(
        solana.get_lamports(owner).await,
        owner_before + ooa_lamports - reward + indexer_lamports
    );
    assert_eq!(solana.get_lamports(JANITOR).await, janitor_before + reward);

    Ok(())
}

#[tokio::test]
async fn test_janitor_close_open_orders_indexer() -> Result<(), TransportError> {
    let init = new_market(TestNewMarketInitialize::default()).await;
    let solana = &init.context.solana.clone();
    let janitor = janitor_keypair();

    // an indexer that never had an open orders account, its owner wallet does not exist
    let owner = TestKeypair::new();
    let indexer =
        create_open_orders_indexer(solana, &init.context.users[1], owner, init.market).await;

    sunset(solana).await;

    let close_indexer = |owner| JanitorCloseOpenOrdersIndexerInstruction {
        janitor,
        owner,
        open_orders_indexer: indexer,
    };

    // the indexer does not store its owner, the seeds check the one the janitor passes
    let result = send_tx(solana, close_indexer(init.owner.pubkey())).await;
    assert_openbook_error(
        &result,
        anchor_lang::error::ErrorCode::ConstraintSeeds.into(),
        "wrong owner".to_string(),
    );
    let result = send_tx(solana, close_indexer(JANITOR)).await;
    assert!(result.is_err(), "janitor as owner");

    let indexer_lamports = solana.get_lamports(indexer).await;
    send_tx(solana, close_indexer(owner.pubkey()))
        .await
        .unwrap();

    assert!(!solana.account_exists(indexer).await);
    assert_eq!(solana.get_lamports(owner.pubkey()).await, indexer_lamports);

    Ok(())
}

#[tokio::test]
async fn test_janitor_close_open_orders_indexer_wrong_account() -> Result<(), TransportError> {
    let init = new_market(TestNewMarketInitialize::default()).await;
    let solana = &init.context.solana.clone();
    let janitor = janitor_keypair();

    sunset(solana).await;

    // an open orders account is not an indexer, even with matching seeds
    let result = send_tx(
        solana,
        JanitorCloseOpenOrdersIndexerInstruction {
            janitor,
            owner: init.owner.pubkey(),
            open_orders_indexer: init.account_1,
        },
    )
    .await;
    assert!(result.is_err());
    assert!(solana.account_exists(init.account_1).await);

    Ok(())
}

/// Market with fills, a resting bid and pending fees. Returns the taker's account.
async fn busy_market(solana: &SolanaCookie, init: &TestInitialize) -> Pubkey {
    let taker_account = make_fills(solana, init, None).await;
    send_tx(
        solana,
        limit_order(
            init,
            init.account_2,
            init.owner,
            (init.owner_token_0, init.owner_token_1),
            Side::Bid,
            1,
        ),
    )
    .await
    .unwrap();
    taker_account
}

#[tokio::test]
async fn test_janitor_close_market_blocked() -> Result<(), TransportError> {
    let init = new_market(TestNewMarketInitialize::default()).await;
    let solana = &init.context.solana.clone();
    let janitor = janitor_keypair();
    let taker_account = busy_market(solana, &init).await;
    let fee_admin_ata = solana
        .create_associated_token_account(&init.collect_fee_admin.pubkey(), init.mints[1].pubkey)
        .await;

    sunset(solana).await;

    let close_market = || JanitorCloseMarketInstruction {
        janitor,
        market: init.market,
    };
    let result = send_tx(solana, close_market()).await;
    assert_error(result, OpenBookError::EventHeapContainsElements, "fills");

    send_tx(
        solana,
        ConsumeEventsInstruction {
            consume_events_admin: Some(janitor),
            market: init.market,
            open_orders_accounts: vec![init.account_1, taker_account],
        },
    )
    .await
    .unwrap();
    let result = send_tx(solana, close_market()).await;
    assert_error(result, OpenBookError::BookContainsElements, "resting bid");

    send_tx(solana, cancel_all(janitor, init.market, init.account_2))
        .await
        .unwrap();
    let result = send_tx(solana, close_market()).await;
    assert_error(result, OpenBookError::NonEmptyMarket, "deposits");

    for account in [init.account_1, init.account_2, taker_account] {
        send_tx(solana, close_ooa(janitor, account)).await.unwrap();
    }
    let market = solana.get_account::<Market>(init.market).await;
    assert_eq!(market.base_deposit_total, 0);
    assert!(market.fees_available > 0);
    let result = send_tx(solana, close_market()).await;
    assert_error(result, OpenBookError::NonEmptyMarket, "fees");

    // the janitor may sweep only to the collect_fee_admin
    let sweep = |collect_fee_admin, token_receiver_account| SweepFeesInstruction {
        collect_fee_admin,
        market: init.market,
        market_quote_vault: init.market_quote_vault,
        token_receiver_account,
    };
    let janitor_quote_ata = solana
        .create_associated_token_account(&JANITOR, init.mints[1].pubkey)
        .await;
    let result = send_tx(solana, sweep(janitor, janitor_quote_ata)).await;
    assert_error(
        result,
        OpenBookError::InvalidFeeReceiver,
        "sweep to janitor",
    );
    let result = send_tx(solana, sweep(TestKeypair::new(), fee_admin_ata)).await;
    assert_openbook_error(
        &result,
        ANCHOR_CONSTRAINT_HAS_ONE,
        "random signer".to_string(),
    );

    send_tx(solana, sweep(janitor, fee_admin_ata))
        .await
        .unwrap();
    assert_eq!(
        solana.token_account_balance(fee_admin_ata).await,
        market.fees_available
    );

    send_tx(solana, close_market()).await.unwrap();

    Ok(())
}

#[tokio::test]
async fn test_janitor_close_market() -> Result<(), TransportError> {
    let init = new_market(TestNewMarketInitialize::default()).await;
    let solana = &init.context.solana.clone();
    let janitor = janitor_keypair();

    sunset(solana).await;

    for account in [init.account_1, init.account_2] {
        send_tx(solana, close_ooa(janitor, account)).await.unwrap();
    }

    // donations to the vaults
    solana
        .set_account_balance(init.market_base_vault, 123)
        .await;
    solana
        .set_account_balance(init.market_quote_vault, 456)
        .await;

    let market = solana.get_account::<Market>(init.market).await;
    let closed = [
        init.market,
        market.bids,
        market.asks,
        market.event_heap,
        init.market_base_vault,
        init.market_quote_vault,
    ];
    let mut rent = 0;
    for account in closed {
        rent += solana.get_lamports(account).await;
    }

    // the donations are burned; the test mints start at supply 0, a real chain's supply covers
    // what its token accounts hold
    const SUPPLY: u64 = 1_000_000;
    solana.set_mint_supply(init.mints[0].pubkey, SUPPLY).await;
    solana.set_mint_supply(init.mints[1].pubkey, SUPPLY).await;

    let escrow_before = solana.get_lamports(ESCROW).await;
    send_tx(
        solana,
        JanitorCloseMarketInstruction {
            janitor,
            market: init.market,
        },
    )
    .await
    .unwrap();

    for account in closed {
        assert!(!solana.account_exists(account).await, "{account} closed");
    }
    assert_eq!(solana.get_lamports(ESCROW).await, escrow_before + rent);
    assert_eq!(
        mint_supply(solana, init.mints[0].pubkey).await,
        SUPPLY - 123
    );
    assert_eq!(
        mint_supply(solana, init.mints[1].pubkey).await,
        SUPPLY - 456
    );

    Ok(())
}

#[tokio::test]
async fn test_janitor_wrong_signer() -> Result<(), TransportError> {
    let init = new_market(TestNewMarketInitialize::default()).await;
    let solana = &init.context.solana.clone();
    let impostor = TestKeypair::new();

    sunset(solana).await;

    let result = send_tx(solana, close_ooa(impostor, init.account_1)).await;
    assert_error(result, OpenBookError::InvalidJanitor, "close ooa");

    let result = send_tx(
        solana,
        JanitorCloseOpenOrdersIndexerInstruction {
            janitor: impostor,
            owner: init.owner.pubkey(),
            open_orders_indexer: indexer_address(init.owner.pubkey()),
        },
    )
    .await;
    assert_error(result, OpenBookError::InvalidJanitor, "close indexer");

    let result = send_tx(
        solana,
        JanitorCloseMarketInstruction {
            janitor: impostor,
            market: init.market,
        },
    )
    .await;
    assert_error(result, OpenBookError::InvalidJanitor, "close market");

    Ok(())
}

#[tokio::test]
async fn test_janitor_inactive_before_sunset() -> Result<(), TransportError> {
    // before SUNSET_TS the janitor is an ordinary key: none of its powers apply
    let init = new_market(TestNewMarketInitialize {
        consume_events_admin_bool: true,
        ..TestNewMarketInitialize::default()
    })
    .await;
    let solana = &init.context.solana.clone();
    let janitor = janitor_keypair();
    let taker_account = busy_market(solana, &init).await;
    let fee_admin_ata = solana
        .create_associated_token_account(&init.collect_fee_admin.pubkey(), init.mints[1].pubkey)
        .await;
    assert!(solana.get_clock().await.unix_timestamp < SUNSET_TS);

    let crank = || ConsumeEventsInstruction {
        consume_events_admin: Some(janitor),
        market: init.market,
        open_orders_accounts: vec![init.account_1, taker_account],
    };
    let sweep = || SweepFeesInstruction {
        collect_fee_admin: janitor,
        market: init.market,
        market_quote_vault: init.market_quote_vault,
        token_receiver_account: fee_admin_ata,
    };

    let result = send_tx(solana, crank()).await;
    assert_error(result, OpenBookError::InvalidConsumeEventsAdmin, "crank");
    let result = send_tx(solana, cancel_all(janitor, init.market, init.account_2)).await;
    assert_error(result, OpenBookError::NoOwnerOrDelegate, "cancel");
    assert!(send_tx(solana, sweep()).await.is_err(), "sweep fees");
    let result = send_tx(solana, close_ooa(janitor, init.account_1)).await;
    assert_error(result, OpenBookError::JanitorNotActive, "close ooa");
    let result = send_tx(
        solana,
        JanitorCloseOpenOrdersIndexerInstruction {
            janitor,
            owner: init.owner.pubkey(),
            open_orders_indexer: indexer_address(init.owner.pubkey()),
        },
    )
    .await;
    assert_error(result, OpenBookError::JanitorNotActive, "close indexer");
    let result = send_tx(
        solana,
        JanitorCloseMarketInstruction {
            janitor,
            market: init.market,
        },
    )
    .await;
    assert_error(result, OpenBookError::JanitorNotActive, "close market");

    let account = solana
        .get_account::<OpenOrdersAccount>(init.account_2)
        .await;
    assert_ne!(account.position.bids_base_lots, 0);
    assert_ne!(event_heap_len(solana, init.market).await, 0);

    sunset(solana).await;

    send_tx(solana, crank()).await.unwrap();
    assert_eq!(event_heap_len(solana, init.market).await, 0);
    send_tx(solana, cancel_all(janitor, init.market, init.account_2))
        .await
        .unwrap();
    let account = solana
        .get_account::<OpenOrdersAccount>(init.account_2)
        .await;
    assert_eq!(account.position.bids_base_lots, 0);
    send_tx(solana, close_ooa(janitor, init.account_1))
        .await
        .unwrap();

    Ok(())
}

#[tokio::test]
async fn test_janitor_full_wind_down() -> Result<(), TransportError> {
    let init = new_market(TestNewMarketInitialize::default()).await;
    let solana = &init.context.solana.clone();
    let janitor = janitor_keypair();
    let users = &init.context.users;

    // a second, permissioned market: base mint 2, quote mint 1
    let market_2 = send_tx(
        solana,
        CreateMarketInstruction {
            collect_fee_admin: init.collect_fee_admin.pubkey(),
            open_orders_admin: Some(init.open_orders_admin.pubkey()),
            consume_events_admin: Some(init.consume_events_admin.pubkey()),
            payer: init.payer,
            market: TestKeypair::new(),
            quote_lot_size: 10,
            base_lot_size: 100,
            maker_fee: -200,
            taker_fee: 400,
            base_mint: init.mints[2].pubkey,
            quote_mint: init.mints[1].pubkey,
            ..CreateMarketInstruction::with_new_book_and_heap(solana, None, None).await
        },
    )
    .await
    .unwrap();
    let (owner_a, owner_b, owner_c) = (users[0].key, users[2].key, users[3].key);
    let tokens = |user: usize, base_mint: usize| {
        (
            users[user].token_accounts[base_mint],
            users[user].token_accounts[1],
        )
    };
    let a_1 = init.account_1;
    let a_2 = init.account_2;
    let a_3 = new_open_orders_account(solana, &init, market_2.market, owner_a, 3).await;
    let b_1 = new_open_orders_account(solana, &init, init.market, owner_b, 1).await;
    let b_2 = new_open_orders_account(solana, &init, market_2.market, owner_b, 2).await;
    let c_1 = new_open_orders_account(solana, &init, init.market, owner_c, 1).await;

    let m1 = |account, owner, user, side, lots| {
        limit_order(&init, account, owner, tokens(user, 0), side, lots)
    };
    let m2 = |account, owner, user, side, lots| PlaceOrderInstruction {
        open_orders_admin: Some(init.open_orders_admin),
        market: market_2.market,
        market_vault: match side {
            Side::Bid => market_2.market_quote_vault,
            Side::Ask => market_2.market_base_vault,
        },
        ..limit_order(&init, account, owner, tokens(user, 2), side, lots)
    };
    let price_2 = solana
        .get_account::<Market>(market_2.market)
        .await
        .native_price_to_lot(I80F48::from(1000))
        .unwrap();
    let orders = [
        m1(a_1, owner_a, 0, Side::Bid, 3),
        m1(b_1, owner_b, 2, Side::Ask, 1),
        m1(c_1, owner_c, 3, Side::Ask, 5),
        m1(a_2, owner_a, 0, Side::Bid, 1),
        PlaceOrderInstruction {
            price_lots: price_2,
            ..m2(a_3, owner_a, 0, Side::Bid, 2)
        },
        PlaceOrderInstruction {
            price_lots: price_2,
            ..m2(b_2, owner_b, 2, Side::Ask, 1)
        },
    ];
    for order in orders {
        send_tx(solana, order).await.unwrap();
    }
    send_tx(
        solana,
        DepositInstruction {
            open_orders_account: a_2,
            market: init.market,
            market_base_vault: init.market_base_vault,
            market_quote_vault: init.market_quote_vault,
            user_base_account: users[0].token_accounts[0],
            user_quote_account: users[0].token_accounts[1],
            owner: owner_a,
            base_amount: 777,
            quote_amount: 888,
        },
    )
    .await
    .unwrap();
    sunset(solana).await;

    let markets = [
        (init.market, vec![a_1, a_2, b_1, c_1]),
        (market_2.market, vec![a_3, b_2]),
    ];
    let owners = [owner_a.pubkey(), owner_b.pubkey(), owner_c.pubkey()];
    let mints = [
        init.mints[0].pubkey,
        init.mints[1].pubkey,
        init.mints[2].pubkey,
    ];

    // snapshot: rent held by the program and the vaults, tokens in the vaults
    let mut program_accounts = vec![];
    let mut vaults = vec![];
    for (market, accounts) in &markets {
        let m = solana.get_account::<Market>(*market).await;
        program_accounts.extend([*market, m.bids, m.asks, m.event_heap]);
        program_accounts.extend(accounts);
        vaults.extend([
            (m.market_base_vault, m.base_mint),
            (m.market_quote_vault, m.quote_mint),
        ]);
    }
    program_accounts.extend(owners.iter().map(|owner| indexer_address(*owner)));
    let mut rent_before = 0;
    for account in program_accounts
        .iter()
        .chain(vaults.iter().map(|(vault, _)| vault))
    {
        rent_before += solana.get_lamports(*account).await;
    }
    let mut tokens_before = std::collections::HashMap::<Pubkey, u64>::new();
    for (vault, mint) in &vaults {
        *tokens_before.entry(*mint).or_default() += solana.token_account_balance(*vault).await;
    }
    let mut wallets_before = vec![];
    for owner in owners {
        wallets_before.push(solana.get_lamports(owner).await);
    }
    let janitor_before = solana.get_lamports(JANITOR).await;
    let escrow_before = solana.get_lamports(ESCROW).await;
    for owner in owners {
        for mint in mints {
            assert!(
                !solana
                    .account_exists(get_associated_token_address(&owner, &mint))
                    .await
            );
        }
    }

    // crank
    for (market, accounts) in &markets {
        while event_heap_len(solana, *market).await > 0 {
            send_tx(
                solana,
                ConsumeEventsInstruction {
                    consume_events_admin: Some(janitor),
                    market: *market,
                    open_orders_accounts: accounts.clone(),
                },
            )
            .await
            .unwrap();
        }
    }

    // close open orders accounts, cancelling first where needed
    for (market, accounts) in &markets {
        for account in accounts {
            let position = solana
                .get_account::<OpenOrdersAccount>(*account)
                .await
                .position;
            let mut tx = ClientTransaction::new(&init.context.solana);
            if position.bids_base_lots != 0 || position.asks_base_lots != 0 {
                tx.add_instruction(cancel_all(janitor, *market, *account))
                    .await;
            }
            tx.add_instruction(close_ooa(janitor, *account)).await;
            tx.send().await.unwrap();
        }
    }

    // the indexers were closed into their owners with the last open orders account

    // sweep fees to the collect_fee_admin
    let fee_admin_ata = solana
        .create_associated_token_account(&init.collect_fee_admin.pubkey(), init.mints[1].pubkey)
        .await;
    for (market, _) in &markets {
        let m = solana.get_account::<Market>(*market).await;
        if m.fees_available > 0 {
            send_tx(
                solana,
                SweepFeesInstruction {
                    collect_fee_admin: janitor,
                    market: *market,
                    market_quote_vault: m.market_quote_vault,
                    token_receiver_account: fee_admin_ata,
                },
            )
            .await
            .unwrap();
        }
    }

    // close markets, burning what is left in the vaults; the test mints start at supply 0
    const SUPPLY: u64 = 1_000_000_000_000;
    for mint in mints {
        solana.set_mint_supply(mint, SUPPLY).await;
    }
    for (market, _) in &markets {
        send_tx(
            solana,
            JanitorCloseMarketInstruction {
                janitor,
                market: *market,
            },
        )
        .await
        .unwrap();
    }

    // nothing left
    for account in program_accounts
        .iter()
        .chain(vaults.iter().map(|(vault, _)| vault))
    {
        assert!(!solana.account_exists(*account).await, "{account} closed");
    }

    // every lamport went to an owner, the janitor, the escrow or an ATA the janitor paid for
    let mut rent_after = solana.get_lamports(JANITOR).await - janitor_before
        + solana.get_lamports(ESCROW).await
        - escrow_before;
    let mut tokens_after = std::collections::HashMap::<Pubkey, u64>::new();
    for (owner, before) in owners.iter().zip(wallets_before) {
        rent_after += solana.get_lamports(*owner).await - before;
        for mint in mints {
            let ata = get_associated_token_address(owner, &mint);
            if solana.account_exists(ata).await {
                rent_after += solana.get_lamports(ata).await;
                *tokens_after.entry(mint).or_default() += solana.token_account_balance(ata).await;
            }
        }
    }
    assert_eq!(rent_after, rent_before);

    // every token went to an owner or the collect_fee_admin, or was burned with the vaults
    *tokens_after.entry(init.mints[1].pubkey).or_default() +=
        solana.token_account_balance(fee_admin_ata).await;
    for mint in mints {
        *tokens_after.entry(mint).or_default() += SUPPLY - mint_supply(solana, mint).await;
    }
    for (mint, amount) in tokens_before {
        assert_eq!(tokens_after.get(&mint).copied().unwrap_or(0), amount);
    }

    Ok(())
}
