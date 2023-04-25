import { SwitchboardProgram } from '../../program';
import {
  TransactionInstruction,
  PublicKey,
  AccountMeta,
} from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars

export interface VrfRequestRandomnessArgs {
  params: types.VrfRequestRandomnessParamsFields;
}

export interface VrfRequestRandomnessAccounts {
  authority: PublicKey;
  vrf: PublicKey;
  oracleQueue: PublicKey;
  queueAuthority: PublicKey;
  dataBuffer: PublicKey;
  permission: PublicKey;
  escrow: PublicKey;
  payerWallet: PublicKey;
  payerAuthority: PublicKey;
  recentBlockhashes: PublicKey;
  programState: PublicKey;
  tokenProgram: PublicKey;
}

export const layout = borsh.struct([
  types.VrfRequestRandomnessParams.layout('params'),
]);

export function vrfRequestRandomness(
  program: SwitchboardProgram,
  args: VrfRequestRandomnessArgs,
  accounts: VrfRequestRandomnessAccounts
) {
  const keys: Array<AccountMeta> = [
    { pubkey: accounts.authority, isSigner: true, isWritable: false },
    { pubkey: accounts.vrf, isSigner: false, isWritable: true },
    { pubkey: accounts.oracleQueue, isSigner: false, isWritable: true },
    { pubkey: accounts.queueAuthority, isSigner: false, isWritable: false },
    { pubkey: accounts.dataBuffer, isSigner: false, isWritable: false },
    { pubkey: accounts.permission, isSigner: false, isWritable: true },
    { pubkey: accounts.escrow, isSigner: false, isWritable: true },
    { pubkey: accounts.payerWallet, isSigner: false, isWritable: true },
    { pubkey: accounts.payerAuthority, isSigner: true, isWritable: false },
    { pubkey: accounts.recentBlockhashes, isSigner: false, isWritable: false },
    { pubkey: accounts.programState, isSigner: false, isWritable: false },
    { pubkey: accounts.tokenProgram, isSigner: false, isWritable: false },
  ];
  const identifier = Buffer.from([230, 121, 14, 164, 28, 222, 117, 118]);
  const buffer = Buffer.alloc(1000);
  const len = layout.encode(
    {
      params: types.VrfRequestRandomnessParams.toEncodable(args.params),
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
