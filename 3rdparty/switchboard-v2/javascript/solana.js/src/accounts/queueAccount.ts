import * as anchor from '@project-serum/anchor';
import * as spl from '@solana/spl-token';
import {
  AccountInfo,
  Commitment,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  TransactionSignature,
} from '@solana/web3.js';
import { SwitchboardDecimal, toUtf8 } from '@switchboard-xyz/common';
import Big from 'big.js';
import { BN } from 'bn.js';
import * as errors from '../errors';
import * as types from '../generated';
import {
  PermitOracleHeartbeat,
  PermitOracleQueueUsage,
} from '../generated/types/SwitchboardPermission';
import { SwitchboardProgram } from '../program';
import { SolanaClock } from '../SolanaClock';
import { TransactionObject } from '../transaction';
import { Account, OnAccountChangeCallback } from './account';
import { AggregatorAccount, AggregatorInitParams } from './aggregatorAccount';
import { AggregatorHistoryBuffer } from './aggregatorHistoryBuffer';
import { BufferRelayerAccount, BufferRelayerInit } from './bufferRelayAccount';
import { CrankAccount, CrankInitParams } from './crankAccount';
import { JobAccount, JobInitParams } from './jobAccount';
import { LeaseAccount } from './leaseAccount';
import {
  OracleAccount,
  OracleInitParams,
  OracleStakeParams,
} from './oracleAccount';
import { PermissionAccount, PermissionSetParams } from './permissionAccount';
import { QueueDataBuffer } from './queueDataBuffer';
import { VrfAccount, VrfInitParams } from './vrfAccount';

/**
 * Account type representing an oracle queue's configuration along with a buffer account holding a list of oracles that are actively heartbeating.
 *
 * A QueueAccount is responsible for allocating update requests to it's round robin queue of {@linkcode OracleAccount}'s.
 *
 * Data: {@linkcode types.OracleQueueAccountData}
 *
 * Buffer: {@linkcode QueueDataBuffer}
 */
export class QueueAccount extends Account<types.OracleQueueAccountData> {
  static accountName = 'OracleQueueAccountData';

  /** The {@linkcode QueueDataBuffer} storing a list of oracle's that are actively heartbeating */
  dataBuffer?: QueueDataBuffer;

  public static size = 1269;

  /**
   * Get the size of an {@linkcode QueueAccount} on-chain.
   */
  public readonly size = this.program.account.oracleQueueAccountData.size;

  /**
   * Returns the queue's name buffer in a stringified format.
   */
  public static getName = (queue: types.OracleQueueAccountData) =>
    toUtf8(queue.name);

  /**
   * Returns the queue's metadata buffer in a stringified format.
   */
  public static getMetadata = (queue: types.OracleQueueAccountData) =>
    toUtf8(queue.metadata);
  /** Load an existing QueueAccount with its current on-chain state */
  public static async load(
    program: SwitchboardProgram,
    publicKey: PublicKey | string
  ): Promise<[QueueAccount, types.OracleQueueAccountData]> {
    const account = new QueueAccount(
      program,
      typeof publicKey === 'string' ? new PublicKey(publicKey) : publicKey
    );
    const state = await account.loadData();
    return [account, state];
  }

  public static default(): types.OracleQueueAccountData {
    const buffer = Buffer.alloc(1269, 0);
    types.OracleQueueAccountData.discriminator.copy(buffer, 0);
    return types.OracleQueueAccountData.decode(buffer);
  }

  public static createMock(
    programId: PublicKey,
    data: Partial<types.OracleQueueAccountData>,
    options?: {
      lamports?: number;
      rentEpoch?: number;
    }
  ): AccountInfo<Buffer> {
    const fields: types.OracleQueueAccountDataFields = {
      ...QueueAccount.default(),
      ...data,
      // any cleanup actions here
    };
    const state = new types.OracleQueueAccountData(fields);

    const buffer = Buffer.alloc(QueueAccount.size, 0);
    types.OracleQueueAccountData.discriminator.copy(buffer, 0);
    types.OracleQueueAccountData.layout.encode(state, buffer, 8);

    return {
      executable: false,
      owner: programId,
      lamports: options?.lamports ?? 1 * LAMPORTS_PER_SOL,
      data: buffer,
      rentEpoch: options?.rentEpoch ?? 0,
    };
  }

  /**
   * Invoke a callback each time a QueueAccount's data has changed on-chain.
   * @param callback - the callback invoked when the queues state changes
   * @param commitment - optional, the desired transaction finality. defaults to 'confirmed'
   * @returns the websocket subscription id
   */
  onChange(
    callback: OnAccountChangeCallback<types.OracleQueueAccountData>,
    commitment: Commitment = 'confirmed'
  ): number {
    return this.program.connection.onAccountChange(
      this.publicKey,
      accountInfo =>
        callback(types.OracleQueueAccountData.decode(accountInfo.data)),
      commitment
    );
  }

  /**
   * Retrieve and decode the {@linkcode types.OracleQueueAccountData} stored in this account.
   */
  public async loadData(): Promise<types.OracleQueueAccountData> {
    const data = await types.OracleQueueAccountData.fetch(
      this.program,
      this.publicKey
    );
    if (data === null)
      throw new errors.AccountNotFoundError('Queue', this.publicKey);
    this.dataBuffer = new QueueDataBuffer(this.program, data.dataBuffer);
    return data;
  }

  /**
   * Get the spl Mint associated with this {@linkcode QueueAccount}.
   */
  public get mint(): spl.Mint {
    return this.program.mint.mint;
  }

  /**
   * Creates a transaction object with oracleQueueInit instructions.
   *
   * @param program The SwitchboardProgram.
   *
   * @param payer - the publicKey of the account that will pay for the new accounts. Will also be used as the account authority if no other authority is provided.
   *
   * @param params oracle queue configuration parameters.
   *
   * @return Transaction signature and the newly created QueueAccount.
   *
   * Basic usage example:
   *
   * ```ts
   * import { QueueAccount } from '@switchboard-xyz/solana.js';
   * const [queueAccount, queueInitTxn] = await QueueAccount.createInstructions(program, payer, {
        name: 'My Queue',
        metadata: 'Top Secret',
        queueSize: 100,
        reward: 0.00001337,
        minStake: 10,
        oracleTimeout: 60,
        slashingEnabled: false,
        unpermissionedFeeds: true,
        unpermissionedVrf: true,
        enableBufferRelayers: false,
   * });
   * const queueInitSignature = await program.signAndSend(queueInitTxn);
   * const queue = await queueAccount.loadData();
   * ```
   */
  public static async createInstructions(
    program: SwitchboardProgram,
    payer: PublicKey,
    params: QueueInitParams
  ): Promise<[QueueAccount, TransactionObject]> {
    const keypair = params.keypair ?? Keypair.generate();
    program.verifyNewKeypair(keypair);

    const dataBuffer = params.dataBufferKeypair ?? Keypair.generate();
    program.verifyNewKeypair(dataBuffer);

    const queueAccount = new QueueAccount(program, keypair.publicKey);
    queueAccount.dataBuffer = new QueueDataBuffer(
      program,
      dataBuffer.publicKey
    );

    const queueSize = params.queueSize ?? 500;
    const queueDataSize = QueueDataBuffer.getAccountSize(queueSize);

    const reward = program.mint.toTokenAmountBN(params.reward);
    const minStake = program.mint.toTokenAmountBN(params.minStake);

    const txn = new TransactionObject(
      payer,
      [
        SystemProgram.createAccount({
          fromPubkey: payer,
          newAccountPubkey: dataBuffer.publicKey,
          space: queueDataSize,
          lamports: await program.connection.getMinimumBalanceForRentExemption(
            queueDataSize
          ),
          programId: program.programId,
        }),
        types.oracleQueueInit(
          program,
          {
            params: {
              name: Array.from(
                new Uint8Array(Buffer.from(params.name ?? '').slice(0, 32))
              ),
              metadata: [
                ...new Uint8Array(
                  Buffer.from(params.metadata ?? '').slice(0, 64)
                ),
              ],
              reward: reward,
              minStake: minStake,
              feedProbationPeriod: params.feedProbationPeriod ?? 0,
              oracleTimeout: params.oracleTimeout ?? 180,
              slashingEnabled: params.slashingEnabled ?? false,
              varianceToleranceMultiplier: SwitchboardDecimal.fromBig(
                new Big(params.varianceToleranceMultiplier ?? 2)
              ),
              consecutiveFeedFailureLimit: new anchor.BN(
                params.consecutiveFeedFailureLimit ?? 1000
              ),
              consecutiveOracleFailureLimit: new anchor.BN(
                params.consecutiveOracleFailureLimit ?? 1000
              ),
              queueSize: queueSize,
              unpermissionedFeeds: params.unpermissionedFeeds ?? false,
              unpermissionedVrf: params.unpermissionedVrf ?? false,
              enableBufferRelayers: params.enableBufferRelayers ?? false,
            },
          },
          {
            oracleQueue: queueAccount.publicKey,
            authority: params.authority ?? payer,
            buffer: dataBuffer.publicKey,
            systemProgram: SystemProgram.programId,
            payer,
            mint: program.mint.address,
          }
        ),
      ],
      [dataBuffer, keypair]
    );

    return [queueAccount, txn];
  }

  /**
   * Creates an oracle queue on-chain and return the transaction signature and created account resource.
   *
   * @param program The SwitchboardProgram.
   *
   * @param params oracle queue configuration parameters.
   *
   * @return Transaction signature and the newly created QueueAccount.
   *
   * Basic usage example:
   *
   * ```ts
   * import { QueueAccount } from '@switchboard-xyz/solana.js';
   * const [queueAccount, txnSignature] = await QueueAccount.create(program, {
        name: 'My Queue',
        metadata: 'Top Secret',
        queueSize: 100,
        reward: 0.00001337,
        minStake: 10,
        oracleTimeout: 60,
        slashingEnabled: false,
        unpermissionedFeeds: true,
        unpermissionedVrf: true,
        enableBufferRelayers: false,
   * });
   * const queue = await queueAccount.loadData();
   * ```
   */
  public static async create(
    program: SwitchboardProgram,
    params: QueueInitParams
  ): Promise<[QueueAccount, string]> {
    const [account, txnObject] = await this.createInstructions(
      program,
      program.walletPubkey,
      params
    );
    const txnSignature = await program.signAndSend(txnObject);
    return [account, txnSignature];
  }

  /**
   * Creates a transaction object with oracleInit instructions for the given QueueAccount.
   *
   * @param payer - the publicKey of the account that will pay for the new accounts. Will also be used as the account authority if no other authority is provided.
   *
   * @param params - the oracle configuration parameters.
   *
   * @return Transaction signature and the newly created OracleAccount.
   *
   * Basic usage example:
   *
   * ```ts
   * import { QueueAccount } from '@switchboard-xyz/solana.js';
   * const queueAccount = new QueueAccount(program, queuePubkey);
   * const [oracleAccount, oracleInitTxn] = await queueAccount.createOracleInstructions(payer, {
   *  name: "My Oracle",
   *  metadata: "Oracle #1"
   * });
   * const oracleInitSignature = await program.signAndSend(oracleInitTxn);
   * const oracle = await oracleAccount.loadData();
   * ```
   */
  public async createOracleInstructions(
    /** The publicKey of the account that will pay for the new accounts. Will also be used as the account authority if no other authority is provided. */
    payer: PublicKey,
    params: OracleInitParams &
      OracleStakeParams &
      Partial<PermissionSetParams> & {
        queueAuthorityPubkey?: PublicKey;
      }
  ): Promise<[OracleAccount, Array<TransactionObject>]> {
    const queueAuthorityPubkey = params.queueAuthority
      ? params.queueAuthority.publicKey
      : params.queueAuthorityPubkey ?? (await this.loadData()).authority;

    const [oracleAccount, createOracleTxnObject] =
      await OracleAccount.createInstructions(this.program, payer, {
        ...params,
        queueAccount: this,
      });

    const [permissionAccount, createPermissionTxnObject] =
      PermissionAccount.createInstruction(this.program, payer, {
        granter: this.publicKey,
        grantee: oracleAccount.publicKey,
        authority: queueAuthorityPubkey,
      });

    if (
      params.enable &&
      (params.queueAuthority || queueAuthorityPubkey.equals(payer))
    ) {
      const permissionSetTxn = permissionAccount.setInstruction(payer, {
        permission: new PermitOracleHeartbeat(),
        enable: true,
        queueAuthority: params.queueAuthority,
      });
      createPermissionTxnObject.combine(permissionSetTxn);
    }

    return [
      oracleAccount,
      TransactionObject.pack([
        createOracleTxnObject,
        createPermissionTxnObject,
      ]),
    ];
  }

  /**
   * Creates a new {@linkcode OracleAccount}.
   *
   * @param params - the oracle configuration parameters.
   *
   * @return Transaction signature and the newly created OracleAccount.
   *
   * Basic usage example:
   *
   * ```ts
   * import { QueueAccount } from '@switchboard-xyz/solana.js';
   * const queueAccount = new QueueAccount(program, queuePubkey);
   * const [oracleAccount, oracleInitSignature] = await queueAccount.createOracle({
   *  name: "My Oracle",
   *  metadata: "Oracle #1"
   * });
   * const oracle = await oracleAccount.loadData();
   * ```
   */
  public async createOracle(
    params: OracleInitParams &
      OracleStakeParams &
      Partial<PermissionSetParams> & {
        queueAuthorityPubkey?: PublicKey;
      }
  ): Promise<[OracleAccount, Array<TransactionSignature>]> {
    const signers: Keypair[] = [];

    const queue = await this.loadData();

    if (
      params.queueAuthority &&
      params.queueAuthority.publicKey.equals(queue.authority)
    ) {
      signers.push(params.queueAuthority);
    }

    const [oracleAccount, txn] = await this.createOracleInstructions(
      this.program.walletPubkey,
      params
    );

    const signatures = await this.program.signAndSendAll(txn);

    return [oracleAccount, signatures];
  }

  /**
   * Create a new {@linkcode TransactionObject} constaining the instructions and signers needed to create a new {@linkcode AggregatorAccount} for the queue along with its {@linkcode PermissionAccount} and {@linkcode LeaseAccount}.
   *
   * @param payer - the publicKey of the account that will pay for the new accounts. Will also be used as the account authority if no other authority is provided.
   *
   * @param params - the aggregatorInit, jobInit, permissionInit, permissionSet, leaseInit, and crankPush configuration parameters.
   *
   * Optionally, specify a crankPubkey in order to push it onto an existing {@linkcode CrankAccount}.
   *
   * Optionally, enable the permissions by setting a queueAuthority keypair along with the enable boolean set to true.
   *
   * ```ts
   * import { QueueAccount } from '@switchboard-xyz/solana.js';
   * const queueAccount = new QueueAccount(program, queuePubkey);
   * const [aggregatorAccount, aggregatorInitTxnObject] =
      await queueAccount.createFeedInstructions({
        enable: true, // not needed if queue has unpermissionedFeedsEnabled
        queueAuthority: queueAuthority, // not needed if queue has unpermissionedFeedsEnabled
        batchSize: 1,
        minRequiredOracleResults: 1,
        minRequiredJobResults: 1,
        minUpdateDelaySeconds: 60,
        fundAmount: 2.5, // deposit 2.5 wSOL into the leaseAccount escrow
        jobs: [
          { pubkey: jobAccount.publicKey },
          {
            weight: 2,
            data: OracleJob.encodeDelimited(
              OracleJob.fromObject({
                tasks: [
                  {
                    valueTask: {
                      value: 1,
                    },
                  },
                ],
              })
            ).finish(),
          },
        ],
      });
      const aggregatorInitSignatures = await this.program.signAndSendAll(txns);
   * ```
   */
  public async createFeedInstructions(
    payer: PublicKey,
    params: Omit<
      Omit<Omit<AggregatorInitParams, 'queueAccount'>, 'queueAuthority'>,
      'authority'
    > & {
      authority?: Keypair;
      crankPubkey?: PublicKey;
      historyLimit?: number;
    } & {
      // lease params
      fundAmount?: number;
      funderAuthority?: Keypair;
      funderTokenAccount?: PublicKey;
    } & Partial<PermissionSetParams> & {
        // job params
        jobs?: Array<{ pubkey: PublicKey; weight?: number } | JobInitParams>;
      } & {
        queueAuthorityPubkey?: PublicKey;
      }
  ): Promise<[AggregatorAccount, TransactionObject[]]> {
    const queueAuthorityPubkey = params.queueAuthority
      ? params.queueAuthority.publicKey
      : params.queueAuthorityPubkey ?? (await this.loadData()).authority;

    const pre: TransactionObject[] = [];
    const txns: TransactionObject[] = [];
    const post: TransactionObject[] = [];

    // getOrCreate token account for
    const userTokenAddress = this.program.mint.getAssociatedAddress(payer);
    const userTokenAccountInfo = await this.program.connection.getAccountInfo(
      userTokenAddress
    );

    if (userTokenAccountInfo === null) {
      const [createTokenAccount] =
        this.program.mint.createAssocatedUserInstruction(payer);
      pre.push(createTokenAccount);
    }

    // create / load jobs
    const jobs: { job: JobAccount; weight: number }[] = [];
    if (params.jobs && Array.isArray(params.jobs)) {
      for await (const job of params.jobs) {
        if ('data' in job) {
          const [jobAccount, jobInit] = JobAccount.createInstructions(
            this.program,
            payer,
            {
              data: job.data,
              name: job.name ?? '',
              authority: job.authority ?? payer,
              expiration: job.expiration,
              variables: job.variables,
              keypair: job.keypair,
            }
          );
          pre.push(...jobInit);
          jobs.push({ job: jobAccount, weight: job.weight ?? 1 });
        } else if ('pubkey' in job) {
          const jobAccount = new JobAccount(this.program, job.pubkey);
          // should we verify its a valid job account?
          jobs.push({ job: jobAccount, weight: job.weight ?? 1 });
        } else {
          throw new Error(`Failed to create job account ${job}`);
        }
      }
    }

    const [aggregatorAccount, aggregatorInit] =
      await AggregatorAccount.createInstruction(this.program, payer, {
        ...params,
        queueAccount: this,
        queueAuthority: queueAuthorityPubkey,
        keypair: params.keypair,
        authority: params.authority ? params.authority.publicKey : undefined,
      });

    txns.push(aggregatorInit);

    const leaseInit = (
      await LeaseAccount.createInstructions(this.program, payer, {
        loadAmount: params.fundAmount,
        funderTokenAccount: params.funderTokenAccount ?? userTokenAddress,
        funderAuthority: params.funderAuthority ?? undefined,
        aggregatorAccount: aggregatorAccount,
        queueAccount: this,
        jobAuthorities: [], // create lease before adding jobs to skip this step
        jobPubkeys: [],
      })
    )[1];
    txns.push(leaseInit);

    // create permission account
    const [permissionAccount, permissionInit] =
      PermissionAccount.createInstruction(this.program, payer, {
        granter: this.publicKey,
        authority: queueAuthorityPubkey,
        grantee: aggregatorAccount.publicKey,
      });

    // // set permissions if needed
    if (
      params.enable &&
      (params.queueAuthority || queueAuthorityPubkey.equals(payer))
    ) {
      const permissionSetTxn = permissionAccount.setInstruction(payer, {
        permission: new PermitOracleQueueUsage(),
        enable: true,
        queueAuthority: params.queueAuthority,
      });
      permissionInit.combine(permissionSetTxn);
    }

    txns.push(permissionInit);

    for await (const { job, weight } of jobs) {
      const addJobTxn = aggregatorAccount.addJobInstruction(payer, {
        job: job,
        weight: weight,
        authority: params.authority,
      });
      post.push(addJobTxn);
    }

    if (params.crankPubkey) {
      const crankAccount = new CrankAccount(this.program, params.crankPubkey);
      post.push(
        await crankAccount.pushInstruction(this.program.walletPubkey, {
          aggregatorAccount: aggregatorAccount,
        })
      );
    }

    if (params.historyLimit && params.historyLimit > 0) {
      const historyBufferInit = (
        await AggregatorHistoryBuffer.createInstructions(this.program, payer, {
          aggregatorAccount,
          maxSamples: params.historyLimit,
        })
      )[1];
      post.push(historyBufferInit);
    }

    const packed = TransactionObject.pack([
      ...TransactionObject.pack(pre),
      ...TransactionObject.pack(txns),
      ...TransactionObject.pack(post),
    ]);

    return [aggregatorAccount, packed];
  }

  /**
   * Create a new {@linkcode AggregatorAccount} for the queue, along with its {@linkcode PermissionAccount} and {@linkcode LeaseAccount}.
   *
   * Optionally, specify a crankPubkey in order to push it onto an existing {@linkcode CrankAccount}.
   *
   * Optionally, enable the permissions by setting a queueAuthority keypair along with the enable boolean set to true.
   *
   * ```ts
   * import { QueueAccount } from '@switchboard-xyz/solana.js';
   * const queueAccount = new QueueAccount(program, queuePubkey);
   * const [aggregatorAccount, aggregatorInitSignatures] =
      await queueAccount.createFeed({
        enable: true, // not needed if queue has unpermissionedFeedsEnabled
        queueAuthority: queueAuthority, // not needed if queue has unpermissionedFeedsEnabled
        batchSize: 1,
        minRequiredOracleResults: 1,
        minRequiredJobResults: 1,
        minUpdateDelaySeconds: 60,
        fundAmount: 2.5, // deposit 2.5 wSOL into the leaseAccount escrow
        jobs: [
          { pubkey: jobAccount.publicKey },
          {
            weight: 2,
            data: OracleJob.encodeDelimited(
              OracleJob.fromObject({
                tasks: [
                  {
                    valueTask: {
                      value: 1,
                    },
                  },
                ],
              })
            ).finish(),
          },
        ],
      });
   * ```
   */
  public async createFeed(
    params: Omit<
      Omit<Omit<AggregatorInitParams, 'queueAccount'>, 'queueAuthority'>,
      'authority'
    > & {
      authority?: Keypair;
      crankPubkey?: PublicKey;
      historyLimit?: number;
    } & {
      // lease params
      fundAmount?: number;
      funderAuthority?: Keypair;
      funderTokenAccount?: PublicKey;
    } & Partial<PermissionSetParams> & {
        // job params
        jobs?: Array<{ pubkey: PublicKey; weight?: number } | JobInitParams>;
      } & {
        queueAuthorityPubkey?: PublicKey;
      }
  ): Promise<[AggregatorAccount, Array<TransactionSignature>]> {
    const signers: Keypair[] = [];

    const queue = await this.loadData();

    if (
      params.queueAuthority &&
      params.queueAuthority.publicKey.equals(queue.authority)
    ) {
      signers.push(params.queueAuthority);
    }

    const [aggregatorAccount, txns] = await this.createFeedInstructions(
      this.program.walletPubkey,
      params
    );

    const signatures = await this.program.signAndSendAll(txns, {
      skipPreflight: true,
    });

    return [aggregatorAccount, signatures];
  }

  /**
   * Creates a transaction object with crankInit instructions for the given QueueAccount.
   *
   * @param payer - the publicKey of the account that will pay for the new accounts. Will also be used as the account authority if no other authority is provided.
   *
   * @param params - the crank configuration parameters.
   *
   * @return Transaction signature and the newly created CrankAccount.
   *
   * Basic usage example:
   *
   * ```ts
   * import { QueueAccount } from '@switchboard-xyz/solana.js';
   * const queueAccount = new QueueAccount(program, queuePubkey);
   * const [crankAccount, crankInitTxn] = await queueAccount.createCrankInstructions(payer, {
   *  name: "My Crank",
   *  metadata: "Crank #1",
   *  maxRows: 1000,
   * });
   * const crankInitSignature = await program.signAndSend(crankInitTxn);
   * const crank = await crankAccount.loadData();
   * ```
   */
  public async createCrankInstructions(
    payer: PublicKey,
    params: Omit<CrankInitParams, 'queueAccount'>
  ): Promise<[CrankAccount, TransactionObject]> {
    return await CrankAccount.createInstructions(this.program, payer, {
      ...params,
      queueAccount: this,
    });
  }

  /**
   * Creates a new {@linkcode CrankAccount}.
   *
   * @param params - the crank configuration parameters.
   *
   * @return Transaction signature and the newly created CrankAccount.
   *
   * Basic usage example:
   *
   * ```ts
   * import { QueueAccount } from '@switchboard-xyz/solana.js';
   * const queueAccount = new QueueAccount(program, queuePubkey);
   * const [crankAccount, crankInitSignature] = await queueAccount.createCrank({
   *  name: "My Crank",
   *  metadata: "Crank #1",
   *  maxRows: 1000,
   * });
   * const crank = await crankAccount.loadData();
   * ```
   */
  public async createCrank(
    params: Omit<CrankInitParams, 'queueAccount'>
  ): Promise<[CrankAccount, TransactionSignature]> {
    const [crankAccount, txn] = await this.createCrankInstructions(
      this.program.walletPubkey,
      params
    );
    const txnSignature = await this.program.signAndSend(txn);
    return [crankAccount, txnSignature];
  }

  /**
   * Creates a transaction object with vrfInit instructions for the given QueueAccount.
   *
   * @param payer - the publicKey of the account that will pay for the new accounts. Will also be used as the account authority if no other authority is provided.
   *
   * @param params - the vrf configuration parameters.
   *
   * @return Transaction signature and the newly created VrfAccount.
   *
   * Basic usage example:
   *
   * ```ts
   * import { QueueAccount } from '@switchboard-xyz/solana.js';
   * const queueAccount = new QueueAccount(program, queuePubkey);
   * const vrfKeypair = Keypair.generate();
   * const [vrfAccount, vrfInitTxn] = await queueAccount.createVrfInstructions(payer, {
   *  vrfKeypair: vrfKeypair,
   *  callback: {
   *    programId: "",
   *    accounts: [],
   *    ixData: Buffer.from("")
   *  },
   * });
   * const vrfInitSignature = await program.signAndSend(vrfInitTxn);
   * const vrf = await vrfAccount.loadData();
   * ```
   */
  public async createVrfInstructions(
    payer: PublicKey,
    params: Omit<VrfInitParams, 'queueAccount'> &
      Partial<PermissionSetParams> & {
        queueAuthorityPubkey?: PublicKey;
      }
  ): Promise<[VrfAccount, TransactionObject]> {
    const queueAuthorityPubkey = params.queueAuthority
      ? params.queueAuthority.publicKey
      : params.queueAuthorityPubkey ?? (await this.loadData()).authority;

    const [vrfAccount, vrfInit] = await VrfAccount.createInstructions(
      this.program,
      payer,
      {
        vrfKeypair: params.vrfKeypair,
        queueAccount: this,
        callback: params.callback,
        authority: params.authority,
      }
    );

    // eslint-disable-next-line prefer-const
    let [permissionAccount, permissionInit] =
      PermissionAccount.createInstruction(this.program, payer, {
        granter: this.publicKey,
        grantee: vrfAccount.publicKey,
        authority: queueAuthorityPubkey,
      });

    if (
      params.enable &&
      (params.queueAuthority || queueAuthorityPubkey.equals(payer))
    ) {
      const permissionSet = permissionAccount.setInstruction(payer, {
        permission: new PermitOracleQueueUsage(),
        enable: true,
        queueAuthority: params.queueAuthority,
      });
      permissionInit = permissionInit.combine(permissionSet);
    }

    return [vrfAccount, vrfInit.combine(permissionInit)];
  }

  /**
   * Creates a new {@linkcode VrfAccount} for a given QueueAccount.
   *
   * @param params - the vrf configuration parameters.
   *
   * @return Transaction signature and the newly created VrfAccount.
   *
   * Basic usage example:
   *
   * ```ts
   * import { QueueAccount } from '@switchboard-xyz/solana.js';
   * const queueAccount = new QueueAccount(program, queuePubkey);
   * const vrfKeypair = Keypair.generate();
   * const [vrfAccount, vrfInitSignature] = await queueAccount.createVrf({
   *  vrfKeypair: vrfKeypair,
   *  callback: {
   *    programId: "",
   *    accounts: [],
   *    ixData: Buffer.from("")
   *  },
   * });
   * const vrf = await vrfAccount.loadData();
   * ```
   */
  public async createVrf(
    params: Omit<VrfInitParams, 'queueAccount'> &
      Partial<PermissionSetParams> & {
        queueAuthorityPubkey?: PublicKey;
      }
  ): Promise<[VrfAccount, TransactionSignature]> {
    const [vrfAccount, txn] = await this.createVrfInstructions(
      this.program.walletPubkey,
      params
    );
    const txnSignature = await this.program.signAndSend(txn);
    return [vrfAccount, txnSignature];
  }

  /**
   * Creates a transaction object with bufferRelayerInit instructions for the given QueueAccount.
   *
   * @param payer - the publicKey of the account that will pay for the new accounts. Will also be used as the account authority if no other authority is provided.
   *
   * @param params - the buffer relayer configuration parameters.
   *
   * @return Transaction signature and the newly created BufferRelayerAccount.
   *
   * Basic usage example:
   *
   * ```ts
   * import { QueueAccount } from '@switchboard-xyz/solana.js';
   * const queueAccount = new QueueAccount(program, queuePubkey);
   * const [bufferRelayerAccount, bufferRelayerInitTxn] = await queueAccount.createBufferRelayerInstructions(payer, {
   *  name: "My Buffer",
   *  minUpdateDelaySeconds: 30,
   *  job: existingJobPubkey,
   * });
   * const bufferRelayerInitSignature = await program.signAndSend(bufferRelayerInitTxn);
   * const bufferRelayer = await bufferRelayerAccount.loadData();
   * ```
   */
  public async createBufferRelayerInstructions(
    payer: PublicKey,
    params: Omit<Omit<BufferRelayerInit, 'jobAccount'>, 'queueAccount'> &
      Partial<PermissionSetParams> & {
        // job params
        job: JobAccount | PublicKey | Omit<JobInitParams, 'weight'>;
      } & {
        queueAuthorityPubkey?: PublicKey;
      }
  ): Promise<[BufferRelayerAccount, TransactionObject]> {
    const queueAuthorityPubkey = params.queueAuthority
      ? params.queueAuthority.publicKey
      : params.queueAuthorityPubkey ?? (await this.loadData()).authority;

    const txns: TransactionObject[] = [];

    let job: JobAccount;
    if ('data' in params.job) {
      const [jobAccount, jobInit] = JobAccount.createInstructions(
        this.program,
        payer,
        {
          data: params.job.data,
          name: params.job.name ?? '',
          authority: params.job.authority ?? payer,
          expiration: params.job.expiration,
          variables: params.job.variables,
          keypair: params.job.keypair,
        }
      );
      txns.push(...jobInit);
      job = jobAccount;
    } else if (params.job instanceof PublicKey) {
      const jobAccount = new JobAccount(this.program, params.job);
      // should we verify its a valid job account?
      job = jobAccount;
    } else if (params.job instanceof JobAccount) {
      job = params.job;
    } else {
      throw new Error(
        `Failed to create BufferRelayer job account. 'data' or 'pubkey' was not defined in jobDefinition`
      );
    }

    const [bufferAccount, bufferInit] =
      await BufferRelayerAccount.createInstructions(this.program, payer, {
        name: params.name,
        minUpdateDelaySeconds: params.minUpdateDelaySeconds,
        queueAccount: this,
        authority: params.authority,
        jobAccount: job,
        keypair: params.keypair,
      });

    txns.push(bufferInit);

    // eslint-disable-next-line prefer-const
    let [permissionAccount, permissionInit] =
      PermissionAccount.createInstruction(this.program, payer, {
        granter: this.publicKey,
        grantee: bufferAccount.publicKey,
        authority: queueAuthorityPubkey,
      });

    if (
      params.enable &&
      (params.queueAuthority || queueAuthorityPubkey.equals(payer))
    ) {
      const permissionSet = permissionAccount.setInstruction(payer, {
        permission: new PermitOracleQueueUsage(),
        enable: true,
        queueAuthority: params.queueAuthority,
      });
      permissionInit = permissionInit.combine(permissionSet);
    }

    txns.push(permissionInit);

    const packed = TransactionObject.pack(txns);
    if (packed.length > 1) {
      throw new Error(
        `Failed to pack buffer relayer instructions into a single transaction`
      );
    }

    return [bufferAccount, packed[0]];
  }

  /**
   * Creates a new {@linkcode BufferRelayerAccount} for a given QueueAccount.
   *
   * @param params - the buffer relayer configuration parameters.
   *
   * @return Transaction signature and the newly created BufferRelayerAccount.
   *
   * Basic usage example:
   *
   * ```ts
   * import { QueueAccount } from '@switchboard-xyz/solana.js';
   * const queueAccount = new QueueAccount(program, queuePubkey);
   * const [bufferRelayerAccount, bufferRelayerInitSignature] = await queueAccount.createBufferRelayer({
   *  name: "My Buffer",
   *  minUpdateDelaySeconds: 30,
   *  job: existingJobPubkey,
   * });
   * const bufferRelayer = await bufferRelayerAccount.loadData();
   * ```
   */
  public async createBufferRelayer(
    params: Omit<Omit<BufferRelayerInit, 'jobAccount'>, 'queueAccount'> &
      Partial<PermissionSetParams> & {
        // job params
        job: JobAccount | PublicKey | Omit<JobInitParams, 'weight'>;
      } & {
        queueAuthorityPubkey?: PublicKey;
      }
  ): Promise<[BufferRelayerAccount, TransactionSignature]> {
    const [bufferRelayerAccount, txn] =
      await this.createBufferRelayerInstructions(
        this.program.walletPubkey,
        params
      );
    const txnSignature = await this.program.signAndSend(txn);
    return [bufferRelayerAccount, txnSignature];
  }

  /** Load the list of oracles that are currently stored in the buffer */
  public async loadOracles(): Promise<Array<PublicKey>> {
    let queue: QueueDataBuffer;
    if (this.dataBuffer) {
      queue = this.dataBuffer;
    } else {
      const queueData = await this.loadData();
      queue = new QueueDataBuffer(this.program, queueData.dataBuffer);
    }

    return queue.loadData();
  }

  /** Loads the oracle states for the oracles currently on the queue's dataBuffer */
  public async loadOracleAccounts(_oracles?: Array<PublicKey>): Promise<
    Array<{
      account: OracleAccount;
      data: types.OracleAccountData;
    }>
  > {
    const oraclePubkeys = _oracles ?? (await this.loadOracles());

    return await OracleAccount.fetchMultiple(this.program, oraclePubkeys);
  }

  public async loadActiveOracleAccounts(
    _queue?: types.OracleQueueAccountData
  ): Promise<
    Array<{
      account: OracleAccount;
      data: types.OracleAccountData;
    }>
  > {
    const queue = _queue ?? (await this.loadData());

    const oracles = await this.loadOracleAccounts();

    const unixTimestamp = (await SolanaClock.fetch(this.program.connection))
      .unixTimestamp;
    const timeout = unixTimestamp.sub(new BN(queue.oracleTimeout));
    const activeOracles = oracles.filter(
      o => o.data && o.data.lastHeartbeat.gte(timeout)
    );
    return activeOracles;
  }

  /** Returns a flag dictating whether enough oracles are actively heartbeating on an oracle queue and ready for on-chain update requests */
  public async isReady(
    _queue?: types.OracleQueueAccountData,
    oraclesNeeded = 1
  ): Promise<boolean> {
    const activeOracles = await this.loadActiveOracleAccounts(_queue);
    return activeOracles.length >= oraclesNeeded ? true : false;
  }

  public async setConfig(
    params: QueueSetConfigParams & { authority?: Keypair }
  ): Promise<TransactionSignature> {
    const setConfigTxn = this.setConfigInstruction(
      this.program.walletPubkey,
      params
    );
    const txnSignature = await this.program.signAndSend(setConfigTxn);
    return txnSignature;
  }

  public setConfigInstruction(
    payer: PublicKey,
    params: QueueSetConfigParams & { authority?: Keypair }
  ): TransactionObject {
    const multiplier =
      params.varianceToleranceMultiplier &&
      Number.isFinite(params.varianceToleranceMultiplier)
        ? SwitchboardDecimal.fromBig(
            new Big(params.varianceToleranceMultiplier)
          )
        : null;

    const reward = params.reward
      ? this.program.mint.toTokenAmountBN(params.reward)
      : null;
    const minStake = params.minStake
      ? this.program.mint.toTokenAmountBN(params.minStake)
      : null;

    return new TransactionObject(
      payer,
      [
        types.oracleQueueSetConfig(
          this.program,
          {
            params: {
              name: params.name
                ? [
                    ...new Uint8Array(
                      Buffer.from(params.name ?? '').slice(0, 32)
                    ),
                  ]
                : null,
              metadata: params.metadata
                ? [
                    ...new Uint8Array(
                      Buffer.from(params.metadata ?? '').slice(0, 64)
                    ),
                  ]
                : null,
              unpermissionedFeedsEnabled:
                params.unpermissionedFeedsEnabled ?? null,
              unpermissionedVrfEnabled: params.unpermissionedVrfEnabled ?? null,
              enableBufferRelayers: params.enableBufferRelayers ?? null,
              slashingEnabled: params.slashingEnabled ?? null,
              reward: reward,
              minStake: minStake,
              oracleTimeout: params.oracleTimeout ?? null,
              consecutiveFeedFailureLimit: params.consecutiveFeedFailureLimit
                ? new anchor.BN(params.consecutiveFeedFailureLimit)
                : null,
              consecutiveOracleFailureLimit:
                params.consecutiveOracleFailureLimit
                  ? new anchor.BN(params.consecutiveOracleFailureLimit)
                  : null,
              varianceToleranceMultiplier: multiplier,
            },
          },
          {
            authority: params.authority ? params.authority.publicKey : payer,
            queue: this.publicKey,
          }
        ),
      ],
      params.authority ? [params.authority] : []
    );
  }

  public async toAccountsJSON(
    _queue?: types.OracleQueueAccountData,
    _oracles?: Array<PublicKey>
  ): Promise<QueueAccountsJSON> {
    const queue = _queue ?? (await this.loadData());
    const oracles = _oracles ?? (await this.loadOracles());
    const oracleAccounts = await this.loadOracleAccounts(oracles);

    return {
      publicKey: this.publicKey,
      ...queue.toJSON(),
      dataBuffer: {
        publicKey: queue.dataBuffer,
        data: oracles,
      },
      oracles: oracleAccounts.map(o => {
        return {
          publicKey: o.account.publicKey,
          data: o.data.toJSON(),
        };
      }),
    };
  }

  public async fetchAccounts(
    _queue?: types.OracleQueueAccountData,
    _oracles?: Array<PublicKey>
  ): Promise<QueueAccounts> {
    const queue = _queue ?? (await this.loadData());
    const oracles = _oracles ?? (await this.loadOracles());

    const oracleAccounts = await this.loadOracleAccounts(oracles);

    return {
      queue: {
        publicKey: this.publicKey,
        data: queue,
      },
      dataBuffer: {
        publicKey: queue.dataBuffer,
        data: oracles,
      },
      oracles: oracleAccounts.map(o => {
        return {
          publicKey: o.account.publicKey,
          data: o.data,
        };
      }),
    };
  }
}

/**
 *  Parameters for initializing an {@linkcode QueueAccount}
 */
export interface QueueInitParams {
  /**
   *  A name to assign to this {@linkcode QueueAccount}
   */
  name?: string;
  /**
   *  Metadata for the queue for easier identification.
   */
  metadata?: string;
  /**
   *  Rewards to provide oracles and round openers on this queue.
   */
  reward: number;
  /**
   *  The minimum amount of stake oracles must present to remain on the queue.
   */
  minStake: number;
  /**
   *  After a feed lease is funded or re-funded, it must consecutively succeed
   *  N amount of times or its authorization to use the queue is auto-revoked.
   */
  feedProbationPeriod?: number;
  /**
   *  Time period (in seconds) we should remove an oracle after if no response.
   */
  oracleTimeout?: number;
  /**
   *  Whether slashing is enabled on this queue.
   */
  slashingEnabled?: boolean;
  /**
   *  The tolerated variance amount oracle results can have from the accepted round result
   *  before being slashed.
   *  slashBound = varianceToleranceMultiplier * stdDeviation
   *  Default: 2
   */
  varianceToleranceMultiplier?: number;
  /**
   *  Consecutive failure limit for a feed before feed permission is revoked.
   */
  consecutiveFeedFailureLimit?: number;
  /**
   *  Consecutive failure limit for an oracle before oracle permission is revoked.
   */
  consecutiveOracleFailureLimit?: number;
  /**
   *  Optionally set the size of the queue.
   */
  queueSize?: number;
  /**
   *  Enabling this setting means data feeds do not need explicit permission to join the queue.
   */
  unpermissionedFeeds?: boolean;
  /**
   *  Enabling this setting means data feeds do not need explicit permission
   *  to request VRF proofs and verifications from this queue.
   */
  unpermissionedVrf?: boolean;
  /**
   *  Enabling this setting will allow buffer relayer accounts to call openRound.
   */
  enableBufferRelayers?: boolean;
  /**
   *  The account to delegate authority to for creating permissions targeted at the queue.
   *
   *  Defaults to the payer.
   */
  authority?: PublicKey;

  keypair?: Keypair;
  dataBufferKeypair?: Keypair;
}

export interface QueueSetConfigParams {
  /** Alternative keypair that is the queue authority and is permitted to make account changes. Defaults to the payer if not provided. */
  authority?: anchor.web3.Keypair;
  /**
   *  A name to assign to this {@linkcode QueueAccount}
   */
  name?: string;
  /**
   *  Metadata for the queue for easier identification.
   */
  metadata?: string;
  /**
   *  Enabling this setting means data feeds do not need explicit permission to join the queue.
   */
  unpermissionedFeedsEnabled?: boolean;
  /**
   *  Enabling this setting means data feeds do not need explicit permission
   *  to request VRF proofs and verifications from this queue.
   */
  unpermissionedVrfEnabled?: boolean;
  /**
   *  Enabling this setting will allow buffer relayer accounts to call openRound.
   */
  enableBufferRelayers?: boolean;
  /**
   *  Whether slashing is enabled on this queue.
   */
  slashingEnabled?: boolean;
  /**
   *  The tolerated variance amount oracle results can have from the accepted round result
   *  before being slashed.
   *  slashBound = varianceToleranceMultiplier * stdDeviation
   */
  varianceToleranceMultiplier?: number;
  /**
   *  Time period (in seconds) we should remove an oracle after if no response.
   */
  oracleTimeout?: number;
  /**
   *  Rewards to provide oracles and round openers on this queue.
   */
  reward?: number;
  /**
   *  The minimum amount of stake oracles must present to remain on the queue.
   */
  minStake?: number;
  /**
   *  Consecutive failure limit for a feed before feed permission is revoked.
   */
  consecutiveFeedFailureLimit?: number;
  /**
   *  Consecutive failure limit for an oracle before oracle permission is revoked.
   */
  consecutiveOracleFailureLimit?: number;
}

export type QueueAccountsJSON = Omit<
  types.OracleQueueAccountDataJSON,
  'dataBuffer'
> & {
  publicKey: PublicKey;
  dataBuffer: { publicKey: PublicKey; data: Array<PublicKey> };
  oracles: Array<{
    publicKey: PublicKey;
    data: types.OracleAccountDataJSON;
  }>;
};

export type QueueAccounts = {
  queue: {
    publicKey: PublicKey;
    data: types.OracleQueueAccountData;
  };
  dataBuffer: {
    publicKey: PublicKey;
    data: Array<PublicKey>;
  };
  oracles: Array<{
    publicKey: PublicKey;
    data: types.OracleAccountData;
  }>;
};
