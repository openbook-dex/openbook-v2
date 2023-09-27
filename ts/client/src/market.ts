import {
  PublicKey,
  type Connection,
  type AccountInfo,
  type Message,
} from '@solana/web3.js';
import { getFilteredProgramAccounts } from './client';
import { utils, Program, type Provider, getProvider } from '@coral-xyz/anchor';

import { IDL, type OpenbookV2 } from './openbook_v2';
const BATCH_TX_SIZE = 50;

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
  timestamp: number | null | undefined;
}
export async function findAllMarkets(
  connection: Connection,
  programId: PublicKey,
  provider?: Provider,
): Promise<Market[]> {
  if (provider == null) {
    provider = getProvider();
  }
  const program = new Program<OpenbookV2>(IDL, programId, provider);

  const [eventAuthority] = PublicKey.findProgramAddressSync(
    [Buffer.from('__event_authority')],
    programId,
  );
  const marketsAll: Market[] = [];

  const signatures = (
    await connection.getSignaturesForAddress(eventAuthority)
  ).map((x) => x.signature);
  const batchSignatures: [string[]] = [[]];
  for (let i = 0; i < signatures.length; i += BATCH_TX_SIZE) {
    batchSignatures.push(signatures.slice(0, BATCH_TX_SIZE));
  }
  for (const batch of batchSignatures) {
    const allTxs = await connection.getTransactions(batch, {
      commitment: 'confirmed',
      maxSupportedTransactionVersion: 0,
    });
    for (const tx of allTxs) {
      if (
        tx?.meta?.innerInstructions !== null &&
        tx?.meta?.innerInstructions !== undefined
      )
        for (const innerIns of tx.meta.innerInstructions) {
          if (innerIns.instructions?.[1]?.accounts?.[0] !== undefined) {
            console.log('err');
            // validate key and program key
            const eventAuthorityKey = innerIns.instructions[1].accounts[0];
            const programKey = innerIns.instructions[1].programIdIndex;
            if (
              (tx.transaction.message as Message).staticAccountKeys[
                eventAuthorityKey
              ].toString() !== eventAuthority.toString() ||
              (tx.transaction.message as Message).staticAccountKeys[
                programKey
              ].toString() !== programId.toString()
            ) {
              continue;
            } else {
              const ixData = utils.bytes.bs58.decode(
                innerIns.instructions[1].data,
              );
              const eventData = utils.bytes.base64.encode(ixData.slice(8));
              const event = program.coder.events.decode(eventData);

              if (event != null) {
                const market: Market = {
                  market: (event.data.market as PublicKey).toString(),
                  baseMint: (event.data.baseMint as PublicKey).toString(),
                  quoteMint: (event.data.quoteMint as PublicKey).toString(),
                  name: event.data.name as string,
                  timestamp: tx.blockTime,
                };
                marketsAll.push(market);
              }
            }
          }
        }
    }
  }
  return marketsAll;
}
