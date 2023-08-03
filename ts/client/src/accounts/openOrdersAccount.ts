import { type PublicKey } from '@solana/web3.js';

export class OpenOrdersAccount {
  publicKey: PublicKey;
  owner: PublicKey;
  name: string;
  market: PublicKey;
  delegate: PublicKey;
  oracle: PublicKey;
  baseVault: PublicKey;
  quoteVault: PublicKey;

  constructor(
    publicKey: PublicKey,
    owner: PublicKey,
    name: string,
    delegate: PublicKey,
    oracle: PublicKey,
  ) {
    this.publicKey = publicKey;
    this.owner = owner;
    this.name = name;
    this.delegate = delegate;
    this.oracle = oracle;
  }
}
