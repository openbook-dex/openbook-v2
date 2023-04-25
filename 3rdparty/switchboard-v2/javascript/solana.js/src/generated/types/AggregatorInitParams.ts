import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface AggregatorInitParamsFields {
  name: Array<number>;
  metadata: Array<number>;
  batchSize: number;
  minOracleResults: number;
  minJobResults: number;
  minUpdateDelaySeconds: number;
  startAfter: BN;
  varianceThreshold: types.BorshDecimalFields;
  forceReportPeriod: BN;
  expiration: BN;
  stateBump: number;
  disableCrank: boolean;
}

export interface AggregatorInitParamsJSON {
  name: Array<number>;
  metadata: Array<number>;
  batchSize: number;
  minOracleResults: number;
  minJobResults: number;
  minUpdateDelaySeconds: number;
  startAfter: string;
  varianceThreshold: types.BorshDecimalJSON;
  forceReportPeriod: string;
  expiration: string;
  stateBump: number;
  disableCrank: boolean;
}

export class AggregatorInitParams {
  readonly name: Array<number>;
  readonly metadata: Array<number>;
  readonly batchSize: number;
  readonly minOracleResults: number;
  readonly minJobResults: number;
  readonly minUpdateDelaySeconds: number;
  readonly startAfter: BN;
  readonly varianceThreshold: types.BorshDecimal;
  readonly forceReportPeriod: BN;
  readonly expiration: BN;
  readonly stateBump: number;
  readonly disableCrank: boolean;

  constructor(fields: AggregatorInitParamsFields) {
    this.name = fields.name;
    this.metadata = fields.metadata;
    this.batchSize = fields.batchSize;
    this.minOracleResults = fields.minOracleResults;
    this.minJobResults = fields.minJobResults;
    this.minUpdateDelaySeconds = fields.minUpdateDelaySeconds;
    this.startAfter = fields.startAfter;
    this.varianceThreshold = new types.BorshDecimal({
      ...fields.varianceThreshold,
    });
    this.forceReportPeriod = fields.forceReportPeriod;
    this.expiration = fields.expiration;
    this.stateBump = fields.stateBump;
    this.disableCrank = fields.disableCrank;
  }

  static layout(property?: string) {
    return borsh.struct(
      [
        borsh.array(borsh.u8(), 32, 'name'),
        borsh.array(borsh.u8(), 128, 'metadata'),
        borsh.u32('batchSize'),
        borsh.u32('minOracleResults'),
        borsh.u32('minJobResults'),
        borsh.u32('minUpdateDelaySeconds'),
        borsh.i64('startAfter'),
        types.BorshDecimal.layout('varianceThreshold'),
        borsh.i64('forceReportPeriod'),
        borsh.i64('expiration'),
        borsh.u8('stateBump'),
        borsh.bool('disableCrank'),
      ],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new AggregatorInitParams({
      name: obj.name,
      metadata: obj.metadata,
      batchSize: obj.batchSize,
      minOracleResults: obj.minOracleResults,
      minJobResults: obj.minJobResults,
      minUpdateDelaySeconds: obj.minUpdateDelaySeconds,
      startAfter: obj.startAfter,
      varianceThreshold: types.BorshDecimal.fromDecoded(obj.varianceThreshold),
      forceReportPeriod: obj.forceReportPeriod,
      expiration: obj.expiration,
      stateBump: obj.stateBump,
      disableCrank: obj.disableCrank,
    });
  }

  static toEncodable(fields: AggregatorInitParamsFields) {
    return {
      name: fields.name,
      metadata: fields.metadata,
      batchSize: fields.batchSize,
      minOracleResults: fields.minOracleResults,
      minJobResults: fields.minJobResults,
      minUpdateDelaySeconds: fields.minUpdateDelaySeconds,
      startAfter: fields.startAfter,
      varianceThreshold: types.BorshDecimal.toEncodable(
        fields.varianceThreshold
      ),
      forceReportPeriod: fields.forceReportPeriod,
      expiration: fields.expiration,
      stateBump: fields.stateBump,
      disableCrank: fields.disableCrank,
    };
  }

  toJSON(): AggregatorInitParamsJSON {
    return {
      name: this.name,
      metadata: this.metadata,
      batchSize: this.batchSize,
      minOracleResults: this.minOracleResults,
      minJobResults: this.minJobResults,
      minUpdateDelaySeconds: this.minUpdateDelaySeconds,
      startAfter: this.startAfter.toString(),
      varianceThreshold: this.varianceThreshold.toJSON(),
      forceReportPeriod: this.forceReportPeriod.toString(),
      expiration: this.expiration.toString(),
      stateBump: this.stateBump,
      disableCrank: this.disableCrank,
    };
  }

  static fromJSON(obj: AggregatorInitParamsJSON): AggregatorInitParams {
    return new AggregatorInitParams({
      name: obj.name,
      metadata: obj.metadata,
      batchSize: obj.batchSize,
      minOracleResults: obj.minOracleResults,
      minJobResults: obj.minJobResults,
      minUpdateDelaySeconds: obj.minUpdateDelaySeconds,
      startAfter: new BN(obj.startAfter),
      varianceThreshold: types.BorshDecimal.fromJSON(obj.varianceThreshold),
      forceReportPeriod: new BN(obj.forceReportPeriod),
      expiration: new BN(obj.expiration),
      stateBump: obj.stateBump,
      disableCrank: obj.disableCrank,
    });
  }

  toEncodable() {
    return AggregatorInitParams.toEncodable(this);
  }
}
