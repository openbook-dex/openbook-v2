import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface CallbackFields {
  programId: PublicKey;
  accounts: Array<types.AccountMetaBorshFields>;
  ixData: Uint8Array;
}

export interface CallbackJSON {
  programId: string;
  accounts: Array<types.AccountMetaBorshJSON>;
  ixData: Array<number>;
}

export class Callback {
  readonly programId: PublicKey;
  readonly accounts: Array<types.AccountMetaBorsh>;
  readonly ixData: Uint8Array;

  constructor(fields: CallbackFields) {
    this.programId = fields.programId;
    this.accounts = fields.accounts.map(
      item => new types.AccountMetaBorsh({ ...item })
    );
    this.ixData = fields.ixData;
  }

  static layout(property?: string) {
    return borsh.struct(
      [
        borsh.publicKey('programId'),
        borsh.vec(types.AccountMetaBorsh.layout(), 'accounts'),
        borsh.vecU8('ixData'),
      ],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new Callback({
      programId: obj.programId,
      accounts: obj.accounts.map(
        (
          item: any /* eslint-disable-line @typescript-eslint/no-explicit-any */
        ) => types.AccountMetaBorsh.fromDecoded(item)
      ),
      ixData: new Uint8Array(
        obj.ixData.buffer,
        obj.ixData.byteOffset,
        obj.ixData.length
      ),
    });
  }

  static toEncodable(fields: CallbackFields) {
    return {
      programId: fields.programId,
      accounts: fields.accounts.map(item =>
        types.AccountMetaBorsh.toEncodable(item)
      ),
      ixData: Buffer.from(
        fields.ixData.buffer,
        fields.ixData.byteOffset,
        fields.ixData.length
      ),
    };
  }

  toJSON(): CallbackJSON {
    return {
      programId: this.programId.toString(),
      accounts: this.accounts.map(item => item.toJSON()),
      ixData: Array.from(this.ixData.values()),
    };
  }

  static fromJSON(obj: CallbackJSON): Callback {
    return new Callback({
      programId: new PublicKey(obj.programId),
      accounts: obj.accounts.map(item => types.AccountMetaBorsh.fromJSON(item)),
      ixData: Uint8Array.from(obj.ixData),
    });
  }

  toEncodable() {
    return Callback.toEncodable(this);
  }
}
