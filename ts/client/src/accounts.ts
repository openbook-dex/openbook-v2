import { type Keypair, type PublicKey } from '@solana/web3.js';

export class Market {
  constructor(
    public publicKey: PublicKey,
    public asks: PublicKey,
    public bids: PublicKey,
    public eventQueue: PublicKey,
    public baseVault: PublicKey,
    public quoteVault: PublicKey,
    public oracleA: PublicKey | null,
    public oracleB: PublicKey | null,
  ) {}
}

export class OpenOrdersAccount {
  constructor(
    public publicKey: PublicKey,
    public owner: PublicKey,
    public name: string,
    public delegate: PublicKey,
    public market: Market,
    public ownerOrDelegateKeypair?: Keypair,
  ) {}
}
