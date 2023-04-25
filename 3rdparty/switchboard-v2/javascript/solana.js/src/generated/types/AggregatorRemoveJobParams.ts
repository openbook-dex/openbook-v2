import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface AggregatorRemoveJobParamsFields {
  jobIdx: number;
}

export interface AggregatorRemoveJobParamsJSON {
  jobIdx: number;
}

export class AggregatorRemoveJobParams {
  readonly jobIdx: number;

  constructor(fields: AggregatorRemoveJobParamsFields) {
    this.jobIdx = fields.jobIdx;
  }

  static layout(property?: string) {
    return borsh.struct([borsh.u32('jobIdx')], property);
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new AggregatorRemoveJobParams({
      jobIdx: obj.jobIdx,
    });
  }

  static toEncodable(fields: AggregatorRemoveJobParamsFields) {
    return {
      jobIdx: fields.jobIdx,
    };
  }

  toJSON(): AggregatorRemoveJobParamsJSON {
    return {
      jobIdx: this.jobIdx,
    };
  }

  static fromJSON(
    obj: AggregatorRemoveJobParamsJSON
  ): AggregatorRemoveJobParams {
    return new AggregatorRemoveJobParams({
      jobIdx: obj.jobIdx,
    });
  }

  toEncodable() {
    return AggregatorRemoveJobParams.toEncodable(this);
  }
}
