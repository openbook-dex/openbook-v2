import 'mocha';
import assert from 'assert';

import * as sbv2 from '../src';
import { setupTest, TestContext } from './utilts';
import { Keypair } from '@solana/web3.js';
import { PermissionAccount, TransactionMissingSignerError } from '../src';

describe('Queue Tests', () => {
  let ctx: TestContext;

  before(async () => {
    ctx = await setupTest();
  });

  const queueAuthority = Keypair.generate();
  let queueAccount: sbv2.QueueAccount;

  const oracleAuthority = Keypair.generate();
  let oracleAccount: sbv2.OracleAccount;

  it('Creates a Queue', async () => {
    [queueAccount] = await sbv2.QueueAccount.create(ctx.program, {
      name: 'q1',
      metadata: '',
      queueSize: 2,
      reward: 0,
      minStake: 0,
      oracleTimeout: 60,
      slashingEnabled: false,
      unpermissionedFeeds: true,
      unpermissionedVrf: true,
      enableBufferRelayers: false,
      authority: queueAuthority.publicKey,
    });
    await queueAccount.loadData();
  });

  it('Adds an oracle to a queue', async () => {
    if (!queueAccount) {
      throw new Error('OracleQueue does not exist');
    }

    // Create a new oracle
    [oracleAccount] = await queueAccount.createOracle({
      name: 'oracle2',
      metadata: '',
      queueAuthority,
      enable: true,
      authority: oracleAuthority,
      stakeAmount: 2,
    });

    const oracle = await oracleAccount.loadData();
    const [permissionAccount] = PermissionAccount.fromSeed(
      ctx.program,
      queueAuthority.publicKey,
      queueAccount.publicKey,
      oracleAccount.publicKey
    );
    await permissionAccount.loadData();

    await oracleAccount.heartbeat({
      queueAccount,
      tokenWallet: oracle.tokenAccount,
      authority: oracleAuthority,
    });

    const oracles = await queueAccount.loadOracles();
    const idx = oracles.findIndex(o => o.equals(oracleAccount.publicKey));
    if (idx === -1) {
      throw new Error('Failed to push oracle #2 onto queue');
    }
  });

  it('Pushes a second oracle onto the queue', async () => {
    if (!queueAccount) {
      throw new Error('OracleQueue does not exist');
    }
    const oracleAuthority = Keypair.generate();

    // Create a new oracle
    const [oracleAccount] = await queueAccount.createOracle({
      name: 'oracle2',
      metadata: '',
      queueAuthority,
      enable: true,
      authority: oracleAuthority,
    });

    const oracle = await oracleAccount.loadData();

    await oracleAccount.heartbeat({
      queueAccount,
      tokenWallet: oracle.tokenAccount,
      authority: oracleAuthority,
    });

    const oracles = await queueAccount.loadOracles();

    const idx = oracles.findIndex(o => o.equals(oracleAccount.publicKey));
    if (idx === -1) {
      throw new Error('Failed to push oracle #2 onto queue');
    }
  });

  it('Fails to push oracle #3 - Queue Size Exceeded', async () => {
    if (!queueAccount) {
      throw new Error('OracleQueue does not exist');
    }
    const oracleAuthority = Keypair.generate();

    // Create a new oracle
    const [oracleAccount] = await queueAccount.createOracle({
      name: 'oracle3',
      metadata: '',
      queueAuthority,
      enable: true,
      authority: oracleAuthority,
    });

    const oracle = await oracleAccount.loadData();

    await assert.rejects(
      async () => {
        await oracleAccount.heartbeat({
          queueAccount,
          tokenWallet: oracle.tokenAccount,
          authority: oracleAuthority,
        });
      },
      new RegExp(/custom program error: 0x1771/g)
      // { code: 6001 } // QueueOperationError
    );
  });

  it('Deposits into an oracle staking wallet', async () => {
    if (!queueAccount) {
      throw new Error('OracleQueue does not exist');
    }
    if (!oracleAccount) {
      throw new Error('oracleAccount does not exist');
    }

    const STAKE_AMOUNT = 1.25;

    const oracle = await oracleAccount.loadData();

    const initialStakeBalance = await oracleAccount.getBalance(
      oracle.tokenAccount
    );

    await oracleAccount.stake({
      stakeAmount: STAKE_AMOUNT,
      tokenAccount: oracle.tokenAccount,
    });

    const finalStakeBalance = await oracleAccount.getBalance(
      oracle.tokenAccount
    );

    assert(
      finalStakeBalance === initialStakeBalance + STAKE_AMOUNT,
      `Oracle token balance mismatch, expected ${
        initialStakeBalance + STAKE_AMOUNT
      }, received ${finalStakeBalance}`
    );
  });

  it('Fails to withdraw if authority is missing', async () => {
    if (!queueAccount) {
      throw new Error('OracleQueue does not exist');
    }
    if (!oracleAccount) {
      throw new Error('oracleAccount does not exist');
    }

    await assert.rejects(async () => {
      await oracleAccount.withdraw({
        amount: 1,
      });
    }, TransactionMissingSignerError);
  });

  it('Withdraws from an oracle staking wallet', async () => {
    if (!queueAccount) {
      throw new Error('OracleQueue does not exist');
    }
    if (!oracleAccount) {
      throw new Error('oracleAccount does not exist');
    }

    const WITHDRAW_AMOUNT = 0.55;

    const oracle = await oracleAccount.loadData();

    const initialStakeBalance = await oracleAccount.getBalance(
      oracle.tokenAccount
    );

    await oracleAccount.withdraw({
      amount: WITHDRAW_AMOUNT,
      authority: oracleAuthority,
    });

    const finalStakeBalance = await oracleAccount.getBalance(
      oracle.tokenAccount
    );

    assert(
      finalStakeBalance === initialStakeBalance - WITHDRAW_AMOUNT,
      `Oracle token balance mismatch, expected ${
        initialStakeBalance - WITHDRAW_AMOUNT
      }, received ${finalStakeBalance}`
    );
  });
});
