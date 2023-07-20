use anchor_lang::prelude::*;

use crate::error::*;
use crate::pubkey_option::NonZeroPubkeyOption;
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
    maker_fee: i64,
    taker_fee: i64,
    time_expiry: i64,
) -> Result<()> {
    let now_ts: u64 = Clock::get()?.unix_timestamp.try_into().unwrap();

    require!(
        maker_fee.unsigned_abs() as i128 <= FEES_SCALE_FACTOR,
        OpenBookError::InvalidInputMarketFees
    );
    require!(
        taker_fee.unsigned_abs() as i128 <= FEES_SCALE_FACTOR,
        OpenBookError::InvalidInputMarketFees
    );
    require!(
        taker_fee >= 0 && (maker_fee >= 0 || maker_fee.abs() <= taker_fee),
        OpenBookError::InvalidInputMarketFees
    );

    require!(
        time_expiry == 0 || time_expiry > Clock::get()?.unix_timestamp,
        OpenBookError::InvalidInputMarketExpired
    );

    require_gt!(quote_lot_size, 0, OpenBookError::InvalidInputLots);
    require_gt!(base_lot_size, 0, OpenBookError::InvalidInputLots);

    let open_orders_admin: NonZeroPubkeyOption = ctx
        .accounts
        .open_orders_admin
        .as_ref()
        .map(|account| account.key())
        .into();

    let consume_events_admin: NonZeroPubkeyOption = ctx
        .accounts
        .consume_events_admin
        .as_ref()
        .map(|account| account.key())
        .into();

    let close_market_admin: NonZeroPubkeyOption = ctx
        .accounts
        .close_market_admin
        .as_ref()
        .map(|account| account.key())
        .into();

    let oracle_a: NonZeroPubkeyOption = ctx
        .accounts
        .oracle_a
        .as_ref()
        .map(|account| account.key())
        .into();

    let oracle_b: NonZeroPubkeyOption = ctx
        .accounts
        .oracle_b
        .as_ref()
        .map(|account| account.key())
        .into();

    let mut openbook_market = ctx.accounts.market.load_init()?;
    *openbook_market = Market {
        signer_creator: ctx.accounts.payer.key(),
        collect_fee_admin: ctx.accounts.collect_fee_admin.key(),
        open_orders_admin,
        consume_events_admin,
        close_market_admin,
        market_index,
        bump: *ctx.bumps.get("market").ok_or(OpenBookError::SomeError)?,
        base_decimals: ctx.accounts.base_mint.decimals,
        quote_decimals: ctx.accounts.quote_mint.decimals,
        padding1: Default::default(),
        time_expiry,
        name: fill_from_str(&name)?,
        bids: ctx.accounts.bids.key(),
        asks: ctx.accounts.asks.key(),
        event_queue: ctx.accounts.event_queue.key(),
        oracle_a,
        oracle_b,
        oracle_config: oracle_config.to_oracle_config(),
        quote_lot_size,
        base_lot_size,
        seq_num: 0,
        registration_time: now_ts,
        maker_fee,
        taker_fee,
        fees_accrued: 0,
        fees_to_referrers: 0,
        taker_volume_wo_oo: 0,
        base_mint: ctx.accounts.base_mint.key(),
        quote_mint: ctx.accounts.quote_mint.key(),
        base_vault: ctx.accounts.base_vault.key(),
        base_deposit_total: 0,
        quote_vault: ctx.accounts.quote_vault.key(),
        quote_deposit_total: 0,
        quote_fees_accrued: 0,
        referrer_rebates_accrued: 0,

        reserved: [0; 1768],
    };

    let mut orderbook = Orderbook {
        bids: ctx.accounts.bids.load_init()?,
        asks: ctx.accounts.asks.load_init()?,
    };
    orderbook.init();

    let mut event_queue = ctx.accounts.event_queue.load_init()?;
    event_queue.init();

    emit!(MarketMetaDataLog {
        market: ctx.accounts.market.key(),
        market_index,
        base_decimals: ctx.accounts.base_mint.decimals,
        quote_decimals: ctx.accounts.quote_mint.decimals,
        base_lot_size,
        quote_lot_size,
        oracle_a: oracle_a.into(),
        oracle_b: oracle_b.into(),
    });

    Ok(())
}
