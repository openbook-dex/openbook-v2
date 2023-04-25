import 'mocha';
import assert from 'assert';

import * as sbv2 from '../src';
import { setupTest, TestContext } from './utilts';
import { Keypair } from '@solana/web3.js';
import { CrankAccount, QueueAccount, types } from '../src';

describe('Crank Tests', () => {
  let ctx: TestContext;

  const queueAuthority = Keypair.generate();

  let queueAccount: QueueAccount;
  let queue: types.OracleQueueAccountData;

  let crankAccount: CrankAccount;

  before(async () => {
    ctx = await setupTest();

    [queueAccount] = await sbv2.QueueAccount.create(ctx.program, {
      name: 'q1',
      metadata: '',
      queueSize: 2,
      reward: 0.0025,
      minStake: 0,
      oracleTimeout: 60,
      slashingEnabled: false,
      unpermissionedFeeds: false,
      unpermissionedVrf: true,
      enableBufferRelayers: false,
      authority: queueAuthority.publicKey,
    });
    queue = await queueAccount.loadData();
    assert(
      queue.authority.equals(queueAuthority.publicKey),
      'Incorrect queue authority'
    );
  });

  it('Creates a Crank', async () => {
    [crankAccount] = await queueAccount.createCrank({
      name: 'Crank #1',
      maxRows: 10,
    });
    const crank = await crankAccount.loadData();
    const crankRows = await crankAccount.loadCrank();
    assert(
      crankRows.length === 0,
      `Crank should be empty but found ${crankRows.length} rows`
    );
  });
});
