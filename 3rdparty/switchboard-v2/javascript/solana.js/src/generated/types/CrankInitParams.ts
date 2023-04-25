import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface CrankInitParamsFields {
  name: Uint8Array;
  metadata: Uint8Array;
  crankSize: number;
}

export interface CrankInitParamsJSON {
  name: Array<number>;
  metadata: Array<number>;
  crankSize: number;
}

export class CrankInitParams {
  readonly name: Uint8Array;
  readonly metadata: Uint8Array;
  readonly crankSize: number;

  constructor(fields: CrankInitParamsFields) {
    this.name = fields.name;
    this.metadata = fields.metadata;
    this.crankSize = fields.crankSize;
  }

  static layout(property?: string) {
    return borsh.struct(
      [borsh.vecU8('name'), borsh.vecU8('metadata'), borsh.u32('crankSize')],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new CrankInitParams({
      name: new Uint8Array(
        obj.name.buffer,
        obj.name.byteOffset,
        obj.name.length
      ),
      metadata: new Uint8Array(
        obj.metadata.buffer,
        obj.metadata.byteOffset,
        obj.metadata.length
      ),
      crankSize: obj.crankSize,
    });
  }

  static toEncodable(fields: CrankInitParamsFields) {
    return {
      name: Buffer.from(
        fields.name.buffer,
        fields.name.byteOffset,
        fields.name.length
      ),
      metadata: Buffer.from(
        fields.metadata.buffer,
        fields.metadata.byteOffset,
        fields.metadata.length
      ),
      crankSize: fields.crankSize,
    };
  }

  toJSON(): CrankInitParamsJSON {
    return {
      name: Array.from(this.name.values()),
      metadata: Array.from(this.metadata.values()),
      crankSize: this.crankSize,
    };
  }

  static fromJSON(obj: CrankInitParamsJSON): CrankInitParams {
    return new CrankInitParams({
      name: Uint8Array.from(obj.name),
      metadata: Uint8Array.from(obj.metadata),
      crankSize: obj.crankSize,
    });
  }

  toEncodable() {
    return CrankInitParams.toEncodable(this);
  }
}
