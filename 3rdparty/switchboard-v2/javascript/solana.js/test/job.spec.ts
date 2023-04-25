import 'mocha';
import assert from 'assert';

import { setupTest, TestContext } from './utilts';
import { OracleJob } from '@switchboard-xyz/common';
import { JobAccount } from '../src';

describe('Job Tests', () => {
  let ctx: TestContext;

  before(async () => {
    ctx = await setupTest();
  });

  it('Creates a big job', async () => {
    const tasks: Array<OracleJob.Task> = Array(2000).fill(
      OracleJob.ValueTask.fromObject({ value: 1 })
    );
    const oracleJob = OracleJob.fromObject({
      tasks,
    });
    const data = OracleJob.encodeDelimited(oracleJob).finish();

    const [jobAccount, jobInit] = JobAccount.createInstructions(
      ctx.program,
      ctx.program.walletPubkey,
      {
        data: data,
      }
    );

    await ctx.program.signAndSendAll(jobInit);

    const job = await jobAccount.loadData();

    assert(job.isInitializing === 0);
  });

  it('Fails creating a job over 6400 bytes', async () => {
    const tasks: Array<OracleJob.Task> = Array(3200).fill(
      OracleJob.ValueTask.fromObject({ value: 1 })
    );
    const oracleJob = OracleJob.fromObject({
      tasks,
    });
    const data = OracleJob.encodeDelimited(oracleJob).finish(); // 6402 bytes

    await assert.rejects(async () => {
      await JobAccount.create(ctx.program, {
        data: data,
      });
    }, new RegExp(/Switchboard jobs need to be less than 6400 bytes/));
  });
});
