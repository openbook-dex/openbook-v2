import { PublicKey, type Connection, type AccountInfo } from '@solana/web3.js';
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
      maxSupportedTransactionVersion: 0, // Set the maximum supported transaction version
    });
    
    for (const tx of allTxs) {
      if (
        tx?.meta?.innerInstructions !== null &&
        tx?.meta?.innerInstructions !== undefined
      )
        for (const innerIns of tx.meta.innerInstructions) {
          // Check if innerIns.instructions[1] exists and has the 'accounts' property
          if (
            innerIns.instructions[1] &&
            innerIns.instructions[1].accounts &&
            Array.isArray(innerIns.instructions[1].accounts)
          ) {
            const eventAuthorityKeyIndex = innerIns.instructions[1].accounts[0];
            const programKeyIndex = innerIns.instructions[1].programIdIndex;
    
            // console.log("eventAuthorityKeyIndex:", eventAuthorityKeyIndex);
            // console.log("programKeyIndex:", programKeyIndex);
      
            const message = tx.transaction.message;
            // console.log("message:", message);
      
            if (!message) {
              // console.error("Message is undefined!");
              continue;
            }
      
            const accountKeys = message.getAccountKeys();
            // console.log("accountKeys:", accountKeys);
      
            if (!accountKeys) {
              console.error("Account keys are undefined!");
              continue;
            }
      
            const eventAuthorityKey: any = accountKeys.get(eventAuthorityKeyIndex);
            const programKey: any = accountKeys.get(programKeyIndex);
      
            // console.log("eventAuthorityKey:", eventAuthorityKey);
            // console.log("programKey:", programKey);
      
            if (
              eventAuthorityKey.toString() !== eventAuthority.toString() ||
              programKey.toString() !== programId.toString()
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
          } else {
            console.error("Inner instruction is missing 'accounts' property.");
          }
        }
    }
  }

  return marketsAll;
}
