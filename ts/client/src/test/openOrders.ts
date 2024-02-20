import { PublicKey } from '@solana/web3.js';
import {
  Market,
  OpenOrders,
  SideUtils,
  initOpenbookClient,
  initReadOnlyOpenbookClient,
} from '..';
import { OpenOrdersIndexer } from '../accounts/openOrdersIndexer';

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
  ]);
  // console.log(oo?.pubkey.toBase58());
  // console.log(oo?.account.position);
  // console.log(oo?.account.openOrders);
  // console.log(oo);
  console.log(oo?.toPrettyString());
  // TODO: test reading orders
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

  await oo?.reload();

  console.log(oo?.toPrettyString());

  const sigCancel = await oo?.cancelAllOrders(SideUtils.Bid);

  console.log('cancelled order', sigCancel);
}

testLoadIndexerNonExistent();
testLoadOOForMarket();
testPlaceAndCancelOrder();
