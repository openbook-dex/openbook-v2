import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface BufferRelayerSaveResultParamsFields {
  stateBump: number;
  permissionBump: number;
  result: Uint8Array;
  success: boolean;
}

export interface BufferRelayerSaveResultParamsJSON {
  stateBump: number;
  permissionBump: number;
  result: Array<number>;
  success: boolean;
}

export class BufferRelayerSaveResultParams {
  readonly stateBump: number;
  readonly permissionBump: number;
  readonly result: Uint8Array;
  readonly success: boolean;

  constructor(fields: BufferRelayerSaveResultParamsFields) {
    this.stateBump = fields.stateBump;
    this.permissionBump = fields.permissionBump;
    this.result = fields.result;
    this.success = fields.success;
  }

  static layout(property?: string) {
    return borsh.struct(
      [
        borsh.u8('stateBump'),
        borsh.u8('permissionBump'),
        borsh.vecU8('result'),
        borsh.bool('success'),
      ],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new BufferRelayerSaveResultParams({
      stateBump: obj.stateBump,
      permissionBump: obj.permissionBump,
      result: new Uint8Array(
        obj.result.buffer,
        obj.result.byteOffset,
        obj.result.length
      ),
      success: obj.success,
    });
  }

  static toEncodable(fields: BufferRelayerSaveResultParamsFields) {
    return {
      stateBump: fields.stateBump,
      permissionBump: fields.permissionBump,
      result: Buffer.from(
        fields.result.buffer,
        fields.result.byteOffset,
        fields.result.length
      ),
      success: fields.success,
    };
  }

  toJSON(): BufferRelayerSaveResultParamsJSON {
    return {
      stateBump: this.stateBump,
      permissionBump: this.permissionBump,
      result: Array.from(this.result.values()),
      success: this.success,
    };
  }

  static fromJSON(
    obj: BufferRelayerSaveResultParamsJSON
  ): BufferRelayerSaveResultParams {
    return new BufferRelayerSaveResultParams({
      stateBump: obj.stateBump,
      permissionBump: obj.permissionBump,
      result: Uint8Array.from(obj.result),
      success: obj.success,
    });
  }

  toEncodable() {
    return BufferRelayerSaveResultParams.toEncodable(this);
  }
}
