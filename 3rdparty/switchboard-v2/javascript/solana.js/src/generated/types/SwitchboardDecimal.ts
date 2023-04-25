import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';
import Big from 'big.js';

export interface SwitchboardDecimalFields {
  /**
   * The part of a floating-point number that represents the significant digits of that number,
   * and that is multiplied by the base, 10, raised to the power of scale to give the actual value of the number.
   */
  mantissa: BN;
  /** The number of decimal places to move to the left to yield the actual value. */
  scale: number;
}

export interface SwitchboardDecimalJSON {
  /**
   * The part of a floating-point number that represents the significant digits of that number,
   * and that is multiplied by the base, 10, raised to the power of scale to give the actual value of the number.
   */
  mantissa: string;
  /** The number of decimal places to move to the left to yield the actual value. */
  scale: number;
}

export class SwitchboardDecimal {
  /**
   * The part of a floating-point number that represents the significant digits of that number,
   * and that is multiplied by the base, 10, raised to the power of scale to give the actual value of the number.
   */
  readonly mantissa: BN;
  /** The number of decimal places to move to the left to yield the actual value. */
  readonly scale: number;

  constructor(fields: SwitchboardDecimalFields) {
    this.mantissa = fields.mantissa;
    this.scale = fields.scale;
  }

  static layout(property?: string) {
    return borsh.struct([borsh.i128('mantissa'), borsh.u32('scale')], property);
  }

  get borsh(): types.BorshDecimal {
    return new types.BorshDecimal({
      mantissa: this.mantissa,
      scale: this.scale,
    });
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new SwitchboardDecimal({
      mantissa: obj.mantissa,
      scale: obj.scale,
    });
  }

  static toEncodable(fields: SwitchboardDecimalFields) {
    return {
      mantissa: fields.mantissa,
      scale: fields.scale,
    };
  }

  toJSON(): SwitchboardDecimalJSON {
    return {
      mantissa: this.mantissa.toString(),
      scale: this.scale,
    };
  }

  static fromJSON(obj: SwitchboardDecimalJSON): SwitchboardDecimal {
    return new SwitchboardDecimal({
      mantissa: new BN(obj.mantissa),
      scale: obj.scale,
    });
  }

  toEncodable() {
    return SwitchboardDecimal.toEncodable(this);
  }

  /**
   * Convert untyped object to a Switchboard decimal, if possible.
   * @param obj raw object to convert from
   * @return SwitchboardDecimal
   */
  public static from(obj: {
    mantissa: string | number | BN;
    scale: number;
  }): SwitchboardDecimal {
    return new SwitchboardDecimal({
      mantissa: new BN(obj.mantissa),
      scale: obj.scale,
    });
  }

  /**
   * Convert a Big.js decimal to a Switchboard decimal.
   * @param big a Big.js decimal
   * @return a SwitchboardDecimal
   */
  public static fromBig(big: Big): SwitchboardDecimal {
    // Round to fit in Switchboard Decimal
    // TODO: smarter logic.
    big = big.round(20);
    let mantissa: BN = new BN(big.c.join(''), 10);
    // Set the scale. Big.exponenet sets scale from the opposite side
    // SwitchboardDecimal does.
    let scale = big.c.slice(1).length - big.e;

    if (scale < 0) {
      mantissa = mantissa.mul(new BN(10, 10).pow(new BN(Math.abs(scale), 10)));
      scale = 0;
    }
    if (scale < 0) {
      throw new Error('SwitchboardDecimal: Unexpected negative scale.');
    }
    if (scale >= 28) {
      throw new Error('SwitchboardDecimalExcessiveScaleError');
    }

    // Set sign for the coefficient (mantissa)
    mantissa = mantissa.mul(new BN(big.s, 10));

    const result = new SwitchboardDecimal({ mantissa, scale });
    if (big.sub(result.toBig()).abs().gt(new Big(0.00005))) {
      throw new Error(
        'SwitchboardDecimal: Converted decimal does not match original:\n' +
          `out: ${result.toBig().toNumber()} vs in: ${big.toNumber()}\n` +
          `-- result mantissa and scale: ${result.mantissa.toString()} ${result.scale.toString()}\n` +
          `${result} ${result.toBig()}`
      );
    }
    return result;
  }

  /**
   * SwitchboardDecimal equality comparator.
   * @param other object to compare to.
   * @return true iff equal
   */
  public eq(other: SwitchboardDecimal): boolean {
    return this.mantissa.eq(other.mantissa) && this.scale === other.scale;
  }

  /**
   * Convert SwitchboardDecimal to big.js Big type.
   * @return Big representation
   */
  public toBig(): Big {
    let mantissa: BN = new BN(this.mantissa, 10);
    let s = 1;
    const c: Array<number> = [];
    const ZERO = new BN(0, 10);
    const TEN = new BN(10, 10);
    if (mantissa.lt(ZERO)) {
      s = -1;
      mantissa = mantissa.abs();
    }
    while (mantissa.gt(ZERO)) {
      c.unshift(mantissa.mod(TEN).toNumber());
      mantissa = mantissa.div(TEN);
    }
    const e = c.length - this.scale - 1;
    const result = new Big(0);
    if (c.length === 0) {
      return result;
    }
    result.s = s;
    result.c = c;
    result.e = e;
    return result;
  }

  toString() {
    return this.toBig().toString();
  }
}
