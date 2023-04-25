import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface JobInitParamsFields {
  name: Array<number>;
  expiration: BN;
  stateBump: number;
  data: Uint8Array;
  size: number | null;
}

export interface JobInitParamsJSON {
  name: Array<number>;
  expiration: string;
  stateBump: number;
  data: Array<number>;
  size: number | null;
}

export class JobInitParams {
  readonly name: Array<number>;
  readonly expiration: BN;
  readonly stateBump: number;
  readonly data: Uint8Array;
  readonly size: number | null;

  constructor(fields: JobInitParamsFields) {
    this.name = fields.name;
    this.expiration = fields.expiration;
    this.stateBump = fields.stateBump;
    this.data = fields.data;
    this.size = fields.size;
  }

  static layout(property?: string) {
    return borsh.struct(
      [
        borsh.array(borsh.u8(), 32, 'name'),
        borsh.i64('expiration'),
        borsh.u8('stateBump'),
        borsh.vecU8('data'),
        borsh.option(borsh.u32(), 'size'),
      ],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new JobInitParams({
      name: obj.name,
      expiration: obj.expiration,
      stateBump: obj.stateBump,
      data: new Uint8Array(
        obj.data.buffer,
        obj.data.byteOffset,
        obj.data.length
      ),
      size: obj.size,
    });
  }

  static toEncodable(fields: JobInitParamsFields) {
    return {
      name: fields.name,
      expiration: fields.expiration,
      stateBump: fields.stateBump,
      data: Buffer.from(
        fields.data.buffer,
        fields.data.byteOffset,
        fields.data.length
      ),
      size: fields.size,
    };
  }

  toJSON(): JobInitParamsJSON {
    return {
      name: this.name,
      expiration: this.expiration.toString(),
      stateBump: this.stateBump,
      data: Array.from(this.data.values()),
      size: this.size,
    };
  }

  static fromJSON(obj: JobInitParamsJSON): JobInitParams {
    return new JobInitParams({
      name: obj.name,
      expiration: new BN(obj.expiration),
      stateBump: obj.stateBump,
      data: Uint8Array.from(obj.data),
      size: obj.size,
    });
  }

  toEncodable() {
    return JobInitParams.toEncodable(this);
  }
}
