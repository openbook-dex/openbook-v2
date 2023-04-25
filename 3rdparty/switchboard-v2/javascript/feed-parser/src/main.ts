/* eslint-disable unicorn/no-process-exit */
import { clusterApiUrl, Connection, Keypair, PublicKey } from "@solana/web3.js";
import {
  AggregatorAccount,
  SwitchboardProgram,
} from "@switchboard-xyz/solana.js";

// SOL/USD Feed https://switchboard.xyz/explorer/2/GvDMxPzN1sCj7L26YDK2HnMRXEQmQ2aemov8YBtPS7vR
// Create your own feed here https://publish.switchboard.xyz/
const switchboardFeed = new PublicKey(
  "GvDMxPzN1sCj7L26YDK2HnMRXEQmQ2aemov8YBtPS7vR"
);

async function main() {
  // load the switchboard program
  const program = await SwitchboardProgram.load(
    "devnet",
    new Connection(clusterApiUrl("devnet")),
    Keypair.fromSeed(new Uint8Array(32).fill(1)) // using dummy keypair since we wont be submitting any transactions
  );

  // load the switchboard aggregator
  const aggregator = new AggregatorAccount(program, switchboardFeed);

  // get the result
  const result = await aggregator.fetchLatestValue();
  console.log(`Switchboard Result: ${result}`);
}

main().then(
  () => process.exit(),
  (error) => {
    console.error("Failed to parse Switchboard Feed");
    console.error(error);
    process.exit(-1);
  }
);
