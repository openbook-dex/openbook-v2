import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface ProgramInitParamsFields {
  stateBump: number;
}

export interface ProgramInitParamsJSON {
  stateBump: number;
}

export class ProgramInitParams {
  readonly stateBump: number;

  constructor(fields: ProgramInitParamsFields) {
    this.stateBump = fields.stateBump;
  }

  static layout(property?: string) {
    return borsh.struct([borsh.u8('stateBump')], property);
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new ProgramInitParams({
      stateBump: obj.stateBump,
    });
  }

  static toEncodable(fields: ProgramInitParamsFields) {
    return {
      stateBump: fields.stateBump,
    };
  }

  toJSON(): ProgramInitParamsJSON {
    return {
      stateBump: this.stateBump,
    };
  }

  static fromJSON(obj: ProgramInitParamsJSON): ProgramInitParams {
    return new ProgramInitParams({
      stateBump: obj.stateBump,
    });
  }

  toEncodable() {
    return ProgramInitParams.toEncodable(this);
  }
}
