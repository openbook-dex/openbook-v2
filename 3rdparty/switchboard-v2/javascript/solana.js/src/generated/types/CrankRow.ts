import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface CrankRowFields {
  /** The PublicKey of the AggregatorAccountData. */
  pubkey: PublicKey;
  /** The aggregator's next available update time. */
  nextTimestamp: BN;
}

export interface CrankRowJSON {
  /** The PublicKey of the AggregatorAccountData. */
  pubkey: string;
  /** The aggregator's next available update time. */
  nextTimestamp: string;
}

export class CrankRow {
  /** The PublicKey of the AggregatorAccountData. */
  readonly pubkey: PublicKey;
  /** The aggregator's next available update time. */
  readonly nextTimestamp: BN;

  constructor(fields: CrankRowFields) {
    this.pubkey = fields.pubkey;
    this.nextTimestamp = fields.nextTimestamp;
  }

  static layout(property?: string) {
    return borsh.struct(
      [borsh.publicKey('pubkey'), borsh.i64('nextTimestamp')],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new CrankRow({
      pubkey: obj.pubkey,
      nextTimestamp: obj.nextTimestamp,
    });
  }

  static toEncodable(fields: CrankRowFields) {
    return {
      pubkey: fields.pubkey,
      nextTimestamp: fields.nextTimestamp,
    };
  }

  toJSON(): CrankRowJSON {
    return {
      pubkey: this.pubkey.toString(),
      nextTimestamp: this.nextTimestamp.toString(),
    };
  }

  static fromJSON(obj: CrankRowJSON): CrankRow {
    return new CrankRow({
      pubkey: new PublicKey(obj.pubkey),
      nextTimestamp: new BN(obj.nextTimestamp),
    });
  }

  toEncodable() {
    return CrankRow.toEncodable(this);
  }
}
