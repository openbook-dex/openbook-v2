import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface AggregatorSetVarianceThresholdParamsFields {
  varianceThreshold: types.BorshDecimalFields;
}

export interface AggregatorSetVarianceThresholdParamsJSON {
  varianceThreshold: types.BorshDecimalJSON;
}

export class AggregatorSetVarianceThresholdParams {
  readonly varianceThreshold: types.BorshDecimal;

  constructor(fields: AggregatorSetVarianceThresholdParamsFields) {
    this.varianceThreshold = new types.BorshDecimal({
      ...fields.varianceThreshold,
    });
  }

  static layout(property?: string) {
    return borsh.struct(
      [types.BorshDecimal.layout('varianceThreshold')],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new AggregatorSetVarianceThresholdParams({
      varianceThreshold: types.BorshDecimal.fromDecoded(obj.varianceThreshold),
    });
  }

  static toEncodable(fields: AggregatorSetVarianceThresholdParamsFields) {
    return {
      varianceThreshold: types.BorshDecimal.toEncodable(
        fields.varianceThreshold
      ),
    };
  }

  toJSON(): AggregatorSetVarianceThresholdParamsJSON {
    return {
      varianceThreshold: this.varianceThreshold.toJSON(),
    };
  }

  static fromJSON(
    obj: AggregatorSetVarianceThresholdParamsJSON
  ): AggregatorSetVarianceThresholdParams {
    return new AggregatorSetVarianceThresholdParams({
      varianceThreshold: types.BorshDecimal.fromJSON(obj.varianceThreshold),
    });
  }

  toEncodable() {
    return AggregatorSetVarianceThresholdParams.toEncodable(this);
  }
}
