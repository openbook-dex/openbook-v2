import 'mocha';
import assert from 'assert';

import { AggregatorAccount, SwitchboardProgram } from '../src';
import { clusterApiUrl, Connection } from '@solana/web3.js';

describe('History Tests', () => {
  let program: SwitchboardProgram;

  before(async () => {
    program = await SwitchboardProgram.load(
      'devnet',
      new Connection(process.env.SOLANA_DEVNET_RPC ?? clusterApiUrl('devnet'))
    );
  });

  /** History buffer should be returned with the oldest elements (lowest timestamps) first */
  it('Verifies a history buffer is decoded in order', async () => {
    const [aggregatorAccount, aggregator] = await AggregatorAccount.load(
      program,
      'GvDMxPzN1sCj7L26YDK2HnMRXEQmQ2aemov8YBtPS7vR'
    );

    const history = await aggregatorAccount.loadHistory();

    let lastTimestamp: number | undefined = undefined;
    for (const [n, row] of history.entries()) {
      if (lastTimestamp === undefined) {
        lastTimestamp = row.timestamp.toNumber();
        continue;
      }

      const currentTimestamp = row.timestamp.toNumber();

      assert(
        lastTimestamp < currentTimestamp,
        `Aggregator History is out of order at element ${n}, prev ${lastTimestamp}, curr ${currentTimestamp}`
      );

      lastTimestamp = currentTimestamp;
    }
  });
});
