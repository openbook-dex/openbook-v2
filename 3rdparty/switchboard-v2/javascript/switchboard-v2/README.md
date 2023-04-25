# Switchboard-v2 API module

<!--- https://badgen.net/npm/v/@switchboard-xyz/switchboardv2-cli --->

<!---  [![GitHub last commit](https://img.shields.io/github/last-commit/switchboard-xyz/switchboardv2-cli)](https://github.com/switchboard-xyz/switchboardv2-cli/commit/) --->

[![GitHub](https://img.shields.io/badge/--181717?logo=github&logoColor=ffffff)](https://github.com/switchboard-xyz/sbv2-solana/tree/main/libraries/ts)&nbsp;&nbsp;
[![npm](https://img.shields.io/npm/v/@switchboard-xyz/switchboard-v2)](https://www.npmjs.com/package/@switchboard-xyz/switchboard-v2)&nbsp;&nbsp;
[![twitter](https://badgen.net/twitter/follow/switchboardxyz)](https://twitter.com/switchboardxyz)&nbsp;&nbsp;

A library of utility functions to interact with the Switchboardv2 program

## Install

```
npm i @switchboard-xyz/switchboard-v2
```

## Creating Feeds

```ts
import * as anchor from "@project-serum/anchor";
import { clusterApiUrl, Connection, Keypair, PublicKey } from "@solana/web3.js";
import {
  AggregatorAccount,
  OracleQueueAccount,
  loadSwitchboardProgram,
} from "@switchboard-xyz/switchboard-v2";

const payerKeypair = Keypair.fromSecretKey(
  JSON.parse(fs.readFileSync("../keypair-path.json", "utf-8"))
);
const program = await loadSwitchboardProgram(
  "devnet",
  new Connection(clusterApiUrl("devnet")),
  payerKeypair
);
const queueAccount = new OracleQueueAccount({
  program: program,
  // devnet permissionless queue
  publicKey: new PublicKey("F8ce7MsckeZAbAGmxjJNetxYXQa9mKr9nnrC3qKubyYy"),
});

const aggregatorAccount = await AggregatorAccount.create(program, {
  name: Buffer.from("FeedName"),
  batchSize: 6,
  minRequiredJobResults: 1,
  minRequiredOracleResults: 1,
  minUpdateDelaySeconds: 30,
  queueAccount,
});
```

### Updating Feeds

```ts
import * as anchor from "@project-serum/anchor";
import {
  AggregatorAccount,
  OracleQueueAccount,
} from "@switchboard-xyz/switchboard-v2";

const program: anchor.Program;
const queueAccount: OracleQueueAccount;

await aggregatorAccount.openRound({
  oracleQueueAccount: queueAccount,
  payoutWallet: tokenAccount,
});
```

### Reading Feeds

```ts
import { AggregatorAccount } from "@switchboard-xyz/switchboard-v2";
import { Big } from "big.js";

const aggregatorAccount: AggregatorAccount;
const result: Big = await aggregatorAccount.getLatestValue();

console.log(result.toNumber());
```
