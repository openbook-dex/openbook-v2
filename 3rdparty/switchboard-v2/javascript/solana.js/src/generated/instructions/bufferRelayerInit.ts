import { SwitchboardProgram } from '../../program';
import {
  TransactionInstruction,
  PublicKey,
  AccountMeta,
} from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars

export interface BufferRelayerInitArgs {
  params: types.BufferRelayerInitParamsFields;
}

export interface BufferRelayerInitAccounts {
  bufferRelayer: PublicKey;
  escrow: PublicKey;
  authority: PublicKey;
  queue: PublicKey;
  job: PublicKey;
  programState: PublicKey;
  mint: PublicKey;
  payer: PublicKey;
  tokenProgram: PublicKey;
  associatedTokenProgram: PublicKey;
  systemProgram: PublicKey;
  rent: PublicKey;
}

export const layout = borsh.struct([
  types.BufferRelayerInitParams.layout('params'),
]);

export function bufferRelayerInit(
  program: SwitchboardProgram,
  args: BufferRelayerInitArgs,
  accounts: BufferRelayerInitAccounts
) {
  const keys: Array<AccountMeta> = [
    { pubkey: accounts.bufferRelayer, isSigner: false, isWritable: true },
    { pubkey: accounts.escrow, isSigner: false, isWritable: true },
    { pubkey: accounts.authority, isSigner: false, isWritable: false },
    { pubkey: accounts.queue, isSigner: false, isWritable: false },
    { pubkey: accounts.job, isSigner: false, isWritable: false },
    { pubkey: accounts.programState, isSigner: false, isWritable: false },
    { pubkey: accounts.mint, isSigner: false, isWritable: false },
    { pubkey: accounts.payer, isSigner: true, isWritable: true },
    { pubkey: accounts.tokenProgram, isSigner: false, isWritable: false },
    {
      pubkey: accounts.associatedTokenProgram,
      isSigner: false,
      isWritable: false,
    },
    { pubkey: accounts.systemProgram, isSigner: false, isWritable: false },
    { pubkey: accounts.rent, isSigner: false, isWritable: false },
  ];
  const identifier = Buffer.from([127, 205, 59, 151, 4, 47, 164, 82]);
  const buffer = Buffer.alloc(1000);
  const len = layout.encode(
    {
      params: types.BufferRelayerInitParams.toEncodable(args.params),
    },
    buffer
  );
  const data = Buffer.concat([identifier, buffer]).slice(0, 8 + len);
  const ix = new TransactionInstruction({
    keys,
    programId: program.programId,
    data,
  });
  return ix;
}
