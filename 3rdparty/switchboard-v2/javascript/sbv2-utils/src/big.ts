import type { OrcaU64 } from "@orca-so/sdk";
import * as anchor from "@project-serum/anchor";
import type {
  Fraction,
  Price,
  TokenAmount as SaberTokenAmount,
} from "@saberhq/token-utils";
import type { TokenAmount } from "@solana/web3.js";
import { SwitchboardDecimal } from "@switchboard-xyz/switchboard-v2";
import Big from "big.js";
import Decimal from "decimal.js";

export class BigUtils {
  static safeDiv(number_: Big, denominator: Big, decimals = 20): Big {
    const oldDp = Big.DP;
    Big.DP = decimals;
    const result = number_.div(denominator);
    Big.DP = oldDp;
    return result;
  }

  static safeMul(...n: Big[]): Big {
    if (n.length === 0) {
      throw new Error(`need to provide elements to multiply ${n}`);
    }

    let result = new Big(1);
    for (const x of n) {
      result = result.mul(x);
    }

    return result;
  }

  static safeNthRoot(big: Big, nthRoot: number, decimals = 20): Big {
    if (nthRoot <= 0) {
      throw new Error(`cannot take the nth root of a negative number`);
    }

    const oldDp = Big.DP;
    Big.DP = decimals;

    const decimal = BigUtils.toDecimal(big);
    const frac = new Decimal(1).div(nthRoot);
    const root: Decimal =
      big.s === -1
        ? decimal.abs().pow(frac).mul(new Decimal(big.s))
        : decimal.pow(frac);

    const result: Big = BigUtils.fromDecimal(root);

    Big.DP = oldDp;

    return result;
  }

  static safeSqrt(n: Big, decimals = 20): Big {
    const oldDp = Big.DP;
    Big.DP = decimals;
    const result = n.sqrt();
    Big.DP = oldDp;
    return result;
  }

  static safePow(n: Big, exp: number, decimals = 20): Big {
    const oldDp = Big.DP;
    Big.DP = decimals;

    const oldPrecision = Decimal.precision;
    Decimal.set({ precision: decimals });
    const base = BigUtils.toDecimal(n);
    const value = base.pow(exp);
    const result = BigUtils.fromDecimal(value);
    Decimal.set({ precision: oldPrecision });

    Big.DP = oldDp;
    return result;
  }

  static fromBN(n: anchor.BN, decimals = 0): Big {
    const big = new SwitchboardDecimal(n, decimals).toBig();
    // assert(n.cmp(new anchor.BN(big.toFixed())) === 0);
    return big;
  }

  static toDecimal(big: Big, decimals = 20): Decimal {
    const decimal = new Decimal(big.toFixed(decimals, 0));
    // assert(decimal.toFixed() === big.toFixed());
    return decimal;
    // const b = new Big(big);

    // const decimal = new Decimal(0);
    // (decimal as any).d = groupArray(b.c);
    // (decimal as any).e = b.e;
    // (decimal as any).s = b.s;

    // console.log(`toDecimal: ${big.toString()} => ${decimal.toString()}`);
    // return decimal;
  }

  static fromDecimal(decimal: Decimal, decimals = 20): Big {
    if (decimal.isNaN()) {
      throw new TypeError(`cannot convert NaN decimal.js to Big.js`);
    }

    if (!decimal.isFinite()) {
      throw new TypeError(`cannot convert INF decimal.js to Big.js`);
    }

    const big = new Big(decimal.toFixed(decimals, 0));
    // assert(big.toFixed() === decimal.toFixed());
    return big;
    // const d = new Decimal(decimal);

    // const big = new Big(0);
    // console.log(`fromDecimal (${d.toString()}) d.d ${d.d}`);
    // big.c = splitToDigits(d.d);
    // big.e = d.e;
    // big.s = d.s;

    // console.log(`fromDecimal: ${decimal.toString()} => ${big.toString()}`);
    // return big;
  }

  static fromOrcaU64(u64: OrcaU64): Big {
    return BigUtils.fromBN(new anchor.BN(u64.value), u64.scale);
  }

  static fromSaberTokenAmount(token: SaberTokenAmount): Big {
    return BigUtils.fromBN(
      new anchor.BN(token.toU64()),
      token.token.info.decimals
    );
  }

  static fromTokenAmount(token: TokenAmount): Big {
    return BigUtils.fromBN(new anchor.BN(token.amount), token.decimals);
  }

  static fromPrice(price: Price | Fraction): Big {
    const numerator = new Big(price.numerator.toString());
    const denominator = new Big(price.denominator.toString());
    return BigUtils.safeDiv(numerator, denominator);
  }
}
