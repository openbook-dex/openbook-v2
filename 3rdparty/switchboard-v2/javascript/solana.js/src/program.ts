import * as anchor from '@project-serum/anchor';
import * as errors from './errors';
import * as sbv2 from './accounts';
import {
  AccountInfo,
  Cluster,
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  TransactionSignature,
} from '@solana/web3.js';
import { NativeMint } from './mint';
import { TransactionObject } from './transaction';
import { SwitchboardEvents } from './switchboardEvents';
import { fromCode as fromSwitchboardCode } from './generated/errors/custom';
import { fromCode as fromAnchorCode } from './generated/errors/anchor';
import { ACCOUNT_DISCRIMINATOR_SIZE } from '@project-serum/anchor';
import {
  AggregatorAccountData,
  BufferRelayerAccountData,
  CrankAccountData,
  JobAccountData,
  LeaseAccountData,
  OracleAccountData,
  OracleQueueAccountData,
  PermissionAccountData,
  SbState,
  SlidingResultAccountData,
  VrfAccountData,
} from './generated';
import {
  CrankAccount,
  CrankInitParams,
  DISCRIMINATOR_MAP,
  OracleAccount,
  OracleInitParams,
  OracleStakeParams,
  PermissionAccount,
  PermissionSetParams,
  ProgramStateAccount,
  QueueAccount,
  QueueInitParams,
} from './accounts';
import {
  SWITCHBOARD_LABS_DEVNET_PERMISSIONED_CRANK,
  SWITCHBOARD_LABS_DEVNET_PERMISSIONED_QUEUE,
  SWITCHBOARD_LABS_MAINNET_PERMISSIONED_CRANK,
  SWITCHBOARD_LABS_MAINNET_PERMISSIONED_QUEUE,
  SWITCHBOARD_LABS_DEVNET_PERMISSIONLESS_CRANK,
  SWITCHBOARD_LABS_DEVNET_PERMISSIONLESS_QUEUE,
  SWITCHBOARD_LABS_MAINNET_PERMISSIONLESS_CRANK,
  SWITCHBOARD_LABS_MAINNET_PERMISSIONLESS_QUEUE,
} from './const';

/**
 * Switchboard Devnet Program ID
 */
export const SBV2_DEVNET_PID = new PublicKey(
  '2TfB33aLaneQb5TNVwyDz3jSZXS6jdW2ARw1Dgf84XCG'
);
/**
 * Switchboard Mainnet Program ID
 */
export const SBV2_MAINNET_PID = new PublicKey(
  'SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f'
);
/**
 *  A generated keypair that is assigned as the _payerKeypair_ when in read-only mode.
 */
export const READ_ONLY_KEYPAIR = Keypair.generate();
/**
 * Returns the Switchboard Program ID for the specified Cluster.
 */
export const getSwitchboardProgramId = (
  cluster: Cluster | 'localnet'
): PublicKey => {
  switch (cluster) {
    case 'devnet':
      return SBV2_DEVNET_PID;
    case 'mainnet-beta':
      return SBV2_MAINNET_PID;
    case 'testnet':
    default:
      throw new Error(`Switchboard PID not found for cluster (${cluster})`);
  }
};
/**
 * Returns true if being run inside a web browser, false if in a Node process or electron app.
 *
 * Taken from @project-serum/anchor implementation.
 */
const isBrowser =
  process.env.ANCHOR_BROWSER ||
  (typeof window !== 'undefined' && !window.process?.hasOwnProperty('type')); // eslint-disable-line no-prototype-builtins

/**
 * Wrapper class for the Switchboard anchor Program.
 *
 * Basic usage example:
 *
 * ```ts
 * import { Connection } from "@solana/web3.js";
 * import { SwitchboardProgram, TransactionObject } from '@switchboard-xyz/solana.js';
 *
 * const program = await SwitchboardProgram.load(
 *    "mainnet-beta",
 *    new Connection("https://api.mainnet-beta.solana.com"),
 *    payerKeypair
 * );
 *
 * const txn = new TransactionObject(program.walletPubkey, [], []);
 * const txnSignature = await program.signAndSend(txn);
 * ```
 */
export class SwitchboardProgram {
  private static readonly _readOnlyKeypair = READ_ONLY_KEYPAIR;

  private readonly _program: anchor.Program;

  /** The solana cluster to load the Switchboard program for. */
  readonly cluster: Cluster | 'localnet';

  readonly programState: {
    publicKey: PublicKey;
    bump: number;
  };

  readonly mint: NativeMint;

  /**
   * Constructor.
   */
  constructor(
    program: anchor.Program,
    cluster: Cluster | 'localnet',
    mint: NativeMint
  ) {
    this._program = program;
    this.cluster = cluster;

    const stateAccount = sbv2.ProgramStateAccount.fromSeed(this);
    this.programState = {
      publicKey: stateAccount[0].publicKey,
      bump: stateAccount[1],
    };
    this.mint = mint;
  }

  static async loadAnchorProgram(
    cluster: Cluster | 'localnet',
    connection: Connection,
    payerKeypair: Keypair = READ_ONLY_KEYPAIR,
    programId: PublicKey = getSwitchboardProgramId(cluster)
  ): Promise<anchor.Program> {
    const provider = new anchor.AnchorProvider(
      connection,
      // If no keypair is provided, default to dummy keypair
      new AnchorWallet(payerKeypair ?? SwitchboardProgram._readOnlyKeypair),
      { commitment: 'confirmed' }
    );
    const anchorIdl = await anchor.Program.fetchIdl(programId, provider);
    if (!anchorIdl) {
      throw new Error(`Failed to find IDL for ${programId.toBase58()}`);
    }
    const program = new anchor.Program(anchorIdl, programId, provider);

    return program;
  }

  /**
   * Create and initialize a {@linkcode SwitchboardProgram} connection object.
   *
   * @param cluster - the solana cluster to load the Switchboard program for.
   *
   * @param connection - the Solana connection object used to connect to an RPC node.
   *
   * @param payerKeypair - optional, payer keypair used to pay for on-chain transactions.
   *
   * @param programId - optional, override the cluster's default programId.
   *
   * @return the {@linkcode SwitchboardProgram} used to create and interact with Switchboard accounts.
   *
   * Basic usage example:
   *
   * ```ts
   * import { Connection } from "@solana/web3.js";
   * import { SwitchboardProgram, TransactionObject } from '@switchboard-xyz/solana.js';
   *
   * const program = await SwitchboardProgram.load(
   *    "mainnet-beta",
   *    new Connection("https://api.mainnet-beta.solana.com"),
   *    payerKeypair
   * );
   *
   * const txn = new TransactionObject(program.walletPubkey, [], []);
   * const txnSignature = await program.signAndSend(txn);
   * ```
   */
  static load = async (
    cluster: Cluster | 'localnet',
    connection: Connection,
    payerKeypair: Keypair = READ_ONLY_KEYPAIR,
    programId: PublicKey = getSwitchboardProgramId(cluster)
  ): Promise<SwitchboardProgram> => {
    const program = await SwitchboardProgram.loadAnchorProgram(
      cluster,
      connection,
      payerKeypair,
      programId
    );
    const mint = await NativeMint.load(
      program.provider as anchor.AnchorProvider
    );
    return new SwitchboardProgram(program, cluster, mint);
  };

  /**
   * Create and initialize a {@linkcode SwitchboardProgram} connection object.
   *
   * @param provider - the anchor provider containing the rpc and wallet connection.
   *
   * @return the {@linkcode SwitchboardProgram} used to create and interact with Switchboard accounts.
   *
   * Basic usage example:
   *
   * ```ts
   * import * as anchor from "@project-serum/anchor";
   * import { Connection } from "@solana/web3.js";
   * import { AnchorWallet, SwitchboardProgram, TransactionObject } from '@switchboard-xyz/solana.js';
   *
   * const connection = new Connection("https://api.mainnet-beta.solana.com");
   * const provider = new anchor.AnchorProvider(
      connection,
      new AnchorWallet(payerKeypair ?? SwitchboardProgram._readOnlyKeypair),
      { commitment: 'confirmed' }
    );
   * const program = await SwitchboardProgram.fromProvider(provider);
   *
   * const txn = new TransactionObject(program.walletPubkey, [], []);
   * const txnSignature = await program.signAndSend(txn);
   * ```
   */
  static fromProvider = async (
    provider: anchor.AnchorProvider
  ): Promise<SwitchboardProgram> => {
    const payer = (provider.wallet as AnchorWallet).payer;

    // try mainnet program ID
    const mainnetAccountInfo = await provider.connection.getAccountInfo(
      SBV2_MAINNET_PID
    );
    if (mainnetAccountInfo && mainnetAccountInfo.executable) {
      return await SwitchboardProgram.load(
        'mainnet-beta',
        provider.connection,
        payer,
        SBV2_MAINNET_PID
      );
    }

    // try devnet program ID
    const devnetAccountInfo = await provider.connection.getAccountInfo(
      SBV2_DEVNET_PID
    );
    if (devnetAccountInfo && devnetAccountInfo.executable) {
      return await SwitchboardProgram.load(
        'devnet',
        provider.connection,
        payer,
        SBV2_DEVNET_PID
      );
    }

    throw new Error(
      `Failed to find the Switchboard program using the mainnet or devnet program ID`
    );
  };

  /**
   * The Switchboard Program ID for the currently connected cluster.
   */
  public get programId(): PublicKey {
    return this._program.programId;
  }
  /**
   * The Switchboard Program ID for the currently connected cluster.
   */
  public get idl(): anchor.Idl {
    return this._program.idl;
  }
  /**
   * The Switchboard Program ID for the currently connected cluster.
   */
  public get coder(): anchor.BorshAccountsCoder {
    return new anchor.BorshAccountsCoder(this._program.idl);
  }
  /**
   * The anchor Provider used by this program to connect with Solana cluster.
   */
  public get provider(): anchor.AnchorProvider {
    return this._program.provider as anchor.AnchorProvider;
  }
  /**
   * The Connection used by this program to connect with Solana cluster.
   */
  public get connection(): Connection {
    return this._program.provider.connection;
  }
  /**
   * The Connection used by this program to connect with Solana cluster.
   */
  public get wallet(): AnchorWallet {
    return this.provider.wallet as AnchorWallet;
  }

  public get walletPubkey(): PublicKey {
    return this.wallet.payer.publicKey;
  }
  /**
   * Some actions exposed by this SDK require that a payer Keypair has been
   * provided to {@linkcode SwitchboardProgram} in order to send transactions.
   */
  public get isReadOnly(): boolean {
    return (
      this.provider.publicKey.toBase58() ===
      SwitchboardProgram._readOnlyKeypair.publicKey.toBase58()
    );
  }

  /** Verify a payer keypair was supplied. */
  public verifyPayer(): void {
    if (this.isReadOnly) {
      throw new errors.SwitchboardProgramReadOnlyError();
    }
  }

  /**
   * Verify a fresh keypair was provided.
   *
   * **NOTE:** Creating new accounts without this check will prevent the ability to remove any existing funds. */
  public async verifyNewKeypair(keypair: Keypair): Promise<void> {
    const accountInfo = await this.connection.getAccountInfo(keypair.publicKey);
    if (accountInfo) {
      throw new errors.ExistingKeypair();
    }
  }

  public get account(): anchor.AccountNamespace {
    return this._program.account;
  }

  /**
   * Load the Switchboard Labs permissionless Queue for either devnet or mainnet. The permissionless queue has the following permissions:
   *  - unpermissionedFeedsEnabled: True
   *  - unpermissionedVrfEnabled: True
   *  - enableBufferRelayers: False
   *
   * **Note:** {@linkcode AggregatorAccount}s and {@linkcode VrfAccount}s do not require permissions to join this queue. {@linkcode BufferRelayerAccount}s are disabled.
   */
  async loadPermissionless(): Promise<{
    queueAccount: QueueAccount;
    queue: OracleQueueAccountData;
    crankAccount: CrankAccount;
    crank: CrankAccountData;
  }> {
    const queueKey =
      this.cluster === 'mainnet-beta'
        ? SWITCHBOARD_LABS_MAINNET_PERMISSIONLESS_QUEUE
        : this.cluster === 'devnet'
        ? SWITCHBOARD_LABS_DEVNET_PERMISSIONLESS_QUEUE
        : null;
    if (!queueKey) {
      throw new Error(
        `Failed to load the permissionless queue for cluster ${this.cluster}`
      );
    }
    const [queueAccount, queue] = await QueueAccount.load(this, queueKey);

    const crankKey =
      this.cluster === 'mainnet-beta'
        ? SWITCHBOARD_LABS_MAINNET_PERMISSIONLESS_CRANK
        : this.cluster === 'devnet'
        ? SWITCHBOARD_LABS_DEVNET_PERMISSIONLESS_CRANK
        : null;
    if (!crankKey) {
      throw new Error(
        `Failed to load the permissionless queue for cluster ${this.cluster}`
      );
    }
    const [crankAccount, crank] = await CrankAccount.load(this, crankKey);

    return { queueAccount, queue, crankAccount, crank };
  }

  /**
   * Load the Switchboard Labs permissionled Queue for either devnet or mainnet. The permissioned queue has the following permissions:
   *  - unpermissionedFeedsEnabled: False
   *  - unpermissionedVrfEnabled: False
   *  - enableBufferRelayers: False
   *
   * **Note:** The queue authority must grant {@linkcode AggregatorAccount}s PERMIT_ORACLE_QUEUE_USAGE and {@linkcode VrfAccount}s PERMIT_VRF_REQUESTS permissions before joining the queue and requesting oracle updates. {@linkcode BufferRelayerAccount}s are disabled.
   */
  async loadPermissioned(): Promise<{
    queueAccount: QueueAccount;
    queue: OracleQueueAccountData;
    crankAccount: CrankAccount;
    crank: CrankAccountData;
  }> {
    const queueKey =
      this.cluster === 'mainnet-beta'
        ? SWITCHBOARD_LABS_MAINNET_PERMISSIONED_QUEUE
        : this.cluster === 'devnet'
        ? SWITCHBOARD_LABS_DEVNET_PERMISSIONED_QUEUE
        : null;
    if (!queueKey) {
      throw new Error(
        `Failed to load the permissioned queue for cluster ${this.cluster}`
      );
    }
    const [queueAccount, queue] = await QueueAccount.load(
      this,
      this.cluster === 'mainnet-beta'
        ? SWITCHBOARD_LABS_MAINNET_PERMISSIONED_QUEUE
        : SWITCHBOARD_LABS_DEVNET_PERMISSIONED_QUEUE
    );

    const crankKey =
      this.cluster === 'mainnet-beta'
        ? SWITCHBOARD_LABS_MAINNET_PERMISSIONED_CRANK
        : this.cluster === 'devnet'
        ? SWITCHBOARD_LABS_DEVNET_PERMISSIONED_CRANK
        : null;
    if (!crankKey) {
      throw new Error(
        `Failed to load the permissionless queue for cluster ${this.cluster}`
      );
    }
    const [crankAccount, crank] = await CrankAccount.load(this, crankKey);

    return { queueAccount, queue, crankAccount, crank };
  }

  public addEventListener<EventName extends keyof SwitchboardEvents>(
    eventName: EventName,
    callback: (
      data: SwitchboardEvents[EventName],
      slot: number,
      signature: string
    ) => void
  ): number {
    return this._program.addEventListener(eventName, callback);
  }

  public async removeEventListener(listenerId: number) {
    return await this._program.removeEventListener(listenerId);
  }

  public async signAndSendAll(
    txns: Array<TransactionObject>,
    opts: anchor.web3.ConfirmOptions = {
      skipPreflight: false,
      maxRetries: 10,
    },
    blockhash?: { blockhash: string; lastValidBlockHeight: number }
  ): Promise<Array<TransactionSignature>> {
    if (isBrowser) throw new errors.SwitchboardProgramIsBrowserError();
    if (this.isReadOnly) throw new errors.SwitchboardProgramReadOnlyError();

    const packed = TransactionObject.pack(txns);

    const txnSignatures: Array<TransactionSignature> = [];
    for await (const txn of packed) {
      txnSignatures.push(await this.signAndSend(txn, opts, blockhash));
    }

    return txnSignatures;
  }

  public async signAndSend(
    txn: TransactionObject,
    opts: anchor.web3.ConfirmOptions = {
      skipPreflight: false,
      maxRetries: 10,
    },
    blockhash?: { blockhash: string; lastValidBlockHeight: number }
  ): Promise<TransactionSignature> {
    if (isBrowser) throw new errors.SwitchboardProgramIsBrowserError();
    if (this.isReadOnly) throw new errors.SwitchboardProgramReadOnlyError();

    // filter extra signers
    const signers = [this.wallet.payer, ...txn.signers];
    const reqSigners = txn.ixns.reduce((signers, ixn) => {
      ixn.keys.map(a => {
        if (a.isSigner) {
          signers.add(a.pubkey.toBase58());
        }
      });
      return signers;
    }, new Set<string>());
    const filteredSigners = signers.filter(
      s =>
        s.publicKey.equals(txn.payer) || reqSigners.has(s.publicKey.toBase58())
    );

    const transaction = txn.toTxn(
      blockhash ?? (await this.connection.getLatestBlockhash())
    );

    try {
      const txnSignature = await this.provider.sendAndConfirm(
        transaction,
        filteredSigners,
        {
          skipPreflight: false,
          maxRetries: 10,
          ...opts,
        }
      );

      return txnSignature;
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
    } catch (error: any) {
      if ('code' in error && typeof error.code === 'number') {
        // Check for other switchboard error.
        const switchboardError = fromSwitchboardCode(error.code);
        if (switchboardError) throw switchboardError;
        // Check for other anchor error.
        const anchorError = fromAnchorCode(error.code);
        if (anchorError) throw anchorError;
      }

      throw error;
    }
  }

  async getProgramAccounts(): Promise<{
    aggregators: Map<string, AggregatorAccountData>;
    buffers: Map<string, Buffer>;
    bufferRelayers: Map<string, BufferRelayerAccountData>;
    cranks: Map<string, CrankAccountData>;
    jobs: Map<string, JobAccountData>;
    leases: Map<string, LeaseAccountData>;
    oracles: Map<string, OracleAccountData>;
    permissions: Map<string, PermissionAccountData>;
    programState: Map<string, SbState>;
    queues: Map<string, OracleQueueAccountData>;
    slidingResult: Map<string, SlidingResultAccountData>;
    vrfs: Map<string, VrfAccountData>;
  }> {
    const accountInfos: Array<AccountInfoResponse> =
      await this.connection.getProgramAccounts(this.programId);

    // buffer - [42, 55, 46, 46, 45, 52, 78, 78]
    // bufferRelayer - [50, 35, 51, 115, 169, 219, 158, 52]
    // lease - [55, 254, 208, 251, 164, 44, 150, 50]
    // permissions - [77, 37, 177, 164, 38, 39, 34, 109]
    // slidingResult - [91, 4, 83, 187, 102, 216, 153, 254]
    // vrf - [101, 35, 62, 239, 103, 151, 6, 18]
    // crank - [111, 81, 146, 73, 172, 180, 134, 209]
    // job - [124, 69, 101, 195, 229, 218, 144, 63]
    // oracles - [128, 30, 16, 241, 170, 73, 55, 54]
    // sbState - [159, 42, 192, 191, 139, 62, 168, 28]
    // queue - [164, 207, 200, 51, 199, 113, 35, 109]
    // aggregator - [217, 230, 65, 101, 201, 162, 27, 125]

    const discriminatorMap: Map<
      string,
      Array<AccountInfoResponse>
    > = accountInfos.reduce((map, account) => {
      const discriminator = account.account.data
        .slice(0, ACCOUNT_DISCRIMINATOR_SIZE)
        .toString('utf-8');

      const accounts = map.get(discriminator) ?? [];
      accounts.push(account);
      map.set(discriminator, accounts);

      return map;
    }, new Map<string, Array<AccountInfoResponse>>());

    function decodeAccounts<T extends sbv2.SwitchboardAccount>(
      accounts: Array<AccountInfoResponse>,
      decode: (data: Buffer) => T
    ): Map<string, T> {
      return accounts.reduce((map, account) => {
        try {
          const decoded = decode(account.account.data);
          map.set(account.pubkey.toBase58(), decoded);
          // eslint-disable-next-line no-empty
        } catch {}

        return map;
      }, new Map<string, T>());
    }

    const aggregators: Map<string, AggregatorAccountData> = decodeAccounts(
      discriminatorMap.get(
        AggregatorAccountData.discriminator.toString('utf-8')
      ) ?? [],
      AggregatorAccountData.decode
    );

    // TODO: Use aggregator.historyBuffer, crank.dataBuffer, queue.dataBuffer to filter these down and decode
    const buffers: Map<string, Buffer> = (
      discriminatorMap.get(sbv2.BUFFER_DISCRIMINATOR.toString('utf-8')) ?? []
    ).reduce((map, buffer) => {
      map.set(buffer.pubkey.toBase58(), buffer.account.data);
      return map;
    }, new Map<string, Buffer>());

    const bufferRelayers: Map<string, BufferRelayerAccountData> =
      decodeAccounts(
        discriminatorMap.get(
          BufferRelayerAccountData.discriminator.toString('utf-8')
        ) ?? [],
        BufferRelayerAccountData.decode
      );

    const cranks: Map<string, CrankAccountData> = decodeAccounts(
      discriminatorMap.get(CrankAccountData.discriminator.toString('utf-8')) ??
        [],
      CrankAccountData.decode
    );

    const jobs: Map<string, JobAccountData> = decodeAccounts(
      discriminatorMap.get(JobAccountData.discriminator.toString('utf-8')) ??
        [],
      JobAccountData.decode
    );

    const leases: Map<string, LeaseAccountData> = decodeAccounts(
      discriminatorMap.get(LeaseAccountData.discriminator.toString('utf-8')) ??
        [],
      LeaseAccountData.decode
    );

    const oracles: Map<string, OracleAccountData> = decodeAccounts(
      discriminatorMap.get(OracleAccountData.discriminator.toString('utf-8')) ??
        [],
      OracleAccountData.decode
    );

    const permissions: Map<string, PermissionAccountData> = decodeAccounts(
      discriminatorMap.get(
        PermissionAccountData.discriminator.toString('utf-8')
      ) ?? [],
      PermissionAccountData.decode
    );

    const programState: Map<string, SbState> = decodeAccounts(
      discriminatorMap.get(SbState.discriminator.toString('utf-8')) ?? [],
      SbState.decode
    );

    const queues: Map<string, OracleQueueAccountData> = decodeAccounts(
      discriminatorMap.get(
        OracleQueueAccountData.discriminator.toString('utf-8')
      ) ?? [],
      OracleQueueAccountData.decode
    );

    const slidingResult: Map<string, SlidingResultAccountData> = decodeAccounts(
      discriminatorMap.get(
        SlidingResultAccountData.discriminator.toString('utf-8')
      ) ?? [],
      SlidingResultAccountData.decode
    );

    const vrfs: Map<string, VrfAccountData> = decodeAccounts(
      discriminatorMap.get(VrfAccountData.discriminator.toString('utf-8')) ??
        [],
      VrfAccountData.decode
    );

    return {
      aggregators,
      buffers,
      bufferRelayers,
      cranks,
      jobs,
      leases,
      oracles,
      permissions,
      programState,
      slidingResult,
      queues,
      vrfs,
    };
  }

  static getAccountType(
    accountInfo: AccountInfo<Buffer>
  ): sbv2.SwitchboardAccountType | null {
    const discriminator = accountInfo.data
      .slice(0, ACCOUNT_DISCRIMINATOR_SIZE)
      .toString('utf-8');
    const accountType = DISCRIMINATOR_MAP.get(discriminator);
    if (accountType) {
      return accountType;
    }

    return null;
  }

  async createNetworkInstructions(
    payer: PublicKey,
    params: QueueInitParams & {
      cranks?: Array<Omit<CrankInitParams, 'queueAccount'>>;
      oracles?: Array<
        OracleInitParams &
          OracleStakeParams &
          Partial<PermissionSetParams> & {
            queueAuthorityPubkey?: PublicKey;
          }
      >;
    }
  ): Promise<[Array<TransactionObject>, NetworkInitResponse]> {
    const txns: TransactionObject[] = [];

    // get or create the program state
    const [programState, stateBump, programInit] =
      await ProgramStateAccount.getOrCreateInstructions(
        this,
        this.walletPubkey
      );
    if (programInit) {
      txns.push(programInit);
    }

    // create a new queue
    const [queueAccount, queueInit] = await QueueAccount.createInstructions(
      this,
      this.walletPubkey,
      params
    );
    txns.push(queueInit);

    const cranks: Array<[CrankAccount, TransactionObject]> = await Promise.all(
      (params.cranks ?? []).map(async crankInitParams => {
        return await queueAccount.createCrankInstructions(
          payer,
          crankInitParams
        );
      })
    );
    txns.push(...cranks.map(crank => crank[1]));

    const oracles: Array<
      [TransactionObject[], OracleAccount, PermissionAccount, number]
    > = await Promise.all(
      (params.oracles ?? []).map(async oracleInitParams => {
        const [oracleAccount, oracleInit] =
          await queueAccount.createOracleInstructions(payer, {
            ...oracleInitParams,
            queueAuthorityPubkey:
              oracleInitParams.queueAuthorityPubkey ?? payer,
            enable: true,
          });

        const [oraclePermissionAccount, oraclePermissionBump] =
          PermissionAccount.fromSeed(
            this,
            this.walletPubkey,
            queueAccount.publicKey,
            oracleAccount.publicKey
          );

        return [
          oracleInit,
          oracleAccount,
          oraclePermissionAccount,
          oraclePermissionBump,
        ];
      })
    );
    txns.push(...oracles.map(oracle => oracle[0]).flat());

    const accounts: NetworkInitResponse = {
      programState: {
        account: programState,
        bump: stateBump,
      },
      queueAccount,
      cranks: cranks.map(c => c[0]),
      oracles: oracles.map(o => {
        return {
          account: o[1],
          permissions: {
            account: o[2],
            bump: o[3],
          },
        };
      }),
    };

    return [TransactionObject.pack(txns), accounts];
  }

  async createNetwork(
    params: QueueInitParams & {
      cranks?: Array<Omit<CrankInitParams, 'queueAccount'>>;
      oracles?: Array<
        OracleInitParams &
          OracleStakeParams &
          Partial<PermissionSetParams> & {
            queueAuthorityPubkey?: PublicKey;
          }
      >;
    }
  ): Promise<[NetworkInitResponse, Array<TransactionSignature>]> {
    const [networkInit, accounts] = await this.createNetworkInstructions(
      this.walletPubkey,
      params
    );
    const txnSignatures = await this.signAndSendAll(networkInit);
    return [accounts, txnSignatures];
  }
}

export class AnchorWallet implements anchor.Wallet {
  constructor(readonly payer: Keypair) {
    this.payer = payer;
  }
  get publicKey(): PublicKey {
    return this.payer.publicKey;
  }
  private sign = (txn: Transaction): Transaction => {
    txn.partialSign(this.payer);
    return txn;
  };
  async signTransaction(txn: Transaction) {
    return this.sign(txn);
  }
  async signAllTransactions(txns: Transaction[]) {
    return txns.map(this.sign);
  }
}

interface AccountInfoResponse {
  pubkey: anchor.web3.PublicKey;
  account: anchor.web3.AccountInfo<Buffer>;
}

export interface NetworkInitResponse {
  programState: {
    account: ProgramStateAccount;
    bump: number;
  };
  queueAccount: QueueAccount;
  cranks: Array<CrankAccount>;
  oracles: Array<{
    account: OracleAccount;
    permissions: {
      account: PermissionAccount;
      bump: number;
    };
  }>;
}
