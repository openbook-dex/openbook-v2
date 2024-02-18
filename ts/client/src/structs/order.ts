import { BN } from "@coral-xyz/anchor";
import { LeafNode, Market, Side, SideUtils, U64_MAX_BN } from "..";


export class Order {

  public seqNum: BN;
  public priceLots: BN;

  constructor(
    public market: Market,
    public leafNode: LeafNode,
    public side: Side,
    public isExpired = false,
    public isOraclePegged = false
  ){
    this.seqNum = this.side === SideUtils.Bid ? U64_MAX_BN.sub(this.leafNode.key.maskn(64)) : this.leafNode.key.maskn(64);
    const priceData = this.leafNode.key.ushrn(64);
    if (this.isOraclePegged) {
      const priceOffset = priceData.sub(new BN(1).ushln(63));
      throw new Error("Not implemented yet");
      // TODO: add oracle price logic to Market
    } else {
      this.priceLots = priceData;
    }
  }

  get price(): number {
    return this.market.priceLotsToUi(this.priceLots);
  }

  get size(): number {
    return this.market.baseLotsToUi(this.leafNode.quantity);
  }

  get sizeLots(): BN {
    return this.leafNode.quantity;
  }
}