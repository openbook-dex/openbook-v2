use anchor_lang::prelude::*;

use crate::accounts_ix::*;
use crate::accounts_zerocopy::*;
use crate::error::*;
use crate::logs::MarketMetaDataLog;
use crate::pubkey_option::NonZeroKey;
use crate::state::*;
use crate::util::fill_from_str;

#[allow(clippy::too_many_arguments)]
pub fn create_market(
    ctx: Context<CreateMarket>,
    name: String,
    oracle_config: OracleConfigParams,
    quote_lot_size: i64,
    base_lot_size: i64,
    maker_fee: i64,
    taker_fee: i64,
    time_expiry: i64,
) -> Result<()> {
    let registration_time = Clock::get()?.unix_timestamp;

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

    let oracle_a = ctx.accounts.oracle_a.non_zero_key();
    let oracle_b = ctx.accounts.oracle_b.non_zero_key();

    if oracle_a.is_some() && oracle_b.is_some() {
        let oracle_a = AccountInfoRef::borrow(ctx.accounts.oracle_a.as_ref().unwrap())?;
        let oracle_b = AccountInfoRef::borrow(ctx.accounts.oracle_b.as_ref().unwrap())?;

        require_keys_neq!(*oracle_a.key, *oracle_b.key);
        require!(
            oracle::determine_oracle_type(&oracle_a)? == oracle::determine_oracle_type(&oracle_b)?,
            OpenBookError::InvalidOracleTypes
        );
    } else if oracle_a.is_some() {
        let oracle_a = AccountInfoRef::borrow(ctx.accounts.oracle_a.as_ref().unwrap())?;
        oracle::determine_oracle_type(&oracle_a)?;
    } else if oracle_b.is_some() {
        return Err(OpenBookError::InvalidSecondOracle.into());
    }

    let mut openbook_market = ctx.accounts.market.load_init()?;
    *openbook_market = Market {
        market_authority: ctx.accounts.market_authority.key(),
        collect_fee_admin: ctx.accounts.collect_fee_admin.key(),
        open_orders_admin: ctx.accounts.open_orders_admin.non_zero_key(),
        consume_events_admin: ctx.accounts.consume_events_admin.non_zero_key(),
        close_market_admin: ctx.accounts.close_market_admin.non_zero_key(),
        bump: ctx.bumps.market_authority,
        base_decimals: ctx.accounts.base_mint.decimals,
        quote_decimals: ctx.accounts.quote_mint.decimals,
        padding1: Default::default(),
        time_expiry,
        name: fill_from_str(&name)?,
        bids: ctx.accounts.bids.key(),
        asks: ctx.accounts.asks.key(),
        event_heap: ctx.accounts.event_heap.key(),
        oracle_a,
        oracle_b,
        oracle_config: oracle_config.to_oracle_config(),
        quote_lot_size,
        base_lot_size,
        seq_num: 0,
        registration_time,
        maker_fee,
        taker_fee,
        fees_accrued: 0,
        fees_to_referrers: 0,
        maker_volume: 0,
        taker_volume_wo_oo: 0,
        base_mint: ctx.accounts.base_mint.key(),
        quote_mint: ctx.accounts.quote_mint.key(),
        market_base_vault: ctx.accounts.market_base_vault.key(),
        base_deposit_total: 0,
        market_quote_vault: ctx.accounts.market_quote_vault.key(),
        quote_deposit_total: 0,
        fees_available: 0,
        referrer_rebates_accrued: 0,

        reserved: [0; 128],
    };

    let mut orderbook = Orderbook {
        bids: ctx.accounts.bids.load_init()?,
        asks: ctx.accounts.asks.load_init()?,
    };
    orderbook.init();

    let mut event_heap = ctx.accounts.event_heap.load_init()?;
    event_heap.init();

    emit_cpi!(MarketMetaDataLog {
        market: ctx.accounts.market.key(),
        name,
        base_mint: ctx.accounts.base_mint.key(),
        quote_mint: ctx.accounts.quote_mint.key(),
        base_decimals: ctx.accounts.base_mint.decimals,
        quote_decimals: ctx.accounts.quote_mint.decimals,
        base_lot_size,
        quote_lot_size,
    });

    Ok(())
}
