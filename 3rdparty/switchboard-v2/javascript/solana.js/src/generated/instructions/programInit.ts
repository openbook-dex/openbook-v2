import { SwitchboardProgram } from '../../program';
import {
  TransactionInstruction,
  PublicKey,
  AccountMeta,
} from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars

export interface ProgramInitArgs {
  params: types.ProgramInitParamsFields;
}

export interface ProgramInitAccounts {
  state: PublicKey;
  authority: PublicKey;
  tokenMint: PublicKey;
  vault: PublicKey;
  payer: PublicKey;
  systemProgram: PublicKey;
  tokenProgram: PublicKey;
  daoMint: PublicKey;
}

export const layout = borsh.struct([types.ProgramInitParams.layout('params')]);

export function programInit(
  program: SwitchboardProgram,
  args: ProgramInitArgs,
  accounts: ProgramInitAccounts
) {
  const keys: Array<AccountMeta> = [
    { pubkey: accounts.state, isSigner: false, isWritable: true },
    { pubkey: accounts.authority, isSigner: false, isWritable: false },
    { pubkey: accounts.tokenMint, isSigner: false, isWritable: true },
    { pubkey: accounts.vault, isSigner: false, isWritable: true },
    { pubkey: accounts.payer, isSigner: true, isWritable: true },
    { pubkey: accounts.systemProgram, isSigner: false, isWritable: false },
    { pubkey: accounts.tokenProgram, isSigner: false, isWritable: false },
    { pubkey: accounts.daoMint, isSigner: false, isWritable: false },
  ];
  const identifier = Buffer.from([199, 209, 193, 213, 138, 30, 175, 13]);
  const buffer = Buffer.alloc(1000);
  const len = layout.encode(
    {
      params: types.ProgramInitParams.toEncodable(args.params),
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
