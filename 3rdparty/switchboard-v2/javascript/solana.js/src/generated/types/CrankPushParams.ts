import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface CrankPushParamsFields {
  stateBump: number;
  permissionBump: number;
  notifiRef: Array<number> | null;
}

export interface CrankPushParamsJSON {
  stateBump: number;
  permissionBump: number;
  notifiRef: Array<number> | null;
}

export class CrankPushParams {
  readonly stateBump: number;
  readonly permissionBump: number;
  readonly notifiRef: Array<number> | null;

  constructor(fields: CrankPushParamsFields) {
    this.stateBump = fields.stateBump;
    this.permissionBump = fields.permissionBump;
    this.notifiRef = fields.notifiRef;
  }

  static layout(property?: string) {
    return borsh.struct(
      [
        borsh.u8('stateBump'),
        borsh.u8('permissionBump'),
        borsh.option(borsh.array(borsh.u8(), 64), 'notifiRef'),
      ],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new CrankPushParams({
      stateBump: obj.stateBump,
      permissionBump: obj.permissionBump,
      notifiRef: obj.notifiRef,
    });
  }

  static toEncodable(fields: CrankPushParamsFields) {
    return {
      stateBump: fields.stateBump,
      permissionBump: fields.permissionBump,
      notifiRef: fields.notifiRef,
    };
  }

  toJSON(): CrankPushParamsJSON {
    return {
      stateBump: this.stateBump,
      permissionBump: this.permissionBump,
      notifiRef: this.notifiRef,
    };
  }

  static fromJSON(obj: CrankPushParamsJSON): CrankPushParams {
    return new CrankPushParams({
      stateBump: obj.stateBump,
      permissionBump: obj.permissionBump,
      notifiRef: obj.notifiRef,
    });
  }

  toEncodable() {
    return CrankPushParams.toEncodable(this);
  }
}
