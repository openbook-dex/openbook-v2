import { Keypair, PublicKey, Signer, TransactionInstruction } from "@solana/web3.js";
import { OpenOrdersAccount, PlaceOrderType, SelfTradeBehavior as SelfTradeBehaviorType, Side } from "../client";
import { Market } from "./market";
import { PlaceOrderTypeUtils, SelfTradeBehaviorUtils, U64_MAX_BN } from "../utils/utils";
import { BN } from "@coral-xyz/anchor";

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
    public market: Market) {}

  public setDelegate(delegate: Keypair): this {
    this.delegate = delegate;
    return this;
  }

  public async placeOrderIx(
    order: OrderToPlace,
    userTokenAccount: PublicKey,
    remainingAccounts: PublicKey[] = [],
  ): Promise<[TransactionInstruction, Signer[]]> {
    const priceLots = this.market.priceUiToLots(order.price);
    const maxBaseLots = this.market.baseUiToLots(order.size);
    const maxQuoteLotsIncludingFees = order.quoteLimit ? new BN(order.quoteLimit) : U64_MAX_BN;
    const clientOrderId = new BN(order.clientOrderId || Date.now());
    const orderType = order.orderType || PlaceOrderTypeUtils.Limit;
    const expiryTimestamp = new BN(order.expiryTimestamp || -1);
    const selfTradeBehavior = order.selfTradeBehavior || SelfTradeBehaviorUtils.DecrementTake;
    const limit = order.matchLoopLimit || 16;

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
      null,
      args,
      remainingAccounts,
      this.delegate);
  }


}

