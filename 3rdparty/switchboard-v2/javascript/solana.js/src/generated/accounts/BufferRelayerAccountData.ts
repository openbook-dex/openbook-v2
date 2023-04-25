import { SwitchboardProgram } from '../../program';
import { PublicKey, Connection } from '@solana/web3.js';
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars

export interface BufferRelayerAccountDataFields {
  /** Name of the buffer account to store on-chain. */
  name: Array<number>;
  /** Public key of the OracleQueueAccountData that is currently assigned to fulfill buffer relayer update request. */
  queuePubkey: PublicKey;
  /** Token account to reward oracles for completing update request. */
  escrow: PublicKey;
  /** The account delegated as the authority for making account changes. */
  authority: PublicKey;
  /** Public key of the JobAccountData that defines how the buffer relayer is updated. */
  jobPubkey: PublicKey;
  /** Used to protect against malicious RPC nodes providing incorrect task definitions to oracles before fulfillment */
  jobHash: Array<number>;
  /** Minimum delay between update request. */
  minUpdateDelaySeconds: number;
  /** Whether buffer relayer config is locked for further changes. */
  isLocked: boolean;
  /** The current buffer relayer update round that is yet to be confirmed. */
  currentRound: types.BufferRelayerRoundFields;
  /** The latest confirmed buffer relayer update round. */
  latestConfirmedRound: types.BufferRelayerRoundFields;
  /** The buffer holding the latest confirmed result. */
  result: Uint8Array;
}

export interface BufferRelayerAccountDataJSON {
  /** Name of the buffer account to store on-chain. */
  name: Array<number>;
  /** Public key of the OracleQueueAccountData that is currently assigned to fulfill buffer relayer update request. */
  queuePubkey: string;
  /** Token account to reward oracles for completing update request. */
  escrow: string;
  /** The account delegated as the authority for making account changes. */
  authority: string;
  /** Public key of the JobAccountData that defines how the buffer relayer is updated. */
  jobPubkey: string;
  /** Used to protect against malicious RPC nodes providing incorrect task definitions to oracles before fulfillment */
  jobHash: Array<number>;
  /** Minimum delay between update request. */
  minUpdateDelaySeconds: number;
  /** Whether buffer relayer config is locked for further changes. */
  isLocked: boolean;
  /** The current buffer relayer update round that is yet to be confirmed. */
  currentRound: types.BufferRelayerRoundJSON;
  /** The latest confirmed buffer relayer update round. */
  latestConfirmedRound: types.BufferRelayerRoundJSON;
  /** The buffer holding the latest confirmed result. */
  result: Array<number>;
}

export class BufferRelayerAccountData {
  /** Name of the buffer account to store on-chain. */
  readonly name: Array<number>;
  /** Public key of the OracleQueueAccountData that is currently assigned to fulfill buffer relayer update request. */
  readonly queuePubkey: PublicKey;
  /** Token account to reward oracles for completing update request. */
  readonly escrow: PublicKey;
  /** The account delegated as the authority for making account changes. */
  readonly authority: PublicKey;
  /** Public key of the JobAccountData that defines how the buffer relayer is updated. */
  readonly jobPubkey: PublicKey;
  /** Used to protect against malicious RPC nodes providing incorrect task definitions to oracles before fulfillment */
  readonly jobHash: Array<number>;
  /** Minimum delay between update request. */
  readonly minUpdateDelaySeconds: number;
  /** Whether buffer relayer config is locked for further changes. */
  readonly isLocked: boolean;
  /** The current buffer relayer update round that is yet to be confirmed. */
  readonly currentRound: types.BufferRelayerRound;
  /** The latest confirmed buffer relayer update round. */
  readonly latestConfirmedRound: types.BufferRelayerRound;
  /** The buffer holding the latest confirmed result. */
  readonly result: Uint8Array;

  static readonly discriminator = Buffer.from([
    50, 35, 51, 115, 169, 219, 158, 52,
  ]);

  static readonly layout = borsh.struct([
    borsh.array(borsh.u8(), 32, 'name'),
    borsh.publicKey('queuePubkey'),
    borsh.publicKey('escrow'),
    borsh.publicKey('authority'),
    borsh.publicKey('jobPubkey'),
    borsh.array(borsh.u8(), 32, 'jobHash'),
    borsh.u32('minUpdateDelaySeconds'),
    borsh.bool('isLocked'),
    types.BufferRelayerRound.layout('currentRound'),
    types.BufferRelayerRound.layout('latestConfirmedRound'),
    borsh.vecU8('result'),
  ]);

  constructor(fields: BufferRelayerAccountDataFields) {
    this.name = fields.name;
    this.queuePubkey = fields.queuePubkey;
    this.escrow = fields.escrow;
    this.authority = fields.authority;
    this.jobPubkey = fields.jobPubkey;
    this.jobHash = fields.jobHash;
    this.minUpdateDelaySeconds = fields.minUpdateDelaySeconds;
    this.isLocked = fields.isLocked;
    this.currentRound = new types.BufferRelayerRound({
      ...fields.currentRound,
    });
    this.latestConfirmedRound = new types.BufferRelayerRound({
      ...fields.latestConfirmedRound,
    });
    this.result = fields.result;
  }

  static async fetch(
    program: SwitchboardProgram,
    address: PublicKey
  ): Promise<BufferRelayerAccountData | null> {
    const info = await program.connection.getAccountInfo(address);

    if (info === null) {
      return null;
    }
    if (!info.owner.equals(program.programId)) {
      throw new Error("account doesn't belong to this program");
    }

    return this.decode(info.data);
  }

  static async fetchMultiple(
    program: SwitchboardProgram,
    addresses: PublicKey[]
  ): Promise<Array<BufferRelayerAccountData | null>> {
    const infos = await program.connection.getMultipleAccountsInfo(addresses);

    return infos.map(info => {
      if (info === null) {
        return null;
      }
      if (!info.owner.equals(program.programId)) {
        throw new Error("account doesn't belong to this program");
      }

      return this.decode(info.data);
    });
  }

  static decode(data: Buffer): BufferRelayerAccountData {
    if (!data.slice(0, 8).equals(BufferRelayerAccountData.discriminator)) {
      throw new Error('invalid account discriminator');
    }

    const dec = BufferRelayerAccountData.layout.decode(data.slice(8));

    return new BufferRelayerAccountData({
      name: dec.name,
      queuePubkey: dec.queuePubkey,
      escrow: dec.escrow,
      authority: dec.authority,
      jobPubkey: dec.jobPubkey,
      jobHash: dec.jobHash,
      minUpdateDelaySeconds: dec.minUpdateDelaySeconds,
      isLocked: dec.isLocked,
      currentRound: types.BufferRelayerRound.fromDecoded(dec.currentRound),
      latestConfirmedRound: types.BufferRelayerRound.fromDecoded(
        dec.latestConfirmedRound
      ),
      result: new Uint8Array(
        dec.result.buffer,
        dec.result.byteOffset,
        dec.result.length
      ),
    });
  }

  toJSON(): BufferRelayerAccountDataJSON {
    return {
      name: this.name,
      queuePubkey: this.queuePubkey.toString(),
      escrow: this.escrow.toString(),
      authority: this.authority.toString(),
      jobPubkey: this.jobPubkey.toString(),
      jobHash: this.jobHash,
      minUpdateDelaySeconds: this.minUpdateDelaySeconds,
      isLocked: this.isLocked,
      currentRound: this.currentRound.toJSON(),
      latestConfirmedRound: this.latestConfirmedRound.toJSON(),
      result: Array.from(this.result.values()),
    };
  }

  static fromJSON(obj: BufferRelayerAccountDataJSON): BufferRelayerAccountData {
    return new BufferRelayerAccountData({
      name: obj.name,
      queuePubkey: new PublicKey(obj.queuePubkey),
      escrow: new PublicKey(obj.escrow),
      authority: new PublicKey(obj.authority),
      jobPubkey: new PublicKey(obj.jobPubkey),
      jobHash: obj.jobHash,
      minUpdateDelaySeconds: obj.minUpdateDelaySeconds,
      isLocked: obj.isLocked,
      currentRound: types.BufferRelayerRound.fromJSON(obj.currentRound),
      latestConfirmedRound: types.BufferRelayerRound.fromJSON(
        obj.latestConfirmedRound
      ),
      result: Uint8Array.from(obj.result),
    });
  }
}
