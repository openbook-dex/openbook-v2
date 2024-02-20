import { PublicKey } from '@solana/web3.js';
import { OpenBookV2Client, OpenOrdersIndexerAccount } from '../client';

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
}
