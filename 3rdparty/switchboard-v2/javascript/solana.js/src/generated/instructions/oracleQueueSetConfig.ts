import { SwitchboardProgram } from '../../program';
import {
  TransactionInstruction,
  PublicKey,
  AccountMeta,
} from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars

export interface OracleQueueSetConfigArgs {
  params: types.OracleQueueSetConfigParamsFields;
}

export interface OracleQueueSetConfigAccounts {
  queue: PublicKey;
  authority: PublicKey;
}

export const layout = borsh.struct([
  types.OracleQueueSetConfigParams.layout('params'),
]);

export function oracleQueueSetConfig(
  program: SwitchboardProgram,
  args: OracleQueueSetConfigArgs,
  accounts: OracleQueueSetConfigAccounts
) {
  const keys: Array<AccountMeta> = [
    { pubkey: accounts.queue, isSigner: false, isWritable: true },
    { pubkey: accounts.authority, isSigner: true, isWritable: false },
  ];
  const identifier = Buffer.from([239, 87, 216, 48, 119, 222, 83, 220]);
  const buffer = Buffer.alloc(1000);
  const len = layout.encode(
    {
      params: types.OracleQueueSetConfigParams.toEncodable(args.params),
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
