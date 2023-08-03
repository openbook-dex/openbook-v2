import { type Connection, type PublicKey } from '@solana/web3.js';
import { getFilteredProgramAccounts } from './client';

async function findAccountsByMints(
  connection: Connection,
  baseMintAddress: PublicKey,
  quoteMintAddress: PublicKey,
  programId: PublicKey,
) {
  const filters = [
    {
      memcmp: {
        offset: 792,
        bytes: baseMintAddress.toBase58(),
      },
    },
    {
      memcmp: {
        offset: 824,
        bytes: quoteMintAddress.toBase58(),
      },
    },
  ];
  return await getFilteredProgramAccounts(connection, programId, filters);
}
