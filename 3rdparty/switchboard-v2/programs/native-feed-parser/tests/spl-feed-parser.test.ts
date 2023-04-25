import * as anchor from "@project-serum/anchor";
import {
  Keypair,
  PublicKey,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
import { SwitchboardTestContext } from "@switchboard-xyz/sbv2-utils";
import fs from "fs";
import path from "path";

function getProgramId(): PublicKey {
  const programKeypairPath = path.join(
    __dirname,
    "..",
    "..",
    "..",
    "..",
    "target",
    "deploy",
    "native_feed_parser-keypair.json"
  );
  const PROGRAM_ID = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync(programKeypairPath, "utf8")))
  ).publicKey;

  return PROGRAM_ID;
}

const sleep = (ms: number): Promise<any> =>
  new Promise((s) => setTimeout(s, ms));

// Anchor.toml will copy this to localnet when we start our tests
const DEFAULT_SOL_USD_FEED = new PublicKey(
  "GvDMxPzN1sCj7L26YDK2HnMRXEQmQ2aemov8YBtPS7vR"
);

describe("native-feed-parser test", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  let switchboard: SwitchboardTestContext;
  let aggregatorKey: PublicKey;

  before(async () => {
    // First, attempt to load the switchboard devnet PID
    try {
      switchboard = await SwitchboardTestContext.loadDevnetQueue(
        provider,
        "F8ce7MsckeZAbAGmxjJNetxYXQa9mKr9nnrC3qKubyYy"
      );
      aggregatorKey = DEFAULT_SOL_USD_FEED;
      console.log("devnet detected");
      return;
    } catch (error: any) {
      console.log(`Error: SBV2 Devnet - ${error.message}`);
    }
    // If fails, fallback to looking for a local env file
    try {
      switchboard = await SwitchboardTestContext.loadFromEnv(provider);
      const aggregatorAccount = await switchboard.createStaticFeed(100);
      aggregatorKey = aggregatorAccount.publicKey ?? PublicKey.default;
      console.log("local env detected");
      return;
    } catch (error: any) {
      console.log(`Error: SBV2 Localnet - ${error.message}`);
    }
    // If fails, throw error
    throw new Error(
      `Failed to load the SwitchboardTestContext from devnet or from a switchboard.env file`
    );
  });

  it("Read SOL/USD Feed", async () => {
    const PROGRAM_ID = getProgramId();

    const readSwitchboardAggregatorTxn = new Transaction().add(
      new TransactionInstruction({
        keys: [
          {
            pubkey: aggregatorKey,
            isSigner: false,
            isWritable: false,
          },
        ],
        programId: new PublicKey(PROGRAM_ID),
        data: Buffer.from([]),
      })
    );

    const signature = await provider.sendAndConfirm(
      readSwitchboardAggregatorTxn
    );

    // wait for RPC
    await sleep(2000);

    const confirmedTxn = await provider.connection.getParsedTransaction(
      signature,
      "confirmed"
    );

    console.log(JSON.stringify(confirmedTxn?.meta?.logMessages, undefined, 2));
  });
});
