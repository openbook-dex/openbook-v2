import { PublicKey } from '@solana/web3.js';
import {
  Market,
  OPENBOOK_PROGRAM_ID,
  findAccountsByMints,
  findAllMarkets,
  Watcher,
  sleep,
  BookSide,
} from '..';
import { initReadOnlyOpenbookClient } from './util';

async function testFindAccountsByMints(): Promise<void> {
  const client = initReadOnlyOpenbookClient();
  const accounts = await findAccountsByMints(
    client.connection,
    new PublicKey('So11111111111111111111111111111111111111112'),
    new PublicKey('EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v'),
    OPENBOOK_PROGRAM_ID,
  );
  console.log(accounts.map((a) => a.publicKey.toBase58()));
}

async function testFindAllMarkets(): Promise<void> {
  const client = initReadOnlyOpenbookClient();
  const markets = await findAllMarkets(
    client.connection,
    OPENBOOK_PROGRAM_ID,
    client.provider,
  );
  console.log('markets', markets);
}

async function testDecodeMarket(): Promise<void> {
  const client = initReadOnlyOpenbookClient();
  const marketPk = new PublicKey(
    'BU3EaRVo9WN44muCBy3mwkCQ4uYQWiuqay1whEmeSXK3',
  );
  const market = await Market.load(client, marketPk);
  await market.loadOrderBook();
  await market.loadEventHeap();

  console.log(market.toPrettyString());
}

async function benchDecodeMarket(): Promise<void> {
  const client = initReadOnlyOpenbookClient();
  const marketPk = new PublicKey(
    'CFSMrBssNG8Ud1edW59jNLnq2cwrQ9uY5cM3wXmqRJj3',
  );
  const market = await Market.load(client, marketPk);
  await market.loadOrderBook();
  await market.loadEventHeap();

  const bookSideAccount = await client.connection.getAccountInfo(
    market.bids!.pubkey,
  );

  const start = new Date();
  for (let i = 0; i < 10000; ++i) {
    const side = BookSide.decodeAccountfromBuffer(bookSideAccount!.data);
    market.bids!.account = side;
    market.bids!.getL2(16);
  }
  const end = new Date();
  console.log({ start, end, duration: end.valueOf() - start.valueOf() });
  console.log();
}

async function testDecodeMultiple(): Promise<void> {
  const client = initReadOnlyOpenbookClient();
  const markets = [
    new PublicKey('BU3EaRVo9WN44muCBy3mwkCQ4uYQWiuqay1whEmeSXK3'),
    new PublicKey('D8BPZXCYvVBkXR5NAoDnuzjFGuF2kFKWyfEUtZbmjRg7'),
  ];
  for (const marketPk of markets) {
    const market = await Market.load(client, marketPk);
    await market.loadOrderBook();

    const mktTag = marketPk.toString().substring(0, 6);

    console.log(mktTag, market.bids?.getL2(300));
    console.log(mktTag, market.asks?.getL2(300));
  }
}

async function testWatchMarket(): Promise<void> {
  const client = initReadOnlyOpenbookClient();
  const marketPk = new PublicKey(
    'CFSMrBssNG8Ud1edW59jNLnq2cwrQ9uY5cM3wXmqRJj3',
  );
  const market = await Market.load(client, marketPk);
  await market.loadOrderBook();

  console.log('bids before sub', market.bids?.getL2(2));

  const w = new Watcher(client.connection);
  w.addMarket(market);

  await sleep(5_000);

  console.log('bids after sub', market.bids?.getL2(2));
}

async function testMarketLots(): Promise<void> {
  const client = initReadOnlyOpenbookClient();
  const marketPk1 = new PublicKey(
    'Hojg6SoyQAjXRBU4HtR48RB5YVfNzu2vwcLMK6xXPSJS',
  );
  const market1 = await Market.load(client, marketPk1);
  const tick1 = market1.tickSize.toNumber();
  if ('1' !== market1.priceUiToLots(tick1).toString()) {
    throw new Error('price lot calculation rounds wrongly');
  }
  if ('0' !== market1.priceUiToLots(0.9 * tick1).toString()) {
    throw new Error('price lot calculation rounds wrongly');
  }
  if ('1' !== market1.priceUiToLots(1.9 * tick1).toString()) {
    throw new Error('price lot calculation rounds wrongly');
  }
  if ('10000000000' !== market1.priceUiToLots(1).toString()) {
    throw new Error('price lot calculation rounds wrongly');
  }

  const marketPk2 = new PublicKey(
    'DBSZ24hqXS5o8djunrTzBsJUb1P8ZvBs1nng5rmZKsJt',
  );
  const market2 = await Market.load(client, marketPk2);
  const tick2 = market2.tickSize.toNumber();
  if ('1' !== market2.priceUiToLots(tick2).toString()) {
    throw new Error('price lot calculation rounds wrongly');
  }
  if ('0' !== market2.priceUiToLots(0.9 * tick2).toString()) {
    throw new Error('price lot calculation rounds wrongly');
  }
  if ('1' !== market2.priceUiToLots(1.9 * tick2).toString()) {
    throw new Error('price lot calculation rounds wrongly');
  }
  if ('10000000000000' !== market2.priceUiToLots(1).toString()) {
    throw new Error('price lot calculation rounds wrongly');
  }
}

// void testFindAccountsByMints();
// void testFindAllMarkets();
// void testDecodeMarket();
// void testWatchMarket();
// void testMarketLots();
void benchDecodeMarket();
// void testDecodeMultiple();
