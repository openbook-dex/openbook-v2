import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createTransferInstruction,
  getAccount,
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
import * as spl from '@solana/spl-token';
import {
  Commitment,
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  TransactionInstruction,
  TransactionSignature,
} from '@solana/web3.js';
import Big from 'big.js';
import { BN } from 'bn.js';
import * as errors from '../errors';
import * as types from '../generated';
import { SwitchboardProgram } from '../program';
import { TransactionObject } from '../transaction';
import { Account, OnAccountChangeCallback } from './account';
import { JobAccount } from './jobAccount';
import { PermissionAccount } from './permissionAccount';
import { QueueAccount } from './queueAccount';

/**
 * Account type holding a buffer of data sourced from the buffers sole {@linkcode JobAccount}. A buffer relayer has no consensus mechanism and relies on trusting an {@linkcode OracleAccount} to respond honestly. A buffer relayer has a max capacity of 500 bytes.
 *
 * Data: {@linkcode types.BufferRelayerAccountData}
 */
export class BufferRelayerAccount extends Account<types.BufferRelayerAccountData> {
  static accountName = 'BufferRelayerAccountData';

  /**
   * Returns the size of an on-chain {@linkcode BufferRelayerAccount}.
   */
  public get size(): number {
    return this.program.account.bufferRelayerAccountData.size;
  }

  public decode(data: Buffer): types.BufferRelayerAccountData {
    try {
      return types.BufferRelayerAccountData.decode(data);
    } catch {
      return this.program.coder.decode<types.BufferRelayerAccountData>(
        BufferRelayerAccount.accountName,
        data
      );
    }
  }

  /**
   * Invoke a callback each time a BufferRelayerAccount's data has changed on-chain.
   * @param callback - the callback invoked when the buffer relayer state changes
   * @param commitment - optional, the desired transaction finality. defaults to 'confirmed'
   * @returns the websocket subscription id
   */
  public onChange(
    callback: OnAccountChangeCallback<types.BufferRelayerAccountData>,
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

  /** Load an existing BufferRelayer with its current on-chain state */
  public static async load(
    program: SwitchboardProgram,
    publicKey: PublicKey | string
  ): Promise<[BufferRelayerAccount, types.BufferRelayerAccountData]> {
    const account = new BufferRelayerAccount(
      program,
      typeof publicKey === 'string' ? new PublicKey(publicKey) : publicKey
    );
    const state = await account.loadData();
    return [account, state];
  }

  /**
   * Load and parse {@linkcode BufferRelayerAccount} data based on the program IDL.
   */
  public async loadData(): Promise<types.BufferRelayerAccountData> {
    const data = await types.BufferRelayerAccountData.fetch(
      this.program,
      this.publicKey
    );
    if (data === null)
      throw new errors.AccountNotFoundError('Buffer Relayer', this.publicKey);
    return data;
  }

  public static async createInstructions(
    program: SwitchboardProgram,
    payer: PublicKey,
    params: {
      name?: string;
      minUpdateDelaySeconds: number;
      queueAccount: QueueAccount;
      authority?: PublicKey;
      jobAccount: JobAccount;
      keypair?: Keypair;
    }
  ): Promise<[BufferRelayerAccount, TransactionObject]> {
    const keypair = params.keypair ?? Keypair.generate();
    program.verifyNewKeypair(keypair);

    const size = 2048;

    const ixns: TransactionInstruction[] = [];

    const escrow = program.mint.getAssociatedAddress(keypair.publicKey);

    ixns.push(
      SystemProgram.createAccount({
        fromPubkey: payer,
        newAccountPubkey: keypair.publicKey,
        space: size,
        lamports:
          await program.provider.connection.getMinimumBalanceForRentExemption(
            size
          ),
        programId: program.programId,
      })
    );

    ixns.push(
      types.bufferRelayerInit(
        program,
        {
          params: {
            name: [...Buffer.from(params.name ?? '', 'utf8').slice(0, 32)],
            minUpdateDelaySeconds: params.minUpdateDelaySeconds ?? 30,
            stateBump: program.programState.bump,
          },
        },
        {
          bufferRelayer: keypair.publicKey,
          escrow: escrow,
          authority: params.authority ?? payer,
          queue: params.queueAccount.publicKey,
          job: params.jobAccount.publicKey,
          programState: program.programState.publicKey,
          mint: program.mint.address,
          payer: payer,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        }
      )
    );

    return [
      new BufferRelayerAccount(program, keypair.publicKey),
      new TransactionObject(payer, ixns, [keypair]),
    ];
  }

  public static async create(
    program: SwitchboardProgram,
    params: {
      name?: string;
      minUpdateDelaySeconds: number;
      queueAccount: QueueAccount;
      authority?: PublicKey;
      jobAccount: JobAccount;
      keypair?: Keypair;
    }
  ): Promise<[BufferRelayerAccount, TransactionSignature]> {
    const [bufferAccount, bufferInit] =
      await BufferRelayerAccount.createInstructions(
        program,
        program.walletPubkey,
        params
      );
    const txnSignature = await program.signAndSend(bufferInit);
    return [bufferAccount, txnSignature];
  }

  public async openRoundInstructions(
    payer: PublicKey,
    params: {
      tokenWallet: PublicKey;
      bufferRelayer?: types.BufferRelayerAccountData;
      queueAccount?: QueueAccount;
      queue?: types.OracleQueueAccountData;
    }
  ): Promise<TransactionObject> {
    const ixns: TransactionInstruction[] = [];
    const bufferRelayer = params.bufferRelayer ?? (await this.loadData());

    const queueAccount =
      params.queueAccount ??
      new QueueAccount(this.program, bufferRelayer.queuePubkey);
    const queue = params.queue ?? (await queueAccount.loadData());

    const tokenAccount = await getAccount(
      this.program.connection,
      params.tokenWallet
    );
    const tokenAmountBN = new BN(tokenAccount.amount.toString());
    if (tokenAmountBN.lt(queue.reward)) {
      const wrapTxn = await this.program.mint.wrapInstructions(payer, {
        fundUpTo: new Big(this.program.mint.fromTokenAmountBN(queue.reward)),
      });
      ixns.push(...wrapTxn.ixns);
    }
    const [permissionAccount, permissionBump] = PermissionAccount.fromSeed(
      this.program,
      queue.authority,
      queueAccount.publicKey,
      this.publicKey
    );

    ixns.push(
      createTransferInstruction(
        params.tokenWallet,
        bufferRelayer.escrow,
        payer,
        BigInt(queue.reward.toString())
      ),
      types.bufferRelayerOpenRound(
        this.program,
        {
          params: {
            stateBump: this.program.programState.bump,
            permissionBump,
          },
        },
        {
          bufferRelayer: this.publicKey,
          oracleQueue: queueAccount.publicKey,
          dataBuffer: queue.dataBuffer,
          permission: permissionAccount.publicKey,
          escrow: bufferRelayer.escrow,
          programState: this.program.programState.publicKey,
        }
      )
    );

    return new TransactionObject(payer, ixns, []);
  }

  public async openRound(params: {
    tokenWallet: PublicKey;
    bufferRelayer?: types.BufferRelayerAccountData;
    queueAccount?: QueueAccount;
    queue?: types.OracleQueueAccountData;
  }): Promise<TransactionSignature> {
    const openRound = await this.openRoundInstructions(
      this.program.walletPubkey,
      params
    );
    const txnSignature = await this.program.signAndSend(openRound);
    return txnSignature;
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
    _bufferRelayer?: types.BufferRelayerAccountData,
    _queueAccount?: QueueAccount,
    _queue?: types.OracleQueueAccountData
  ): Promise<BufferRelayerAccounts> {
    const bufferRelayer = _bufferRelayer ?? (await this.loadData());

    const queueAccount =
      _queueAccount ??
      new QueueAccount(this.program, bufferRelayer.queuePubkey);
    const queue = _queue ?? (await queueAccount.loadData());

    const { permissionAccount, permissionBump } = this.getAccounts({
      queueAccount,
      queueAuthority: queue.authority,
    });
    const permission = await permissionAccount.loadData();

    const bufferEscrow = await this.program.mint.getAccount(
      bufferRelayer.escrow
    );
    if (!bufferEscrow) {
      throw new errors.AccountNotFoundError(
        'Buffer Relayer Escrow',
        bufferRelayer.escrow
      );
    }
    const vrfEscrowBalance: number = this.program.mint.fromTokenAmount(
      bufferEscrow.amount
    );

    return {
      bufferRelayer: { publicKey: this.publicKey, data: bufferRelayer },
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
        publicKey: bufferRelayer.escrow,
        data: bufferEscrow,
        balance: vrfEscrowBalance,
      },
    };
  }
}

export interface BufferRelayerInit {
  name?: string;
  minUpdateDelaySeconds: number;
  queueAccount: QueueAccount;
  authority?: PublicKey;
  jobAccount: JobAccount;
  keypair?: Keypair;
}

export type BufferRelayerAccounts = {
  bufferRelayer: {
    publicKey: PublicKey;
    data: types.BufferRelayerAccountData;
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
