import * as anchor from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { VrfAccount } from "@switchboard-xyz/switchboard-v2";
import { promiseWithTimeout } from "./async.js";

export async function awaitOpenRound(
  vrfAccount: VrfAccount,
  counter: anchor.BN,
  timeout = 30
): Promise<Buffer> {
  // call open round and wait for new value
  const accountsCoder = new anchor.BorshAccountsCoder(vrfAccount.program.idl);

  let accountWs: number;
  const awaitUpdatePromise = new Promise((resolve: (value: Buffer) => void) => {
    accountWs = vrfAccount.program.provider.connection.onAccountChange(
      vrfAccount?.publicKey ?? PublicKey.default,
      async (accountInfo) => {
        const vrf = accountsCoder.decode("VrfAccountData", accountInfo.data);
        if (!counter.eq(vrf.counter)) {
          return;
        }
        if (vrf.result.every((val) => val === 0)) {
          return;
        }
        resolve(vrf.result as Buffer);
      }
    );
  });

  const updatedValuePromise = promiseWithTimeout(
    timeout * 1000,
    awaitUpdatePromise,
    new Error(`vrf failed to update in ${timeout} seconds`)
  ).finally(() => {
    if (accountWs) {
      vrfAccount.program.provider.connection.removeAccountChangeListener(
        accountWs
      );
    }
  });

  const result = await updatedValuePromise;

  if (!result) {
    throw new Error(`failed to update VRF`);
  }

  return result;
}
