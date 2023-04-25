import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface OracleInitParamsFields {
  name: Uint8Array;
  metadata: Uint8Array;
  stateBump: number;
  oracleBump: number;
}

export interface OracleInitParamsJSON {
  name: Array<number>;
  metadata: Array<number>;
  stateBump: number;
  oracleBump: number;
}

export class OracleInitParams {
  readonly name: Uint8Array;
  readonly metadata: Uint8Array;
  readonly stateBump: number;
  readonly oracleBump: number;

  constructor(fields: OracleInitParamsFields) {
    this.name = fields.name;
    this.metadata = fields.metadata;
    this.stateBump = fields.stateBump;
    this.oracleBump = fields.oracleBump;
  }

  static layout(property?: string) {
    return borsh.struct(
      [
        borsh.vecU8('name'),
        borsh.vecU8('metadata'),
        borsh.u8('stateBump'),
        borsh.u8('oracleBump'),
      ],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new OracleInitParams({
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
      stateBump: obj.stateBump,
      oracleBump: obj.oracleBump,
    });
  }

  static toEncodable(fields: OracleInitParamsFields) {
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
      stateBump: fields.stateBump,
      oracleBump: fields.oracleBump,
    };
  }

  toJSON(): OracleInitParamsJSON {
    return {
      name: Array.from(this.name.values()),
      metadata: Array.from(this.metadata.values()),
      stateBump: this.stateBump,
      oracleBump: this.oracleBump,
    };
  }

  static fromJSON(obj: OracleInitParamsJSON): OracleInitParams {
    return new OracleInitParams({
      name: Uint8Array.from(obj.name),
      metadata: Uint8Array.from(obj.metadata),
      stateBump: obj.stateBump,
      oracleBump: obj.oracleBump,
    });
  }

  toEncodable() {
    return OracleInitParams.toEncodable(this);
  }
}
