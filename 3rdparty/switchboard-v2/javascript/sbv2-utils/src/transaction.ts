import * as anchor from "@project-serum/anchor";
import {
  ConfirmOptions,
  Connection,
  Keypair,
  PublicKey,
  TransactionInstruction,
  TransactionSignature,
} from "@solana/web3.js";
import {
  packInstructions,
  signTransactions,
} from "@switchboard-xyz/switchboard-v2";

export async function packAndSend(
  program: anchor.Program,
  ixnsBatches: (TransactionInstruction | TransactionInstruction[])[][],
  signers: Keypair[],
  feePayer: PublicKey
): Promise<TransactionSignature[]> {
  const signatures: Promise<TransactionSignature>[] = [];

  for await (const batch of ixnsBatches) {
    const { blockhash } =
      await program.provider.connection.getLatestBlockhash();

    const packedTransactions = packInstructions(batch, feePayer, blockhash);
    const signedTransactions = signTransactions(packedTransactions, signers);
    const signedTxs = await (
      program.provider as anchor.AnchorProvider
    ).wallet.signAllTransactions(signedTransactions);

    for (let k = 0; k < packedTransactions.length; k += 1) {
      const tx = signedTxs[k]!;
      const rawTx = tx.serialize();
      signatures.push(
        sendAndConfirmRawTransaction(program.provider.connection, rawTx, {
          maxRetries: 10,
          commitment: "processed",
        }).catch((error) => {
          console.error(error);
          throw error;
        })
      );
    }

    await Promise.all(signatures);
  }

  return Promise.all(signatures);
}

/**
 * Send and confirm a raw transaction
 *
 * If `commitment` option is not specified, defaults to 'max' commitment.
 */
export async function sendAndConfirmRawTransaction(
  connection: Connection,
  rawTransaction: Buffer,
  options: ConfirmOptions
): Promise<TransactionSignature> {
  const sendOptions = options && {
    skipPreflight: options.skipPreflight,
    preflightCommitment: options.preflightCommitment || options.commitment,
  };
  const signature: TransactionSignature = await connection.sendRawTransaction(
    rawTransaction,
    sendOptions
  );
  const status = (
    await connection.confirmTransaction(
      signature as any,
      options.commitment || "max"
    )
  ).value;

  if (status.err) {
    throw new Error(
      `Raw transaction ${signature} failed (${JSON.stringify(status)})`
    );
  }

  return signature;
}
