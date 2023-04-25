import { SwitchboardProgram } from '../../program';
import {
  TransactionInstruction,
  PublicKey,
  AccountMeta,
} from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars

export interface JobSetDataArgs {
  params: types.JobSetDataParamsFields;
}

export interface JobSetDataAccounts {
  job: PublicKey;
  authority: PublicKey;
}

export const layout = borsh.struct([types.JobSetDataParams.layout('params')]);

export function jobSetData(
  program: SwitchboardProgram,
  args: JobSetDataArgs,
  accounts: JobSetDataAccounts
) {
  const keys: Array<AccountMeta> = [
    { pubkey: accounts.job, isSigner: false, isWritable: true },
    { pubkey: accounts.authority, isSigner: true, isWritable: false },
  ];
  const identifier = Buffer.from([225, 207, 69, 27, 161, 171, 223, 104]);
  const buffer = Buffer.alloc(1000);
  const len = layout.encode(
    {
      params: types.JobSetDataParams.toEncodable(args.params),
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
