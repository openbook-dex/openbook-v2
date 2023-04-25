/* eslint-disable @typescript-eslint/no-shadow */
/* eslint-disable @typescript-eslint/no-var-requires */
import * as anchor from "@project-serum/anchor";
import * as spl from "@solana/spl-token-v2";
import { Keypair, PublicKey } from "@solana/web3.js";
import { OracleJob } from "@switchboard-xyz/common";
import * as sbv2 from "@switchboard-xyz/switchboard-v2";
import Big from "big.js";
import fs from "fs";
import path from "path";
import { sleep } from "../async.js";
import { awaitOpenRound, createAggregator } from "../feed.js";
import { transferWrappedSol } from "../token.js";

export interface ISwitchboardTestContext {
  program: anchor.Program;
  mint: spl.Mint;
  payerTokenWallet: PublicKey;
  queue: sbv2.OracleQueueAccount;
  oracle?: sbv2.OracleAccount;
}

export class SwitchboardTestContext implements ISwitchboardTestContext {
  program: anchor.Program;

  mint: spl.Mint;

  payerTokenWallet: PublicKey;

  queue: sbv2.OracleQueueAccount;

  oracle?: sbv2.OracleAccount;

  constructor(ctx: ISwitchboardTestContext) {
    this.program = ctx.program;
    this.mint = ctx.mint;
    this.payerTokenWallet = ctx.payerTokenWallet;
    this.queue = ctx.queue;
    this.oracle = ctx.oracle;
  }

  /** Load the associated token wallet for the given payer with a prefunded balance
   * @param program anchor program
   * @param mint the switchboard mint address
   * @param tokenAmount number of tokens to populate in switchboard mint's associated token account
   */
  static async getOrCreateSwitchboardWallet(
    program: anchor.Program,
    mint: spl.Mint,
    tokenAmount: number
  ): Promise<PublicKey> {
    const payerKeypair = sbv2.programWallet(program);

    if (tokenAmount <= 0) {
      return spl.getAssociatedTokenAddress(
        mint.address,
        payerKeypair.publicKey
      );
    }

    const associatedTokenAccount = await spl.getOrCreateAssociatedTokenAccount(
      program.provider.connection,
      payerKeypair,
      mint.address,
      payerKeypair.publicKey
    );

    if (tokenAmount <= associatedTokenAccount.amount) {
      return associatedTokenAccount.address;
    }

    const amountNeeded = tokenAmount - Number(associatedTokenAccount.amount);
    if (amountNeeded <= 0) {
      return associatedTokenAccount.address;
    }

    const balance = await program.provider.connection.getBalance(
      payerKeypair.publicKey
    );

    if (amountNeeded > balance) {
      throw new Error(
        `Payer account does not enough balance to fund new token account, need ${amountNeeded}, have ${balance}`
      );
    }

    const finalBalance = await transferWrappedSol(
      program.provider.connection,
      payerKeypair,
      amountNeeded
    );

    return associatedTokenAccount.address;
  }

  /** Load SwitchboardTestContext using a specified queue
   * @param provider anchor Provider containing connection and payer Keypair
   * @param queueKey the oracle queue to load
   * @param tokenAmount number of tokens to populate in switchboard mint's associated token account
   */
  static async loadDevnetQueue(
    provider: anchor.AnchorProvider,
    queueKey = "F8ce7MsckeZAbAGmxjJNetxYXQa9mKr9nnrC3qKubyYy",
    tokenAmount = 0
  ) {
    const payerKeypair = (provider.wallet as sbv2.AnchorWallet).payer;

    const balance = await provider.connection.getBalance(
      payerKeypair.publicKey
    );
    if (!balance) {
      try {
        await provider.connection.requestAirdrop(
          payerKeypair.publicKey,
          1_000_000_000
        );
      } catch {}
    }

    let program: anchor.Program;
    try {
      program = await sbv2.loadSwitchboardProgram(
        "devnet",
        provider.connection,
        payerKeypair
      );
    } catch (error: any) {
      throw new Error(
        `Failed to load the SBV2 program for the given cluster, ${error.message}`
      );
    }
    let queue: sbv2.OracleQueueAccount;
    let queueData: any;
    try {
      queue = new sbv2.OracleQueueAccount({
        program,
        publicKey: new PublicKey(queueKey),
      });
      queueData = await queue.loadData();
      if (queueData.queue.length < 1) {
        throw new Error(`OracleQueue has no active oracles heartbeating`);
      }
    } catch (error: any) {
      throw new Error(
        `Failed to load the SBV2 queue for the given cluster, ${error.message}`
      );
    }
    let mint: spl.Mint;
    try {
      mint = await queue.loadMint();
    } catch (error: any) {
      throw new Error(
        `Failed to load the SBV2 mint for the given cluster, ${error.message}`
      );
    }

    const payerTokenWallet =
      await SwitchboardTestContext.getOrCreateSwitchboardWallet(
        program,
        mint,
        tokenAmount
      );

    return new SwitchboardTestContext({
      program,
      queue,
      mint,
      payerTokenWallet,
    });
  }

  /** Recursively loop through directories and return the filepath of switchboard.env
   * @param envFileName alternative filename to search for. defaults to switchboard.env
   * @returns the filepath for a switchboard env file to load
   */
  public static findSwitchboardEnv(envFileName = "switchboard.env"): string {
    const NotFoundError = new Error(
      "failed to find switchboard.env file in current directory recursively"
    );
    let retryCount = 5;

    let currentDirectory = process.cwd();
    while (retryCount > 0) {
      // look for switchboard.env
      try {
        const currentPath = path.join(currentDirectory, envFileName);
        if (fs.existsSync(currentPath)) {
          return currentPath;
        }
      } catch {}

      // look for .switchboard directory
      try {
        const localSbvPath = path.join(currentDirectory, ".switchboard");
        if (fs.existsSync(localSbvPath)) {
          const localSbvEnvPath = path.join(localSbvPath, envFileName);
          if (fs.existsSync(localSbvEnvPath)) {
            return localSbvEnvPath;
          }
        }
      } catch {}

      currentDirectory = path.join(currentDirectory, "../");

      --retryCount;
    }

    throw NotFoundError;
  }

  /** Load SwitchboardTestContext from an env file containing $SWITCHBOARD_PROGRAM_ID, $ORACLE_QUEUE, $AGGREGATOR
   * @param provider anchor Provider containing connection and payer Keypair
   * @param filePath filesystem path to env file
   * @param tokenAmount number of tokens to populate in switchboard mint's associated token account
   */
  public static async loadFromEnv(
    provider: anchor.AnchorProvider,
    filePath = SwitchboardTestContext.findSwitchboardEnv(),
    tokenAmount = 0
  ): Promise<SwitchboardTestContext> {
    require("dotenv").config({ path: filePath });
    if (!process.env.SWITCHBOARD_PROGRAM_ID) {
      throw new Error(`your env file must have $SWITCHBOARD_PROGRAM_ID set`);
    }

    const payerKeypair = (provider.wallet as sbv2.AnchorWallet).payer;

    const balance = await provider.connection.getBalance(
      payerKeypair.publicKey
    );
    if (!balance) {
      try {
        const airdropSignature = await provider.connection.requestAirdrop(
          payerKeypair.publicKey,
          1_000_000_000
        );
        await provider.connection.confirmTransaction(airdropSignature);
      } catch {}
    }

    const SWITCHBOARD_PID = new PublicKey(process.env.SWITCHBOARD_PROGRAM_ID);
    const switchboardIdl = await anchor.Program.fetchIdl(
      SWITCHBOARD_PID,
      provider
    );
    if (!switchboardIdl) {
      throw new Error(`failed to load Switchboard IDL`);
    }
    const switchboardProgram = new anchor.Program(
      switchboardIdl,
      SWITCHBOARD_PID,
      provider
    );

    if (!process.env.ORACLE_QUEUE) {
      throw new Error(`your env file must have $ORACLE_QUEUE set`);
    }
    const SWITCHBOARD_QUEUE = new PublicKey(process.env.ORACLE_QUEUE);
    const queue = new sbv2.OracleQueueAccount({
      program: switchboardProgram,
      publicKey: SWITCHBOARD_QUEUE,
    });

    const oracle = process.env.ORACLE
      ? new sbv2.OracleAccount({
          program: switchboardProgram,
          publicKey: new PublicKey(process.env.ORACLE),
        })
      : undefined;

    let mint: spl.Mint;
    try {
      mint = await queue.loadMint();
    } catch (error: any) {
      console.error(error);
      throw new Error(
        `Failed to load the SBV2 mint for the given cluster, ${error}`
      );
    }

    const payerTokenWallet =
      await SwitchboardTestContext.getOrCreateSwitchboardWallet(
        switchboardProgram,
        mint,
        tokenAmount
      );

    const context: ISwitchboardTestContext = {
      program: switchboardProgram,
      mint: mint,
      payerTokenWallet,
      queue,
      oracle,
    };

    return new SwitchboardTestContext(context);
  }

  /** Create a static data feed that resolves to an expected value */
  public async createStaticFeed(
    value: number,
    timeout = 30
  ): Promise<sbv2.AggregatorAccount> {
    const payerKeypair = sbv2.programWallet(this.program);

    const staticJob = await sbv2.JobAccount.create(this.program, {
      name: Buffer.from(`Value ${value}`),
      authority: this.payerTokenWallet,
      data: Buffer.from(
        OracleJob.encodeDelimited(
          OracleJob.create({
            tasks: [
              OracleJob.Task.create({
                valueTask: OracleJob.ValueTask.create({
                  value,
                }),
              }),
            ],
          })
        ).finish()
      ),
    });

    const aggregatorAccount = await createAggregator(
      this.program,
      this.queue,
      {
        batchSize: 1,
        minRequiredJobResults: 1,
        minRequiredOracleResults: 1,
        minUpdateDelaySeconds: 5,
        queueAccount: this.queue,
        authorWallet: this.payerTokenWallet,
        authority: payerKeypair.publicKey,
      },
      [[staticJob, 1]]
    );

    const aggValue = await awaitOpenRound(
      aggregatorAccount,
      this.queue,
      this.payerTokenWallet,
      new Big(value),
      timeout
    );

    return aggregatorAccount;
  }

  /** Update a feed to a single job that resolves to a new expected value
   * @param aggregatorAccount the aggregator to change a job definition for
   * @param value the new expected value
   * @param timeout how long to wait for the oracle to update the aggregator's latestRound result
   */
  public async updateStaticFeed(
    aggregatorAccount: sbv2.AggregatorAccount,
    value: number,
    timeout = 30
  ): Promise<void> {
    const payerKeypair = sbv2.programWallet(this.program);
    const aggregator = await aggregatorAccount.loadData();
    const expectedValue = new Big(value);

    const queue = await this.queue.loadData();

    // remove all existing jobs
    const existingJobs: sbv2.JobAccount[] = aggregator.jobPubkeysData
      // eslint-disable-next-line array-callback-return
      .filter((jobKey: PublicKey) => {
        if (!jobKey.equals(PublicKey.default)) {
          return jobKey;
        }
        return undefined;
      })
      .filter((item: PublicKey | undefined) => item !== undefined)
      .map(
        (jobKey: PublicKey) =>
          new sbv2.JobAccount({
            program: this.program,
            publicKey: jobKey,
          })
      );
    await Promise.all(
      existingJobs.map((job) => aggregatorAccount.removeJob(job, payerKeypair))
    );

    // add new static job
    const staticJob = await sbv2.JobAccount.create(this.program, {
      name: Buffer.from(`Value ${value}`),
      authority: Keypair.generate().publicKey,
      data: Buffer.from(
        OracleJob.encodeDelimited(
          OracleJob.create({
            tasks: [
              OracleJob.Task.create({
                valueTask: OracleJob.ValueTask.create({
                  value,
                }),
              }),
            ],
          })
        ).finish()
      ),
    });
    await aggregatorAccount.addJob(staticJob, payerKeypair);

    const aggValue = await awaitOpenRound(
      aggregatorAccount,
      this.queue,
      this.payerTokenWallet,
      expectedValue,
      timeout
    );
  }

  /** Checks whether the queue has any active oracles heartbeating */
  public async isQueueReady(): Promise<boolean> {
    const queueData = await this.queue.loadData();
    return queueData.queue.length > 0;
  }

  /** Awaits the specified timeout for an oracle to start heartbeating on the queue
   * @param timeout number of seconds to wait for an oracle to start heartbeating
   */
  public async oracleHeartbeat(timeout = 30): Promise<void> {
    const delay = Math.ceil(timeout / 10) * 1000;
    let retryCount = 10;
    while (retryCount) {
      try {
        if (await this.isQueueReady()) {
          return;
        }
      } catch (error) {
        if (
          !(error instanceof Error) ||
          !error.toString().includes("connection refused")
        ) {
          throw error;
        }
      }
      await sleep(delay);
      --retryCount;
    }
    if (timeout <= 0) {
      throw new Error(
        `Timed out waiting for the OracleQueue to have an active oracle heartbeating`
      );
    }
  }
}
