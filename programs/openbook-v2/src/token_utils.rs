use super::*;
use anchor_lang::system_program;
use anchor_spl::token_2022;
use anchor_spl::token;
use anchor_spl::token_interface;
use spl_token_2022::{
    processor::Processor,
    check_spl_token_program_account,
    error::TokenError,
    extension::{
        BaseStateWithExtensions, mint_close_authority::MintCloseAuthority,
        StateWithExtensions, transfer_fee::TransferFeeConfig,
    },
    state::{Account, Mint},
};

// How do these P, A, S work??
pub fn token_transfer<
    'info,
    P: ToAccountInfo<'info>,
    A: ToAccountInfo<'info>,
    S: ToAccountInfo<'info>,
>(
    amount: u64,
    token_program: &P,
    from: &A,
    to: &A,
    authority: &S,
    mint: AccountInfo<'info>,
    decimals: u8,
) -> Result<()> {
    if amount > 0 {
        token_interface::transfer_checked(
            CpiContext::new(
                token_program.to_account_info(),
                token_interface::TransferChecked {
                    from: from.to_account_info(),
                    to: to.to_account_info(),
                    authority: authority.to_account_info(),
                    mint: mint.to_account_info(),
                },
            ),
            amount, decimals
        )
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
    amount: u64,
    token_program: &P,
    from: &A,
    to: &A,
    authority: &L,
    seeds: &[&[u8]],
    mint: AccountInfo<'info>,
    decimals: u8,
) -> Result<()> {
    if amount > 0 {
        token_2022::transfer_checked(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                token_2022::TransferChecked {
                    from: from.to_account_info(),
                    to: to.to_account_info(),
                    authority: authority.to_account_info(),
                    mint: mint.to_account_info(),
                },
                &[seeds],
            ),
            amount, decimals
        )
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
pub fn unpack_mint_with_extensions<'a>(
    account_data: &'a [u8],
    owner: &Pubkey,
    token_program_id: &Pubkey,
) -> Result<StateWithExtensions<'a, Mint>> {
    if owner != token_program_id && check_spl_token_program_account(owner).is_err() {
        Err(OpenBookError::SomeError.into())
    } else {
        StateWithExtensions::<Mint>::unpack(account_data).map_err(|_| OpenBookError::SomeError)
    }
}

// pub fn get_token_fee
pub fn get_token_fee<
    'info,
>(
    account_info: AccountInfo<'_>,
    token_program: AccountInfo<'_>,
    amount: u64,
) -> Result<Option<u64>> {
    let token_fee = {
        let source_data = account_info.data.borrow();
        let source_mint = unpack_mint_with_extensions(
            &source_data,
            account_info.owner,
            token_program.key,
        )?;

        let Ok(transfer_fee_config) = source_mint.get_extension::<TransferFeeConfig>(); 
        let transfer_fee = transfer_fee_config
                .calculate_epoch_fee(Clock::get()?.epoch, amount);
        transfer_fee
    };
    Ok(token_fee)
}