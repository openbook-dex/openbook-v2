import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface EcvrfProofZCFields {
  gamma: types.EdwardsPointZCFields;
  c: types.ScalarFields;
  s: types.ScalarFields;
}

export interface EcvrfProofZCJSON {
  gamma: types.EdwardsPointZCJSON;
  c: types.ScalarJSON;
  s: types.ScalarJSON;
}

export class EcvrfProofZC {
  readonly gamma: types.EdwardsPointZC;
  readonly c: types.Scalar;
  readonly s: types.Scalar;

  constructor(fields: EcvrfProofZCFields) {
    this.gamma = new types.EdwardsPointZC({ ...fields.gamma });
    this.c = new types.Scalar({ ...fields.c });
    this.s = new types.Scalar({ ...fields.s });
  }

  static layout(property?: string) {
    return borsh.struct(
      [
        types.EdwardsPointZC.layout('gamma'),
        types.Scalar.layout('c'),
        types.Scalar.layout('s'),
      ],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new EcvrfProofZC({
      gamma: types.EdwardsPointZC.fromDecoded(obj.gamma),
      c: types.Scalar.fromDecoded(obj.c),
      s: types.Scalar.fromDecoded(obj.s),
    });
  }

  static toEncodable(fields: EcvrfProofZCFields) {
    return {
      gamma: types.EdwardsPointZC.toEncodable(fields.gamma),
      c: types.Scalar.toEncodable(fields.c),
      s: types.Scalar.toEncodable(fields.s),
    };
  }

  toJSON(): EcvrfProofZCJSON {
    return {
      gamma: this.gamma.toJSON(),
      c: this.c.toJSON(),
      s: this.s.toJSON(),
    };
  }

  static fromJSON(obj: EcvrfProofZCJSON): EcvrfProofZC {
    return new EcvrfProofZC({
      gamma: types.EdwardsPointZC.fromJSON(obj.gamma),
      c: types.Scalar.fromJSON(obj.c),
      s: types.Scalar.fromJSON(obj.s),
    });
  }

  toEncodable() {
    return EcvrfProofZC.toEncodable(this);
  }
}
