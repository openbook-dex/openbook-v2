import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface LeaseExtendParamsFields {
  loadAmount: BN;
  leaseBump: number;
  stateBump: number;
  walletBumps: Uint8Array;
}

export interface LeaseExtendParamsJSON {
  loadAmount: string;
  leaseBump: number;
  stateBump: number;
  walletBumps: Array<number>;
}

export class LeaseExtendParams {
  readonly loadAmount: BN;
  readonly leaseBump: number;
  readonly stateBump: number;
  readonly walletBumps: Uint8Array;

  constructor(fields: LeaseExtendParamsFields) {
    this.loadAmount = fields.loadAmount;
    this.leaseBump = fields.leaseBump;
    this.stateBump = fields.stateBump;
    this.walletBumps = fields.walletBumps;
  }

  static layout(property?: string) {
    return borsh.struct(
      [
        borsh.u64('loadAmount'),
        borsh.u8('leaseBump'),
        borsh.u8('stateBump'),
        borsh.vecU8('walletBumps'),
      ],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new LeaseExtendParams({
      loadAmount: obj.loadAmount,
      leaseBump: obj.leaseBump,
      stateBump: obj.stateBump,
      walletBumps: new Uint8Array(
        obj.walletBumps.buffer,
        obj.walletBumps.byteOffset,
        obj.walletBumps.length
      ),
    });
  }

  static toEncodable(fields: LeaseExtendParamsFields) {
    return {
      loadAmount: fields.loadAmount,
      leaseBump: fields.leaseBump,
      stateBump: fields.stateBump,
      walletBumps: Buffer.from(
        fields.walletBumps.buffer,
        fields.walletBumps.byteOffset,
        fields.walletBumps.length
      ),
    };
  }

  toJSON(): LeaseExtendParamsJSON {
    return {
      loadAmount: this.loadAmount.toString(),
      leaseBump: this.leaseBump,
      stateBump: this.stateBump,
      walletBumps: Array.from(this.walletBumps.values()),
    };
  }

  static fromJSON(obj: LeaseExtendParamsJSON): LeaseExtendParams {
    return new LeaseExtendParams({
      loadAmount: new BN(obj.loadAmount),
      leaseBump: obj.leaseBump,
      stateBump: obj.stateBump,
      walletBumps: Uint8Array.from(obj.walletBumps),
    });
  }

  toEncodable() {
    return LeaseExtendParams.toEncodable(this);
  }
}
