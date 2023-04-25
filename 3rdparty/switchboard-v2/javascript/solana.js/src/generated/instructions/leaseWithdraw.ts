import { SwitchboardProgram } from '../../program';
import {
  TransactionInstruction,
  PublicKey,
  AccountMeta,
} from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars

export interface LeaseWithdrawArgs {
  params: types.LeaseWithdrawParamsFields;
}

export interface LeaseWithdrawAccounts {
  lease: PublicKey;
  escrow: PublicKey;
  aggregator: PublicKey;
  queue: PublicKey;
  withdrawAuthority: PublicKey;
  withdrawAccount: PublicKey;
  tokenProgram: PublicKey;
  programState: PublicKey;
  mint: PublicKey;
}

export const layout = borsh.struct([
  types.LeaseWithdrawParams.layout('params'),
]);

export function leaseWithdraw(
  program: SwitchboardProgram,
  args: LeaseWithdrawArgs,
  accounts: LeaseWithdrawAccounts
) {
  const keys: Array<AccountMeta> = [
    { pubkey: accounts.lease, isSigner: false, isWritable: true },
    { pubkey: accounts.escrow, isSigner: false, isWritable: true },
    { pubkey: accounts.aggregator, isSigner: false, isWritable: false },
    { pubkey: accounts.queue, isSigner: false, isWritable: false },
    { pubkey: accounts.withdrawAuthority, isSigner: true, isWritable: false },
    { pubkey: accounts.withdrawAccount, isSigner: false, isWritable: true },
    { pubkey: accounts.tokenProgram, isSigner: false, isWritable: false },
    { pubkey: accounts.programState, isSigner: false, isWritable: false },
    { pubkey: accounts.mint, isSigner: false, isWritable: false },
  ];
  const identifier = Buffer.from([186, 41, 100, 248, 234, 81, 61, 169]);
  const buffer = Buffer.alloc(1000);
  const len = layout.encode(
    {
      params: types.LeaseWithdrawParams.toEncodable(args.params),
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
