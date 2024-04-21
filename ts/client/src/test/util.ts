import { Connection, Keypair } from '@solana/web3.js';
import { OpenBookV2Client } from '..';
import { AnchorProvider, Wallet } from '@coral-xyz/anchor';

export function initReadOnlyOpenbookClient(): OpenBookV2Client {
  const conn = new Connection(process.env.SOL_RPC_URL!);
  const stubWallet = new Wallet(Keypair.generate());
  const provider = new AnchorProvider(conn, stubWallet, {});
  return new OpenBookV2Client(provider);
}

export function initOpenbookClient(): OpenBookV2Client {
  const conn = new Connection(process.env.SOL_RPC_URL!, 'processed');
  const wallet = new Wallet(
    Keypair.fromSecretKey(Uint8Array.from(JSON.parse(process.env.KEYPAIR!))),
  );
  const provider = new AnchorProvider(conn, wallet, {});
  return new OpenBookV2Client(provider, undefined, {
    prioritizationFee: 10_000,
  });
}
