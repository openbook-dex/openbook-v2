import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface OracleQueueSetRewardsParamsFields {
  rewards: BN;
}

export interface OracleQueueSetRewardsParamsJSON {
  rewards: string;
}

export class OracleQueueSetRewardsParams {
  readonly rewards: BN;

  constructor(fields: OracleQueueSetRewardsParamsFields) {
    this.rewards = fields.rewards;
  }

  static layout(property?: string) {
    return borsh.struct([borsh.u64('rewards')], property);
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new OracleQueueSetRewardsParams({
      rewards: obj.rewards,
    });
  }

  static toEncodable(fields: OracleQueueSetRewardsParamsFields) {
    return {
      rewards: fields.rewards,
    };
  }

  toJSON(): OracleQueueSetRewardsParamsJSON {
    return {
      rewards: this.rewards.toString(),
    };
  }

  static fromJSON(
    obj: OracleQueueSetRewardsParamsJSON
  ): OracleQueueSetRewardsParams {
    return new OracleQueueSetRewardsParams({
      rewards: new BN(obj.rewards),
    });
  }

  toEncodable() {
    return OracleQueueSetRewardsParams.toEncodable(this);
  }
}
