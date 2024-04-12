import { PublicKey } from '@solana/web3.js';
import { AnyEvent, EventHeapAccount, FillEvent, Market, OutEvent } from '..';

export enum EventType {
  Fill = 0,
  Out = 1,
}

export class EventHeap {
  constructor(
    public pubkey: PublicKey,
    public account: EventHeapAccount,
    public market: Market,
  ) {}

  public *rawEvents(): Generator<AnyEvent> {
    let currentIndex = this.account.header.usedHead;
    for (let i = 0; i < this.account.header.count; ++i) {
      const { event, next } = this.account.nodes[currentIndex];
      yield event;
      currentIndex = next;
    }
  }

  public *parsedEvents(): Generator<FillEvent | OutEvent> {
    const { decode } = this.market.client.program.coder.types;
    for (const event of this.rawEvents()) {
      // TODO find out how not to re-allocate
      const buffer = Buffer.from([event.eventType].concat(event.padding));
      switch (event.eventType) {
        case EventType.Fill: {
          yield decode('FillEvent', buffer);
          continue;
        }
        case EventType.Out: {
          yield decode('OutEvent', buffer);
          continue;
        }
      }
    }
  }
}
