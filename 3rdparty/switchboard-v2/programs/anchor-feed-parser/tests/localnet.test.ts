import * as anchor from "@project-serum/anchor";
import { Connection } from "@solana/web3.js";
import { sleep, SwitchboardTestContext } from "@switchboard-xyz/sbv2-utils";
import type {
  AggregatorAccount,
  AnchorWallet,
} from "@switchboard-xyz/switchboard-v2";
import chai from "chai";
import { PROGRAM_ID } from "../client/programId";
import { AnchorFeedParser, IDL } from "../target/types/anchor_feed_parser";
const expect = chai.expect;

describe("anchor-feed-parser test", () => {
  const tomlProvider = anchor.AnchorProvider.env();
  const provider = new anchor.AnchorProvider(
    new Connection("http://localhost:8899"),
    tomlProvider.wallet,
    {}
  );
  anchor.setProvider(provider);

  // const feedParserProgram = anchor.workspace
  //   .AnchorFeedParser as Program<AnchorFeedParser>;

  const feedParserProgram = new anchor.Program(
    IDL,
    PROGRAM_ID,
    provider,
    new anchor.BorshCoder(IDL)
  ) as anchor.Program<AnchorFeedParser>;

  const payer = (provider.wallet as AnchorWallet).payer;

  let switchboard: SwitchboardTestContext;
  let aggregatorAccount: AggregatorAccount;

  before(async () => {
    try {
      switchboard = await SwitchboardTestContext.loadFromEnv(provider);
      console.log("local env detected");
      return;
    } catch (error: any) {
      console.log(`Error: SBV2 Localnet - ${error.message}`);
      console.error(error);
    }

    throw new Error(`Failed to load the localnet Switchboard environment`);
  });

  it("Creates a static feed that resolves to 100", async () => {
    aggregatorAccount = await switchboard.createStaticFeed(100);

    console.log(`Created Feed: ${aggregatorAccount.publicKey}`);
  });

  it("Reads the static feed", async () => {
    const signature = await feedParserProgram.methods
      .readResult({ maxConfidenceInterval: 0.25 })
      .accounts({ aggregator: aggregatorAccount.publicKey })
      .rpc();

    // wait for RPC
    await sleep(2000);

    const logs = await provider.connection.getParsedTransaction(
      signature,
      "confirmed"
    );

    // TODO: grep logs and verify the price

    console.log(JSON.stringify(logs?.meta?.logMessages, undefined, 2));
  });

  it("Fails to read feed if confidence interval is exceeded", async () => {
    try {
      await feedParserProgram.methods
        .readResult({ maxConfidenceInterval: 0.0000000001 })
        .accounts({ aggregator: aggregatorAccount.publicKey })
        .rpc();
    } catch (error: any) {
      if (!error.toString().includes("ConfidenceIntervalExceeded")) {
        throw error;
      }
    }
  });

  it("Updates static feed to resolve to 110", async () => {
    await switchboard.updateStaticFeed(aggregatorAccount, 110, 45);

    const signature = await feedParserProgram.methods
      .readResult({ maxConfidenceInterval: 0.25 })
      .accounts({ aggregator: aggregatorAccount.publicKey })
      .rpc();

    // wait for RPC
    await sleep(2000);

    const logs = await provider.connection.getParsedTransaction(
      signature,
      "confirmed"
    );

    // TODO: grep logs and verify the price

    console.log(JSON.stringify(logs?.meta?.logMessages, undefined, 2));
  });
});
