import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface ScalarFields {
  /**
   * `bytes` is a little-endian byte encoding of an integer representing a scalar modulo the
   * group order.
   *
   * # Invariant
   *
   * The integer representing this scalar must be bounded above by \\(2\^{255}\\), or
   * equivalently the high bit of `bytes[31]` must be zero.
   *
   * This ensures that there is room for a carry bit when computing a NAF representation.
   */
  bytes: Array<number>;
}

export interface ScalarJSON {
  /**
   * `bytes` is a little-endian byte encoding of an integer representing a scalar modulo the
   * group order.
   *
   * # Invariant
   *
   * The integer representing this scalar must be bounded above by \\(2\^{255}\\), or
   * equivalently the high bit of `bytes[31]` must be zero.
   *
   * This ensures that there is room for a carry bit when computing a NAF representation.
   */
  bytes: Array<number>;
}

/**
 * The `Scalar` struct holds an integer \\(s < 2\^{255} \\) which
 * represents an element of \\(\mathbb Z / \ell\\).
 */
export class Scalar {
  /**
   * `bytes` is a little-endian byte encoding of an integer representing a scalar modulo the
   * group order.
   *
   * # Invariant
   *
   * The integer representing this scalar must be bounded above by \\(2\^{255}\\), or
   * equivalently the high bit of `bytes[31]` must be zero.
   *
   * This ensures that there is room for a carry bit when computing a NAF representation.
   */
  readonly bytes: Array<number>;

  constructor(fields: ScalarFields) {
    this.bytes = fields.bytes;
  }

  static layout(property?: string) {
    return borsh.struct([borsh.array(borsh.u8(), 32, 'bytes')], property);
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new Scalar({
      bytes: obj.bytes,
    });
  }

  static toEncodable(fields: ScalarFields) {
    return {
      bytes: fields.bytes,
    };
  }

  toJSON(): ScalarJSON {
    return {
      bytes: this.bytes,
    };
  }

  static fromJSON(obj: ScalarJSON): Scalar {
    return new Scalar({
      bytes: obj.bytes,
    });
  }

  toEncodable() {
    return Scalar.toEncodable(this);
  }
}
