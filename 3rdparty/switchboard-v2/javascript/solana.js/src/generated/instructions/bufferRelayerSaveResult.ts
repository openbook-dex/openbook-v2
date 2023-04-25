import { SwitchboardProgram } from '../../program';
import {
  TransactionInstruction,
  PublicKey,
  AccountMeta,
} from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars

export interface BufferRelayerSaveResultArgs {
  params: types.BufferRelayerSaveResultParamsFields;
}

export interface BufferRelayerSaveResultAccounts {
  bufferRelayer: PublicKey;
  oracleAuthority: PublicKey;
  oracle: PublicKey;
  oracleQueue: PublicKey;
  dataBuffer: PublicKey;
  queueAuthority: PublicKey;
  permission: PublicKey;
  escrow: PublicKey;
  oracleWallet: PublicKey;
  programState: PublicKey;
  tokenProgram: PublicKey;
}

export const layout = borsh.struct([
  types.BufferRelayerSaveResultParams.layout('params'),
]);

export function bufferRelayerSaveResult(
  program: SwitchboardProgram,
  args: BufferRelayerSaveResultArgs,
  accounts: BufferRelayerSaveResultAccounts
) {
  const keys: Array<AccountMeta> = [
    { pubkey: accounts.bufferRelayer, isSigner: false, isWritable: true },
    { pubkey: accounts.oracleAuthority, isSigner: true, isWritable: false },
    { pubkey: accounts.oracle, isSigner: false, isWritable: false },
    { pubkey: accounts.oracleQueue, isSigner: false, isWritable: true },
    { pubkey: accounts.dataBuffer, isSigner: false, isWritable: true },
    { pubkey: accounts.queueAuthority, isSigner: false, isWritable: false },
    { pubkey: accounts.permission, isSigner: false, isWritable: true },
    { pubkey: accounts.escrow, isSigner: false, isWritable: true },
    { pubkey: accounts.oracleWallet, isSigner: false, isWritable: true },
    { pubkey: accounts.programState, isSigner: false, isWritable: false },
    { pubkey: accounts.tokenProgram, isSigner: false, isWritable: false },
  ];
  const identifier = Buffer.from([253, 170, 164, 84, 155, 112, 1, 46]);
  const buffer = Buffer.alloc(1000);
  const len = layout.encode(
    {
      params: types.BufferRelayerSaveResultParams.toEncodable(args.params),
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
