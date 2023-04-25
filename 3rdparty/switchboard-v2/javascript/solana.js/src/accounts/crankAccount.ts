import * as anchor from '@project-serum/anchor';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import {
  AccountMeta,
  Commitment,
  Keypair,
  PublicKey,
  SystemProgram,
  TransactionSignature,
} from '@solana/web3.js';
import * as errors from '../errors';
import * as types from '../generated';
import { SwitchboardProgram } from '../program';
import { TransactionObject } from '../transaction';
import { Account, OnAccountChangeCallback } from './account';
import { AggregatorAccount } from './aggregatorAccount';
import { CrankDataBuffer } from './crankDataBuffer';
import { QueueAccount } from './queueAccount';

/**
 * Account holding a priority queue of aggregators and their next available update time. This is a scheduling mechanism to ensure {@linkcode AggregatorAccount}'s are updated as close as possible to their specified update interval.
 *
 * Data: {@linkcode types.CrankAccountData}
 *
 * Buffer: {@linkcode CrankDataBuffer}
 */
export class CrankAccount extends Account<types.CrankAccountData> {
  static accountName = 'CrankAccountData';

  /** The public key of the crank's data buffer storing a priority queue of {@linkcode AggregatorAccount}'s and their next available update timestamp */
  dataBuffer?: CrankDataBuffer;

  /**
   * Get the size of an {@linkcode CrankAccount} on-chain.
   */
  public size = this.program.account.crankAccountData.size;

  public static default(): types.CrankAccountData {
    const buffer = Buffer.alloc(432, 0);
    types.CrankAccountData.discriminator.copy(buffer, 0);
    return types.CrankAccountData.decode(buffer);
  }

  /**
   * Invoke a callback each time a CrankAccount's data has changed on-chain.
   * @param callback - the callback invoked when the cranks state changes
   * @param commitment - optional, the desired transaction finality. defaults to 'confirmed'
   * @returns the websocket subscription id
   */
  onChange(
    callback: OnAccountChangeCallback<types.CrankAccountData>,
    commitment: Commitment = 'confirmed'
  ): number {
    return this.program.connection.onAccountChange(
      this.publicKey,
      accountInfo => callback(types.CrankAccountData.decode(accountInfo.data)),
      commitment
    );
  }

  /** Load an existing CrankAccount with its current on-chain state */
  public static async load(
    program: SwitchboardProgram,
    publicKey: PublicKey | string
  ): Promise<[CrankAccount, types.CrankAccountData]> {
    const account = new CrankAccount(
      program,
      typeof publicKey === 'string' ? new PublicKey(publicKey) : publicKey
    );
    const state = await account.loadData();
    return [account, state];
  }

  /**
   * Retrieve and decode the {@linkcode types.CrankAccountData} stored in this account.
   */
  public async loadData(): Promise<types.CrankAccountData> {
    const data = await types.CrankAccountData.fetch(
      this.program,
      this.publicKey
    );
    if (data === null)
      throw new errors.AccountNotFoundError('Crank', this.publicKey);
    if (!this.dataBuffer) {
      this.dataBuffer = CrankDataBuffer.fromCrank(this.program, data);
    }

    return data;
  }

  public static async createInstructions(
    program: SwitchboardProgram,
    payer: PublicKey,
    params: CrankInitParams
  ): Promise<[CrankAccount, TransactionObject]> {
    const keypair = params.keypair ?? Keypair.generate();
    program.verifyNewKeypair(keypair);

    const buffer = params.dataBufferKeypair ?? anchor.web3.Keypair.generate();
    program.verifyNewKeypair(buffer);

    const maxRows = params.maxRows ?? 500;
    const crankSize = CrankDataBuffer.getAccountSize(maxRows);

    const crankInit = new TransactionObject(
      payer,
      [
        SystemProgram.createAccount({
          fromPubkey: payer,
          newAccountPubkey: buffer.publicKey,
          space: crankSize,
          lamports: await program.connection.getMinimumBalanceForRentExemption(
            crankSize
          ),
          programId: program.programId,
        }),
        types.crankInit(
          program,
          {
            params: {
              name: Buffer.from(params.name ?? '').slice(0, 32),
              metadata: Buffer.from(params.metadata ?? '').slice(0, 64),
              crankSize: maxRows,
            },
          },
          {
            crank: keypair.publicKey,
            queue: params.queueAccount.publicKey,
            buffer: buffer.publicKey,
            systemProgram: SystemProgram.programId,
            payer: payer,
          }
        ),
      ],
      [keypair, buffer]
    );

    const crankAccount = new CrankAccount(program, keypair.publicKey);
    crankAccount.dataBuffer = new CrankDataBuffer(program, buffer.publicKey);

    return [crankAccount, crankInit];
  }

  public static async create(
    program: SwitchboardProgram,
    params: CrankInitParams
  ): Promise<[CrankAccount, TransactionSignature]> {
    const [crankAccount, crankInit] = await CrankAccount.createInstructions(
      program,
      program.walletPubkey,
      params
    );
    const txnSignature = await program.signAndSend(crankInit);
    return [crankAccount, txnSignature];
  }

  public async popInstruction(
    payer: PublicKey,
    params: CrankPopParams
  ): Promise<TransactionObject> {
    const next = params.readyPubkeys ?? (await this.peakNextReady(5));
    if (next.length === 0) throw new Error('Crank is not ready to be turned.');

    const remainingAccounts: PublicKey[] = [];
    const leaseBumpsMap: Map<string, number> = new Map();
    const permissionBumpsMap: Map<string, number> = new Map();
    const crankData = await this.loadData();
    const queueAccount = new QueueAccount(this.program, crankData.queuePubkey);
    const queueData = await queueAccount.loadData();

    const toBumps = (bumpMap: Map<string, number>) =>
      new Uint8Array(Array.from(bumpMap.values()));

    for (const row of next) {
      const aggregatorAccount = new AggregatorAccount(this.program, row);

      const {
        leaseAccount,
        leaseBump,
        permissionAccount,
        permissionBump,
        leaseEscrow,
      } = aggregatorAccount.getAccounts({
        queueAccount: queueAccount,
        queueAuthority: queueData.authority,
      });

      remainingAccounts.push(aggregatorAccount.publicKey);
      remainingAccounts.push(leaseAccount.publicKey);
      remainingAccounts.push(leaseEscrow);
      remainingAccounts.push(permissionAccount.publicKey);

      leaseBumpsMap.set(row.toBase58(), leaseBump);
      permissionBumpsMap.set(row.toBase58(), permissionBump);
    }
    remainingAccounts.sort((a: PublicKey, b: PublicKey) =>
      a.toBuffer().compare(b.toBuffer())
    );

    const payoutWallet =
      params?.payoutWallet ?? this.program.mint.getAssociatedAddress(payer);

    const crankPopIxn = types.crankPop(
      this.program,
      {
        params: {
          stateBump: this.program.programState.bump,
          leaseBumps: toBumps(leaseBumpsMap),
          permissionBumps: toBumps(permissionBumpsMap),
          nonce: params.nonce ?? null,
          failOpenOnAccountMismatch: null,
        },
      },
      {
        crank: this.publicKey,
        oracleQueue: queueAccount.publicKey,
        queueAuthority: queueData.authority,
        programState: this.program.programState.publicKey,
        payoutWallet: payoutWallet,
        tokenProgram: TOKEN_PROGRAM_ID,
        crankDataBuffer: crankData.dataBuffer,
        queueDataBuffer: queueData.dataBuffer,
        mint: this.program.mint.address,
      }
    );
    crankPopIxn.keys.push(
      ...remainingAccounts.map((pubkey): AccountMeta => {
        return { isSigner: false, isWritable: true, pubkey };
      })
    );

    const txnObject: TransactionObject =
      (await this.program.connection.getAccountInfo(payoutWallet)) === null
        ? (() =>
            this.program.mint
              .createAssocatedUserInstruction(payer)[0]
              .add(crankPopIxn))()
        : new TransactionObject(payer, [crankPopIxn], []);
    return txnObject;
  }

  public async pop(params: CrankPopParams): Promise<TransactionSignature> {
    const popTxn = await this.popInstruction(this.program.walletPubkey, params);
    const txnSignature = await this.program.signAndSend(popTxn);
    return txnSignature;
  }

  /**
   * Pushes a new aggregator onto the crank.
   * @param params The crank push parameters.
   * @return TransactionSignature
   */
  async pushInstruction(
    payer: PublicKey,
    params: CrankPushParams
  ): Promise<TransactionObject> {
    const crankData = await this.loadData();
    const queueAccount = new QueueAccount(this.program, crankData.queuePubkey);
    const queue = await queueAccount.loadData();

    const { permissionAccount, permissionBump, leaseAccount, leaseEscrow } =
      params.aggregatorAccount.getAccounts({
        queueAccount: queueAccount,
        queueAuthority: queue.authority,
      });

    return new TransactionObject(
      payer,
      [
        types.crankPush(
          this.program,
          {
            params: {
              stateBump: this.program.programState.bump,
              permissionBump: permissionBump,
              notifiRef: null,
            },
          },
          {
            crank: this.publicKey,
            aggregator: params.aggregatorAccount.publicKey,
            oracleQueue: queueAccount.publicKey,
            queueAuthority: queue.authority,
            permission: permissionAccount.publicKey,
            lease: leaseAccount.publicKey,
            escrow: leaseEscrow,
            programState: this.program.programState.publicKey,
            dataBuffer:
              this.dataBuffer?.publicKey ?? (await this.loadData()).dataBuffer,
          }
        ),
      ],
      []
    );
  }

  /**
   * Pushes a new aggregator onto the crank.
   * @param params The crank push parameters.
   * @return TransactionSignature
   */
  async push(params: CrankPushParams): Promise<TransactionSignature> {
    const pushTxn = await this.pushInstruction(
      this.program.walletPubkey,
      params
    );
    const txnSignature = await this.program.signAndSend(pushTxn);
    return txnSignature;
  }

  /**
   * Get an array of the next aggregator pubkeys to be popped from the crank, limited by n
   * @param num The limit of pubkeys to return.
   * @return List of {@linkcode types.CrankRow}, ordered by timestamp.
   */
  async peakNextWithTime(num?: number): Promise<types.CrankRow[]> {
    let crank: CrankDataBuffer;
    if (this.dataBuffer) {
      crank = this.dataBuffer;
    } else {
      const crankData = await this.loadData();
      crank = new CrankDataBuffer(this.program, crankData.dataBuffer);
    }
    const crankRows = await crank.loadData();

    return crankRows.slice(0, num ?? crankRows.length);
  }

  /**
   * Get an array of the next readily updateable aggregator pubkeys to be popped
   * from the crank, limited by n
   * @param num The limit of pubkeys to return.
   * @return Pubkey list of Aggregator pubkeys.
   */
  async peakNextReady(num?: number): Promise<PublicKey[]> {
    const now = Math.floor(Date.now() / 1000);
    const crankRows = await this.peakNextWithTime(num);
    return crankRows
      .filter(row => now >= row.nextTimestamp.toNumber())
      .map(row => row.pubkey);
  }

  /**
   * Get an array of the next aggregator pubkeys to be popped from the crank, limited by n
   * @param num The limit of pubkeys to return.
   * @return Pubkey list of Aggregators next up to be popped.
   */
  async peakNext(num?: number): Promise<PublicKey[]> {
    const crankRows = await this.peakNextWithTime(num);
    return crankRows.map(row => row.pubkey);
  }

  /**
   * Load a cranks {@linkcode CrankDataBuffer}.
   * @return the list of aggregtors and their next available update time.
   */
  async loadCrank(): Promise<Array<types.CrankRow>> {
    if (!this.dataBuffer) {
      this.dataBuffer = new CrankDataBuffer(
        this.program,
        (await this.loadData()).dataBuffer
      );
    }

    const crankRows = await this.dataBuffer.loadData();

    return crankRows;
  }

  /** Whether an aggregator pubkey is active on a Crank */
  async isOnCrank(
    pubkey: PublicKey,
    crankRows?: Array<types.CrankRow>
  ): Promise<boolean> {
    const rows = crankRows ?? (await this.loadCrank());

    const idx = rows.findIndex(r => r.pubkey.equals(pubkey));
    if (idx === -1) {
      return false;
    }

    return true;
  }

  public async fetchAccounts(
    _crank?: types.CrankAccountData,
    _queueAccount?: QueueAccount,
    _queue?: types.OracleQueueAccountData
  ): Promise<CrankAccounts> {
    const crank = _crank ?? (await this.loadData());
    const queueAccount =
      _queueAccount ?? new QueueAccount(this.program, crank.queuePubkey);
    const queue = _queue ?? (await queueAccount.loadData());

    const crankRows = await this.loadCrank();

    const aggregatorPubkeys = crankRows.map(r => r.pubkey);
    const aggregators = await AggregatorAccount.fetchMultiple(
      this.program,
      aggregatorPubkeys
    );

    return {
      crank: {
        publicKey: this.publicKey,
        data: crank,
      },
      queue: {
        publicKey: queueAccount.publicKey,
        data: queue,
      },
      dataBuffer: {
        publicKey: crank.dataBuffer,
        data: crankRows,
      },
      aggregators: aggregators.map(a => {
        return {
          publicKey: a.account.publicKey,
          data: a.data,
        };
      }),
    };
  }

  public async toAccountsJSON(
    _crank?: types.CrankAccountData,
    _crankRows?: Array<types.CrankRow>
  ): Promise<CrankAccountsJSON> {
    const crank = _crank ?? (await this.loadData());
    const crankRows = _crankRows ?? (await this.loadCrank());

    return {
      publicKey: this.publicKey,
      ...crank.toJSON(),
      dataBuffer: {
        publicKey: crank.dataBuffer,
        data: crankRows,
      },
    };
  }
}

/**
 * Parameters for initializing a CrankAccount
 */
export interface CrankInitParams {
  /**
   *  OracleQueueAccount for which this crank is associated
   */
  queueAccount: QueueAccount;
  /**
   *  String specifying crank name
   */
  name?: string;
  /**
   *  String specifying crank metadata
   */
  metadata?: String;
  /**
   * Optional max number of rows
   */
  maxRows?: number;
  /**
   * Optional
   */
  keypair?: Keypair;
  /**
   * Optional
   */
  dataBufferKeypair?: Keypair;
}

/**
 * Parameters for pushing an element into a CrankAccount.
 */
export interface CrankPushParams {
  /**
   * Specifies the aggregator to push onto the crank.
   */
  aggregatorAccount: AggregatorAccount;
}

/**
 * Parameters for popping an element from a CrankAccount.
 */
export interface CrankPopParams {
  /**
   * Specifies the wallet to reward for turning the crank.
   *
   * Defaults to the payer.
   */
  payoutWallet?: PublicKey;
  /**
   * Array of pubkeys to attempt to pop. If discluded, this will be loaded
   * from the crank upon calling.
   */
  readyPubkeys?: PublicKey[];
  /**
   * Nonce to allow consecutive crank pops with the same blockhash.
   */
  nonce?: number;
  failOpenOnMismatch?: boolean;
  popIdx?: number;
}

export type CrankAccountsJSON = Omit<
  types.CrankAccountDataJSON,
  'dataBuffer'
> & {
  publicKey: PublicKey;
  dataBuffer: { publicKey: PublicKey; data: Array<types.CrankRow> };
};

export type CrankAccounts = {
  crank: {
    publicKey: PublicKey;
    data: types.CrankAccountData;
  };
  queue: {
    publicKey: PublicKey;
    data: types.OracleQueueAccountData;
  };
  dataBuffer: {
    publicKey: PublicKey;
    data: Array<types.CrankRow>;
  };
  aggregators: Array<{
    publicKey: PublicKey;
    data: types.AggregatorAccountData;
  }>;
};
