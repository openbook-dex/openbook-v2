import * as anchor from '@project-serum/anchor';
import { ACCOUNT_DISCRIMINATOR_SIZE } from '@project-serum/anchor';
import {
  AccountInfo,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  TransactionSignature,
} from '@solana/web3.js';
import * as errors from '../errors';
import * as types from '../generated';
import {
  PermitOracleHeartbeat,
  PermitOracleQueueUsage,
  PermitVrfRequests,
} from '../generated/types/SwitchboardPermission';
import { SwitchboardProgram } from '../program';
import { TransactionObject } from '../transaction';
import { Account } from './account';

/**
 *  Parameters for initializing an {@linkcode PermissionAccount}
 */
export interface PermissionAccountInitParams {
  granter: PublicKey;
  grantee: PublicKey;
  authority: PublicKey;
}

export interface PermitNoneJSON {
  kind: 'PermitNone';
}

export class PermitNone {
  static readonly discriminator = 0;
  static readonly kind = 'NONE';
  readonly discriminator = 0;
  readonly kind = 'PermitNone';

  toJSON(): PermitNoneJSON {
    return {
      kind: 'PermitNone',
    };
  }

  toEncodable() {
    return {
      PermitOracleHeartbeat: {},
    };
  }
}

export interface PermissionSetParams {
  /** Whether to enable PERMIT_ORACLE_HEARTBEAT permissions. **Note:** Requires a provided queueAuthority keypair or payer to be the assigned queue authority. */
  enable: boolean;
  /** Keypair used to enable heartbeat permissions if payer is not the queue authority. */
  queueAuthority?: Keypair;
}

/**
 * Account type dictating the level of permissions between a granter and a grantee.
 *
 * A {@linkcode QueueAccount} acts as the granter where the queue authority assigns or revokes a grantee's {@linkcode types.SwitchboardPermission}. A grantee can be one of the following: {@linkcode AggregatorAccount}, {@linkcode BufferRelayerAccount}, or {@linkcode VrfAccount}.
 *
 * Data: {@linkcode types.PermissionAccountData}
 */
export class PermissionAccount extends Account<types.PermissionAccountData> {
  static accountName = 'PermissionAccountData';

  public static size = 372;

  /**
   * Returns the size of an on-chain {@linkcode PermissionAccount}.
   */
  public readonly size = this.program.account.permissionAccountData.size;

  static getPermissions(
    permission: types.PermissionAccountData
  ): types.SwitchboardPermissionKind | PermitNone {
    switch (permission.permissions) {
      case 0:
        return new PermitNone();
      case 1:
        return new PermitOracleHeartbeat();
      case 2:
        return new PermitOracleQueueUsage();
      case 3:
        return new PermitVrfRequests();
    }

    throw new Error(
      `Failed to find the assigned permissions, expected a value from 0 - 3, received ${permission.permissions}`
    );
  }

  public static default(): types.PermissionAccountData {
    const buffer = Buffer.alloc(PermissionAccount.size, 0);
    types.PermissionAccountData.discriminator.copy(buffer, 0);
    return types.PermissionAccountData.decode(buffer);
  }

  public static createMock(
    programId: PublicKey,
    data: Partial<types.PermissionAccountData>,
    options?: {
      lamports?: number;
      rentEpoch?: number;
    }
  ): AccountInfo<Buffer> {
    const fields: types.PermissionAccountDataFields = {
      ...PermissionAccount.default(),
      ...data,
      // any cleanup actions here
    };
    const state = new types.PermissionAccountData(fields);

    const buffer = Buffer.alloc(PermissionAccount.size, 0);
    types.PermissionAccountData.discriminator.copy(buffer, 0);
    types.PermissionAccountData.layout.encode(state, buffer, 8);

    return {
      executable: false,
      owner: programId,
      lamports: options?.lamports ?? 1 * LAMPORTS_PER_SOL,
      data: buffer,
      rentEpoch: options?.rentEpoch ?? 0,
    };
  }

  /** Load an existing PermissionAccount with its current on-chain state */
  public static async load(
    program: SwitchboardProgram,
    authority: PublicKey | string,
    granter: PublicKey | string,
    grantee: PublicKey | string
  ): Promise<[PermissionAccount, types.PermissionAccountData, number]> {
    const [account, bump] = PermissionAccount.fromSeed(
      program,
      typeof authority === 'string' ? new PublicKey(authority) : authority,
      typeof granter === 'string' ? new PublicKey(granter) : granter,
      typeof grantee === 'string' ? new PublicKey(grantee) : grantee
    );
    const state = await account.loadData();
    return [account, state, bump];
  }

  /**
   * Loads a PermissionAccount from the expected PDA seed format.
   * @param program The Switchboard program for the current connection.
   * @param authority The authority pubkey to be incorporated into the account seed.
   * @param granter The granter pubkey to be incorporated into the account seed.
   * @param grantee The grantee pubkey to be incorporated into the account seed.
   * @return PermissionAccount and PDA bump.
   */
  public static fromSeed(
    program: SwitchboardProgram,
    authority: PublicKey,
    granter: PublicKey,
    grantee: PublicKey
  ): [PermissionAccount, number] {
    const [publicKey, bump] = anchor.utils.publicKey.findProgramAddressSync(
      [
        Buffer.from('PermissionAccountData'),
        authority.toBytes(),
        granter.toBytes(),
        grantee.toBytes(),
      ],
      program.programId
    );
    return [new PermissionAccount(program, publicKey), bump];
  }

  public static createInstruction(
    program: SwitchboardProgram,
    payer: PublicKey,
    params: PermissionAccountInitParams
  ): [PermissionAccount, TransactionObject] {
    const [account] = PermissionAccount.fromSeed(
      program,
      params.authority,
      params.granter,
      params.grantee
    );
    const instruction = types.permissionInit(
      program,
      { params: {} },
      {
        permission: account.publicKey,
        granter: params.granter,
        grantee: params.grantee,
        systemProgram: SystemProgram.programId,
        authority: params.authority,
        payer,
      }
    );
    return [account, new TransactionObject(payer, [instruction], [])];
  }

  public static async create(
    program: SwitchboardProgram,
    params: PermissionAccountInitParams
  ): Promise<[PermissionAccount, TransactionSignature]> {
    const [account, txnObject] = this.createInstruction(
      program,
      program.walletPubkey,
      params
    );
    const txSignature = await program.signAndSend(txnObject);
    return [account, txSignature];
  }

  /**
   * Retrieve and decode the {@linkcode types.PermissionAccountData} stored in this account.
   */
  public async loadData(): Promise<types.PermissionAccountData> {
    const data = await types.PermissionAccountData.fetch(
      this.program,
      this.publicKey
    );
    if (data === null)
      throw new errors.AccountNotFoundError('Permissions', this.publicKey);
    return data;
  }

  /**
   * Check if a specific permission is enabled on this permission account
   */
  public async isPermissionEnabled(permission): Promise<boolean> {
    const permissions = (await this.loadData()).permissions;
    return (permissions & (permission as number)) !== 0;
  }

  /**
   * Sets the permission in the PermissionAccount
   */
  public async set(
    params: PermissionSetParams & {
      /** The {@linkcode types.SwitchboardPermission} to set for the grantee. */
      permission: types.SwitchboardPermissionKind;
    }
  ): Promise<string> {
    const setTxn = this.setInstruction(this.program.walletPubkey, params);
    const txnSignature = await this.program.signAndSend(setTxn);
    return txnSignature;
  }

  /**
   * Produces the instruction to set the permission in the PermissionAccount
   */
  public setInstruction(
    payer: PublicKey,
    params: PermissionSetParams & {
      /** The {@linkcode types.SwitchboardPermission} to set for the grantee. */
      permission: types.SwitchboardPermissionKind;
    }
  ): TransactionObject {
    return new TransactionObject(
      payer,
      [
        types.permissionSet(
          this.program,
          {
            params: {
              permission: params.permission,
              enable: params.enable,
            },
          },
          {
            permission: this.publicKey,
            authority: params.queueAuthority
              ? params.queueAuthority.publicKey
              : payer,
          }
        ),
      ],
      params.queueAuthority ? [params.queueAuthority] : []
    );
  }

  static getGranteePermissions(
    grantee: AccountInfo<Buffer>
  ): types.SwitchboardPermissionKind {
    if (grantee.data.byteLength < ACCOUNT_DISCRIMINATOR_SIZE) {
      throw new Error(`Cannot assign permissions to grantee`);
    }
    const discriminator = grantee.data.slice(0, ACCOUNT_DISCRIMINATOR_SIZE);

    // check oracle
    if (types.OracleAccountData.discriminator.compare(discriminator) === 0) {
      return new PermitOracleHeartbeat();
    }

    // check aggregator and buffer relayer
    if (
      types.AggregatorAccountData.discriminator.compare(discriminator) === 0 ||
      types.BufferRelayerAccountData.discriminator.compare(discriminator)
    ) {
      return new PermitOracleQueueUsage();
    }

    // check vrf
    if (types.VrfAccountData.discriminator.compare(discriminator) === 0) {
      return new PermitVrfRequests();
    }

    throw new Error(
      `Cannot find permissions to assign for account with discriminator of [${discriminator.join(
        ', '
      )}]`
    );
  }
}
