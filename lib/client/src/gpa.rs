use anchor_lang::{AccountDeserialize, Discriminator};

use openbook_v2::state::{Market, MarketIndex, OpenOrdersAccount};

use solana_account_decoder::UiAccountEncoding;
use solana_client::nonblocking::rpc_client::RpcClient as RpcClientAsync;
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
use solana_client::rpc_filter::{Memcmp, RpcFilterType};
use solana_sdk::pubkey::Pubkey;

pub async fn fetch_openbook_accounts(
    rpc: &RpcClientAsync,
    owner: Pubkey,
) -> anyhow::Result<Vec<(Pubkey, OpenOrdersAccount)>> {
    let config = RpcProgramAccountsConfig {
        filters: Some(vec![
            RpcFilterType::Memcmp(Memcmp::new_raw_bytes(
                0,
                OpenOrdersAccount::discriminator().to_vec(),
            )),
            RpcFilterType::Memcmp(Memcmp::new_raw_bytes(8, owner.to_bytes().to_vec())),
        ]),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            ..RpcAccountInfoConfig::default()
        },
        ..RpcProgramAccountsConfig::default()
    };
    rpc.get_program_accounts_with_config(&openbook_v2::id(), config)
        .await?
        .into_iter()
        .map(|(key, account)| {
            Ok((
                key,
                OpenOrdersAccount::try_deserialize(&mut (&account.data as &[u8]))?,
            ))
        })
        .collect()
}

pub async fn fetch_anchor_account<T: AccountDeserialize>(
    rpc: &RpcClientAsync,
    address: &Pubkey,
) -> anyhow::Result<T> {
    let account = rpc.get_account(address).await?;
    Ok(T::try_deserialize(&mut (&account.data as &[u8]))?)
}

async fn fetch_anchor_accounts<T: AccountDeserialize + Discriminator>(
    rpc: &RpcClientAsync,
    program: Pubkey,
) -> anyhow::Result<Vec<(Pubkey, T)>> {
    let account_type_filter =
        RpcFilterType::Memcmp(Memcmp::new_raw_bytes(0, T::discriminator().to_vec()));
    let config = RpcProgramAccountsConfig {
        filters: Some([vec![account_type_filter]].concat()),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            ..RpcAccountInfoConfig::default()
        },
        ..RpcProgramAccountsConfig::default()
    };
    rpc.get_program_accounts_with_config(&program, config)
        .await?
        .into_iter()
        .map(|(key, account)| Ok((key, T::try_deserialize(&mut (&account.data as &[u8]))?)))
        .collect()
}

pub async fn fetch_markets(rpc: &RpcClientAsync) -> anyhow::Result<Vec<(Pubkey, Market)>> {
    fetch_anchor_accounts::<Market>(rpc, openbook_v2::id()).await
}

pub async fn fetch_market_by_payer_and_index(
    index: MarketIndex,
    payer: Pubkey,
    rpc: &RpcClientAsync,
) -> anyhow::Result<Market> {
    let market_pubkey: Pubkey = Pubkey::find_program_address(
        &[
            b"Market".as_ref(),
            payer.to_bytes().as_ref(),
            &index.to_le_bytes(),
        ],
        &openbook_v2::id(),
    )
    .0;
    let account = rpc.get_account_data(&market_pubkey).await?;
    let market = Market::try_deserialize(&mut (&account as &[u8]))?;
    Ok(market)
}
