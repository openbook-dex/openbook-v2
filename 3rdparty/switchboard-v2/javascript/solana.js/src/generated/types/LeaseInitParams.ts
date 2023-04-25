import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface LeaseInitParamsFields {
  loadAmount: BN;
  withdrawAuthority: PublicKey;
  leaseBump: number;
  stateBump: number;
  walletBumps: Uint8Array;
}

export interface LeaseInitParamsJSON {
  loadAmount: string;
  withdrawAuthority: string;
  leaseBump: number;
  stateBump: number;
  walletBumps: Array<number>;
}

export class LeaseInitParams {
  readonly loadAmount: BN;
  readonly withdrawAuthority: PublicKey;
  readonly leaseBump: number;
  readonly stateBump: number;
  readonly walletBumps: Uint8Array;

  constructor(fields: LeaseInitParamsFields) {
    this.loadAmount = fields.loadAmount;
    this.withdrawAuthority = fields.withdrawAuthority;
    this.leaseBump = fields.leaseBump;
    this.stateBump = fields.stateBump;
    this.walletBumps = fields.walletBumps;
  }

  static layout(property?: string) {
    return borsh.struct(
      [
        borsh.u64('loadAmount'),
        borsh.publicKey('withdrawAuthority'),
        borsh.u8('leaseBump'),
        borsh.u8('stateBump'),
        borsh.vecU8('walletBumps'),
      ],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new LeaseInitParams({
      loadAmount: obj.loadAmount,
      withdrawAuthority: obj.withdrawAuthority,
      leaseBump: obj.leaseBump,
      stateBump: obj.stateBump,
      walletBumps: new Uint8Array(
        obj.walletBumps.buffer,
        obj.walletBumps.byteOffset,
        obj.walletBumps.length
      ),
    });
  }

  static toEncodable(fields: LeaseInitParamsFields) {
    return {
      loadAmount: fields.loadAmount,
      withdrawAuthority: fields.withdrawAuthority,
      leaseBump: fields.leaseBump,
      stateBump: fields.stateBump,
      walletBumps: Buffer.from(
        fields.walletBumps.buffer,
        fields.walletBumps.byteOffset,
        fields.walletBumps.length
      ),
    };
  }

  toJSON(): LeaseInitParamsJSON {
    return {
      loadAmount: this.loadAmount.toString(),
      withdrawAuthority: this.withdrawAuthority.toString(),
      leaseBump: this.leaseBump,
      stateBump: this.stateBump,
      walletBumps: Array.from(this.walletBumps.values()),
    };
  }

  static fromJSON(obj: LeaseInitParamsJSON): LeaseInitParams {
    return new LeaseInitParams({
      loadAmount: new BN(obj.loadAmount),
      withdrawAuthority: new PublicKey(obj.withdrawAuthority),
      leaseBump: obj.leaseBump,
      stateBump: obj.stateBump,
      walletBumps: Uint8Array.from(obj.walletBumps),
    });
  }

  toEncodable() {
    return LeaseInitParams.toEncodable(this);
  }
}
