import { OPENBOOK_PROGRAM_ID, findAllMarkets, initReadOnlyOpenbookClient } from "..";

async function testListAll(): Promise<void> {
  const client = initReadOnlyOpenbookClient(process.env.SOL_RPC_URL as any);
  const markets = await findAllMarkets(client.connection, OPENBOOK_PROGRAM_ID, client.provider);
  console.log('markets', markets);
}

void testListAll();