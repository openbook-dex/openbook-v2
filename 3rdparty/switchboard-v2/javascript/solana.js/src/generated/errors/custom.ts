import { SwitchboardProgram } from '../../program';
export type CustomError =
  | ArrayOperationError
  | QueueOperationError
  | IncorrectProgramOwnerError
  | InvalidAggregatorRound
  | TooManyAggregatorJobs
  | AggregatorCurrentRoundClosed
  | AggregatorInvalidSaveResult
  | InvalidStrDecimalConversion
  | AccountLoaderMissingSignature
  | MissingRequiredSignature
  | ArrayOverflowError
  | ArrayUnderflowError
  | PubkeyNotFoundError
  | AggregatorIllegalRoundOpenCall
  | AggregatorIllegalRoundCloseCall
  | AggregatorClosedError
  | IllegalOracleIdxError
  | OracleAlreadyRespondedError
  | ProtoDeserializeError
  | UnauthorizedStateUpdateError
  | MissingOracleAccountsError
  | OracleMismatchError
  | CrankMaxCapacityError
  | AggregatorLeaseInsufficientFunds
  | IncorrectTokenAccountMint
  | InvalidEscrowAccount
  | CrankEmptyError
  | PdaDeriveError
  | AggregatorAccountNotFound
  | PermissionAccountNotFound
  | LeaseAccountDeriveFailure
  | PermissionAccountDeriveFailure
  | EscrowAccountNotFound
  | LeaseAccountNotFound
  | DecimalConversionError
  | PermissionDenied
  | QueueAtCapacity
  | ExcessiveCrankRowsError
  | AggregatorLockedError
  | AggregatorInvalidBatchSizeError
  | AggregatorJobChecksumMismatch
  | IntegerOverflowError
  | InvalidUpdatePeriodError
  | NoResultsError
  | InvalidExpirationError
  | InsufficientStakeError
  | LeaseInactiveError
  | NoAggregatorJobsFound
  | IntegerUnderflowError
  | OracleQueueMismatch
  | OracleWalletMismatchError
  | InvalidBufferAccountError
  | InsufficientOracleQueueError
  | InvalidAuthorityError
  | InvalidTokenAccountMintError
  | ExcessiveLeaseWithdrawlError
  | InvalideHistoryAccountError
  | InvalidLeaseAccountEscrowError
  | InvalidCrankAccountError
  | CrankNoElementsReadyError
  | IndexOutOfBoundsError
  | VrfInvalidRequestError
  | VrfInvalidProofSubmissionError
  | VrfVerifyError
  | VrfCallbackError
  | VrfCallbackParamsError
  | VrfCallbackAlreadyCalledError
  | VrfInvalidPubkeyError
  | VrfTooManyVerifyCallsError
  | VrfRequestAlreadyLaunchedError
  | VrfInsufficientVerificationError
  | InvalidVrfProducerError
  | InvalidGovernancePidError
  | InvalidGovernanceAccountError
  | MissingOptionalAccount
  | InvalidSpawnRecordOwner
  | NoopError
  | MissingRequiredAccountsError
  | InvalidMintError
  | InvalidTokenAccountKeyError
  | InvalidJobAccountError
  | VoterStakeRegistryError
  | AccountDiscriminatorMismatch
  | FuckingImpossibleError
  | InvalidVrfRound
  | JobSizeExceeded
  | JobChunksExceeded
  | JobDataLocked
  | JobNotInitialized
  | BufferRelayerIllegalRoundOpenCall
  | InvalidSliderAccount;

export class ArrayOperationError extends Error {
  static readonly code = 6000;
  readonly code = 6000;
  readonly name = 'ArrayOperationError';
  readonly msg = 'Illegal operation on a Switchboard array.';

  constructor(readonly logs?: string[]) {
    super('6000: Illegal operation on a Switchboard array.');
  }
}

export class QueueOperationError extends Error {
  static readonly code = 6001;
  readonly code = 6001;
  readonly name = 'QueueOperationError';
  readonly msg = 'Illegal operation on a Switchboard queue.';

  constructor(readonly logs?: string[]) {
    super('6001: Illegal operation on a Switchboard queue.');
  }
}

export class IncorrectProgramOwnerError extends Error {
  static readonly code = 6002;
  readonly code = 6002;
  readonly name = 'IncorrectProgramOwnerError';
  readonly msg =
    'An account required to be owned by the program has a different owner.';

  constructor(readonly logs?: string[]) {
    super(
      '6002: An account required to be owned by the program has a different owner.'
    );
  }
}

export class InvalidAggregatorRound extends Error {
  static readonly code = 6003;
  readonly code = 6003;
  readonly name = 'InvalidAggregatorRound';
  readonly msg = 'Aggregator is not currently populated with a valid round.';

  constructor(readonly logs?: string[]) {
    super('6003: Aggregator is not currently populated with a valid round.');
  }
}

export class TooManyAggregatorJobs extends Error {
  static readonly code = 6004;
  readonly code = 6004;
  readonly name = 'TooManyAggregatorJobs';
  readonly msg = 'Aggregator cannot fit any more jobs.';

  constructor(readonly logs?: string[]) {
    super('6004: Aggregator cannot fit any more jobs.');
  }
}

export class AggregatorCurrentRoundClosed extends Error {
  static readonly code = 6005;
  readonly code = 6005;
  readonly name = 'AggregatorCurrentRoundClosed';
  readonly msg =
    "Aggregator's current round is closed. No results are being accepted.";

  constructor(readonly logs?: string[]) {
    super(
      "6005: Aggregator's current round is closed. No results are being accepted."
    );
  }
}

export class AggregatorInvalidSaveResult extends Error {
  static readonly code = 6006;
  readonly code = 6006;
  readonly name = 'AggregatorInvalidSaveResult';
  readonly msg = 'Aggregator received an invalid save result instruction.';

  constructor(readonly logs?: string[]) {
    super('6006: Aggregator received an invalid save result instruction.');
  }
}

export class InvalidStrDecimalConversion extends Error {
  static readonly code = 6007;
  readonly code = 6007;
  readonly name = 'InvalidStrDecimalConversion';
  readonly msg = 'Failed to convert string to decimal format.';

  constructor(readonly logs?: string[]) {
    super('6007: Failed to convert string to decimal format.');
  }
}

export class AccountLoaderMissingSignature extends Error {
  static readonly code = 6008;
  readonly code = 6008;
  readonly name = 'AccountLoaderMissingSignature';
  readonly msg = 'AccountLoader account is missing a required signature.';

  constructor(readonly logs?: string[]) {
    super('6008: AccountLoader account is missing a required signature.');
  }
}

export class MissingRequiredSignature extends Error {
  static readonly code = 6009;
  readonly code = 6009;
  readonly name = 'MissingRequiredSignature';
  readonly msg = 'Account is missing a required signature.';

  constructor(readonly logs?: string[]) {
    super('6009: Account is missing a required signature.');
  }
}

export class ArrayOverflowError extends Error {
  static readonly code = 6010;
  readonly code = 6010;
  readonly name = 'ArrayOverflowError';
  readonly msg =
    'The attempted action will overflow a zero-copy account array.';

  constructor(readonly logs?: string[]) {
    super(
      '6010: The attempted action will overflow a zero-copy account array.'
    );
  }
}

export class ArrayUnderflowError extends Error {
  static readonly code = 6011;
  readonly code = 6011;
  readonly name = 'ArrayUnderflowError';
  readonly msg =
    'The attempted action will underflow a zero-copy account array.';

  constructor(readonly logs?: string[]) {
    super(
      '6011: The attempted action will underflow a zero-copy account array.'
    );
  }
}

export class PubkeyNotFoundError extends Error {
  static readonly code = 6012;
  readonly code = 6012;
  readonly name = 'PubkeyNotFoundError';
  readonly msg = 'The queried public key was not found.';

  constructor(readonly logs?: string[]) {
    super('6012: The queried public key was not found.');
  }
}

export class AggregatorIllegalRoundOpenCall extends Error {
  static readonly code = 6013;
  readonly code = 6013;
  readonly name = 'AggregatorIllegalRoundOpenCall';
  readonly msg = 'Aggregator round open called too early.';

  constructor(readonly logs?: string[]) {
    super('6013: Aggregator round open called too early.');
  }
}

export class AggregatorIllegalRoundCloseCall extends Error {
  static readonly code = 6014;
  readonly code = 6014;
  readonly name = 'AggregatorIllegalRoundCloseCall';
  readonly msg = 'Aggregator round close called too early.';

  constructor(readonly logs?: string[]) {
    super('6014: Aggregator round close called too early.');
  }
}

export class AggregatorClosedError extends Error {
  static readonly code = 6015;
  readonly code = 6015;
  readonly name = 'AggregatorClosedError';
  readonly msg = 'Aggregator is closed. Illegal action.';

  constructor(readonly logs?: string[]) {
    super('6015: Aggregator is closed. Illegal action.');
  }
}

export class IllegalOracleIdxError extends Error {
  static readonly code = 6016;
  readonly code = 6016;
  readonly name = 'IllegalOracleIdxError';
  readonly msg = 'Illegal oracle index.';

  constructor(readonly logs?: string[]) {
    super('6016: Illegal oracle index.');
  }
}

export class OracleAlreadyRespondedError extends Error {
  static readonly code = 6017;
  readonly code = 6017;
  readonly name = 'OracleAlreadyRespondedError';
  readonly msg = 'The provided oracle has already responded this round.';

  constructor(readonly logs?: string[]) {
    super('6017: The provided oracle has already responded this round.');
  }
}

export class ProtoDeserializeError extends Error {
  static readonly code = 6018;
  readonly code = 6018;
  readonly name = 'ProtoDeserializeError';
  readonly msg = 'Failed to deserialize protocol buffer.';

  constructor(readonly logs?: string[]) {
    super('6018: Failed to deserialize protocol buffer.');
  }
}

export class UnauthorizedStateUpdateError extends Error {
  static readonly code = 6019;
  readonly code = 6019;
  readonly name = 'UnauthorizedStateUpdateError';
  readonly msg = 'Unauthorized program state modification attempted.';

  constructor(readonly logs?: string[]) {
    super('6019: Unauthorized program state modification attempted.');
  }
}

export class MissingOracleAccountsError extends Error {
  static readonly code = 6020;
  readonly code = 6020;
  readonly name = 'MissingOracleAccountsError';
  readonly msg = 'Not enough oracle accounts provided to closeRounds.';

  constructor(readonly logs?: string[]) {
    super('6020: Not enough oracle accounts provided to closeRounds.');
  }
}

export class OracleMismatchError extends Error {
  static readonly code = 6021;
  readonly code = 6021;
  readonly name = 'OracleMismatchError';
  readonly msg =
    'An unexpected oracle account was provided for the transaction.';

  constructor(readonly logs?: string[]) {
    super(
      '6021: An unexpected oracle account was provided for the transaction.'
    );
  }
}

export class CrankMaxCapacityError extends Error {
  static readonly code = 6022;
  readonly code = 6022;
  readonly name = 'CrankMaxCapacityError';
  readonly msg = "Attempted to push to a Crank that's at capacity";

  constructor(readonly logs?: string[]) {
    super("6022: Attempted to push to a Crank that's at capacity");
  }
}

export class AggregatorLeaseInsufficientFunds extends Error {
  static readonly code = 6023;
  readonly code = 6023;
  readonly name = 'AggregatorLeaseInsufficientFunds';
  readonly msg =
    'Aggregator update call attempted but attached lease has insufficient funds.';

  constructor(readonly logs?: string[]) {
    super(
      '6023: Aggregator update call attempted but attached lease has insufficient funds.'
    );
  }
}

export class IncorrectTokenAccountMint extends Error {
  static readonly code = 6024;
  readonly code = 6024;
  readonly name = 'IncorrectTokenAccountMint';
  readonly msg =
    'The provided token account does not point to the Switchboard token mint.';

  constructor(readonly logs?: string[]) {
    super(
      '6024: The provided token account does not point to the Switchboard token mint.'
    );
  }
}

export class InvalidEscrowAccount extends Error {
  static readonly code = 6025;
  readonly code = 6025;
  readonly name = 'InvalidEscrowAccount';
  readonly msg = 'An invalid escrow account was provided.';

  constructor(readonly logs?: string[]) {
    super('6025: An invalid escrow account was provided.');
  }
}

export class CrankEmptyError extends Error {
  static readonly code = 6026;
  readonly code = 6026;
  readonly name = 'CrankEmptyError';
  readonly msg = 'Crank empty. Pop failed.';

  constructor(readonly logs?: string[]) {
    super('6026: Crank empty. Pop failed.');
  }
}

export class PdaDeriveError extends Error {
  static readonly code = 6027;
  readonly code = 6027;
  readonly name = 'PdaDeriveError';
  readonly msg = 'Failed to derive a PDA from the provided seed.';

  constructor(readonly logs?: string[]) {
    super('6027: Failed to derive a PDA from the provided seed.');
  }
}

export class AggregatorAccountNotFound extends Error {
  static readonly code = 6028;
  readonly code = 6028;
  readonly name = 'AggregatorAccountNotFound';
  readonly msg = 'Aggregator account missing from provided account list.';

  constructor(readonly logs?: string[]) {
    super('6028: Aggregator account missing from provided account list.');
  }
}

export class PermissionAccountNotFound extends Error {
  static readonly code = 6029;
  readonly code = 6029;
  readonly name = 'PermissionAccountNotFound';
  readonly msg = 'Permission account missing from provided account list.';

  constructor(readonly logs?: string[]) {
    super('6029: Permission account missing from provided account list.');
  }
}

export class LeaseAccountDeriveFailure extends Error {
  static readonly code = 6030;
  readonly code = 6030;
  readonly name = 'LeaseAccountDeriveFailure';
  readonly msg = 'Failed to derive a lease account.';

  constructor(readonly logs?: string[]) {
    super('6030: Failed to derive a lease account.');
  }
}

export class PermissionAccountDeriveFailure extends Error {
  static readonly code = 6031;
  readonly code = 6031;
  readonly name = 'PermissionAccountDeriveFailure';
  readonly msg = 'Failed to derive a permission account.';

  constructor(readonly logs?: string[]) {
    super('6031: Failed to derive a permission account.');
  }
}

export class EscrowAccountNotFound extends Error {
  static readonly code = 6032;
  readonly code = 6032;
  readonly name = 'EscrowAccountNotFound';
  readonly msg = 'Escrow account missing from provided account list.';

  constructor(readonly logs?: string[]) {
    super('6032: Escrow account missing from provided account list.');
  }
}

export class LeaseAccountNotFound extends Error {
  static readonly code = 6033;
  readonly code = 6033;
  readonly name = 'LeaseAccountNotFound';
  readonly msg = 'Lease account missing from provided account list.';

  constructor(readonly logs?: string[]) {
    super('6033: Lease account missing from provided account list.');
  }
}

export class DecimalConversionError extends Error {
  static readonly code = 6034;
  readonly code = 6034;
  readonly name = 'DecimalConversionError';
  readonly msg = 'Decimal conversion method failed.';

  constructor(readonly logs?: string[]) {
    super('6034: Decimal conversion method failed.');
  }
}

export class PermissionDenied extends Error {
  static readonly code = 6035;
  readonly code = 6035;
  readonly name = 'PermissionDenied';
  readonly msg =
    'Permission account is missing required flags for the given action.';

  constructor(readonly logs?: string[]) {
    super(
      '6035: Permission account is missing required flags for the given action.'
    );
  }
}

export class QueueAtCapacity extends Error {
  static readonly code = 6036;
  readonly code = 6036;
  readonly name = 'QueueAtCapacity';
  readonly msg = 'Oracle queue is at lease capacity.';

  constructor(readonly logs?: string[]) {
    super('6036: Oracle queue is at lease capacity.');
  }
}

export class ExcessiveCrankRowsError extends Error {
  static readonly code = 6037;
  readonly code = 6037;
  readonly name = 'ExcessiveCrankRowsError';
  readonly msg = 'Data feed is already pushed on a crank.';

  constructor(readonly logs?: string[]) {
    super('6037: Data feed is already pushed on a crank.');
  }
}

export class AggregatorLockedError extends Error {
  static readonly code = 6038;
  readonly code = 6038;
  readonly name = 'AggregatorLockedError';
  readonly msg =
    'Aggregator is locked, no setting modifications or job additions allowed.';

  constructor(readonly logs?: string[]) {
    super(
      '6038: Aggregator is locked, no setting modifications or job additions allowed.'
    );
  }
}

export class AggregatorInvalidBatchSizeError extends Error {
  static readonly code = 6039;
  readonly code = 6039;
  readonly name = 'AggregatorInvalidBatchSizeError';
  readonly msg = 'Aggregator invalid batch size.';

  constructor(readonly logs?: string[]) {
    super('6039: Aggregator invalid batch size.');
  }
}

export class AggregatorJobChecksumMismatch extends Error {
  static readonly code = 6040;
  readonly code = 6040;
  readonly name = 'AggregatorJobChecksumMismatch';
  readonly msg = 'Oracle provided an incorrect aggregator job checksum.';

  constructor(readonly logs?: string[]) {
    super('6040: Oracle provided an incorrect aggregator job checksum.');
  }
}

export class IntegerOverflowError extends Error {
  static readonly code = 6041;
  readonly code = 6041;
  readonly name = 'IntegerOverflowError';
  readonly msg = 'An integer overflow occurred.';

  constructor(readonly logs?: string[]) {
    super('6041: An integer overflow occurred.');
  }
}

export class InvalidUpdatePeriodError extends Error {
  static readonly code = 6042;
  readonly code = 6042;
  readonly name = 'InvalidUpdatePeriodError';
  readonly msg = 'Minimum update period is 5 seconds.';

  constructor(readonly logs?: string[]) {
    super('6042: Minimum update period is 5 seconds.');
  }
}

export class NoResultsError extends Error {
  static readonly code = 6043;
  readonly code = 6043;
  readonly name = 'NoResultsError';
  readonly msg = 'Aggregator round evaluation attempted with no results.';

  constructor(readonly logs?: string[]) {
    super('6043: Aggregator round evaluation attempted with no results.');
  }
}

export class InvalidExpirationError extends Error {
  static readonly code = 6044;
  readonly code = 6044;
  readonly name = 'InvalidExpirationError';
  readonly msg = 'An expiration constraint was broken.';

  constructor(readonly logs?: string[]) {
    super('6044: An expiration constraint was broken.');
  }
}

export class InsufficientStakeError extends Error {
  static readonly code = 6045;
  readonly code = 6045;
  readonly name = 'InsufficientStakeError';
  readonly msg = 'An account provided insufficient stake for action.';

  constructor(readonly logs?: string[]) {
    super('6045: An account provided insufficient stake for action.');
  }
}

export class LeaseInactiveError extends Error {
  static readonly code = 6046;
  readonly code = 6046;
  readonly name = 'LeaseInactiveError';
  readonly msg = 'The provided lease account is not active.';

  constructor(readonly logs?: string[]) {
    super('6046: The provided lease account is not active.');
  }
}

export class NoAggregatorJobsFound extends Error {
  static readonly code = 6047;
  readonly code = 6047;
  readonly name = 'NoAggregatorJobsFound';
  readonly msg = 'No jobs are currently included in the aggregator.';

  constructor(readonly logs?: string[]) {
    super('6047: No jobs are currently included in the aggregator.');
  }
}

export class IntegerUnderflowError extends Error {
  static readonly code = 6048;
  readonly code = 6048;
  readonly name = 'IntegerUnderflowError';
  readonly msg = 'An integer underflow occurred.';

  constructor(readonly logs?: string[]) {
    super('6048: An integer underflow occurred.');
  }
}

export class OracleQueueMismatch extends Error {
  static readonly code = 6049;
  readonly code = 6049;
  readonly name = 'OracleQueueMismatch';
  readonly msg = 'An invalid oracle queue account was provided.';

  constructor(readonly logs?: string[]) {
    super('6049: An invalid oracle queue account was provided.');
  }
}

export class OracleWalletMismatchError extends Error {
  static readonly code = 6050;
  readonly code = 6050;
  readonly name = 'OracleWalletMismatchError';
  readonly msg =
    'An unexpected oracle wallet account was provided for the transaction.';

  constructor(readonly logs?: string[]) {
    super(
      '6050: An unexpected oracle wallet account was provided for the transaction.'
    );
  }
}

export class InvalidBufferAccountError extends Error {
  static readonly code = 6051;
  readonly code = 6051;
  readonly name = 'InvalidBufferAccountError';
  readonly msg = 'An invalid buffer account was provided.';

  constructor(readonly logs?: string[]) {
    super('6051: An invalid buffer account was provided.');
  }
}

export class InsufficientOracleQueueError extends Error {
  static readonly code = 6052;
  readonly code = 6052;
  readonly name = 'InsufficientOracleQueueError';
  readonly msg = 'Insufficient oracle queue size.';

  constructor(readonly logs?: string[]) {
    super('6052: Insufficient oracle queue size.');
  }
}

export class InvalidAuthorityError extends Error {
  static readonly code = 6053;
  readonly code = 6053;
  readonly name = 'InvalidAuthorityError';
  readonly msg = 'Invalid authority account provided.';

  constructor(readonly logs?: string[]) {
    super('6053: Invalid authority account provided.');
  }
}

export class InvalidTokenAccountMintError extends Error {
  static readonly code = 6054;
  readonly code = 6054;
  readonly name = 'InvalidTokenAccountMintError';
  readonly msg =
    'A provided token wallet is associated with an incorrect mint.';

  constructor(readonly logs?: string[]) {
    super(
      '6054: A provided token wallet is associated with an incorrect mint.'
    );
  }
}

export class ExcessiveLeaseWithdrawlError extends Error {
  static readonly code = 6055;
  readonly code = 6055;
  readonly name = 'ExcessiveLeaseWithdrawlError';
  readonly msg =
    'You must leave enough funds to perform at least 1 update in the lease.';

  constructor(readonly logs?: string[]) {
    super(
      '6055: You must leave enough funds to perform at least 1 update in the lease.'
    );
  }
}

export class InvalideHistoryAccountError extends Error {
  static readonly code = 6056;
  readonly code = 6056;
  readonly name = 'InvalideHistoryAccountError';
  readonly msg = 'Invalid history account provided.';

  constructor(readonly logs?: string[]) {
    super('6056: Invalid history account provided.');
  }
}

export class InvalidLeaseAccountEscrowError extends Error {
  static readonly code = 6057;
  readonly code = 6057;
  readonly name = 'InvalidLeaseAccountEscrowError';
  readonly msg = 'Invalid lease account escrow.';

  constructor(readonly logs?: string[]) {
    super('6057: Invalid lease account escrow.');
  }
}

export class InvalidCrankAccountError extends Error {
  static readonly code = 6058;
  readonly code = 6058;
  readonly name = 'InvalidCrankAccountError';
  readonly msg = 'Invalid crank provided.';

  constructor(readonly logs?: string[]) {
    super('6058: Invalid crank provided.');
  }
}

export class CrankNoElementsReadyError extends Error {
  static readonly code = 6059;
  readonly code = 6059;
  readonly name = 'CrankNoElementsReadyError';
  readonly msg = 'No elements ready to be popped.';

  constructor(readonly logs?: string[]) {
    super('6059: No elements ready to be popped.');
  }
}

export class IndexOutOfBoundsError extends Error {
  static readonly code = 6060;
  readonly code = 6060;
  readonly name = 'IndexOutOfBoundsError';
  readonly msg = 'Index out of bounds';

  constructor(readonly logs?: string[]) {
    super('6060: Index out of bounds');
  }
}

export class VrfInvalidRequestError extends Error {
  static readonly code = 6061;
  readonly code = 6061;
  readonly name = 'VrfInvalidRequestError';
  readonly msg = 'Invalid vrf request params';

  constructor(readonly logs?: string[]) {
    super('6061: Invalid vrf request params');
  }
}

export class VrfInvalidProofSubmissionError extends Error {
  static readonly code = 6062;
  readonly code = 6062;
  readonly name = 'VrfInvalidProofSubmissionError';
  readonly msg = 'Vrf proof failed to verify';

  constructor(readonly logs?: string[]) {
    super('6062: Vrf proof failed to verify');
  }
}

export class VrfVerifyError extends Error {
  static readonly code = 6063;
  readonly code = 6063;
  readonly name = 'VrfVerifyError';
  readonly msg = 'Error in verifying vrf proof.';

  constructor(readonly logs?: string[]) {
    super('6063: Error in verifying vrf proof.');
  }
}

export class VrfCallbackError extends Error {
  static readonly code = 6064;
  readonly code = 6064;
  readonly name = 'VrfCallbackError';
  readonly msg = 'Vrf callback function failed.';

  constructor(readonly logs?: string[]) {
    super('6064: Vrf callback function failed.');
  }
}

export class VrfCallbackParamsError extends Error {
  static readonly code = 6065;
  readonly code = 6065;
  readonly name = 'VrfCallbackParamsError';
  readonly msg = 'Invalid vrf callback params provided.';

  constructor(readonly logs?: string[]) {
    super('6065: Invalid vrf callback params provided.');
  }
}

export class VrfCallbackAlreadyCalledError extends Error {
  static readonly code = 6066;
  readonly code = 6066;
  readonly name = 'VrfCallbackAlreadyCalledError';
  readonly msg = 'Vrf callback has already been triggered.';

  constructor(readonly logs?: string[]) {
    super('6066: Vrf callback has already been triggered.');
  }
}

export class VrfInvalidPubkeyError extends Error {
  static readonly code = 6067;
  readonly code = 6067;
  readonly name = 'VrfInvalidPubkeyError';
  readonly msg = 'The provided pubkey is invalid to use in ecvrf proofs';

  constructor(readonly logs?: string[]) {
    super('6067: The provided pubkey is invalid to use in ecvrf proofs');
  }
}

export class VrfTooManyVerifyCallsError extends Error {
  static readonly code = 6068;
  readonly code = 6068;
  readonly name = 'VrfTooManyVerifyCallsError';
  readonly msg = 'Number of required verify calls exceeded';

  constructor(readonly logs?: string[]) {
    super('6068: Number of required verify calls exceeded');
  }
}

export class VrfRequestAlreadyLaunchedError extends Error {
  static readonly code = 6069;
  readonly code = 6069;
  readonly name = 'VrfRequestAlreadyLaunchedError';
  readonly msg = 'Vrf request is already pending';

  constructor(readonly logs?: string[]) {
    super('6069: Vrf request is already pending');
  }
}

export class VrfInsufficientVerificationError extends Error {
  static readonly code = 6070;
  readonly code = 6070;
  readonly name = 'VrfInsufficientVerificationError';
  readonly msg = 'Insufficient amount of proofs collected for VRF callback';

  constructor(readonly logs?: string[]) {
    super('6070: Insufficient amount of proofs collected for VRF callback');
  }
}

export class InvalidVrfProducerError extends Error {
  static readonly code = 6071;
  readonly code = 6071;
  readonly name = 'InvalidVrfProducerError';
  readonly msg = 'An incorrect oracle attempted to submit a proof';

  constructor(readonly logs?: string[]) {
    super('6071: An incorrect oracle attempted to submit a proof');
  }
}

export class InvalidGovernancePidError extends Error {
  static readonly code = 6072;
  readonly code = 6072;
  readonly name = 'InvalidGovernancePidError';
  readonly msg = 'Invalid SPLGovernance Account Supplied';

  constructor(readonly logs?: string[]) {
    super('6072: Invalid SPLGovernance Account Supplied');
  }
}

export class InvalidGovernanceAccountError extends Error {
  static readonly code = 6073;
  readonly code = 6073;
  readonly name = 'InvalidGovernanceAccountError';
  readonly msg = 'An Invalid Governance Account was supplied';

  constructor(readonly logs?: string[]) {
    super('6073: An Invalid Governance Account was supplied');
  }
}

export class MissingOptionalAccount extends Error {
  static readonly code = 6074;
  readonly code = 6074;
  readonly name = 'MissingOptionalAccount';
  readonly msg = 'Expected an optional account';

  constructor(readonly logs?: string[]) {
    super('6074: Expected an optional account');
  }
}

export class InvalidSpawnRecordOwner extends Error {
  static readonly code = 6075;
  readonly code = 6075;
  readonly name = 'InvalidSpawnRecordOwner';
  readonly msg = 'Invalid Owner for Spawn Record';

  constructor(readonly logs?: string[]) {
    super('6075: Invalid Owner for Spawn Record');
  }
}

export class NoopError extends Error {
  static readonly code = 6076;
  readonly code = 6076;
  readonly name = 'NoopError';
  readonly msg = 'Noop error';

  constructor(readonly logs?: string[]) {
    super('6076: Noop error');
  }
}

export class MissingRequiredAccountsError extends Error {
  static readonly code = 6077;
  readonly code = 6077;
  readonly name = 'MissingRequiredAccountsError';
  readonly msg = 'A required instruction account was not included';

  constructor(readonly logs?: string[]) {
    super('6077: A required instruction account was not included');
  }
}

export class InvalidMintError extends Error {
  static readonly code = 6078;
  readonly code = 6078;
  readonly name = 'InvalidMintError';
  readonly msg = 'Invalid mint account passed for instruction';

  constructor(readonly logs?: string[]) {
    super('6078: Invalid mint account passed for instruction');
  }
}

export class InvalidTokenAccountKeyError extends Error {
  static readonly code = 6079;
  readonly code = 6079;
  readonly name = 'InvalidTokenAccountKeyError';
  readonly msg = 'An invalid token account was passed into the instruction';

  constructor(readonly logs?: string[]) {
    super('6079: An invalid token account was passed into the instruction');
  }
}

export class InvalidJobAccountError extends Error {
  static readonly code = 6080;
  readonly code = 6080;
  readonly name = 'InvalidJobAccountError';

  constructor(readonly logs?: string[]) {
    super('6080: ');
  }
}

export class VoterStakeRegistryError extends Error {
  static readonly code = 6081;
  readonly code = 6081;
  readonly name = 'VoterStakeRegistryError';

  constructor(readonly logs?: string[]) {
    super('6081: ');
  }
}

export class AccountDiscriminatorMismatch extends Error {
  static readonly code = 6082;
  readonly code = 6082;
  readonly name = 'AccountDiscriminatorMismatch';
  readonly msg = 'Account discriminator did not match.';

  constructor(readonly logs?: string[]) {
    super('6082: Account discriminator did not match.');
  }
}

export class FuckingImpossibleError extends Error {
  static readonly code = 6083;
  readonly code = 6083;
  readonly name = 'FuckingImpossibleError';
  readonly msg = 'This error is fucking impossible.';

  constructor(readonly logs?: string[]) {
    super('6083: This error is fucking impossible.');
  }
}

export class InvalidVrfRound extends Error {
  static readonly code = 6084;
  readonly code = 6084;
  readonly name = 'InvalidVrfRound';
  readonly msg = 'Responding to the wrong VRF round';

  constructor(readonly logs?: string[]) {
    super('6084: Responding to the wrong VRF round');
  }
}

export class JobSizeExceeded extends Error {
  static readonly code = 6085;
  readonly code = 6085;
  readonly name = 'JobSizeExceeded';
  readonly msg = 'Job size has exceeded the max of 6400 bytes';

  constructor(readonly logs?: string[]) {
    super('6085: Job size has exceeded the max of 6400 bytes');
  }
}

export class JobChunksExceeded extends Error {
  static readonly code = 6086;
  readonly code = 6086;
  readonly name = 'JobChunksExceeded';
  readonly msg = 'Job loading can only support a maximum of 8 chunks';

  constructor(readonly logs?: string[]) {
    super('6086: Job loading can only support a maximum of 8 chunks');
  }
}

export class JobDataLocked extends Error {
  static readonly code = 6087;
  readonly code = 6087;
  readonly name = 'JobDataLocked';
  readonly msg = 'Job has finished initializing and is immutable';

  constructor(readonly logs?: string[]) {
    super('6087: Job has finished initializing and is immutable');
  }
}

export class JobNotInitialized extends Error {
  static readonly code = 6088;
  readonly code = 6088;
  readonly name = 'JobNotInitialized';
  readonly msg = 'Job account has not finished initializing';

  constructor(readonly logs?: string[]) {
    super('6088: Job account has not finished initializing');
  }
}

export class BufferRelayerIllegalRoundOpenCall extends Error {
  static readonly code = 6089;
  readonly code = 6089;
  readonly name = 'BufferRelayerIllegalRoundOpenCall';
  readonly msg = 'BufferRelayer round open called too early.';

  constructor(readonly logs?: string[]) {
    super('6089: BufferRelayer round open called too early.');
  }
}

export class InvalidSliderAccount extends Error {
  static readonly code = 6090;
  readonly code = 6090;
  readonly name = 'InvalidSliderAccount';
  readonly msg = 'Invalid slider account.';

  constructor(readonly logs?: string[]) {
    super('6090: Invalid slider account.');
  }
}

export function fromCode(code: number, logs?: string[]): CustomError | null {
  switch (code) {
    case 6000:
      return new ArrayOperationError(logs);
    case 6001:
      return new QueueOperationError(logs);
    case 6002:
      return new IncorrectProgramOwnerError(logs);
    case 6003:
      return new InvalidAggregatorRound(logs);
    case 6004:
      return new TooManyAggregatorJobs(logs);
    case 6005:
      return new AggregatorCurrentRoundClosed(logs);
    case 6006:
      return new AggregatorInvalidSaveResult(logs);
    case 6007:
      return new InvalidStrDecimalConversion(logs);
    case 6008:
      return new AccountLoaderMissingSignature(logs);
    case 6009:
      return new MissingRequiredSignature(logs);
    case 6010:
      return new ArrayOverflowError(logs);
    case 6011:
      return new ArrayUnderflowError(logs);
    case 6012:
      return new PubkeyNotFoundError(logs);
    case 6013:
      return new AggregatorIllegalRoundOpenCall(logs);
    case 6014:
      return new AggregatorIllegalRoundCloseCall(logs);
    case 6015:
      return new AggregatorClosedError(logs);
    case 6016:
      return new IllegalOracleIdxError(logs);
    case 6017:
      return new OracleAlreadyRespondedError(logs);
    case 6018:
      return new ProtoDeserializeError(logs);
    case 6019:
      return new UnauthorizedStateUpdateError(logs);
    case 6020:
      return new MissingOracleAccountsError(logs);
    case 6021:
      return new OracleMismatchError(logs);
    case 6022:
      return new CrankMaxCapacityError(logs);
    case 6023:
      return new AggregatorLeaseInsufficientFunds(logs);
    case 6024:
      return new IncorrectTokenAccountMint(logs);
    case 6025:
      return new InvalidEscrowAccount(logs);
    case 6026:
      return new CrankEmptyError(logs);
    case 6027:
      return new PdaDeriveError(logs);
    case 6028:
      return new AggregatorAccountNotFound(logs);
    case 6029:
      return new PermissionAccountNotFound(logs);
    case 6030:
      return new LeaseAccountDeriveFailure(logs);
    case 6031:
      return new PermissionAccountDeriveFailure(logs);
    case 6032:
      return new EscrowAccountNotFound(logs);
    case 6033:
      return new LeaseAccountNotFound(logs);
    case 6034:
      return new DecimalConversionError(logs);
    case 6035:
      return new PermissionDenied(logs);
    case 6036:
      return new QueueAtCapacity(logs);
    case 6037:
      return new ExcessiveCrankRowsError(logs);
    case 6038:
      return new AggregatorLockedError(logs);
    case 6039:
      return new AggregatorInvalidBatchSizeError(logs);
    case 6040:
      return new AggregatorJobChecksumMismatch(logs);
    case 6041:
      return new IntegerOverflowError(logs);
    case 6042:
      return new InvalidUpdatePeriodError(logs);
    case 6043:
      return new NoResultsError(logs);
    case 6044:
      return new InvalidExpirationError(logs);
    case 6045:
      return new InsufficientStakeError(logs);
    case 6046:
      return new LeaseInactiveError(logs);
    case 6047:
      return new NoAggregatorJobsFound(logs);
    case 6048:
      return new IntegerUnderflowError(logs);
    case 6049:
      return new OracleQueueMismatch(logs);
    case 6050:
      return new OracleWalletMismatchError(logs);
    case 6051:
      return new InvalidBufferAccountError(logs);
    case 6052:
      return new InsufficientOracleQueueError(logs);
    case 6053:
      return new InvalidAuthorityError(logs);
    case 6054:
      return new InvalidTokenAccountMintError(logs);
    case 6055:
      return new ExcessiveLeaseWithdrawlError(logs);
    case 6056:
      return new InvalideHistoryAccountError(logs);
    case 6057:
      return new InvalidLeaseAccountEscrowError(logs);
    case 6058:
      return new InvalidCrankAccountError(logs);
    case 6059:
      return new CrankNoElementsReadyError(logs);
    case 6060:
      return new IndexOutOfBoundsError(logs);
    case 6061:
      return new VrfInvalidRequestError(logs);
    case 6062:
      return new VrfInvalidProofSubmissionError(logs);
    case 6063:
      return new VrfVerifyError(logs);
    case 6064:
      return new VrfCallbackError(logs);
    case 6065:
      return new VrfCallbackParamsError(logs);
    case 6066:
      return new VrfCallbackAlreadyCalledError(logs);
    case 6067:
      return new VrfInvalidPubkeyError(logs);
    case 6068:
      return new VrfTooManyVerifyCallsError(logs);
    case 6069:
      return new VrfRequestAlreadyLaunchedError(logs);
    case 6070:
      return new VrfInsufficientVerificationError(logs);
    case 6071:
      return new InvalidVrfProducerError(logs);
    case 6072:
      return new InvalidGovernancePidError(logs);
    case 6073:
      return new InvalidGovernanceAccountError(logs);
    case 6074:
      return new MissingOptionalAccount(logs);
    case 6075:
      return new InvalidSpawnRecordOwner(logs);
    case 6076:
      return new NoopError(logs);
    case 6077:
      return new MissingRequiredAccountsError(logs);
    case 6078:
      return new InvalidMintError(logs);
    case 6079:
      return new InvalidTokenAccountKeyError(logs);
    case 6080:
      return new InvalidJobAccountError(logs);
    case 6081:
      return new VoterStakeRegistryError(logs);
    case 6082:
      return new AccountDiscriminatorMismatch(logs);
    case 6083:
      return new FuckingImpossibleError(logs);
    case 6084:
      return new InvalidVrfRound(logs);
    case 6085:
      return new JobSizeExceeded(logs);
    case 6086:
      return new JobChunksExceeded(logs);
    case 6087:
      return new JobDataLocked(logs);
    case 6088:
      return new JobNotInitialized(logs);
    case 6089:
      return new BufferRelayerIllegalRoundOpenCall(logs);
    case 6090:
      return new InvalidSliderAccount(logs);
  }

  return null;
}
