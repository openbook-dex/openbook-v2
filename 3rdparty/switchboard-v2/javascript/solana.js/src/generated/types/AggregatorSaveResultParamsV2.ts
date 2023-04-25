import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface AggregatorSaveResultParamsV2Fields {
  oracleIdx: number;
  error: boolean;
  value: types.BorshDecimalFields;
  jobsChecksum: Array<number>;
  minResponse: types.BorshDecimalFields;
  maxResponse: types.BorshDecimalFields;
  feedPermissionBump: number;
  oraclePermissionBump: number;
  leaseBump: number;
  stateBump: number;
  jobValues: Array<types.BorshDecimalFields | null>;
}

export interface AggregatorSaveResultParamsV2JSON {
  oracleIdx: number;
  error: boolean;
  value: types.BorshDecimalJSON;
  jobsChecksum: Array<number>;
  minResponse: types.BorshDecimalJSON;
  maxResponse: types.BorshDecimalJSON;
  feedPermissionBump: number;
  oraclePermissionBump: number;
  leaseBump: number;
  stateBump: number;
  jobValues: Array<types.BorshDecimalJSON | null>;
}

export class AggregatorSaveResultParamsV2 {
  readonly oracleIdx: number;
  readonly error: boolean;
  readonly value: types.BorshDecimal;
  readonly jobsChecksum: Array<number>;
  readonly minResponse: types.BorshDecimal;
  readonly maxResponse: types.BorshDecimal;
  readonly feedPermissionBump: number;
  readonly oraclePermissionBump: number;
  readonly leaseBump: number;
  readonly stateBump: number;
  readonly jobValues: Array<types.BorshDecimal | null>;

  constructor(fields: AggregatorSaveResultParamsV2Fields) {
    this.oracleIdx = fields.oracleIdx;
    this.error = fields.error;
    this.value = new types.BorshDecimal({ ...fields.value });
    this.jobsChecksum = fields.jobsChecksum;
    this.minResponse = new types.BorshDecimal({ ...fields.minResponse });
    this.maxResponse = new types.BorshDecimal({ ...fields.maxResponse });
    this.feedPermissionBump = fields.feedPermissionBump;
    this.oraclePermissionBump = fields.oraclePermissionBump;
    this.leaseBump = fields.leaseBump;
    this.stateBump = fields.stateBump;
    this.jobValues = fields.jobValues.map(
      item => (item && new types.BorshDecimal({ ...item })) || null
    );
  }

  static layout(property?: string) {
    return borsh.struct(
      [
        borsh.u32('oracleIdx'),
        borsh.bool('error'),
        types.BorshDecimal.layout('value'),
        borsh.array(borsh.u8(), 32, 'jobsChecksum'),
        types.BorshDecimal.layout('minResponse'),
        types.BorshDecimal.layout('maxResponse'),
        borsh.u8('feedPermissionBump'),
        borsh.u8('oraclePermissionBump'),
        borsh.u8('leaseBump'),
        borsh.u8('stateBump'),
        borsh.vec(borsh.option(types.BorshDecimal.layout()), 'jobValues'),
      ],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new AggregatorSaveResultParamsV2({
      oracleIdx: obj.oracleIdx,
      error: obj.error,
      value: types.BorshDecimal.fromDecoded(obj.value),
      jobsChecksum: obj.jobsChecksum,
      minResponse: types.BorshDecimal.fromDecoded(obj.minResponse),
      maxResponse: types.BorshDecimal.fromDecoded(obj.maxResponse),
      feedPermissionBump: obj.feedPermissionBump,
      oraclePermissionBump: obj.oraclePermissionBump,
      leaseBump: obj.leaseBump,
      stateBump: obj.stateBump,
      jobValues: obj.jobValues.map(
        (
          item: any /* eslint-disable-line @typescript-eslint/no-explicit-any */
        ) => (item && types.BorshDecimal.fromDecoded(item)) || null
      ),
    });
  }

  static toEncodable(fields: AggregatorSaveResultParamsV2Fields) {
    return {
      oracleIdx: fields.oracleIdx,
      error: fields.error,
      value: types.BorshDecimal.toEncodable(fields.value),
      jobsChecksum: fields.jobsChecksum,
      minResponse: types.BorshDecimal.toEncodable(fields.minResponse),
      maxResponse: types.BorshDecimal.toEncodable(fields.maxResponse),
      feedPermissionBump: fields.feedPermissionBump,
      oraclePermissionBump: fields.oraclePermissionBump,
      leaseBump: fields.leaseBump,
      stateBump: fields.stateBump,
      jobValues: fields.jobValues.map(
        item => (item && types.BorshDecimal.toEncodable(item)) || null
      ),
    };
  }

  toJSON(): AggregatorSaveResultParamsV2JSON {
    return {
      oracleIdx: this.oracleIdx,
      error: this.error,
      value: this.value.toJSON(),
      jobsChecksum: this.jobsChecksum,
      minResponse: this.minResponse.toJSON(),
      maxResponse: this.maxResponse.toJSON(),
      feedPermissionBump: this.feedPermissionBump,
      oraclePermissionBump: this.oraclePermissionBump,
      leaseBump: this.leaseBump,
      stateBump: this.stateBump,
      jobValues: this.jobValues.map(item => (item && item.toJSON()) || null),
    };
  }

  static fromJSON(
    obj: AggregatorSaveResultParamsV2JSON
  ): AggregatorSaveResultParamsV2 {
    return new AggregatorSaveResultParamsV2({
      oracleIdx: obj.oracleIdx,
      error: obj.error,
      value: types.BorshDecimal.fromJSON(obj.value),
      jobsChecksum: obj.jobsChecksum,
      minResponse: types.BorshDecimal.fromJSON(obj.minResponse),
      maxResponse: types.BorshDecimal.fromJSON(obj.maxResponse),
      feedPermissionBump: obj.feedPermissionBump,
      oraclePermissionBump: obj.oraclePermissionBump,
      leaseBump: obj.leaseBump,
      stateBump: obj.stateBump,
      jobValues: obj.jobValues.map(
        item => (item && types.BorshDecimal.fromJSON(item)) || null
      ),
    });
  }

  toEncodable() {
    return AggregatorSaveResultParamsV2.toEncodable(this);
  }
}
