import type { PublicKey } from '@solana/web3.js';

export class Market {
  publicKey: PublicKey;
  asks: PublicKey;
  bids: PublicKey;
  eventQueue: PublicKey;
  oracleA: PublicKey;
  oracleB: PublicKey;

  constructor(
    publicKey: PublicKey,
    asks: PublicKey,
    bids: PublicKey,
    eventQueue: PublicKey,
    oracleA: PublicKey,
    oracleB: PublicKey,
  ) {
    this.publicKey = publicKey;
    this.asks = asks;
    this.bids = bids;
    this.eventQueue = eventQueue;
    this.oracleA = oracleA;
    this.oracleB = oracleB;
  }
}
