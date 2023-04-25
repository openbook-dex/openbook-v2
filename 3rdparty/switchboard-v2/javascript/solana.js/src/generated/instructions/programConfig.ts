import { SwitchboardProgram } from '../../program';
import {
  TransactionInstruction,
  PublicKey,
  AccountMeta,
} from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars

export interface ProgramConfigArgs {
  params: types.ProgramConfigParamsFields;
}

export interface ProgramConfigAccounts {
  authority: PublicKey;
  programState: PublicKey;
  daoMint: PublicKey;
}

export const layout = borsh.struct([
  types.ProgramConfigParams.layout('params'),
]);

export function programConfig(
  program: SwitchboardProgram,
  args: ProgramConfigArgs,
  accounts: ProgramConfigAccounts
) {
  const keys: Array<AccountMeta> = [
    { pubkey: accounts.authority, isSigner: true, isWritable: false },
    { pubkey: accounts.programState, isSigner: false, isWritable: true },
    { pubkey: accounts.daoMint, isSigner: false, isWritable: false },
  ];
  const identifier = Buffer.from([62, 123, 20, 150, 56, 109, 209, 145]);
  const buffer = Buffer.alloc(1000);
  const len = layout.encode(
    {
      params: types.ProgramConfigParams.toEncodable(args.params),
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
