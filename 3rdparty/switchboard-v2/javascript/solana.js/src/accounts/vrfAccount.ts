import * as anchor from '@project-serum/anchor';
import * as spl from '@solana/spl-token';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import {
  Commitment,
  Keypair,
  ParsedTransactionWithMeta,
  PublicKey,
  SystemProgram,
  SYSVAR_INSTRUCTIONS_PUBKEY,
  SYSVAR_RECENT_BLOCKHASHES_PUBKEY,
  TransactionInstruction,
  TransactionSignature,
} from '@solana/web3.js';
import { promiseWithTimeout } from '@switchboard-xyz/common';
import * as errors from '../errors';
import * as types from '../generated';
import { SwitchboardProgram } from '../program';
import { TransactionObject } from '../transaction';
import { Account, OnAccountChangeCallback } from './account';
import { OracleAccount } from './oracleAccount';
import { PermissionAccount } from './permissionAccount';
import { QueueAccount } from './queueAccount';

/**
 * Account holding a Verifiable Random Function result with a callback instruction for consuming on-chain pseudo-randomness.
 *
 * Data: {@linkcode types.VrfAccountData}
 * Result: [u8;32]
 */
export class VrfAccount extends Account<types.VrfAccountData> {
  static accountName = 'VrfAccountData';

  /**
   * Returns the size of an on-chain {@linkcode VrfAccount}.
   */
  public readonly size = this.program.account.vrfAccountData.size;

  public static default(): types.VrfAccountData {
    const buffer = Buffer.alloc(29058, 0);
    types.VrfAccountData.discriminator.copy(buffer, 0);
    return types.VrfAccountData.decode(buffer);
  }

  /** Load an existing VrfAccount with its current on-chain state */
  public static async load(
    program: SwitchboardProgram,
    publicKey: PublicKey | string
  ): Promise<[VrfAccount, types.VrfAccountData]> {
    const account = new VrfAccount(
      program,
      typeof publicKey === 'string' ? new PublicKey(publicKey) : publicKey
    );
    const state = await account.loadData();
    return [account, state];
  }

  /**
   * Invoke a callback each time a VrfAccount's data has changed on-chain.
   * @param callback - the callback invoked when the vrf state changes
   * @param commitment - optional, the desired transaction finality. defaults to 'confirmed'
   * @returns the websocket subscription id
   */
  onChange(
    callback: OnAccountChangeCallback<types.VrfAccountData>,
    commitment: Commitment = 'confirmed'
  ): number {
    return this.program.connection.onAccountChange(
      this.publicKey,
      accountInfo => callback(types.VrfAccountData.decode(accountInfo.data)),
      commitment
    );
  }

  /**
   * Retrieve and decode the {@linkcode types.VrfAccountData} stored in this account.
   */
  async loadData(): Promise<types.VrfAccountData> {
    const data = await types.VrfAccountData.fetch(this.program, this.publicKey);
    if (data === null)
      throw new errors.AccountNotFoundError('Vrf', this.publicKey);
    return data;
  }

  /**
   *  Creates a list of instructions that will produce a {@linkcode VrfAccount}.
   *
   *  _NOTE_: The transaction that includes these instructions should be signed by both
   *  payer and vrfKeypair.
   */
  public static async createInstructions(
    program: SwitchboardProgram,
    payer: PublicKey,
    params: VrfInitParams
  ): Promise<[VrfAccount, TransactionObject]> {
    program.verifyNewKeypair(params.vrfKeypair);
    const vrfAccount = new VrfAccount(program, params.vrfKeypair.publicKey);
    const size = program.account.vrfAccountData.size;

    const escrow = program.mint.getAssociatedAddress(vrfAccount.publicKey);

    const ixns = [
      spl.createAssociatedTokenAccountInstruction(
        payer,
        escrow,
        params.vrfKeypair.publicKey,
        program.mint.address
      ),
      spl.createSetAuthorityInstruction(
        escrow,
        params.vrfKeypair.publicKey,
        spl.AuthorityType.AccountOwner,
        program.programState.publicKey
      ),
      SystemProgram.createAccount({
        fromPubkey: payer,
        newAccountPubkey: params.vrfKeypair.publicKey,
        space: size,
        lamports: await program.connection.getMinimumBalanceForRentExemption(
          size
        ),
        programId: program.programId,
      }),
      types.vrfInit(
        program,
        {
          params: {
            stateBump: program.programState.bump,
            callback: params.callback,
          },
        },
        {
          vrf: params.vrfKeypair.publicKey,
          authority: params.authority ?? payer,
          escrow,
          oracleQueue: params.queueAccount.publicKey,
          programState: program.programState.publicKey,
          tokenProgram: spl.TOKEN_PROGRAM_ID,
        }
      ),
    ];

    return [
      vrfAccount,
      new TransactionObject(payer, ixns, [params.vrfKeypair]),
    ];
  }

  /**
   *  Produces a Switchboard {@linkcode VrfAccount}.
   *
   *  _NOTE_: program wallet is used to sign the transaction.
   */
  public static async create(
    program: SwitchboardProgram,
    params: VrfInitParams
  ): Promise<[VrfAccount, string]> {
    const [vrfAccount, vrfInitTxn] = await VrfAccount.createInstructions(
      program,
      program.walletPubkey,
      params
    );
    const txnSignature = await program.signAndSend(vrfInitTxn);
    return [vrfAccount, txnSignature];
  }

  public async requestRandomnessInstruction(
    payer: PublicKey,
    params: VrfRequestRandomnessParams
  ): Promise<TransactionObject> {
    const vrf = params.vrf ?? (await this.loadData());
    const queueAccount =
      params.queueAccount ?? new QueueAccount(this.program, vrf.oracleQueue);
    const queue = params.queue ?? (await queueAccount.loadData());

    const [permissionAccount, permissionBump] = PermissionAccount.fromSeed(
      this.program,
      queue.authority,
      queueAccount.publicKey,
      this.publicKey
    );

    const requestRandomness = new TransactionObject(
      payer,
      [
        types.vrfRequestRandomness(
          this.program,
          {
            params: {
              stateBump: this.program.programState.bump,
              permissionBump,
            },
          },
          {
            authority: params.authority?.publicKey ?? payer,
            vrf: this.publicKey,
            oracleQueue: queueAccount.publicKey,
            queueAuthority: queue.authority,
            dataBuffer: queue.dataBuffer,
            permission: permissionAccount.publicKey,
            escrow: vrf.escrow,
            payerWallet: payer,
            payerAuthority: PublicKey.default,
            recentBlockhashes: SYSVAR_RECENT_BLOCKHASHES_PUBKEY,
            programState: this.program.programState.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          }
        ),
      ],
      params.authority ? [params.authority] : []
    );

    return requestRandomness;
  }

  public async requestRandomness(
    params: VrfRequestRandomnessParams
  ): Promise<TransactionSignature> {
    const requestRandomness = await this.requestRandomnessInstruction(
      this.program.walletPubkey,
      params
    );
    const txnSignature = await this.program.signAndSend(requestRandomness);
    return txnSignature;
  }

  public proveAndVerifyInstructions(
    params: VrfProveAndVerifyParams,
    tryCount = 278
  ): TransactionInstruction[] {
    const ixns: TransactionInstruction[] = [];

    const vrf = params.vrf;

    const idx = vrf.builders.findIndex(builder =>
      params.oraclePubkey.equals(builder.producer)
    );
    if (idx === -1) {
      throw new Error('OracleNotFoundError');
    }

    // only add proof in first ixn to optimally pack
    ixns.push(
      types.vrfProveAndVerify(
        this.program,
        {
          params: {
            nonce: 1,
            stateBump: this.program.programState.bump,
            idx: idx,
            proof: new Uint8Array(),
            proofEncoded: params.proof,
            counter: vrf.counter,
          },
        },
        {
          vrf: this.publicKey,
          callbackPid: vrf.callback.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          escrow: vrf.escrow,
          programState: this.program.programState.publicKey,
          oracle: params.oraclePubkey,
          oracleAuthority: params.oracleAuthority,
          oracleWallet: params.oracleTokenWallet,
          instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
        }
      )
    );

    // add verify ixns
    for (let i = 0; i < tryCount; ++i) {
      ixns.push(
        types.vrfProveAndVerify(
          this.program,
          {
            params: {
              nonce: i,
              stateBump: this.program.programState.bump,
              idx: idx,
              proof: new Uint8Array(),
              proofEncoded: '',
              counter: vrf.counter,
            },
          },
          {
            vrf: this.publicKey,
            callbackPid: vrf.callback.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            escrow: vrf.escrow,
            programState: this.program.programState.publicKey,
            oracle: params.oraclePubkey,
            oracleAuthority: params.oracleAuthority,
            oracleWallet: params.oracleTokenWallet,
            instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
          }
        )
      );
    }

    return ixns;
  }

  public async proveAndVerify(params: {
    vrf?: types.VrfAccountData;
    proof?: string;
    oraclePubkey?: PublicKey;
    oracleTokenWallet?: PublicKey;
    oracleAuthority?: PublicKey;
    skipPreflight?: boolean;
  }): Promise<Array<TransactionSignature>> {
    const vrf = params.vrf ?? (await this.loadData());
    const oraclePubkey = params.oraclePubkey ?? vrf.builders[0].producer;

    let oracleTokenWallet = params.oracleTokenWallet;
    let oracleAuthority = params.oracleAuthority;
    if (!oracleTokenWallet || !oracleAuthority) {
      const oracleAccount = new OracleAccount(this.program, oraclePubkey);
      const oracle = await oracleAccount.loadData();
      oracleTokenWallet = oracle.tokenAccount;
      oracleAuthority = oracle.oracleAuthority;
    }

    const ixns = this.proveAndVerifyInstructions({
      vrf,
      proof: params.proof ?? '',
      oraclePubkey,
      oracleTokenWallet,
      oracleAuthority,
    });

    const txns = TransactionObject.packIxns(this.program.walletPubkey, ixns);
    const txnSignatures = await this.program.signAndSendAll(txns, {
      skipPreflight: params.skipPreflight ?? true,
    });

    return txnSignatures;
  }

  public setCallbackInstruction(
    payer: PublicKey,
    params: {
      authority: Keypair | PublicKey;
      callback: Callback;
    }
  ): TransactionObject {
    const authorityPubkey =
      params.authority instanceof PublicKey
        ? params.authority
        : params.authority.publicKey;

    return new TransactionObject(
      payer,
      [
        types.vrfSetCallback(
          this.program,
          {
            params: {
              callback: params.callback,
            },
          },
          {
            vrf: this.publicKey,
            authority: authorityPubkey,
          }
        ),
      ],
      params.authority instanceof Keypair ? [params.authority] : []
    );
  }

  public async setCallback(params: {
    authority: Keypair | PublicKey;
    callback: Callback;
  }): Promise<TransactionSignature> {
    const setCallbackTxn = this.setCallbackInstruction(
      this.program.walletPubkey,
      params
    );
    const txnSignature = await this.program.signAndSend(setCallbackTxn);
    return txnSignature;
  }

  /** Return parsed transactions for a VRF request */
  public async getCallbackTransactions(
    requestSlot?: anchor.BN,
    txnLimit = 50
  ): Promise<Array<ParsedTransactionWithMeta>> {
    const slot =
      requestSlot ?? (await this.loadData()).currentRound.requestSlot;
    // TODO: Add options and allow getting signatures by slot
    const transactions = await this.program.connection.getSignaturesForAddress(
      this.publicKey,
      { limit: txnLimit, minContextSlot: slot.toNumber() }
    );
    const signatures = transactions.map(txn => txn.signature);
    const parsedTransactions =
      await this.program.connection.getParsedTransactions(signatures);

    const callbackTransactions: ParsedTransactionWithMeta[] = [];

    for (const txn of parsedTransactions) {
      if (txn === null) {
        continue;
      }

      const logs = txn.meta?.logMessages?.join('\n') ?? '';
      if (logs.includes('Invoking callback')) {
        callbackTransactions.push(txn);
      }
    }

    return callbackTransactions;
  }

  public getAccounts(params: {
    queueAccount: QueueAccount;
    queueAuthority: PublicKey;
  }) {
    const queueAccount = params.queueAccount;

    const [permissionAccount, permissionBump] = PermissionAccount.fromSeed(
      this.program,
      params.queueAuthority,
      queueAccount.publicKey,
      this.publicKey
    );

    return {
      queueAccount,
      permissionAccount,
      permissionBump,
    };
  }

  public async fetchAccounts(
    _vrf?: types.VrfAccountData,
    _queueAccount?: QueueAccount,
    _queue?: types.OracleQueueAccountData
  ): Promise<VrfAccounts> {
    const vrf = _vrf ?? (await this.loadData());

    const queueAccount =
      _queueAccount ?? new QueueAccount(this.program, vrf.oracleQueue);
    const queue = _queue ?? (await queueAccount.loadData());

    const { permissionAccount, permissionBump } = this.getAccounts({
      queueAccount,
      queueAuthority: queue.authority,
    });
    const permission = await permissionAccount.loadData();

    const vrfEscrow = await this.program.mint.getAccount(vrf.escrow);
    if (!vrfEscrow) {
      throw new errors.AccountNotFoundError('Vrf Escrow', vrf.escrow);
    }
    const vrfEscrowBalance: number = this.program.mint.fromTokenAmount(
      vrfEscrow.amount
    );

    return {
      vrf: { publicKey: this.publicKey, data: vrf },
      queue: {
        publicKey: queueAccount.publicKey,
        data: queue,
      },
      permission: {
        publicKey: permissionAccount.publicKey,
        bump: permissionBump,
        data: permission,
      },
      escrow: {
        publicKey: vrf.escrow,
        data: vrfEscrow,
        balance: vrfEscrowBalance,
      },
    };
  }

  public async toAccountsJSON(
    _vrf?: types.VrfAccountData,
    _queueAccount?: QueueAccount,
    _queue?: types.OracleQueueAccountData
  ): Promise<VrfAccountsJSON> {
    const accounts = await this.fetchAccounts(_vrf, _queueAccount, _queue);

    return {
      publicKey: this.publicKey,
      ...accounts.vrf.data.toJSON(),
      queue: {
        publicKey: accounts.queue.publicKey,
        ...accounts.queue.data.toJSON(),
      },
      permission: {
        publicKey: accounts.permission.publicKey,
        ...accounts.permission.data.toJSON(),
      },
      escrow: {
        publicKey: accounts.escrow.publicKey,
        balance: accounts.escrow.balance,
      },
    };
  }

  public async requestAndAwaitResult(
    params: { vrf?: types.VrfAccountData } & (
      | VrfRequestRandomnessParams
      | {
          requestFunction: (...args: any[]) => Promise<TransactionSignature>;
        }
    ),
    timeout = 30000
  ): Promise<[types.VrfAccountData, TransactionSignature]> {
    const vrf = params?.vrf ?? (await this.loadData());
    const currentRoundOpenSlot = vrf.currentRound.requestSlot;

    let ws: number | undefined = undefined;

    const closeWebsocket = async () => {
      if (ws !== undefined) {
        await this.program.connection.removeAccountChangeListener(ws);
        ws = undefined;
      }
    };

    const statePromise: Promise<types.VrfAccountData> = promiseWithTimeout(
      timeout,
      new Promise(
        (
          resolve: (result: types.VrfAccountData) => void,
          reject: (reason: string) => void
        ) => {
          ws = this.onChange(vrf => {
            if (vrf.currentRound.requestSlot.gt(currentRoundOpenSlot)) {
              if (
                vrf.status.kind ===
                  types.VrfStatus.StatusCallbackSuccess.kind ||
                vrf.status.kind === types.VrfStatus.StatusVerified.kind
              ) {
                resolve(vrf);
              }
              if (
                vrf.status.kind === types.VrfStatus.StatusVerifyFailure.kind
              ) {
                reject(
                  `Vrf failed to verify with status ${vrf.status.kind} (${vrf.status.discriminator})`
                );
              }
            }
          });
        }
      )
    ).finally(async () => {
      await closeWebsocket();
    });

    let requestRandomnessSignature: string | undefined = undefined;
    if ('requestFunction' in params) {
      requestRandomnessSignature = await params
        .requestFunction()
        .catch(async error => {
          await closeWebsocket();
          throw new Error(`Failed to call requestRandomness, ${error}`);
        });
    } else {
      requestRandomnessSignature = await this.requestRandomness(params).catch(
        async error => {
          await closeWebsocket();
          throw new Error(`Failed to call requestRandomness, ${error}`);
        }
      );
    }

    const state = await statePromise;

    await closeWebsocket();

    return [state, requestRandomnessSignature];
  }

  /**
   * Await for the next vrf result
   *
   * @param roundId - optional, the id associated with the VRF round to watch. If not provided the current round Id will be used.
   * @param timeout - the number of milliseconds to wait for the round to close
   *
   * @throws {string} when the timeout interval is exceeded or when the latestConfirmedRound.roundOpenSlot exceeds the target roundOpenSlot
   */
  public async nextResult(
    roundId?: anchor.BN,
    timeout = 30000
  ): Promise<VrfResult> {
    let id: anchor.BN;
    if (roundId) {
      id = roundId;
    } else {
      const vrf = await this.loadData();
      if (vrf.status.kind === 'StatusVerifying') {
        id = vrf.counter;
      } else {
        // wait for the next round
        id = vrf.counter.add(new anchor.BN(1));
      }
    }
    let ws: number | undefined;

    let result: VrfResult;
    try {
      result = await promiseWithTimeout(
        timeout,
        new Promise(
          (
            resolve: (result: VrfResult) => void,
            reject: (reason: string) => void
          ) => {
            ws = this.onChange(vrf => {
              if (vrf.counter.gt(id)) {
                reject(`Current counter is higher than requested roundId`);
              }
              if (vrf.counter.eq(id)) {
                switch (vrf.status.kind) {
                  case 'StatusCallbackSuccess': {
                    resolve({
                      success: true,
                      result: new Uint8Array(vrf.currentRound.result),
                      status: vrf.status,
                    });
                    break;
                  }
                  case 'StatusVerifyFailure': {
                    resolve({
                      success: false,
                      result: new Uint8Array(),
                      status: vrf.status,
                    });
                    break;
                  }
                }
              }
            });
          }
        )
      );
    } finally {
      if (ws !== undefined) {
        await this.program.connection.removeAccountChangeListener(ws);
        ws = undefined;
      }
    }

    if (ws !== undefined) {
      await this.program.connection.removeAccountChangeListener(ws);
    }

    return result;
  }
}

export interface VrfResult {
  success: boolean;
  result: Uint8Array;
  status: types.VrfStatusKind;
}

/**
 * Interface for a VRF callback.
 */
export interface Callback {
  programId: PublicKey;
  accounts: Array<anchor.web3.AccountMeta>;
  ixData: Buffer;
}

/**
 * Parameters for a VrfInit request.
 */
export interface VrfInitParams {
  /**
   *  Keypair to use for the vrf account
   */
  vrfKeypair: anchor.web3.Keypair;
  queueAccount: QueueAccount;
  /**
   * Callback function that is invoked when a new randomness value is produced.
   */
  callback: Callback;
  /**
   *  Optional authority for the resulting {@linkcode VrfAccount}. If not provided,
   *  the payer will default to the VRF authority.
   */
  authority?: PublicKey;
}

/**
 * Parameters for a VrfSetCallback request.
 */
export interface VrfSetCallbackParams {
  authority?: Keypair;
  callback: Callback;
  vrf: types.VrfAccountData;
}

export interface VrfProveAndVerifyParams {
  vrf: types.VrfAccountData;
  proof: string;
  oraclePubkey: PublicKey;
  oracleTokenWallet: PublicKey;
  oracleAuthority: PublicKey;
}

export interface VrfRequestRandomnessParams {
  authority?: Keypair;
  payerTokenWallet: PublicKey;
  payerAuthority?: Keypair;
  queue?: types.OracleQueueAccountData;
  queueAccount?: QueueAccount;
  vrf?: types.VrfAccountData;
}

export type VrfAccountsJSON = Omit<types.VrfAccountDataJSON, 'escrow'> & {
  publicKey: PublicKey;
  queue: types.OracleQueueAccountDataJSON & { publicKey: PublicKey };
  permission: types.PermissionAccountDataJSON & { publicKey: PublicKey };
  escrow: { publicKey: PublicKey; balance: number };
};

export type VrfAccounts = {
  vrf: {
    publicKey: PublicKey;
    data: types.VrfAccountData;
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
  escrow: {
    publicKey: PublicKey;
    data: spl.Account;
    balance: number;
  };
};
