import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface OracleQueueInitParamsFields {
  name: Array<number>;
  metadata: Array<number>;
  reward: BN;
  minStake: BN;
  feedProbationPeriod: number;
  oracleTimeout: number;
  slashingEnabled: boolean;
  varianceToleranceMultiplier: types.BorshDecimalFields;
  consecutiveFeedFailureLimit: BN;
  consecutiveOracleFailureLimit: BN;
  queueSize: number;
  unpermissionedFeeds: boolean;
  unpermissionedVrf: boolean;
  enableBufferRelayers: boolean;
}

export interface OracleQueueInitParamsJSON {
  name: Array<number>;
  metadata: Array<number>;
  reward: string;
  minStake: string;
  feedProbationPeriod: number;
  oracleTimeout: number;
  slashingEnabled: boolean;
  varianceToleranceMultiplier: types.BorshDecimalJSON;
  consecutiveFeedFailureLimit: string;
  consecutiveOracleFailureLimit: string;
  queueSize: number;
  unpermissionedFeeds: boolean;
  unpermissionedVrf: boolean;
  enableBufferRelayers: boolean;
}

export class OracleQueueInitParams {
  readonly name: Array<number>;
  readonly metadata: Array<number>;
  readonly reward: BN;
  readonly minStake: BN;
  readonly feedProbationPeriod: number;
  readonly oracleTimeout: number;
  readonly slashingEnabled: boolean;
  readonly varianceToleranceMultiplier: types.BorshDecimal;
  readonly consecutiveFeedFailureLimit: BN;
  readonly consecutiveOracleFailureLimit: BN;
  readonly queueSize: number;
  readonly unpermissionedFeeds: boolean;
  readonly unpermissionedVrf: boolean;
  readonly enableBufferRelayers: boolean;

  constructor(fields: OracleQueueInitParamsFields) {
    this.name = fields.name;
    this.metadata = fields.metadata;
    this.reward = fields.reward;
    this.minStake = fields.minStake;
    this.feedProbationPeriod = fields.feedProbationPeriod;
    this.oracleTimeout = fields.oracleTimeout;
    this.slashingEnabled = fields.slashingEnabled;
    this.varianceToleranceMultiplier = new types.BorshDecimal({
      ...fields.varianceToleranceMultiplier,
    });
    this.consecutiveFeedFailureLimit = fields.consecutiveFeedFailureLimit;
    this.consecutiveOracleFailureLimit = fields.consecutiveOracleFailureLimit;
    this.queueSize = fields.queueSize;
    this.unpermissionedFeeds = fields.unpermissionedFeeds;
    this.unpermissionedVrf = fields.unpermissionedVrf;
    this.enableBufferRelayers = fields.enableBufferRelayers;
  }

  static layout(property?: string) {
    return borsh.struct(
      [
        borsh.array(borsh.u8(), 32, 'name'),
        borsh.array(borsh.u8(), 64, 'metadata'),
        borsh.u64('reward'),
        borsh.u64('minStake'),
        borsh.u32('feedProbationPeriod'),
        borsh.u32('oracleTimeout'),
        borsh.bool('slashingEnabled'),
        types.BorshDecimal.layout('varianceToleranceMultiplier'),
        borsh.u64('consecutiveFeedFailureLimit'),
        borsh.u64('consecutiveOracleFailureLimit'),
        borsh.u32('queueSize'),
        borsh.bool('unpermissionedFeeds'),
        borsh.bool('unpermissionedVrf'),
        borsh.bool('enableBufferRelayers'),
      ],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new OracleQueueInitParams({
      name: obj.name,
      metadata: obj.metadata,
      reward: obj.reward,
      minStake: obj.minStake,
      feedProbationPeriod: obj.feedProbationPeriod,
      oracleTimeout: obj.oracleTimeout,
      slashingEnabled: obj.slashingEnabled,
      varianceToleranceMultiplier: types.BorshDecimal.fromDecoded(
        obj.varianceToleranceMultiplier
      ),
      consecutiveFeedFailureLimit: obj.consecutiveFeedFailureLimit,
      consecutiveOracleFailureLimit: obj.consecutiveOracleFailureLimit,
      queueSize: obj.queueSize,
      unpermissionedFeeds: obj.unpermissionedFeeds,
      unpermissionedVrf: obj.unpermissionedVrf,
      enableBufferRelayers: obj.enableBufferRelayers,
    });
  }

  static toEncodable(fields: OracleQueueInitParamsFields) {
    return {
      name: fields.name,
      metadata: fields.metadata,
      reward: fields.reward,
      minStake: fields.minStake,
      feedProbationPeriod: fields.feedProbationPeriod,
      oracleTimeout: fields.oracleTimeout,
      slashingEnabled: fields.slashingEnabled,
      varianceToleranceMultiplier: types.BorshDecimal.toEncodable(
        fields.varianceToleranceMultiplier
      ),
      consecutiveFeedFailureLimit: fields.consecutiveFeedFailureLimit,
      consecutiveOracleFailureLimit: fields.consecutiveOracleFailureLimit,
      queueSize: fields.queueSize,
      unpermissionedFeeds: fields.unpermissionedFeeds,
      unpermissionedVrf: fields.unpermissionedVrf,
      enableBufferRelayers: fields.enableBufferRelayers,
    };
  }

  toJSON(): OracleQueueInitParamsJSON {
    return {
      name: this.name,
      metadata: this.metadata,
      reward: this.reward.toString(),
      minStake: this.minStake.toString(),
      feedProbationPeriod: this.feedProbationPeriod,
      oracleTimeout: this.oracleTimeout,
      slashingEnabled: this.slashingEnabled,
      varianceToleranceMultiplier: this.varianceToleranceMultiplier.toJSON(),
      consecutiveFeedFailureLimit: this.consecutiveFeedFailureLimit.toString(),
      consecutiveOracleFailureLimit:
        this.consecutiveOracleFailureLimit.toString(),
      queueSize: this.queueSize,
      unpermissionedFeeds: this.unpermissionedFeeds,
      unpermissionedVrf: this.unpermissionedVrf,
      enableBufferRelayers: this.enableBufferRelayers,
    };
  }

  static fromJSON(obj: OracleQueueInitParamsJSON): OracleQueueInitParams {
    return new OracleQueueInitParams({
      name: obj.name,
      metadata: obj.metadata,
      reward: new BN(obj.reward),
      minStake: new BN(obj.minStake),
      feedProbationPeriod: obj.feedProbationPeriod,
      oracleTimeout: obj.oracleTimeout,
      slashingEnabled: obj.slashingEnabled,
      varianceToleranceMultiplier: types.BorshDecimal.fromJSON(
        obj.varianceToleranceMultiplier
      ),
      consecutiveFeedFailureLimit: new BN(obj.consecutiveFeedFailureLimit),
      consecutiveOracleFailureLimit: new BN(obj.consecutiveOracleFailureLimit),
      queueSize: obj.queueSize,
      unpermissionedFeeds: obj.unpermissionedFeeds,
      unpermissionedVrf: obj.unpermissionedVrf,
      enableBufferRelayers: obj.enableBufferRelayers,
    });
  }

  toEncodable() {
    return OracleQueueInitParams.toEncodable(this);
  }
}
