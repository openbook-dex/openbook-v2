use anchor_lang::prelude::*;

#[error_code]
#[derive(Eq, PartialEq)]
pub enum SwitchboardError {
    #[msg("Aggregator is not currently populated with a valid round")]
    InvalidAggregatorRound,
    #[msg("Failed to convert string to decimal format")]
    InvalidStrDecimalConversion,
    #[msg("Decimal conversion method failed")]
    DecimalConversionError,
    #[msg("An integer overflow occurred")]
    IntegerOverflowError,
    #[msg("Account discriminator did not match")]
    AccountDiscriminatorMismatch,
    #[msg("Vrf value is empty")]
    VrfEmptyError,
    #[msg("Failed to send requestRandomness instruction")]
    VrfCpiError,
    #[msg("Failed to send signed requestRandomness instruction")]
    VrfCpiSignedError,
    #[msg("Failed to deserialize account")]
    AccountDeserializationError,
    #[msg("Switchboard feed exceeded the staleness threshold")]
    StaleFeed,
    #[msg("Switchboard feed exceeded the confidence interval threshold")]
    ConfidenceIntervalExceeded,
    #[msg("Invalid authority provided to Switchboard account")]
    InvalidAuthority,
    #[msg("Switchboard value variance exceeded threshold")]
    AllowedVarianceExceeded,
    #[msg("Invalid function input")]
    InvalidFunctionInput,
}
