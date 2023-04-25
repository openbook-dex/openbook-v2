import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface AggregatorSetConfigParamsFields {
  name: Array<number> | null;
  metadata: Array<number> | null;
  minUpdateDelaySeconds: number | null;
  minJobResults: number | null;
  batchSize: number | null;
  minOracleResults: number | null;
  forceReportPeriod: number | null;
  varianceThreshold: types.BorshDecimalFields | null;
  basePriorityFee: number | null;
  priorityFeeBump: number | null;
  priorityFeeBumpPeriod: number | null;
  maxPriorityFeeMultiplier: number | null;
}

export interface AggregatorSetConfigParamsJSON {
  name: Array<number> | null;
  metadata: Array<number> | null;
  minUpdateDelaySeconds: number | null;
  minJobResults: number | null;
  batchSize: number | null;
  minOracleResults: number | null;
  forceReportPeriod: number | null;
  varianceThreshold: types.BorshDecimalJSON | null;
  basePriorityFee: number | null;
  priorityFeeBump: number | null;
  priorityFeeBumpPeriod: number | null;
  maxPriorityFeeMultiplier: number | null;
}

export class AggregatorSetConfigParams {
  readonly name: Array<number> | null;
  readonly metadata: Array<number> | null;
  readonly minUpdateDelaySeconds: number | null;
  readonly minJobResults: number | null;
  readonly batchSize: number | null;
  readonly minOracleResults: number | null;
  readonly forceReportPeriod: number | null;
  readonly varianceThreshold: types.BorshDecimal | null;
  readonly basePriorityFee: number | null;
  readonly priorityFeeBump: number | null;
  readonly priorityFeeBumpPeriod: number | null;
  readonly maxPriorityFeeMultiplier: number | null;

  constructor(fields: AggregatorSetConfigParamsFields) {
    this.name = fields.name;
    this.metadata = fields.metadata;
    this.minUpdateDelaySeconds = fields.minUpdateDelaySeconds;
    this.minJobResults = fields.minJobResults;
    this.batchSize = fields.batchSize;
    this.minOracleResults = fields.minOracleResults;
    this.forceReportPeriod = fields.forceReportPeriod;
    this.varianceThreshold =
      (fields.varianceThreshold &&
        new types.BorshDecimal({ ...fields.varianceThreshold })) ||
      null;
    this.basePriorityFee = fields.basePriorityFee;
    this.priorityFeeBump = fields.priorityFeeBump;
    this.priorityFeeBumpPeriod = fields.priorityFeeBumpPeriod;
    this.maxPriorityFeeMultiplier = fields.maxPriorityFeeMultiplier;
  }

  static layout(property?: string) {
    return borsh.struct(
      [
        borsh.option(borsh.array(borsh.u8(), 32), 'name'),
        borsh.option(borsh.array(borsh.u8(), 128), 'metadata'),
        borsh.option(borsh.u32(), 'minUpdateDelaySeconds'),
        borsh.option(borsh.u32(), 'minJobResults'),
        borsh.option(borsh.u32(), 'batchSize'),
        borsh.option(borsh.u32(), 'minOracleResults'),
        borsh.option(borsh.u32(), 'forceReportPeriod'),
        borsh.option(types.BorshDecimal.layout(), 'varianceThreshold'),
        borsh.option(borsh.u32(), 'basePriorityFee'),
        borsh.option(borsh.u32(), 'priorityFeeBump'),
        borsh.option(borsh.u32(), 'priorityFeeBumpPeriod'),
        borsh.option(borsh.u32(), 'maxPriorityFeeMultiplier'),
      ],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new AggregatorSetConfigParams({
      name: obj.name,
      metadata: obj.metadata,
      minUpdateDelaySeconds: obj.minUpdateDelaySeconds,
      minJobResults: obj.minJobResults,
      batchSize: obj.batchSize,
      minOracleResults: obj.minOracleResults,
      forceReportPeriod: obj.forceReportPeriod,
      varianceThreshold:
        (obj.varianceThreshold &&
          types.BorshDecimal.fromDecoded(obj.varianceThreshold)) ||
        null,
      basePriorityFee: obj.basePriorityFee,
      priorityFeeBump: obj.priorityFeeBump,
      priorityFeeBumpPeriod: obj.priorityFeeBumpPeriod,
      maxPriorityFeeMultiplier: obj.maxPriorityFeeMultiplier,
    });
  }

  static toEncodable(fields: AggregatorSetConfigParamsFields) {
    return {
      name: fields.name,
      metadata: fields.metadata,
      minUpdateDelaySeconds: fields.minUpdateDelaySeconds,
      minJobResults: fields.minJobResults,
      batchSize: fields.batchSize,
      minOracleResults: fields.minOracleResults,
      forceReportPeriod: fields.forceReportPeriod,
      varianceThreshold:
        (fields.varianceThreshold &&
          types.BorshDecimal.toEncodable(fields.varianceThreshold)) ||
        null,
      basePriorityFee: fields.basePriorityFee,
      priorityFeeBump: fields.priorityFeeBump,
      priorityFeeBumpPeriod: fields.priorityFeeBumpPeriod,
      maxPriorityFeeMultiplier: fields.maxPriorityFeeMultiplier,
    };
  }

  toJSON(): AggregatorSetConfigParamsJSON {
    return {
      name: this.name,
      metadata: this.metadata,
      minUpdateDelaySeconds: this.minUpdateDelaySeconds,
      minJobResults: this.minJobResults,
      batchSize: this.batchSize,
      minOracleResults: this.minOracleResults,
      forceReportPeriod: this.forceReportPeriod,
      varianceThreshold:
        (this.varianceThreshold && this.varianceThreshold.toJSON()) || null,
      basePriorityFee: this.basePriorityFee,
      priorityFeeBump: this.priorityFeeBump,
      priorityFeeBumpPeriod: this.priorityFeeBumpPeriod,
      maxPriorityFeeMultiplier: this.maxPriorityFeeMultiplier,
    };
  }

  static fromJSON(
    obj: AggregatorSetConfigParamsJSON
  ): AggregatorSetConfigParams {
    return new AggregatorSetConfigParams({
      name: obj.name,
      metadata: obj.metadata,
      minUpdateDelaySeconds: obj.minUpdateDelaySeconds,
      minJobResults: obj.minJobResults,
      batchSize: obj.batchSize,
      minOracleResults: obj.minOracleResults,
      forceReportPeriod: obj.forceReportPeriod,
      varianceThreshold:
        (obj.varianceThreshold &&
          types.BorshDecimal.fromJSON(obj.varianceThreshold)) ||
        null,
      basePriorityFee: obj.basePriorityFee,
      priorityFeeBump: obj.priorityFeeBump,
      priorityFeeBumpPeriod: obj.priorityFeeBumpPeriod,
      maxPriorityFeeMultiplier: obj.maxPriorityFeeMultiplier,
    });
  }

  toEncodable() {
    return AggregatorSetConfigParams.toEncodable(this);
  }
}
