import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface JobSetDataParamsFields {
  data: Uint8Array;
  chunkIdx: number;
}

export interface JobSetDataParamsJSON {
  data: Array<number>;
  chunkIdx: number;
}

export class JobSetDataParams {
  readonly data: Uint8Array;
  readonly chunkIdx: number;

  constructor(fields: JobSetDataParamsFields) {
    this.data = fields.data;
    this.chunkIdx = fields.chunkIdx;
  }

  static layout(property?: string) {
    return borsh.struct([borsh.vecU8('data'), borsh.u8('chunkIdx')], property);
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new JobSetDataParams({
      data: new Uint8Array(
        obj.data.buffer,
        obj.data.byteOffset,
        obj.data.length
      ),
      chunkIdx: obj.chunkIdx,
    });
  }

  static toEncodable(fields: JobSetDataParamsFields) {
    return {
      data: Buffer.from(
        fields.data.buffer,
        fields.data.byteOffset,
        fields.data.length
      ),
      chunkIdx: fields.chunkIdx,
    };
  }

  toJSON(): JobSetDataParamsJSON {
    return {
      data: Array.from(this.data.values()),
      chunkIdx: this.chunkIdx,
    };
  }

  static fromJSON(obj: JobSetDataParamsJSON): JobSetDataParams {
    return new JobSetDataParams({
      data: Uint8Array.from(obj.data),
      chunkIdx: obj.chunkIdx,
    });
  }

  toEncodable() {
    return JobSetDataParams.toEncodable(this);
  }
}
