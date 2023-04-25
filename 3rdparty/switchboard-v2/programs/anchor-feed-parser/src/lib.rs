use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock;
use std::convert::TryInto;
pub use switchboard_v2::{AggregatorAccountData, SwitchboardDecimal, SWITCHBOARD_PROGRAM_ID};

declare_id!("5cjAZXFoY4KN1Sq35ZYE72fGdVy8sy5YGpNix1N4rtHY");

#[derive(Accounts)]
#[instruction(params: ReadResultParams)]
pub struct ReadResult<'info> {
    pub aggregator: AccountLoader<'info, AggregatorAccountData>,
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct ReadResultParams {
    pub max_confidence_interval: Option<f64>,
}

#[program]
pub mod anchor_feed_parser {
    use super::*;

    pub fn read_result(
        ctx: Context<ReadResult>,
        params: ReadResultParams,
    ) -> anchor_lang::Result<()> {
        let feed = &ctx.accounts.aggregator.load()?;

        // get result
        let val: f64 = feed.get_result()?.try_into()?;

        // check whether the feed has been updated in the last 300 seconds
        feed.check_staleness(clock::Clock::get().unwrap().unix_timestamp, 300)
            .map_err(|_| error!(FeedErrorCode::StaleFeed))?;

        // check feed does not exceed max_confidence_interval
        if let Some(max_confidence_interval) = params.max_confidence_interval {
            feed.check_confidence_interval(SwitchboardDecimal::from_f64(max_confidence_interval))
                .map_err(|_| error!(FeedErrorCode::ConfidenceIntervalExceeded))?;
        }

        msg!("Current feed result is {}!", val);

        Ok(())
    }
}

#[error_code]
#[derive(Eq, PartialEq)]
pub enum FeedErrorCode {
    #[msg("Not a valid Switchboard account")]
    InvalidSwitchboardAccount,
    #[msg("Switchboard feed has not been updated in 5 minutes")]
    StaleFeed,
    #[msg("Switchboard feed exceeded provided confidence interval")]
    ConfidenceIntervalExceeded,
}
