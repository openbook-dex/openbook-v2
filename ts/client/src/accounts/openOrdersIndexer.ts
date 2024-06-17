import { PublicKey } from '@solana/web3.js';
import {
  OpenBookV2Client,
  OpenOrdersIndexerAccount,
  OpenOrders,
  Market,
  BookSide,
  SideUtils,
} from '..';

export class OpenOrdersIndexer {
  constructor(
    public client: OpenBookV2Client,
    public pubkey: PublicKey,
    public account: OpenOrdersIndexerAccount,
  ) {}

  public static async load(
    client: OpenBookV2Client,
    owner?: PublicKey,
  ): Promise<OpenOrdersIndexer> {
    const pubkey = client.findOpenOrdersIndexer(owner);
    const account = await client.program.account.openOrdersIndexer.fetch(
      pubkey,
    );
    return new OpenOrdersIndexer(client, pubkey, account);
  }

  public static async loadNullable(
    client: OpenBookV2Client,
    owner?: PublicKey,
  ): Promise<OpenOrdersIndexer | null> {
    const pubkey = client.findOpenOrdersIndexer(owner);
    const account =
      await client.program.account.openOrdersIndexer.fetchNullable(pubkey);
    return account && new OpenOrdersIndexer(client, pubkey, account);
  }

  public async loadAllOpenOrders(): Promise<OpenOrders[]> {
    const ooPks = this.account.addresses;
    const oos =
      await this.client.program.account.openOrdersAccount.fetchMultiple(ooPks);
    const marketPks = oos.map((oo) => oo!.market);
    const markets = await this.client.program.account.market.fetchMultiple(
      marketPks,
    );
    const bookSidePks = markets.flatMap((m) => [m!.bids, m!.asks]);
    const bookSides = await this.client.program.account.bookSide.fetchMultiple(
      bookSidePks,
    );
    return oos.map((oo, i) => {
      const mkt = new Market(this.client, marketPks[i], markets[i]!);
      mkt.bids = new BookSide(
        mkt,
        bookSidePks[2 * i],
        bookSides[2 * i]!,
        SideUtils.Bid,
      );
      mkt.asks = new BookSide(
        mkt,
        bookSidePks[2 * i + 1],
        bookSides[2 * i + 1]!,
        SideUtils.Ask,
      );
      return new OpenOrders(ooPks[i], oo!, mkt);
    });
  }
}
