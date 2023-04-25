import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface VrfInitParamsFields {
  callback: types.CallbackFields;
  stateBump: number;
}

export interface VrfInitParamsJSON {
  callback: types.CallbackJSON;
  stateBump: number;
}

export class VrfInitParams {
  readonly callback: types.Callback;
  readonly stateBump: number;

  constructor(fields: VrfInitParamsFields) {
    this.callback = new types.Callback({ ...fields.callback });
    this.stateBump = fields.stateBump;
  }

  static layout(property?: string) {
    return borsh.struct(
      [types.Callback.layout('callback'), borsh.u8('stateBump')],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new VrfInitParams({
      callback: types.Callback.fromDecoded(obj.callback),
      stateBump: obj.stateBump,
    });
  }

  static toEncodable(fields: VrfInitParamsFields) {
    return {
      callback: types.Callback.toEncodable(fields.callback),
      stateBump: fields.stateBump,
    };
  }

  toJSON(): VrfInitParamsJSON {
    return {
      callback: this.callback.toJSON(),
      stateBump: this.stateBump,
    };
  }

  static fromJSON(obj: VrfInitParamsJSON): VrfInitParams {
    return new VrfInitParams({
      callback: types.Callback.fromJSON(obj.callback),
      stateBump: obj.stateBump,
    });
  }

  toEncodable() {
    return VrfInitParams.toEncodable(this);
  }
}
