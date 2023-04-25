import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface VrfRoundFields {
  /** The alpha bytes used to calculate the VRF proof. */
  alpha: Array<number>;
  /** The number of bytes in the alpha buffer. */
  alphaLen: number;
  /** The Slot when the VRF round was opened. */
  requestSlot: BN;
  /** The unix timestamp when the VRF round was opened. */
  requestTimestamp: BN;
  /** The VRF round result. Will be zeroized if still awaiting fulfillment. */
  result: Array<number>;
  /** The number of builders who verified the VRF proof. */
  numVerified: number;
  /** Reserved for future info. */
  ebuf: Array<number>;
}

export interface VrfRoundJSON {
  /** The alpha bytes used to calculate the VRF proof. */
  alpha: Array<number>;
  /** The number of bytes in the alpha buffer. */
  alphaLen: number;
  /** The Slot when the VRF round was opened. */
  requestSlot: string;
  /** The unix timestamp when the VRF round was opened. */
  requestTimestamp: string;
  /** The VRF round result. Will be zeroized if still awaiting fulfillment. */
  result: Array<number>;
  /** The number of builders who verified the VRF proof. */
  numVerified: number;
  /** Reserved for future info. */
  ebuf: Array<number>;
}

export class VrfRound {
  /** The alpha bytes used to calculate the VRF proof. */
  readonly alpha: Array<number>;
  /** The number of bytes in the alpha buffer. */
  readonly alphaLen: number;
  /** The Slot when the VRF round was opened. */
  readonly requestSlot: BN;
  /** The unix timestamp when the VRF round was opened. */
  readonly requestTimestamp: BN;
  /** The VRF round result. Will be zeroized if still awaiting fulfillment. */
  readonly result: Array<number>;
  /** The number of builders who verified the VRF proof. */
  readonly numVerified: number;
  /** Reserved for future info. */
  readonly ebuf: Array<number>;

  constructor(fields: VrfRoundFields) {
    this.alpha = fields.alpha;
    this.alphaLen = fields.alphaLen;
    this.requestSlot = fields.requestSlot;
    this.requestTimestamp = fields.requestTimestamp;
    this.result = fields.result;
    this.numVerified = fields.numVerified;
    this.ebuf = fields.ebuf;
  }

  static layout(property?: string) {
    return borsh.struct(
      [
        borsh.array(borsh.u8(), 256, 'alpha'),
        borsh.u32('alphaLen'),
        borsh.u64('requestSlot'),
        borsh.i64('requestTimestamp'),
        borsh.array(borsh.u8(), 32, 'result'),
        borsh.u32('numVerified'),
        borsh.array(borsh.u8(), 256, 'ebuf'),
      ],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new VrfRound({
      alpha: obj.alpha,
      alphaLen: obj.alphaLen,
      requestSlot: obj.requestSlot,
      requestTimestamp: obj.requestTimestamp,
      result: obj.result,
      numVerified: obj.numVerified,
      ebuf: obj.ebuf,
    });
  }

  static toEncodable(fields: VrfRoundFields) {
    return {
      alpha: fields.alpha,
      alphaLen: fields.alphaLen,
      requestSlot: fields.requestSlot,
      requestTimestamp: fields.requestTimestamp,
      result: fields.result,
      numVerified: fields.numVerified,
      ebuf: fields.ebuf,
    };
  }

  toJSON(): VrfRoundJSON {
    return {
      alpha: this.alpha,
      alphaLen: this.alphaLen,
      requestSlot: this.requestSlot.toString(),
      requestTimestamp: this.requestTimestamp.toString(),
      result: this.result,
      numVerified: this.numVerified,
      ebuf: this.ebuf,
    };
  }

  static fromJSON(obj: VrfRoundJSON): VrfRound {
    return new VrfRound({
      alpha: obj.alpha,
      alphaLen: obj.alphaLen,
      requestSlot: new BN(obj.requestSlot),
      requestTimestamp: new BN(obj.requestTimestamp),
      result: obj.result,
      numVerified: obj.numVerified,
      ebuf: obj.ebuf,
    });
  }

  toEncodable() {
    return VrfRound.toEncodable(this);
  }
}
