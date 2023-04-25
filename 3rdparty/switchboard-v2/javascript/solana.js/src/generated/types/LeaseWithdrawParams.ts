import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface LeaseWithdrawParamsFields {
  stateBump: number;
  leaseBump: number;
  amount: BN;
}

export interface LeaseWithdrawParamsJSON {
  stateBump: number;
  leaseBump: number;
  amount: string;
}

export class LeaseWithdrawParams {
  readonly stateBump: number;
  readonly leaseBump: number;
  readonly amount: BN;

  constructor(fields: LeaseWithdrawParamsFields) {
    this.stateBump = fields.stateBump;
    this.leaseBump = fields.leaseBump;
    this.amount = fields.amount;
  }

  static layout(property?: string) {
    return borsh.struct(
      [borsh.u8('stateBump'), borsh.u8('leaseBump'), borsh.u64('amount')],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new LeaseWithdrawParams({
      stateBump: obj.stateBump,
      leaseBump: obj.leaseBump,
      amount: obj.amount,
    });
  }

  static toEncodable(fields: LeaseWithdrawParamsFields) {
    return {
      stateBump: fields.stateBump,
      leaseBump: fields.leaseBump,
      amount: fields.amount,
    };
  }

  toJSON(): LeaseWithdrawParamsJSON {
    return {
      stateBump: this.stateBump,
      leaseBump: this.leaseBump,
      amount: this.amount.toString(),
    };
  }

  static fromJSON(obj: LeaseWithdrawParamsJSON): LeaseWithdrawParams {
    return new LeaseWithdrawParams({
      stateBump: obj.stateBump,
      leaseBump: obj.leaseBump,
      amount: new BN(obj.amount),
    });
  }

  toEncodable() {
    return LeaseWithdrawParams.toEncodable(this);
  }
}
