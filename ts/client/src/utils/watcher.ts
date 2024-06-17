import { Connection } from '@solana/web3.js';
import { BookSide, EventHeap, Market, OpenOrders } from '..';

export class Watcher {
  accountSubs: { [pk: string]: number } = {};

  constructor(public connection: Connection) {}

  addMarket(market: Market, includeBook = true, includeEvents = true): this {
    const { client, asks, bids, eventHeap, pubkey } = market;

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
    if (includeEvents && eventHeap) {
      this.addEventHeap(eventHeap);
    }
    return this;
  }

  addBookSide(bookSide: BookSide): this {
    const { pubkey } = bookSide;
    this.accountSubs[pubkey.toBase58()] = this.connection.onAccountChange(
      pubkey,
      (ai) => {
        bookSide.account = BookSide.decodeAccountfromBuffer(ai.data);
      },
    );
    return this;
  }

  addEventHeap(eventHeap: EventHeap): this {
    const { market, pubkey } = eventHeap;
    this.accountSubs[pubkey.toBase58()] = this.connection.onAccountChange(
      pubkey,
      (ai) => {
        eventHeap.account = market.client.program.coder.accounts.decode(
          'eventHeap',
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
