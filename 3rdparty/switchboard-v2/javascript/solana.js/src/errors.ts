import * as anchor from '@project-serum/anchor';
import { PublicKey } from '@solana/web3.js';

export class SwitchboardProgramIsBrowserError extends Error {
  constructor() {
    super("SwitchboardProgram can't sign and submit from browsers.");
    Object.setPrototypeOf(this, SwitchboardProgramIsBrowserError.prototype);
  }
}
export class SwitchboardProgramReadOnlyError extends Error {
  constructor() {
    super('SwitchboardProgram is in Read-Only mode, no keypair was provided.');
    Object.setPrototypeOf(this, SwitchboardProgramReadOnlyError.prototype);
  }
}
export class ExistingKeypair extends Error {
  constructor() {
    super('Provided keypair corresponds to an existing account.');
    Object.setPrototypeOf(this, ExistingKeypair.prototype);
  }
}
export class AccountNotFoundError extends Error {
  constructor(label: string, publicKey: anchor.web3.PublicKey) {
    super(`Failed to find ${label} at address ${publicKey.toBase58()}`);
    Object.setPrototypeOf(this, AccountNotFoundError.prototype);
  }
}
export class InstructionsPackingError extends Error {
  constructor() {
    super('Each instruction group must fit into a single transaction');
    Object.setPrototypeOf(this, InstructionsPackingError.prototype);
  }
}
export class NativeMintOnlyError extends Error {
  constructor() {
    super('Wrap/Unwrap can only be called on a native mint');
    Object.setPrototypeOf(this, NativeMintOnlyError.prototype);
  }
}
export class InsufficientFundsError extends Error {
  constructor() {
    super('Insufficient funds to perform this action');
    Object.setPrototypeOf(this, InsufficientFundsError.prototype);
  }
}
export class TransactionOverflowError extends Error {
  constructor(msg: string) {
    super(`TransactionOverflowError: ${msg}`);
    Object.setPrototypeOf(this, TransactionOverflowError.prototype);
  }
}
export class TransactionInstructionOverflowError extends TransactionOverflowError {
  constructor(numIxns: number) {
    super(`number of instructions exceeded (${numIxns})`);
    Object.setPrototypeOf(this, TransactionInstructionOverflowError.prototype);
  }
}
export class TransactionSerializationOverflowError extends TransactionOverflowError {
  constructor(numBytes: number) {
    super(`serialized transaction size exceeded (${numBytes})`);
    Object.setPrototypeOf(
      this,
      TransactionSerializationOverflowError.prototype
    );
  }
}
export class TransactionMissingSignerError extends Error {
  constructor(signers: string[]) {
    super(`missing signers [${signers.join(', ')}]`);
    Object.setPrototypeOf(this, TransactionMissingSignerError.prototype);
  }
}
export class IncorrectAuthority extends Error {
  constructor(expectedAuthority: PublicKey, receivedAuthority: PublicKey) {
    super(
      `incorrect authority, expected ${expectedAuthority}, received ${receivedAuthority}`
    );
    Object.setPrototypeOf(this, IncorrectAuthority.prototype);
  }
}
export class AggregatorConfigError extends Error {
  constructor(property: string, message: string) {
    super(`failed to validate property '${property}' - ${message}`);
    Object.setPrototypeOf(this, AggregatorConfigError.prototype);
  }
}
