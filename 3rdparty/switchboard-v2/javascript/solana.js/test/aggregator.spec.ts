/* eslint-disable no-unused-vars */
import 'mocha';

import * as sbv2 from '../src';
import { setupTest, TestContext } from './utilts';
import { Keypair } from '@solana/web3.js';
import {
  AggregatorAccount,
  JobAccount,
  LeaseAccount,
  PermissionAccount,
  QueueAccount,
} from '../src';
import { OracleJob } from '@switchboard-xyz/common';
import { assert } from 'console';
import { PermitOracleQueueUsage } from '../src/generated/types/SwitchboardPermission';

describe('Aggregator Tests', () => {
  let ctx: TestContext;

  const queueAuthority = Keypair.generate();
  let queueAccount: QueueAccount;

  let jobAccount: JobAccount;

  let fundedAggregator: AggregatorAccount;

  before(async () => {
    ctx = await setupTest();

    const [oracleQueue] = await sbv2.QueueAccount.create(ctx.program, {
      name: 'aggregator-queue',
      metadata: '',
      authority: queueAuthority.publicKey,
      queueSize: 1,
      reward: 0,
      minStake: 0,
      oracleTimeout: 86400,
      slashingEnabled: false,
      unpermissionedFeeds: true,
      unpermissionedVrf: true,
      enableBufferRelayers: false,
    });

    queueAccount = oracleQueue;

    await ctx.program.mint.getOrCreateAssociatedUser(ctx.program.walletPubkey);

    // add a single oracle for open round calls
    await queueAccount.createOracle({
      name: 'oracle-1',
    });

    const [jobAccount1] = await JobAccount.create(ctx.program, {
      data: OracleJob.encodeDelimited(
        OracleJob.fromObject({
          tasks: [
            {
              valueTask: {
                value: 1,
              },
            },
          ],
        })
      ).finish(),
      name: 'Job1',
    });
    jobAccount = jobAccount1;
  });

  it("Adds job, updates it's weight, then removes it from aggregator", async () => {
    const aggregatorKeypair = Keypair.generate();
    const aggregatorAuthority = Keypair.generate();

    const [aggregatorAccount] = await AggregatorAccount.create(ctx.program, {
      queueAccount,
      queueAuthority: queueAuthority.publicKey,
      authority: aggregatorAuthority.publicKey,
      batchSize: 1,
      minRequiredOracleResults: 1,
      minRequiredJobResults: 1,
      minUpdateDelaySeconds: 60,
      keypair: aggregatorKeypair,
    });
    await aggregatorAccount.loadData();

    const oracleJob = OracleJob.fromObject({
      tasks: [
        {
          valueTask: {
            value: 1,
          },
        },
      ],
    });

    const [jobAccount] = await JobAccount.create(ctx.program, {
      data: OracleJob.encodeDelimited(oracleJob).finish(),
      name: 'Job1',
    });

    await aggregatorAccount.addJob({
      job: jobAccount,
      weight: 1,
      authority: aggregatorAuthority,
    });

    const postAddJobAggregatorState = await aggregatorAccount.loadData();
    const jobIdx = postAddJobAggregatorState.jobPubkeysData.findIndex(pubkey =>
      pubkey.equals(jobAccount.publicKey)
    );
    if (jobIdx === -1) {
      throw new Error(`Failed to add job to aggregator`);
    }

    await aggregatorAccount.updateJobWeight({
      job: jobAccount,
      jobIdx: jobIdx,
      weight: 2,
      authority: aggregatorAuthority,
    });
    const postUpdateWeightAggregatorState = await aggregatorAccount.loadData();
    if (postUpdateWeightAggregatorState.jobWeights[0] !== 2) {
      throw new Error(`Failed to update job weight in aggregator`);
    }

    await aggregatorAccount.removeJob({
      job: jobAccount,
      jobIdx: jobIdx,
      authority: aggregatorAuthority,
    });
    const postRemoveJobAggregatorState = await aggregatorAccount.loadData();
    const jobIdx1 = postRemoveJobAggregatorState.jobPubkeysData.findIndex(
      pubkey => pubkey.equals(jobAccount.publicKey)
    );
    if (jobIdx1 !== -1) {
      throw new Error(`Failed to remove job from aggregator`);
    }
  });

  it('Creates and funds aggregator', async () => {
    const [aggregatorAccount] = await queueAccount.createFeed({
      queueAuthority: queueAuthority,
      batchSize: 1,
      minRequiredOracleResults: 1,
      minRequiredJobResults: 1,
      minUpdateDelaySeconds: 60,
      fundAmount: 2.5,
      enable: true,
      jobs: [
        { pubkey: jobAccount.publicKey },
        {
          weight: 2,
          data: OracleJob.encodeDelimited(
            OracleJob.fromObject({
              tasks: [
                {
                  valueTask: {
                    value: 1,
                  },
                },
              ],
            })
          ).finish(),
        },
      ],
    });

    const aggregator = await aggregatorAccount.loadData();

    if (aggregator.jobPubkeysSize !== 2) {
      throw new Error(`Aggregator failed to add the correct number of jobs`);
    }

    if (aggregator.jobWeights[0] !== 1 || aggregator.jobWeights[1] !== 2) {
      throw new Error(`Aggregator set the incorrect job weights`);
    }

    const [leaseAccount] = LeaseAccount.fromSeed(
      ctx.program,
      queueAccount.publicKey,
      aggregatorAccount.publicKey
    );
    const leaseBalance = await leaseAccount.getBalance();
    if (leaseBalance !== 2.5) {
      throw new Error(
        `Lease balance has incorrect funds, expected 2.5 wSOL, received ${leaseBalance}`
      );
    }

    fundedAggregator = aggregatorAccount;
  });

  it('Withdraws funds from an aggregator lease', async () => {
    if (!fundedAggregator) {
      throw new Error(`Aggregator does not exist`);
    }
    const aggregatorAccount = fundedAggregator;

    const userTokenAddress = ctx.program.mint.getAssociatedAddress(
      ctx.payer.publicKey
    );

    let initialUserTokenBalance = await ctx.program.mint.getBalance(
      userTokenAddress
    );

    if (initialUserTokenBalance === null || initialUserTokenBalance <= 0) {
      const [user, userInit] =
        await ctx.program.mint.getOrCreateWrappedUserInstructions(
          ctx.payer.publicKey,
          { fundUpTo: 0.1 }
        );

      if (userInit && userInit.ixns.length > 0) {
        await ctx.program.signAndSend(userInit);
      }

      initialUserTokenBalance =
        (await ctx.program.mint.getBalance(userTokenAddress)) ?? 0;
    }

    const [leaseAccount] = LeaseAccount.fromSeed(
      ctx.program,
      queueAccount.publicKey,
      aggregatorAccount.publicKey
    );
    const leaseBalance = await leaseAccount.getBalance();

    const expectedFinalBalance = leaseBalance - 1;

    await leaseAccount.withdraw({
      amount: 1,
      unwrap: true,
      withdrawWallet: userTokenAddress,
    });

    const finalBalance = await leaseAccount.getBalance();
    if (expectedFinalBalance !== finalBalance) {
      throw new Error(
        `Lease balance has incorrect funds, expected ${expectedFinalBalance} wSOL, received ${finalBalance}`
      );
    }

    const finalUserBalance = await ctx.program.mint.getBalance(
      userTokenAddress
    );
    if (!finalUserBalance) {
      throw new Error(`Users wrapped account was closed`);
    }

    const finalUserTokenBalance =
      (await ctx.program.mint.getBalance(userTokenAddress)) ?? 0;
    if (initialUserTokenBalance !== finalUserTokenBalance) {
      throw new Error(
        `User token balance has incorrect funds, expected ${initialUserTokenBalance} wSOL, received ${finalUserTokenBalance}`
      );
    }
  });

  it("Adds job, updates it's config, then removes it from aggregator", async () => {
    const aggregatorKeypair = Keypair.generate();
    const aggregatorAuthority = Keypair.generate();

    const [aggregatorAccount] = await AggregatorAccount.create(ctx.program, {
      queueAccount,
      queueAuthority: queueAuthority.publicKey,
      authority: aggregatorAuthority.publicKey,
      batchSize: 1,
      minRequiredOracleResults: 1,
      minRequiredJobResults: 1,
      minUpdateDelaySeconds: 60,
      keypair: aggregatorKeypair,
    });
    await aggregatorAccount.loadData();

    const oracleJob = OracleJob.fromObject({
      tasks: [{ valueTask: { value: 1 } }],
    });

    const [jobAccount] = await JobAccount.create(ctx.program, {
      data: OracleJob.encodeDelimited(oracleJob).finish(),
      name: 'Job1',
    });

    await aggregatorAccount.addJob({
      job: jobAccount,
      weight: 1,
      authority: aggregatorAuthority,
    });

    const postAddJobAggregatorState = await aggregatorAccount.loadData();
    const jobIdx = postAddJobAggregatorState.jobPubkeysData.findIndex(pubkey =>
      pubkey.equals(jobAccount.publicKey)
    );
    if (jobIdx === -1) {
      throw new Error(`Failed to add job to aggregator`);
    }

    const badSetConfigSignature = await aggregatorAccount
      .setConfigInstruction(aggregatorAuthority.publicKey, { minJobResults: 2 })
      .catch(() => undefined);
    // If badSetConfigSignature isn't undefined, a (bad) transaction was built and sent.
    if (badSetConfigSignature) {
      throw new Error(
        'Aggregator should not let minJobResults increase above numJobs'
      );
    }

    await aggregatorAccount.setConfig({
      authority: aggregatorAuthority,
      minUpdateDelaySeconds: 300,
      force: true, // Bypass validation rules.
    });
    const postUpdateAggregatorState = await aggregatorAccount.loadData();
    if (postUpdateAggregatorState.minUpdateDelaySeconds !== 300) {
      throw new Error(`Failed to setConfig on aggregator`);
    }
  });
});
