import { SwitchboardProgram } from '../../program';
import {
  TransactionInstruction,
  PublicKey,
  AccountMeta,
} from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars

export interface VrfSetCallbackArgs {
  params: types.VrfSetCallbackParamsFields;
}

export interface VrfSetCallbackAccounts {
  vrf: PublicKey;
  authority: PublicKey;
}

export const layout = borsh.struct([
  types.VrfSetCallbackParams.layout('params'),
]);

export function vrfSetCallback(
  program: SwitchboardProgram,
  args: VrfSetCallbackArgs,
  accounts: VrfSetCallbackAccounts
) {
  const keys: Array<AccountMeta> = [
    { pubkey: accounts.vrf, isSigner: false, isWritable: true },
    { pubkey: accounts.authority, isSigner: true, isWritable: false },
  ];
  const identifier = Buffer.from([121, 167, 168, 191, 180, 247, 251, 78]);
  const buffer = Buffer.alloc(1000);
  const len = layout.encode(
    {
      params: types.VrfSetCallbackParams.toEncodable(args.params),
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
