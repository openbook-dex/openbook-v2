import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface HashFields {
  /** The bytes used to derive the hash. */
  data: Array<number>;
}

export interface HashJSON {
  /** The bytes used to derive the hash. */
  data: Array<number>;
}

export class Hash {
  /** The bytes used to derive the hash. */
  readonly data: Array<number>;

  constructor(fields: HashFields) {
    this.data = fields.data;
  }

  static layout(property?: string) {
    return borsh.struct([borsh.array(borsh.u8(), 32, 'data')], property);
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new Hash({
      data: obj.data,
    });
  }

  static toEncodable(fields: HashFields) {
    return {
      data: fields.data,
    };
  }

  toJSON(): HashJSON {
    return {
      data: this.data,
    };
  }

  static fromJSON(obj: HashJSON): Hash {
    return new Hash({
      data: obj.data,
    });
  }

  toEncodable() {
    return Hash.toEncodable(this);
  }
}
