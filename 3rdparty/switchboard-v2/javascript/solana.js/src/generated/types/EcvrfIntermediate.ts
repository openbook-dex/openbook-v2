import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface EcvrfIntermediateFields {
  r: types.FieldElementZCFields;
  nS: types.FieldElementZCFields;
  d: types.FieldElementZCFields;
  t13: types.FieldElementZCFields;
  t15: types.FieldElementZCFields;
}

export interface EcvrfIntermediateJSON {
  r: types.FieldElementZCJSON;
  nS: types.FieldElementZCJSON;
  d: types.FieldElementZCJSON;
  t13: types.FieldElementZCJSON;
  t15: types.FieldElementZCJSON;
}

export class EcvrfIntermediate {
  readonly r: types.FieldElementZC;
  readonly nS: types.FieldElementZC;
  readonly d: types.FieldElementZC;
  readonly t13: types.FieldElementZC;
  readonly t15: types.FieldElementZC;

  constructor(fields: EcvrfIntermediateFields) {
    this.r = new types.FieldElementZC({ ...fields.r });
    this.nS = new types.FieldElementZC({ ...fields.nS });
    this.d = new types.FieldElementZC({ ...fields.d });
    this.t13 = new types.FieldElementZC({ ...fields.t13 });
    this.t15 = new types.FieldElementZC({ ...fields.t15 });
  }

  static layout(property?: string) {
    return borsh.struct(
      [
        types.FieldElementZC.layout('r'),
        types.FieldElementZC.layout('nS'),
        types.FieldElementZC.layout('d'),
        types.FieldElementZC.layout('t13'),
        types.FieldElementZC.layout('t15'),
      ],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new EcvrfIntermediate({
      r: types.FieldElementZC.fromDecoded(obj.r),
      nS: types.FieldElementZC.fromDecoded(obj.nS),
      d: types.FieldElementZC.fromDecoded(obj.d),
      t13: types.FieldElementZC.fromDecoded(obj.t13),
      t15: types.FieldElementZC.fromDecoded(obj.t15),
    });
  }

  static toEncodable(fields: EcvrfIntermediateFields) {
    return {
      r: types.FieldElementZC.toEncodable(fields.r),
      nS: types.FieldElementZC.toEncodable(fields.nS),
      d: types.FieldElementZC.toEncodable(fields.d),
      t13: types.FieldElementZC.toEncodable(fields.t13),
      t15: types.FieldElementZC.toEncodable(fields.t15),
    };
  }

  toJSON(): EcvrfIntermediateJSON {
    return {
      r: this.r.toJSON(),
      nS: this.nS.toJSON(),
      d: this.d.toJSON(),
      t13: this.t13.toJSON(),
      t15: this.t15.toJSON(),
    };
  }

  static fromJSON(obj: EcvrfIntermediateJSON): EcvrfIntermediate {
    return new EcvrfIntermediate({
      r: types.FieldElementZC.fromJSON(obj.r),
      nS: types.FieldElementZC.fromJSON(obj.nS),
      d: types.FieldElementZC.fromJSON(obj.d),
      t13: types.FieldElementZC.fromJSON(obj.t13),
      t15: types.FieldElementZC.fromJSON(obj.t15),
    });
  }

  toEncodable() {
    return EcvrfIntermediate.toEncodable(this);
  }
}
