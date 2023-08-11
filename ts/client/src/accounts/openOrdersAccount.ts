import { type Keypair, type PublicKey } from '@solana/web3.js';
import { type Market } from './market';

export class OpenOrdersAccount {
  publicKey: PublicKey;
  owner: PublicKey;
  name: string;
  market: Market;
  delegate: PublicKey;
  ownerOrDelegateKeypair?: Keypair;

  constructor(
    publicKey: PublicKey,
    owner: PublicKey,
    name: string,
    delegate: PublicKey,
    market: Market,
    ownerOrDelegateKeypair?: Keypair,
  ) {
    this.publicKey = publicKey;
    this.owner = owner;
    this.name = name;
    this.delegate = delegate;
    this.market = market;
    this.ownerOrDelegateKeypair = ownerOrDelegateKeypair;
  }
}
