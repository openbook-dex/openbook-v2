/** This test will only work on devnet because we need a populated history to read */
import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { SwitchboardTestContext } from "@switchboard-xyz/sbv2-utils";
import { OracleJob } from "@switchboard-xyz/common";
import {
  AggregatorAccount,
  JobAccount,
  loadSwitchboardProgram,
  programWallet,
} from "@switchboard-xyz/switchboard-v2";
import { AnchorHistoryParser } from "../target/types/anchor_history_parser";

export const AGGREGATOR_PUBKEY: anchor.web3.PublicKey =
  new anchor.web3.PublicKey("GvDMxPzN1sCj7L26YDK2HnMRXEQmQ2aemov8YBtPS7vR");

export const HISTORY_BUFFER_PUBKEY: anchor.web3.PublicKey =
  new anchor.web3.PublicKey("7LLvRhMs73FqcLkA8jvEE1AM2mYZXTmqfUv8GAEurymx");

export const sleep = (ms: number): Promise<any> =>
  new Promise((s) => setTimeout(s, ms));

describe("anchor-history-parser", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace
    .AnchorHistoryParser as Program<AnchorHistoryParser>;

  let aggregatorAccount: AggregatorAccount;
  let historyBuffer: anchor.web3.PublicKey;

  let switchboard: SwitchboardTestContext;

  before(async () => {
    try {
      switchboard = await SwitchboardTestContext.loadDevnetQueue(
        program.provider as anchor.AnchorProvider,
        "F8ce7MsckeZAbAGmxjJNetxYXQa9mKr9nnrC3qKubyYy"
      );
      console.log("devnet detected");
      return;
    } catch (error: any) {
      console.log(`Error: SBV2 Devnet - ${error.message}`);
    }
    // If fails, throw error
    throw new Error(`Failed to load the SwitchboardTestContext from devnet`);
  });

  /** Example showing how to create a new data feed with a history buffer storing 200k samples.
   * Note: This will not update until added to a crank or manually calling openRound
   */
  it("Creates a feed with a history buffer", async () => {
    const aggregatorAccount = await AggregatorAccount.create(
      switchboard.program,
      {
        name: Buffer.from("History Aggregator"),
        batchSize: 1,
        minRequiredOracleResults: 1,
        minRequiredJobResults: 1,
        minUpdateDelaySeconds: 30,
        queueAccount: switchboard.queue,
      }
    );
    const jobAccount = await JobAccount.create(switchboard.program, {
      name: Buffer.from("Example Job"),
      authority: anchor.web3.PublicKey.default,
      data: Buffer.from(
        OracleJob.encodeDelimited(
          OracleJob.create({
            tasks: [
              {
                valueTask: {
                  value: 1,
                },
              },
            ],
          })
        ).finish()
      ),
    });
    await aggregatorAccount.addJob(jobAccount);

    const historyBufferKeypair = anchor.web3.Keypair.generate();
    await aggregatorAccount.setHistoryBuffer({ size: 200_000 });
  });

  /** Example showing how to read a history buffer on-chain for an existing feed with an existing history buffer with pre-populated samples. (This will only work on devnet) */
  it("Reads an aggregator history buffer", async () => {
    // const ONE_HOUR_AGO: number = Math.floor(Date.now()) - 60 * 60;

    const aggregatorAccount = new AggregatorAccount({
      program: switchboard.program,
      publicKey: AGGREGATOR_PUBKEY,
    });
    const aggregator = await aggregatorAccount.loadData();

    // TODO: Verify the value in the program logs matches the history samples
    const history = await aggregatorAccount.loadHistory();

    const tx = await program.methods
      .readHistory({ timestamp: null })
      .accounts({
        aggregator: AGGREGATOR_PUBKEY,
        historyBuffer: aggregator.historyBuffer,
      })
      .rpc();
    console.log("Your transaction signature", tx);

    await sleep(5000);

    const confirmedTxn = await program.provider.connection.getParsedTransaction(
      tx,
      "confirmed"
    );

    console.log(JSON.stringify(confirmedTxn?.meta?.logMessages, undefined, 2));
  });
});
