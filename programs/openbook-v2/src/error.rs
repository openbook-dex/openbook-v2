use anchor_lang::prelude::*;
use core::fmt::Display;

// todo: group error blocks by kind
// todo: add comments which indicate decimal code for an error
#[error_code]
pub enum OpenBookError {
    #[msg("")]
    SomeError,
    #[msg("The header version is not 1")]
    HeaderVersionNotKnown,
    #[msg("oracle type cannot be determined")]
    UnknownOracleType,
    #[msg("an oracle does not reach the confidence threshold")]
    OracleConfidence,
    #[msg("an oracle is stale")]
    OracleStale,
    #[msg("perp order id not found on the orderbook")]
    OrderIdNotFound,
    #[msg("Event queue contains elements and market can't be closed")]
    EventQueueContainsElements,
    #[msg("Taker fees should be positive and if maker fees are negative, greater or equal to their abs value")]
    InvalidFeesError,
    #[msg("The order type is invalid. A taker order must be Market or ImmediateOrCancel")]
    InvalidOrderType,
    #[msg("would self trade")]
    WouldSelfTrade,
    #[msg("This market does not have a `close_market_admin` and thus cannot be closed.")]
    NoCloseMarketAdmin,
    #[msg("The signer of this transaction is not this market's `close_market_admin`.")]
    InvalidCloseMarketAdmin,
    #[msg("This market requires `open_orders_admin` to sign all instructions that create orders.")]
    MissingOpenOrdersAdmin,
    #[msg("The `open_orders_admin` passed does not match this market's `open_orders_admin`.")]
    InvalidOpenOrdersAdmin,
    #[msg(
        "This market requires `consume_events_admin` to sign all instructions that consume events."
    )]
    MissingConsumeEventsAdmin,
    #[msg(
        "The `consume_events_admin` passed does not match this market's `consume_events_admin`."
    )]
    InvalidConsumeEventsAdmin,
    #[msg("The Market has already expired.")]
    MarketHasExpired,
    #[msg("Lots cannot be negative")]
    NegativeLots,
    #[msg("Price lots should be greater than zero")]
    InvalidPriceLots,
    #[msg("Peg limit should be greater than zero")]
    InvalidPegLimit,
    #[msg("Order size above market limits")]
    InvalidOrderSize,
    #[msg("Oracle staleness limit is currently unimplemented")]
    Unimplemented,
}

impl From<OpenBookError> for ProgramError {
    fn from(error: OpenBookError) -> Self {
        ProgramError::from(Error::from(error))
    }
}

impl OpenBookError {
    pub fn error_code(&self) -> u32 {
        (*self).into()
    }
}

pub trait IsAnchorErrorWithCode {
    fn is_anchor_error_with_code(&self, code: u32) -> bool;
}

impl<T> IsAnchorErrorWithCode for anchor_lang::Result<T> {
    fn is_anchor_error_with_code(&self, code: u32) -> bool {
        match self {
            Err(Error::AnchorError(error)) => error.error_code_number == code,
            _ => false,
        }
    }
}

pub trait Contextable {
    /// Add a context string `c` to a Result or Error
    ///
    /// Example: foo().context("calling foo")?;
    fn context(self, c: impl Display) -> Self;

    /// Like `context()`, but evaluate the context string lazily
    ///
    /// Use this if it's expensive to generate, like a format!() call.
    fn with_context<C, F>(self, c: F) -> Self
    where
        C: Display,
        F: FnOnce() -> C;
}

impl Contextable for Error {
    fn context(self, c: impl Display) -> Self {
        match self {
            Error::AnchorError(err) => Error::AnchorError(AnchorError {
                error_msg: if err.error_msg.is_empty() {
                    format!("{}", c)
                } else {
                    format!("{}; {}", err.error_msg, c)
                },
                ..err
            }),
            // Maybe wrap somehow?
            Error::ProgramError(err) => Error::ProgramError(err),
        }
    }
    fn with_context<C, F>(self, c: F) -> Self
    where
        C: Display,
        F: FnOnce() -> C,
    {
        self.context(c())
    }
}

impl<T> Contextable for Result<T> {
    fn context(self, c: impl Display) -> Self {
        if let Err(err) = self {
            Err(err.context(c))
        } else {
            self
        }
    }
    fn with_context<C, F>(self, c: F) -> Self
    where
        C: Display,
        F: FnOnce() -> C,
    {
        if let Err(err) = self {
            Err(err.context(c()))
        } else {
            self
        }
    }
}

/// Creates an Error with a particular message, using format!() style arguments
///
/// Example: error_msg!("index {} not found", index)
#[macro_export]
macro_rules! error_msg {
    ($($arg:tt)*) => {
        error!(OpenBookError::SomeError).context(format!($($arg)*))
    };
}

/// Creates an Error with a particular message, using format!() style arguments
///
/// Example: error_msg_typed!(TokenPositionMissing, "index {} not found", index)
#[macro_export]
macro_rules! error_msg_typed {
    ($code:expr, $($arg:tt)*) => {
        error!($code).context(format!($($arg)*))
    };
}

/// Like anchor's require!(), but with a customizable message
///
/// Example: require_msg!(condition, "the condition on account {} was violated", account_key);
#[macro_export]
macro_rules! require_msg {
    ($invariant:expr, $($arg:tt)*) => {
        if !($invariant) {
            return Err(error_msg!($($arg)*));
        }
    };
}

/// Like anchor's require!(), but with a customizable message and type
///
/// Example: require_msg_typed!(condition, "the condition on account {} was violated", account_key);
#[macro_export]
macro_rules! require_msg_typed {
    ($invariant:expr, $code:expr, $($arg:tt)*) => {
        if !($invariant) {
            return Err(error_msg_typed!($code, $($arg)*));
        }
    };
}

pub use error_msg;
pub use error_msg_typed;
pub use require_msg;
pub use require_msg_typed;
