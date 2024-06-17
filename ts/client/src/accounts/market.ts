import Big from 'big.js';
import { BN } from '@coral-xyz/anchor';
import { PublicKey } from '@solana/web3.js';
import {
  toNative,
  MarketAccount,
  OpenBookV2Client,
  BookSide,
  SideUtils,
  nameToString,
  EventHeapAccount,
  EventHeap,
  EventType,
} from '..';

const FEES_SCALE_FACTOR = new BN(1_000_000);

export class Market {
  public minOrderSize: Big;
  public tickSize: Big;
  public quoteLotFactor: Big;
  public baseNativeFactor: Big;
  public quoteNativeFactor: Big;

  /**
   * use async loadBids() or loadOrderBook() to populate bids
   */
  public bids: BookSide | undefined;

  /**
   * use async loadAsks() or loadOrderBook() to populate asks
   */
  public asks: BookSide | undefined;
  public eventHeap: EventHeap | undefined;

  constructor(
    public client: OpenBookV2Client,
    public pubkey: PublicKey,
    public account: MarketAccount,
  ) {
    this.baseNativeFactor = new Big(10).pow(-account.baseDecimals);
    this.quoteNativeFactor = new Big(10).pow(-account.quoteDecimals);
    this.minOrderSize = new Big(account.baseLotSize.toString()).mul(
      this.baseNativeFactor,
    );
    this.quoteLotFactor = new Big(account.quoteLotSize.toString()).mul(
      this.quoteNativeFactor,
    );
    this.tickSize = new Big(10)
      .pow(account.baseDecimals - account.quoteDecimals)
      .mul(new Big(account.quoteLotSize.toString()))
      .div(new Big(account.baseLotSize.toString()));
  }

  public static async load(
    client: OpenBookV2Client,
    pubkey: PublicKey,
  ): Promise<Market> {
    const account = await client.program.account.market.fetch(pubkey);
    return new Market(client, pubkey, account);
  }

  public baseLotsToUi(lots: BN): number {
    return new Big(lots.toString()).mul(this.minOrderSize).toNumber();
  }
  public baseNativeToUi(native: BN): number {
    return new Big(native.toString()).mul(this.baseNativeFactor).toNumber();
  }
  public quoteLotsToUi(lots: BN): number {
    return new Big(lots.toString()).mul(this.quoteLotFactor).toNumber();
  }
  public quoteNativeToUi(native: BN): number {
    return new Big(native.toString()).mul(this.quoteNativeFactor).toNumber();
  }
  public priceLotsToUi(lots: BN): number {
    return new Big(lots.toString()).mul(this.tickSize).toNumber();
  }

  public baseUiToLots(uiAmount: number): BN {
    return toNative(uiAmount, this.account.baseDecimals).div(
      this.account.baseLotSize,
    );
  }
  public quoteUiToLots(uiAmount: number): BN {
    return toNative(uiAmount, this.account.quoteDecimals).div(
      this.account.quoteLotSize,
    );
  }
  public priceUiToLots(uiAmount: number): BN {
    return toNative(
      uiAmount * Number(this.account.baseLotSize.toString()),
      this.account.quoteDecimals,
    ).div(
      new BN(Math.pow(10, this.account.baseDecimals)).imul(
        this.account.quoteLotSize,
      ),
    );
  }

  public makerFeeFloor(quoteNative: BN): BN {
    return quoteNative.mul(this.account.makerFee).div(FEES_SCALE_FACTOR);
  }

  public async loadBids(): Promise<BookSide> {
    const bidsAi = await this.client.connection.getAccountInfo(
      this.account.bids,
    );
    this.bids = new BookSide(
      this,
      this.account.bids,
      BookSide.decodeAccountfromBuffer(bidsAi!.data),
      SideUtils.Bid,
    );
    return this.bids;
  }

  public async loadAsks(): Promise<BookSide> {
    const asksAi = await this.client.connection.getAccountInfo(
      this.account.asks,
    );
    this.asks = new BookSide(
      this,
      this.account.asks,
      BookSide.decodeAccountfromBuffer(asksAi!.data),
      SideUtils.Ask,
    );
    return this.asks;
  }

  public async loadEventHeap(): Promise<EventHeap> {
    const eventHeap = (await this.client.program.account.eventHeap.fetch(
      this.account.eventHeap,
    )) as EventHeapAccount;
    this.eventHeap = new EventHeap(this.account.eventHeap, eventHeap, this);
    return this.eventHeap;
  }

  public async loadOrderBook(): Promise<this> {
    await Promise.all([this.loadBids(), this.loadAsks()]);
    return this;
  }

  public toPrettyString(): string {
    const mkt = this.account;
    let debug = `Market: ${nameToString(mkt.name)}\n`;
    debug += ` authority: ${mkt.marketAuthority.toBase58()}\n`;
    debug += ` collectFeeAdmin: ${mkt.collectFeeAdmin.toBase58()}\n`;
    if (!mkt.openOrdersAdmin.key.equals(PublicKey.default))
      debug += ` openOrdersAdmin: ${mkt.openOrdersAdmin.key.toBase58()}\n`;
    if (!mkt.consumeEventsAdmin.key.equals(PublicKey.default))
      debug += ` consumeEventsAdmin: ${mkt.consumeEventsAdmin.key.toBase58()}\n`;
    if (!mkt.closeMarketAdmin.key.equals(PublicKey.default))
      debug += ` closeMarketAdmin: ${mkt.closeMarketAdmin.key.toBase58()}\n`;

    debug += ` baseMint: ${mkt.baseMint.toBase58()}\n`;
    debug += ` quoteMint: ${mkt.quoteMint.toBase58()}\n`;
    debug += ` marketBaseVault: ${mkt.marketBaseVault.toBase58()}\n`;
    debug += ` marketQuoteVault: ${mkt.marketQuoteVault.toBase58()}\n`;

    if (!mkt.oracleA.key.equals(PublicKey.default)) {
      debug += ` oracleConfig: { confFilter: ${
        mkt.oracleConfig.confFilter
      }, maxStalenessSlots: ${mkt.oracleConfig.maxStalenessSlots.toString()} }\n`;
      debug += ` oracleA: ${mkt.oracleA.key.toBase58()}\n`;
    }
    if (!mkt.oracleB.key.equals(PublicKey.default))
      debug += ` oracleB: ${mkt.oracleB.key.toBase58()}\n`;

    debug += ` bids: ${mkt.bids.toBase58()}\n`;
    const bb = this.bids?.best();
    if (bb) {
      debug += `  best: ${bb.price} ${
        bb.size
      } ${bb.leafNode.owner.toBase58()}\n`;
    }

    debug += ` asks: ${mkt.asks.toBase58()}\n`;
    const ba = this.asks?.best();
    if (ba) {
      debug += `  best: ${ba.price} ${
        ba.size
      } ${ba.leafNode.owner.toBase58()}\n`;
    }

    debug += ` eventHeap: ${mkt.eventHeap.toBase58()}\n`;
    if (this.eventHeap) {
      let fillEvents = 0;
      let outEvents = 0;
      for (const event of this.eventHeap.parsedEvents()) {
        switch (event.eventType) {
          case EventType.Fill: {
            fillEvents += 1;
            continue;
          }
          case EventType.Out: {
            outEvents += 1;
            continue;
          }
        }
      }

      debug += `  fillEvents: ${fillEvents}\n`;
      debug += `  outEvents: ${outEvents}\n`;
    } else {
      debug += `  loaded: false\n`;
    }

    debug += ` minOrderSize: ${this.minOrderSize}\n`;
    debug += ` tickSize: ${this.tickSize}\n`;

    return debug;
  }
}
