import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface AggregatorSetResolutionModeParamsFields {
  mode: number;
}

export interface AggregatorSetResolutionModeParamsJSON {
  mode: number;
}

export class AggregatorSetResolutionModeParams {
  readonly mode: number;

  constructor(fields: AggregatorSetResolutionModeParamsFields) {
    this.mode = fields.mode;
  }

  static layout(property?: string) {
    return borsh.struct([borsh.u8('mode')], property);
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new AggregatorSetResolutionModeParams({
      mode: obj.mode,
    });
  }

  static toEncodable(fields: AggregatorSetResolutionModeParamsFields) {
    return {
      mode: fields.mode,
    };
  }

  toJSON(): AggregatorSetResolutionModeParamsJSON {
    return {
      mode: this.mode,
    };
  }

  static fromJSON(
    obj: AggregatorSetResolutionModeParamsJSON
  ): AggregatorSetResolutionModeParams {
    return new AggregatorSetResolutionModeParams({
      mode: obj.mode,
    });
  }

  toEncodable() {
    return AggregatorSetResolutionModeParams.toEncodable(this);
  }
}
