import { PublicKey } from "@solana/web3.js";
import { Market, BookSideAccount, SideUtils, Side, OpenBookV2Client, LeafNode, InnerNode, U64_MAX_BN } from "..";
import { BN } from "@coral-xyz/anchor";
import { Order } from "../structs/order";

export class BookSide {

  public clusterTime: BN;

  constructor(
    public market: Market,
    public pubkey: PublicKey,
    public account: BookSideAccount,
    public side: Side) {
    this.clusterTime = new BN(0);
  }

  public *items(): Generator<Order> {
    const fGen = this.fixedItems();
    const oPegGen = this.oraclePeggedItems();

    let fOrderRes = fGen.next();
    let oPegOrderRes = oPegGen.next();

    while (true) {
      if (fOrderRes.value && oPegOrderRes.value) {
        if (this.compareOrders(fOrderRes.value, oPegOrderRes.value)) {
          yield fOrderRes.value;
          fOrderRes = fGen.next();
        } else {
          yield oPegOrderRes.value;
          oPegOrderRes = oPegGen.next();
        }
      } else if (fOrderRes.value && !oPegOrderRes.value) {
        yield fOrderRes.value;
        fOrderRes = fGen.next();
      } else if (!fOrderRes.value && oPegOrderRes.value) {
        yield oPegOrderRes.value;
        oPegOrderRes = oPegGen.next();
      } else if (!fOrderRes.value && !oPegOrderRes.value) {
        break;
      }
    }
  }

  get rootFixed() {
    return this.account.roots[0];
  }

  get rootOraclePegged() {
    return this.account.roots[1];
  }

  public *fixedItems(): Generator<Order> {
    if (this.rootFixed.leafCount === 0) {
      return;
    }
    const stack = [this.rootFixed.maybeNode];
    const [left, right] = this.side === SideUtils.Bid ? [1, 0] : [0, 1];

    while (stack.length > 0) {
      const index = stack.pop()!;
      const node = this.account.nodes.nodes[index];
      if (node.tag === BookSide.INNER_NODE_TAG) {
        const innerNode = this.toInnerNode(node.data);
        stack.push(innerNode.children[right], innerNode.children[left]);
      } else if (node.tag === BookSide.LEAF_NODE_TAG) {
        const leafNode = this.toLeafNode(node.data);
        const expiryTimestamp = leafNode.timeInForce
          ? leafNode.timestamp.add(new BN(leafNode.timeInForce))
          : U64_MAX_BN;

        yield new Order(
          this.market,
          leafNode,
          this.side,
          this.clusterTime.gt(expiryTimestamp),
        );
      }
    }
  }

  public *oraclePeggedItems(): Generator<Order> {
    if (this.rootOraclePegged.leafCount === 0) {
      return;
    }
    const stack = [this.rootOraclePegged.maybeNode];
    const [left, right] = this.side === SideUtils.Bid ? [1, 0] : [0, 1];

    while (stack.length > 0) {
      const index = stack.pop()!;
      const node = this.account.nodes.nodes[index];
      if (node.tag === BookSide.INNER_NODE_TAG) {
        const innerNode = this.toInnerNode(node.data);
        stack.push(innerNode.children[right], innerNode.children[left]);
      } else if (node.tag === BookSide.LEAF_NODE_TAG) {
        const leafNode = this.toLeafNode(node.data);
        const expiryTimestamp = leafNode.timeInForce
          ? leafNode.timestamp.add(new BN(leafNode.timeInForce))
          : U64_MAX_BN;

        yield new Order(
          this.market,
          leafNode,
          this.side,
          this.clusterTime.gt(expiryTimestamp),
          true,
        );
      }
    }
  }

  public compareOrders(a: Order, b: Order): boolean {
    return a.priceLots.eq(b.priceLots)
      ? a.seqNum.lt(b.seqNum) // if prices are equal prefer orders in the order they are placed
      : this.side === SideUtils.Bid // else compare the actual prices
      ? a.priceLots.gt(b.priceLots)
      : b.priceLots.gt(a.priceLots);
  }

  public best(): Order | undefined {
    return this.items().next().value;
  }

  public getL2(depth: number): [number, number, BN, BN][] {
    const levels: [BN, BN][] = [];
    for (const { priceLots, sizeLots } of this.items()) {
      if (levels.length > 0 && levels[levels.length - 1][0].eq(priceLots)) {
        levels[levels.length - 1][1].iadd(sizeLots);
      } else if (levels.length === depth) {
        break;
      } else {
        levels.push([priceLots, sizeLots]);
      }
    }
    return levels.map(([priceLots, sizeLots]) => [
      this.market.priceLotsToUi(priceLots),
      this.market.baseLotsToUi(sizeLots),
      priceLots,
      sizeLots,
    ]);
  }

  private static INNER_NODE_TAG = 1;
  private static LEAF_NODE_TAG = 2;

  private toInnerNode(data: number[]): InnerNode {
    return (this.market.client.program as any)._coder.types.typeLayouts
      .get('InnerNode')
      .decode(Buffer.from([BookSide.INNER_NODE_TAG].concat(data)));
  }
  private toLeafNode(data: number[]): LeafNode {
    return (this.market.client.program as any)._coder.types.typeLayouts
        .get('LeafNode')
        .decode(Buffer.from([BookSide.LEAF_NODE_TAG].concat(data)));
  }

}
