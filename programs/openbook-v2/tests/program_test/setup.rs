#![allow(dead_code)]

use anchor_lang::prelude::*;

use super::client::*;
use super::solana::SolanaCookie;
use super::{send_tx, MintCookie, TestKeypair, UserCookie};

#[derive(Clone)]
pub struct Token {
    pub index: u16,
    pub mint: MintCookie,
    pub oracle: Pubkey,
    pub mint_info: Pubkey,
}

impl Token {
    pub async fn create(
        mints: Vec<MintCookie>,
        solana: &SolanaCookie,
        admin: TestKeypair,
        payer: TestKeypair,
    ) -> Vec<Token> {
        let mut tokens = vec![];

        for (index, mint) in mints.iter().enumerate() {
            let create_stub_oracle_accounts = send_tx(
                solana,
                StubOracleCreate {
                    mint: mint.pubkey,
                    admin,
                    payer,
                },
            )
            .await
            .unwrap();
            let oracle = create_stub_oracle_accounts.oracle;
            send_tx(
                solana,
                StubOracleSetInstruction {
                    admin,
                    mint: mint.pubkey,
                    price: 1.0,
                },
            )
            .await
            .unwrap();
            let token_index = index as u16;
            tokens.push(Token {
                index: token_index,
                mint: mint.clone(),
                oracle,
                mint_info: mint.pubkey,
            });
        }
        tokens
    }
}

pub async fn create_funded_account(
    solana: &SolanaCookie,
    owner: TestKeypair,
    market: Pubkey,
    account_num: u32,
    payer: &UserCookie,
    mints: &[MintCookie],
    amounts: u64,
) -> Pubkey {
    let account = send_tx(
        solana,
        InitOpenOrdersInstruction {
            account_num,
            open_orders_count: 8,
            market,
            owner,
            payer: payer.key,
        },
    )
    .await
    .unwrap()
    .open_orders_account;
    account
}
