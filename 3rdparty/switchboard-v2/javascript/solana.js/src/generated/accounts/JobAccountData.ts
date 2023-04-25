import { SwitchboardProgram } from '../../program';
import { PublicKey, Connection } from '@solana/web3.js';
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars

export interface JobAccountDataFields {
  /** Name of the job to store on-chain. */
  name: Array<number>;
  /** Metadata of the job to store on-chain. */
  metadata: Array<number>;
  /** The account delegated as the authority for making account changes. */
  authority: PublicKey;
  /** Unix timestamp when the job is considered invalid */
  expiration: BN;
  /** Hash of the serialized data to prevent tampering. */
  hash: Array<number>;
  /** Serialized protobuf containing the collection of task to retrieve data off-chain. */
  data: Uint8Array;
  /** The number of data feeds referencing the job account.. */
  referenceCount: number;
  /** The token amount funded into a feed that contains this job account. */
  totalSpent: BN;
  /** Unix timestamp when the job was created on-chain. */
  createdAt: BN;
  isInitializing: number;
}

export interface JobAccountDataJSON {
  /** Name of the job to store on-chain. */
  name: Array<number>;
  /** Metadata of the job to store on-chain. */
  metadata: Array<number>;
  /** The account delegated as the authority for making account changes. */
  authority: string;
  /** Unix timestamp when the job is considered invalid */
  expiration: string;
  /** Hash of the serialized data to prevent tampering. */
  hash: Array<number>;
  /** Serialized protobuf containing the collection of task to retrieve data off-chain. */
  data: Array<number>;
  /** The number of data feeds referencing the job account.. */
  referenceCount: number;
  /** The token amount funded into a feed that contains this job account. */
  totalSpent: string;
  /** Unix timestamp when the job was created on-chain. */
  createdAt: string;
  isInitializing: number;
}

export class JobAccountData {
  /** Name of the job to store on-chain. */
  readonly name: Array<number>;
  /** Metadata of the job to store on-chain. */
  readonly metadata: Array<number>;
  /** The account delegated as the authority for making account changes. */
  readonly authority: PublicKey;
  /** Unix timestamp when the job is considered invalid */
  readonly expiration: BN;
  /** Hash of the serialized data to prevent tampering. */
  readonly hash: Array<number>;
  /** Serialized protobuf containing the collection of task to retrieve data off-chain. */
  readonly data: Uint8Array;
  /** The number of data feeds referencing the job account.. */
  readonly referenceCount: number;
  /** The token amount funded into a feed that contains this job account. */
  readonly totalSpent: BN;
  /** Unix timestamp when the job was created on-chain. */
  readonly createdAt: BN;
  readonly isInitializing: number;

  static readonly discriminator = Buffer.from([
    124, 69, 101, 195, 229, 218, 144, 63,
  ]);

  static readonly layout = borsh.struct([
    borsh.array(borsh.u8(), 32, 'name'),
    borsh.array(borsh.u8(), 64, 'metadata'),
    borsh.publicKey('authority'),
    borsh.i64('expiration'),
    borsh.array(borsh.u8(), 32, 'hash'),
    borsh.vecU8('data'),
    borsh.u32('referenceCount'),
    borsh.u64('totalSpent'),
    borsh.i64('createdAt'),
    borsh.u8('isInitializing'),
  ]);

  constructor(fields: JobAccountDataFields) {
    this.name = fields.name;
    this.metadata = fields.metadata;
    this.authority = fields.authority;
    this.expiration = fields.expiration;
    this.hash = fields.hash;
    this.data = fields.data;
    this.referenceCount = fields.referenceCount;
    this.totalSpent = fields.totalSpent;
    this.createdAt = fields.createdAt;
    this.isInitializing = fields.isInitializing;
  }

  static async fetch(
    program: SwitchboardProgram,
    address: PublicKey
  ): Promise<JobAccountData | null> {
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
  ): Promise<Array<JobAccountData | null>> {
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

  static decode(data: Buffer): JobAccountData {
    if (!data.slice(0, 8).equals(JobAccountData.discriminator)) {
      throw new Error('invalid account discriminator');
    }

    const dec = JobAccountData.layout.decode(data.slice(8));

    return new JobAccountData({
      name: dec.name,
      metadata: dec.metadata,
      authority: dec.authority,
      expiration: dec.expiration,
      hash: dec.hash,
      data: new Uint8Array(
        dec.data.buffer,
        dec.data.byteOffset,
        dec.data.length
      ),
      referenceCount: dec.referenceCount,
      totalSpent: dec.totalSpent,
      createdAt: dec.createdAt,
      isInitializing: dec.isInitializing,
    });
  }

  toJSON(): JobAccountDataJSON {
    return {
      name: this.name,
      metadata: this.metadata,
      authority: this.authority.toString(),
      expiration: this.expiration.toString(),
      hash: this.hash,
      data: Array.from(this.data.values()),
      referenceCount: this.referenceCount,
      totalSpent: this.totalSpent.toString(),
      createdAt: this.createdAt.toString(),
      isInitializing: this.isInitializing,
    };
  }

  static fromJSON(obj: JobAccountDataJSON): JobAccountData {
    return new JobAccountData({
      name: obj.name,
      metadata: obj.metadata,
      authority: new PublicKey(obj.authority),
      expiration: new BN(obj.expiration),
      hash: obj.hash,
      data: Uint8Array.from(obj.data),
      referenceCount: obj.referenceCount,
      totalSpent: new BN(obj.totalSpent),
      createdAt: new BN(obj.createdAt),
      isInitializing: obj.isInitializing,
    });
  }
}
