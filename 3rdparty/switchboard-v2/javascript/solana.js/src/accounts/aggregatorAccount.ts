import * as types from '../generated';
import * as anchor from '@project-serum/anchor';
import { Account, OnAccountChangeCallback } from './account';
import * as errors from '../errors';
import Big from 'big.js';
import { SwitchboardProgram } from '../program';
import {
  AccountInfo,
  AccountMeta,
  Commitment,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  TransactionInstruction,
  TransactionSignature,
} from '@solana/web3.js';
import { OracleAccount } from './oracleAccount';
import { OracleJob, promiseWithTimeout, toUtf8 } from '@switchboard-xyz/common';
import crypto from 'crypto';
import { JobAccount } from './jobAccount';
import { QueueAccount } from './queueAccount';
import { LeaseAccount } from './leaseAccount';
import { PermissionAccount } from './permissionAccount';
import * as spl from '@solana/spl-token';
import { TransactionObject } from '../transaction';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { AggregatorHistoryBuffer } from './aggregatorHistoryBuffer';
import { CrankAccount } from './crankAccount';
import assert from 'assert';

/**
 * Account type holding a data feed's update configuration, job accounts, and its current result.
 *
 * Data: {@linkcode types.AggregatorAccountData}
 *
 * Result: {@linkcode types.SwitchboardDecimal}
 *
 * HistoryBuffer?: Array<{@linkcode types.AggregatorHistoryRow}>
 *
 * An aggregator account belongs to a single {@linkcode QueueAccount} but can later be transferred by the aggregator's authority. In order for an {@linkcode OracleAccount} to respond to an aggregator's update request, the aggregator must initialize a {@linkcode PermissionAccount} and {@linkcode LeaseAccount}. These will need to be recreated when transferring queues.
 *
 * Optionally, An aggregator can be pushed onto a {@linkcode CrankAccount} in order to be updated
 *
 * Optionally, an aggregator can add a history buffer to store the last N historical samples along with their update timestamp.
 */
export class AggregatorAccount extends Account<types.AggregatorAccountData> {
  static accountName = 'AggregatorAccountData';

  public history?: AggregatorHistoryBuffer;

  /**
   * Returns the aggregator's name buffer in a stringified format.
   */
  public static getName = (aggregator: types.AggregatorAccountData) =>
    toUtf8(aggregator.name);
  /**
   * Returns the aggregator's metadata buffer in a stringified format.
   */
  public static getMetadata = (aggregator: types.AggregatorAccountData) =>
    toUtf8(aggregator.metadata);

  public static size = 3851;

  /**
   * Get the size of an {@linkcode AggregatorAccount} on-chain.
   */
  public size = this.program.account.aggregatorAccountData.size;

  public decode(data: Buffer): types.AggregatorAccountData {
    try {
      return types.AggregatorAccountData.decode(data);
    } catch {
      return this.program.coder.decode<types.AggregatorAccountData>(
        AggregatorAccount.accountName,
        data
      );
    }
  }

  public static default(): types.AggregatorAccountData {
    const buffer = Buffer.alloc(AggregatorAccount.size, 0);
    types.AggregatorAccountData.discriminator.copy(buffer, 0);
    return types.AggregatorAccountData.decode(buffer);
  }

  public static createMock(
    programId: PublicKey,
    data: Partial<types.AggregatorAccountData>,
    options?: {
      lamports?: number;
      rentEpoch?: number;
    }
  ): AccountInfo<Buffer> {
    const fields: types.AggregatorAccountDataFields = {
      ...AggregatorAccount.default(),
      ...data,
      // any cleanup actions here
    };
    const state = new types.AggregatorAccountData(fields);

    const buffer = Buffer.alloc(AggregatorAccount.size, 0);
    types.AggregatorAccountData.discriminator.copy(buffer, 0);
    types.AggregatorAccountData.layout.encode(state, buffer, 8);

    return {
      executable: false,
      owner: programId,
      lamports: options?.lamports ?? 1 * LAMPORTS_PER_SOL,
      data: buffer,
      rentEpoch: options?.rentEpoch ?? 0,
    };
  }

  /**
   * Invoke a callback each time an AggregatorAccount's data has changed on-chain.
   * @param callback - the callback invoked when the aggregator state changes
   * @param commitment - optional, the desired transaction finality. defaults to 'confirmed'
   * @returns the websocket subscription id
   */
  public onChange(
    callback: OnAccountChangeCallback<types.AggregatorAccountData>,
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
   * Retrieve and decode the {@linkcode types.AggregatorAccountData} stored in this account.
   */
  public async loadData(): Promise<types.AggregatorAccountData> {
    const data = await types.AggregatorAccountData.fetch(
      this.program,
      this.publicKey
    );
    if (data === null)
      throw new errors.AccountNotFoundError('Aggregator', this.publicKey);
    this.history = AggregatorHistoryBuffer.fromAggregator(this.program, data);
    return data;
  }

  public get slidingWindowKey(): PublicKey {
    return anchor.utils.publicKey.findProgramAddressSync(
      [Buffer.from('SlidingResultAccountData'), this.publicKey.toBytes()],
      this.program.programId
    )[0];
  }

  /** Load an existing AggregatorAccount with its current on-chain state */
  public static async load(
    program: SwitchboardProgram,
    publicKey: PublicKey | string
  ): Promise<[AggregatorAccount, types.AggregatorAccountData]> {
    const account = new AggregatorAccount(
      program,
      typeof publicKey === 'string' ? new PublicKey(publicKey) : publicKey
    );
    const state = await account.loadData();
    return [account, state];
  }

  /**
   * Creates a transaction object with aggregatorInit instructions.
   * @param program The SwitchboardProgram.
   * @param payer The account that will pay for the new accounts.
   * @param params aggregator configuration parameters.
   * @return {@linkcode TransactionObject} that will create the aggregatorAccount.
   *
   * Basic usage example:
   *
   * ```ts
   * import {AggregatorAccount} from '@switchboard-xyz/solana.js';
   * const [aggregatorAccount, aggregatorInit ] = await AggregatorAccount.createInstruction(program, payer, {
   *    queueAccount,
   *    queueAuthority,
   *    batchSize: 5,
   *    minRequiredOracleResults: 3,
   *    minRequiredJobResults: 1,
   *    minUpdateDelaySeconds: 30,
   * });
   * const txnSignature = await program.signAndSend(aggregatorInit);
   * ```
   */
  public static async createInstruction(
    program: SwitchboardProgram,
    payer: PublicKey,
    params: AggregatorInitParams
  ): Promise<[AggregatorAccount, TransactionObject]> {
    const keypair = params.keypair ?? Keypair.generate();
    program.verifyNewKeypair(keypair);

    const ixns: TransactionInstruction[] = [];
    const signers: Keypair[] = [keypair];

    ixns.push(
      SystemProgram.createAccount({
        fromPubkey: payer,
        newAccountPubkey: keypair.publicKey,
        space: program.account.aggregatorAccountData.size,
        lamports: await program.connection.getMinimumBalanceForRentExemption(
          program.account.aggregatorAccountData.size
        ),
        programId: program.programId,
      })
    );

    ixns.push(
      types.aggregatorInit(
        program,
        {
          params: {
            name: [...Buffer.from(params.name ?? '', 'utf8').slice(0, 32)],
            metadata: [
              ...Buffer.from(params.metadata ?? '', 'utf8').slice(0, 128),
            ],
            batchSize: params.batchSize,
            minOracleResults: params.minRequiredOracleResults,
            minJobResults: params.minRequiredJobResults,
            minUpdateDelaySeconds: params.minUpdateDelaySeconds,
            startAfter: new anchor.BN(params.startAfter ?? 0),
            varianceThreshold: types.SwitchboardDecimal.fromBig(
              new Big(params.varianceThreshold ?? 0)
            ).borsh,
            forceReportPeriod: new anchor.BN(params.forceReportPeriod ?? 0),
            expiration: new anchor.BN(params.expiration ?? 0),
            stateBump: program.programState.bump,
            disableCrank: params.disableCrank ?? false,
          },
        },
        {
          aggregator: keypair.publicKey,
          authority: params.authority ?? payer,
          queue: params.queueAccount.publicKey,
          programState: program.programState.publicKey,
        }
      )
    );

    const aggregatorInit = new TransactionObject(payer, ixns, signers);
    const aggregatorAccount = new AggregatorAccount(program, keypair.publicKey);

    return [aggregatorAccount, aggregatorInit];
  }

  /**
   * Creates an aggregator on-chain and return the transaction signature and created account resource.
   * @param program The SwitchboardProgram.
   * @param params aggregator configuration parameters.
   * @return Transaction signature and the newly created aggregatorAccount.
   *
   * Basic usage example:
   *
   * ```ts
   * import {AggregatorAccount} from '@switchboard-xyz/solana.js';
   * const [aggregatorAccount, txnSignature] = await AggregatorAccount.create(program, {
   *    queueAccount,
   *    queueAuthority,
   *    batchSize: 5,
   *    minRequiredOracleResults: 3,
   *    minRequiredJobResults: 1,
   *    minUpdateDelaySeconds: 30,
   * });
   * const aggregator = await aggregatorAccount.loadData();
   * ```
   */
  public static async create(
    program: SwitchboardProgram,
    params: AggregatorInitParams
  ): Promise<[AggregatorAccount, TransactionSignature]> {
    const [account, transaction] = await AggregatorAccount.createInstruction(
      program,
      program.walletPubkey,
      params
    );
    const txnSignature = await program.signAndSend(transaction);
    return [account, txnSignature];
  }

  /**
   * Create the {@linkcode PermissionAccount} and {@linkcode LeaseAccount} for a new oracle queue without affecting the feed's rhythm. This does not evict a feed from the current queue.
   */
  async transferQueuePart1Instructions(
    payer: PublicKey,
    params: {
      newQueue: QueueAccount;
      loadAmount?: number;
      funderTokenAddress?: PublicKey;
      funderAuthority?: Keypair;
    }
  ): Promise<[TransactionObject, PermissionAccount, LeaseAccount]> {
    const txns: Array<TransactionObject> = [];

    const aggregator = await this.loadData();
    const newQueue = await params.newQueue.loadData();

    const jobs = await this.loadJobs(aggregator);
    const jobAuthorities = jobs.map(job => job.state.authority);

    const [oldLeaseAccount] = LeaseAccount.fromSeed(
      this.program,
      aggregator.queuePubkey,
      this.publicKey
    );

    const [newPermissionAccount] = PermissionAccount.fromSeed(
      this.program,
      newQueue.authority,
      params.newQueue.publicKey,
      this.publicKey
    );
    const newPermissionAccountInfo =
      await this.program.connection.getAccountInfo(
        newPermissionAccount.publicKey
      );
    if (newPermissionAccountInfo === null) {
      const [_newPermissionAccount, permissionInitTxn] =
        PermissionAccount.createInstruction(this.program, payer, {
          authority: newQueue.authority,
          granter: params.newQueue.publicKey,
          grantee: this.publicKey,
        });
      assert(
        newPermissionAccount.publicKey.equals(_newPermissionAccount.publicKey)
      );
      txns.push(permissionInitTxn);
    }

    const [newLeaseAccount] = LeaseAccount.fromSeed(
      this.program,
      params.newQueue.publicKey,
      this.publicKey
    );
    const newLeaseAccountInfo = await this.program.connection.getAccountInfo(
      newLeaseAccount.publicKey
    );
    if (newLeaseAccountInfo === null) {
      const [userTokenWallet, userWrap] =
        params.loadAmount && params.loadAmount > 0 && !params.funderTokenAddress
          ? await this.program.mint.getOrCreateWrappedUserInstructions(payer, {
              fundUpTo: params.loadAmount ?? 0,
            })
          : [undefined, undefined];
      if (userWrap) {
        txns.push(userWrap);
      }

      // create lease for the new queue but dont transfer any balance
      const [_newLeaseAccount, leaseInit] =
        await LeaseAccount.createInstructions(this.program, payer, {
          aggregatorAccount: this,
          queueAccount: params.newQueue,
          funderTokenAccount:
            params.funderTokenAddress ?? userTokenWallet ?? undefined,
          funderAuthority: params.funderAuthority ?? undefined,
          jobAuthorities,
          loadAmount: params.loadAmount ?? 0,
          withdrawAuthority: aggregator.authority, // set to current authority
          jobPubkeys: aggregator.jobPubkeysData.slice(
            0,
            aggregator.jobPubkeysSize
          ),
        });
      assert(newLeaseAccount.publicKey.equals(_newLeaseAccount.publicKey));
      txns.push(leaseInit);
    }

    const packed = TransactionObject.pack(txns);
    if (packed.length !== 1) {
      throw new Error(
        `QueueTransfer-Part1: Expected a single TransactionObject`
      );
    }

    return [packed[0], newPermissionAccount, newLeaseAccount];
  }

  /**
   * Create the {@linkcode PermissionAccount} and {@linkcode LeaseAccount} for a new oracle queue without affecting the feed's rhythm. This does not evict a feed from the current queue.
   */
  async transferQueuePart1(params: {
    newQueue: QueueAccount;
    loadAmount?: number;
    funderTokenAddress?: PublicKey;
    funderAuthority?: Keypair;
  }): Promise<[PermissionAccount, LeaseAccount, TransactionSignature]> {
    const [txn, permissionAccount, leaseAccount] =
      await this.transferQueuePart1Instructions(
        this.program.walletPubkey,
        params
      );
    const signature = await this.program.signAndSend(txn);
    return [permissionAccount, leaseAccount, signature];
  }

  /**
   * Approve the feed to use the new queue. Must be signed by the {@linkcode QueueAccount} authority.
   *
   * This does not affect the feed's rhythm nor evict it from the current queue. The Aggregator authority is still required to sign a transaction to move the feed.
   */
  async transferQueuePart2Instructions(
    payer: PublicKey,
    params: {
      newQueue: QueueAccount;
      enable: boolean;
      queueAuthority?: Keypair;
      // dont check if new accounts have been created yet
      force?: boolean;
    }
  ): Promise<[TransactionObject, PermissionAccount]> {
    const newQueue = await params.newQueue.loadData();
    if (
      params.queueAuthority &&
      !params.queueAuthority.publicKey.equals(newQueue.authority)
    ) {
      throw new errors.IncorrectAuthority(
        newQueue.authority,
        params.queueAuthority.publicKey
      );
    }

    const [permissionAccount] = PermissionAccount.fromSeed(
      this.program,
      newQueue.authority,
      params.newQueue.publicKey,
      this.publicKey
    );
    if (!params.force) {
      await permissionAccount.loadData().catch(() => {
        throw new Error(`Expected permissionAccount to be created already`);
      });
    }

    const permissionSet = permissionAccount.setInstruction(payer, {
      enable: params.enable,
      queueAuthority: params.queueAuthority,
      permission: new types.SwitchboardPermission.PermitOracleQueueUsage(),
    });

    return [permissionSet, permissionAccount];
  }

  /**
   * Approve the feed to use the new queue. Must be signed by the {@linkcode QueueAccount} authority.
   *
   * This does not affect the feed's rhythm nor evict it from the current queue. The Aggregator authority is still required to sign a transaction to move the feed.
   */
  async transferQueuePart2(params: {
    newQueue: QueueAccount;
    enable: boolean;
    queueAuthority?: Keypair;
    // dont check if new accounts have been created yet
    force?: boolean;
  }): Promise<[PermissionAccount, TransactionSignature]> {
    const [txn, permissionAccount] = await this.transferQueuePart2Instructions(
      this.program.walletPubkey,
      params
    );
    const signature = await this.program.signAndSend(txn);
    return [permissionAccount, signature];
  }

  /**
   * Transfer the feed to the new {@linkcode QueueAccount} and optionally push it on a crank. Must be signed by the Aggregator authority to approve the transfer.
   *
   * This will evict a feed from the previous queue and crank.
   */
  async transferQueuePart3Instructions(
    payer: PublicKey,
    params: {
      newQueue: QueueAccount;
      authority?: Keypair;
      newCrank?: CrankAccount;
      // dont check if new accounts have been created yet
      force?: boolean;
    }
  ): Promise<Array<TransactionObject>> {
    const txns: TransactionObject[] = [];

    const newQueue = await params.newQueue.loadData();
    const aggregator = await this.loadData();

    // new permission account needs to be created and approved
    const [permissionAccount] = PermissionAccount.fromSeed(
      this.program,
      newQueue.authority,
      params.newQueue.publicKey,
      this.publicKey
    );
    if (!params.force) {
      await permissionAccount
        .loadData()
        .catch(() => {
          throw new Error(`Expected permissionAccount to be created already`);
        })
        .then(permission => {
          if (
            !newQueue.unpermissionedFeedsEnabled &&
            permission.permissions !== 2
          ) {
            throw new Error(
              `PermissionAccount missing required permissions to enable this queue`
            );
          }
        });
    }

    // check the new lease has been created
    const [newLeaseAccount] = LeaseAccount.fromSeed(
      this.program,
      params.newQueue.publicKey,
      this.publicKey
    );
    if (!params.force) {
      await newLeaseAccount.loadData().catch(() => {
        throw new Error(`Expected leaseAccount to be created already`);
      });
    }

    // set the feeds queue
    const setQueueTxn = new TransactionObject(
      payer,
      [
        types.aggregatorSetQueue(
          this.program,
          { params: {} },
          {
            aggregator: this.publicKey,
            authority: aggregator.authority,
            queue: params.newQueue.publicKey,
          }
        ),
      ],
      params.authority ? [params.authority] : []
    );
    txns.push(setQueueTxn);

    // push to crank
    if (params.newCrank) {
      const newCrank = await params.newCrank.loadData();
      if (!newCrank.queuePubkey.equals(params.newQueue.publicKey)) {
        throw new Error(`Crank is owned by the wrong queue`);
      }
      const crankPush = await params.newCrank.pushInstruction(payer, {
        aggregatorAccount: this,
      });
      txns.push(crankPush);
    }

    // remove any funds from the old lease account
    const [oldLeaseAccount] = LeaseAccount.fromSeed(
      this.program,
      aggregator.queuePubkey,
      this.publicKey
    );
    const oldLease = await oldLeaseAccount.loadData();
    const oldLeaseBalance = await oldLeaseAccount.getBalance(oldLease.escrow);
    if (oldLease.withdrawAuthority.equals(payer) && oldLeaseBalance > 0) {
      const withdrawTxn = await oldLeaseAccount.withdrawInstruction(payer, {
        amount: oldLeaseBalance,
        unwrap: false,
        // withdraw old lease funds into the new lease
        withdrawWallet: this.program.mint.getAssociatedAddress(
          newLeaseAccount.publicKey
        ),
      });
      txns.push(withdrawTxn);
    }

    return TransactionObject.pack(txns);
  }

  /**
   * Transfer the feed to the new {@linkcode QueueAccount} and optionally push it on a crank. Must be signed by the Aggregator authority to approve the transfer.
   *
   * This will evict a feed from the previous queue and crank.
   */
  async transferQueuePart3(params: {
    newQueue: QueueAccount;
    authority?: Keypair;
    newCrank?: CrankAccount;
    // dont check if new accounts have been created yet
    force?: boolean;
  }): Promise<Array<TransactionSignature>> {
    const txns = await this.transferQueuePart3Instructions(
      this.program.walletPubkey,
      params
    );
    const signatures = await this.program.signAndSendAll(txns, {
      skipPreflight: true,
    });
    return signatures;
  }

  async transferQueueInstructions(
    payer: PublicKey,
    params: {
      authority?: Keypair;
      newQueue: QueueAccount;
      newCrank?: CrankAccount;
      enable: boolean;
      queueAuthority?: Keypair;
      loadAmount?: number;
      funderTokenAddress?: PublicKey;
      funderAuthority?: Keypair;
    }
  ): Promise<[Array<TransactionObject>, PermissionAccount, LeaseAccount]> {
    const [part1, permissionAccount, leaseAccount] =
      await this.transferQueuePart1Instructions(payer, {
        newQueue: params.newQueue,
        loadAmount: params.loadAmount,
        funderTokenAddress: params.funderTokenAddress,
        funderAuthority: params.funderAuthority,
      });
    const [part2] = await this.transferQueuePart2Instructions(payer, {
      newQueue: params.newQueue,
      enable: params.enable,
      queueAuthority: params.queueAuthority,
      force: true,
    });
    const part3 = await this.transferQueuePart3Instructions(payer, {
      newQueue: params.newQueue,
      authority: params.authority,
      newCrank: params.newCrank,
      force: true,
    });

    return [
      TransactionObject.pack([part1, part2, ...part3]),
      permissionAccount,
      leaseAccount,
    ];
  }

  async transferQueue(params: {
    authority?: Keypair;
    newQueue: QueueAccount;
    newCrank?: CrankAccount;
    enable: boolean;
    queueAuthority?: Keypair;
    loadAmount?: number;
    funderTokenAddress?: PublicKey;
    funderAuthority?: Keypair;
  }): Promise<[PermissionAccount, LeaseAccount, Array<TransactionSignature>]> {
    const [txns, permissionAccount, leaseAccount] =
      await this.transferQueueInstructions(this.program.walletPubkey, params);
    const signatures = await this.program.signAndSendAll(txns, {
      skipPreflight: true,
    });
    return [permissionAccount, leaseAccount, signatures];
  }

  public getAccounts(params: {
    queueAccount: QueueAccount;
    queueAuthority: PublicKey;
  }): {
    queueAccount: QueueAccount;
    permissionAccount: PermissionAccount;
    permissionBump: number;
    leaseAccount: LeaseAccount;
    leaseBump: number;
    leaseEscrow: PublicKey;
  } {
    const queueAccount = params.queueAccount;

    const [permissionAccount, permissionBump] = PermissionAccount.fromSeed(
      this.program,
      params.queueAuthority,
      queueAccount.publicKey,
      this.publicKey
    );

    const [leaseAccount, leaseBump] = LeaseAccount.fromSeed(
      this.program,
      queueAccount.publicKey,
      this.publicKey
    );

    const leaseEscrow = this.program.mint.getAssociatedAddress(
      leaseAccount.publicKey
    );

    return {
      queueAccount,
      permissionAccount,
      permissionBump,
      leaseAccount,
      leaseBump,
      leaseEscrow,
    };
  }

  /**
   * Get the latest confirmed value stored in the aggregator account.
   * @param aggregator Optional parameter representing the already loaded
   * aggregator info.
   * @return latest feed value
   */
  public static decodeLatestValue(
    aggregator: types.AggregatorAccountData
  ): Big | null {
    if ((aggregator.latestConfirmedRound?.numSuccess ?? 0) === 0) {
      return null;
    }
    const result = aggregator.latestConfirmedRound.result.toBig();
    return result;
  }

  /**
   * Get the latest confirmed value stored in the aggregator account.
   * @return latest feed value or null if not populated
   */
  public async fetchLatestValue(): Promise<Big | null> {
    const aggregator = await this.loadData();
    return AggregatorAccount.decodeLatestValue(aggregator);
  }

  /**
   * Get the timestamp latest confirmed round stored in the aggregator account.
   * @param aggregator Optional parameter representing the already loaded
   * aggregator info.
   * @return latest feed timestamp
   */
  public static decodeLatestTimestamp(
    aggregator: types.AggregatorAccountData
  ): anchor.BN {
    if ((aggregator.latestConfirmedRound?.numSuccess ?? 0) === 0) {
      throw new Error('Aggregator currently holds no value.');
    }
    return aggregator.latestConfirmedRound.roundOpenTimestamp;
  }

  public static decodeConfirmedRoundResults(
    aggregator: types.AggregatorAccountData
  ): Array<{ oraclePubkeys: PublicKey; value: Big }> {
    if ((aggregator.latestConfirmedRound?.numSuccess ?? 0) === 0) {
      throw new Error('Aggregator currently holds no value.');
    }
    const results: Array<{ oraclePubkeys: PublicKey; value: Big }> = [];
    for (let i = 0; i < aggregator.oracleRequestBatchSize; ++i) {
      if (aggregator.latestConfirmedRound.mediansFulfilled[i] === true) {
        results.push({
          oraclePubkeys: aggregator.latestConfirmedRound.oraclePubkeysData[i],
          value: aggregator.latestConfirmedRound.mediansData[i].toBig(),
        });
      }
    }
    return results;
  }

  /**
   * Get the individual oracle results of the latest confirmed round.
   * @param aggregator Optional parameter representing the already loaded
   * aggregator info.
   * @return latest results by oracle pubkey
   */
  public getConfirmedRoundResults(
    aggregator: types.AggregatorAccountData
  ): Array<{ oracleAccount: OracleAccount; value: Big }> {
    return AggregatorAccount.decodeConfirmedRoundResults(aggregator).map(o => {
      return {
        oracleAccount: new OracleAccount(this.program, o.oraclePubkeys),
        value: o.value,
      };
    });
  }

  /**
   * Produces a hash of all the jobs currently in the aggregator
   * @return hash of all the feed jobs.
   */
  public produceJobsHash(jobs: Array<OracleJob>): crypto.Hash {
    const hash = crypto.createHash('sha256');
    for (const job of jobs) {
      const jobHasher = crypto.createHash('sha256');
      jobHasher.update(OracleJob.encodeDelimited(job).finish());
      hash.update(jobHasher.digest());
    }
    return hash;
  }

  public static decodeCurrentRoundOracles(
    aggregator: types.AggregatorAccountData
  ): Array<PublicKey> {
    return aggregator.currentRound.oraclePubkeysData.slice(
      0,
      aggregator.oracleRequestBatchSize
    );
  }

  public async loadCurrentRoundOracles(
    aggregator: types.AggregatorAccountData
  ): Promise<
    Array<{ account: OracleAccount; state: types.OracleAccountData }>
  > {
    return await Promise.all(
      AggregatorAccount.decodeCurrentRoundOracles(aggregator).map(async o => {
        const oracleAccount = new OracleAccount(this.program, o);
        return {
          account: oracleAccount,
          state: await oracleAccount.loadData(),
        };
      })
    );
  }

  public static decodeJobPubkeys(
    aggregator: types.AggregatorAccountData
  ): Array<PublicKey> {
    return aggregator.jobPubkeysData.slice(0, aggregator.jobPubkeysSize);
  }

  public async loadJobs(
    aggregator: types.AggregatorAccountData
  ): Promise<
    Array<{ account: JobAccount; state: types.JobAccountData; job: OracleJob }>
  > {
    const jobAccountDatas = await anchor.utils.rpc.getMultipleAccounts(
      this.program.connection,
      AggregatorAccount.decodeJobPubkeys(aggregator)
    );

    return await Promise.all(
      jobAccountDatas.map(async j => {
        if (!j?.account) {
          throw new Error(
            `Failed to fetch account data for job ${j?.publicKey}`
          );
        }
        const jobAccount = new JobAccount(this.program, j.publicKey);
        const jobState: types.JobAccountData = this.program.coder.decode(
          'JobAccountData',
          j.account.data
        );
        return {
          account: jobAccount,
          state: jobState,
          job: OracleJob.decodeDelimited(jobState.data),
        };
      })
    );
  }

  public getJobHashes(
    jobs: Array<{
      account: JobAccount;
      state: types.JobAccountData;
    }>
  ): Array<Buffer> {
    return jobs.map(j => Buffer.from(new Uint8Array(j.state.hash)));
  }

  /**
   * Validate an aggregators config
   *
   * @throws {AggregatorConfigError} if minUpdateDelaySeconds < 5, if batchSize > queueSize, if minOracleResults > batchSize, if minJobResults > aggregator.jobPubkeysSize
   */
  public verifyConfig(
    aggregator:
      | types.AggregatorAccountData
      | {
          oracleRequestBatchSize: number;
          minOracleResults: number;
          minJobResults: number;
          minUpdateDelaySeconds: number;
          jobPubkeysSize: number;
        },
    queue: types.OracleQueueAccountData,
    target: {
      batchSize?: number;
      minOracleResults?: number;
      minJobResults?: number;
      minUpdateDelaySeconds?: number;
    }
  ): void {
    const numberOfOracles = queue.size;

    const endState = {
      batchSize: target.batchSize ?? aggregator.oracleRequestBatchSize,
      minOracleResults: target.minOracleResults ?? aggregator.minOracleResults,
      minJobResults: target.minJobResults ?? aggregator.minJobResults,
      minUpdateDelaySeconds:
        target.minUpdateDelaySeconds ?? aggregator.minUpdateDelaySeconds,
    };

    if (endState.minUpdateDelaySeconds < 5) {
      throw new errors.AggregatorConfigError(
        'minUpdateDelaySeconds',
        'must be greater than 5 seconds'
      );
    }

    if (endState.minJobResults > aggregator.jobPubkeysSize) {
      throw new errors.AggregatorConfigError(
        'minJobResults',
        `must be less than the number of jobs (${aggregator.jobPubkeysSize})`
      );
    }

    if (endState.batchSize > numberOfOracles) {
      throw new errors.AggregatorConfigError(
        'oracleRequestBatchSize',
        `must be less than the number of oracles actively heartbeating on the queue (${numberOfOracles})`
      );
    }

    if (endState.minOracleResults > endState.batchSize) {
      throw new errors.AggregatorConfigError(
        'minOracleResults',
        `must be less than the oracleRequestBatchSize (${endState.batchSize})`
      );
    }
  }

  public async setConfigInstruction(
    payer: PublicKey,
    params: Partial<{
      name: string;
      metadata: string;
      batchSize: number;
      minOracleResults: number;
      minJobResults: number;
      minUpdateDelaySeconds: number;
      forceReportPeriod: number;
      varianceThreshold: number;
      authority: Keypair;
      basePriorityFee: number;
      priorityFeeBump: number;
      priorityFeeBumpPeriod: number;
      maxPriorityFeeMultiplier: number;
      force: boolean;
    }>
  ): Promise<TransactionObject> {
    if (!(params.force ?? false)) {
      const aggregator = await this.loadData();
      const queueAccount = new QueueAccount(
        this.program,
        aggregator.queuePubkey
      );
      const queue = await queueAccount.loadData();
      this.verifyConfig(aggregator, queue, {
        batchSize: params.batchSize,
        minOracleResults: params.minOracleResults,
        minJobResults: params.minJobResults,
        minUpdateDelaySeconds: params.minUpdateDelaySeconds,
      });
    }

    const setConfigIxn = types.aggregatorSetConfig(
      this.program,
      {
        params: {
          name: params.name
            ? ([
                ...Buffer.from(params.name, 'utf-8').slice(0, 32),
              ] as Array<number>)
            : null,
          metadata: params.metadata
            ? ([
                ...Buffer.from(params.metadata, 'utf-8').slice(0, 128),
              ] as Array<number>)
            : null,
          batchSize: params.batchSize ?? null,
          minOracleResults: params.minOracleResults ?? null,
          minUpdateDelaySeconds: params.minUpdateDelaySeconds ?? null,
          minJobResults: params.minJobResults ?? null,
          forceReportPeriod: params.forceReportPeriod ?? null,
          varianceThreshold:
            params.varianceThreshold && params.varianceThreshold >= 0
              ? new types.BorshDecimal(
                  types.SwitchboardDecimal.fromBig(
                    new Big(params.varianceThreshold)
                  )
                )
              : null,
          basePriorityFee: params.basePriorityFee ?? 0,
          priorityFeeBump: params.priorityFeeBump ?? 0,
          priorityFeeBumpPeriod: params.priorityFeeBumpPeriod ?? 0,
          maxPriorityFeeMultiplier: params.maxPriorityFeeMultiplier ?? 0,
        },
      },
      {
        aggregator: this.publicKey,
        authority: params.authority ? params.authority.publicKey : payer,
      }
    );

    return new TransactionObject(
      payer,
      [setConfigIxn],
      params.authority ? [params.authority] : []
    );
  }

  public async setConfig(
    params: Partial<{
      name: string;
      metadata: string;
      batchSize: number;
      minOracleResults: number;
      minJobResults: number;
      minUpdateDelaySeconds: number;
      forceReportPeriod: number;
      varianceThreshold: number;
      authority?: Keypair;
      basePriorityFee?: number;
      priorityFeeBump?: number;
      priorityFeeBumpPeriod?: number;
      maxPriorityFeeMultiplier?: number;
      force: boolean;
    }>
  ): Promise<TransactionSignature> {
    const setConfigTxn = await this.setConfigInstruction(
      this.program.walletPubkey,
      params
    );
    const txnSignature = await this.program.signAndSend(setConfigTxn);
    return txnSignature;
  }

  public setQueueInstruction(
    payer: PublicKey,
    params: {
      queueAccount: QueueAccount;
      authority?: Keypair;
    }
  ): TransactionObject {
    const setQueueIxn = types.aggregatorSetQueue(
      this.program,
      {
        params: {},
      },
      {
        aggregator: this.publicKey,
        authority: params.authority ? params.authority.publicKey : payer,
        queue: params.queueAccount.publicKey,
      }
    );
    return new TransactionObject(
      payer,
      [setQueueIxn],
      params.authority ? [params.authority] : []
    );
  }

  public async setQueue(params: {
    queueAccount: QueueAccount;
    authority?: Keypair;
  }): Promise<TransactionSignature> {
    const setQueueTxn = this.setQueueInstruction(
      this.program.walletPubkey,
      params
    );
    const txnSignature = await this.program.signAndSend(setQueueTxn);
    return txnSignature;
  }

  public addJobInstruction(
    payer: PublicKey,
    params: {
      job: JobAccount;
      weight?: number;
      authority?: Keypair;
    }
  ): TransactionObject {
    const authority = params.authority ? params.authority.publicKey : payer;
    const addJobIxn = types.aggregatorAddJob(
      this.program,
      { params: { weight: params.weight ?? 1 } },
      {
        aggregator: this.publicKey,
        authority: authority,
        job: params.job.publicKey,
      }
    );
    return new TransactionObject(
      payer,
      [addJobIxn],
      params.authority ? [params.authority] : []
    );
  }

  public async addJob(params: {
    job: JobAccount;
    weight?: number;
    authority?: Keypair;
  }): Promise<TransactionSignature> {
    const txn = this.addJobInstruction(this.program.walletPubkey, params);
    const txnSignature = await this.program.signAndSend(txn);
    return txnSignature;
  }

  public lockInstruction(
    payer: PublicKey,
    params: {
      authority?: Keypair;
    }
  ): TransactionObject {
    return new TransactionObject(
      payer,
      [
        types.aggregatorLock(
          this.program,
          { params: {} },
          {
            aggregator: this.publicKey,
            authority: params.authority ? params.authority.publicKey : payer,
          }
        ),
      ],
      params.authority ? [params.authority] : []
    );
  }

  public async lock(params: {
    authority?: Keypair;
  }): Promise<TransactionSignature> {
    const lockTxn = this.lockInstruction(this.program.walletPubkey, params);
    const txnSignature = await this.program.signAndSend(lockTxn);
    return txnSignature;
  }

  public setAuthorityInstruction(
    payer: PublicKey,
    params: {
      newAuthority: PublicKey;
      authority?: Keypair;
    }
  ): TransactionObject {
    return new TransactionObject(
      payer,
      [
        types.aggregatorSetAuthority(
          this.program,
          { params: {} },
          {
            aggregator: this.publicKey,
            authority: params.authority ? params.authority.publicKey : payer,
            newAuthority: params.newAuthority,
          }
        ),
      ],
      params.authority ? [params.authority] : []
    );
  }

  public async setAuthority(params: {
    newAuthority: PublicKey;
    authority?: Keypair;
  }): Promise<TransactionSignature> {
    const setAuthorityTxn = this.setAuthorityInstruction(
      this.program.walletPubkey,
      params
    );
    const txnSignature = await this.program.signAndSend(setAuthorityTxn);
    return txnSignature;
  }

  public updateJobWeightInstruction(
    payer: PublicKey,
    params: {
      job: JobAccount;
      jobIdx: number;
      weight: number;
      authority?: Keypair;
    }
  ): TransactionObject {
    const removeJob = this.removeJobInstruction(payer, {
      job: params.job,
      jobIdx: params.jobIdx,
      authority: params.authority,
    });
    const addJob = this.addJobInstruction(payer, {
      job: params.job,
      weight: params.weight,
      authority: params.authority,
    });
    return removeJob.combine(addJob);
  }

  public async updateJobWeight(params: {
    job: JobAccount;
    jobIdx: number;
    weight: number;
    authority?: Keypair;
  }): Promise<TransactionSignature> {
    const transaction = this.updateJobWeightInstruction(
      this.program.walletPubkey,
      params
    );
    const signature = await this.program.signAndSend(transaction);
    return signature;
  }

  public removeJobInstruction(
    payer: PublicKey,
    params: {
      job: JobAccount;
      jobIdx: number;
      authority?: Keypair;
    }
  ): TransactionObject {
    const authority = params.authority ? params.authority.publicKey : payer;
    const removeJobIxn = types.aggregatorRemoveJob(
      this.program,
      { params: { jobIdx: params.jobIdx } },
      {
        aggregator: this.publicKey,
        authority: authority,
        job: params.job.publicKey,
      }
    );
    return new TransactionObject(
      payer,
      [removeJobIxn],
      params.authority ? [params.authority] : []
    );
  }

  public async removeJob(params: {
    job: JobAccount;
    jobIdx: number;
    authority?: Keypair;
  }): Promise<TransactionSignature> {
    const removeJobTxn = this.removeJobInstruction(
      this.program.walletPubkey,
      params
    );
    const txnSignature = await this.program.signAndSend(removeJobTxn);
    return txnSignature;
  }

  public async openRoundInstruction(
    payer: PublicKey,
    params?: { payoutWallet?: PublicKey }
  ): Promise<TransactionObject> {
    const aggregatorData = await this.loadData();
    const queueAccount = new QueueAccount(
      this.program,
      aggregatorData.queuePubkey
    );
    const queue = await queueAccount.loadData();

    const {
      permissionAccount,
      permissionBump,
      leaseAccount,
      leaseBump,
      leaseEscrow,
    } = this.getAccounts({
      queueAccount: queueAccount,
      queueAuthority: queue.authority,
    });

    const ixns: Array<TransactionInstruction> = [];

    const payoutWallet =
      params?.payoutWallet ?? this.program.mint.getAssociatedAddress(payer);
    const payoutWalletAccountInfo =
      await this.program.connection.getAccountInfo(payoutWallet);
    if (payoutWalletAccountInfo === null) {
      const [createTokenAccountTxn] =
        this.program.mint.createAssocatedUserInstruction(payer);
      ixns.push(...createTokenAccountTxn.ixns);
    }

    ixns.push(
      types.aggregatorOpenRound(
        this.program,
        {
          params: {
            stateBump: this.program.programState.bump,
            leaseBump,
            permissionBump,
            jitter: 0,
          },
        },
        {
          aggregator: this.publicKey,
          lease: leaseAccount.publicKey,
          oracleQueue: queueAccount.publicKey,
          queueAuthority: queue.authority,
          permission: permissionAccount.publicKey,
          escrow: leaseEscrow,
          programState: this.program.programState.publicKey,
          payoutWallet: payoutWallet,
          tokenProgram: TOKEN_PROGRAM_ID,
          dataBuffer: queue.dataBuffer,
          mint: this.program.mint.address,
        }
      )
    );

    return new TransactionObject(payer, ixns, []);
  }

  public async openRound(params?: {
    payoutWallet?: PublicKey;
  }): Promise<TransactionSignature> {
    const openRoundTxn = await this.openRoundInstruction(
      this.program.walletPubkey,
      params
    );
    const txnSignature = await this.program.signAndSend(openRoundTxn);
    return txnSignature;
  }

  public saveResultInstruction(
    params: {
      queueAccount: QueueAccount;
      queueAuthority: PublicKey;
      feedPermission: [PermissionAccount, number];
      jobs: Array<OracleJob>;
      // oracle
      oracleAccount: OracleAccount;
      oracleAuthority: PublicKey;
      oracleIdx: number;
      oraclePermission: [PermissionAccount, number];
      // response
      value: Big;
      minResponse: Big;
      maxResponse: Big;
      // token
      mint: PublicKey;
      payoutWallet: PublicKey;
      lease: [LeaseAccount, number];
      leaseEscrow: PublicKey;
    } & Partial<{
      error?: boolean;
      historyBuffer?: PublicKey;
      oracles: Array<types.OracleAccountData>;
    }>
  ): TransactionInstruction {
    const [leaseAccount, leaseBump] = params.lease;
    const [feedPermissionAccount, feedPermissionBump] = params.feedPermission;
    const [oraclePermissionAccount, oraclePermissionBump] =
      params.oraclePermission;

    if (params.oracleIdx < 0) {
      throw new Error('Failed to find oracle in current round');
    }

    return types.aggregatorSaveResult(
      this.program,
      {
        params: {
          oracleIdx: params.oracleIdx,
          error: params.error ?? false,
          value: types.SwitchboardDecimal.fromBig(params.value).borsh,
          jobsChecksum: [...this.produceJobsHash(params.jobs).digest()],
          minResponse: types.SwitchboardDecimal.fromBig(params.minResponse)
            .borsh,
          maxResponse: types.SwitchboardDecimal.fromBig(params.maxResponse)
            .borsh,
          feedPermissionBump: feedPermissionBump,
          oraclePermissionBump: oraclePermissionBump,
          leaseBump: leaseBump,
          stateBump: this.program.programState.bump,
        },
      },
      {
        aggregator: this.publicKey,
        oracle: params.oracleAccount.publicKey,
        oracleAuthority: params.oracleAuthority,
        oracleQueue: params.queueAccount.publicKey,
        queueAuthority: params.queueAuthority,
        feedPermission: feedPermissionAccount.publicKey,
        oraclePermission: oraclePermissionAccount.publicKey,
        lease: leaseAccount.publicKey,
        escrow: params.leaseEscrow,
        tokenProgram: spl.TOKEN_PROGRAM_ID,
        programState: this.program.programState.publicKey,
        historyBuffer: params.historyBuffer ?? this.publicKey,
        mint: params.mint,
      }
    );
  }

  public async saveResult(
    params: {
      jobs: Array<OracleJob>;
      oracleAccount: OracleAccount;
      value: Big;
      minResponse: Big;
      maxResponse: Big;
    } & Partial<{
      queueAccount: QueueAccount;
      queueAuthority: PublicKey;
      aggregator: types.AggregatorAccountData;
      feedPermission: [PermissionAccount, number];
      historyBuffer: PublicKey;
      oracleIdx: number;
      oracleAuthority: PublicKey;
      oraclePermission: [PermissionAccount, number];
      error: boolean;
      mint: PublicKey;
      payoutWallet: PublicKey;
      lease: [LeaseAccount, number];
      leaseEscrow: PublicKey;
      oracles: Array<types.OracleAccountData>;
    }>
  ): Promise<TransactionSignature> {
    const payer = this.program.walletPubkey;
    const aggregator = params.aggregator ?? (await this.loadData());

    const remainingAccounts: Array<PublicKey> = [];
    for (let i = 0; i < aggregator.oracleRequestBatchSize; ++i) {
      remainingAccounts.push(aggregator.currentRound.oraclePubkeysData[i]);
    }
    for (const oracle of params?.oracles ??
      (await this.loadCurrentRoundOracles(aggregator)).map(a => a.state)) {
      remainingAccounts.push(oracle.tokenAccount);
    }
    remainingAccounts.push(this.slidingWindowKey);

    const oracleIdx =
      params.oracleIdx ??
      aggregator.currentRound.oraclePubkeysData
        .slice(0, aggregator.oracleRequestBatchSize)
        .findIndex(o => o.equals(params.oracleAccount.publicKey));

    if (oracleIdx < 0) {
      throw new Error('Failed to find oracle in current round');
    }

    const ixns: Array<TransactionInstruction> = [];

    const payoutWallet =
      params.payoutWallet ?? this.program.mint.getAssociatedAddress(payer);
    const payoutWalletAccountInfo =
      await this.program.connection.getAccountInfo(payoutWallet);
    if (payoutWalletAccountInfo === null) {
      const [createTokenAccountTxn] =
        this.program.mint.createAssocatedUserInstruction(payer);
      ixns.push(...createTokenAccountTxn.ixns);
    }

    const queueAccount =
      params.queueAccount ??
      new QueueAccount(this.program, aggregator.queuePubkey);

    let queueAuthority = params.queueAuthority;
    let mint = params.mint;
    if (!queueAuthority || !mint) {
      const queue = await queueAccount.loadData();
      queueAuthority = queue.authority;
      mint = mint ?? queue.mint ?? spl.NATIVE_MINT;
    }

    const {
      permissionAccount,
      permissionBump,
      leaseAccount,
      leaseBump,
      leaseEscrow,
    } = this.getAccounts({
      queueAccount: queueAccount,
      queueAuthority: queueAuthority,
    });

    const [oraclePermissionAccount, oraclePermissionBump] =
      params.oraclePermission ??
      PermissionAccount.fromSeed(
        this.program,
        queueAuthority,
        queueAccount.publicKey,
        params.oracleAccount.publicKey
      );
    try {
      await oraclePermissionAccount.loadData();
    } catch (_) {
      throw new Error(
        'A requested oracle permission pda account has not been initialized.'
      );
    }

    const historyBuffer = params.historyBuffer ?? aggregator.historyBuffer;

    const saveResultIxn = this.saveResultInstruction({
      queueAccount,
      queueAuthority,
      feedPermission: [permissionAccount, permissionBump],
      jobs: params.jobs,
      historyBuffer: historyBuffer.equals(PublicKey.default)
        ? undefined
        : historyBuffer,
      oracleAccount: params.oracleAccount,
      oracleAuthority: (await params.oracleAccount.loadData()).oracleAuthority,
      oracleIdx,
      oraclePermission: [oraclePermissionAccount, oraclePermissionBump],
      value: params.value,
      minResponse: params.minResponse,
      maxResponse: params.maxResponse,
      error: params.error ?? false,
      mint,
      payoutWallet: payoutWallet,
      lease: [leaseAccount, leaseBump],
      leaseEscrow: leaseEscrow,
    });

    // add remaining accounts
    saveResultIxn.keys.push(
      ...remainingAccounts.map((pubkey): AccountMeta => {
        return { isSigner: false, isWritable: true, pubkey };
      })
    );

    ixns.push(saveResultIxn);
    const saveResultTxn = new TransactionObject(
      this.program.walletPubkey,
      ixns,
      []
    );
    const txnSignature = await this.program.signAndSend(saveResultTxn);
    return txnSignature;
  }

  public async fetchAccounts(
    _aggregator?: types.AggregatorAccountData,
    _queueAccount?: QueueAccount,
    _queue?: types.OracleQueueAccountData
  ): Promise<AggregatorAccounts> {
    const aggregator = _aggregator ?? (await this.loadData());
    const queueAccount =
      _queueAccount ?? new QueueAccount(this.program, aggregator.queuePubkey);
    const queue = _queue ?? (await queueAccount.loadData());

    const {
      permissionAccount,
      permissionBump,
      leaseAccount,
      leaseEscrow,
      leaseBump,
    } = this.getAccounts({
      queueAccount,
      queueAuthority: queue.authority,
    });

    const jobPubkeys = aggregator.jobPubkeysData.slice(
      0,
      aggregator.jobPubkeysSize
    );

    const accountInfos = await anchor.utils.rpc.getMultipleAccounts(
      this.program.connection,
      [
        permissionAccount.publicKey,
        leaseAccount.publicKey,
        leaseEscrow,
        ...jobPubkeys,
      ]
    );

    const permissionAccountInfo = accountInfos.shift();
    if (!permissionAccountInfo || !permissionAccountInfo.account) {
      throw new Error(
        `PermissionAccount has not been created yet for this aggregator`
      );
    }
    const permission = types.PermissionAccountData.decode(
      permissionAccountInfo.account.data
    );

    const leaseAccountInfo = accountInfos.shift();
    if (!leaseAccountInfo || !leaseAccountInfo.account) {
      throw new Error(
        `LeaseAccount has not been created yet for this aggregator`
      );
    }
    const lease = types.LeaseAccountData.decode(leaseAccountInfo.account.data);

    const leaseEscrowAccountInfo = accountInfos.shift();
    if (!leaseEscrowAccountInfo || !leaseEscrowAccountInfo.account) {
      throw new Error(
        `LeaseAccount escrow has not been created yet for this aggregator`
      );
    }
    const leaseEscrowAccount = spl.unpackAccount(
      leaseEscrow,
      leaseEscrowAccountInfo.account
    );

    const jobs: Array<{
      publicKey: PublicKey;
      data: types.JobAccountData;
      tasks: Array<OracleJob.ITask>;
    }> = [];
    accountInfos.map(accountInfo => {
      if (!accountInfo || !accountInfo.account) {
        throw new Error(`Failed to fetch JobAccount`);
      }
      const job = types.JobAccountData.decode(accountInfo.account.data);
      const oracleJob = OracleJob.decodeDelimited(job.data);
      jobs.push({
        publicKey: accountInfo.publicKey,
        data: job,
        tasks: oracleJob.tasks,
      });
    });

    return {
      aggregator: {
        publicKey: this.publicKey,
        data: aggregator,
      },
      queue: {
        publicKey: queueAccount.publicKey,
        data: queue,
      },
      permission: {
        publicKey: permissionAccount.publicKey,
        bump: permissionBump,
        data: permission,
      },
      lease: {
        publicKey: leaseAccount.publicKey,
        bump: leaseBump,
        data: lease,
        balance: this.program.mint.fromTokenAmount(leaseEscrowAccount.amount),
      },
      jobs: jobs,
    };
  }

  public async toAccountsJSON(
    _aggregator?: types.AggregatorAccountData,
    _queueAccount?: QueueAccount,
    _queue?: types.OracleQueueAccountData
  ): Promise<AggregatorAccountsJSON> {
    const accounts = await this.fetchAccounts(
      _aggregator,
      _queueAccount,
      _queue
    );

    return {
      publicKey: this.publicKey,
      ...accounts.aggregator.data.toJSON(),
      queue: {
        publicKey: accounts.queue.publicKey,
        ...accounts.queue.data.toJSON(),
      },
      permission: {
        publicKey: accounts.permission.publicKey,
        bump: accounts.permission.bump,
        ...accounts.permission.data.toJSON(),
      },
      lease: {
        publicKey: accounts.lease.publicKey,
        bump: accounts.lease.bump,
        balance: accounts.lease.balance,
        ...accounts.lease.data.toJSON(),
      },
      jobs: accounts.jobs.map(j => {
        return {
          publicKey: j.publicKey,
          ...j.data.toJSON(),
          tasks: j.tasks,
        };
      }),
    };
  }

  setSlidingWindowInstruction(
    payer: PublicKey,
    params: {
      authority?: Keypair;
      mode: types.AggregatorResolutionModeKind;
    }
  ): TransactionObject {
    return new TransactionObject(
      payer,
      [
        types.aggregatorSetResolutionMode(
          this.program,
          {
            params: { mode: params.mode.discriminator },
          },
          {
            aggregator: this.publicKey,
            authority: params.authority ? params.authority.publicKey : payer,
            slidingWindow: this.slidingWindowKey,
            payer: payer,
            systemProgram: SystemProgram.programId,
          }
        ),
      ],
      params.authority ? [params.authority] : []
    );
  }

  async setSlidingWindow(params: {
    authority?: Keypair;
    mode: types.AggregatorResolutionModeKind;
  }): Promise<TransactionSignature> {
    const setSlidingWindowTxn = this.setSlidingWindowInstruction(
      this.program.walletPubkey,
      params
    );
    const txnSignature = await this.program.signAndSend(setSlidingWindowTxn);
    return txnSignature;
  }

  async openRoundAndAwaitResult(
    params?: { payoutWallet?: PublicKey } & {
      aggregator?: types.AggregatorAccountData;
    },
    timeout = 30000
  ): Promise<[types.AggregatorAccountData, TransactionSignature | undefined]> {
    const aggregator = params?.aggregator ?? (await this.loadData());
    const currentRoundOpenSlot = aggregator.latestConfirmedRound.roundOpenSlot;

    let ws: number | undefined = undefined;

    const closeWebsocket = async () => {
      if (ws !== undefined) {
        await this.program.connection.removeAccountChangeListener(ws);
        ws = undefined;
      }
    };

    const statePromise: Promise<types.AggregatorAccountData> =
      promiseWithTimeout(
        timeout,
        new Promise(
          (
            resolve: (result: types.AggregatorAccountData) => void,
            reject: (reason: string) => void
          ) => {
            ws = this.onChange(aggregator => {
              // if confirmed round slot larger than last open slot
              // AND sliding window mode or sufficient oracle results
              if (
                aggregator.latestConfirmedRound.roundOpenSlot.gt(
                  currentRoundOpenSlot
                ) &&
                (aggregator.resolutionMode.kind ===
                  types.AggregatorResolutionMode.ModeSlidingResolution.kind ||
                  (aggregator.latestConfirmedRound.numSuccess ?? 0) >=
                    aggregator.minOracleResults)
              ) {
                resolve(aggregator);
              }
            });
          }
        )
      ).finally(async () => {
        await closeWebsocket();
      });

    const openRoundSignature = await this.openRound(params).catch(
      async error => {
        await closeWebsocket();
        throw new Error(`Failed to call openRound, ${error}`);
      }
    );

    const state = await statePromise;

    await closeWebsocket();

    return [state, openRoundSignature];
  }

  /**
   * Await for the next round to close and return the aggregator round result
   *
   * @param roundOpenSlot - optional, the slot when the current round was opened. if not provided then it will be loaded.
   * @param timeout - the number of milliseconds to wait for the round to close
   *
   * @throws {string} when the timeout interval is exceeded or when the latestConfirmedRound.roundOpenSlot exceeds the target roundOpenSlot
   */
  async nextRound(
    roundOpenSlot?: anchor.BN,
    timeout = 30000
  ): Promise<types.AggregatorAccountData> {
    const slot =
      roundOpenSlot ?? (await this.loadData()).currentRound.roundOpenSlot;
    let ws: number | undefined;

    let result: types.AggregatorAccountData;
    try {
      result = await promiseWithTimeout(
        timeout,
        new Promise(
          (
            resolve: (result: types.AggregatorAccountData) => void,
            reject: (reason: string) => void
          ) => {
            ws = this.onChange(aggregator => {
              if (aggregator.latestConfirmedRound.roundOpenSlot.eq(slot)) {
                resolve(aggregator);
              }
            });
          }
        )
      );
    } finally {
      if (ws) {
        await this.program.connection.removeAccountChangeListener(ws);
      }
    }

    return result;
  }

  /**
   * Load an aggregators {@linkcode AggregatorHistoryBuffer}.
   * @return the list of historical samples attached to the aggregator.
   */
  async loadHistory(): Promise<Array<types.AggregatorHistoryRow>> {
    if (!this.history) {
      this.history = new AggregatorHistoryBuffer(
        this.program,
        (await this.loadData()).historyBuffer
      );
    }

    const history = await this.history.loadData();

    return history;
  }

  static async fetchMultiple(
    program: SwitchboardProgram,
    publicKeys: Array<PublicKey>,
    commitment: Commitment = 'confirmed'
  ): Promise<
    Array<{
      account: AggregatorAccount;
      data: types.AggregatorAccountData;
    }>
  > {
    const aggregators: Array<{
      account: AggregatorAccount;
      data: types.AggregatorAccountData;
    }> = [];

    const accountInfos = await anchor.utils.rpc.getMultipleAccounts(
      program.connection,
      publicKeys,
      commitment
    );

    for (const accountInfo of accountInfos) {
      if (!accountInfo?.publicKey) {
        continue;
      }
      try {
        const account = new AggregatorAccount(program, accountInfo.publicKey);
        const data = types.AggregatorAccountData.decode(
          accountInfo.account.data
        );
        aggregators.push({ account, data });
        // eslint-disable-next-line no-empty
      } catch {}
    }

    return aggregators;
  }
}

/**
 * Parameters to initialize an aggregator account.
 */
export interface AggregatorInitParams {
  /**
   *  Name of the aggregator to store on-chain.
   */
  name?: string;
  /**
   *  Metadata of the aggregator to store on-chain.
   */
  metadata?: string;
  /**
   *  Number of oracles to request on aggregator update.
   */
  batchSize: number;
  /**
   *  Minimum number of oracle responses required before a round is validated.
   */
  minRequiredOracleResults: number;
  /**
   *  Minimum number of feed jobs suggested to be successful before an oracle
   *  sends a response.
   */
  minRequiredJobResults: number;
  /**
   *  Minimum number of seconds required between aggregator rounds.
   */
  minUpdateDelaySeconds: number;
  /**
   *  unix_timestamp for which no feed update will occur before.
   */
  startAfter?: number;
  /**
   *  Change percentage required between a previous round and the current round.
   *  If variance percentage is not met, reject new oracle responses.
   */
  varianceThreshold?: number;
  /**
   *  Number of seconds for which, even if the variance threshold is not passed,
   *  accept new responses from oracles.
   */
  forceReportPeriod?: number;
  /**
   *  unix_timestamp after which funds may be withdrawn from the aggregator.
   *  null/undefined/0 means the feed has no expiration.
   */
  expiration?: number;
  /**
   *  If true, this aggregator is disallowed from being updated by a crank on the queue.
   */
  disableCrank?: boolean;
  /**
   *  Optional pre-existing keypair to use for aggregator initialization.
   */
  keypair?: Keypair;
  /**
   *  If included, this keypair will be the aggregator authority rather than
   *  the aggregator keypair.
   */
  authority?: PublicKey;
  /**
   *  The queue to which this aggregator will be linked
   */
  queueAccount: QueueAccount;
  /**
   * The authority of the queue.
   */
  queueAuthority: PublicKey;
}

export interface AggregatorSetQueueParams {
  queueAccount: QueueAccount;
  authority?: Keypair;
}

/**
 * Parameters required to open an aggregator round
 */
export interface AggregatorOpenRoundParams {
  /**
   *  The oracle queue from which oracles are assigned this update.
   */
  oracleQueueAccount: QueueAccount;
  /**
   *  The token wallet which will receive rewards for calling update on this feed.
   */
  payoutWallet: PublicKey;
}

/**
 * Parameters for creating and setting a history buffer for an aggregator
 */
export interface AggregatorSetHistoryBufferParams {
  /**
   * Authority keypair for the aggregator.
   */
  authority?: Keypair;
  /**
   * Number of elements for the history buffer to fit.
   */
  size: number;
}

/**
 * Parameters for which oracles must submit for responding to update requests.
 */
export interface AggregatorSaveResultParams {
  /**
   *  Index in the list of oracles in the aggregator assigned to this round update.
   */
  oracleIdx: number;
  /**
   *  Reports that an error occured and the oracle could not send a value.
   */
  error: boolean;
  /**
   *  Value the oracle is responding with for this update.
   */
  value: Big;
  /**
   *  The minimum value this oracle has seen this round for the jobs listed in the
   *  aggregator.
   */
  minResponse: Big;
  /**
   *  The maximum value this oracle has seen this round for the jobs listed in the
   *  aggregator.
   */
  maxResponse: Big;
  /**
   *  List of OracleJobs that were performed to produce this result.
   */
  jobs: Array<OracleJob>;
  /**
   *  Authority of the queue the aggregator is attached to.
   */
  queueAuthority: PublicKey;
  /**
   *  Program token mint.
   */
  tokenMint: PublicKey;
  /**
   *  List of parsed oracles.
   */
  oracles: Array<types.OracleAccountData>;
}

export type AggregatorAccountsJSON = types.AggregatorAccountDataJSON & {
  publicKey: PublicKey;
  queue: types.OracleQueueAccountDataJSON & { publicKey: PublicKey };
  permission: types.PermissionAccountDataJSON & {
    bump: number;
    publicKey: PublicKey;
  };
  lease: types.LeaseAccountDataJSON & { bump: number; publicKey: PublicKey } & {
    balance: number;
  };
  jobs: Array<
    types.JobAccountDataJSON & {
      publicKey: PublicKey;
      tasks: Array<OracleJob.ITask>;
    }
  >;
};
export type AggregatorAccounts = {
  aggregator: {
    publicKey: PublicKey;
    data: types.AggregatorAccountData;
  };
  queue: {
    publicKey: PublicKey;
    data: types.OracleQueueAccountData;
  };
  permission: {
    publicKey: PublicKey;
    bump: number;
    data: types.PermissionAccountData;
  };
  lease: {
    publicKey: PublicKey;
    bump: number;
    balance: number;
    data: types.LeaseAccountData;
  };
  jobs: Array<{
    publicKey: PublicKey;
    data: types.JobAccountData;
    tasks: Array<OracleJob.ITask>;
  }>;
};
