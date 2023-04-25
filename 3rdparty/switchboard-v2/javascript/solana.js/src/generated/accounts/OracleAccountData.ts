import { SwitchboardProgram } from '../../program';
import { PublicKey, Connection } from '@solana/web3.js';
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars

export interface OracleAccountDataFields {
  /** Name of the oracle to store on-chain. */
  name: Array<number>;
  /** Metadata of the oracle to store on-chain. */
  metadata: Array<number>;
  /** The account delegated as the authority for making account changes or withdrawing funds from a staking wallet. */
  oracleAuthority: PublicKey;
  /** Unix timestamp when the oracle last heartbeated */
  lastHeartbeat: BN;
  /** Flag dictating if an oracle is active and has heartbeated before the queue's oracle timeout parameter. */
  numInUse: number;
  /** Stake account and reward/slashing wallet. */
  tokenAccount: PublicKey;
  /** Public key of the oracle queue who has granted it permission to use its resources. */
  queuePubkey: PublicKey;
  /** Oracle track record. */
  metrics: types.OracleMetricsFields;
  /** Reserved for future info. */
  ebuf: Array<number>;
}

export interface OracleAccountDataJSON {
  /** Name of the oracle to store on-chain. */
  name: Array<number>;
  /** Metadata of the oracle to store on-chain. */
  metadata: Array<number>;
  /** The account delegated as the authority for making account changes or withdrawing funds from a staking wallet. */
  oracleAuthority: string;
  /** Unix timestamp when the oracle last heartbeated */
  lastHeartbeat: string;
  /** Flag dictating if an oracle is active and has heartbeated before the queue's oracle timeout parameter. */
  numInUse: number;
  /** Stake account and reward/slashing wallet. */
  tokenAccount: string;
  /** Public key of the oracle queue who has granted it permission to use its resources. */
  queuePubkey: string;
  /** Oracle track record. */
  metrics: types.OracleMetricsJSON;
  /** Reserved for future info. */
  ebuf: Array<number>;
}

export class OracleAccountData {
  /** Name of the oracle to store on-chain. */
  readonly name: Array<number>;
  /** Metadata of the oracle to store on-chain. */
  readonly metadata: Array<number>;
  /** The account delegated as the authority for making account changes or withdrawing funds from a staking wallet. */
  readonly oracleAuthority: PublicKey;
  /** Unix timestamp when the oracle last heartbeated */
  readonly lastHeartbeat: BN;
  /** Flag dictating if an oracle is active and has heartbeated before the queue's oracle timeout parameter. */
  readonly numInUse: number;
  /** Stake account and reward/slashing wallet. */
  readonly tokenAccount: PublicKey;
  /** Public key of the oracle queue who has granted it permission to use its resources. */
  readonly queuePubkey: PublicKey;
  /** Oracle track record. */
  readonly metrics: types.OracleMetrics;
  /** Reserved for future info. */
  readonly ebuf: Array<number>;

  static readonly discriminator = Buffer.from([
    128, 30, 16, 241, 170, 73, 55, 54,
  ]);

  static readonly layout = borsh.struct([
    borsh.array(borsh.u8(), 32, 'name'),
    borsh.array(borsh.u8(), 128, 'metadata'),
    borsh.publicKey('oracleAuthority'),
    borsh.i64('lastHeartbeat'),
    borsh.u32('numInUse'),
    borsh.publicKey('tokenAccount'),
    borsh.publicKey('queuePubkey'),
    types.OracleMetrics.layout('metrics'),
    borsh.array(borsh.u8(), 256, 'ebuf'),
  ]);

  constructor(fields: OracleAccountDataFields) {
    this.name = fields.name;
    this.metadata = fields.metadata;
    this.oracleAuthority = fields.oracleAuthority;
    this.lastHeartbeat = fields.lastHeartbeat;
    this.numInUse = fields.numInUse;
    this.tokenAccount = fields.tokenAccount;
    this.queuePubkey = fields.queuePubkey;
    this.metrics = new types.OracleMetrics({ ...fields.metrics });
    this.ebuf = fields.ebuf;
  }

  static async fetch(
    program: SwitchboardProgram,
    address: PublicKey
  ): Promise<OracleAccountData | null> {
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
  ): Promise<Array<OracleAccountData | null>> {
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

  static decode(data: Buffer): OracleAccountData {
    if (!data.slice(0, 8).equals(OracleAccountData.discriminator)) {
      throw new Error('invalid account discriminator');
    }

    const dec = OracleAccountData.layout.decode(data.slice(8));

    return new OracleAccountData({
      name: dec.name,
      metadata: dec.metadata,
      oracleAuthority: dec.oracleAuthority,
      lastHeartbeat: dec.lastHeartbeat,
      numInUse: dec.numInUse,
      tokenAccount: dec.tokenAccount,
      queuePubkey: dec.queuePubkey,
      metrics: types.OracleMetrics.fromDecoded(dec.metrics),
      ebuf: dec.ebuf,
    });
  }

  toJSON(): OracleAccountDataJSON {
    return {
      name: this.name,
      metadata: this.metadata,
      oracleAuthority: this.oracleAuthority.toString(),
      lastHeartbeat: this.lastHeartbeat.toString(),
      numInUse: this.numInUse,
      tokenAccount: this.tokenAccount.toString(),
      queuePubkey: this.queuePubkey.toString(),
      metrics: this.metrics.toJSON(),
      ebuf: this.ebuf,
    };
  }

  static fromJSON(obj: OracleAccountDataJSON): OracleAccountData {
    return new OracleAccountData({
      name: obj.name,
      metadata: obj.metadata,
      oracleAuthority: new PublicKey(obj.oracleAuthority),
      lastHeartbeat: new BN(obj.lastHeartbeat),
      numInUse: obj.numInUse,
      tokenAccount: new PublicKey(obj.tokenAccount),
      queuePubkey: new PublicKey(obj.queuePubkey),
      metrics: types.OracleMetrics.fromJSON(obj.metrics),
      ebuf: obj.ebuf,
    });
  }
}
