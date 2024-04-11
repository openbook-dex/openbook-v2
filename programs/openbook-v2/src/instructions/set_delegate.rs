use anchor_lang::prelude::*;

use crate::accounts_ix::*;
use crate::logs::{emit_stack, SetDelegateLog};
use crate::pubkey_option::NonZeroPubkeyOption;

pub fn set_delegate(ctx: Context<SetDelegate>) -> Result<()> {
    let mut account = ctx.accounts.open_orders_account.load_mut()?;

    let delegate_account: NonZeroPubkeyOption = ctx
        .accounts
        .delegate_account
        .as_ref()
        .map(|account| account.key())
        .into();

    account.delegate = delegate_account;

    emit_stack(SetDelegateLog {
        open_orders_account: ctx.accounts.open_orders_account.key(),
        delegate: delegate_account.into(),
    });

    Ok(())
}
