import { PublicKey } from '@solana/web3.js';
import {
  Market,
  OPENBOOK_PROGRAM_ID,
  findAccountsByMints,
  findAllMarkets,
  initReadOnlyOpenbookClient,
  Watcher,
  sleep,
} from '..';

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

// void testFindAccountsByMints();
// void testFindAllMarkets();
void testDecodeMarket();
// void testWatchMarket();
