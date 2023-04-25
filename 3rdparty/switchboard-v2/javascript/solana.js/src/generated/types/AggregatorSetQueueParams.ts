import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface AggregatorSetQueueParamsFields {}

export interface AggregatorSetQueueParamsJSON {}

export class AggregatorSetQueueParams {
  constructor(fields: AggregatorSetQueueParamsFields) {}

  static layout(property?: string) {
    return borsh.struct([], property);
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new AggregatorSetQueueParams({});
  }

  static toEncodable(fields: AggregatorSetQueueParamsFields) {
    return {};
  }

  toJSON(): AggregatorSetQueueParamsJSON {
    return {};
  }

  static fromJSON(obj: AggregatorSetQueueParamsJSON): AggregatorSetQueueParams {
    return new AggregatorSetQueueParams({});
  }

  toEncodable() {
    return AggregatorSetQueueParams.toEncodable(this);
  }
}
