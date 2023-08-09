import { PublicKey, type Connection, type AccountInfo } from '@solana/web3.js';
import { getFilteredProgramAccounts } from './client';
import { utils, getProvider, Program, type Provider } from '@coral-xyz/anchor';

import { IDL, type OpenbookV2 } from './openbook_v2';

export async function findAccountsByMints(
  connection: Connection,
  baseMintAddress: PublicKey,
  quoteMintAddress: PublicKey,
  programId: PublicKey,
): Promise<Array<{ publicKey: PublicKey; accountInfo: AccountInfo<Buffer> }>> {
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

interface Market {
  market: string;
  baseMint: string;
  quoteMint: string;
  name: string;
}
export async function findAllMarkets(
  connection: Connection,
  programId: PublicKey,
  provider?: Provider,
): Promise<Market[]> {
  if (provider === null) {
    provider = getProvider();
  }
  const program = new Program<OpenbookV2>(IDL, programId, provider);

  const [eventAuthority] = PublicKey.findProgramAddressSync(
    [Buffer.from('__event_authority')],
    programId,
  );

  const signatures = await connection.getSignaturesForAddress(eventAuthority);
  const marketsAll: Market[] = [];
  for (const signature of signatures) {
    const tx = await connection.getTransaction(signature.signature, {
      commitment: 'confirmed',
    });
    if (
      tx?.meta?.innerInstructions !== null &&
      tx?.meta?.innerInstructions !== undefined &&
      tx.meta.innerInstructions[0].instructions.length === 2
    ) {
      // validate key and program key
      const eventAuthorityKey =
        tx.meta.innerInstructions[0].instructions[1].accounts[0];
      const programKey =
        tx.meta.innerInstructions[0].instructions[1].programIdIndex;

      if (
        tx.transaction.message.accountKeys[eventAuthorityKey].toString() !==
          eventAuthority.toString() ||
        tx.transaction.message.accountKeys[programKey].toString() !==
          programId.toString()
      ) {
        console.log('This is not a valid event!');
        continue;
      } else {
        const ixData = utils.bytes.bs58.decode(
          tx.meta.innerInstructions[0].instructions[1].data,
        );
        const eventData = utils.bytes.base64.encode(ixData.slice(8));
        const event = program.coder.events.decode(eventData);

        if (event != null) {
          const market: Market = {
            market: (event.data.market as PublicKey).toString(),
            baseMint: (event.data.baseMint as PublicKey).toString(),
            quoteMint: (event.data.quoteMint as PublicKey).toString(),
            name: event.data.name as string,
          };
          marketsAll.push(market);
        }
      }
    }
  }

  return marketsAll;
}
