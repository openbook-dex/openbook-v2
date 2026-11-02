use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount};

use crate::accounts_ix::*;
use crate::error::*;
use crate::state::*;

pub fn janitor_close_market(ctx: Context<JanitorCloseMarket>) -> Result<()> {
    // checked in wind-down order: crank, cancel, close open orders accounts, sweep fees
    let event_heap = ctx.accounts.event_heap.load()?;
    require!(
        event_heap.is_empty(),
        OpenBookError::EventHeapContainsElements
    );
    drop(event_heap);

    require!(
        ctx.accounts.bids.load()?.is_empty() && ctx.accounts.asks.load()?.is_empty(),
        OpenBookError::BookContainsElements
    );

    let market = ctx.accounts.market.load()?;
    // Zero deposit totals prove that no open orders account still has a claim on the vaults,
    // zero fees that they were swept. Events written off for closed accounts leave the fee
    // roundoff of their vanished maker behind, see `orphan_fee_roundoff`.
    require!(janitor_market_empty(&market), OpenBookError::NonEmptyMarket);

    let seeds = market_seeds!(market, ctx.accounts.market.key());
    drop(market);

    burn_and_close_vault(
        ctx.accounts,
        &ctx.accounts.market_base_vault,
        &ctx.accounts.base_mint,
        seeds,
    )?;
    burn_and_close_vault(
        ctx.accounts,
        &ctx.accounts.market_quote_vault,
        &ctx.accounts.quote_mint,
        seeds,
    )?;

    // market, bids, asks and event heap are closed into the escrow by anchor
    Ok(())
}

/// Burn whatever is left in the vault (amounts written off for orphaned events, dust, donations:
/// nobody has a claim on it and nobody should gain from it) and close it, rent to the escrow.
/// Wrapped SOL can not be burned, the token program refuses it for native accounts; closing the
/// vault moves it to the escrow with the rent instead.
fn burn_and_close_vault<'info>(
    accounts: &JanitorCloseMarket<'info>,
    vault: &Account<'info, TokenAccount>,
    mint: &AccountInfo<'info>,
    seeds: &[&[u8]],
) -> Result<()> {
    if vault.amount > 0 && vault.is_native.is_none() {
        token::burn(
            CpiContext::new_with_signer(
                accounts.token_program.to_account_info(),
                token::Burn {
                    mint: mint.clone(),
                    from: vault.to_account_info(),
                    authority: accounts.market_authority.to_account_info(),
                },
                &[seeds],
            ),
            vault.amount,
        )?;
    }

    token::close_account(CpiContext::new_with_signer(
        accounts.token_program.to_account_info(),
        token::CloseAccount {
            account: vault.to_account_info(),
            destination: accounts.escrow.to_account_info(),
            authority: accounts.market_authority.to_account_info(),
        },
        &[seeds],
    ))
}
