import * as types from '../generated';
import * as borsh from '@project-serum/borsh'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as anchor from '@project-serum/anchor';
import {
  Account,
  BUFFER_DISCRIMINATOR,
  OnAccountChangeCallback,
} from './account';
import * as errors from '../errors';
import { SwitchboardProgram } from '../program';
import {
  Commitment,
  Keypair,
  PublicKey,
  SystemProgram,
  TransactionInstruction,
  TransactionSignature,
} from '@solana/web3.js';
import { TransactionObject } from '../transaction';
import { AggregatorAccount } from './aggregatorAccount';

export interface AggregatorHistoryInit {
  /** Aggregator account to add a history buffer for. */
  aggregatorAccount: AggregatorAccount;
  /** Maximum number of samples to store in a round robin history buffer. */
  maxSamples: number;
  /** Alternative keypair that is the authority for the aggregatorAccount and authorized to add a historyBuffer. */
  aggregatorAuthority?: Keypair;
  /** Existing keypair to create the history buffer for. Must be a fresh keypair not tied to an existing on-chain account. */
  keypair?: Keypair;
}

/**
 * Account type representing a round robin buffer of historical samples.
 *
 * Data: Array<{@linkcode types.AggregatorHistoryRow}>
 */
export class AggregatorHistoryBuffer extends Account<
  Array<types.AggregatorHistoryRow>
> {
  static accountName = 'AggregatorHistoryBuffer';

  public size = 28;

  public static getAccountSize(size: number): number {
    return 12 + size * 28;
  }

  public static default(size = 1000): Buffer {
    const buffer = Buffer.alloc(
      AggregatorHistoryBuffer.getAccountSize(size),
      0
    );
    BUFFER_DISCRIMINATOR.copy(buffer, 0);
    return buffer;
  }

  /**
   * Decode an aggregators history buffer and return an array of historical samples in ascending order by timestamp.
   * @params historyBuffer the historyBuffer AccountInfo stored on-chain
   * @return the array of {@linkcode types.AggregatorHistoryRow} samples
   */
  public static decode(
    historyBuffer: Buffer
  ): Array<types.AggregatorHistoryRow> {
    const ROW_SIZE = 28;

    if (historyBuffer.length < 12) {
      return [];
    }

    const insertIdx = historyBuffer.readUInt32LE(8) * ROW_SIZE;

    const front: Array<types.AggregatorHistoryRow> = [];
    const tail: Array<types.AggregatorHistoryRow> = [];

    const buffer = historyBuffer.slice(12);

    for (let i = 0; i < buffer.length; i += ROW_SIZE) {
      if (i + ROW_SIZE > buffer.length) {
        break;
      }

      const row = types.AggregatorHistoryRow.fromDecoded(
        types.AggregatorHistoryRow.layout().decode(buffer, i)
      );

      if (row.timestamp.eq(new anchor.BN(0))) {
        break;
      }

      if (i <= insertIdx) {
        tail.push(row);
      } else {
        front.push(row);
      }
    }

    return front.concat(tail);
  }

  /**
   * Return an aggregator's assigned history buffer or undefined if it doesn't exist.
   */
  static fromAggregator(
    program: SwitchboardProgram,
    aggregator: types.AggregatorAccountData
  ): AggregatorHistoryBuffer | undefined {
    if (aggregator.historyBuffer.equals(PublicKey.default)) {
      return undefined;
    }

    return new AggregatorHistoryBuffer(program, aggregator.historyBuffer);
  }

  /**
   * Decode an aggregators history buffer and return an array of historical samples
   * @params historyBuffer the historyBuffer AccountInfo stored on-chain
   * @return the array of {@linkcode types.AggregatorHistoryRow} samples
   */
  public decode(historyBuffer: Buffer): Array<types.AggregatorHistoryRow> {
    return AggregatorHistoryBuffer.decode(historyBuffer);
  }

  /**
   * Fetch an aggregators history buffer and return an array of historical samples
   * @params aggregator the pre-loaded aggregator state
   * @return the array of {@linkcode types.AggregatorHistoryRow} samples
   */
  public async loadData(): Promise<Array<types.AggregatorHistoryRow>> {
    if (PublicKey.default.equals(this.publicKey)) {
      return [];
    }
    const bufferAccountInfo = await this.program.connection.getAccountInfo(
      this.publicKey
    );
    if (bufferAccountInfo === null) {
      throw new errors.AccountNotFoundError(
        'Aggregator History',
        this.publicKey
      );
    }
    return AggregatorHistoryBuffer.decode(bufferAccountInfo.data);
  }

  /**
   * Invoke a callback each time an AggregatorAccount's data has changed on-chain.
   * @param callback - the callback invoked when the aggregator state changes
   * @param commitment - optional, the desired transaction finality. defaults to 'confirmed'
   * @returns the websocket subscription id
   */
  public onChange(
    callback: OnAccountChangeCallback<Array<types.AggregatorHistoryRow>>,
    commitment: Commitment = 'confirmed'
  ): number {
    return this.program.connection.onAccountChange(
      this.publicKey,
      accountInfo => {
        callback(this.decode(accountInfo.data));
      },
      commitment
    );
  }

  /**
   * Create a history buffer for an aggregator and store the last N samples in a round robin history buffer.
   * @param program The SwitchboardProgram.
   * @param payer The account that will pay for the new account.
   * @param params history buffer configuration parameters.
   * @return {@linkcode TransactionObject} that will create the AggregatorHistoryBuffer.
   *
   * Basic usage example:
   *
   * ```ts
   * import { AggregatorAccount,AggregatorHistoryBuffer } from '@switchboard-xyz/solana.js';
   * const aggregatorAccount = new AggregatorAccount(program, aggregatorKey);
   * const aggregator = await aggregatorAccount.loadData();
   * const [historyBuffer, addHistoryTxn] = await AggregatorHistoryBuffer.createInstructions(program, payer, {
   *    aggregatorAccount,
   *    maxSamples: 10000,
   * });
   * const aggregatorHistorySignature = await program.signAndSendAll(aggregatorHistoryTxn);
   * const history = await historyBuffer.loadData();
   * ```
   */
  public static async createInstructions(
    program: SwitchboardProgram,
    payer: PublicKey,
    params: AggregatorHistoryInit
  ): Promise<[AggregatorHistoryBuffer, TransactionObject]> {
    const buffer = params.keypair ?? Keypair.generate();
    program.verifyNewKeypair(buffer);

    const ixns: TransactionInstruction[] = [];
    const signers: Keypair[] = params.aggregatorAuthority
      ? [params.aggregatorAuthority, buffer]
      : [buffer];

    const size = AggregatorHistoryBuffer.getAccountSize(params.maxSamples);

    ixns.push(
      SystemProgram.createAccount({
        fromPubkey: payer,
        newAccountPubkey: buffer.publicKey,
        space: size,
        lamports: await program.connection.getMinimumBalanceForRentExemption(
          size
        ),
        programId: program.programId,
      }),
      types.aggregatorSetHistoryBuffer(
        program,
        { params: {} },
        {
          aggregator: params.aggregatorAccount.publicKey,
          authority: params.aggregatorAuthority
            ? params.aggregatorAuthority.publicKey
            : payer,
          buffer: buffer.publicKey,
        }
      )
    );

    return [
      new AggregatorHistoryBuffer(program, buffer.publicKey),
      new TransactionObject(payer, ixns, signers),
    ];
  }

  /**
   * Create a history buffer for an aggregator and store the last N samples in a round robin history buffer.
   * @param program The SwitchboardProgram.
   * @param payer The account that will pay for the new account.
   * @param params history buffer configuration parameters.
   * @return {@linkcode TransactionObject} that will create the AggregatorHistoryBuffer.
   *
   * Basic usage example:
   *
   * ```ts
   * import { AggregatorAccount,AggregatorHistoryBuffer } from '@switchboard-xyz/solana.js';
   * const aggregatorAccount = new AggregatorAccount(program, aggregatorKey);
   * const aggregator = await aggregatorAccount.loadData();
   * const [historyBuffer, addHistorySignature] = await AggregatorHistoryBuffer.create(program, {
   *    aggregatorAccount,
   *    maxSamples: 10000,
   * });
   * const history = await historyBuffer.loadData();
   * ```
   */
  public static async create(
    program: SwitchboardProgram,
    params: AggregatorHistoryInit
  ): Promise<[AggregatorHistoryBuffer, TransactionSignature]> {
    const [account, transaction] =
      await AggregatorHistoryBuffer.createInstructions(
        program,
        program.walletPubkey,
        params
      );
    const txnSignature = await program.signAndSend(transaction);
    return [account, txnSignature];
  }
}
