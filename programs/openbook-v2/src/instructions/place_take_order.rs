use anchor_lang::prelude::*;

use crate::accounts_ix::*;
use crate::accounts_zerocopy::AccountInfoRef;
use crate::error::*;
use crate::state::*;
use crate::token_utils::*;
use anchor_spl::token_interface::Mint;

#[allow(clippy::too_many_arguments)]
pub fn place_take_order<'info>(
    ctx: Context<'_, '_, '_, 'info, PlaceTakeOrder<'info>>,
    order: Order,
    limit: u8,
) -> Result<()> {
    require_gte!(order.max_base_lots, 0, OpenBookError::InvalidInputLots);
    require_gte!(
        order.max_quote_lots_including_fees,
        0,
        OpenBookError::InvalidInputLots
    );

    let clock = Clock::get()?;

    let remaining_accounts = ctx.remaining_accounts;

    let mut market = ctx.accounts.market.load_mut()?;
    require!(
        !market.is_expired(clock.unix_timestamp),
        OpenBookError::MarketHasExpired
    );

    let mut book = Orderbook {
        bids: ctx.accounts.bids.load_mut()?,
        asks: ctx.accounts.asks.load_mut()?,
    };

    let mut event_heap = ctx.accounts.event_heap.load_mut()?;
    let event_heap_size_before = event_heap.len();

    let now_ts: u64 = clock.unix_timestamp.try_into().unwrap();

    let oracle_price = market.oracle_price(
        AccountInfoRef::borrow_some(ctx.accounts.oracle_a.as_ref())?.as_ref(),
        AccountInfoRef::borrow_some(ctx.accounts.oracle_b.as_ref())?.as_ref(),
        clock.slot,
    )?;

    let side = order.side;

    let OrderWithAmounts {
        total_base_taken_native,
        total_quote_taken_native,
        referrer_amount,
        taker_fees,
        ..
    } = book.new_order(
        &order,
        &mut market,
        &mut event_heap,
        oracle_price,
        None,
        &ctx.accounts.signer.key(),
        now_ts,
        limit,
        ctx.remaining_accounts,
    )?;

    let (deposit_amount, withdraw_amount) = match side {
        Side::Bid => {
            let total_quote_including_fees = total_quote_taken_native + taker_fees;
            market.base_deposit_total -= total_base_taken_native;
            market.quote_deposit_total += total_quote_including_fees;
            (total_quote_including_fees, total_base_taken_native)
        }
        Side::Ask => {
            let total_quote_discounting_fees = total_quote_taken_native - taker_fees;
            market.base_deposit_total += total_base_taken_native;
            market.quote_deposit_total -= total_quote_discounting_fees;
            (total_base_taken_native, total_quote_discounting_fees)
        }
    };

    if ctx.accounts.referrer_account.is_some() {
        market.fees_to_referrers += referrer_amount as u128;
        market.quote_deposit_total -= referrer_amount;
    } else {
        market.fees_available += referrer_amount;
    }

    let seeds = market_seeds!(market, ctx.accounts.market.key());

    drop(market);

    if event_heap.len() > event_heap_size_before {
        system_program_transfer(
            PENALTY_EVENT_HEAP,
            &ctx.accounts.system_program,
            &ctx.accounts.signer,
            &ctx.accounts.market,
        )?;
    }

    let (user_deposit_acc, user_withdraw_acc, market_deposit_acc, market_withdraw_acc) = match side
    {
        Side::Bid => (
            &ctx.accounts.user_quote_account,
            &ctx.accounts.user_base_account,
            &ctx.accounts.market_quote_vault,
            &ctx.accounts.market_base_vault,
        ),
        Side::Ask => (
            &ctx.accounts.user_base_account,
            &ctx.accounts.user_quote_account,
            &ctx.accounts.market_base_vault,
            &ctx.accounts.market_quote_vault,
        ),
    };

    let deposit_token_fee_wrapped = {
        get_token_fee(remaining_accounts[0].to_account_info(), ctx.accounts.token_program.to_account_info(), deposit_amount)
    };
    let deposit_token_fee = deposit_token_fee_wrapped.unwrap().unwrap();

    let deposit_actual_amount = deposit_amount + deposit_token_fee;

    let deposit_data = &mut &**remaining_accounts[0].try_borrow_data()?;
    let deposit_mint = Mint::try_deserialize(deposit_data).unwrap();
    let deposit_decimals = deposit_mint.decimals;


    let withdraw_token_fee_wrapped = {
        get_token_fee(remaining_accounts[1].to_account_info(), ctx.accounts.token_program.to_account_info(), withdraw_amount)
    };
    let withdraw_token_fee = withdraw_token_fee_wrapped.unwrap().unwrap();

    let withdraw_actual_amount = withdraw_amount - withdraw_token_fee;

    let withdraw_data = &mut &**remaining_accounts[1].try_borrow_data()?;
    let withdraw_mint = Mint::try_deserialize(withdraw_data).unwrap();
    let withdraw_decimals = withdraw_mint.decimals;

    
    if &market_deposit_acc.mint == &remaining_accounts[0].key() {
        token_transfer(
            deposit_actual_amount,
            &ctx.accounts.token_program,
            user_deposit_acc.as_ref(),
            market_deposit_acc,
            &ctx.accounts.signer,
            remaining_accounts[0].to_account_info(),
            deposit_decimals,
        )?;

        token_transfer_signed(
            withdraw_actual_amount,
            &ctx.accounts.token_program,
            market_withdraw_acc,
            user_withdraw_acc.as_ref(),
            &ctx.accounts.market_authority,
            seeds,
            remaining_accounts[1].to_account_info(),
            withdraw_decimals,
        )?;
        
    } else if &market_deposit_acc.mint == &remaining_accounts[1].key() {
        token_transfer(
            deposit_actual_amount,
            &ctx.accounts.token_program,
            user_deposit_acc.as_ref(),
            market_deposit_acc,
            &ctx.accounts.signer,
            remaining_accounts[1].to_account_info(),
            deposit_decimals,
        )?;

        token_transfer_signed(
            withdraw_actual_amount,
            &ctx.accounts.token_program,
            market_withdraw_acc,
            user_withdraw_acc.as_ref(),
            &ctx.accounts.market_authority,
            seeds,
            remaining_accounts[0].to_account_info(),
            withdraw_decimals,
        )?;
    }

    if let Some(referrer_account) = &ctx.accounts.referrer_account {

        // let referrer_token_account_info = remaining_accounts[1]; // referrer token mint

        let referrer_token_fee_wrapped = {
            get_token_fee(remaining_accounts[1].to_account_info(), ctx.accounts.token_program.to_account_info(), referrer_amount)
        };
        let referrer_token_fee = referrer_token_fee_wrapped.unwrap().unwrap();

        let referrer_actual_amount = referrer_amount - referrer_token_fee;

        let referrer_data = &mut &**remaining_accounts[1].try_borrow_data()?;
        let referrer_mint = Mint::try_deserialize(referrer_data).unwrap();
        let referrer_decimals = referrer_mint.decimals;

        if &referrer_account.mint == &remaining_accounts[0].key() {
            token_transfer_signed(
                referrer_actual_amount,
                &ctx.accounts.token_program,
                &ctx.accounts.market_quote_vault,
                referrer_account,
                &ctx.accounts.market_authority,
                seeds,
                remaining_accounts[0].to_account_info(),
                referrer_decimals,
            )?;
            
        } else if &referrer_account.mint == &remaining_accounts[1].key() {
            token_transfer_signed(
                referrer_actual_amount,
                &ctx.accounts.token_program,
                &ctx.accounts.market_quote_vault,
                referrer_account,
                &ctx.accounts.market_authority,
                seeds,
                remaining_accounts[1].to_account_info(),
                referrer_decimals,
            )?;
        }
    }

    Ok(())
}
