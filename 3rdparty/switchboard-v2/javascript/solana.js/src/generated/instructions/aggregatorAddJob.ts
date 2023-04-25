import { SwitchboardProgram } from '../../program';
import {
  TransactionInstruction,
  PublicKey,
  AccountMeta,
} from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars

export interface AggregatorAddJobArgs {
  params: types.AggregatorAddJobParamsFields;
}

export interface AggregatorAddJobAccounts {
  aggregator: PublicKey;
  authority: PublicKey;
  job: PublicKey;
}

export const layout = borsh.struct([
  types.AggregatorAddJobParams.layout('params'),
]);

export function aggregatorAddJob(
  program: SwitchboardProgram,
  args: AggregatorAddJobArgs,
  accounts: AggregatorAddJobAccounts
) {
  const keys: Array<AccountMeta> = [
    { pubkey: accounts.aggregator, isSigner: false, isWritable: true },
    { pubkey: accounts.authority, isSigner: true, isWritable: false },
    { pubkey: accounts.job, isSigner: false, isWritable: true },
  ];
  const identifier = Buffer.from([132, 30, 35, 51, 115, 142, 186, 10]);
  const buffer = Buffer.alloc(1000);
  const len = layout.encode(
    {
      params: types.AggregatorAddJobParams.toEncodable(args.params),
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
