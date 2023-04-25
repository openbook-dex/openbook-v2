import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface VrfProveParamsFields {
  proof: Uint8Array;
  idx: number;
}

export interface VrfProveParamsJSON {
  proof: Array<number>;
  idx: number;
}

export class VrfProveParams {
  readonly proof: Uint8Array;
  readonly idx: number;

  constructor(fields: VrfProveParamsFields) {
    this.proof = fields.proof;
    this.idx = fields.idx;
  }

  static layout(property?: string) {
    return borsh.struct([borsh.vecU8('proof'), borsh.u32('idx')], property);
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new VrfProveParams({
      proof: new Uint8Array(
        obj.proof.buffer,
        obj.proof.byteOffset,
        obj.proof.length
      ),
      idx: obj.idx,
    });
  }

  static toEncodable(fields: VrfProveParamsFields) {
    return {
      proof: Buffer.from(
        fields.proof.buffer,
        fields.proof.byteOffset,
        fields.proof.length
      ),
      idx: fields.idx,
    };
  }

  toJSON(): VrfProveParamsJSON {
    return {
      proof: Array.from(this.proof.values()),
      idx: this.idx,
    };
  }

  static fromJSON(obj: VrfProveParamsJSON): VrfProveParams {
    return new VrfProveParams({
      proof: Uint8Array.from(obj.proof),
      idx: obj.idx,
    });
  }

  toEncodable() {
    return VrfProveParams.toEncodable(this);
  }
}
