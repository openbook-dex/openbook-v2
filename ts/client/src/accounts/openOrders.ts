import {
  Keypair,
  PublicKey,
  Signer,
  TransactionInstruction,
} from '@solana/web3.js';
import { BN } from '@coral-xyz/anchor';
import { createAssociatedTokenAccountIdempotentInstruction } from '@solana/spl-token';
import {
  FillEvent,
  OpenBookV2Client,
  OpenOrdersAccount,
  OutEvent,
  PlaceOrderType,
  SelfTradeBehavior as SelfTradeBehaviorType,
  Side,
  nameToString,
  I64_MAX_BN,
  PlaceOrderTypeUtils,
  SelfTradeBehaviorUtils,
  SideUtils,
  getAssociatedTokenAddress,
  EventType,
  Order,
  OpenOrdersIndexer,
  Market,
  BookSide,
} from '..';

export interface OrderToPlace {
  side: Side;
  price: number;
  size: number;
  quoteLimit?: number;
  clientOrderId?: number;
  orderType?: PlaceOrderType;
  expiryTimestamp?: number;
  selfTradeBehavior?: SelfTradeBehaviorType;
  matchLoopLimit?: number;
}

export class OpenOrders {
  public delegate: Keypair | undefined;

  constructor(
    public pubkey: PublicKey,
    public account: OpenOrdersAccount,
    public market: Market,
  ) {}

  /// high-level API

  public static async load(
    pubkey: PublicKey,
    market?: Market,
    client?: OpenBookV2Client,
  ): Promise<OpenOrders> {
    client ??= market?.client;
    if (!client) throw new Error('provide either market or client');

    const account = await client.program.account.openOrdersAccount.fetch(
      pubkey,
    );

    if (!market) {
      market = await Market.load(client, account.market);
    }

    return new OpenOrders(pubkey, account, market);
  }

  /**
   * Try loading the OpenOrders account associated with a Market
   * @param market
   * @param owner optional if configured already on the Market's client
   * @param indexer optional, pass in to speed up fetch
   * @returns null if the user does not have an OpenOrders account for this market
   */
  public static async loadNullableForMarketAndOwner(
    market: Market,
    owner?: PublicKey,
    indexer?: OpenOrdersIndexer | null,
  ): Promise<OpenOrders | null> {
    indexer ??= await OpenOrdersIndexer.loadNullable(market.client, owner);
    if (!indexer) return null;
    const ooPks = indexer.account.addresses;
    const ooAccs =
      await market.client.program.account.openOrdersAccount.fetchMultiple(
        ooPks,
      );
    const ooIndex = ooAccs.findIndex((o) => o?.market.equals(market.pubkey));
    if (ooIndex == -1) return null;
    const ooPk = ooPks[ooIndex];
    const ooAcc = ooAccs[ooIndex];
    // note: ooPk & ooAcc most certainly will always be defined here, due to the index check
    return ooPk && ooAcc && new OpenOrders(ooPk, ooAcc, market);
  }

  public async reload(): Promise<this> {
    // Need to reload orderbooks because not all information about orders, like
    // size, is stored on the open orders account. Do all fetches together to
    // ensure they are synced to the same slot.
    const [bidsAi, asksAi, ooAi] =
      await this.market.client.connection.getMultipleAccountsInfo([
        this.market.account.bids,
        this.market.account.asks,
        this.pubkey,
      ]);
    this.market.bids = new BookSide(
      this.market,
      this.market.account.bids,
      BookSide.decodeAccountfromBuffer(bidsAi!.data),
      SideUtils.Bid,
    );
    this.market.asks = new BookSide(
      this.market,
      this.market.account.asks,
      BookSide.decodeAccountfromBuffer(asksAi!.data),
      SideUtils.Ask,
    );
    this.account = this.market.client.program.coder.accounts.decode(
      'openOrdersAccount',
      ooAi!.data,
    );

    return this;
  }

  public getBalanceNative(): [BN, BN] {
    const {
      asksBaseLots,
      bidsQuoteLots,
      baseFreeNative,
      quoteFreeNative,
      lockedMakerFees,
    } = this.account.position;
    const { baseLotSize, quoteLotSize } = this.market.account;

    // TODO count in lots to save compute
    const base = asksBaseLots.mul(baseLotSize).iadd(baseFreeNative);
    const quote = bidsQuoteLots
      .mul(quoteLotSize)
      .iadd(quoteFreeNative)
      .iadd(lockedMakerFees);

    if (this.market.eventHeap) {
      for (const event of this.market.eventHeap.parsedEvents()) {
        switch (event.eventType) {
          case EventType.Fill: {
            const { maker, quantity, price, takerSide } = event as FillEvent;
            if (maker.equals(this.account.owner)) {
              const baseNative = quantity.mul(baseLotSize);
              const quoteNative = quantity.mul(price).imul(quoteLotSize);
              const quoteFeesNative = this.market.makerFeeFloor(quoteNative);
              if (takerSide === 1) {
                // buy
                base.iadd(baseNative);
                quote.isub(quoteNative.iadd(quoteFeesNative));
              } else {
                // sell
                base.isub(baseNative);
                quote.iadd(quoteNative.isub(quoteFeesNative));
              }
            }
            continue;
          }
          case EventType.Out: {
            // out events don't change balances
            continue;
          }
        }
      }
    }

    return [base, quote];
  }

  public setDelegate(delegate: Keypair): this {
    this.delegate = delegate;
    return this;
  }

  public async placeOrder(order: OrderToPlace): Promise<string> {
    // derive token account
    const mint = order.side.bid
      ? this.market.account.quoteMint
      : this.market.account.baseMint;
    const userTokenAccount = await getAssociatedTokenAddress(
      mint,
      this.market.client.walletPk,
    );

    // TODO: derive wrap sol instruction

    const remainingAccounts = new Set<string>();
    const bookSide = order.side.bid ? this.market.asks : this.market.bids;
    if (
      bookSide &&
      !order.orderType?.postOnly &&
      !order.orderType?.postOnlySlide
    ) {
      for (const order of bookSide.items()) {
        remainingAccounts.add(order.leafNode.owner.toBase58());
        if (remainingAccounts.size >= 3) break;
      }
    }

    const [placeIx] = await this.placeOrderIx(
      order,
      userTokenAccount,
      [...remainingAccounts].map((a) => new PublicKey(a)),
    );

    const additionalSigners = this.delegate ? [this.delegate] : [];

    return this.market.client.sendAndConfirmTransaction([placeIx], {
      additionalSigners,
    });
  }

  public async cancelOrder(
    order: Order | { clientOrderId: number },
  ): Promise<string> {
    const ixs: TransactionInstruction[] = [];
    if ('clientOrderId' in order) {
      const id = new BN(order.clientOrderId);
      const [ix] = await this.cancelOrderByClientIdIx(id);
      ixs.push(ix);
    } else {
      const id = order.leafNode.key;
      const [ix] = await this.cancelOrderByIdIx(id);
      ixs.push(ix);
    }

    const additionalSigners = this.delegate ? [this.delegate] : [];

    return this.market.client.sendAndConfirmTransaction(ixs, {
      additionalSigners,
    });
  }

  public async cancelAllOrders(side: Side | null): Promise<string> {
    const [cancelIx] = await this.cancelAllOrdersIx(side);

    const { baseMint, quoteMint } = this.market.account;
    const owner = this.market.client.walletPk;
    const payer = this.delegate?.publicKey ?? owner;

    const ataIxs: TransactionInstruction[] = [];
    const baseATA = await getAssociatedTokenAddress(baseMint, owner);
    ataIxs.push(
      createAssociatedTokenAccountIdempotentInstruction(
        payer,
        baseATA,
        owner,
        baseMint,
      ),
    );

    const quoteATA = await getAssociatedTokenAddress(quoteMint, owner);
    ataIxs.push(
      createAssociatedTokenAccountIdempotentInstruction(
        payer,
        quoteATA,
        owner,
        quoteMint,
      ),
    );

    const referrer = this.market.client.referrerWallet;
    let referrerATA: PublicKey | null = null;
    if (referrer) {
      referrerATA = await getAssociatedTokenAddress(quoteMint, referrer);
      ataIxs.push(
        createAssociatedTokenAccountIdempotentInstruction(
          payer,
          referrerATA,
          referrer,
          quoteMint,
        ),
      );
    }

    const [settleIx] = await this.settleFundsIx(
      baseATA,
      quoteATA,
      referrerATA,
      payer,
    );

    // TODO: derive unwrap sol instruction

    const additionalSigners = this.delegate ? [this.delegate] : [];

    return this.market.client.sendAndConfirmTransaction(
      [cancelIx, ...ataIxs, settleIx],
      { additionalSigners },
    );
  }

  public *items(): Generator<Order> {
    const { bids, asks } = this.market;
    if (!bids || !asks)
      throw new Error('requires OrderBook of Market to be loaded');

    for (const slot of this.account.openOrders) {
      if (slot.isFree) continue;

      let gen;
      switch (slot.sideAndTree) {
        case 0:
          gen = bids.fixedItems();
          break;
        case 1:
          gen = asks.fixedItems();
          break;
        case 2:
          gen = bids.oraclePeggedItems();
          break;
        case 3:
          gen = asks.oraclePeggedItems();
          break;
      }

      inner: for (const order of gen as Generator<Order>) {
        if (order.leafNode.key.eq(slot.id)) {
          yield order;
          break inner;
        }
      }
    }
  }

  public toPrettyString(): string {
    const oo = this.account;
    let debug = `OO: ${this.pubkey.toBase58()}\n`;
    debug += ` owner: ${oo.owner.toBase58()}\n`;
    debug += ` market: ${oo.market.toBase58()} (${nameToString(
      this.market.account.name,
    )})\n`;
    if (!oo.delegate.key.equals(PublicKey.default))
      debug += ` delegate: ${oo.delegate.key.toBase58()}\n`;

    debug += ` accountNum: ${oo.accountNum}\n`;
    debug += ` version: ${oo.version}\n`;
    debug += ` bidsBaseLots: ${oo.position.bidsBaseLots.toString()}\n`;
    debug += ` bidsQuoteLots: ${oo.position.bidsQuoteLots.toString()}\n`;
    debug += ` asksBaseLots: ${oo.position.asksBaseLots.toString()}\n`;
    debug += ` baseFreeNative: ${oo.position.baseFreeNative.toString()}\n`;
    debug += ` quoteFreeNative: ${oo.position.quoteFreeNative.toString()}\n`;
    debug += ` lockedMakerFees: ${oo.position.lockedMakerFees.toString()}\n`;
    debug += ` referrerRebatesAvailable: ${oo.position.referrerRebatesAvailable.toString()}\n`;
    debug += ` penaltyHeapCount: ${oo.position.penaltyHeapCount.toString()}\n`;
    debug += ` makerVolume: ${oo.position.makerVolume.toString()}\n`;
    debug += ` takerVolume: ${oo.position.takerVolume.toString()}\n`;

    debug += ` orders:\n`;
    for (const order of this.items()) {
      debug += `  ${order.toPrettyString()}\n`;
    }

    if (this.market.eventHeap) {
      debug += ` events:\n`;
      for (const event of this.market.eventHeap.parsedEvents()) {
        switch (event.eventType) {
          case EventType.Fill: {
            const { maker, quantity, price, takerSide } = event as FillEvent;
            if (maker.equals(this.pubkey)) {
              const fillBase = this.market.baseLotsToUi(quantity);
              const fillPrice = this.market.priceLotsToUi(price);
              debug += `  fill side=${
                takerSide === 1 ? 'Bid' : 'Ask'
              } qty=${fillBase} price=${fillPrice}\n`;
            }
            continue;
          }
          case EventType.Out: {
            const { owner } = event as OutEvent;
            if (owner.equals(this.pubkey))
              debug += `  out ${JSON.stringify(event)}\n`;
            continue;
          }
        }
      }

      debug += ` balance:\n`;
      const [base, quote] = this.getBalanceNative();
      debug += `  base: ${this.market.baseNativeToUi(base)}\n`;
      debug += `  quote: ${this.market.quoteNativeToUi(quote)}\n`;
    }

    return debug;
  }

  getBaseBalanceNative(): BN {
    return this.account.position.asksBaseLots
      .mul(this.market.account.baseLotSize)
      .iadd(this.account.position.baseFreeNative);
  }

  getQuoteBalanceNative(): BN {
    return this.account.position.bidsQuoteLots
      .mul(this.market.account.quoteLotSize)
      .iadd(this.account.position.quoteFreeNative)
      .iadd(this.account.position.lockedMakerFees);
  }

  getBaseBalanceUi(): number {
    return (
      Number(this.getBaseBalanceNative().toString()) /
      10 ** this.market.account.baseDecimals
    );
  }

  getQuoteBalanceUi(): number {
    return (
      Number(this.getQuoteBalanceNative().toString()) /
      10 ** this.market.account.quoteDecimals
    );
  }

  /// low-level API

  public async placeOrderIx(
    order: OrderToPlace,
    userTokenAccount: PublicKey,
    remainingAccounts: PublicKey[] = [],
  ): Promise<[TransactionInstruction, Signer[]]> {
    const priceLots = this.market.priceUiToLots(order.price);
    const maxBaseLots = this.market.baseUiToLots(order.size);
    const maxQuoteLotsIncludingFees = order.quoteLimit
      ? new BN(order.quoteLimit)
      : I64_MAX_BN;
    const clientOrderId = new BN(order.clientOrderId || Date.now());
    const orderType = order.orderType ?? PlaceOrderTypeUtils.Limit;
    const expiryTimestamp = new BN(order.expiryTimestamp ?? 0);
    const selfTradeBehavior =
      order.selfTradeBehavior ?? SelfTradeBehaviorUtils.DecrementTake;
    const limit = order.matchLoopLimit ?? 16;

    const args = {
      side: order.side,
      priceLots,
      maxBaseLots,
      maxQuoteLotsIncludingFees,
      clientOrderId,
      orderType,
      expiryTimestamp,
      selfTradeBehavior,
      limit,
    };

    return await this.market.client.placeOrderIx(
      this.pubkey,
      this.market.pubkey,
      this.market.account,
      userTokenAccount,
      args,
      remainingAccounts,
      this.delegate,
    );
  }

  public async cancelAllOrdersIx(
    side: Side | null,
  ): Promise<[TransactionInstruction, Signer[]]> {
    return this.market.client.cancelAllOrdersIx(
      this.pubkey,
      this.account,
      this.market.account,
      24,
      side,
      this.delegate,
    );
  }

  public async cancelOrderByIdIx(id: BN) {
    return this.market.client.cancelOrderByIdIx(
      this.pubkey,
      this.account,
      this.market.account,
      id,
      this.delegate,
    );
  }

  public async cancelOrderByClientIdIx(id: BN) {
    return this.market.client.cancelOrderByClientIdIx(
      this.pubkey,
      this.account,
      this.market.account,
      id,
      this.delegate,
    );
  }

  public async settleFundsIx(
    userBaseAccount: PublicKey,
    userQuoteAccount: PublicKey,
    referrerAccount: PublicKey | null,
    penaltyPayer: PublicKey,
  ): Promise<[TransactionInstruction, Signer[]]> {
    return this.market.client.settleFundsIx(
      this.pubkey,
      this.account,
      this.market.pubkey,
      this.market.account,
      userBaseAccount,
      userQuoteAccount,
      referrerAccount,
      penaltyPayer,
      this.delegate,
    );
  }
}
