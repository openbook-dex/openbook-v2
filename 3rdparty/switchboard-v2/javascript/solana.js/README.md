# Switchboard Solana SDK

A library of utility functions to interact with Switchboard v2 on Solana.

**Official Documentation**: [https://docs.switchboard.xyz/solana](https://docs.switchboard.xyz/solana)

**Typedocs**: [https://docs.switchboard.xyz/api/@switchboard-xyz/solana.js](https://docs.switchboard.xyz/api/@switchboard-xyz/solana.js)

## Install

```bash
npm i --save @switchboard-xyz/solana.js
```

## Usage

Load the Switchboard Program

```ts
import { Connection } from '@solana/web3.js';
import {
  SwitchboardProgram,
  TransactionObject,
} from '@switchboard-xyz/solana.js';

const program = await SwitchboardProgram.load(
  'mainnet-beta',
  new Connection('https://api.mainnet-beta.solana.com'),
  payerKeypair /** Optional, READ-ONLY if not provided */
);
```

Create a queue

```ts
import { QueueAccount } from '@switchboard-xyz/solana.js';

const [queueAccount, txnSignature] = await QueueAccount.create(program, {
  name: 'My Queue',
  metadata: 'Top Secret',
  queueSize: 100,
  reward: 0.00001337,
  minStake: 10,
  oracleTimeout: 60,
  slashingEnabled: false,
  unpermissionedFeeds: true,
  unpermissionedVrf: true,
  enableBufferRelayers: false,
});
const queue = await queueAccount.loadData();
```

Add an oracle

```ts
import { QueueAccount } from '@switchboard-xyz/solana.js';

const queueAccount = new QueueAccount(program, queuePubkey);

const [oracleAccount, oracleInitSignature] = await queueAccount.createOracle({
  name: 'My Oracle',
  metadata: 'Oracle #1',
  stakeAmount: 10,
});
const oracle = await oracleAccount.loadData();

await oracleAccount.heartbeat();
```

Add a data feed

```ts
import { QueueAccount } from '@switchboard-xyz/solana.js';

const queueAccount = new QueueAccount(program, queuePubkey);

const [aggregatorAccount, aggregatorInitSignatures] =
  await queueAccount.createFeed({
    batchSize: 1,
    minRequiredOracleResults: 1,
    minRequiredJobResults: 1,
    minUpdateDelaySeconds: 60,
    fundAmount: 2.5, // deposit 2.5 wSOL into the leaseAccount escrow
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
```

Request a new value

```ts
import { AggregatorAccount } from '@switchboard-xyz/solana.js';

const aggregatorAccount = new AggregatorAccount(program, aggregatorPubkey);

await aggregatorAccount.openRound();
```

After the oracles respond, read the feed result

```ts
import Big from 'big.js';
import { AggregatorAccount } from '@switchboard-xyz/solana.js';

const aggregatorAccount = new AggregatorAccount(program, aggregatorPubkey);

const result: Big | null = await aggregatorAccount.fetchLatestValue();
if (result === null) {
  throw new Error('Aggregator holds no value');
}
console.log(result.toString());
```

Optionally, add a history buffer to your feed to store the last N historical samples

```ts
import {
  AggregatorAccount,
  AggregatorHistoryBuffer,
} from '@switchboard-xyz/solana.js';

const aggregatorAccount = new AggregatorAccount(program, aggregatorPubkey);
const aggregator = await aggregatorAccount.loadData();

const [historyBuffer, addHistorySignature] =
  await AggregatorHistoryBuffer.create(program, {
    aggregatorAccount,
    maxSamples: 10000,
  });
const history = await historyBuffer.loadData();
```

Setup a websocket listener to invoke a callback whenever an aggregator is updated

```ts
import Big from 'big.js';
import { AggregatorAccount } from '@switchboard-xyz/solana.js';

const aggregatorAccount = new AggregatorAccount(program, aggregatorPubkey);

const ws = aggregatorAccount.onChange(aggregator => {
  const result = AggregatorAccount.decodeLatestValue(aggregator);
  if (result !== null) {
    console.log(result.toString());
  }
});
```
