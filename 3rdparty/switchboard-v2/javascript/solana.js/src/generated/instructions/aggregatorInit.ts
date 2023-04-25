import { SwitchboardProgram } from '../../program';
import {
  TransactionInstruction,
  PublicKey,
  AccountMeta,
} from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars

export interface AggregatorInitArgs {
  params: types.AggregatorInitParamsFields;
}

export interface AggregatorInitAccounts {
  aggregator: PublicKey;
  authority: PublicKey;
  queue: PublicKey;
  programState: PublicKey;
}

export const layout = borsh.struct([
  types.AggregatorInitParams.layout('params'),
]);

export function aggregatorInit(
  program: SwitchboardProgram,
  args: AggregatorInitArgs,
  accounts: AggregatorInitAccounts
) {
  const keys: Array<AccountMeta> = [
    { pubkey: accounts.aggregator, isSigner: false, isWritable: true },
    { pubkey: accounts.authority, isSigner: false, isWritable: false },
    { pubkey: accounts.queue, isSigner: false, isWritable: false },
    { pubkey: accounts.programState, isSigner: false, isWritable: false },
  ];
  const identifier = Buffer.from([200, 41, 88, 11, 36, 21, 181, 110]);
  const buffer = Buffer.alloc(1000);
  const len = layout.encode(
    {
      params: types.AggregatorInitParams.toEncodable(args.params),
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
