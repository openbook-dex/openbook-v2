import * as anchor from "@project-serum/anchor";
import * as spl from "@solana/spl-token-v2";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  TransactionInstruction,
} from "@solana/web3.js";
import {
  CrankAccount,
  OracleAccount,
  OracleQueueAccount,
  PermissionAccount,
  ProgramStateAccount,
  programWallet,
  SwitchboardDecimal,
} from "@switchboard-xyz/switchboard-v2";
import Big from "big.js";
import { chalkString } from "./print.js";
import { packAndSend } from "./transaction.js";

export interface CreateQueueParams {
  authority?: PublicKey;
  name?: string;
  metadata?: string;
  minStake: anchor.BN;
  reward: anchor.BN;
  crankSize?: number;
  oracleTimeout?: number;
  numOracles?: number;
  unpermissionedFeeds?: boolean;
  unpermissionedVrf?: boolean;
  enableBufferRelayers?: boolean;
}

export interface CreateQueueResponse {
  queueAccount: OracleQueueAccount;
  crankPubkey: PublicKey;
  oracles: PublicKey[];
}

export async function createQueue(
  program: anchor.Program,
  params: CreateQueueParams,
  queueSize = 500,
  authorityKeypair = programWallet(program)
): Promise<CreateQueueResponse> {
  const payerKeypair = programWallet(program);

  const [programStateAccount, stateBump] =
    ProgramStateAccount.fromSeed(program);
  const mint = await spl.getMint(
    program.provider.connection,
    spl.NATIVE_MINT,
    undefined,
    spl.TOKEN_PROGRAM_ID
  );

  const ixns: (TransactionInstruction | TransactionInstruction[])[] = [];
  const signers: Keypair[] = [payerKeypair, authorityKeypair];

  try {
    await programStateAccount.loadData();
  } catch {
    const vaultKeypair = anchor.web3.Keypair.generate();
    ixns.push([
      SystemProgram.createAccount({
        fromPubkey: payerKeypair.publicKey,
        newAccountPubkey: vaultKeypair.publicKey,
        lamports:
          await program.provider.connection.getMinimumBalanceForRentExemption(
            spl.AccountLayout.span
          ),
        space: spl.AccountLayout.span,
        programId: spl.TOKEN_PROGRAM_ID,
      }),
      spl.createInitializeAccountInstruction(
        vaultKeypair.publicKey,
        mint.address,
        payerKeypair.publicKey,
        spl.TOKEN_PROGRAM_ID
      ),
      await program.methods
        .programInit({
          stateBump,
        })
        .accounts({
          state: programStateAccount.publicKey,
          authority: payerKeypair.publicKey,
          tokenMint: mint.address,
          vault: vaultKeypair.publicKey,
          payer: payerKeypair.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: spl.TOKEN_PROGRAM_ID,
          daoMint: mint.address,
        })
        .instruction(),
    ]);
    signers.push(vaultKeypair);
  }

  const queueKeypair = anchor.web3.Keypair.generate();
  const queueBuffer = anchor.web3.Keypair.generate();
  const queueBufferSize = queueSize * 32 + 8;

  const queueAccount = new OracleQueueAccount({
    program: program,
    publicKey: queueKeypair.publicKey,
  });

  console.debug(chalkString("OracleQueue", queueKeypair.publicKey));
  console.debug(chalkString("OracleBuffer", queueBuffer.publicKey));

  const crankKeypair = anchor.web3.Keypair.generate();
  const crankBuffer = anchor.web3.Keypair.generate();
  const crankSize = params.crankSize ? params.crankSize * 40 + 8 : 0;

  console.debug(chalkString("CrankAccount", crankKeypair.publicKey));
  console.debug(chalkString("CrankBuffer", crankBuffer.publicKey));

  const crankAccount = new CrankAccount({
    program: program,
    publicKey: crankKeypair.publicKey,
  });

  ixns.push(
    anchor.web3.SystemProgram.createAccount({
      fromPubkey: payerKeypair.publicKey,
      newAccountPubkey: queueBuffer.publicKey,
      space: queueBufferSize,
      lamports:
        await program.provider.connection.getMinimumBalanceForRentExemption(
          queueBufferSize
        ),
      programId: program.programId,
    }),
    await program.methods
      .oracleQueueInit({
        name: Buffer.from(params.name ?? "").slice(0, 32),
        metadata: Buffer.from("").slice(0, 64),
        reward: params.reward ? new anchor.BN(params.reward) : new anchor.BN(0),
        minStake: params.minStake
          ? new anchor.BN(params.minStake)
          : new anchor.BN(0),
        // feedProbationPeriod: 0,
        oracleTimeout: params.oracleTimeout,
        slashingEnabled: false,
        varianceToleranceMultiplier: SwitchboardDecimal.fromBig(new Big(2)),
        authority: authorityKeypair.publicKey,
        // consecutiveFeedFailureLimit: new anchor.BN(1000),
        // consecutiveOracleFailureLimit: new anchor.BN(1000),
        minimumDelaySeconds: 5,
        queueSize: queueSize,
        unpermissionedFeeds: params.unpermissionedFeeds ?? false,
        unpermissionedVrf: params.unpermissionedVrf ?? false,
        enableBufferRelayers: params.enableBufferRelayers ?? false,
      })
      .accounts({
        oracleQueue: queueKeypair.publicKey,
        authority: authorityKeypair.publicKey,
        buffer: queueBuffer.publicKey,
        systemProgram: SystemProgram.programId,
        payer: payerKeypair.publicKey,
        mint: mint.address,
      })
      .instruction(),
    anchor.web3.SystemProgram.createAccount({
      fromPubkey: payerKeypair.publicKey,
      newAccountPubkey: crankBuffer.publicKey,
      space: crankSize,
      lamports:
        await program.provider.connection.getMinimumBalanceForRentExemption(
          crankSize
        ),
      programId: program.programId,
    }),
    await program.methods
      .crankInit({
        name: Buffer.from("Crank").slice(0, 32),
        metadata: Buffer.from("").slice(0, 64),
        crankSize: params.crankSize,
      })
      .accounts({
        crank: crankKeypair.publicKey,
        queue: queueKeypair.publicKey,
        buffer: crankBuffer.publicKey,
        systemProgram: SystemProgram.programId,
        payer: payerKeypair.publicKey,
      })
      .instruction()
  );
  signers.push(queueKeypair, queueBuffer, crankKeypair, crankBuffer);

  const finalTransactions: (
    | TransactionInstruction
    | TransactionInstruction[]
  )[] = [];

  const oracleAccounts = await Promise.all(
    Array.from(Array(params.numOracles).keys()).map(async (n) => {
      const name = `Oracle-${n + 1}`;
      const tokenWalletKeypair = anchor.web3.Keypair.generate();
      const [oracleAccount, oracleBump] = OracleAccount.fromSeed(
        program,
        queueAccount,
        tokenWalletKeypair.publicKey
      );

      console.debug(chalkString(name, oracleAccount.publicKey));

      const [permissionAccount, permissionBump] = PermissionAccount.fromSeed(
        program,
        authorityKeypair.publicKey,
        queueAccount.publicKey,
        oracleAccount.publicKey
      );
      console.debug(
        chalkString(`Permission-${n + 1}`, permissionAccount.publicKey)
      );

      finalTransactions.push([
        SystemProgram.createAccount({
          fromPubkey: payerKeypair.publicKey,
          newAccountPubkey: tokenWalletKeypair.publicKey,
          lamports:
            await program.provider.connection.getMinimumBalanceForRentExemption(
              spl.AccountLayout.span
            ),
          space: spl.AccountLayout.span,
          programId: spl.TOKEN_PROGRAM_ID,
        }),
        spl.createInitializeAccountInstruction(
          tokenWalletKeypair.publicKey,
          mint.address,
          programStateAccount.publicKey,
          spl.TOKEN_PROGRAM_ID
        ),
        await program.methods
          .oracleInit({
            name: Buffer.from(name).slice(0, 32),
            metadata: Buffer.from("").slice(0, 128),
            stateBump,
            oracleBump,
          })
          .accounts({
            oracle: oracleAccount.publicKey,
            oracleAuthority: authorityKeypair.publicKey,
            queue: queueKeypair.publicKey,
            wallet: tokenWalletKeypair.publicKey,
            programState: programStateAccount.publicKey,
            systemProgram: SystemProgram.programId,
            payer: payerKeypair.publicKey,
          })
          .instruction(),
        await program.methods
          .permissionInit({})
          .accounts({
            permission: permissionAccount.publicKey,
            authority: authorityKeypair.publicKey,
            granter: queueAccount.publicKey,
            grantee: oracleAccount.publicKey,
            payer: payerKeypair.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .instruction(),
        await program.methods
          .permissionSet({
            permission: { permitOracleHeartbeat: null },
            enable: true,
          })
          .accounts({
            permission: permissionAccount.publicKey,
            authority: authorityKeypair.publicKey,
          })
          .instruction(),
      ]);
      signers.push(tokenWalletKeypair);
      return {
        oracleAccount,
        name,
        permissionAccount,
        tokenWalletKeypair,
      };
    })
  );

  const createAccountSignatures = await packAndSend(
    program,
    [ixns, finalTransactions],
    signers,
    payerKeypair.publicKey
  );

  // const result = await program.provider.connection.confirmTransaction(
  //   createAccountSignatures[-1]
  // );

  return {
    queueAccount,
    crankPubkey: crankAccount.publicKey,
    oracles: oracleAccounts.map((o) => o.oracleAccount.publicKey) ?? [],
  };
}
