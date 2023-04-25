import * as anchor from "@project-serum/anchor";
import { ACCOUNT_DISCRIMINATOR_SIZE } from "@project-serum/anchor";
import type { PublicKey } from "@solana/web3.js";
import {
  AggregatorAccount,
  BufferRelayerAccount,
  CrankAccount,
  JobAccount,
  LeaseAccount,
  OracleAccount,
  OracleQueueAccount,
  PermissionAccount,
  ProgramStateAccount,
  VrfAccount,
} from "@switchboard-xyz/switchboard-v2";
import { InvalidSwitchboardAccount } from "./errors.js";

export const SWITCHBOARD_ACCOUNT_TYPES = [
  JobAccount.accountName,
  AggregatorAccount.accountName,
  OracleAccount.accountName,
  OracleQueueAccount.accountName,
  PermissionAccount.accountName,
  LeaseAccount.accountName,
  ProgramStateAccount.accountName,
  VrfAccount.accountName,
  CrankAccount.accountName,
  BufferRelayerAccount.accountName,
  "SbState",
  "BUFFERxx",
] as const;

export type SwitchboardAccount =
  | JobAccount
  | AggregatorAccount
  | OracleAccount
  | OracleQueueAccount
  | PermissionAccount
  | LeaseAccount
  | ProgramStateAccount
  | VrfAccount
  | CrankAccount
  | BufferRelayerAccount;

export type SwitchboardAccountType = typeof SWITCHBOARD_ACCOUNT_TYPES[number];

export const SWITCHBOARD_DISCRIMINATOR_MAP = new Map<
  SwitchboardAccountType,
  Buffer
>(
  SWITCHBOARD_ACCOUNT_TYPES.map((accountType) => [
    accountType,
    anchor.BorshAccountsCoder.accountDiscriminator(accountType),
  ])
);

// should also check if pubkey is a token account
export const findAccountType = async (
  program: anchor.Program,
  publicKey: PublicKey
): Promise<SwitchboardAccountType> => {
  const account = await program.provider.connection.getAccountInfo(publicKey);
  if (!account) {
    throw new Error(`failed to fetch account info for ${publicKey}`);
  }

  const accountDiscriminator = account.data.slice(
    0,
    ACCOUNT_DISCRIMINATOR_SIZE
  );

  for (const [name, discriminator] of SWITCHBOARD_DISCRIMINATOR_MAP.entries()) {
    if (Buffer.compare(accountDiscriminator, discriminator) === 0) {
      return name;
    }
  }

  throw new InvalidSwitchboardAccount();
};

export const loadSwitchboardAccount = async (
  program: anchor.Program,
  publicKey: PublicKey
): Promise<[SwitchboardAccountType, SwitchboardAccount]> => {
  const accountType = await findAccountType(program, publicKey);
  switch (accountType) {
    case JobAccount.accountName: {
      return [accountType, new JobAccount({ program, publicKey })];
    }
    case AggregatorAccount.accountName: {
      return [accountType, new AggregatorAccount({ program, publicKey })];
    }
    case OracleAccount.accountName: {
      return [accountType, new OracleAccount({ program, publicKey })];
    }
    case PermissionAccount.accountName: {
      return [accountType, new PermissionAccount({ program, publicKey })];
    }
    case LeaseAccount.accountName: {
      return [accountType, new LeaseAccount({ program, publicKey })];
    }
    case OracleQueueAccount.accountName: {
      return [accountType, new OracleQueueAccount({ program, publicKey })];
    }
    case CrankAccount.accountName: {
      return [accountType, new CrankAccount({ program, publicKey })];
    }
    case "SbState":
    case ProgramStateAccount.accountName: {
      return [accountType, new ProgramStateAccount({ program, publicKey })];
    }
    case VrfAccount.accountName: {
      return [accountType, new VrfAccount({ program, publicKey })];
    }
    case BufferRelayerAccount.accountName: {
      return [accountType, new BufferRelayerAccount({ program, publicKey })];
    }
  }

  throw new InvalidSwitchboardAccount();
};
