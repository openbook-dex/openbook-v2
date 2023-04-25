import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface AggregatorSetForceReportPeriodParamsFields {
  forceReportPeriod: number;
}

export interface AggregatorSetForceReportPeriodParamsJSON {
  forceReportPeriod: number;
}

export class AggregatorSetForceReportPeriodParams {
  readonly forceReportPeriod: number;

  constructor(fields: AggregatorSetForceReportPeriodParamsFields) {
    this.forceReportPeriod = fields.forceReportPeriod;
  }

  static layout(property?: string) {
    return borsh.struct([borsh.u32('forceReportPeriod')], property);
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new AggregatorSetForceReportPeriodParams({
      forceReportPeriod: obj.forceReportPeriod,
    });
  }

  static toEncodable(fields: AggregatorSetForceReportPeriodParamsFields) {
    return {
      forceReportPeriod: fields.forceReportPeriod,
    };
  }

  toJSON(): AggregatorSetForceReportPeriodParamsJSON {
    return {
      forceReportPeriod: this.forceReportPeriod,
    };
  }

  static fromJSON(
    obj: AggregatorSetForceReportPeriodParamsJSON
  ): AggregatorSetForceReportPeriodParams {
    return new AggregatorSetForceReportPeriodParams({
      forceReportPeriod: obj.forceReportPeriod,
    });
  }

  toEncodable() {
    return AggregatorSetForceReportPeriodParams.toEncodable(this);
  }
}
