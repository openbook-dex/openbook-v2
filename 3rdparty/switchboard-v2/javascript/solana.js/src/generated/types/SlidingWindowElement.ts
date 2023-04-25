import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface SlidingWindowElementFields {
  oracleKey: PublicKey;
  value: types.SwitchboardDecimalFields;
  slot: BN;
  timestamp: BN;
}

export interface SlidingWindowElementJSON {
  oracleKey: string;
  value: types.SwitchboardDecimalJSON;
  slot: string;
  timestamp: string;
}

export class SlidingWindowElement {
  readonly oracleKey: PublicKey;
  readonly value: types.SwitchboardDecimal;
  readonly slot: BN;
  readonly timestamp: BN;

  constructor(fields: SlidingWindowElementFields) {
    this.oracleKey = fields.oracleKey;
    this.value = new types.SwitchboardDecimal({ ...fields.value });
    this.slot = fields.slot;
    this.timestamp = fields.timestamp;
  }

  static layout(property?: string) {
    return borsh.struct(
      [
        borsh.publicKey('oracleKey'),
        types.SwitchboardDecimal.layout('value'),
        borsh.u64('slot'),
        borsh.i64('timestamp'),
      ],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new SlidingWindowElement({
      oracleKey: obj.oracleKey,
      value: types.SwitchboardDecimal.fromDecoded(obj.value),
      slot: obj.slot,
      timestamp: obj.timestamp,
    });
  }

  static toEncodable(fields: SlidingWindowElementFields) {
    return {
      oracleKey: fields.oracleKey,
      value: types.SwitchboardDecimal.toEncodable(fields.value),
      slot: fields.slot,
      timestamp: fields.timestamp,
    };
  }

  toJSON(): SlidingWindowElementJSON {
    return {
      oracleKey: this.oracleKey.toString(),
      value: this.value.toJSON(),
      slot: this.slot.toString(),
      timestamp: this.timestamp.toString(),
    };
  }

  static fromJSON(obj: SlidingWindowElementJSON): SlidingWindowElement {
    return new SlidingWindowElement({
      oracleKey: new PublicKey(obj.oracleKey),
      value: types.SwitchboardDecimal.fromJSON(obj.value),
      slot: new BN(obj.slot),
      timestamp: new BN(obj.timestamp),
    });
  }

  toEncodable() {
    return SlidingWindowElement.toEncodable(this);
  }
}
