import typing
from anchorpy.error import ProgramError


class ArrayOperationError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6000, "Illegal operation on a Switchboard array.")

    code = 6000
    name = "ArrayOperationError"
    msg = "Illegal operation on a Switchboard array."


class QueueOperationError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6001, "Illegal operation on a Switchboard queue.")

    code = 6001
    name = "QueueOperationError"
    msg = "Illegal operation on a Switchboard queue."


class IncorrectProgramOwnerError(ProgramError):
    def __init__(self) -> None:
        super().__init__(
            6002,
            "An account required to be owned by the program has a different owner.",
        )

    code = 6002
    name = "IncorrectProgramOwnerError"
    msg = "An account required to be owned by the program has a different owner."


class InvalidAggregatorRound(ProgramError):
    def __init__(self) -> None:
        super().__init__(
            6003, "Aggregator is not currently populated with a valid round."
        )

    code = 6003
    name = "InvalidAggregatorRound"
    msg = "Aggregator is not currently populated with a valid round."


class TooManyAggregatorJobs(ProgramError):
    def __init__(self) -> None:
        super().__init__(6004, "Aggregator cannot fit any more jobs.")

    code = 6004
    name = "TooManyAggregatorJobs"
    msg = "Aggregator cannot fit any more jobs."


class AggregatorCurrentRoundClosed(ProgramError):
    def __init__(self) -> None:
        super().__init__(
            6005, "Aggregator's current round is closed. No results are being accepted."
        )

    code = 6005
    name = "AggregatorCurrentRoundClosed"
    msg = "Aggregator's current round is closed. No results are being accepted."


class AggregatorInvalidSaveResult(ProgramError):
    def __init__(self) -> None:
        super().__init__(
            6006, "Aggregator received an invalid save result instruction."
        )

    code = 6006
    name = "AggregatorInvalidSaveResult"
    msg = "Aggregator received an invalid save result instruction."


class InvalidStrDecimalConversion(ProgramError):
    def __init__(self) -> None:
        super().__init__(6007, "Failed to convert string to decimal format.")

    code = 6007
    name = "InvalidStrDecimalConversion"
    msg = "Failed to convert string to decimal format."


class AccountLoaderMissingSignature(ProgramError):
    def __init__(self) -> None:
        super().__init__(6008, "AccountLoader account is missing a required signature.")

    code = 6008
    name = "AccountLoaderMissingSignature"
    msg = "AccountLoader account is missing a required signature."


class MissingRequiredSignature(ProgramError):
    def __init__(self) -> None:
        super().__init__(6009, "Account is missing a required signature.")

    code = 6009
    name = "MissingRequiredSignature"
    msg = "Account is missing a required signature."


class ArrayOverflowError(ProgramError):
    def __init__(self) -> None:
        super().__init__(
            6010, "The attempted action will overflow a zero-copy account array."
        )

    code = 6010
    name = "ArrayOverflowError"
    msg = "The attempted action will overflow a zero-copy account array."


class ArrayUnderflowError(ProgramError):
    def __init__(self) -> None:
        super().__init__(
            6011, "The attempted action will underflow a zero-copy account array."
        )

    code = 6011
    name = "ArrayUnderflowError"
    msg = "The attempted action will underflow a zero-copy account array."


class PubkeyNotFoundError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6012, "The queried public key was not found.")

    code = 6012
    name = "PubkeyNotFoundError"
    msg = "The queried public key was not found."


class AggregatorIllegalRoundOpenCall(ProgramError):
    def __init__(self) -> None:
        super().__init__(6013, "Aggregator round open called too early.")

    code = 6013
    name = "AggregatorIllegalRoundOpenCall"
    msg = "Aggregator round open called too early."


class AggregatorIllegalRoundCloseCall(ProgramError):
    def __init__(self) -> None:
        super().__init__(6014, "Aggregator round close called too early.")

    code = 6014
    name = "AggregatorIllegalRoundCloseCall"
    msg = "Aggregator round close called too early."


class AggregatorClosedError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6015, "Aggregator is closed. Illegal action.")

    code = 6015
    name = "AggregatorClosedError"
    msg = "Aggregator is closed. Illegal action."


class IllegalOracleIdxError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6016, "Illegal oracle index.")

    code = 6016
    name = "IllegalOracleIdxError"
    msg = "Illegal oracle index."


class OracleAlreadyRespondedError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6017, "The provided oracle has already responded this round.")

    code = 6017
    name = "OracleAlreadyRespondedError"
    msg = "The provided oracle has already responded this round."


class ProtoDeserializeError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6018, "Failed to deserialize protocol buffer.")

    code = 6018
    name = "ProtoDeserializeError"
    msg = "Failed to deserialize protocol buffer."


class UnauthorizedStateUpdateError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6019, "Unauthorized program state modification attempted.")

    code = 6019
    name = "UnauthorizedStateUpdateError"
    msg = "Unauthorized program state modification attempted."


class MissingOracleAccountsError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6020, "Not enough oracle accounts provided to closeRounds.")

    code = 6020
    name = "MissingOracleAccountsError"
    msg = "Not enough oracle accounts provided to closeRounds."


class OracleMismatchError(ProgramError):
    def __init__(self) -> None:
        super().__init__(
            6021, "An unexpected oracle account was provided for the transaction."
        )

    code = 6021
    name = "OracleMismatchError"
    msg = "An unexpected oracle account was provided for the transaction."


class CrankMaxCapacityError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6022, "Attempted to push to a Crank that's at capacity")

    code = 6022
    name = "CrankMaxCapacityError"
    msg = "Attempted to push to a Crank that's at capacity"


class AggregatorLeaseInsufficientFunds(ProgramError):
    def __init__(self) -> None:
        super().__init__(
            6023,
            "Aggregator update call attempted but attached lease has insufficient funds.",
        )

    code = 6023
    name = "AggregatorLeaseInsufficientFunds"
    msg = "Aggregator update call attempted but attached lease has insufficient funds."


class IncorrectTokenAccountMint(ProgramError):
    def __init__(self) -> None:
        super().__init__(
            6024,
            "The provided token account does not point to the Switchboard token mint.",
        )

    code = 6024
    name = "IncorrectTokenAccountMint"
    msg = "The provided token account does not point to the Switchboard token mint."


class InvalidEscrowAccount(ProgramError):
    def __init__(self) -> None:
        super().__init__(6025, "An invalid escrow account was provided.")

    code = 6025
    name = "InvalidEscrowAccount"
    msg = "An invalid escrow account was provided."


class CrankEmptyError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6026, "Crank empty. Pop failed.")

    code = 6026
    name = "CrankEmptyError"
    msg = "Crank empty. Pop failed."


class PdaDeriveError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6027, "Failed to derive a PDA from the provided seed.")

    code = 6027
    name = "PdaDeriveError"
    msg = "Failed to derive a PDA from the provided seed."


class AggregatorAccountNotFound(ProgramError):
    def __init__(self) -> None:
        super().__init__(6028, "Aggregator account missing from provided account list.")

    code = 6028
    name = "AggregatorAccountNotFound"
    msg = "Aggregator account missing from provided account list."


class PermissionAccountNotFound(ProgramError):
    def __init__(self) -> None:
        super().__init__(6029, "Permission account missing from provided account list.")

    code = 6029
    name = "PermissionAccountNotFound"
    msg = "Permission account missing from provided account list."


class LeaseAccountDeriveFailure(ProgramError):
    def __init__(self) -> None:
        super().__init__(6030, "Failed to derive a lease account.")

    code = 6030
    name = "LeaseAccountDeriveFailure"
    msg = "Failed to derive a lease account."


class PermissionAccountDeriveFailure(ProgramError):
    def __init__(self) -> None:
        super().__init__(6031, "Failed to derive a permission account.")

    code = 6031
    name = "PermissionAccountDeriveFailure"
    msg = "Failed to derive a permission account."


class EscrowAccountNotFound(ProgramError):
    def __init__(self) -> None:
        super().__init__(6032, "Escrow account missing from provided account list.")

    code = 6032
    name = "EscrowAccountNotFound"
    msg = "Escrow account missing from provided account list."


class LeaseAccountNotFound(ProgramError):
    def __init__(self) -> None:
        super().__init__(6033, "Lease account missing from provided account list.")

    code = 6033
    name = "LeaseAccountNotFound"
    msg = "Lease account missing from provided account list."


class DecimalConversionError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6034, "Decimal conversion method failed.")

    code = 6034
    name = "DecimalConversionError"
    msg = "Decimal conversion method failed."


class PermissionDenied(ProgramError):
    def __init__(self) -> None:
        super().__init__(
            6035, "Permission account is missing required flags for the given action."
        )

    code = 6035
    name = "PermissionDenied"
    msg = "Permission account is missing required flags for the given action."


class QueueAtCapacity(ProgramError):
    def __init__(self) -> None:
        super().__init__(6036, "Oracle queue is at lease capacity.")

    code = 6036
    name = "QueueAtCapacity"
    msg = "Oracle queue is at lease capacity."


class ExcessiveCrankRowsError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6037, "Data feed is already pushed on a crank.")

    code = 6037
    name = "ExcessiveCrankRowsError"
    msg = "Data feed is already pushed on a crank."


class AggregatorLockedError(ProgramError):
    def __init__(self) -> None:
        super().__init__(
            6038,
            "Aggregator is locked, no setting modifications or job additions allowed.",
        )

    code = 6038
    name = "AggregatorLockedError"
    msg = "Aggregator is locked, no setting modifications or job additions allowed."


class AggregatorInvalidBatchSizeError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6039, "Aggregator invalid batch size.")

    code = 6039
    name = "AggregatorInvalidBatchSizeError"
    msg = "Aggregator invalid batch size."


class AggregatorJobChecksumMismatch(ProgramError):
    def __init__(self) -> None:
        super().__init__(6040, "Oracle provided an incorrect aggregator job checksum.")

    code = 6040
    name = "AggregatorJobChecksumMismatch"
    msg = "Oracle provided an incorrect aggregator job checksum."


class IntegerOverflowError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6041, "An integer overflow occurred.")

    code = 6041
    name = "IntegerOverflowError"
    msg = "An integer overflow occurred."


class InvalidUpdatePeriodError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6042, "Minimum update period is 5 seconds.")

    code = 6042
    name = "InvalidUpdatePeriodError"
    msg = "Minimum update period is 5 seconds."


class NoResultsError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6043, "Aggregator round evaluation attempted with no results.")

    code = 6043
    name = "NoResultsError"
    msg = "Aggregator round evaluation attempted with no results."


class InvalidExpirationError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6044, "An expiration constraint was broken.")

    code = 6044
    name = "InvalidExpirationError"
    msg = "An expiration constraint was broken."


class InsufficientStakeError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6045, "An account provided insufficient stake for action.")

    code = 6045
    name = "InsufficientStakeError"
    msg = "An account provided insufficient stake for action."


class LeaseInactiveError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6046, "The provided lease account is not active.")

    code = 6046
    name = "LeaseInactiveError"
    msg = "The provided lease account is not active."


class NoAggregatorJobsFound(ProgramError):
    def __init__(self) -> None:
        super().__init__(6047, "No jobs are currently included in the aggregator.")

    code = 6047
    name = "NoAggregatorJobsFound"
    msg = "No jobs are currently included in the aggregator."


class IntegerUnderflowError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6048, "An integer underflow occurred.")

    code = 6048
    name = "IntegerUnderflowError"
    msg = "An integer underflow occurred."


class OracleQueueMismatch(ProgramError):
    def __init__(self) -> None:
        super().__init__(6049, "An invalid oracle queue account was provided.")

    code = 6049
    name = "OracleQueueMismatch"
    msg = "An invalid oracle queue account was provided."


class OracleWalletMismatchError(ProgramError):
    def __init__(self) -> None:
        super().__init__(
            6050,
            "An unexpected oracle wallet account was provided for the transaction.",
        )

    code = 6050
    name = "OracleWalletMismatchError"
    msg = "An unexpected oracle wallet account was provided for the transaction."


class InvalidBufferAccountError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6051, "An invalid buffer account was provided.")

    code = 6051
    name = "InvalidBufferAccountError"
    msg = "An invalid buffer account was provided."


class InsufficientOracleQueueError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6052, "Insufficient oracle queue size.")

    code = 6052
    name = "InsufficientOracleQueueError"
    msg = "Insufficient oracle queue size."


class InvalidAuthorityError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6053, "Invalid authority account provided.")

    code = 6053
    name = "InvalidAuthorityError"
    msg = "Invalid authority account provided."


class InvalidTokenAccountMintError(ProgramError):
    def __init__(self) -> None:
        super().__init__(
            6054, "A provided token wallet is associated with an incorrect mint."
        )

    code = 6054
    name = "InvalidTokenAccountMintError"
    msg = "A provided token wallet is associated with an incorrect mint."


class ExcessiveLeaseWithdrawlError(ProgramError):
    def __init__(self) -> None:
        super().__init__(
            6055,
            "You must leave enough funds to perform at least 1 update in the lease.",
        )

    code = 6055
    name = "ExcessiveLeaseWithdrawlError"
    msg = "You must leave enough funds to perform at least 1 update in the lease."


class InvalideHistoryAccountError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6056, "Invalid history account provided.")

    code = 6056
    name = "InvalideHistoryAccountError"
    msg = "Invalid history account provided."


class InvalidLeaseAccountEscrowError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6057, "Invalid lease account escrow.")

    code = 6057
    name = "InvalidLeaseAccountEscrowError"
    msg = "Invalid lease account escrow."


class InvalidCrankAccountError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6058, "Invalid crank provided.")

    code = 6058
    name = "InvalidCrankAccountError"
    msg = "Invalid crank provided."


class CrankNoElementsReadyError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6059, "No elements ready to be popped.")

    code = 6059
    name = "CrankNoElementsReadyError"
    msg = "No elements ready to be popped."


class IndexOutOfBoundsError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6060, "Index out of bounds")

    code = 6060
    name = "IndexOutOfBoundsError"
    msg = "Index out of bounds"


class VrfInvalidRequestError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6061, "Invalid vrf request params")

    code = 6061
    name = "VrfInvalidRequestError"
    msg = "Invalid vrf request params"


class VrfInvalidProofSubmissionError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6062, "Vrf proof failed to verify")

    code = 6062
    name = "VrfInvalidProofSubmissionError"
    msg = "Vrf proof failed to verify"


class VrfVerifyError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6063, "Error in verifying vrf proof.")

    code = 6063
    name = "VrfVerifyError"
    msg = "Error in verifying vrf proof."


class VrfCallbackError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6064, "Vrf callback function failed.")

    code = 6064
    name = "VrfCallbackError"
    msg = "Vrf callback function failed."


class VrfCallbackParamsError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6065, "Invalid vrf callback params provided.")

    code = 6065
    name = "VrfCallbackParamsError"
    msg = "Invalid vrf callback params provided."


class VrfCallbackAlreadyCalledError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6066, "Vrf callback has already been triggered.")

    code = 6066
    name = "VrfCallbackAlreadyCalledError"
    msg = "Vrf callback has already been triggered."


class VrfInvalidPubkeyError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6067, "The provided pubkey is invalid to use in ecvrf proofs")

    code = 6067
    name = "VrfInvalidPubkeyError"
    msg = "The provided pubkey is invalid to use in ecvrf proofs"


class VrfTooManyVerifyCallsError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6068, "Number of required verify calls exceeded")

    code = 6068
    name = "VrfTooManyVerifyCallsError"
    msg = "Number of required verify calls exceeded"


class VrfRequestAlreadyLaunchedError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6069, "Vrf request is already pending")

    code = 6069
    name = "VrfRequestAlreadyLaunchedError"
    msg = "Vrf request is already pending"


class VrfInsufficientVerificationError(ProgramError):
    def __init__(self) -> None:
        super().__init__(
            6070, "Insufficient amount of proofs collected for VRF callback"
        )

    code = 6070
    name = "VrfInsufficientVerificationError"
    msg = "Insufficient amount of proofs collected for VRF callback"


class InvalidVrfProducerError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6071, "An incorrect oracle attempted to submit a proof")

    code = 6071
    name = "InvalidVrfProducerError"
    msg = "An incorrect oracle attempted to submit a proof"


class NoopError(ProgramError):
    def __init__(self) -> None:
        super().__init__(6072, "Noop error")

    code = 6072
    name = "NoopError"
    msg = "Noop error"


CustomError = typing.Union[
    ArrayOperationError,
    QueueOperationError,
    IncorrectProgramOwnerError,
    InvalidAggregatorRound,
    TooManyAggregatorJobs,
    AggregatorCurrentRoundClosed,
    AggregatorInvalidSaveResult,
    InvalidStrDecimalConversion,
    AccountLoaderMissingSignature,
    MissingRequiredSignature,
    ArrayOverflowError,
    ArrayUnderflowError,
    PubkeyNotFoundError,
    AggregatorIllegalRoundOpenCall,
    AggregatorIllegalRoundCloseCall,
    AggregatorClosedError,
    IllegalOracleIdxError,
    OracleAlreadyRespondedError,
    ProtoDeserializeError,
    UnauthorizedStateUpdateError,
    MissingOracleAccountsError,
    OracleMismatchError,
    CrankMaxCapacityError,
    AggregatorLeaseInsufficientFunds,
    IncorrectTokenAccountMint,
    InvalidEscrowAccount,
    CrankEmptyError,
    PdaDeriveError,
    AggregatorAccountNotFound,
    PermissionAccountNotFound,
    LeaseAccountDeriveFailure,
    PermissionAccountDeriveFailure,
    EscrowAccountNotFound,
    LeaseAccountNotFound,
    DecimalConversionError,
    PermissionDenied,
    QueueAtCapacity,
    ExcessiveCrankRowsError,
    AggregatorLockedError,
    AggregatorInvalidBatchSizeError,
    AggregatorJobChecksumMismatch,
    IntegerOverflowError,
    InvalidUpdatePeriodError,
    NoResultsError,
    InvalidExpirationError,
    InsufficientStakeError,
    LeaseInactiveError,
    NoAggregatorJobsFound,
    IntegerUnderflowError,
    OracleQueueMismatch,
    OracleWalletMismatchError,
    InvalidBufferAccountError,
    InsufficientOracleQueueError,
    InvalidAuthorityError,
    InvalidTokenAccountMintError,
    ExcessiveLeaseWithdrawlError,
    InvalideHistoryAccountError,
    InvalidLeaseAccountEscrowError,
    InvalidCrankAccountError,
    CrankNoElementsReadyError,
    IndexOutOfBoundsError,
    VrfInvalidRequestError,
    VrfInvalidProofSubmissionError,
    VrfVerifyError,
    VrfCallbackError,
    VrfCallbackParamsError,
    VrfCallbackAlreadyCalledError,
    VrfInvalidPubkeyError,
    VrfTooManyVerifyCallsError,
    VrfRequestAlreadyLaunchedError,
    VrfInsufficientVerificationError,
    InvalidVrfProducerError,
    NoopError,
]
CUSTOM_ERROR_MAP: dict[int, CustomError] = {
    6000: ArrayOperationError(),
    6001: QueueOperationError(),
    6002: IncorrectProgramOwnerError(),
    6003: InvalidAggregatorRound(),
    6004: TooManyAggregatorJobs(),
    6005: AggregatorCurrentRoundClosed(),
    6006: AggregatorInvalidSaveResult(),
    6007: InvalidStrDecimalConversion(),
    6008: AccountLoaderMissingSignature(),
    6009: MissingRequiredSignature(),
    6010: ArrayOverflowError(),
    6011: ArrayUnderflowError(),
    6012: PubkeyNotFoundError(),
    6013: AggregatorIllegalRoundOpenCall(),
    6014: AggregatorIllegalRoundCloseCall(),
    6015: AggregatorClosedError(),
    6016: IllegalOracleIdxError(),
    6017: OracleAlreadyRespondedError(),
    6018: ProtoDeserializeError(),
    6019: UnauthorizedStateUpdateError(),
    6020: MissingOracleAccountsError(),
    6021: OracleMismatchError(),
    6022: CrankMaxCapacityError(),
    6023: AggregatorLeaseInsufficientFunds(),
    6024: IncorrectTokenAccountMint(),
    6025: InvalidEscrowAccount(),
    6026: CrankEmptyError(),
    6027: PdaDeriveError(),
    6028: AggregatorAccountNotFound(),
    6029: PermissionAccountNotFound(),
    6030: LeaseAccountDeriveFailure(),
    6031: PermissionAccountDeriveFailure(),
    6032: EscrowAccountNotFound(),
    6033: LeaseAccountNotFound(),
    6034: DecimalConversionError(),
    6035: PermissionDenied(),
    6036: QueueAtCapacity(),
    6037: ExcessiveCrankRowsError(),
    6038: AggregatorLockedError(),
    6039: AggregatorInvalidBatchSizeError(),
    6040: AggregatorJobChecksumMismatch(),
    6041: IntegerOverflowError(),
    6042: InvalidUpdatePeriodError(),
    6043: NoResultsError(),
    6044: InvalidExpirationError(),
    6045: InsufficientStakeError(),
    6046: LeaseInactiveError(),
    6047: NoAggregatorJobsFound(),
    6048: IntegerUnderflowError(),
    6049: OracleQueueMismatch(),
    6050: OracleWalletMismatchError(),
    6051: InvalidBufferAccountError(),
    6052: InsufficientOracleQueueError(),
    6053: InvalidAuthorityError(),
    6054: InvalidTokenAccountMintError(),
    6055: ExcessiveLeaseWithdrawlError(),
    6056: InvalideHistoryAccountError(),
    6057: InvalidLeaseAccountEscrowError(),
    6058: InvalidCrankAccountError(),
    6059: CrankNoElementsReadyError(),
    6060: IndexOutOfBoundsError(),
    6061: VrfInvalidRequestError(),
    6062: VrfInvalidProofSubmissionError(),
    6063: VrfVerifyError(),
    6064: VrfCallbackError(),
    6065: VrfCallbackParamsError(),
    6066: VrfCallbackAlreadyCalledError(),
    6067: VrfInvalidPubkeyError(),
    6068: VrfTooManyVerifyCallsError(),
    6069: VrfRequestAlreadyLaunchedError(),
    6070: VrfInsufficientVerificationError(),
    6071: InvalidVrfProducerError(),
    6072: NoopError(),
}


def from_code(code: int) -> typing.Optional[CustomError]:
    maybe_err = CUSTOM_ERROR_MAP.get(code)
    if maybe_err is None:
        return None
    return maybe_err
