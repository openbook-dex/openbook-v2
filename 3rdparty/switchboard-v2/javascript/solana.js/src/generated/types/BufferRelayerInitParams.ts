import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface BufferRelayerInitParamsFields {
  name: Array<number>;
  minUpdateDelaySeconds: number;
  stateBump: number;
}

export interface BufferRelayerInitParamsJSON {
  name: Array<number>;
  minUpdateDelaySeconds: number;
  stateBump: number;
}

export class BufferRelayerInitParams {
  readonly name: Array<number>;
  readonly minUpdateDelaySeconds: number;
  readonly stateBump: number;

  constructor(fields: BufferRelayerInitParamsFields) {
    this.name = fields.name;
    this.minUpdateDelaySeconds = fields.minUpdateDelaySeconds;
    this.stateBump = fields.stateBump;
  }

  static layout(property?: string) {
    return borsh.struct(
      [
        borsh.array(borsh.u8(), 32, 'name'),
        borsh.u32('minUpdateDelaySeconds'),
        borsh.u8('stateBump'),
      ],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new BufferRelayerInitParams({
      name: obj.name,
      minUpdateDelaySeconds: obj.minUpdateDelaySeconds,
      stateBump: obj.stateBump,
    });
  }

  static toEncodable(fields: BufferRelayerInitParamsFields) {
    return {
      name: fields.name,
      minUpdateDelaySeconds: fields.minUpdateDelaySeconds,
      stateBump: fields.stateBump,
    };
  }

  toJSON(): BufferRelayerInitParamsJSON {
    return {
      name: this.name,
      minUpdateDelaySeconds: this.minUpdateDelaySeconds,
      stateBump: this.stateBump,
    };
  }

  static fromJSON(obj: BufferRelayerInitParamsJSON): BufferRelayerInitParams {
    return new BufferRelayerInitParams({
      name: obj.name,
      minUpdateDelaySeconds: obj.minUpdateDelaySeconds,
      stateBump: obj.stateBump,
    });
  }

  toEncodable() {
    return BufferRelayerInitParams.toEncodable(this);
  }
}
