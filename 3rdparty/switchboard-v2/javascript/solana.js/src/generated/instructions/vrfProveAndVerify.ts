import { SwitchboardProgram } from '../../program';
import {
  TransactionInstruction,
  PublicKey,
  AccountMeta,
} from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars

export interface VrfProveAndVerifyArgs {
  params: types.VrfProveAndVerifyParamsFields;
}

export interface VrfProveAndVerifyAccounts {
  vrf: PublicKey;
  callbackPid: PublicKey;
  tokenProgram: PublicKey;
  escrow: PublicKey;
  programState: PublicKey;
  oracle: PublicKey;
  oracleAuthority: PublicKey;
  oracleWallet: PublicKey;
  instructionsSysvar: PublicKey;
}

export const layout = borsh.struct([
  types.VrfProveAndVerifyParams.layout('params'),
]);

export function vrfProveAndVerify(
  program: SwitchboardProgram,
  args: VrfProveAndVerifyArgs,
  accounts: VrfProveAndVerifyAccounts
) {
  const keys: Array<AccountMeta> = [
    { pubkey: accounts.vrf, isSigner: false, isWritable: true },
    { pubkey: accounts.callbackPid, isSigner: false, isWritable: false },
    { pubkey: accounts.tokenProgram, isSigner: false, isWritable: false },
    { pubkey: accounts.escrow, isSigner: false, isWritable: true },
    { pubkey: accounts.programState, isSigner: false, isWritable: false },
    { pubkey: accounts.oracle, isSigner: false, isWritable: false },
    { pubkey: accounts.oracleAuthority, isSigner: true, isWritable: false },
    { pubkey: accounts.oracleWallet, isSigner: false, isWritable: true },
    { pubkey: accounts.instructionsSysvar, isSigner: false, isWritable: false },
  ];
  const identifier = Buffer.from([133, 190, 186, 48, 208, 164, 205, 34]);
  const buffer = Buffer.alloc(1000);
  const len = layout.encode(
    {
      params: types.VrfProveAndVerifyParams.toEncodable(args.params),
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
