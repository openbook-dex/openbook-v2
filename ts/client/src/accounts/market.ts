import { type PublicKey, type Connection } from '@solana/web3.js';
import { Program } from '@coral-xyz/anchor';
import { ProgramId } from '../utils/utils';
import { IDL, type OpenbookV2 } from '../openbook_v2';

export class Market {
  publicKey: PublicKey;
  asks: PublicKey;
  bids: PublicKey;
  baseVault: PublicKey;
  quoteVault: PublicKey;
  eventQueue: PublicKey;
  oracleA: PublicKey | null;
  oracleB: PublicKey | null;

  constructor(
    publicKey: PublicKey,
    asks: PublicKey,
    bids: PublicKey,
    eventQueue: PublicKey,
    oracleA: PublicKey | null,
    oracleB: PublicKey | null,
  ) {
    this.publicKey = publicKey;
    this.asks = asks;
    this.bids = bids;
    this.eventQueue = eventQueue;
    this.oracleA = oracleA ?? null;
    this.oracleB = oracleB ?? null;
  }

  async fromPublicKey(
    publicKey: PublicKey,
    programId = ProgramId,
    connection: Connection,
  ): Promise<Market> {
    const program = new Program<OpenbookV2>(IDL, programId);

    const account = await connection.getAccountInfo(publicKey);
    if (account != null) {
      const market = program.coder.accounts.decode('Market', account.data);

      this.publicKey = publicKey;
      this.asks = market.asks;
      this.bids = market.bids;
      this.eventQueue = market.eventQueue;
      this.oracleA = market.oracleA ?? null;
      this.oracleB = market.oracleB ?? null;
    }
    return this;
  }
}
