import { SwitchboardProgram } from '../../program';
import { PublicKey, Connection } from '@solana/web3.js';
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars

export interface AggregatorAccountDataFields {
  /** Name of the aggregator to store on-chain. */
  name: Array<number>;
  /** Metadata of the aggregator to store on-chain. */
  metadata: Array<number>;
  /** Reserved. */
  reserved1: Array<number>;
  /** Pubkey of the queue the aggregator belongs to. */
  queuePubkey: PublicKey;
  /**
   * CONFIGS
   * Number of oracles assigned to an update request.
   */
  oracleRequestBatchSize: number;
  /** Minimum number of oracle responses required before a round is validated. */
  minOracleResults: number;
  /** Minimum number of job results before an oracle accepts a result. */
  minJobResults: number;
  /** Minimum number of seconds required between aggregator rounds. */
  minUpdateDelaySeconds: number;
  /** Unix timestamp for which no feed update will occur before. */
  startAfter: BN;
  /** Change percentage required between a previous round and the current round. If variance percentage is not met, reject new oracle responses. */
  varianceThreshold: types.SwitchboardDecimalFields;
  /** Number of seconds for which, even if the variance threshold is not passed, accept new responses from oracles. */
  forceReportPeriod: BN;
  /** Timestamp when the feed is no longer needed. */
  expiration: BN;
  /** Counter for the number of consecutive failures before a feed is removed from a queue. If set to 0, failed feeds will remain on the queue. */
  consecutiveFailureCount: BN;
  /** Timestamp when the next update request will be available. */
  nextAllowedUpdateTime: BN;
  /** Flag for whether an aggregators configuration is locked for editing. */
  isLocked: boolean;
  /** Optional, public key of the crank the aggregator is currently using. Event based feeds do not need a crank. */
  crankPubkey: PublicKey;
  /** Latest confirmed update request result that has been accepted as valid. */
  latestConfirmedRound: types.AggregatorRoundFields;
  /** Oracle results from the current round of update request that has not been accepted as valid yet. */
  currentRound: types.AggregatorRoundFields;
  /** List of public keys containing the job definitions for how data is sourced off-chain by oracles. */
  jobPubkeysData: Array<PublicKey>;
  /** Used to protect against malicious RPC nodes providing incorrect task definitions to oracles before fulfillment. */
  jobHashes: Array<types.HashFields>;
  /** Number of jobs assigned to an oracle. */
  jobPubkeysSize: number;
  /** Used to protect against malicious RPC nodes providing incorrect task definitions to oracles before fulfillment. */
  jobsChecksum: Array<number>;
  /** The account delegated as the authority for making account changes. */
  authority: PublicKey;
  /** Optional, public key of a history buffer account storing the last N accepted results and their timestamps. */
  historyBuffer: PublicKey;
  /** The previous confirmed round result. */
  previousConfirmedRoundResult: types.SwitchboardDecimalFields;
  /** The slot when the previous confirmed round was opened. */
  previousConfirmedRoundSlot: BN;
  /** Whether an aggregator is permitted to join a crank. */
  disableCrank: boolean;
  /** Job weights used for the weighted median of the aggregator's assigned job accounts. */
  jobWeights: Array<number>;
  /** Unix timestamp when the feed was created. */
  creationTimestamp: BN;
  /**
   * Use sliding window or round based resolution
   * NOTE: This changes result propogation in latest_round_result
   */
  resolutionMode: types.AggregatorResolutionModeKind;
  basePriorityFee: number;
  priorityFeeBump: number;
  priorityFeeBumpPeriod: number;
  maxPriorityFeeMultiplier: number;
  /** Reserved for future info. */
  ebuf: Array<number>;
}

export interface AggregatorAccountDataJSON {
  /** Name of the aggregator to store on-chain. */
  name: Array<number>;
  /** Metadata of the aggregator to store on-chain. */
  metadata: Array<number>;
  /** Reserved. */
  reserved1: Array<number>;
  /** Pubkey of the queue the aggregator belongs to. */
  queuePubkey: string;
  /**
   * CONFIGS
   * Number of oracles assigned to an update request.
   */
  oracleRequestBatchSize: number;
  /** Minimum number of oracle responses required before a round is validated. */
  minOracleResults: number;
  /** Minimum number of job results before an oracle accepts a result. */
  minJobResults: number;
  /** Minimum number of seconds required between aggregator rounds. */
  minUpdateDelaySeconds: number;
  /** Unix timestamp for which no feed update will occur before. */
  startAfter: string;
  /** Change percentage required between a previous round and the current round. If variance percentage is not met, reject new oracle responses. */
  varianceThreshold: types.SwitchboardDecimalJSON;
  /** Number of seconds for which, even if the variance threshold is not passed, accept new responses from oracles. */
  forceReportPeriod: string;
  /** Timestamp when the feed is no longer needed. */
  expiration: string;
  /** Counter for the number of consecutive failures before a feed is removed from a queue. If set to 0, failed feeds will remain on the queue. */
  consecutiveFailureCount: string;
  /** Timestamp when the next update request will be available. */
  nextAllowedUpdateTime: string;
  /** Flag for whether an aggregators configuration is locked for editing. */
  isLocked: boolean;
  /** Optional, public key of the crank the aggregator is currently using. Event based feeds do not need a crank. */
  crankPubkey: string;
  /** Latest confirmed update request result that has been accepted as valid. */
  latestConfirmedRound: types.AggregatorRoundJSON;
  /** Oracle results from the current round of update request that has not been accepted as valid yet. */
  currentRound: types.AggregatorRoundJSON;
  /** List of public keys containing the job definitions for how data is sourced off-chain by oracles. */
  jobPubkeysData: Array<string>;
  /** Used to protect against malicious RPC nodes providing incorrect task definitions to oracles before fulfillment. */
  jobHashes: Array<types.HashJSON>;
  /** Number of jobs assigned to an oracle. */
  jobPubkeysSize: number;
  /** Used to protect against malicious RPC nodes providing incorrect task definitions to oracles before fulfillment. */
  jobsChecksum: Array<number>;
  /** The account delegated as the authority for making account changes. */
  authority: string;
  /** Optional, public key of a history buffer account storing the last N accepted results and their timestamps. */
  historyBuffer: string;
  /** The previous confirmed round result. */
  previousConfirmedRoundResult: types.SwitchboardDecimalJSON;
  /** The slot when the previous confirmed round was opened. */
  previousConfirmedRoundSlot: string;
  /** Whether an aggregator is permitted to join a crank. */
  disableCrank: boolean;
  /** Job weights used for the weighted median of the aggregator's assigned job accounts. */
  jobWeights: Array<number>;
  /** Unix timestamp when the feed was created. */
  creationTimestamp: string;
  /**
   * Use sliding window or round based resolution
   * NOTE: This changes result propogation in latest_round_result
   */
  resolutionMode: types.AggregatorResolutionModeJSON;
  basePriorityFee: number;
  priorityFeeBump: number;
  priorityFeeBumpPeriod: number;
  maxPriorityFeeMultiplier: number;
  /** Reserved for future info. */
  ebuf: Array<number>;
}

export class AggregatorAccountData {
  /** Name of the aggregator to store on-chain. */
  readonly name: Array<number>;
  /** Metadata of the aggregator to store on-chain. */
  readonly metadata: Array<number>;
  /** Reserved. */
  readonly reserved1: Array<number>;
  /** Pubkey of the queue the aggregator belongs to. */
  readonly queuePubkey: PublicKey;
  /**
   * CONFIGS
   * Number of oracles assigned to an update request.
   */
  readonly oracleRequestBatchSize: number;
  /** Minimum number of oracle responses required before a round is validated. */
  readonly minOracleResults: number;
  /** Minimum number of job results before an oracle accepts a result. */
  readonly minJobResults: number;
  /** Minimum number of seconds required between aggregator rounds. */
  readonly minUpdateDelaySeconds: number;
  /** Unix timestamp for which no feed update will occur before. */
  readonly startAfter: BN;
  /** Change percentage required between a previous round and the current round. If variance percentage is not met, reject new oracle responses. */
  readonly varianceThreshold: types.SwitchboardDecimal;
  /** Number of seconds for which, even if the variance threshold is not passed, accept new responses from oracles. */
  readonly forceReportPeriod: BN;
  /** Timestamp when the feed is no longer needed. */
  readonly expiration: BN;
  /** Counter for the number of consecutive failures before a feed is removed from a queue. If set to 0, failed feeds will remain on the queue. */
  readonly consecutiveFailureCount: BN;
  /** Timestamp when the next update request will be available. */
  readonly nextAllowedUpdateTime: BN;
  /** Flag for whether an aggregators configuration is locked for editing. */
  readonly isLocked: boolean;
  /** Optional, public key of the crank the aggregator is currently using. Event based feeds do not need a crank. */
  readonly crankPubkey: PublicKey;
  /** Latest confirmed update request result that has been accepted as valid. */
  readonly latestConfirmedRound: types.AggregatorRound;
  /** Oracle results from the current round of update request that has not been accepted as valid yet. */
  readonly currentRound: types.AggregatorRound;
  /** List of public keys containing the job definitions for how data is sourced off-chain by oracles. */
  readonly jobPubkeysData: Array<PublicKey>;
  /** Used to protect against malicious RPC nodes providing incorrect task definitions to oracles before fulfillment. */
  readonly jobHashes: Array<types.Hash>;
  /** Number of jobs assigned to an oracle. */
  readonly jobPubkeysSize: number;
  /** Used to protect against malicious RPC nodes providing incorrect task definitions to oracles before fulfillment. */
  readonly jobsChecksum: Array<number>;
  /** The account delegated as the authority for making account changes. */
  readonly authority: PublicKey;
  /** Optional, public key of a history buffer account storing the last N accepted results and their timestamps. */
  readonly historyBuffer: PublicKey;
  /** The previous confirmed round result. */
  readonly previousConfirmedRoundResult: types.SwitchboardDecimal;
  /** The slot when the previous confirmed round was opened. */
  readonly previousConfirmedRoundSlot: BN;
  /** Whether an aggregator is permitted to join a crank. */
  readonly disableCrank: boolean;
  /** Job weights used for the weighted median of the aggregator's assigned job accounts. */
  readonly jobWeights: Array<number>;
  /** Unix timestamp when the feed was created. */
  readonly creationTimestamp: BN;
  /**
   * Use sliding window or round based resolution
   * NOTE: This changes result propogation in latest_round_result
   */
  readonly resolutionMode: types.AggregatorResolutionModeKind;
  readonly basePriorityFee: number;
  readonly priorityFeeBump: number;
  readonly priorityFeeBumpPeriod: number;
  readonly maxPriorityFeeMultiplier: number;
  /** Reserved for future info. */
  readonly ebuf: Array<number>;

  static readonly discriminator = Buffer.from([
    217, 230, 65, 101, 201, 162, 27, 125,
  ]);

  static readonly layout = borsh.struct([
    borsh.array(borsh.u8(), 32, 'name'),
    borsh.array(borsh.u8(), 128, 'metadata'),
    borsh.array(borsh.u8(), 32, 'reserved1'),
    borsh.publicKey('queuePubkey'),
    borsh.u32('oracleRequestBatchSize'),
    borsh.u32('minOracleResults'),
    borsh.u32('minJobResults'),
    borsh.u32('minUpdateDelaySeconds'),
    borsh.i64('startAfter'),
    types.SwitchboardDecimal.layout('varianceThreshold'),
    borsh.i64('forceReportPeriod'),
    borsh.i64('expiration'),
    borsh.u64('consecutiveFailureCount'),
    borsh.i64('nextAllowedUpdateTime'),
    borsh.bool('isLocked'),
    borsh.publicKey('crankPubkey'),
    types.AggregatorRound.layout('latestConfirmedRound'),
    types.AggregatorRound.layout('currentRound'),
    borsh.array(borsh.publicKey(), 16, 'jobPubkeysData'),
    borsh.array(types.Hash.layout(), 16, 'jobHashes'),
    borsh.u32('jobPubkeysSize'),
    borsh.array(borsh.u8(), 32, 'jobsChecksum'),
    borsh.publicKey('authority'),
    borsh.publicKey('historyBuffer'),
    types.SwitchboardDecimal.layout('previousConfirmedRoundResult'),
    borsh.u64('previousConfirmedRoundSlot'),
    borsh.bool('disableCrank'),
    borsh.array(borsh.u8(), 16, 'jobWeights'),
    borsh.i64('creationTimestamp'),
    types.AggregatorResolutionMode.layout('resolutionMode'),
    borsh.u32('basePriorityFee'),
    borsh.u32('priorityFeeBump'),
    borsh.u32('priorityFeeBumpPeriod'),
    borsh.u32('maxPriorityFeeMultiplier'),
    borsh.array(borsh.u8(), 122, 'ebuf'),
  ]);

  constructor(fields: AggregatorAccountDataFields) {
    this.name = fields.name;
    this.metadata = fields.metadata;
    this.reserved1 = fields.reserved1;
    this.queuePubkey = fields.queuePubkey;
    this.oracleRequestBatchSize = fields.oracleRequestBatchSize;
    this.minOracleResults = fields.minOracleResults;
    this.minJobResults = fields.minJobResults;
    this.minUpdateDelaySeconds = fields.minUpdateDelaySeconds;
    this.startAfter = fields.startAfter;
    this.varianceThreshold = new types.SwitchboardDecimal({
      ...fields.varianceThreshold,
    });
    this.forceReportPeriod = fields.forceReportPeriod;
    this.expiration = fields.expiration;
    this.consecutiveFailureCount = fields.consecutiveFailureCount;
    this.nextAllowedUpdateTime = fields.nextAllowedUpdateTime;
    this.isLocked = fields.isLocked;
    this.crankPubkey = fields.crankPubkey;
    this.latestConfirmedRound = new types.AggregatorRound({
      ...fields.latestConfirmedRound,
    });
    this.currentRound = new types.AggregatorRound({ ...fields.currentRound });
    this.jobPubkeysData = fields.jobPubkeysData;
    this.jobHashes = fields.jobHashes.map(item => new types.Hash({ ...item }));
    this.jobPubkeysSize = fields.jobPubkeysSize;
    this.jobsChecksum = fields.jobsChecksum;
    this.authority = fields.authority;
    this.historyBuffer = fields.historyBuffer;
    this.previousConfirmedRoundResult = new types.SwitchboardDecimal({
      ...fields.previousConfirmedRoundResult,
    });
    this.previousConfirmedRoundSlot = fields.previousConfirmedRoundSlot;
    this.disableCrank = fields.disableCrank;
    this.jobWeights = fields.jobWeights;
    this.creationTimestamp = fields.creationTimestamp;
    this.resolutionMode = fields.resolutionMode;
    this.basePriorityFee = fields.basePriorityFee;
    this.priorityFeeBump = fields.priorityFeeBump;
    this.priorityFeeBumpPeriod = fields.priorityFeeBumpPeriod;
    this.maxPriorityFeeMultiplier = fields.maxPriorityFeeMultiplier;
    this.ebuf = fields.ebuf;
  }

  static async fetch(
    program: SwitchboardProgram,
    address: PublicKey
  ): Promise<AggregatorAccountData | null> {
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
  ): Promise<Array<AggregatorAccountData | null>> {
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

  static decode(data: Buffer): AggregatorAccountData {
    if (!data.slice(0, 8).equals(AggregatorAccountData.discriminator)) {
      throw new Error('invalid account discriminator');
    }

    const dec = AggregatorAccountData.layout.decode(data.slice(8));

    return new AggregatorAccountData({
      name: dec.name,
      metadata: dec.metadata,
      reserved1: dec.reserved1,
      queuePubkey: dec.queuePubkey,
      oracleRequestBatchSize: dec.oracleRequestBatchSize,
      minOracleResults: dec.minOracleResults,
      minJobResults: dec.minJobResults,
      minUpdateDelaySeconds: dec.minUpdateDelaySeconds,
      startAfter: dec.startAfter,
      varianceThreshold: types.SwitchboardDecimal.fromDecoded(
        dec.varianceThreshold
      ),
      forceReportPeriod: dec.forceReportPeriod,
      expiration: dec.expiration,
      consecutiveFailureCount: dec.consecutiveFailureCount,
      nextAllowedUpdateTime: dec.nextAllowedUpdateTime,
      isLocked: dec.isLocked,
      crankPubkey: dec.crankPubkey,
      latestConfirmedRound: types.AggregatorRound.fromDecoded(
        dec.latestConfirmedRound
      ),
      currentRound: types.AggregatorRound.fromDecoded(dec.currentRound),
      jobPubkeysData: dec.jobPubkeysData,
      jobHashes: dec.jobHashes.map(
        (
          item: any /* eslint-disable-line @typescript-eslint/no-explicit-any */
        ) => types.Hash.fromDecoded(item)
      ),
      jobPubkeysSize: dec.jobPubkeysSize,
      jobsChecksum: dec.jobsChecksum,
      authority: dec.authority,
      historyBuffer: dec.historyBuffer,
      previousConfirmedRoundResult: types.SwitchboardDecimal.fromDecoded(
        dec.previousConfirmedRoundResult
      ),
      previousConfirmedRoundSlot: dec.previousConfirmedRoundSlot,
      disableCrank: dec.disableCrank,
      jobWeights: dec.jobWeights,
      creationTimestamp: dec.creationTimestamp,
      resolutionMode: types.AggregatorResolutionMode.fromDecoded(
        dec.resolutionMode
      ),
      basePriorityFee: dec.basePriorityFee,
      priorityFeeBump: dec.priorityFeeBump,
      priorityFeeBumpPeriod: dec.priorityFeeBumpPeriod,
      maxPriorityFeeMultiplier: dec.maxPriorityFeeMultiplier,
      ebuf: dec.ebuf,
    });
  }

  toJSON(): AggregatorAccountDataJSON {
    return {
      name: this.name,
      metadata: this.metadata,
      reserved1: this.reserved1,
      queuePubkey: this.queuePubkey.toString(),
      oracleRequestBatchSize: this.oracleRequestBatchSize,
      minOracleResults: this.minOracleResults,
      minJobResults: this.minJobResults,
      minUpdateDelaySeconds: this.minUpdateDelaySeconds,
      startAfter: this.startAfter.toString(),
      varianceThreshold: this.varianceThreshold.toJSON(),
      forceReportPeriod: this.forceReportPeriod.toString(),
      expiration: this.expiration.toString(),
      consecutiveFailureCount: this.consecutiveFailureCount.toString(),
      nextAllowedUpdateTime: this.nextAllowedUpdateTime.toString(),
      isLocked: this.isLocked,
      crankPubkey: this.crankPubkey.toString(),
      latestConfirmedRound: this.latestConfirmedRound.toJSON(),
      currentRound: this.currentRound.toJSON(),
      jobPubkeysData: this.jobPubkeysData.map(item => item.toString()),
      jobHashes: this.jobHashes.map(item => item.toJSON()),
      jobPubkeysSize: this.jobPubkeysSize,
      jobsChecksum: this.jobsChecksum,
      authority: this.authority.toString(),
      historyBuffer: this.historyBuffer.toString(),
      previousConfirmedRoundResult: this.previousConfirmedRoundResult.toJSON(),
      previousConfirmedRoundSlot: this.previousConfirmedRoundSlot.toString(),
      disableCrank: this.disableCrank,
      jobWeights: this.jobWeights,
      creationTimestamp: this.creationTimestamp.toString(),
      resolutionMode: this.resolutionMode.toJSON(),
      basePriorityFee: this.basePriorityFee,
      priorityFeeBump: this.priorityFeeBump,
      priorityFeeBumpPeriod: this.priorityFeeBumpPeriod,
      maxPriorityFeeMultiplier: this.maxPriorityFeeMultiplier,
      ebuf: this.ebuf,
    };
  }

  static fromJSON(obj: AggregatorAccountDataJSON): AggregatorAccountData {
    return new AggregatorAccountData({
      name: obj.name,
      metadata: obj.metadata,
      reserved1: obj.reserved1,
      queuePubkey: new PublicKey(obj.queuePubkey),
      oracleRequestBatchSize: obj.oracleRequestBatchSize,
      minOracleResults: obj.minOracleResults,
      minJobResults: obj.minJobResults,
      minUpdateDelaySeconds: obj.minUpdateDelaySeconds,
      startAfter: new BN(obj.startAfter),
      varianceThreshold: types.SwitchboardDecimal.fromJSON(
        obj.varianceThreshold
      ),
      forceReportPeriod: new BN(obj.forceReportPeriod),
      expiration: new BN(obj.expiration),
      consecutiveFailureCount: new BN(obj.consecutiveFailureCount),
      nextAllowedUpdateTime: new BN(obj.nextAllowedUpdateTime),
      isLocked: obj.isLocked,
      crankPubkey: new PublicKey(obj.crankPubkey),
      latestConfirmedRound: types.AggregatorRound.fromJSON(
        obj.latestConfirmedRound
      ),
      currentRound: types.AggregatorRound.fromJSON(obj.currentRound),
      jobPubkeysData: obj.jobPubkeysData.map(item => new PublicKey(item)),
      jobHashes: obj.jobHashes.map(item => types.Hash.fromJSON(item)),
      jobPubkeysSize: obj.jobPubkeysSize,
      jobsChecksum: obj.jobsChecksum,
      authority: new PublicKey(obj.authority),
      historyBuffer: new PublicKey(obj.historyBuffer),
      previousConfirmedRoundResult: types.SwitchboardDecimal.fromJSON(
        obj.previousConfirmedRoundResult
      ),
      previousConfirmedRoundSlot: new BN(obj.previousConfirmedRoundSlot),
      disableCrank: obj.disableCrank,
      jobWeights: obj.jobWeights,
      creationTimestamp: new BN(obj.creationTimestamp),
      resolutionMode: types.AggregatorResolutionMode.fromJSON(
        obj.resolutionMode
      ),
      basePriorityFee: obj.basePriorityFee,
      priorityFeeBump: obj.priorityFeeBump,
      priorityFeeBumpPeriod: obj.priorityFeeBumpPeriod,
      maxPriorityFeeMultiplier: obj.maxPriorityFeeMultiplier,
      ebuf: obj.ebuf,
    });
  }
}
