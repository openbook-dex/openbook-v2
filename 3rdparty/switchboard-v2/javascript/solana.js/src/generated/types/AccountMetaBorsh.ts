import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface AccountMetaBorshFields {
  pubkey: PublicKey;
  isSigner: boolean;
  isWritable: boolean;
}

export interface AccountMetaBorshJSON {
  pubkey: string;
  isSigner: boolean;
  isWritable: boolean;
}

export class AccountMetaBorsh {
  readonly pubkey: PublicKey;
  readonly isSigner: boolean;
  readonly isWritable: boolean;

  constructor(fields: AccountMetaBorshFields) {
    this.pubkey = fields.pubkey;
    this.isSigner = fields.isSigner;
    this.isWritable = fields.isWritable;
  }

  static layout(property?: string) {
    return borsh.struct(
      [
        borsh.publicKey('pubkey'),
        borsh.bool('isSigner'),
        borsh.bool('isWritable'),
      ],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new AccountMetaBorsh({
      pubkey: obj.pubkey,
      isSigner: obj.isSigner,
      isWritable: obj.isWritable,
    });
  }

  static toEncodable(fields: AccountMetaBorshFields) {
    return {
      pubkey: fields.pubkey,
      isSigner: fields.isSigner,
      isWritable: fields.isWritable,
    };
  }

  toJSON(): AccountMetaBorshJSON {
    return {
      pubkey: this.pubkey.toString(),
      isSigner: this.isSigner,
      isWritable: this.isWritable,
    };
  }

  static fromJSON(obj: AccountMetaBorshJSON): AccountMetaBorsh {
    return new AccountMetaBorsh({
      pubkey: new PublicKey(obj.pubkey),
      isSigner: obj.isSigner,
      isWritable: obj.isWritable,
    });
  }

  toEncodable() {
    return AccountMetaBorsh.toEncodable(this);
  }
}
