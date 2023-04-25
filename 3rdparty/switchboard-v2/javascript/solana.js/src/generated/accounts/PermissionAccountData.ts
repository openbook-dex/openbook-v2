import { SwitchboardProgram } from '../../program';
import { PublicKey, Connection } from '@solana/web3.js';
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars

export interface PermissionAccountDataFields {
  /** The authority that is allowed to set permissions for this account. */
  authority: PublicKey;
  /** The SwitchboardPermission enumeration assigned by the granter to the grantee. */
  permissions: number;
  /** Public key of account that is granting permissions to use its resources. */
  granter: PublicKey;
  /** Public key of account that is being assigned permissions to use a granters resources. */
  grantee: PublicKey;
  /**
   * unused currently. may want permission PDA per permission for
   * unique expiration periods, BUT currently only one permission
   * per account makes sense for the infra. Dont over engineer.
   */
  expiration: BN;
  /** Reserved for future info. */
  ebuf: Array<number>;
}

export interface PermissionAccountDataJSON {
  /** The authority that is allowed to set permissions for this account. */
  authority: string;
  /** The SwitchboardPermission enumeration assigned by the granter to the grantee. */
  permissions: number;
  /** Public key of account that is granting permissions to use its resources. */
  granter: string;
  /** Public key of account that is being assigned permissions to use a granters resources. */
  grantee: string;
  /**
   * unused currently. may want permission PDA per permission for
   * unique expiration periods, BUT currently only one permission
   * per account makes sense for the infra. Dont over engineer.
   */
  expiration: string;
  /** Reserved for future info. */
  ebuf: Array<number>;
}

export class PermissionAccountData {
  /** The authority that is allowed to set permissions for this account. */
  readonly authority: PublicKey;
  /** The SwitchboardPermission enumeration assigned by the granter to the grantee. */
  readonly permissions: number;
  /** Public key of account that is granting permissions to use its resources. */
  readonly granter: PublicKey;
  /** Public key of account that is being assigned permissions to use a granters resources. */
  readonly grantee: PublicKey;
  /**
   * unused currently. may want permission PDA per permission for
   * unique expiration periods, BUT currently only one permission
   * per account makes sense for the infra. Dont over engineer.
   */
  readonly expiration: BN;
  /** Reserved for future info. */
  readonly ebuf: Array<number>;

  static readonly discriminator = Buffer.from([
    77, 37, 177, 164, 38, 39, 34, 109,
  ]);

  static readonly layout = borsh.struct([
    borsh.publicKey('authority'),
    borsh.u32('permissions'),
    borsh.publicKey('granter'),
    borsh.publicKey('grantee'),
    borsh.i64('expiration'),
    borsh.array(borsh.u8(), 256, 'ebuf'),
  ]);

  constructor(fields: PermissionAccountDataFields) {
    this.authority = fields.authority;
    this.permissions = fields.permissions;
    this.granter = fields.granter;
    this.grantee = fields.grantee;
    this.expiration = fields.expiration;
    this.ebuf = fields.ebuf;
  }

  static async fetch(
    program: SwitchboardProgram,
    address: PublicKey
  ): Promise<PermissionAccountData | null> {
    const info = await program.connection.getAccountInfo(address);

    if (info === null) {
      return null;
    }
    if (!info.owner.equals(program.programId)) {
      throw new Error("account doesn't belong to this program");
    }

    return this.decode(info.data);
  }

  static async fetchMultiple(
    program: SwitchboardProgram,
    addresses: PublicKey[]
  ): Promise<Array<PermissionAccountData | null>> {
    const infos = await program.connection.getMultipleAccountsInfo(addresses);

    return infos.map(info => {
      if (info === null) {
        return null;
      }
      if (!info.owner.equals(program.programId)) {
        throw new Error("account doesn't belong to this program");
      }

      return this.decode(info.data);
    });
  }

  static decode(data: Buffer): PermissionAccountData {
    if (!data.slice(0, 8).equals(PermissionAccountData.discriminator)) {
      throw new Error('invalid account discriminator');
    }

    const dec = PermissionAccountData.layout.decode(data.slice(8));

    return new PermissionAccountData({
      authority: dec.authority,
      permissions: dec.permissions,
      granter: dec.granter,
      grantee: dec.grantee,
      expiration: dec.expiration,
      ebuf: dec.ebuf,
    });
  }

  toJSON(): PermissionAccountDataJSON {
    return {
      authority: this.authority.toString(),
      permissions: this.permissions,
      granter: this.granter.toString(),
      grantee: this.grantee.toString(),
      expiration: this.expiration.toString(),
      ebuf: this.ebuf,
    };
  }

  static fromJSON(obj: PermissionAccountDataJSON): PermissionAccountData {
    return new PermissionAccountData({
      authority: new PublicKey(obj.authority),
      permissions: obj.permissions,
      granter: new PublicKey(obj.granter),
      grantee: new PublicKey(obj.grantee),
      expiration: new BN(obj.expiration),
      ebuf: obj.ebuf,
    });
  }
}
