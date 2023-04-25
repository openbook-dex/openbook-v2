import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface OracleWithdrawParamsFields {
  stateBump: number;
  permissionBump: number;
  amount: BN;
}

export interface OracleWithdrawParamsJSON {
  stateBump: number;
  permissionBump: number;
  amount: string;
}

export class OracleWithdrawParams {
  readonly stateBump: number;
  readonly permissionBump: number;
  readonly amount: BN;

  constructor(fields: OracleWithdrawParamsFields) {
    this.stateBump = fields.stateBump;
    this.permissionBump = fields.permissionBump;
    this.amount = fields.amount;
  }

  static layout(property?: string) {
    return borsh.struct(
      [borsh.u8('stateBump'), borsh.u8('permissionBump'), borsh.u64('amount')],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new OracleWithdrawParams({
      stateBump: obj.stateBump,
      permissionBump: obj.permissionBump,
      amount: obj.amount,
    });
  }

  static toEncodable(fields: OracleWithdrawParamsFields) {
    return {
      stateBump: fields.stateBump,
      permissionBump: fields.permissionBump,
      amount: fields.amount,
    };
  }

  toJSON(): OracleWithdrawParamsJSON {
    return {
      stateBump: this.stateBump,
      permissionBump: this.permissionBump,
      amount: this.amount.toString(),
    };
  }

  static fromJSON(obj: OracleWithdrawParamsJSON): OracleWithdrawParams {
    return new OracleWithdrawParams({
      stateBump: obj.stateBump,
      permissionBump: obj.permissionBump,
      amount: new BN(obj.amount),
    });
  }

  toEncodable() {
    return OracleWithdrawParams.toEncodable(this);
  }
}
