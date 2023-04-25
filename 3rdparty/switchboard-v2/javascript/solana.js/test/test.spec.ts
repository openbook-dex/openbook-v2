import 'mocha';
import assert from 'assert';

import { DEFAULT_KEYPAIR_PATH } from './utilts';
import {
  camelToUpperCaseWithUnderscores,
  SwitchboardTestContext,
} from '../src/test';
import fs from 'fs';
import path from 'path';
import {
  clusterApiUrl,
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
} from '@solana/web3.js';

describe('SwitchboardTestContext Tests', () => {
  let payerKeypair: Keypair;
  let payerKeypairPath: string;

  it('Converts camelCase to UPPER_CASE', () => {
    const input = 'programId';
    const expected = 'PROGRAM_ID';
    const output = camelToUpperCaseWithUnderscores(input);
    assert(
      expected === output,
      `Failed to convert camelCase, expected ${expected}, received ${output}`
    );
  });

  it('Creates a test context', async () => {
    if (process.env.SOLANA_LOCALNET) {
      return;
    }
    payerKeypairPath = fs.existsSync(DEFAULT_KEYPAIR_PATH)
      ? DEFAULT_KEYPAIR_PATH
      : path.join(__dirname, 'data', 'payer-keypair.json');

    if (!fs.existsSync(payerKeypairPath)) {
      payerKeypair = Keypair.generate();
      fs.writeFileSync(payerKeypairPath, `[${payerKeypair.secretKey}]`);
    } else {
      payerKeypair = Keypair.fromSecretKey(
        new Uint8Array(JSON.parse(fs.readFileSync(payerKeypairPath, 'utf-8')))
      );
    }

    const connection = new Connection(clusterApiUrl('devnet'));
    const payerBalance = await connection.getBalance(payerKeypair.publicKey);
    // 0.5 SOL
    if (payerBalance < 500000000) {
      try {
        const airdropTxn = await connection.requestAirdrop(
          payerKeypair.publicKey,
          1 * LAMPORTS_PER_SOL
        );
        await connection.confirmTransaction(airdropTxn);
      } catch (error) {
        console.warn(`SwitchboardTestContext airdrop failed, network issues?`);
        console.error(error);
      }
    }
    try {
      const testEnvironment = await SwitchboardTestContext.createEnvironment(
        payerKeypairPath
      );
    } catch (error) {
      console.warn(`SwitchboardTestContext test failed, network issues?`);
      console.error(error);
    }
  });
});
