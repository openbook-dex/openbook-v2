import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface ProgramConfigParamsFields {
  token: PublicKey;
  bump: number;
  daoMint: PublicKey;
}

export interface ProgramConfigParamsJSON {
  token: string;
  bump: number;
  daoMint: string;
}

export class ProgramConfigParams {
  readonly token: PublicKey;
  readonly bump: number;
  readonly daoMint: PublicKey;

  constructor(fields: ProgramConfigParamsFields) {
    this.token = fields.token;
    this.bump = fields.bump;
    this.daoMint = fields.daoMint;
  }

  static layout(property?: string) {
    return borsh.struct(
      [borsh.publicKey('token'), borsh.u8('bump'), borsh.publicKey('daoMint')],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new ProgramConfigParams({
      token: obj.token,
      bump: obj.bump,
      daoMint: obj.daoMint,
    });
  }

  static toEncodable(fields: ProgramConfigParamsFields) {
    return {
      token: fields.token,
      bump: fields.bump,
      daoMint: fields.daoMint,
    };
  }

  toJSON(): ProgramConfigParamsJSON {
    return {
      token: this.token.toString(),
      bump: this.bump,
      daoMint: this.daoMint.toString(),
    };
  }

  static fromJSON(obj: ProgramConfigParamsJSON): ProgramConfigParams {
    return new ProgramConfigParams({
      token: new PublicKey(obj.token),
      bump: obj.bump,
      daoMint: new PublicKey(obj.daoMint),
    });
  }

  toEncodable() {
    return ProgramConfigParams.toEncodable(this);
  }
}
