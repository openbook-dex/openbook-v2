import { PublicKey } from '@solana/web3.js';
import { Market, OpenOrders, SideUtils } from '..';
import { OpenOrdersIndexer } from '../accounts/openOrdersIndexer';
import { initOpenbookClient, initReadOnlyOpenbookClient } from './util';

async function testLoadIndexerNonExistent(): Promise<void> {
  const client = initReadOnlyOpenbookClient();
  try {
    const indexer = await OpenOrdersIndexer.load(client);
    console.error('should not find', indexer);
    process.exit(-1);
  } catch (e) {
    console.log('expected failure');
  }
}

async function testLoadOOForMarket(): Promise<void> {
  const client = initOpenbookClient();
  const marketPk = new PublicKey(
    'CFSMrBssNG8Ud1edW59jNLnq2cwrQ9uY5cM3wXmqRJj3',
  );
  const market = await Market.load(client, marketPk);
  const [oo] = await Promise.all([
    OpenOrders.loadNullableForMarketAndOwner(market),
    market.loadOrderBook(),
    market.loadEventHeap(),
  ]);
  console.log(oo?.toPrettyString());
}

async function testLoadOOForWallet(): Promise<void> {
  const client = initOpenbookClient();
  const ooi = await OpenOrdersIndexer.loadNullable(client);
  const oos = await ooi!.loadAllOpenOrders();
  for (const oo of oos) console.log(oo?.toPrettyString());
}

async function testPlaceAndCancelOrder(): Promise<void> {
  const client = initOpenbookClient();
  const marketPk = new PublicKey(
    'CFSMrBssNG8Ud1edW59jNLnq2cwrQ9uY5cM3wXmqRJj3',
  );
  const market = await Market.load(client, marketPk);

  console.log(market.toPrettyString());
  const [oo] = await Promise.all([
    OpenOrders.loadNullableForMarketAndOwner(market),
    market.loadOrderBook(),
  ]);

  const sigPlace = await oo?.placeOrder({
    side: SideUtils.Bid,
    price: market.tickSize,
    size: market.minOrderSize,
  });

  console.log('placed order', sigPlace);

  await Promise.all([oo?.reload(), market.loadBids()]);

  console.log(oo?.toPrettyString());

  const sigCancel = await oo?.cancelOrder(oo.items().next().value);

  console.log('cancelled order', sigCancel);
}

async function testPlaceAndCancelOrderByClientId(): Promise<void> {
  const client = initOpenbookClient();
  const marketPk = new PublicKey(
    'CFSMrBssNG8Ud1edW59jNLnq2cwrQ9uY5cM3wXmqRJj3',
  );
  const market = await Market.load(client, marketPk);

  console.log(market.toPrettyString());
  const [oo] = await Promise.all([
    OpenOrders.loadNullableForMarketAndOwner(market),
    market.loadOrderBook(),
  ]);

  const sigPlace = await oo?.placeOrder({
    side: SideUtils.Bid,
    price: market.tickSize,
    size: market.minOrderSize,
    clientOrderId: 9999,
  });

  console.log('placed order', sigPlace);

  const sigCancel = await oo?.cancelOrder({ clientOrderId: 9999 });

  console.log('cancelled order', sigCancel);
}

// testLoadIndexerNonExistent();
// void testLoadOOForMarket();
void testLoadOOForWallet();
// testPlaceAndCancelOrder();
// testPlaceAndCancelOrderByClientId();
