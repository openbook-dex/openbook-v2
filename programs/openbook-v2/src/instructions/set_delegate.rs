use anchor_lang::prelude::*;

use crate::accounts_ix::*;
use crate::error::*;
use crate::state::*;

pub fn set_delegate(ctx: Context<SetDelegate>, limit: u8) -> Result<()> {
    let mut account = ctx.accounts.open_orders_account.load_full_mut()?;

    account.delegate = if let Some(delegate) = ctx.accounts.delegate_account {
        delegate;

    } else {
        ctx.accounts.owner.key()
    };
    Ok(())
}
