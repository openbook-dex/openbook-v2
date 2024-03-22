import { Connection } from '@solana/web3.js';
import { BookSide, Market, OpenOrders } from '..';

export class Watcher {
  accountSubs: { [pk: string]: number } = {};

  constructor(public connection: Connection) {}

  addMarket(market: Market, includeBook = true): this {
    const { client, asks, bids, pubkey } = market;

    this.accountSubs[pubkey.toBase58()] = this.connection.onAccountChange(
      pubkey,
      (ai) => {
        market.account = client.program.coder.accounts.decode(
          'market',
          ai.data,
        );
      },
    );

    if (includeBook && asks) {
      this.addBookSide(asks);
    }
    if (includeBook && bids) {
      this.addBookSide(bids);
    }
    return this;
  }

  addBookSide(bookSide: BookSide): this {
    const { market, pubkey } = bookSide;
    this.accountSubs[pubkey.toBase58()] = this.connection.onAccountChange(
      pubkey,
      (ai) => {
        bookSide.account = market.client.program.coder.accounts.decode(
          'bookSide',
          ai.data,
        );
      },
    );
    return this;
  }

  addOpenOrders(openOrders: OpenOrders): this {
    const { market, pubkey } = openOrders;
    this.accountSubs[pubkey.toBase58()] = this.connection.onAccountChange(
      pubkey,
      (ai) => {
        openOrders.account = market.client.program.coder.accounts.decode(
          'OpenOrders',
          ai.data,
        );
      },
    );
    return this;
  }

  clear(): this {
    for (const [_pk, sub] of Object.entries(this.accountSubs)) {
      this.connection.removeAccountChangeListener(sub);
    }
    return this;
  }
}
