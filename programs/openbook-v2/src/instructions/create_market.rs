use anchor_lang::prelude::*;
use fixed::types::I80F48;

use crate::error::*;
use crate::state::*;
use crate::util::fill_from_str;

use crate::accounts_ix::*;
use crate::logs::MarketMetaDataLog;

#[allow(clippy::too_many_arguments)]
pub fn create_market(
    ctx: Context<CreateMarket>,
    market_index: MarketIndex,
    name: String,
    oracle_config: OracleConfigParams,
    quote_lot_size: i64,
    base_lot_size: i64,
    maker_fee: f32,
    taker_fee: f32,
    fee_penalty: f32,
) -> Result<()> {
    let now_ts: u64 = Clock::get()?.unix_timestamp.try_into().unwrap();

    let mut openbook_market = ctx.accounts.market.load_init()?;
    *openbook_market = Market {
        market_index,
        bump: *ctx.bumps.get("market").ok_or(OpenBookError::SomeError)?,
        base_decimals: ctx.accounts.base_mint.decimals,
        quote_decimals: ctx.accounts.quote_mint.decimals,
        padding1: Default::default(),
        name: fill_from_str(&name)?,
        bids: ctx.accounts.bids.key(),
        asks: ctx.accounts.asks.key(),
        event_queue: ctx.accounts.event_queue.key(),
        oracle: ctx.accounts.oracle.key(),
        oracle_config: oracle_config.to_oracle_config(),
        stable_price_model: StablePriceModel::default(),
        quote_lot_size,
        base_lot_size,
        seq_num: 0,
        registration_time: now_ts,

        maker_fee: I80F48::from_num(maker_fee),
        taker_fee: I80F48::from_num(taker_fee),
        fees_accrued: I80F48::ZERO,
        fees_settled: I80F48::ZERO,
        fee_penalty,
        padding2: Default::default(),

        buyback_fees_expiry_interval: 10000,
        vault_signer_nonce: 0,
        base_mint: ctx.accounts.base_mint.key(),
        quote_mint: ctx.accounts.quote_mint.key(),
        base_vault: ctx.accounts.base_vault.key(),
        base_deposit_total: 0,
        base_fees_accrued: 0,
        quote_vault: ctx.accounts.quote_vault.key(),
        quote_deposit_total: 0,
        quote_fees_accrued: 0,

        reserved: [0; 1888],
    };

    let mut orderbook = Orderbook {
        bids: ctx.accounts.bids.load_init()?,
        asks: ctx.accounts.asks.load_init()?,
    };
    orderbook.init();

    emit!(MarketMetaDataLog {
        market: ctx.accounts.market.key(),
        market_index,
        base_decimals: ctx.accounts.base_mint.decimals,
        quote_decimals: ctx.accounts.quote_mint.decimals,
        base_lot_size,
        quote_lot_size,
        oracle: ctx.accounts.oracle.key(),
    });

    Ok(())
}
