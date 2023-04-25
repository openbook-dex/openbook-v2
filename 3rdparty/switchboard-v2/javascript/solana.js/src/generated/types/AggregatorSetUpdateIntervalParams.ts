import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface AggregatorSetUpdateIntervalParamsFields {
  newInterval: number;
}

export interface AggregatorSetUpdateIntervalParamsJSON {
  newInterval: number;
}

export class AggregatorSetUpdateIntervalParams {
  readonly newInterval: number;

  constructor(fields: AggregatorSetUpdateIntervalParamsFields) {
    this.newInterval = fields.newInterval;
  }

  static layout(property?: string) {
    return borsh.struct([borsh.u32('newInterval')], property);
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new AggregatorSetUpdateIntervalParams({
      newInterval: obj.newInterval,
    });
  }

  static toEncodable(fields: AggregatorSetUpdateIntervalParamsFields) {
    return {
      newInterval: fields.newInterval,
    };
  }

  toJSON(): AggregatorSetUpdateIntervalParamsJSON {
    return {
      newInterval: this.newInterval,
    };
  }

  static fromJSON(
    obj: AggregatorSetUpdateIntervalParamsJSON
  ): AggregatorSetUpdateIntervalParams {
    return new AggregatorSetUpdateIntervalParams({
      newInterval: obj.newInterval,
    });
  }

  toEncodable() {
    return AggregatorSetUpdateIntervalParams.toEncodable(this);
  }
}
