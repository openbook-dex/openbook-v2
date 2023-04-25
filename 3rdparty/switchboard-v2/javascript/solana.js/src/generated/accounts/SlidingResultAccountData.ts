import { SwitchboardProgram } from '../../program';
import { PublicKey, Connection } from '@solana/web3.js';
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars

export interface SlidingResultAccountDataFields {
  data: Array<types.SlidingWindowElementFields>;
  bump: number;
  ebuf: Array<number>;
}

export interface SlidingResultAccountDataJSON {
  data: Array<types.SlidingWindowElementJSON>;
  bump: number;
  ebuf: Array<number>;
}

export class SlidingResultAccountData {
  readonly data: Array<types.SlidingWindowElement>;
  readonly bump: number;
  readonly ebuf: Array<number>;

  static readonly discriminator = Buffer.from([
    91, 4, 83, 187, 102, 216, 153, 254,
  ]);

  static readonly layout = borsh.struct([
    borsh.array(types.SlidingWindowElement.layout(), 16, 'data'),
    borsh.u8('bump'),
    borsh.array(borsh.u8(), 512, 'ebuf'),
  ]);

  constructor(fields: SlidingResultAccountDataFields) {
    this.data = fields.data.map(
      item => new types.SlidingWindowElement({ ...item })
    );
    this.bump = fields.bump;
    this.ebuf = fields.ebuf;
  }

  static async fetch(
    program: SwitchboardProgram,
    address: PublicKey
  ): Promise<SlidingResultAccountData | null> {
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
  ): Promise<Array<SlidingResultAccountData | null>> {
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

  static decode(data: Buffer): SlidingResultAccountData {
    if (!data.slice(0, 8).equals(SlidingResultAccountData.discriminator)) {
      throw new Error('invalid account discriminator');
    }

    const dec = SlidingResultAccountData.layout.decode(data.slice(8));

    return new SlidingResultAccountData({
      data: dec.data.map(
        (
          item: any /* eslint-disable-line @typescript-eslint/no-explicit-any */
        ) => types.SlidingWindowElement.fromDecoded(item)
      ),
      bump: dec.bump,
      ebuf: dec.ebuf,
    });
  }

  toJSON(): SlidingResultAccountDataJSON {
    return {
      data: this.data.map(item => item.toJSON()),
      bump: this.bump,
      ebuf: this.ebuf,
    };
  }

  static fromJSON(obj: SlidingResultAccountDataJSON): SlidingResultAccountData {
    return new SlidingResultAccountData({
      data: obj.data.map(item => types.SlidingWindowElement.fromJSON(item)),
      bump: obj.bump,
      ebuf: obj.ebuf,
    });
  }
}
