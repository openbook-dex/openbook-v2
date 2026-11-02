use anchor_lang::prelude::*;
use anchor_spl::associated_token::{self, get_associated_token_address};

use crate::accounts_ix::*;
use crate::accounts_zerocopy::LoadZeroCopyRef;
use crate::error::*;
use crate::instructions::settle_funds_inner;
use crate::logs::{emit_stack, SettleFundsLog};
use crate::state::*;

pub fn janitor_close_open_orders_account<'info>(
    ctx: Context<'_, '_, '_, 'info, JanitorCloseOpenOrdersAccount<'info>>,
) -> Result<()> {
    let accounts = &ctx.accounts;
    let mut open_orders_account = accounts.open_orders_account.load_mut()?;

    // Orphaned accounts point at a market that was already closed: there is no book to cancel
    // on and no vault to settle from, closing only returns the rent.
    if accounts.market.owner == ctx.program_id {
        let position = &open_orders_account.position;
        require!(
            position.bids_base_lots == 0 && position.asks_base_lots == 0,
            OpenBookError::OpenOrdersNotCancelled
        );

        {
            let market = accounts.market.load::<Market>()?;
            require_keys_eq!(accounts.market_authority.key(), market.market_authority);
            require_keys_eq!(
                accounts.market_base_vault.key(),
                market.market_base_vault,
                OpenBookError::InvalidMarketVault
            );
            require_keys_eq!(
                accounts.market_quote_vault.key(),
                market.market_quote_vault,
                OpenBookError::InvalidMarketVault
            );
            require_keys_eq!(accounts.base_mint.key(), market.base_mint);
            require_keys_eq!(accounts.quote_mint.key(), market.quote_mint);
        }

        let position = &open_orders_account.position;
        create_owner_ata_if_needed(
            accounts,
            &accounts.owner_base_ata,
            &accounts.base_mint,
            position.base_free_native,
        )?;
        create_owner_ata_if_needed(
            accounts,
            &accounts.owner_quote_ata,
            &accounts.quote_mint,
            position.quote_free_native,
        )?;

        let (base_native, quote_native, referrer_rebate) = settle_funds_inner(
            &mut open_orders_account,
            &accounts.market,
            &accounts.market_authority,
            &accounts.market_base_vault,
            &accounts.market_quote_vault,
            &accounts.owner_base_ata,
            &accounts.owner_quote_ata,
            None,
            &accounts.janitor,
            &accounts.token_program,
            &accounts.system_program,
        )?;

        emit_stack(SettleFundsLog {
            open_orders_account: accounts.open_orders_account.key(),
            base_native,
            quote_native,
            referrer_rebate,
            referrer: None,
        });
    }
    drop(open_orders_account);

    // No realloc: the indexer's lamports go to the owner once it holds no more accounts.
    let open_orders_account_key = accounts.open_orders_account.key();
    let indexer = &mut ctx.accounts.open_orders_indexer;
    if let Some(index) = indexer
        .addresses
        .iter()
        .position(|x| *x == open_orders_account_key)
    {
        indexer.addresses.remove(index);
    }

    // The owner is known here through the indexer seeds: an indexer left empty is closed into
    // the owner along with the account, without a fee. Anchor does not write back an account that
    // was closed.
    let indexer_empty = !indexer.has_active_open_orders_accounts();
    let indexer_info = ctx.accounts.open_orders_indexer.to_account_info();
    let empty_indexer = indexer_empty.then_some(&indexer_info);
    close_with_reward(
        &ctx.accounts.open_orders_account.to_account_info(),
        empty_indexer,
        &ctx.accounts.owner,
        &ctx.accounts.janitor,
        JANITOR_OPEN_ORDERS_ACCOUNT_REWARD,
        &Rent::get()?,
    )?;

    Ok(())
}

/// Settlement always goes to the owner's associated token account, checked whatever the amount
/// so that it does not depend on what the settlement later transfers. The janitor pays for the
/// account when it does not exist yet and there is something to settle.
fn create_owner_ata_if_needed<'info>(
    accounts: &JanitorCloseOpenOrdersAccount<'info>,
    ata: &AccountInfo<'info>,
    mint: &AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    require_keys_eq!(
        ata.key(),
        get_associated_token_address(&accounts.owner.key(), &mint.key()),
        OpenBookError::InvalidOwnerTokenAccount
    );

    if amount == 0 {
        return Ok(());
    }

    associated_token::create_idempotent(CpiContext::new(
        accounts.associated_token_program.to_account_info(),
        associated_token::Create {
            payer: accounts.janitor.to_account_info(),
            associated_token: ata.clone(),
            authority: accounts.owner.to_account_info(),
            mint: mint.clone(),
            system_program: accounts.system_program.to_account_info(),
            token_program: accounts.token_program.to_account_info(),
        },
    ))
}
