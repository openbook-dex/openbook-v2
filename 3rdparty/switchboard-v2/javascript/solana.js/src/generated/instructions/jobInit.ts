import { SwitchboardProgram } from '../../program';
import {
  TransactionInstruction,
  PublicKey,
  AccountMeta,
} from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars

export interface JobInitArgs {
  params: types.JobInitParamsFields;
}

export interface JobInitAccounts {
  job: PublicKey;
  authority: PublicKey;
  programState: PublicKey;
  payer: PublicKey;
  systemProgram: PublicKey;
}

export const layout = borsh.struct([types.JobInitParams.layout('params')]);

export function jobInit(
  program: SwitchboardProgram,
  args: JobInitArgs,
  accounts: JobInitAccounts
) {
  const keys: Array<AccountMeta> = [
    { pubkey: accounts.job, isSigner: true, isWritable: true },
    { pubkey: accounts.authority, isSigner: true, isWritable: false },
    { pubkey: accounts.programState, isSigner: false, isWritable: false },
    { pubkey: accounts.payer, isSigner: true, isWritable: true },
    { pubkey: accounts.systemProgram, isSigner: false, isWritable: false },
  ];
  const identifier = Buffer.from([101, 86, 105, 192, 34, 201, 147, 159]);
  const buffer = Buffer.alloc(1000);
  const len = layout.encode(
    {
      params: types.JobInitParams.toEncodable(args.params),
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
