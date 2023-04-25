import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface FieldElementZCFields {
  bytes: Array<BN>;
}

export interface FieldElementZCJSON {
  bytes: Array<string>;
}

export class FieldElementZC {
  readonly bytes: Array<BN>;

  constructor(fields: FieldElementZCFields) {
    this.bytes = fields.bytes;
  }

  static layout(property?: string) {
    return borsh.struct([borsh.array(borsh.u64(), 5, 'bytes')], property);
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new FieldElementZC({
      bytes: obj.bytes,
    });
  }

  static toEncodable(fields: FieldElementZCFields) {
    return {
      bytes: fields.bytes,
    };
  }

  toJSON(): FieldElementZCJSON {
    return {
      bytes: this.bytes.map(item => item.toString()),
    };
  }

  static fromJSON(obj: FieldElementZCJSON): FieldElementZC {
    return new FieldElementZC({
      bytes: obj.bytes.map(item => new BN(item)),
    });
  }

  toEncodable() {
    return FieldElementZC.toEncodable(this);
  }
}
