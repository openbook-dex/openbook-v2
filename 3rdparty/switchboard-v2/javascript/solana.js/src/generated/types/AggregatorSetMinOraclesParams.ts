import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface AggregatorSetMinOraclesParamsFields {
  minOracleResults: number;
}

export interface AggregatorSetMinOraclesParamsJSON {
  minOracleResults: number;
}

export class AggregatorSetMinOraclesParams {
  readonly minOracleResults: number;

  constructor(fields: AggregatorSetMinOraclesParamsFields) {
    this.minOracleResults = fields.minOracleResults;
  }

  static layout(property?: string) {
    return borsh.struct([borsh.u32('minOracleResults')], property);
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new AggregatorSetMinOraclesParams({
      minOracleResults: obj.minOracleResults,
    });
  }

  static toEncodable(fields: AggregatorSetMinOraclesParamsFields) {
    return {
      minOracleResults: fields.minOracleResults,
    };
  }

  toJSON(): AggregatorSetMinOraclesParamsJSON {
    return {
      minOracleResults: this.minOracleResults,
    };
  }

  static fromJSON(
    obj: AggregatorSetMinOraclesParamsJSON
  ): AggregatorSetMinOraclesParams {
    return new AggregatorSetMinOraclesParams({
      minOracleResults: obj.minOracleResults,
    });
  }

  toEncodable() {
    return AggregatorSetMinOraclesParams.toEncodable(this);
  }
}
