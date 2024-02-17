import { PublicKey } from "@solana/web3.js";
import { OPENBOOK_PROGRAM_ID, findAccountsByMints, findAllMarkets, initReadOnlyOpenbookClient } from "..";


async function testFindAccountsByMints(): Promise<void> {
  const client = initReadOnlyOpenbookClient(process.env.SOL_RPC_URL as any);
  const accounts = await findAccountsByMints(client.connection, new PublicKey("So11111111111111111111111111111111111111112"), new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), OPENBOOK_PROGRAM_ID);
  console.log(accounts.map(a => a.publicKey.toBase58()));
}

async function testFindAllMarkets(): Promise<void> {
  const client = initReadOnlyOpenbookClient(process.env.SOL_RPC_URL as any);
  const markets = await findAllMarkets(client.connection, OPENBOOK_PROGRAM_ID, client.provider);
  console.log('markets', markets);
}

void testFindAccountsByMints();
void testFindAllMarkets();