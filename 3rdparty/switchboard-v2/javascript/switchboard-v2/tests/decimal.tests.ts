import * as anchor from "@project-serum/anchor";
import { strict as assert } from "assert";
import Big from "big.js";
import "mocha";
import * as sbv2 from "../src";

describe("Decimal tests", () => {
  it("Converts a SwitchboardDecimal to a Big", async () => {
    let sbd = new sbv2.SwitchboardDecimal(new anchor.BN(8675309), 3);
    assert(sbd.toBig().toNumber() === 8675.309);

    sbd = new sbv2.SwitchboardDecimal(new anchor.BN(-5000), 3);
    assert(sbd.toBig().toNumber() === -5);

    sbd = new sbv2.SwitchboardDecimal(new anchor.BN(0), 0);
    assert(sbd.toBig().toNumber() === 0);
  });

  it("Converts a Big to a SwitchboardDecimal", async () => {
    let b = Big(100.25);
    let sbd = sbv2.SwitchboardDecimal.fromBig(b);
    assert(sbd.mantissa.eq(new anchor.BN(10025, 10)));
    assert(sbd.scale === 2);

    b = Big(10.025);
    sbd = sbv2.SwitchboardDecimal.fromBig(b);
    assert(sbd.mantissa.eq(new anchor.BN(10025, 10)));
    assert(sbd.scale === 3);

    b = Big(0.10025);
    sbd = sbv2.SwitchboardDecimal.fromBig(b);
    assert(sbd.mantissa.eq(new anchor.BN(10025, 10)));
    assert(sbd.scale === 5);

    b = Big(0);
    sbd = sbv2.SwitchboardDecimal.fromBig(b);
    assert(sbd.mantissa.eq(new anchor.BN(0, 10)));
    assert(sbd.scale === 0);

    b = Big(-270.4);
    sbd = sbv2.SwitchboardDecimal.fromBig(b);
    assert(sbd.mantissa.eq(new anchor.BN(-2704, 10)));
    assert(sbd.scale === 1);
  });

  it("Converts a SwitchboardDecimal back and forth", async () => {
    const big = new Big(4.847);
    let sbd = sbv2.SwitchboardDecimal.fromBig(big);
    assert(sbd.toBig().toNumber() === 4.847);

    sbd = sbv2.SwitchboardDecimal.fromBig(sbd.toBig());
    assert(sbd.toBig().toNumber() === 4.847);
  });
});
