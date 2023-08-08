import {
  PublicKey,
  type Connection,
} from '@solana/web3.js';
import { getFilteredProgramAccounts } from './client';
import {
  utils,
  getProvider,
  Program,
} from '@coral-xyz/anchor';

import { IDL, type OpenbookV2 } from './openbook_v2';

export async function findAccountsByMints(
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

export async function findAllMarkets(
  connection: Connection,
  programId: PublicKey,
): Promise<any[]> {
  const program = new Program<OpenbookV2>(
    IDL ,
    programId,
    getProvider(),
  );

  const [eventAuthority, tmp3] = PublicKey.findProgramAddressSync(
    [Buffer.from('__event_authority')],
    programId,
  );
  console.log('eventAuthority', eventAuthority.toString());

  const signatures = await connection.getSignaturesForAddress(eventAuthority);
  const marketsAll: string[] = [];
  for (const signature of signatures) {
    const tx = await connection.getTransaction(signature.signature, {
      commitment: 'confirmed',
    });
    if (
      ((tx?.meta?.innerInstructions) != null) &&
      tx.meta.innerInstructions[0].instructions.length == 2
    ) {
      const ixData = utils.bytes.bs58.decode(
        tx.meta.innerInstructions[0].instructions[1].data,
      );
      const eventData = utils.bytes.base64.encode(ixData.slice(8));
      const event = program.coder.events.decode(eventData);

      if (event != null) {
        const market = (event.data.market as PublicKey).toString();
        marketsAll.push(market);
      }
    }
  }

  return marketsAll;
}
