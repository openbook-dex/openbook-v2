import { SwitchboardProgram } from '../../program';
import { PublicKey, Connection } from '@solana/web3.js';
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars

export interface OracleQueueAccountDataFields {
  /** Name of the queue to store on-chain. */
  name: Array<number>;
  /** Metadata of the queue to store on-chain. */
  metadata: Array<number>;
  /** The account delegated as the authority for making account changes or assigning permissions targeted at the queue. */
  authority: PublicKey;
  /** Interval when stale oracles will be removed if they fail to heartbeat. */
  oracleTimeout: number;
  /** Rewards to provide oracles and round openers on this queue. */
  reward: BN;
  /** The minimum amount of stake oracles must present to remain on the queue. */
  minStake: BN;
  /** Whether slashing is enabled on this queue. */
  slashingEnabled: boolean;
  /**
   * The tolerated variance amount oracle results can have from the accepted round result before being slashed.
   * slashBound = varianceToleranceMultiplier * stdDeviation Default: 2
   */
  varianceToleranceMultiplier: types.SwitchboardDecimalFields;
  /**
   * Number of update rounds new feeds are on probation for.
   * If a feed returns 429s within probation period, auto disable permissions.
   */
  feedProbationPeriod: number;
  /** Current index of the oracle rotation. */
  currIdx: number;
  /** Current number of oracles on a queue. */
  size: number;
  /** Garbage collection index. */
  gcIdx: number;
  /** Consecutive failure limit for a feed before feed permission is revoked. */
  consecutiveFeedFailureLimit: BN;
  /** Consecutive failure limit for an oracle before oracle permission is revoked. */
  consecutiveOracleFailureLimit: BN;
  /** Enabling this setting means data feeds do not need explicit permission to join the queue and request new values from its oracles. */
  unpermissionedFeedsEnabled: boolean;
  /** Enabling this setting means VRF accounts do not need explicit permission to join the queue and request new values from its oracles. */
  unpermissionedVrfEnabled: boolean;
  /** TODO: Revenue percentage rewarded to job curators overall. */
  curatorRewardCut: types.SwitchboardDecimalFields;
  /**
   * Prevent new leases from being funded n this queue.
   * Useful to turn down a queue for migrations, since authority is always immutable.
   */
  lockLeaseFunding: boolean;
  /** Token mint used for the oracle queue rewards and slashing. */
  mint: PublicKey;
  /** Whether oracles are permitted to fulfill buffer relayer update request. */
  enableBufferRelayers: boolean;
  /** Reserved for future info. */
  ebuf: Array<number>;
  /** Maximum number of oracles a queue can support. */
  maxSize: number;
  /** The public key of the OracleQueueBuffer account holding a collection of Oracle pubkeys that haver successfully heartbeated before the queues `oracleTimeout`. */
  dataBuffer: PublicKey;
}

export interface OracleQueueAccountDataJSON {
  /** Name of the queue to store on-chain. */
  name: Array<number>;
  /** Metadata of the queue to store on-chain. */
  metadata: Array<number>;
  /** The account delegated as the authority for making account changes or assigning permissions targeted at the queue. */
  authority: string;
  /** Interval when stale oracles will be removed if they fail to heartbeat. */
  oracleTimeout: number;
  /** Rewards to provide oracles and round openers on this queue. */
  reward: string;
  /** The minimum amount of stake oracles must present to remain on the queue. */
  minStake: string;
  /** Whether slashing is enabled on this queue. */
  slashingEnabled: boolean;
  /**
   * The tolerated variance amount oracle results can have from the accepted round result before being slashed.
   * slashBound = varianceToleranceMultiplier * stdDeviation Default: 2
   */
  varianceToleranceMultiplier: types.SwitchboardDecimalJSON;
  /**
   * Number of update rounds new feeds are on probation for.
   * If a feed returns 429s within probation period, auto disable permissions.
   */
  feedProbationPeriod: number;
  /** Current index of the oracle rotation. */
  currIdx: number;
  /** Current number of oracles on a queue. */
  size: number;
  /** Garbage collection index. */
  gcIdx: number;
  /** Consecutive failure limit for a feed before feed permission is revoked. */
  consecutiveFeedFailureLimit: string;
  /** Consecutive failure limit for an oracle before oracle permission is revoked. */
  consecutiveOracleFailureLimit: string;
  /** Enabling this setting means data feeds do not need explicit permission to join the queue and request new values from its oracles. */
  unpermissionedFeedsEnabled: boolean;
  /** Enabling this setting means VRF accounts do not need explicit permission to join the queue and request new values from its oracles. */
  unpermissionedVrfEnabled: boolean;
  /** TODO: Revenue percentage rewarded to job curators overall. */
  curatorRewardCut: types.SwitchboardDecimalJSON;
  /**
   * Prevent new leases from being funded n this queue.
   * Useful to turn down a queue for migrations, since authority is always immutable.
   */
  lockLeaseFunding: boolean;
  /** Token mint used for the oracle queue rewards and slashing. */
  mint: string;
  /** Whether oracles are permitted to fulfill buffer relayer update request. */
  enableBufferRelayers: boolean;
  /** Reserved for future info. */
  ebuf: Array<number>;
  /** Maximum number of oracles a queue can support. */
  maxSize: number;
  /** The public key of the OracleQueueBuffer account holding a collection of Oracle pubkeys that haver successfully heartbeated before the queues `oracleTimeout`. */
  dataBuffer: string;
}

export class OracleQueueAccountData {
  /** Name of the queue to store on-chain. */
  readonly name: Array<number>;
  /** Metadata of the queue to store on-chain. */
  readonly metadata: Array<number>;
  /** The account delegated as the authority for making account changes or assigning permissions targeted at the queue. */
  readonly authority: PublicKey;
  /** Interval when stale oracles will be removed if they fail to heartbeat. */
  readonly oracleTimeout: number;
  /** Rewards to provide oracles and round openers on this queue. */
  readonly reward: BN;
  /** The minimum amount of stake oracles must present to remain on the queue. */
  readonly minStake: BN;
  /** Whether slashing is enabled on this queue. */
  readonly slashingEnabled: boolean;
  /**
   * The tolerated variance amount oracle results can have from the accepted round result before being slashed.
   * slashBound = varianceToleranceMultiplier * stdDeviation Default: 2
   */
  readonly varianceToleranceMultiplier: types.SwitchboardDecimal;
  /**
   * Number of update rounds new feeds are on probation for.
   * If a feed returns 429s within probation period, auto disable permissions.
   */
  readonly feedProbationPeriod: number;
  /** Current index of the oracle rotation. */
  readonly currIdx: number;
  /** Current number of oracles on a queue. */
  readonly size: number;
  /** Garbage collection index. */
  readonly gcIdx: number;
  /** Consecutive failure limit for a feed before feed permission is revoked. */
  readonly consecutiveFeedFailureLimit: BN;
  /** Consecutive failure limit for an oracle before oracle permission is revoked. */
  readonly consecutiveOracleFailureLimit: BN;
  /** Enabling this setting means data feeds do not need explicit permission to join the queue and request new values from its oracles. */
  readonly unpermissionedFeedsEnabled: boolean;
  /** Enabling this setting means VRF accounts do not need explicit permission to join the queue and request new values from its oracles. */
  readonly unpermissionedVrfEnabled: boolean;
  /** TODO: Revenue percentage rewarded to job curators overall. */
  readonly curatorRewardCut: types.SwitchboardDecimal;
  /**
   * Prevent new leases from being funded n this queue.
   * Useful to turn down a queue for migrations, since authority is always immutable.
   */
  readonly lockLeaseFunding: boolean;
  /** Token mint used for the oracle queue rewards and slashing. */
  readonly mint: PublicKey;
  /** Whether oracles are permitted to fulfill buffer relayer update request. */
  readonly enableBufferRelayers: boolean;
  /** Reserved for future info. */
  readonly ebuf: Array<number>;
  /** Maximum number of oracles a queue can support. */
  readonly maxSize: number;
  /** The public key of the OracleQueueBuffer account holding a collection of Oracle pubkeys that haver successfully heartbeated before the queues `oracleTimeout`. */
  readonly dataBuffer: PublicKey;

  static readonly discriminator = Buffer.from([
    164, 207, 200, 51, 199, 113, 35, 109,
  ]);

  static readonly layout = borsh.struct([
    borsh.array(borsh.u8(), 32, 'name'),
    borsh.array(borsh.u8(), 64, 'metadata'),
    borsh.publicKey('authority'),
    borsh.u32('oracleTimeout'),
    borsh.u64('reward'),
    borsh.u64('minStake'),
    borsh.bool('slashingEnabled'),
    types.SwitchboardDecimal.layout('varianceToleranceMultiplier'),
    borsh.u32('feedProbationPeriod'),
    borsh.u32('currIdx'),
    borsh.u32('size'),
    borsh.u32('gcIdx'),
    borsh.u64('consecutiveFeedFailureLimit'),
    borsh.u64('consecutiveOracleFailureLimit'),
    borsh.bool('unpermissionedFeedsEnabled'),
    borsh.bool('unpermissionedVrfEnabled'),
    types.SwitchboardDecimal.layout('curatorRewardCut'),
    borsh.bool('lockLeaseFunding'),
    borsh.publicKey('mint'),
    borsh.bool('enableBufferRelayers'),
    borsh.array(borsh.u8(), 968, 'ebuf'),
    borsh.u32('maxSize'),
    borsh.publicKey('dataBuffer'),
  ]);

  constructor(fields: OracleQueueAccountDataFields) {
    this.name = fields.name;
    this.metadata = fields.metadata;
    this.authority = fields.authority;
    this.oracleTimeout = fields.oracleTimeout;
    this.reward = fields.reward;
    this.minStake = fields.minStake;
    this.slashingEnabled = fields.slashingEnabled;
    this.varianceToleranceMultiplier = new types.SwitchboardDecimal({
      ...fields.varianceToleranceMultiplier,
    });
    this.feedProbationPeriod = fields.feedProbationPeriod;
    this.currIdx = fields.currIdx;
    this.size = fields.size;
    this.gcIdx = fields.gcIdx;
    this.consecutiveFeedFailureLimit = fields.consecutiveFeedFailureLimit;
    this.consecutiveOracleFailureLimit = fields.consecutiveOracleFailureLimit;
    this.unpermissionedFeedsEnabled = fields.unpermissionedFeedsEnabled;
    this.unpermissionedVrfEnabled = fields.unpermissionedVrfEnabled;
    this.curatorRewardCut = new types.SwitchboardDecimal({
      ...fields.curatorRewardCut,
    });
    this.lockLeaseFunding = fields.lockLeaseFunding;
    this.mint = fields.mint;
    this.enableBufferRelayers = fields.enableBufferRelayers;
    this.ebuf = fields.ebuf;
    this.maxSize = fields.maxSize;
    this.dataBuffer = fields.dataBuffer;
  }

  static async fetch(
    program: SwitchboardProgram,
    address: PublicKey
  ): Promise<OracleQueueAccountData | null> {
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
  ): Promise<Array<OracleQueueAccountData | null>> {
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

  static decode(data: Buffer): OracleQueueAccountData {
    if (!data.slice(0, 8).equals(OracleQueueAccountData.discriminator)) {
      throw new Error('invalid account discriminator');
    }

    const dec = OracleQueueAccountData.layout.decode(data.slice(8));

    return new OracleQueueAccountData({
      name: dec.name,
      metadata: dec.metadata,
      authority: dec.authority,
      oracleTimeout: dec.oracleTimeout,
      reward: dec.reward,
      minStake: dec.minStake,
      slashingEnabled: dec.slashingEnabled,
      varianceToleranceMultiplier: types.SwitchboardDecimal.fromDecoded(
        dec.varianceToleranceMultiplier
      ),
      feedProbationPeriod: dec.feedProbationPeriod,
      currIdx: dec.currIdx,
      size: dec.size,
      gcIdx: dec.gcIdx,
      consecutiveFeedFailureLimit: dec.consecutiveFeedFailureLimit,
      consecutiveOracleFailureLimit: dec.consecutiveOracleFailureLimit,
      unpermissionedFeedsEnabled: dec.unpermissionedFeedsEnabled,
      unpermissionedVrfEnabled: dec.unpermissionedVrfEnabled,
      curatorRewardCut: types.SwitchboardDecimal.fromDecoded(
        dec.curatorRewardCut
      ),
      lockLeaseFunding: dec.lockLeaseFunding,
      mint: dec.mint,
      enableBufferRelayers: dec.enableBufferRelayers,
      ebuf: dec.ebuf,
      maxSize: dec.maxSize,
      dataBuffer: dec.dataBuffer,
    });
  }

  toJSON(): OracleQueueAccountDataJSON {
    return {
      name: this.name,
      metadata: this.metadata,
      authority: this.authority.toString(),
      oracleTimeout: this.oracleTimeout,
      reward: this.reward.toString(),
      minStake: this.minStake.toString(),
      slashingEnabled: this.slashingEnabled,
      varianceToleranceMultiplier: this.varianceToleranceMultiplier.toJSON(),
      feedProbationPeriod: this.feedProbationPeriod,
      currIdx: this.currIdx,
      size: this.size,
      gcIdx: this.gcIdx,
      consecutiveFeedFailureLimit: this.consecutiveFeedFailureLimit.toString(),
      consecutiveOracleFailureLimit:
        this.consecutiveOracleFailureLimit.toString(),
      unpermissionedFeedsEnabled: this.unpermissionedFeedsEnabled,
      unpermissionedVrfEnabled: this.unpermissionedVrfEnabled,
      curatorRewardCut: this.curatorRewardCut.toJSON(),
      lockLeaseFunding: this.lockLeaseFunding,
      mint: this.mint.toString(),
      enableBufferRelayers: this.enableBufferRelayers,
      ebuf: this.ebuf,
      maxSize: this.maxSize,
      dataBuffer: this.dataBuffer.toString(),
    };
  }

  static fromJSON(obj: OracleQueueAccountDataJSON): OracleQueueAccountData {
    return new OracleQueueAccountData({
      name: obj.name,
      metadata: obj.metadata,
      authority: new PublicKey(obj.authority),
      oracleTimeout: obj.oracleTimeout,
      reward: new BN(obj.reward),
      minStake: new BN(obj.minStake),
      slashingEnabled: obj.slashingEnabled,
      varianceToleranceMultiplier: types.SwitchboardDecimal.fromJSON(
        obj.varianceToleranceMultiplier
      ),
      feedProbationPeriod: obj.feedProbationPeriod,
      currIdx: obj.currIdx,
      size: obj.size,
      gcIdx: obj.gcIdx,
      consecutiveFeedFailureLimit: new BN(obj.consecutiveFeedFailureLimit),
      consecutiveOracleFailureLimit: new BN(obj.consecutiveOracleFailureLimit),
      unpermissionedFeedsEnabled: obj.unpermissionedFeedsEnabled,
      unpermissionedVrfEnabled: obj.unpermissionedVrfEnabled,
      curatorRewardCut: types.SwitchboardDecimal.fromJSON(obj.curatorRewardCut),
      lockLeaseFunding: obj.lockLeaseFunding,
      mint: new PublicKey(obj.mint),
      enableBufferRelayers: obj.enableBufferRelayers,
      ebuf: obj.ebuf,
      maxSize: obj.maxSize,
      dataBuffer: new PublicKey(obj.dataBuffer),
    });
  }
}
