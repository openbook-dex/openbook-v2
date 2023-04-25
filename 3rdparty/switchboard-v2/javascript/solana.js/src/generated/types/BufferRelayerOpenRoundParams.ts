import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface BufferRelayerOpenRoundParamsFields {
  stateBump: number;
  permissionBump: number;
}

export interface BufferRelayerOpenRoundParamsJSON {
  stateBump: number;
  permissionBump: number;
}

export class BufferRelayerOpenRoundParams {
  readonly stateBump: number;
  readonly permissionBump: number;

  constructor(fields: BufferRelayerOpenRoundParamsFields) {
    this.stateBump = fields.stateBump;
    this.permissionBump = fields.permissionBump;
  }

  static layout(property?: string) {
    return borsh.struct(
      [borsh.u8('stateBump'), borsh.u8('permissionBump')],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new BufferRelayerOpenRoundParams({
      stateBump: obj.stateBump,
      permissionBump: obj.permissionBump,
    });
  }

  static toEncodable(fields: BufferRelayerOpenRoundParamsFields) {
    return {
      stateBump: fields.stateBump,
      permissionBump: fields.permissionBump,
    };
  }

  toJSON(): BufferRelayerOpenRoundParamsJSON {
    return {
      stateBump: this.stateBump,
      permissionBump: this.permissionBump,
    };
  }

  static fromJSON(
    obj: BufferRelayerOpenRoundParamsJSON
  ): BufferRelayerOpenRoundParams {
    return new BufferRelayerOpenRoundParams({
      stateBump: obj.stateBump,
      permissionBump: obj.permissionBump,
    });
  }

  toEncodable() {
    return BufferRelayerOpenRoundParams.toEncodable(this);
  }
}
