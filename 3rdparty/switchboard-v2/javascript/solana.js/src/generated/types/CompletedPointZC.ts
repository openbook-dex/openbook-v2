import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface CompletedPointZCFields {
  x: types.FieldElementZCFields;
  y: types.FieldElementZCFields;
  z: types.FieldElementZCFields;
  t: types.FieldElementZCFields;
}

export interface CompletedPointZCJSON {
  x: types.FieldElementZCJSON;
  y: types.FieldElementZCJSON;
  z: types.FieldElementZCJSON;
  t: types.FieldElementZCJSON;
}

export class CompletedPointZC {
  readonly x: types.FieldElementZC;
  readonly y: types.FieldElementZC;
  readonly z: types.FieldElementZC;
  readonly t: types.FieldElementZC;

  constructor(fields: CompletedPointZCFields) {
    this.x = new types.FieldElementZC({ ...fields.x });
    this.y = new types.FieldElementZC({ ...fields.y });
    this.z = new types.FieldElementZC({ ...fields.z });
    this.t = new types.FieldElementZC({ ...fields.t });
  }

  static layout(property?: string) {
    return borsh.struct(
      [
        types.FieldElementZC.layout('x'),
        types.FieldElementZC.layout('y'),
        types.FieldElementZC.layout('z'),
        types.FieldElementZC.layout('t'),
      ],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new CompletedPointZC({
      x: types.FieldElementZC.fromDecoded(obj.x),
      y: types.FieldElementZC.fromDecoded(obj.y),
      z: types.FieldElementZC.fromDecoded(obj.z),
      t: types.FieldElementZC.fromDecoded(obj.t),
    });
  }

  static toEncodable(fields: CompletedPointZCFields) {
    return {
      x: types.FieldElementZC.toEncodable(fields.x),
      y: types.FieldElementZC.toEncodable(fields.y),
      z: types.FieldElementZC.toEncodable(fields.z),
      t: types.FieldElementZC.toEncodable(fields.t),
    };
  }

  toJSON(): CompletedPointZCJSON {
    return {
      x: this.x.toJSON(),
      y: this.y.toJSON(),
      z: this.z.toJSON(),
      t: this.t.toJSON(),
    };
  }

  static fromJSON(obj: CompletedPointZCJSON): CompletedPointZC {
    return new CompletedPointZC({
      x: types.FieldElementZC.fromJSON(obj.x),
      y: types.FieldElementZC.fromJSON(obj.y),
      z: types.FieldElementZC.fromJSON(obj.z),
      t: types.FieldElementZC.fromJSON(obj.t),
    });
  }

  toEncodable() {
    return CompletedPointZC.toEncodable(this);
  }
}
