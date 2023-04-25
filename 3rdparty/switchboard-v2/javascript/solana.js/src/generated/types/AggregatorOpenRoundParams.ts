import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface AggregatorOpenRoundParamsFields {
  stateBump: number;
  leaseBump: number;
  permissionBump: number;
  jitter: number;
}

export interface AggregatorOpenRoundParamsJSON {
  stateBump: number;
  leaseBump: number;
  permissionBump: number;
  jitter: number;
}

export class AggregatorOpenRoundParams {
  readonly stateBump: number;
  readonly leaseBump: number;
  readonly permissionBump: number;
  readonly jitter: number;

  constructor(fields: AggregatorOpenRoundParamsFields) {
    this.stateBump = fields.stateBump;
    this.leaseBump = fields.leaseBump;
    this.permissionBump = fields.permissionBump;
    this.jitter = fields.jitter;
  }

  static layout(property?: string) {
    return borsh.struct(
      [
        borsh.u8('stateBump'),
        borsh.u8('leaseBump'),
        borsh.u8('permissionBump'),
        borsh.u8('jitter'),
      ],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new AggregatorOpenRoundParams({
      stateBump: obj.stateBump,
      leaseBump: obj.leaseBump,
      permissionBump: obj.permissionBump,
      jitter: obj.jitter,
    });
  }

  static toEncodable(fields: AggregatorOpenRoundParamsFields) {
    return {
      stateBump: fields.stateBump,
      leaseBump: fields.leaseBump,
      permissionBump: fields.permissionBump,
      jitter: fields.jitter,
    };
  }

  toJSON(): AggregatorOpenRoundParamsJSON {
    return {
      stateBump: this.stateBump,
      leaseBump: this.leaseBump,
      permissionBump: this.permissionBump,
      jitter: this.jitter,
    };
  }

  static fromJSON(
    obj: AggregatorOpenRoundParamsJSON
  ): AggregatorOpenRoundParams {
    return new AggregatorOpenRoundParams({
      stateBump: obj.stateBump,
      leaseBump: obj.leaseBump,
      permissionBump: obj.permissionBump,
      jitter: obj.jitter,
    });
  }

  toEncodable() {
    return AggregatorOpenRoundParams.toEncodable(this);
  }
}
