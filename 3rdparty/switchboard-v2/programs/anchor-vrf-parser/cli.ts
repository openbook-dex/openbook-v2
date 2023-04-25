#!/usr/bin/env ts-node-esm
/* eslint-disable @typescript-eslint/no-loop-func */
/* eslint-disable @typescript-eslint/no-unused-expressions */
/* eslint-disable @typescript-eslint/no-var-requires */
import * as anchor from "@project-serum/anchor";
import * as spl from "@solana/spl-token-v2";
import {
  clusterApiUrl,
  Connection,
  ParsedTransactionWithMeta,
  PublicKey,
} from "@solana/web3.js";
import * as sbv2Utils from "@switchboard-xyz/sbv2-utils";
import { toVrfStatusString } from "@switchboard-xyz/sbv2-utils";
import * as sbv2 from "@switchboard-xyz/switchboard-v2";
import chalk from "chalk";
const yargs = require("yargs");
const { hideBin } = require("yargs/helpers");
// import yargs from "yargs";
// import { hideBin } from "yargs/helpers";
import fs from "fs";
import path from "path";
import { AnchorVrfParser, IDL } from "./target/types/anchor_vrf_parser";
import { VrfClient } from "./client/accounts";
import { PROGRAM_ID } from "./client/programId";

// const DEFAULT_MAINNET_RPC = "https://ssc-dao.genesysgo.net";
// const DEFAULT_DEVNET_RPC = "https://devnet.genesysgo.net";
const DEFAULT_LOCALNET_RPC = "http://localhost:8899";

const DEFAULT_COMMITMENT = "confirmed";

const VRF_REQUEST_AMOUNT = BigInt(2_000_000);

interface RequestRandomnessResult {
  success: boolean;
  status: string;
  counter: number;
  txRemaining: number;
  producer: string;
  alpha: string;
  alphaHex: string;
  proof?: string;
  proofHex?: string;
  proofBase64?: string;
  result: string;
  stage?: number;
  txs?: ParsedTransactionWithMeta[] | string[] | any[];
}

yargs(hideBin(process.argv))
  .scriptName("sbv2-vrf-example")
  .command(
    "create [queueKey]",
    "create a VRF client",
    (y: any) => {
      return y
        .positional("queueKey", {
          type: "string",
          describe: "publicKey of the oracle queue to create a VRF for",
          default: "F8ce7MsckeZAbAGmxjJNetxYXQa9mKr9nnrC3qKubyYy",
        })
        .option("maxResult", {
          description: "test",
          type: "number",
        });
    },
    async function (argv: any) {
      const { queueKey, rpcUrl, cluster, maxResult } = argv;

      const { vrfClientProgram, switchboardProgram, payer, provider } =
        await loadCli(rpcUrl, cluster);

      const vrfSecret = anchor.web3.Keypair.generate();

      const [vrfClientKey, vrfClientBump] =
        anchor.utils.publicKey.findProgramAddressSync(
          [
            Buffer.from("STATE"),
            vrfSecret.publicKey.toBytes(),
            payer.publicKey.toBytes(),
          ],
          vrfClientProgram.programId
        );

      const vrfIxCoder = new anchor.BorshInstructionCoder(vrfClientProgram.idl);
      const vrfClientCallback = {
        programId: vrfClientProgram.programId,
        accounts: [
          // ensure all accounts in updateResult are populated
          { pubkey: vrfClientKey, isSigner: false, isWritable: true },
          { pubkey: vrfSecret.publicKey, isSigner: false, isWritable: false },
        ],
        ixData: vrfIxCoder.encode("updateResult", ""), // pass any params for instruction here
      };

      const queueAccount = new sbv2.OracleQueueAccount({
        program: switchboardProgram,
        publicKey: new anchor.web3.PublicKey(queueKey),
      });
      const { unpermissionedVrfEnabled, authority, dataBuffer } =
        await queueAccount.loadData();

      // Create Switchboard VRF and Permission account
      const vrfAccount = await sbv2.VrfAccount.create(switchboardProgram, {
        queue: queueAccount,
        callback: vrfClientCallback,
        authority: vrfClientKey, // vrf authority
        keypair: vrfSecret,
      });
      await sbv2Utils.sleep(2000);
      const { escrow } = await vrfAccount.loadData();
      console.log(sbv2Utils.chalkString("VRF Account", vrfAccount.publicKey));

      const permissionAccount = await sbv2.PermissionAccount.create(
        switchboardProgram,
        {
          authority,
          granter: queueAccount.publicKey,
          grantee: vrfAccount.publicKey,
        }
      );
      // console.log(`Created Permission Account: ${permissionAccount.publicKey}`);

      // If queue requires permissions to use VRF, check the correct authority was provided
      if (!unpermissionedVrfEnabled) {
        if (!payer.publicKey.equals(authority)) {
          throw new Error(
            `queue requires PERMIT_VRF_REQUESTS and wrong queue authority provided`
          );
        }

        await permissionAccount.set({
          authority: payer,
          permission: sbv2.SwitchboardPermission.PERMIT_VRF_REQUESTS,
          enable: true,
        });
      }

      // Create VRF Client account
      await vrfClientProgram.methods
        .initState({
          maxResult: new anchor.BN(maxResult),
        })
        .accounts({
          state: vrfClientKey,
          vrf: vrfAccount.publicKey,
          payer: payer.publicKey,
          authority: payer.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      console.log(sbv2Utils.chalkString("VRF Client", vrfClientKey));

      console.log(`\nRun the following command to request randomness`);
      console.log(
        `\t${chalk.yellow(
          "npx sbv2-vrf-example request",
          vrfClientKey.toString()
        )}`
      );
    }
  )
  .command(
    "request [vrfClientKey]",
    "request randomness for a VRF client",
    (y: any) => {
      return y.positional("vrfClientKey", {
        type: "string",
        describe: "publicKey of the VRF client to request randomness for",
        demand: true,
      });
    },
    async function (argv: any) {
      const { vrfClientKey, rpcUrl, cluster } = argv;
      if (!vrfClientKey) {
        throw new Error(`Must provide vrfClientKey arguement`);
      }

      const { vrfClientProgram, switchboardProgram, payer, provider } =
        await loadCli((rpcUrl as string) ?? "", (cluster as string) ?? "");

      const vrfClient = await VrfClient.fetch(
        provider.connection,
        vrfClientProgram.programId,
        new anchor.web3.PublicKey(vrfClientKey)
      );
      if (!vrfClient) {
        throw new Error(
          `Failed to fetch VrfClient for account ${vrfClientKey}`
        );
      }

      const vrfAccount = new sbv2.VrfAccount({
        program: switchboardProgram,
        publicKey: vrfClient.vrf,
      });
      const vrfData = await vrfAccount.loadData();

      const vrfEscrow = await spl.getAccount(
        provider.connection,
        vrfData.escrow,
        DEFAULT_COMMITMENT,
        spl.TOKEN_PROGRAM_ID
      );

      const queueAccount = new sbv2.OracleQueueAccount({
        program: switchboardProgram,
        publicKey: vrfData.oracleQueue,
      });
      const queueData = await queueAccount.loadData();
      const mint = await queueAccount.loadMint();

      const payerTokenAccount = await spl.getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        mint.address,
        payer.publicKey,
        undefined,
        undefined,
        undefined,
        spl.TOKEN_PROGRAM_ID,
        spl.ASSOCIATED_TOKEN_PROGRAM_ID
      );

      const tokensNeeded = VRF_REQUEST_AMOUNT - vrfEscrow.amount;

      if (tokensNeeded > 0 && payerTokenAccount.amount < tokensNeeded) {
        throw new Error(
          `Payer token account has insufficient funds, need 2_000_000, current ${payerTokenAccount.amount}`
        );
      }

      const [programStateAccount, programStateBump] =
        sbv2.ProgramStateAccount.fromSeed(switchboardProgram);

      const [permissionAccount, permissionBump] =
        sbv2.PermissionAccount.fromSeed(
          switchboardProgram,
          queueData.authority,
          queueAccount.publicKey,
          vrfAccount.publicKey
        );

      let ws: number | undefined = undefined;
      const waitForResultPromise = new Promise(
        (
          resolve: (result: anchor.BN) => void,
          reject: (reason: string) => void
        ) => {
          ws = vrfClientProgram.provider.connection.onAccountChange(
            new anchor.web3.PublicKey(vrfClientKey),
            async (
              accountInfo: anchor.web3.AccountInfo<Buffer>,
              context: anchor.web3.Context
            ) => {
              const clientState = VrfClient.decode(accountInfo.data);
              if (clientState.result.gt(new anchor.BN(0))) {
                resolve(clientState.result);
              }
            }
          );
        }
      ).then(async (result) => {
        console.log(
          sbv2Utils.chalkString("Client Result", result.toString(10))
        );
        if (ws) {
          await vrfClientProgram.provider.connection.removeAccountChangeListener(
            ws
          );
        }
        ws = undefined;
        return result;
      });

      const requestTxnPromise = vrfClientProgram.methods
        .requestResult({
          switchboardStateBump: programStateBump,
          permissionBump,
        })
        .accounts({
          state: vrfClientKey,
          authority: payer.publicKey,
          switchboardProgram: switchboardProgram.programId,
          vrf: vrfAccount.publicKey,
          oracleQueue: queueAccount.publicKey,
          queueAuthority: queueData.authority,
          dataBuffer: queueData.dataBuffer,
          permission: permissionAccount.publicKey,
          escrow: vrfData.escrow,
          payerWallet: payerTokenAccount.address,
          payerAuthority: payer.publicKey,
          recentBlockhashes: anchor.web3.SYSVAR_RECENT_BLOCKHASHES_PUBKEY,
          programState: programStateAccount.publicKey,
          tokenProgram: spl.TOKEN_PROGRAM_ID,
        })
        .signers([payer, payer])
        .rpc()
        .then((txnSignature) => {
          console.log(
            `${chalk.yellow("Randomness Requested")}\t${txnSignature}`
          );
        });

      let result: anchor.BN;
      try {
        result = await sbv2Utils.promiseWithTimeout(
          45_000,
          waitForResultPromise,
          new Error(`Timed out waiting for VRF Client callback`)
        );
      } catch (error) {
        throw error;
      } finally {
        if (ws) {
          await vrfClientProgram.provider.connection.removeAccountChangeListener(
            ws
          );
        }
      }

      process.exit(0);
    }
  )
  .command(
    "loop [vrfClientKey] [numLoops]",
    "request randomness for a VRF client N times and record the results",
    (y: any) => {
      return y
        .positional("vrfClientKey", {
          type: "string",
          describe: "publicKey of the VRF client to request randomness for",
          demand: true,
        })
        .positional("numLoops", {
          type: "number",
          describe: "number of times to request randomness",
          default: 100,
        });
    },
    async function (argv: any) {
      const { vrfClientKey, rpcUrl, cluster, numLoops } = argv;
      if (!vrfClientKey) {
        throw new Error(`Must provide vrfClientKey arguement`);
      }

      const startTime = Math.floor(Date.now() / 1000);

      const vrfClientPubkey = new anchor.web3.PublicKey(vrfClientKey);

      const { vrfClientProgram, switchboardProgram, payer, provider } =
        await loadCli(rpcUrl as string, cluster as string);

      const vrfClient = await VrfClient.fetch(
        provider.connection,
        vrfClientProgram.programId,
        vrfClientPubkey
      );
      if (!vrfClient) {
        throw new Error(
          `Failed to fetch VrfClient for account ${vrfClientKey}`
        );
      }

      const vrfAccount = new sbv2.VrfAccount({
        program: switchboardProgram,
        publicKey: vrfClient.vrf,
      });
      const vrfData = await vrfAccount.loadData();

      const vrfEscrow = await spl.getAccount(
        provider.connection,
        vrfData.escrow,
        DEFAULT_COMMITMENT,
        spl.TOKEN_PROGRAM_ID
      );

      const queueAccount = new sbv2.OracleQueueAccount({
        program: switchboardProgram,
        publicKey: vrfData.oracleQueue,
      });
      const queueData = await queueAccount.loadData();
      const mint = await queueAccount.loadMint();

      const payerTokenAccount = await spl.getOrCreateAssociatedTokenAccount(
        provider.connection,
        payer,
        mint.address,
        payer.publicKey,
        undefined,
        undefined,
        undefined,
        spl.TOKEN_PROGRAM_ID,
        spl.ASSOCIATED_TOKEN_PROGRAM_ID
      );

      const tokensNeeded = VRF_REQUEST_AMOUNT - vrfEscrow.amount;

      if (tokensNeeded > 0 && payerTokenAccount.amount < tokensNeeded) {
        throw new Error(
          `Payer token account has insufficient funds, need 2_000_000, current ${payerTokenAccount.amount}`
        );
      }

      const [programStateAccount, programStateBump] =
        sbv2.ProgramStateAccount.fromSeed(switchboardProgram);

      const [permissionAccount, permissionBump] =
        sbv2.PermissionAccount.fromSeed(
          switchboardProgram,
          queueData.authority,
          queueAccount.publicKey,
          vrfAccount.publicKey
        );

      const requestInstruction = await vrfClientProgram.methods
        .requestResult({
          switchboardStateBump: programStateBump,
          permissionBump,
        })
        .accounts({
          state: vrfClientPubkey,
          authority: payer.publicKey,
          switchboardProgram: switchboardProgram.programId,
          vrf: vrfAccount.publicKey,
          oracleQueue: queueAccount.publicKey,
          queueAuthority: queueData.authority,
          dataBuffer: queueData.dataBuffer,
          permission: permissionAccount.publicKey,
          escrow: vrfData.escrow,
          payerWallet: payerTokenAccount.address,
          payerAuthority: payer.publicKey,
          recentBlockhashes: anchor.web3.SYSVAR_RECENT_BLOCKHASHES_PUBKEY,
          programState: programStateAccount.publicKey,
          tokenProgram: spl.TOKEN_PROGRAM_ID,
        })
        .signers([payer])
        .instruction();

      const results: RequestRandomnessResult[] = [];
      let successes = 0;
      let failures = 0;
      try {
        for await (const i of Array.from(Array(numLoops).keys())) {
          let success = false;
          try {
            while (true) {
              try {
                await provider.sendAndConfirm(
                  new anchor.web3.Transaction().add(requestInstruction),
                  [payer]
                );
                // console.log("sent");
                break;
              } catch (error) {
                if (!error.message.includes("0x17b5")) {
                  throw error;
                }
                await sbv2Utils.sleep(2500);
              }
            }

            await awaitCallback(provider.connection, vrfClientPubkey, 90_000)
              .then(() => {
                success = true;
                successes++;

                clearStatus();
                writeStatus(i, numLoops, successes, failures);
              })
              .catch(async (error) => {
                failures++;
                const vrf = await vrfAccount.loadData();

                clearStatus();
                console.error(error.message);
                writeVrfState(vrf);
                writeStatus(i, numLoops, successes, failures);
              });
          } catch (error) {}

          const vrf = await vrfAccount.loadData();
          const vrfStatus = sbv2Utils.toVrfStatusString(vrf.status);
          const result: RequestRandomnessResult = {
            success: vrfStatus === "StatusCallbackSuccess",
            counter: vrf.counter.toString(10),
            producer: vrf.builders[0].producer.toString(),
            status: vrfStatus,
            txRemaining: vrf.builders[0].txRemaining,
            alpha: `[${vrf.currentRound.alpha.slice(0, 32).toString()}]`,
            alphaHex: Buffer.from(vrf.currentRound.alpha.slice(0, 32)).toString(
              "hex"
            ),
            proof: `[${vrf.builders[0].reprProof.slice(0, 80).toString()}]`,
            proofHex: Buffer.from(
              vrf.builders[0].reprProof.slice(0, 80)
            ).toString("hex"),
            proofBase64: Buffer.from(
              vrf.builders[0].reprProof.slice(0, 80)
            ).toString("base64"),
            result: `[${vrf.currentRound.result.toString()}]`,
            stage: vrf.builders[0].stage,
            txs:
              vrfStatus === "StatusCallbackSuccess"
                ? undefined
                : await fetchTransactions(
                    vrfAccount.program.provider.connection,
                    vrfAccount.publicKey,
                    15
                  ),
          };

          results.push(result);
          saveResults(startTime, vrfClientKey, results);
          clearStatus();
          writeStatus(i, numLoops, successes, failures);
        }
      } catch (error) {
        console.error(`GLOBAL: ${error}`);
      }

      writeStatus(numLoops, numLoops, successes, failures);

      saveResults(startTime, vrfClientKey, results);

      process.exit(0);
    }
  )
  .options({
    cluster: {
      type: "string",
      alias: "c",
      describe: "Solana cluster to interact with",
      options: ["devnet", "mainnet-beta", "localnet"],
      default: "devnet",
      demand: false,
    },
    rpcUrl: {
      type: "string",
      alias: "u",
      describe: "Alternative RPC URL",
    },
  })
  .help().argv;

function getRpcUrl(cluster: string): string {
  switch (cluster) {
    case "mainnet-beta":
      return clusterApiUrl("mainnet-beta");
    case "devnet":
      return clusterApiUrl("devnet");
    case "localnet":
      return DEFAULT_LOCALNET_RPC;
    default:
      throw new Error(`Failed to find RPC_URL for cluster ${cluster}`);
  }
}

function writeVrfState(vrf: any) {
  console.log(`Status: ${toVrfStatusString(vrf.builders[0]?.status) ?? ""}`);
  console.log(`TxRemaining: ${vrf.builders[0]?.txRemaining ?? ""}`);
  console.log(
    `Alpha: [${vrf.currentRound.alpha
      .slice(0, 32)
      .map((value) => value.toString())}]`
  );
  console.log(
    `AlphaHex: ${Buffer.from(vrf.currentRound.alpha.slice(0, 32)).toString(
      "hex"
    )}`
  );
  console.log(
    `Proof: [${vrf.builders[0].reprProof
      .slice(0, 80)
      .map((value) => value.toString())}]`
  );
  console.log(
    `ProofHex: ${Buffer.from(vrf.builders[0].reprProof.slice(0, 80)).toString(
      "hex"
    )}`
  );
  console.log(
    `ProofBase64: ${Buffer.from(
      vrf.builders[0].reprProof.slice(0, 80)
    ).toString("base64")}`
  );
  console.log(`Stage: ${vrf.builders[0].stage}`);
}

function saveResults(
  timestamp: number,
  vrfClientKey: anchor.web3.PublicKey,
  results: RequestRandomnessResult[]
) {
  if (results.length) {
    fs.writeFileSync(
      path.join(process.cwd(), `${vrfClientKey}_${timestamp}.json`),
      JSON.stringify(results, undefined, 2),
      "utf-8"
    );
  }

  const errorResults = results.filter((result) => !result.success);

  if (errorResults.length) {
    fs.writeFileSync(
      path.join(process.cwd(), `${vrfClientKey}_ERRORS_${timestamp}.json`),
      JSON.stringify(errorResults, undefined, 2),
      "utf-8"
    );
  }
}

async function loadCli(
  rpcUrl: string,
  cluster: string
): Promise<{
  vrfClientProgram: anchor.Program<AnchorVrfParser>;
  switchboardProgram: anchor.Program;
  payer: anchor.web3.Keypair;
  provider: anchor.AnchorProvider;
}> {
  if (cluster !== "mainnet-beta" && cluster !== "devnet") {
    throw new Error(
      `cluster must be mainnet-beta or devnet, cluster = ${cluster}`
    );
  }

  process.env.ANCHOR_WALLET = sbv2Utils.getAnchorWalletPath();
  const url = rpcUrl || getRpcUrl(cluster);
  const envProvider = anchor.AnchorProvider.local(url);
  const provider = new anchor.AnchorProvider(
    new anchor.web3.Connection(url, {
      commitment: DEFAULT_COMMITMENT,
    }),
    envProvider.wallet,
    {
      commitment: DEFAULT_COMMITMENT,
    }
  );

  const switchboardProgram = await sbv2.loadSwitchboardProgram(
    cluster,
    provider.connection,
    (provider.wallet as sbv2.AnchorWallet).payer,
    {
      commitment: DEFAULT_COMMITMENT,
    }
  );
  const payer = sbv2.programWallet(switchboardProgram);

  // load VRF Client program
  const vrfClientProgram = new anchor.Program(
    IDL,
    PROGRAM_ID,
    provider,
    new anchor.BorshCoder(IDL)
  );

  return {
    vrfClientProgram,
    switchboardProgram,
    payer,
    provider,
  };
}

async function awaitCallback(
  connection: anchor.web3.Connection,
  vrfClientKey: anchor.web3.PublicKey,
  timeoutInterval: number,
  errorMsg = "Timed out waiting for VRF Client callback"
) {
  let ws: number | undefined = undefined;
  const result: anchor.BN = await sbv2Utils
    .promiseWithTimeout(
      timeoutInterval,
      new Promise(
        (
          resolve: (result: anchor.BN) => void,
          reject: (reason: string) => void
        ) => {
          ws = connection.onAccountChange(
            vrfClientKey,
            async (
              accountInfo: anchor.web3.AccountInfo<Buffer>,
              context: anchor.web3.Context
            ) => {
              const clientState = VrfClient.decode(accountInfo.data);
              if (clientState.result.gt(new anchor.BN(0))) {
                resolve(clientState.result);
              }
            }
          );
        }
      ),
      new Error(errorMsg)
    )
    .finally(async () => {
      if (ws) {
        await connection.removeAccountChangeListener(ws);
      }
      ws = undefined;
    });

  return result;
}

const clearLastLine = () => {
  process.stdout.moveCursor(0, -1); // up one line
  process.stdout.clearLine(1); // from cursor to end
};

function writeStatus(
  i: number,
  numLoops: number,
  successes: number,
  failures: number
) {
  process.stdout.write(`# ${i} / ${numLoops}\n`);
  process.stdout.write(`${chalk.green("Success:", successes)} / ${i}\n`);
  process.stdout.write(`${chalk.red("Errors: ", failures)} / ${i}\n`);
}

function clearStatus() {
  process.stdout.moveCursor(0, -1); // up one line
  process.stdout.clearLine(1); // from cursor to end
  process.stdout.moveCursor(0, -1); // up one line
  process.stdout.clearLine(1); // from cursor to end
  process.stdout.moveCursor(0, -1); // up one line
  process.stdout.clearLine(1); // from cursor to end
}

async function fetchTransactions(
  connection: Connection,
  pubkey: PublicKey,
  numTransactions = 10
): Promise<any[]> {
  const signatures = (
    await connection.getSignaturesForAddress(
      pubkey,
      { limit: numTransactions },
      "confirmed"
    )
  ).map((t) => t.signature);

  console.log(`FETCHED ${signatures.length} transactions`);

  let parsedTxns: ParsedTransactionWithMeta[] | null = null;
  while (!parsedTxns) {
    parsedTxns = await connection.getParsedTransactions(
      signatures,
      "confirmed"
    );

    if (!parsedTxns || parsedTxns.length !== signatures.length) {
      await sbv2Utils.sleep(1000);
    }
  }

  return parsedTxns.map((tx, i) => {
    return {
      signature: signatures[i],
      logs: tx.meta.logMessages,
    };
  });
}
