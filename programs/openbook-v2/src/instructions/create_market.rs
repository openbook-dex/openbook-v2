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
    fee_penalty: u64,
    collect_fee_admin: Pubkey,
    manage_oracle_admin: Option<Pubkey>,
    open_orders_admin: Option<Pubkey>,
    consume_events_admin: Option<Pubkey>,
    close_market_admin: Option<Pubkey>,
) -> Result<()> {
    let now_ts: u64 = Clock::get()?.unix_timestamp.try_into().unwrap();

    require!(
        taker_fee.is_sign_positive()
            && (maker_fee.is_sign_positive() || maker_fee.abs() <= taker_fee),
        OpenBookError::InvalidFeesError
    );

    let mut openbook_market = ctx.accounts.market.load_init()?;
    *openbook_market = Market {
        collect_fee_admin,
        manage_oracle_admin: manage_oracle_admin.into(),
        open_orders_admin: open_orders_admin.into(),
        consume_events_admin: consume_events_admin.into(),
        close_market_admin: close_market_admin.into(),
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
        fee_penalty,

        fees_accrued: 0,
        fees_to_referrers: 0,
        vault_signer_nonce: 0,
        base_mint: ctx.accounts.base_mint.key(),
        quote_mint: ctx.accounts.quote_mint.key(),
        base_vault: ctx.accounts.base_vault.key(),
        base_deposit_total: 0,
        base_fees_accrued: 0,
        quote_vault: ctx.accounts.quote_vault.key(),
        quote_deposit_total: 0,
        quote_fees_accrued: 0,
        referrer_rebates_accrued: 0,

        reserved: [0; 1728],
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
