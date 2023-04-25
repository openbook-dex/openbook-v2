import * as anchor from "@project-serum/anchor";
import {
  AnchorWallet,
  JobAccount,
  OracleJob,
} from "@switchboard-xyz/switchboard-v2";
// import fs from "fs";
import "mocha";
import {
  awaitOpenRound,
  createAggregator,
  sleep,
  SwitchboardTestContext,
} from "../lib/cjs";

describe("Feed tests", () => {
  const payer = anchor.web3.Keypair.generate();

  // const payer = anchor.web3.Keypair.fromSecretKey(
  //   new Uint8Array(
  //     JSON.parse(fs.readFileSync("../../../payer-keypair.json", "utf8"))
  //   )
  // );

  let switchboard: SwitchboardTestContext;

  before(async () => {
    try {
      const localnetConnection = new anchor.web3.Connection(
        "http://localhost:8899"
      );
      const provider = new anchor.AnchorProvider(
        localnetConnection,
        new AnchorWallet(payer),
        { commitment: "confirmed" }
      );
      switchboard = await SwitchboardTestContext.loadFromEnv(provider);
      console.log("local env detected");
      return;
    } catch (error: any) {}

    try {
      const devnetConnection = new anchor.web3.Connection(
        "https://switchboard.devnet.rpcpool.com/f9fe774d81ba4527a418f5b19477"
      );
      const provider = new anchor.AnchorProvider(
        devnetConnection,
        new AnchorWallet(payer),
        { commitment: "confirmed" }
      );
      // const airdropSignature = await devnetConnection.requestAirdrop(
      //   payer.publicKey,
      //   anchor.web3.LAMPORTS_PER_SOL
      // );
      // await devnetConnection.confirmTransaction(airdropSignature);
      switchboard = await SwitchboardTestContext.loadDevnetQueue(
        provider,
        "F8ce7MsckeZAbAGmxjJNetxYXQa9mKr9nnrC3qKubyYy"
      );
      console.log("devnet detected");
      return;
    } catch (error: any) {
      // console.log(`Error: SBV2 Devnet - ${error.message}`);
      console.error(error);
    }

    if (!switchboard) {
      // If fails, throw error
      throw new Error(
        `Failed to load the SwitchboardTestContext from devnet or from a switchboard.env file`
      );
    }

    const airdropSignature =
      await switchboard.program.provider.connection.requestAirdrop(
        payer.publicKey,
        anchor.web3.LAMPORTS_PER_SOL
      );
    await switchboard.program.provider.connection.confirmTransaction(
      airdropSignature
    );
  });

  it("Creates a switchboard feed", async () => {
    const job1 = await JobAccount.create(switchboard.program, {
      name: Buffer.from("Job1"),
      authority: payer.publicKey,
      data: Buffer.from(
        OracleJob.encodeDelimited(
          OracleJob.create({
            tasks: [
              OracleJob.Task.create({
                httpTask: OracleJob.HttpTask.create({
                  url: `https://ftx.us/api/markets/SOL_USD`,
                }),
              }),
              OracleJob.Task.create({
                jsonParseTask: OracleJob.JsonParseTask.create({
                  path: "$.result.price",
                }),
              }),
            ],
          })
        ).finish()
      ),
    });

    const job2 = await JobAccount.create(switchboard.program, {
      name: Buffer.from("Job1"),
      authority: payer.publicKey,
      data: Buffer.from(
        OracleJob.encodeDelimited(
          OracleJob.create({
            tasks: [
              OracleJob.Task.create({
                httpTask: OracleJob.HttpTask.create({
                  url: "https://www.binance.com/api/v3/ticker/price?symbol=SOLUSDT",
                }),
              }),
              OracleJob.Task.create({
                jsonParseTask: OracleJob.JsonParseTask.create({
                  path: "$.price",
                }),
              }),
              OracleJob.Task.create({
                multiplyTask: OracleJob.MultiplyTask.create({
                  aggregatorPubkey:
                    "ETAaeeuQBwsh9mC2gCov9WdhJENZuffRMXY2HgjCcSL9",
                }),
              }),
            ],
          })
        ).finish()
      ),
    });

    let retryCount = 10;
    while (retryCount) {
      try {
        await job1.loadData();
        await job2.loadData();
        break;
      } catch {
        await sleep(1000);
        --retryCount;
      }
    }

    try {
      const newAggregator = await createAggregator(
        switchboard.program,
        switchboard.queue,
        {
          name: Buffer.from("Test Feed"),
          batchSize: 1,
          minRequiredOracleResults: 1,
          minRequiredJobResults: 1,
          minUpdateDelaySeconds: 10,
          queueAccount: switchboard.queue,
        },
        [
          [job1, 1],
          [job2, 1],
        ],
        new anchor.BN(12500 * 5)
      );

      console.log(`Created Aggregator: ${newAggregator.publicKey}`);

      // call openRound
      const value = await awaitOpenRound(
        newAggregator,
        switchboard.queue,
        switchboard.payerTokenWallet,
        undefined,
        45
      );

      console.log(`Aggregator Value: ${value.toString()}`);
    } catch (error) {
      console.error(error);
      throw error;
    }
  });
});
