import { SwitchboardProgram } from '../../program';
import { PublicKey, Connection } from '@solana/web3.js';
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars

export interface RealmSpawnRecordAccountDataFields {
  ebuf: Array<number>;
}

export interface RealmSpawnRecordAccountDataJSON {
  ebuf: Array<number>;
}

export class RealmSpawnRecordAccountData {
  readonly ebuf: Array<number>;

  static readonly discriminator = Buffer.from([
    229, 116, 99, 2, 145, 96, 5, 95,
  ]);

  static readonly layout = borsh.struct([borsh.array(borsh.u8(), 256, 'ebuf')]);

  constructor(fields: RealmSpawnRecordAccountDataFields) {
    this.ebuf = fields.ebuf;
  }

  static async fetch(
    program: SwitchboardProgram,
    address: PublicKey
  ): Promise<RealmSpawnRecordAccountData | null> {
    const info = await program.connection.getAccountInfo(address);

    if (info === null) {
      return null;
    }
    if (!info.owner.equals(program.programId)) {
      throw new Error("account doesn't belong to this program");
    }

    return this.decode(info.data);
  }

  static async fetchMultiple(
    program: SwitchboardProgram,
    addresses: PublicKey[]
  ): Promise<Array<RealmSpawnRecordAccountData | null>> {
    const infos = await program.connection.getMultipleAccountsInfo(addresses);

    return infos.map(info => {
      if (info === null) {
        return null;
      }
      if (!info.owner.equals(program.programId)) {
        throw new Error("account doesn't belong to this program");
      }

      return this.decode(info.data);
    });
  }

  static decode(data: Buffer): RealmSpawnRecordAccountData {
    if (!data.slice(0, 8).equals(RealmSpawnRecordAccountData.discriminator)) {
      throw new Error('invalid account discriminator');
    }

    const dec = RealmSpawnRecordAccountData.layout.decode(data.slice(8));

    return new RealmSpawnRecordAccountData({
      ebuf: dec.ebuf,
    });
  }

  toJSON(): RealmSpawnRecordAccountDataJSON {
    return {
      ebuf: this.ebuf,
    };
  }

  static fromJSON(
    obj: RealmSpawnRecordAccountDataJSON
  ): RealmSpawnRecordAccountData {
    return new RealmSpawnRecordAccountData({
      ebuf: obj.ebuf,
    });
  }
}
