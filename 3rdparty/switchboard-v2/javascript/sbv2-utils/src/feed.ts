import * as anchor from "@project-serum/anchor";
import * as spl from "@solana/spl-token-v2";
import {
  AggregatorAccount,
  AggregatorInitParams,
  JobAccount,
  LeaseAccount,
  OracleQueueAccount,
  packInstructions,
  PermissionAccount,
  ProgramStateAccount,
  programWallet,
  signTransactions,
  SwitchboardDecimal,
} from "@switchboard-xyz/switchboard-v2";
import Big from "big.js";
import { promiseWithTimeout } from "./async.js";

export async function awaitOpenRound(
  aggregatorAccount: AggregatorAccount,
  queueAccount: OracleQueueAccount,
  payerTokenWallet: anchor.web3.PublicKey,
  expectedValue: Big | undefined = undefined,
  timeout = 30
): Promise<Big> {
  // call open round and wait for new value
  const accountsCoder = new anchor.BorshAccountsCoder(
    aggregatorAccount.program.idl
  );

  let accountWs: number;
  const awaitUpdatePromise = new Promise(
    (resolve: (value: Big) => void, reject: (reason?: string) => void) => {
      accountWs = aggregatorAccount.program.provider.connection.onAccountChange(
        aggregatorAccount?.publicKey ?? anchor.web3.PublicKey.default,
        async (accountInfo) => {
          const aggregator = accountsCoder.decode(
            "AggregatorAccountData",
            accountInfo.data
          );
          const latestResult = await aggregatorAccount.getLatestValue(
            aggregator
          );
          if (!latestResult) {
            return;
          }
          if (!expectedValue) {
            resolve(latestResult);
          } else if (latestResult?.eq(expectedValue)) {
            resolve(latestResult);
          } else {
            reject(
              `Value mismatch, expected ${expectedValue}, received ${latestResult}`
            );
          }
        }
      );
    }
  );

  const updatedValuePromise = promiseWithTimeout(
    timeout * 1000,
    awaitUpdatePromise,
    new Error(`aggregator failed to update in ${timeout} seconds`)
  ).finally(() => {
    if (accountWs) {
      aggregatorAccount.program.provider.connection.removeAccountChangeListener(
        accountWs
      );
    }
  });

  await aggregatorAccount.openRound({
    oracleQueueAccount: queueAccount,
    payoutWallet: payerTokenWallet,
  });

  const result = await updatedValuePromise;

  if (!result) {
    throw new Error(`failed to update aggregator`);
  }

  return result;
}

async function signAndConfirmTransactions(
  program: anchor.Program,
  transactions: anchor.web3.Transaction[]
) {
  const signedTxs = await (
    program.provider as anchor.AnchorProvider
  ).wallet.signAllTransactions(transactions);
  for (const transaction of signedTxs) {
    // console.log(`Blockhash: ${transaction.recentBlockhash}`);
    const sig = await program.provider.connection.sendRawTransaction(
      transaction.serialize(),
      { skipPreflight: false, maxRetries: 10 }
    );
    await program.provider.connection.confirmTransaction(sig);
  }
}

export async function createAggregator(
  program: anchor.Program,
  queueAccount: OracleQueueAccount,
  params: AggregatorInitParams,
  jobs: [JobAccount, number][],
  fundLeaseAmount = new anchor.BN(0)
): Promise<AggregatorAccount> {
  const req = await createAggregatorReq(
    program,
    queueAccount,
    params,
    jobs,
    fundLeaseAmount
  );
  const { blockhash } = await program.provider.connection.getLatestBlockhash();
  const packedTxns = packInstructions(
    req.ixns,
    programWallet(program).publicKey,
    blockhash
  );
  const signedTxns = signTransactions(
    packedTxns,
    req.signers as anchor.web3.Keypair[]
  );
  await signAndConfirmTransactions(program, signedTxns);
  return req.account;
}

/**
 * Retrieve information about the payer's associated token account. If it does not exist, an
 * instruction to create it will be returned with the account's {@linkcode PublicKey}.
 */
async function getPayerTokenAccount(
  connection: anchor.web3.Connection,
  payer: anchor.web3.PublicKey,
  mint: anchor.web3.PublicKey
): Promise<{
  /**
   * The {@linkcode PublicKey} of the associated token account for this payer.
   */
  publicKey: anchor.web3.PublicKey;
  /**
   * If the token account doesn't currently exist on-chain, it needs to be created using this ixn.
   */
  ixn?: anchor.web3.TransactionInstruction;
}> {
  const publicKey = await spl.getAssociatedTokenAddress(
    mint,
    payer,
    undefined,
    spl.TOKEN_PROGRAM_ID,
    spl.ASSOCIATED_TOKEN_PROGRAM_ID
  );
  const accountExists = await connection
    .getAccountInfo(publicKey)
    .then((info) => info !== null)
    .catch(() => false);

  return {
    publicKey,
    ixn: accountExists
      ? undefined // Account exists, so theres no need to create it.
      : spl.createAssociatedTokenAccountInstruction(
          payer,
          publicKey,
          payer,
          mint,
          spl.TOKEN_PROGRAM_ID,
          spl.ASSOCIATED_TOKEN_PROGRAM_ID
        ),
  };
}

export async function createAggregatorReq(
  program: anchor.Program,
  queueAccount: OracleQueueAccount,
  params: AggregatorInitParams,
  jobs: [JobAccount, number][],
  fundLeaseAmount = new anchor.BN(0),
  payerPubkey = programWallet(program as any).publicKey
): Promise<{
  ixns: anchor.web3.TransactionInstruction[];
  signers: anchor.web3.Signer[];
  account: AggregatorAccount;
}> {
  const queue = await queueAccount.loadData();
  const mint = await queueAccount.loadMint();

  // Aggregator params
  const aggregatorKeypair = params.keypair ?? anchor.web3.Keypair.generate();
  const authority = params.authority ?? payerPubkey;
  const size = program.account.aggregatorAccountData.size;
  const [programStateAccount, stateBump] =
    ProgramStateAccount.fromSeed(program);
  const state = await programStateAccount.loadData();
  const aggregatorAccount = new AggregatorAccount({
    program,
    publicKey: aggregatorKeypair.publicKey,
  });

  // Permission params
  const [permissionAccount, permissionBump] = PermissionAccount.fromSeed(
    program,
    queue.authority,
    queueAccount.publicKey,
    aggregatorKeypair.publicKey
  );

  // Lease params
  const [leaseAccount, leaseBump] = LeaseAccount.fromSeed(
    program,
    queueAccount,
    aggregatorAccount
  );
  const leaseEscrow = await spl.getAssociatedTokenAddress(
    mint.address,
    leaseAccount.publicKey,
    true,
    spl.TOKEN_PROGRAM_ID,
    spl.ASSOCIATED_TOKEN_PROGRAM_ID
  );

  // const jobPubkeys: Array<PublicKey> = [];
  // const jobWallets: Array<PublicKey> = [];
  // const walletBumps: Array<number> = [];
  // for (const idx in jobs) {
  //   const [jobWallet, bump] = anchor.utils.publicKey.findProgramAddressSync(
  //     [
  //       payerKeypair.publicKey.toBuffer(),
  //       spl.TOKEN_PROGRAM_ID.toBuffer(),
  //       mint.address.toBuffer(),
  //     ],
  //     spl.ASSOCIATED_TOKEN_PROGRAM_ID
  //   );
  //   jobPubkeys.push(jobs[idx].publicKey);
  //   jobWallets.push(jobWallet);
  //   walletBumps.push(bump);
  // }

  const ixns: anchor.web3.TransactionInstruction[] = [];

  // Check if the user has created a user token account. If not, they'll need to do that first.
  const payerTokenAcct = await getPayerTokenAccount(
    program.provider.connection,
    payerPubkey,
    mint.address
  );
  if (payerTokenAcct.ixn) {
    ixns.push(payerTokenAcct.ixn);
  }

  // TODO: if fundLeaseAmount, check payer has enough funds

  ixns.push(
    ...([
      // allocate aggregator account
      anchor.web3.SystemProgram.createAccount({
        fromPubkey: programWallet(program).publicKey,
        newAccountPubkey: aggregatorKeypair.publicKey,
        space: size,
        lamports:
          await program.provider.connection.getMinimumBalanceForRentExemption(
            size
          ),
        programId: program.programId,
      }),
      // create aggregator
      await program.methods
        .aggregatorInit({
          name: (params.name ?? Buffer.from("")).slice(0, 32),
          metadata: (params.metadata ?? Buffer.from("")).slice(0, 128),
          batchSize: params.batchSize,
          minOracleResults: params.minRequiredOracleResults,
          minJobResults: params.minRequiredJobResults,
          minUpdateDelaySeconds: params.minUpdateDelaySeconds,
          varianceThreshold: SwitchboardDecimal.fromBig(
            new Big(params.varianceThreshold ?? 0)
          ),
          forceReportPeriod: params.forceReportPeriod ?? new anchor.BN(0),
          expiration: params.expiration ?? new anchor.BN(0),
          stateBump,
        })
        .accounts({
          aggregator: aggregatorKeypair.publicKey,
          authority,
          queue: params.queueAccount.publicKey,
          // authorWallet: params.authorWallet ?? state.tokenVault,
          programState: programStateAccount.publicKey,
        })
        .instruction(),
      await program.methods
        .permissionInit({})
        .accounts({
          permission: permissionAccount.publicKey,
          authority: queue.authority,
          granter: queueAccount.publicKey,
          grantee: aggregatorKeypair.publicKey,
          payer: payerPubkey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .instruction(),
      payerPubkey.equals(queue.authority)
        ? await program.methods
            .permissionSet({
              permission: { permitOracleQueueUsage: null },
              enable: true,
            })
            .accounts({
              permission: permissionAccount.publicKey,
              authority: queue.authority,
            })
            .instruction()
        : undefined,
      spl.createAssociatedTokenAccountInstruction(
        payerPubkey,
        leaseEscrow,
        leaseAccount.publicKey,
        mint.address
      ),
      await program.methods
        .leaseInit({
          loadAmount: fundLeaseAmount,
          stateBump,
          leaseBump,
          withdrawAuthority: payerPubkey,
          walletBumps: Buffer.from([]),
        })
        .accounts({
          programState: programStateAccount.publicKey,
          lease: leaseAccount.publicKey,
          queue: queueAccount.publicKey,
          aggregator: aggregatorAccount.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
          funder: payerTokenAcct.publicKey,
          payer: payerPubkey,
          tokenProgram: spl.TOKEN_PROGRAM_ID,
          escrow: leaseEscrow,
          owner: payerPubkey,
          mint: mint.address,
        })
        // .remainingAccounts(
        //   jobPubkeys.concat(jobWallets).map((pubkey: PublicKey) => {
        //     return { isSigner: false, isWritable: true, pubkey };
        //   })
        // )
        .instruction(),
      ...(await Promise.all(
        jobs.map(async ([jobAccount, weight]) => {
          return program.methods
            .aggregatorAddJob({
              weight,
            })
            .accounts({
              aggregator: aggregatorKeypair.publicKey,
              authority,
              job: jobAccount.publicKey,
            })
            .instruction();
        })
      )),
    ].filter(Boolean) as anchor.web3.TransactionInstruction[])
  );

  return {
    ixns: ixns,
    signers: [aggregatorKeypair],
    account: aggregatorAccount,
  };
}
