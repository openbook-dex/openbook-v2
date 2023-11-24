use super::*;
use anchor_lang::system_program::{self};
use anchor_spl::token;
use anchor_spl::token::Token;
use anchor_spl::token_interface;
use spl_token_2022::{
    check_spl_token_program_account,
    extension::{transfer_fee::TransferFeeConfig, BaseStateWithExtensions, StateWithExtensions},
    state::Mint,
};

pub fn token_transfer<
    'info,
    P: ToAccountInfo<'info>,
    A: ToAccountInfo<'info>,
    S: ToAccountInfo<'info>,
>(
    token_program: &P,
    from: &A,
    to: &A,
    authority: &S,
    mint: &Option<AccountInfo<'info>>,
    amount_and_decimals: AmountAndDecimals,
) -> Result<()> {
    if amount_and_decimals.amount > 0 {
        if let Some(mint_acc) = mint {
            token_interface::transfer_checked(
                CpiContext::new(
                    token_program.to_account_info(),
                    token_interface::TransferChecked {
                        from: from.to_account_info(),
                        to: to.to_account_info(),
                        authority: authority.to_account_info(),
                        mint: mint_acc.to_account_info(),
                    },
                ),
                amount_and_decimals.amount,
                amount_and_decimals.decimals.unwrap(),
            )
        } else {
            token::transfer(
                CpiContext::new(
                    token_program.to_account_info(),
                    token::Transfer {
                        from: from.to_account_info(),
                        to: to.to_account_info(),
                        authority: authority.to_account_info(),
                    },
                ),
                amount_and_decimals.amount,
            )
        }
    } else {
        Ok(())
    }
}

pub fn token_transfer_signed<
    'info,
    P: ToAccountInfo<'info>,
    A: ToAccountInfo<'info>,
    L: ToAccountInfo<'info>,
>(
    token_program: &P,
    from: &A,
    to: &A,
    authority: &L,
    seeds: &[&[u8]],
    mint: &Option<AccountInfo<'info>>,
    amount_and_decimals: AmountAndDecimals,
) -> Result<()> {
    if amount_and_decimals.amount > 0 {
        if let Some(mint_acc) = mint {
            token_interface::transfer_checked(
                CpiContext::new_with_signer(
                    token_program.to_account_info(),
                    token_interface::TransferChecked {
                        from: from.to_account_info(),
                        to: to.to_account_info(),
                        authority: authority.to_account_info(),
                        mint: mint_acc.to_account_info(),
                    },
                    &[seeds],
                ),
                amount_and_decimals.amount,
                amount_and_decimals.decimals.unwrap(),
            )
        } else {
            token::transfer(
                CpiContext::new_with_signer(
                    token_program.to_account_info(),
                    token::Transfer {
                        from: from.to_account_info(),
                        to: to.to_account_info(),
                        authority: authority.to_account_info(),
                    },
                    &[seeds],
                ),
                amount_and_decimals.amount,
            )
        }
    } else {
        Ok(())
    }
}

pub fn system_program_transfer<
    'info,
    S: ToAccountInfo<'info>,
    A: ToAccountInfo<'info>,
    L: ToAccountInfo<'info>,
>(
    amount: u64,
    system_program: &S,
    from: &A,
    to: &L,
) -> Result<()> {
    if amount > 0 {
        system_program::transfer(
            CpiContext::new(
                system_program.to_account_info(),
                system_program::Transfer {
                    from: from.to_account_info(),
                    to: to.to_account_info(),
                },
            ),
            amount,
        )
    } else {
        Ok(())
    }
}

/// Unpacks a spl_token `Mint` with extension data
pub fn unpack_mint_with_extensions<'info>(
    account_data: &'info [u8],
    owner: &Pubkey,
    token_program_id: &Pubkey,
) -> Result<StateWithExtensions<'info, Mint>> {
    if owner != token_program_id && check_spl_token_program_account(owner).is_err() {
        Err(OpenBookError::SomeError.into())
    } else {
        StateWithExtensions::<Mint>::unpack(account_data)
            .map_err(|_| anchor_lang::error::Error::from(OpenBookError::SomeError))
    }
}

// pub fn get_token_fee
pub fn calculate_amount_with_fee(
    account_info: AccountInfo<'_>,
    token_program: AccountInfo<'_>,
    amount: u64,
) -> Result<Option<u64>> {
    let final_amount = {
        if token_program.key() == Token::id() {
            Some(amount)
        } else {
            let source_data = account_info.data.borrow();
            let source_mint =
                unpack_mint_with_extensions(&source_data, account_info.owner, token_program.key)?;

            if let Ok(transfer_fee_config) = source_mint.get_extension::<TransferFeeConfig>() {
                transfer_fee_config
                    .newer_transfer_fee
                    .calculate_post_fee_amount(amount)
            } else {
                Some(amount)
            }
        }
    };
    Ok(final_amount)
}

pub struct AmountAndDecimals {
    pub amount: u64,
    pub decimals: Option<u8>,
}
