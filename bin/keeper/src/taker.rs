use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use fixed::types::I80F48;
use futures::Future;
use openbook_v2::accounts_ix::{Serum3OrderType, Serum3SelfTradeBehavior, Serum3Side};

use tokio::time;

use crate::OpenBookClient;

pub async fn runner(
    openbook_client: Arc<OpenBookClient>,
    _debugging_handle: impl Future,
) -> Result<(), anyhow::Error> {
    ensure_deposit(&openbook_client).await?;
    ensure_oo(&openbook_client).await?;

    let mut price_arcs = HashMap::new();
    for market_name in openbook_client.context.serum3_market_indexes_by_name.keys() {
        let price = openbook_client
            .get_oracle_price(
                market_name
                    .split('/')
                    .collect::<Vec<&str>>()
                    .first()
                    .unwrap(),
            )
            .await
            .unwrap();
        price_arcs.insert(
            market_name.to_owned(),
            Arc::new(RwLock::new(
                I80F48::from_num(price.price) / I80F48::from_num(10u64.pow(-price.expo as u32)),
            )),
        );
    }

    let handles1 = openbook_client
        .context
        .serum3_market_indexes_by_name
        .keys()
        .map(|market_name| {
            loop_blocking_price_update(
                openbook_client.clone(),
                market_name.to_owned(),
                price_arcs.get(market_name).unwrap().clone(),
            )
        })
        .collect::<Vec<_>>();

    let handles2 = openbook_client
        .context
        .serum3_market_indexes_by_name
        .keys()
        .map(|market_name| {
            loop_blocking_orders(
                openbook_client.clone(),
                market_name.to_owned(),
                price_arcs.get(market_name).unwrap().clone(),
            )
        })
        .collect::<Vec<_>>();

    futures::join!(
        futures::future::join_all(handles1),
        futures::future::join_all(handles2)
    );

    Ok(())
}

async fn ensure_oo(openbook_client: &Arc<OpenBookClient>) -> Result<(), anyhow::Error> {
    let account = openbook_client.openbook_account().await?;

    for (market_index, serum3_market) in openbook_client.context.serum3_markets.iter() {
        if account.serum3_orders(*market_index).is_err() {
            openbook_client
                .serum3_create_open_orders(serum3_market.market.name())
                .await?;
        }
    }

    Ok(())
}

async fn ensure_deposit(openbook_client: &Arc<OpenBookClient>) -> Result<(), anyhow::Error> {
    let openbook_account = openbook_client.openbook_account().await?;

    for &token_index in openbook_client.context.tokens.keys() {
        let bank = openbook_client.first_bank(token_index).await?;
        let desired_balance = I80F48::from_num(10_000 * 10u64.pow(bank.mint_decimals as u32));

        let token_account_opt = openbook_account.token_position(token_index).ok();

        let deposit_native = match token_account_opt {
            Some(token_account) => {
                let native = token_account.native(&bank);
                let ui = token_account.ui(&bank);
                log::info!("Current balance {} {}", ui, bank.name());

                if native < I80F48::ZERO {
                    desired_balance - native
                } else {
                    desired_balance - native.min(desired_balance)
                }
            }
            None => desired_balance,
        };

        if deposit_native == I80F48::ZERO {
            continue;
        }

        log::info!("Depositing {} {}", deposit_native, bank.name());
        openbook_client
            .token_deposit(bank.mint, desired_balance.to_num(), false)
            .await?;
    }

    Ok(())
}

pub async fn loop_blocking_price_update(
    openbook_client: Arc<OpenBookClient>,
    market_name: String,
    price: Arc<RwLock<I80F48>>,
) {
    let mut interval = time::interval(Duration::from_secs(1));
    let token_name = market_name.split('/').collect::<Vec<&str>>()[0];
    loop {
        interval.tick().await;

        let fresh_price = openbook_client.get_oracle_price(token_name).await.unwrap();
        log::info!("{} Updated price is {:?}", token_name, fresh_price.price);
        if let Ok(mut price) = price.write() {
            *price = I80F48::from_num(fresh_price.price)
                / I80F48::from_num(10u64.pow(-fresh_price.expo as u32));
        }
    }
}

pub async fn loop_blocking_orders(
    openbook_client: Arc<OpenBookClient>,
    market_name: String,
    price: Arc<RwLock<I80F48>>,
) {
    let mut interval = time::interval(Duration::from_secs(5));

    // Cancel existing orders
    let orders: Vec<u128> = openbook_client
        .serum3_cancel_all_orders(&market_name)
        .await
        .unwrap();
    log::info!("Cancelled orders - {:?} for {}", orders, market_name);

    loop {
        interval.tick().await;

        let client = openbook_client.clone();
        let market_name = market_name.clone();
        let price = price.clone();

        let res = (|| async move {
            client.serum3_settle_funds(&market_name).await?;

            let fresh_price = match price.read() {
                Ok(price) => *price,
                Err(_) => {
                    anyhow::bail!("Price RwLock PoisonError!");
                }
            };

            let fresh_price = fresh_price.to_num::<f64>();

            let bid_price = fresh_price + fresh_price * 0.1;
            let res = client
                .serum3_place_order(
                    &market_name,
                    Serum3Side::Bid,
                    bid_price,
                    0.0001,
                    Serum3SelfTradeBehavior::DecrementTake,
                    Serum3OrderType::ImmediateOrCancel,
                    SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64,
                    10,
                )
                .await;
            if let Err(e) = res {
                log::error!("Error while placing taker bid {:#?}", e)
            } else {
                log::info!("Placed bid at {} for {}", bid_price, market_name)
            }

            let ask_price = fresh_price - fresh_price * 0.1;
            let res = client
                .serum3_place_order(
                    &market_name,
                    Serum3Side::Ask,
                    ask_price,
                    0.0001,
                    Serum3SelfTradeBehavior::DecrementTake,
                    Serum3OrderType::ImmediateOrCancel,
                    SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64,
                    10,
                )
                .await;
            if let Err(e) = res {
                log::error!("Error while placing taker ask {:#?}", e)
            } else {
                log::info!("Placed ask at {} for {}", ask_price, market_name)
            }

            Ok(())
        })()
        .await;

        if let Err(err) = res {
            log::error!("{:?}", err);
        }
    }
}
