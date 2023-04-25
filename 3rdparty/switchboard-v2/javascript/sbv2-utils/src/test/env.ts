import * as anchor from "@project-serum/anchor";
import { clusterApiUrl, Connection, Keypair, PublicKey } from "@solana/web3.js";
import * as sbv2 from "@switchboard-xyz/switchboard-v2";
import {
  CrankAccount,
  OracleAccount,
  PermissionAccount,
  ProgramStateAccount,
} from "@switchboard-xyz/switchboard-v2";
import chalk from "chalk";
import fs from "fs";
import path from "path";
import { getIdlAddress, getProgramDataAddress } from "../anchor.js";
import { anchorBNtoDateString } from "../date.js";
import { createQueue } from "../queue.js";
import { getOrCreateSwitchboardTokenAccount } from "../token.js";

export const LATEST_DOCKER_VERSION = "dev-v2-07-18-22";

export interface ISwitchboardTestEnvironment {
  programId: PublicKey;
  programDataAddress: PublicKey;
  idlAddress: PublicKey;
  programState: PublicKey;
  switchboardVault: PublicKey;
  switchboardMint: PublicKey;
  tokenWallet: PublicKey;
  queue: PublicKey;
  queueAuthority: PublicKey;
  queueBuffer: PublicKey;
  crank: PublicKey;
  crankBuffer: PublicKey;
  oracle: PublicKey;
  oracleAuthority: PublicKey;
  oracleEscrow: PublicKey;
  oraclePermissions: PublicKey;
  payerKeypairPath: string;

  // allow a map of public keys to include in clone script
  additionalClonedAccounts?: Record<string, PublicKey>;
}

/** Contains all of the necessary devnet Switchboard accounts to clone to localnet */
export class SwitchboardTestEnvironment implements ISwitchboardTestEnvironment {
  programId: PublicKey;

  programDataAddress: PublicKey;

  idlAddress: PublicKey;

  programState: PublicKey;

  switchboardVault: PublicKey;

  switchboardMint: PublicKey;

  tokenWallet: PublicKey;

  queue: PublicKey;

  queueAuthority: PublicKey;

  queueBuffer: PublicKey;

  crank: PublicKey;

  crankBuffer: PublicKey;

  oracle: PublicKey;

  oracleAuthority: PublicKey;

  oracleEscrow: PublicKey;

  oraclePermissions: PublicKey;

  payerKeypairPath: string;

  additionalClonedAccounts: Record<string, PublicKey>;

  constructor(ctx: ISwitchboardTestEnvironment) {
    this.programId = ctx.programId;
    this.programDataAddress = ctx.programDataAddress;
    this.idlAddress = ctx.idlAddress;
    this.programState = ctx.programState;
    this.switchboardVault = ctx.switchboardVault;
    this.switchboardMint = ctx.switchboardMint;
    this.tokenWallet = ctx.tokenWallet;
    this.queue = ctx.queue;
    this.queueAuthority = ctx.queueAuthority;
    this.queueBuffer = ctx.queueBuffer;
    this.crank = ctx.crank;
    this.crankBuffer = ctx.crankBuffer;
    this.oracle = ctx.oracle;
    this.oracleAuthority = ctx.oracleAuthority;
    this.oracleEscrow = ctx.oracleEscrow;
    this.oraclePermissions = ctx.oraclePermissions;
    this.payerKeypairPath = ctx.payerKeypairPath;
    this.additionalClonedAccounts = ctx.additionalClonedAccounts ?? {};
  }

  private getAccountCloneString(): string {
    const accounts = Object.keys(this).map((key) => {
      // iterate over additionalClonedAccounts and collect pubkeys
      if (typeof this[key] === "string") {
        return;
      }
      if (key === "additionalClonedAccounts" && this[key]) {
        const additionalPubkeys = Object.values(this.additionalClonedAccounts);
        const cloneStrings = additionalPubkeys.map(
          (pubkey) => `--clone ${pubkey.toBase58()} \`# ${key}\``
        );
        return cloneStrings.join(`\\\n`);
      }

      return `--clone ${(this[key] as PublicKey).toBase58()} \`# ${key}\` `;
    });

    return accounts.filter(Boolean).join(`\\\n`);
  }

  public toJSON(): ISwitchboardTestEnvironment {
    return {
      programId: this.programId,
      programDataAddress: this.programDataAddress,
      idlAddress: this.idlAddress,
      programState: this.programState,
      switchboardVault: this.switchboardVault,
      switchboardMint: this.switchboardMint,
      tokenWallet: this.tokenWallet,
      queue: this.queue,
      queueAuthority: this.queueAuthority,
      queueBuffer: this.queueBuffer,
      crank: this.crank,
      crankBuffer: this.crankBuffer,
      oracle: this.oracle,
      oracleAuthority: this.oracleAuthority,
      oracleEscrow: this.oracleEscrow,
      oraclePermissions: this.oraclePermissions,
      payerKeypairPath: this.payerKeypairPath,
      additionalClonedAccounts: this.additionalClonedAccounts,
    };
  }

  /** Write switchboard test environment to filesystem */
  public writeAll(outputDir: string): void {
    fs.mkdirSync(outputDir, { recursive: true });
    this.writeEnv(outputDir);
    this.writeJSON(outputDir);
    this.writeScripts(outputDir);
    this.writeDockerCompose(outputDir);
    this.writeAnchorToml(outputDir);
  }

  /** Write the env file to filesystem */
  public writeEnv(filePath: string): void {
    const ENV_FILE_PATH = path.join(filePath, "switchboard.env");
    let fileStr = "";
    fileStr += `SWITCHBOARD_PROGRAM_ID="${this.programId.toBase58()}"\n`;
    fileStr += `SWITCHBOARD_PROGRAM_DATA_ADDRESS="${this.programDataAddress.toBase58()}"\n`;
    fileStr += `SWITCHBOARD_IDL_ADDRESS="${this.idlAddress.toBase58()}"\n`;
    fileStr += `SWITCHBOARD_PROGRAM_STATE="${this.programState.toBase58()}"\n`;
    fileStr += `SWITCHBOARD_VAULT="${this.switchboardVault.toBase58()}"\n`;
    fileStr += `SWITCHBOARD_MINT="${this.switchboardMint.toBase58()}"\n`;
    fileStr += `TOKEN_WALLET="${this.tokenWallet.toBase58()}"\n`;
    fileStr += `ORACLE_QUEUE="${this.queue.toBase58()}"\n`;
    fileStr += `ORACLE_QUEUE_AUTHORITY="${this.queueAuthority.toBase58()}"\n`;
    fileStr += `ORACLE_QUEUE_BUFFER="${this.queueBuffer.toBase58()}"\n`;
    fileStr += `CRANK="${this.crank.toBase58()}"\n`;
    fileStr += `CRANK_BUFFER="${this.crankBuffer.toBase58()}"\n`;
    fileStr += `ORACLE="${this.oracle.toBase58()}"\n`;
    fileStr += `ORACLE_AUTHORITY="${this.oracleAuthority.toBase58()}"\n`;
    fileStr += `ORACLE_ESCROW="${this.oracleEscrow.toBase58()}"\n`;
    fileStr += `ORACLE_PERMISSIONS="${this.oraclePermissions.toBase58()}"\n`;
    // fileStr += `SWITCHBOARD_ACCOUNTS="${this.getAccountCloneString()}"\n`;
    // TODO: Write additionalClonedAccounts to env file
    fs.writeFileSync(ENV_FILE_PATH, fileStr);
    console.log(
      `${chalk.green("Env File saved to:")} ${ENV_FILE_PATH.replace(
        process.cwd(),
        "."
      )}`
    );
  }

  public writeJSON(outputDir: string): void {
    const JSON_FILE_PATH = path.join(outputDir, "switchboard.json");
    fs.writeFileSync(
      JSON_FILE_PATH,
      JSON.stringify(
        this.toJSON(),
        (key, value) => {
          if (value instanceof PublicKey) {
            return value.toBase58();
          }
          return value;
        },
        2
      )
    );
  }

  public writeScripts(outputDir: string): void {
    const LOCAL_VALIDATOR_SCRIPT = path.join(
      outputDir,
      "start-local-validator.sh"
    );
    // create bash script to startup local validator with appropriate accounts cloned
    const baseValidatorCommand = `solana-test-validator -r --ledger .anchor/test-ledger --mint ${this.oracleAuthority.toBase58()} --bind-address 0.0.0.0 --url ${clusterApiUrl(
      "devnet"
    )} --rpc-port 8899 `;
    const cloneAccountsString = this.getAccountCloneString();
    const startValidatorCommand = `${baseValidatorCommand} ${cloneAccountsString}`;
    fs.writeFileSync(
      LOCAL_VALIDATOR_SCRIPT,

      `#!/bin/bash\n\nmkdir -p .anchor/test-ledger\n\n${startValidatorCommand}`
    );
    fs.chmodSync(LOCAL_VALIDATOR_SCRIPT, "755");
    console.log(
      `${chalk.green("Bash script saved to:")} ${LOCAL_VALIDATOR_SCRIPT.replace(
        process.cwd(),
        "."
      )}`
    );

    // create bash script to start local oracle
    const ORACLE_SCRIPT = path.join(outputDir, "start-oracle.sh");
    // const startOracleCommand = `docker-compose -f docker-compose.switchboard.yml up`;
    fs.writeFileSync(
      ORACLE_SCRIPT,
      `#!/usr/bin/env bash

script_dir=$( cd -- "$( dirname -- "\${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

docker-compose -f  "$script_dir"/docker-compose.switchboard.yml up
      `
      // `#!/bin/bash\n\n${startOracleCommand}`
    );
    fs.chmodSync(ORACLE_SCRIPT, "755");
    console.log(
      `${chalk.green("Bash script saved to:")} ${ORACLE_SCRIPT.replace(
        process.cwd(),
        "."
      )}`
    );
  }

  public writeDockerCompose(outputDir: string): void {
    const DOCKER_COMPOSE_FILEPATH = path.join(
      outputDir,
      "docker-compose.switchboard.yml"
    );
    const dockerComposeString = `version: "3.3"
services:
  oracle:
    image: "switchboardlabs/node:\${SBV2_ORACLE_VERSION:-${LATEST_DOCKER_VERSION}}" # https://hub.docker.com/r/switchboardlabs/node/tags
    network_mode: host
    restart: always
    secrets:
      - PAYER_SECRETS
    environment:
      - VERBOSE=1
      - LIVE=1
      - CLUSTER=\${CLUSTER:-localnet}
      - HEARTBEAT_INTERVAL=30 # Seconds
      - ORACLE_KEY=${this.oracle.toBase58()}
    #  - RPC_URL=\${RPC_URL}
secrets:
  PAYER_SECRETS:
    file: ${this.payerKeypairPath}
`;
    fs.writeFileSync(DOCKER_COMPOSE_FILEPATH, dockerComposeString);
    console.log(
      `${chalk.green(
        "Docker-Compose saved to:"
      )} ${DOCKER_COMPOSE_FILEPATH.replace(process.cwd(), ".")}`
    );
  }

  public writeAnchorToml(outputDir: string): void {
    const ANCHOR_TOML_FILEPATH = path.join(
      outputDir,
      "Anchor.switchboard.toml"
    );
    const anchorTomlString = `[provider]
cluster = "localnet"
wallet = "${this.payerKeypairPath}"

[test]
startup_wait = 10000

[test.validator]
url = "https://api.devnet.solana.com"

[[test.validator.clone]] # programID
address = "${this.programId}"

[[test.validator.clone]] # idlAddress
address = "${this.idlAddress}"

[[test.validator.clone]] # programState
address = "${this.programState}"

[[test.validator.clone]] # switchboardVault
address = "${this.switchboardVault}"

[[test.validator.clone]] # tokenWallet
address = "${this.tokenWallet}"

[[test.validator.clone]] # queue
address = "${this.queue}"

[[test.validator.clone]] # queueAuthority
address = "${this.queueAuthority}"

[[test.validator.clone]] # queueBuffer
address = "${this.queueBuffer}"

[[test.validator.clone]] # crank
address = "${this.crank}"

[[test.validator.clone]] # crankBuffer
address = "${this.crankBuffer}"

[[test.validator.clone]] # oracle
address = "${this.oracle}"

[[test.validator.clone]] # oracleAuthority
address = "${this.oracleAuthority}"

[[test.validator.clone]] # oracleEscrow
address = "${this.oracleEscrow}"

[[test.validator.clone]] # oraclePermissions
address = "${this.oraclePermissions}"
`;

    fs.writeFileSync(ANCHOR_TOML_FILEPATH, anchorTomlString);
    console.log(
      `${chalk.green("Anchor.toml saved to:")} ${ANCHOR_TOML_FILEPATH.replace(
        process.cwd(),
        "."
      )}`
    );
  }

  /** Build a devnet environment to later clone to localnet */
  static async create(
    payerKeypairPath: string,
    additionalClonedAccounts?: Record<string, PublicKey>,
    alternateProgramId?: PublicKey
  ): Promise<SwitchboardTestEnvironment> {
    const fullKeypairPath =
      payerKeypairPath.charAt(0) === "/"
        ? payerKeypairPath
        : path.join(process.cwd(), payerKeypairPath);
    if (!fs.existsSync(fullKeypairPath)) {
      throw new Error("Failed to find payer keypair path");
    }
    const payerKeypair = Keypair.fromSecretKey(
      new Uint8Array(
        JSON.parse(
          fs.readFileSync(fullKeypairPath, {
            encoding: "utf-8",
          })
        )
      )
    );
    const connection = new Connection(clusterApiUrl("devnet"), {
      commitment: "confirmed",
    });

    const programId = alternateProgramId ?? sbv2.getSwitchboardPid("devnet");
    const wallet = new sbv2.AnchorWallet(payerKeypair);
    const provider = new anchor.AnchorProvider(connection, wallet, {});

    const anchorIdl = await anchor.Program.fetchIdl(programId, provider);
    if (!anchorIdl) {
      throw new Error(`failed to read idl for ${programId}`);
    }

    const switchboardProgram = new anchor.Program(
      anchorIdl,
      programId,
      provider
    );

    const programDataAddress = getProgramDataAddress(
      switchboardProgram.programId
    );
    const idlAddress = await getIdlAddress(switchboardProgram.programId);

    const queueResponse = await createQueue(
      switchboardProgram,
      {
        authority: payerKeypair.publicKey,
        name: "Test Queue",
        metadata: `created ${anchorBNtoDateString(
          new anchor.BN(Math.floor(Date.now() / 1000))
        )}`,
        minStake: new anchor.BN(0),
        reward: new anchor.BN(0),
        crankSize: 10,
        oracleTimeout: 180,
        numOracles: 1,
        unpermissionedFeeds: true,
        unpermissionedVrf: true,
        enableBufferRelayers: true,
      },
      10
    );

    const queueAccount = queueResponse.queueAccount;
    const queue = await queueAccount.loadData();

    const [programStateAccount, stateBump] =
      ProgramStateAccount.fromSeed(switchboardProgram);
    const programState = await programStateAccount.loadData();

    const mint = await queueAccount.loadMint();

    const payerSwitchboardWallet = await getOrCreateSwitchboardTokenAccount(
      switchboardProgram,
      mint
    );

    const crankAccount = new CrankAccount({
      program: switchboardProgram,
      publicKey: queueResponse.crankPubkey,
    });
    const crank = await crankAccount.loadData();

    const oracleAccount = new OracleAccount({
      program: switchboardProgram,
      publicKey: queueResponse.oracles[0],
    });
    const oracle = await oracleAccount.loadData();

    const [permissionAccount] = PermissionAccount.fromSeed(
      switchboardProgram,
      queue.authority,
      queueAccount.publicKey,
      oracleAccount.publicKey
    );
    const permission = await permissionAccount.loadData();

    const ctx: ISwitchboardTestEnvironment = {
      programId: switchboardProgram.programId,
      programDataAddress,
      idlAddress,
      programState: programStateAccount.publicKey,
      switchboardVault: programState.tokenVault,
      switchboardMint: mint.address,
      tokenWallet: payerSwitchboardWallet,
      queue: queueResponse.queueAccount.publicKey,
      queueAuthority: queue.authority,
      queueBuffer: queue.dataBuffer,
      crank: crankAccount.publicKey,
      crankBuffer: crank.dataBuffer,
      oracle: oracleAccount.publicKey,
      oracleAuthority: oracle.oracleAuthority,
      oracleEscrow: oracle.tokenAccount,
      oraclePermissions: permissionAccount.publicKey,
      payerKeypairPath: fullKeypairPath,
      additionalClonedAccounts,
    };

    return new SwitchboardTestEnvironment(ctx);
  }
}
