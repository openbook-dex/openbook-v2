import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface CrankPopParamsV2Fields {
  stateBump: number;
  leaseBumps: Uint8Array;
  permissionBumps: Uint8Array;
  nonce: number | null;
  failOpenOnAccountMismatch: boolean | null;
  popIdx: number | null;
}

export interface CrankPopParamsV2JSON {
  stateBump: number;
  leaseBumps: Array<number>;
  permissionBumps: Array<number>;
  nonce: number | null;
  failOpenOnAccountMismatch: boolean | null;
  popIdx: number | null;
}

export class CrankPopParamsV2 {
  readonly stateBump: number;
  readonly leaseBumps: Uint8Array;
  readonly permissionBumps: Uint8Array;
  readonly nonce: number | null;
  readonly failOpenOnAccountMismatch: boolean | null;
  readonly popIdx: number | null;

  constructor(fields: CrankPopParamsV2Fields) {
    this.stateBump = fields.stateBump;
    this.leaseBumps = fields.leaseBumps;
    this.permissionBumps = fields.permissionBumps;
    this.nonce = fields.nonce;
    this.failOpenOnAccountMismatch = fields.failOpenOnAccountMismatch;
    this.popIdx = fields.popIdx;
  }

  static layout(property?: string) {
    return borsh.struct(
      [
        borsh.u8('stateBump'),
        borsh.vecU8('leaseBumps'),
        borsh.vecU8('permissionBumps'),
        borsh.option(borsh.u32(), 'nonce'),
        borsh.option(borsh.bool(), 'failOpenOnAccountMismatch'),
        borsh.option(borsh.u32(), 'popIdx'),
      ],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new CrankPopParamsV2({
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
      popIdx: obj.popIdx,
    });
  }

  static toEncodable(fields: CrankPopParamsV2Fields) {
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
      popIdx: fields.popIdx,
    };
  }

  toJSON(): CrankPopParamsV2JSON {
    return {
      stateBump: this.stateBump,
      leaseBumps: Array.from(this.leaseBumps.values()),
      permissionBumps: Array.from(this.permissionBumps.values()),
      nonce: this.nonce,
      failOpenOnAccountMismatch: this.failOpenOnAccountMismatch,
      popIdx: this.popIdx,
    };
  }

  static fromJSON(obj: CrankPopParamsV2JSON): CrankPopParamsV2 {
    return new CrankPopParamsV2({
      stateBump: obj.stateBump,
      leaseBumps: Uint8Array.from(obj.leaseBumps),
      permissionBumps: Uint8Array.from(obj.permissionBumps),
      nonce: obj.nonce,
      failOpenOnAccountMismatch: obj.failOpenOnAccountMismatch,
      popIdx: obj.popIdx,
    });
  }

  toEncodable() {
    return CrankPopParamsV2.toEncodable(this);
  }
}
