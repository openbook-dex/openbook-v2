import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface CrankPopParamsFields {
  stateBump: number;
  leaseBumps: Uint8Array;
  permissionBumps: Uint8Array;
  nonce: number | null;
  failOpenOnAccountMismatch: boolean | null;
}

export interface CrankPopParamsJSON {
  stateBump: number;
  leaseBumps: Array<number>;
  permissionBumps: Array<number>;
  nonce: number | null;
  failOpenOnAccountMismatch: boolean | null;
}

export class CrankPopParams {
  readonly stateBump: number;
  readonly leaseBumps: Uint8Array;
  readonly permissionBumps: Uint8Array;
  readonly nonce: number | null;
  readonly failOpenOnAccountMismatch: boolean | null;

  constructor(fields: CrankPopParamsFields) {
    this.stateBump = fields.stateBump;
    this.leaseBumps = fields.leaseBumps;
    this.permissionBumps = fields.permissionBumps;
    this.nonce = fields.nonce;
    this.failOpenOnAccountMismatch = fields.failOpenOnAccountMismatch;
  }

  static layout(property?: string) {
    return borsh.struct(
      [
        borsh.u8('stateBump'),
        borsh.vecU8('leaseBumps'),
        borsh.vecU8('permissionBumps'),
        borsh.option(borsh.u32(), 'nonce'),
        borsh.option(borsh.bool(), 'failOpenOnAccountMismatch'),
      ],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new CrankPopParams({
      stateBump: obj.stateBump,
      leaseBumps: new Uint8Array(
        obj.leaseBumps.buffer,
        obj.leaseBumps.byteOffset,
        obj.leaseBumps.length
      ),
      permissionBumps: new Uint8Array(
        obj.permissionBumps.buffer,
        obj.permissionBumps.byteOffset,
        obj.permissionBumps.length
      ),
      nonce: obj.nonce,
      failOpenOnAccountMismatch: obj.failOpenOnAccountMismatch,
    });
  }

  static toEncodable(fields: CrankPopParamsFields) {
    return {
      stateBump: fields.stateBump,
      leaseBumps: Buffer.from(
        fields.leaseBumps.buffer,
        fields.leaseBumps.byteOffset,
        fields.leaseBumps.length
      ),
      permissionBumps: Buffer.from(
        fields.permissionBumps.buffer,
        fields.permissionBumps.byteOffset,
        fields.permissionBumps.length
      ),
      nonce: fields.nonce,
      failOpenOnAccountMismatch: fields.failOpenOnAccountMismatch,
    };
  }

  toJSON(): CrankPopParamsJSON {
    return {
      stateBump: this.stateBump,
      leaseBumps: Array.from(this.leaseBumps.values()),
      permissionBumps: Array.from(this.permissionBumps.values()),
      nonce: this.nonce,
      failOpenOnAccountMismatch: this.failOpenOnAccountMismatch,
    };
  }

  static fromJSON(obj: CrankPopParamsJSON): CrankPopParams {
    return new CrankPopParams({
      stateBump: obj.stateBump,
      leaseBumps: Uint8Array.from(obj.leaseBumps),
      permissionBumps: Uint8Array.from(obj.permissionBumps),
      nonce: obj.nonce,
      failOpenOnAccountMismatch: obj.failOpenOnAccountMismatch,
    });
  }

  toEncodable() {
    return CrankPopParams.toEncodable(this);
  }
}
