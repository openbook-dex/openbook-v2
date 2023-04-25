import { PublicKey } from "@solana/web3.js";
import { SwitchboardDecimal } from "@switchboard-xyz/switchboard-v2";
import Big from "big.js";
import BN from "bn.js";

function big2NumberOrString(big: Big): number | string {
  const oldStrict = Big.strict;
  Big.strict = true;
  try {
    const num = big.toNumber();
    Big.strict = oldStrict;
    return num;
  } catch {}
  Big.strict = oldStrict;
  return big.toString();
}

export function jsonReplacers(key: any, value: any): any {
  if (typeof value === "string" || typeof value === "number") {
    return value;
  }
  // BN
  if (BN.isBN(value)) {
    return value.toNumber();
  }
  if (
    value instanceof SwitchboardDecimal ||
    (value &&
      typeof value === "object" &&
      "mantissa" in value &&
      "scale" in value)
  ) {
    const swbDecimal = new SwitchboardDecimal(value.mantissa, value.scale);
    return big2NumberOrString(swbDecimal.toBig());
  }
  // big.js
  if (value instanceof Big) {
    return big2NumberOrString(value);
  }
  // pubkey
  if (value instanceof PublicKey) {
    return value.toBase58();
  }
  // bigint
  if (typeof value === "bigint") {
    return value.toString();
  }

  // Fall through for nested objects
  return value;
}
