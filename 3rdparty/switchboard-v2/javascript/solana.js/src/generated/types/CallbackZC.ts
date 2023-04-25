import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface CallbackZCFields {
  /** The program ID of the callback program being invoked. */
  programId: PublicKey;
  /** The accounts being used in the callback instruction. */
  accounts: Array<types.AccountMetaZCFields>;
  /** The number of accounts used in the callback */
  accountsLen: number;
  /** The serialized instruction data. */
  ixData: Array<number>;
  /** The number of serialized bytes in the instruction data. */
  ixDataLen: number;
}

export interface CallbackZCJSON {
  /** The program ID of the callback program being invoked. */
  programId: string;
  /** The accounts being used in the callback instruction. */
  accounts: Array<types.AccountMetaZCJSON>;
  /** The number of accounts used in the callback */
  accountsLen: number;
  /** The serialized instruction data. */
  ixData: Array<number>;
  /** The number of serialized bytes in the instruction data. */
  ixDataLen: number;
}

export class CallbackZC {
  /** The program ID of the callback program being invoked. */
  readonly programId: PublicKey;
  /** The accounts being used in the callback instruction. */
  readonly accounts: Array<types.AccountMetaZC>;
  /** The number of accounts used in the callback */
  readonly accountsLen: number;
  /** The serialized instruction data. */
  readonly ixData: Array<number>;
  /** The number of serialized bytes in the instruction data. */
  readonly ixDataLen: number;

  constructor(fields: CallbackZCFields) {
    this.programId = fields.programId;
    this.accounts = fields.accounts.map(
      item => new types.AccountMetaZC({ ...item })
    );
    this.accountsLen = fields.accountsLen;
    this.ixData = fields.ixData;
    this.ixDataLen = fields.ixDataLen;
  }

  static layout(property?: string) {
    return borsh.struct(
      [
        borsh.publicKey('programId'),
        borsh.array(types.AccountMetaZC.layout(), 32, 'accounts'),
        borsh.u32('accountsLen'),
        borsh.array(borsh.u8(), 1024, 'ixData'),
        borsh.u32('ixDataLen'),
      ],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new CallbackZC({
      programId: obj.programId,
      accounts: obj.accounts.map(
        (
          item: any /* eslint-disable-line @typescript-eslint/no-explicit-any */
        ) => types.AccountMetaZC.fromDecoded(item)
      ),
      accountsLen: obj.accountsLen,
      ixData: obj.ixData,
      ixDataLen: obj.ixDataLen,
    });
  }

  static toEncodable(fields: CallbackZCFields) {
    return {
      programId: fields.programId,
      accounts: fields.accounts.map(item =>
        types.AccountMetaZC.toEncodable(item)
      ),
      accountsLen: fields.accountsLen,
      ixData: fields.ixData,
      ixDataLen: fields.ixDataLen,
    };
  }

  toJSON(): CallbackZCJSON {
    return {
      programId: this.programId.toString(),
      accounts: this.accounts.map(item => item.toJSON()),
      accountsLen: this.accountsLen,
      ixData: this.ixData,
      ixDataLen: this.ixDataLen,
    };
  }

  static fromJSON(obj: CallbackZCJSON): CallbackZC {
    return new CallbackZC({
      programId: new PublicKey(obj.programId),
      accounts: obj.accounts.map(item => types.AccountMetaZC.fromJSON(item)),
      accountsLen: obj.accountsLen,
      ixData: obj.ixData,
      ixDataLen: obj.ixDataLen,
    });
  }

  toEncodable() {
    return CallbackZC.toEncodable(this);
  }
}
