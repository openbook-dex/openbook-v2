import { Keypair } from "@solana/web3.js";
import { assert } from "console";
import "mocha";
import { loadSwitchboardProgram, programWallet } from "../src";

describe("Wallet tests", () => {
  it("Get program wallet", async () => {
    const defaultKeypair = Keypair.fromSeed(new Uint8Array(32).fill(1));
    const keypair = Keypair.generate();
    const program = await loadSwitchboardProgram("devnet", undefined, keypair);

    const getKeypair = programWallet(program);
    assert(
      keypair.publicKey.equals(getKeypair.publicKey),
      "Program Wallet does not match generated keypair"
    );
  });
});
