import { BN } from '@coral-xyz/anchor';
import { type Keypair, type PublicKey } from '@solana/web3.js';
import { MarketAccount, OracleConfig } from './client';

export class Market {
  static from(
    publicKey: PublicKey,
    obj: MarketAccount,
  ): Market {
    return new Market(
      publicKey,
      obj.bump,
      obj.baseDecimals,
      obj.quoteDecimals,
      obj.padding1,
      obj.marketAuthority,
      obj.timeExpiry,
      obj.collectFeeAdmin,
      // obj.openOrdersAdmin,
      // obj.consumeEventsAdmin,
      // obj.closeMarketAdmin,
      // obj.name,
      // obj.bids,
      // obj.asks,
      // obj.eventQueue,
      // obj.oracleA,
      // obj.oracleB,
      // obj.oracleConfig,
    );
  }

  constructor(
    public pubkey: PublicKey,
    public bump: number,
    public baseDecimals: number,
    public quoteDecimals: number,
    public padding1: any,
    public marketAuthority: PublicKey,
    public timeExpiry: BN,
    public collectFeeAdmin: PublicKey,
    // public openOrdersAdmin: PublicKey | null,
    // public consumeEventsAdmin: PublicKey | null,
    // public closeMarketAdmin: PublicKey | null,
    // public name: any,
    // public bids: PublicKey,
    // public asks: PublicKey,
    // public eventQueue: PublicKey,
    // public oracleA: PublicKey | null,
    // public oracleB: PublicKey | null,
    // public oracleConfig: OracleConfig,
  ) {}
}

export class OpenOrders {
  constructor(
    public publicKey: PublicKey,
    public owner: PublicKey,
    public name: string,
    public delegate: PublicKey,
    public market: Market,
    public ownerOrDelegateKeypair?: Keypair,
  ) {}
}
