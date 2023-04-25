import * as anchor from "@project-serum/anchor";
import {
  AccountInfo,
  NonceAccount,
  NONCE_ACCOUNT_LENGTH,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import type { OracleAccount } from "@switchboard-xyz/switchboard-v2";
import assert from "assert";
import crypto from "crypto";

export interface OracleNonceAccounts {
  heartbeatNonce?: PublicKey;
  unwrapStakeNonce?: PublicKey;
  queueNonces: PublicKey[];
}

export function getNoncePubkeyFromSeed(
  oracleAccount: OracleAccount,
  authority: PublicKey,
  baseSeed: string
): [PublicKey, string] {
  const seed = `${baseSeed}-${oracleAccount.publicKey.toBase58()}`;
  const seedHashBuffer = crypto.createHash("sha256").update(seed).digest();
  assert(seedHashBuffer.byteLength === 32);
  const seedHashString = seedHashBuffer.toString("hex").slice(0, 32);
  const derivedPubkey = anchor.utils.publicKey.createWithSeedSync(
    authority,
    seedHashString,
    SystemProgram.programId
  );
  return [derivedPubkey, seedHashString];
}

export function nonceAccountExists(accountInfo?: AccountInfo<Buffer>): boolean {
  if (accountInfo && accountInfo.data) {
    const nonceAccount = decodeNonceAccount(accountInfo);
    if (nonceAccount.nonce) {
      return true;
    }
  }
  return false;
}

export async function getOracleHeartbeatNonceAccount(
  oracleAccount: OracleAccount,
  authority: PublicKey
): Promise<PublicKey | null> {
  const [heartbeatNoncePubkey, heartbeatNonceSeed] = getNoncePubkeyFromSeed(
    oracleAccount,
    authority,
    "OracleHeartbeat"
  );
  const accountInfo =
    await oracleAccount.program.provider.connection.getAccountInfo(
      heartbeatNoncePubkey
    );
  if (nonceAccountExists(accountInfo ?? undefined)) {
    return heartbeatNoncePubkey;
  }
  return null;
}

export async function getOracleStakeUnwrapNonceAccount(
  oracleAccount: OracleAccount,
  authority: PublicKey
): Promise<PublicKey | null> {
  const [heartbeatNoncePubkey, heartbeatNonceSeed] = getNoncePubkeyFromSeed(
    oracleAccount,
    authority,
    "UnwrapStakeAccount"
  );
  const accountInfo =
    await oracleAccount.program.provider.connection.getAccountInfo(
      heartbeatNoncePubkey
    );
  if (nonceAccountExists(accountInfo ?? undefined)) {
    return heartbeatNoncePubkey;
  }
  return null;
}

export async function getOracleNonceQueueAccounts(
  oracleAccount: OracleAccount,
  authority: PublicKey,
  queueSize = 1000
): Promise<PublicKey[]> {
  // const queueBaseSeeds: string[] = Array.from(Array(queueSize).keys()).map(
  //   (n) => `NonceQueue-${n.toString().padStart(5, "0")}`
  // );
  // const noncePubkeyWithSeeds: [PublicKey, string][] = queueBaseSeeds.map(
  //   (seed) => getNoncePubkeyFromSeed(oracleAccount, authority, seed)
  // );
  // const pubkeyChunks: [PublicKey, string][][] = sliceIntoChunks(
  //   noncePubkeyWithSeeds,
  //   100
  // );
  // const nonceAccountInfos: {
  //   accountInfo: AccountInfo<Buffer>;
  //   pubkey: PublicKey;
  //   baseSeed: string;
  // }[] = (
  //   await Promise.all(
  //     pubkeyChunks.map(async (chunk) => {
  //       const accountInfos =
  //         await oracleAccount.program.provider.connection.getMultipleAccountsInfo(
  //           chunk.map((i) => i[0])
  //         );
  //       return accountInfos.map((accountInfo, idx) => {
  //         return {
  //           accountInfo,
  //           pubkey: chunk[idx][0],
  //           baseSeed: chunk[idx][1],
  //         };
  //       });
  //     })
  //   )
  // ).flat();

  const queueBaseSeeds: string[] = Array.from(Array(queueSize).keys()).map(
    (n) => `NonceQueue-${n.toString().padStart(5, "0")}`
  );

  // [derivedPubkey, fullSeed, baseSeed]
  const noncePubkeyWithSeeds: {
    pubkey: PublicKey;
    fullSeed: string;
    baseSeed: string;
  }[] = queueBaseSeeds.map((baseSeed) => {
    const [derivedPubkey, fullSeed] = getNoncePubkeyFromSeed(
      oracleAccount,
      authority,
      baseSeed
    );
    return {
      pubkey: derivedPubkey,
      fullSeed,
      baseSeed,
    };
  });

  const pubkeyChunks: {
    pubkey: PublicKey;
    fullSeed: string;
    baseSeed: string;
  }[][] = sliceIntoChunks(noncePubkeyWithSeeds, 100);

  const nonceAccountInfos: {
    accountInfo: AccountInfo<Buffer>;
    pubkey?: PublicKey;
    fullSeed?: string;
    baseSeed?: string;
  }[] = (
    await Promise.all(
      pubkeyChunks.map(async (chunk, chunkIdx) => {
        const accountInfos = (
          await oracleAccount.program.provider.connection.getMultipleAccountsInfo(
            chunk.map((i) => i.pubkey).filter(Boolean) as PublicKey[]
          )
        ).filter(Boolean) as AccountInfo<Buffer>[];
        return accountInfos.map((accountInfo, idx) => {
          return {
            ...chunk[idx],
            accountInfo,
          };
        });
      })
    )
  ).flat();

  const nonceQueuePubkeys = nonceAccountInfos
    .map((nonce, i) => {
      if (nonceAccountExists(nonce.accountInfo ?? undefined)) {
        return nonce.pubkey;
      }
      return undefined;
    })
    .filter(Boolean) as PublicKey[];
  return nonceQueuePubkeys;
}

export async function getOracleNonceAccounts(
  oracleAccount: OracleAccount
): Promise<OracleNonceAccounts> {
  const oracle = await oracleAccount.loadData();
  const heartbeatNonce = await getOracleHeartbeatNonceAccount(
    oracleAccount,
    oracle.oracleAuthority
  );
  const unwrapStakeNonce = await getOracleStakeUnwrapNonceAccount(
    oracleAccount,
    oracle.oracleAuthority
  );
  const queueNonces = await getOracleNonceQueueAccounts(
    oracleAccount,
    oracle.oracleAuthority
  );

  return {
    heartbeatNonce: heartbeatNonce ?? undefined,
    unwrapStakeNonce: unwrapStakeNonce ?? undefined,
    queueNonces,
  };
}

// slice an array into chunks with a max size
function sliceIntoChunks<T>(arr: Array<T>, chunkSize: number): T[][] {
  const res: T[][] = [[]];
  for (let i = 0; i < arr.length; i += chunkSize) {
    const chunk = arr.slice(i, i + chunkSize);
    res.push(chunk);
  }
  return res;
}

export function decodeNonceAccount(info: AccountInfo<Buffer>): NonceAccount {
  if (info === null) {
    throw new Error("FAILED_TO_FIND_ACCOUNT");
  }
  if (!info.owner.equals(SystemProgram.programId)) {
    throw new Error("INVALID_ACCOUNT_OWNER");
  }
  if (info.data.length != NONCE_ACCOUNT_LENGTH) {
    throw new Error(`Invalid account size`);
  }

  const data = Buffer.from(info.data);
  return NonceAccount.fromAccountData(data);
}
