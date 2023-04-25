import * as sbv2 from '../src';
import fs from 'fs';
import os from 'os';
import path from 'path';
import { Connection, Keypair, LAMPORTS_PER_SOL } from '@solana/web3.js';
import dotenv from 'dotenv';
dotenv.config();

export const sleep = (ms: number): Promise<any> =>
  new Promise(s => setTimeout(s, ms));

export const DEFAULT_KEYPAIR_PATH = path.join(
  os.homedir(),
  '.config/solana/id.json'
);

export interface TestContext {
  cluster: 'localnet' | 'devnet';
  program: sbv2.SwitchboardProgram;
  payer: Keypair;
  toUrl: (signature: string) => string;
}

export function isLocalnet(): boolean {
  if (process.env.SOLANA_LOCALNET) {
    switch (process.env.SOLANA_LOCALNET) {
      case '1':
      case 'true':
      case 'localnet': {
        return true;
      }
    }
  }
  return false;
}

export async function setupTest(): Promise<TestContext> {
  const cluster = isLocalnet() ? 'localnet' : 'devnet';
  const payer: Keypair = fs.existsSync(DEFAULT_KEYPAIR_PATH)
    ? Keypair.fromSecretKey(
        new Uint8Array(
          JSON.parse(fs.readFileSync(DEFAULT_KEYPAIR_PATH, 'utf8'))
        )
      )
    : Keypair.generate();

  const program = await sbv2.SwitchboardProgram.load(
    cluster,
    new Connection(
      isLocalnet() ? 'http://localhost:8899' : 'https://api.devnet.solana.com',
      { commitment: 'confirmed' }
    ),
    payer,
    sbv2.SBV2_DEVNET_PID
  );

  // request airdrop if low on funds
  const payerBalance = await program.connection.getBalance(payer.publicKey);
  if (payerBalance === 0) {
    const airdropTxn = await program.connection.requestAirdrop(
      payer.publicKey,
      1 * LAMPORTS_PER_SOL
    );
    console.log(`Airdrop requested: ${airdropTxn}`);
    await program.connection.confirmTransaction(airdropTxn);
  }

  // Check if programStateAccount exists
  try {
    const programState = await program.connection.getAccountInfo(
      program.programState.publicKey
    );
    if (!programState || programState.data === null) {
      await sbv2.ProgramStateAccount.getOrCreate(program);
    }
  } catch (e) {
    console.error(e);
  }

  await program.mint.getOrCreateAssociatedUser(program.walletPubkey);

  return {
    cluster,
    program,
    payer,
    toUrl: signature =>
      isLocalnet()
        ? `https://explorer.solana.com/tx/${signature}?cluster=custom&customUrl=http%3A%2F%2Flocalhost%3A8899`
        : `https://explorer.solana.com/tx/${signature}?cluster=devnet`,
  };
}
