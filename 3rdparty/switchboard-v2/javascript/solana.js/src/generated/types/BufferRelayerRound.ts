import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface BufferRelayerRoundFields {
  /** Number of successful responses. */
  numSuccess: number;
  /** Number of error responses. */
  numError: number;
  /** Slot when the buffer relayer round was opened. */
  roundOpenSlot: BN;
  /** Timestamp when the buffer relayer round was opened. */
  roundOpenTimestamp: BN;
  /** The public key of the oracle fulfilling the buffer relayer update request. */
  oraclePubkey: PublicKey;
}

export interface BufferRelayerRoundJSON {
  /** Number of successful responses. */
  numSuccess: number;
  /** Number of error responses. */
  numError: number;
  /** Slot when the buffer relayer round was opened. */
  roundOpenSlot: string;
  /** Timestamp when the buffer relayer round was opened. */
  roundOpenTimestamp: string;
  /** The public key of the oracle fulfilling the buffer relayer update request. */
  oraclePubkey: string;
}

export class BufferRelayerRound {
  /** Number of successful responses. */
  readonly numSuccess: number;
  /** Number of error responses. */
  readonly numError: number;
  /** Slot when the buffer relayer round was opened. */
  readonly roundOpenSlot: BN;
  /** Timestamp when the buffer relayer round was opened. */
  readonly roundOpenTimestamp: BN;
  /** The public key of the oracle fulfilling the buffer relayer update request. */
  readonly oraclePubkey: PublicKey;

  constructor(fields: BufferRelayerRoundFields) {
    this.numSuccess = fields.numSuccess;
    this.numError = fields.numError;
    this.roundOpenSlot = fields.roundOpenSlot;
    this.roundOpenTimestamp = fields.roundOpenTimestamp;
    this.oraclePubkey = fields.oraclePubkey;
  }

  static layout(property?: string) {
    return borsh.struct(
      [
        borsh.u32('numSuccess'),
        borsh.u32('numError'),
        borsh.u64('roundOpenSlot'),
        borsh.i64('roundOpenTimestamp'),
        borsh.publicKey('oraclePubkey'),
      ],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new BufferRelayerRound({
      numSuccess: obj.numSuccess,
      numError: obj.numError,
      roundOpenSlot: obj.roundOpenSlot,
      roundOpenTimestamp: obj.roundOpenTimestamp,
      oraclePubkey: obj.oraclePubkey,
    });
  }

  static toEncodable(fields: BufferRelayerRoundFields) {
    return {
      numSuccess: fields.numSuccess,
      numError: fields.numError,
      roundOpenSlot: fields.roundOpenSlot,
      roundOpenTimestamp: fields.roundOpenTimestamp,
      oraclePubkey: fields.oraclePubkey,
    };
  }

  toJSON(): BufferRelayerRoundJSON {
    return {
      numSuccess: this.numSuccess,
      numError: this.numError,
      roundOpenSlot: this.roundOpenSlot.toString(),
      roundOpenTimestamp: this.roundOpenTimestamp.toString(),
      oraclePubkey: this.oraclePubkey.toString(),
    };
  }

  static fromJSON(obj: BufferRelayerRoundJSON): BufferRelayerRound {
    return new BufferRelayerRound({
      numSuccess: obj.numSuccess,
      numError: obj.numError,
      roundOpenSlot: new BN(obj.roundOpenSlot),
      roundOpenTimestamp: new BN(obj.roundOpenTimestamp),
      oraclePubkey: new PublicKey(obj.oraclePubkey),
    });
  }

  toEncodable() {
    return BufferRelayerRound.toEncodable(this);
  }
}
