import * as anchor from "@project-serum/anchor";
/*eslint-disable import/extensions */
import { findProgramAddressSync } from "@project-serum/anchor/dist/cjs/utils/pubkey.js";
import {
  Cluster,
  clusterApiUrl,
  Connection,
  Keypair,
  PublicKey,
} from "@solana/web3.js";
import { AnchorWallet } from "@switchboard-xyz/switchboard-v2";
import fs from "fs";
import path from "path";
import toml from "toml";
import { DEFAULT_KEYPAIR } from "./const.js";
import { NoPayerKeypairProvided } from "./errors.js";

export function programWallet(program: anchor.Program): Keypair {
  return ((program.provider as anchor.AnchorProvider).wallet as AnchorWallet)
    .payer;
}

/** Return the default anchor.AnchorProvider that will fail if a transaction is sent. This is used to avoid accidentally loading a
 * valid keypair from the anchor environment defaults.
 * @param connection a Solana connection object for a given Solana cluster and endpoint
 * @return the anchor.AnchorProvider object
 * */
export const getDefaultProvider = (
  connection: Connection
): anchor.AnchorProvider => {
  return new anchor.AnchorProvider(
    connection,
    new AnchorWallet(DEFAULT_KEYPAIR),
    anchor.AnchorProvider.defaultOptions()
  );
};

/** Get the program data address for a given programId
 * @param programId the programId for a given on-chain program
 * @return the publicKey of the address holding the upgradeable program buffer
 */
export const getProgramDataAddress = (programId: PublicKey): PublicKey => {
  return findProgramAddressSync(
    [programId.toBytes()],
    new PublicKey("BPFLoaderUpgradeab1e11111111111111111111111")
  )[0];
};

/** Get the IDL address for a given programId
 * @param programId the programId for a given on-chain program
 * @return the publicKey of the IDL address
 */
export const getIdlAddress = async (
  programId: PublicKey
): Promise<PublicKey> => {
  const base = (await PublicKey.findProgramAddress([], programId))[0];
  return PublicKey.createWithSeed(base, "anchor:idl", programId);
};

export const programHasPayer = (program: anchor.Program): boolean => {
  const payer = programWallet(program);
  return !payer.publicKey.equals(DEFAULT_KEYPAIR.publicKey);
};

export const getProgramPayer = (program: anchor.Program): Keypair => {
  const wallet = programWallet(program);
  if (programHasPayer(program)) {
    return wallet;
  }
  throw new NoPayerKeypairProvided();
};

export const verifyProgramHasPayer = (program: anchor.Program): void => {
  if (programHasPayer(program)) {
    return;
  }
  throw new NoPayerKeypairProvided();
};

export function getAnchorWalletPath(parsedToml?: any): string {
  let tomlData: any;
  if (parsedToml) {
    tomlData = parsedToml;
  } else {
    const tomlPath = path.join(process.cwd(), "Anchor.toml");
    if (!fs.existsSync(tomlPath)) {
      throw new Error(`failed to find Anchor.toml`);
    }
    tomlData = toml.parse(fs.readFileSync(tomlPath, "utf8"));
  }

  const walletPath = tomlData.provider.wallet;
  if (!walletPath) {
    throw new Error(`Failed to read wallet path`);
  }
  return walletPath;
}

export function getAnchorCluster(parsedToml?: any): string {
  let tomlData: any;
  if (parsedToml) {
    tomlData = parsedToml;
  } else {
    const tomlPath = path.join(process.cwd(), "Anchor.toml");
    if (!fs.existsSync(tomlPath)) {
      throw new Error(`failed to find Anchor.toml`);
    }
    tomlData = toml.parse(fs.readFileSync(tomlPath, "utf8"));
  }

  const cluster = tomlData.provider.cluster;
  if (!cluster) {
    throw new Error(`Failed to read Anchor.toml cluster`);
  }
  return cluster;
}

export function loadPid(programKeypairPath: string): PublicKey {
  if (!fs.existsSync(programKeypairPath)) {
    console.log(programKeypairPath);
    throw new Error(`Could not find keypair. Have you run 'anchor build'?`);
  }
  const programKeypair = Keypair.fromSecretKey(
    new Uint8Array(JSON.parse(fs.readFileSync(programKeypairPath, "utf8")))
  );
  return programKeypair.publicKey;
}

export function getWorkspace(
  programName: string,
  programPath: string
): anchor.Program {
  const tomlPath = path.join(programPath, "Anchor.toml");
  if (!fs.existsSync(tomlPath)) {
    throw new Error(`failed to find Anchor.toml`);
  }
  const tomlData = toml.parse(fs.readFileSync(tomlPath, "utf8"));

  const cluster: Cluster | "localnet" = tomlData.provider.cluster;
  const wallet = Keypair.fromSecretKey(
    Buffer.from(
      JSON.parse(
        fs.readFileSync(tomlData.provider.wallet, {
          encoding: "utf-8",
        })
      )
    )
  );
  const programKeypairPath = path.join(
    programPath,
    `target/deploy/${programName.replace("-", "_")}-keypair.json`
  );

  let programId: PublicKey;
  switch (cluster) {
    case "localnet":
      programId = new PublicKey(tomlData.programs.localnet[programName]);
      break;
    case "devnet":
      programId = new PublicKey(tomlData.programs.devnet[programName]);
      break;
    case "mainnet-beta":
      programId = new PublicKey(tomlData.programs.mainnet[programName]);
      break;
    default:
      programId = loadPid(programKeypairPath);
  }

  const programIdlPath = path.join(
    programPath,
    `target/idl/${programName.replace("-", "_")}.json`
  );

  const idl: anchor.Idl = JSON.parse(fs.readFileSync(programIdlPath, "utf-8"));
  const url =
    cluster === "localnet" ? "http://localhost:8899" : clusterApiUrl(cluster);
  const provider = new anchor.AnchorProvider(
    new Connection(url, { commitment: "confirmed" }),
    new AnchorWallet(wallet),
    { commitment: "confirmed" }
  );
  return new anchor.Program(idl, programId, provider);
}
