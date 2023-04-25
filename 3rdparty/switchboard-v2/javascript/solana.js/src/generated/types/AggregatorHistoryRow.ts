import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface AggregatorHistoryRowFields {
  /** The timestamp of the sample. */
  timestamp: BN;
  /** The value of the sample. */
  value: types.SwitchboardDecimalFields;
}

export interface AggregatorHistoryRowJSON {
  /** The timestamp of the sample. */
  timestamp: string;
  /** The value of the sample. */
  value: types.SwitchboardDecimalJSON;
}

export class AggregatorHistoryRow {
  /** The timestamp of the sample. */
  readonly timestamp: BN;
  /** The value of the sample. */
  readonly value: types.SwitchboardDecimal;

  constructor(fields: AggregatorHistoryRowFields) {
    this.timestamp = fields.timestamp;
    this.value = new types.SwitchboardDecimal({ ...fields.value });
  }

  static layout(property?: string) {
    return borsh.struct(
      [borsh.i64('timestamp'), types.SwitchboardDecimal.layout('value')],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new AggregatorHistoryRow({
      timestamp: obj.timestamp,
      value: types.SwitchboardDecimal.fromDecoded(obj.value),
    });
  }

  static toEncodable(fields: AggregatorHistoryRowFields) {
    return {
      timestamp: fields.timestamp,
      value: types.SwitchboardDecimal.toEncodable(fields.value),
    };
  }

  toJSON(): AggregatorHistoryRowJSON {
    return {
      timestamp: this.timestamp.toString(),
      value: this.value.toJSON(),
    };
  }

  static fromJSON(obj: AggregatorHistoryRowJSON): AggregatorHistoryRow {
    return new AggregatorHistoryRow({
      timestamp: new BN(obj.timestamp),
      value: types.SwitchboardDecimal.fromJSON(obj.value),
    });
  }

  toEncodable() {
    return AggregatorHistoryRow.toEncodable(this);
  }
}
