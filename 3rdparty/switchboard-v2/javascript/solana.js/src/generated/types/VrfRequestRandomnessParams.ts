import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface VrfRequestRandomnessParamsFields {
  permissionBump: number;
  stateBump: number;
}

export interface VrfRequestRandomnessParamsJSON {
  permissionBump: number;
  stateBump: number;
}

export class VrfRequestRandomnessParams {
  readonly permissionBump: number;
  readonly stateBump: number;

  constructor(fields: VrfRequestRandomnessParamsFields) {
    this.permissionBump = fields.permissionBump;
    this.stateBump = fields.stateBump;
  }

  static layout(property?: string) {
    return borsh.struct(
      [borsh.u8('permissionBump'), borsh.u8('stateBump')],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new VrfRequestRandomnessParams({
      permissionBump: obj.permissionBump,
      stateBump: obj.stateBump,
    });
  }

  static toEncodable(fields: VrfRequestRandomnessParamsFields) {
    return {
      permissionBump: fields.permissionBump,
      stateBump: fields.stateBump,
    };
  }

  toJSON(): VrfRequestRandomnessParamsJSON {
    return {
      permissionBump: this.permissionBump,
      stateBump: this.stateBump,
    };
  }

  static fromJSON(
    obj: VrfRequestRandomnessParamsJSON
  ): VrfRequestRandomnessParams {
    return new VrfRequestRandomnessParams({
      permissionBump: obj.permissionBump,
      stateBump: obj.stateBump,
    });
  }

  toEncodable() {
    return VrfRequestRandomnessParams.toEncodable(this);
  }
}
