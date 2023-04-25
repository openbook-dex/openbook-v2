import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface AggregatorRoundFields {
  /**
   * Maintains the number of successful responses received from nodes.
   * Nodes can submit one successful response per round.
   */
  numSuccess: number;
  /** Number of error responses. */
  numError: number;
  /** Whether an update request round has ended. */
  isClosed: boolean;
  /** Maintains the `solana_program::clock::Slot` that the round was opened at. */
  roundOpenSlot: BN;
  /** Maintains the `solana_program::clock::UnixTimestamp;` the round was opened at. */
  roundOpenTimestamp: BN;
  /** Maintains the current median of all successful round responses. */
  result: types.SwitchboardDecimalFields;
  /** Standard deviation of the accepted results in the round. */
  stdDeviation: types.SwitchboardDecimalFields;
  /** Maintains the minimum node response this round. */
  minResponse: types.SwitchboardDecimalFields;
  /** Maintains the maximum node response this round. */
  maxResponse: types.SwitchboardDecimalFields;
  /** Pubkeys of the oracles fulfilling this round. */
  oraclePubkeysData: Array<PublicKey>;
  /** Represents all successful node responses this round. `NaN` if empty. */
  mediansData: Array<types.SwitchboardDecimalFields>;
  /** Current rewards/slashes oracles have received this round. */
  currentPayout: Array<BN>;
  /** Keep track of which responses are fulfilled here. */
  mediansFulfilled: Array<boolean>;
  /** Keeps track of which errors are fulfilled here. */
  errorsFulfilled: Array<boolean>;
}

export interface AggregatorRoundJSON {
  /**
   * Maintains the number of successful responses received from nodes.
   * Nodes can submit one successful response per round.
   */
  numSuccess: number;
  /** Number of error responses. */
  numError: number;
  /** Whether an update request round has ended. */
  isClosed: boolean;
  /** Maintains the `solana_program::clock::Slot` that the round was opened at. */
  roundOpenSlot: string;
  /** Maintains the `solana_program::clock::UnixTimestamp;` the round was opened at. */
  roundOpenTimestamp: string;
  /** Maintains the current median of all successful round responses. */
  result: types.SwitchboardDecimalJSON;
  /** Standard deviation of the accepted results in the round. */
  stdDeviation: types.SwitchboardDecimalJSON;
  /** Maintains the minimum node response this round. */
  minResponse: types.SwitchboardDecimalJSON;
  /** Maintains the maximum node response this round. */
  maxResponse: types.SwitchboardDecimalJSON;
  /** Pubkeys of the oracles fulfilling this round. */
  oraclePubkeysData: Array<string>;
  /** Represents all successful node responses this round. `NaN` if empty. */
  mediansData: Array<types.SwitchboardDecimalJSON>;
  /** Current rewards/slashes oracles have received this round. */
  currentPayout: Array<string>;
  /** Keep track of which responses are fulfilled here. */
  mediansFulfilled: Array<boolean>;
  /** Keeps track of which errors are fulfilled here. */
  errorsFulfilled: Array<boolean>;
}

export class AggregatorRound {
  /**
   * Maintains the number of successful responses received from nodes.
   * Nodes can submit one successful response per round.
   */
  readonly numSuccess: number;
  /** Number of error responses. */
  readonly numError: number;
  /** Whether an update request round has ended. */
  readonly isClosed: boolean;
  /** Maintains the `solana_program::clock::Slot` that the round was opened at. */
  readonly roundOpenSlot: BN;
  /** Maintains the `solana_program::clock::UnixTimestamp;` the round was opened at. */
  readonly roundOpenTimestamp: BN;
  /** Maintains the current median of all successful round responses. */
  readonly result: types.SwitchboardDecimal;
  /** Standard deviation of the accepted results in the round. */
  readonly stdDeviation: types.SwitchboardDecimal;
  /** Maintains the minimum node response this round. */
  readonly minResponse: types.SwitchboardDecimal;
  /** Maintains the maximum node response this round. */
  readonly maxResponse: types.SwitchboardDecimal;
  /** Pubkeys of the oracles fulfilling this round. */
  readonly oraclePubkeysData: Array<PublicKey>;
  /** Represents all successful node responses this round. `NaN` if empty. */
  readonly mediansData: Array<types.SwitchboardDecimal>;
  /** Current rewards/slashes oracles have received this round. */
  readonly currentPayout: Array<BN>;
  /** Keep track of which responses are fulfilled here. */
  readonly mediansFulfilled: Array<boolean>;
  /** Keeps track of which errors are fulfilled here. */
  readonly errorsFulfilled: Array<boolean>;

  constructor(fields: AggregatorRoundFields) {
    this.numSuccess = fields.numSuccess;
    this.numError = fields.numError;
    this.isClosed = fields.isClosed;
    this.roundOpenSlot = fields.roundOpenSlot;
    this.roundOpenTimestamp = fields.roundOpenTimestamp;
    this.result = new types.SwitchboardDecimal({ ...fields.result });
    this.stdDeviation = new types.SwitchboardDecimal({
      ...fields.stdDeviation,
    });
    this.minResponse = new types.SwitchboardDecimal({ ...fields.minResponse });
    this.maxResponse = new types.SwitchboardDecimal({ ...fields.maxResponse });
    this.oraclePubkeysData = fields.oraclePubkeysData;
    this.mediansData = fields.mediansData.map(
      item => new types.SwitchboardDecimal({ ...item })
    );
    this.currentPayout = fields.currentPayout;
    this.mediansFulfilled = fields.mediansFulfilled;
    this.errorsFulfilled = fields.errorsFulfilled;
  }

  static layout(property?: string) {
    return borsh.struct(
      [
        borsh.u32('numSuccess'),
        borsh.u32('numError'),
        borsh.bool('isClosed'),
        borsh.u64('roundOpenSlot'),
        borsh.i64('roundOpenTimestamp'),
        types.SwitchboardDecimal.layout('result'),
        types.SwitchboardDecimal.layout('stdDeviation'),
        types.SwitchboardDecimal.layout('minResponse'),
        types.SwitchboardDecimal.layout('maxResponse'),
        borsh.array(borsh.publicKey(), 16, 'oraclePubkeysData'),
        borsh.array(types.SwitchboardDecimal.layout(), 16, 'mediansData'),
        borsh.array(borsh.i64(), 16, 'currentPayout'),
        borsh.array(borsh.bool(), 16, 'mediansFulfilled'),
        borsh.array(borsh.bool(), 16, 'errorsFulfilled'),
      ],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new AggregatorRound({
      numSuccess: obj.numSuccess,
      numError: obj.numError,
      isClosed: obj.isClosed,
      roundOpenSlot: obj.roundOpenSlot,
      roundOpenTimestamp: obj.roundOpenTimestamp,
      result: types.SwitchboardDecimal.fromDecoded(obj.result),
      stdDeviation: types.SwitchboardDecimal.fromDecoded(obj.stdDeviation),
      minResponse: types.SwitchboardDecimal.fromDecoded(obj.minResponse),
      maxResponse: types.SwitchboardDecimal.fromDecoded(obj.maxResponse),
      oraclePubkeysData: obj.oraclePubkeysData,
      mediansData: obj.mediansData.map(
        (
          item: any /* eslint-disable-line @typescript-eslint/no-explicit-any */
        ) => types.SwitchboardDecimal.fromDecoded(item)
      ),
      currentPayout: obj.currentPayout,
      mediansFulfilled: obj.mediansFulfilled,
      errorsFulfilled: obj.errorsFulfilled,
    });
  }

  static toEncodable(fields: AggregatorRoundFields) {
    return {
      numSuccess: fields.numSuccess,
      numError: fields.numError,
      isClosed: fields.isClosed,
      roundOpenSlot: fields.roundOpenSlot,
      roundOpenTimestamp: fields.roundOpenTimestamp,
      result: types.SwitchboardDecimal.toEncodable(fields.result),
      stdDeviation: types.SwitchboardDecimal.toEncodable(fields.stdDeviation),
      minResponse: types.SwitchboardDecimal.toEncodable(fields.minResponse),
      maxResponse: types.SwitchboardDecimal.toEncodable(fields.maxResponse),
      oraclePubkeysData: fields.oraclePubkeysData,
      mediansData: fields.mediansData.map(item =>
        types.SwitchboardDecimal.toEncodable(item)
      ),
      currentPayout: fields.currentPayout,
      mediansFulfilled: fields.mediansFulfilled,
      errorsFulfilled: fields.errorsFulfilled,
    };
  }

  toJSON(): AggregatorRoundJSON {
    return {
      numSuccess: this.numSuccess,
      numError: this.numError,
      isClosed: this.isClosed,
      roundOpenSlot: this.roundOpenSlot.toString(),
      roundOpenTimestamp: this.roundOpenTimestamp.toString(),
      result: this.result.toJSON(),
      stdDeviation: this.stdDeviation.toJSON(),
      minResponse: this.minResponse.toJSON(),
      maxResponse: this.maxResponse.toJSON(),
      oraclePubkeysData: this.oraclePubkeysData.map(item => item.toString()),
      mediansData: this.mediansData.map(item => item.toJSON()),
      currentPayout: this.currentPayout.map(item => item.toString()),
      mediansFulfilled: this.mediansFulfilled,
      errorsFulfilled: this.errorsFulfilled,
    };
  }

  static fromJSON(obj: AggregatorRoundJSON): AggregatorRound {
    return new AggregatorRound({
      numSuccess: obj.numSuccess,
      numError: obj.numError,
      isClosed: obj.isClosed,
      roundOpenSlot: new BN(obj.roundOpenSlot),
      roundOpenTimestamp: new BN(obj.roundOpenTimestamp),
      result: types.SwitchboardDecimal.fromJSON(obj.result),
      stdDeviation: types.SwitchboardDecimal.fromJSON(obj.stdDeviation),
      minResponse: types.SwitchboardDecimal.fromJSON(obj.minResponse),
      maxResponse: types.SwitchboardDecimal.fromJSON(obj.maxResponse),
      oraclePubkeysData: obj.oraclePubkeysData.map(item => new PublicKey(item)),
      mediansData: obj.mediansData.map(item =>
        types.SwitchboardDecimal.fromJSON(item)
      ),
      currentPayout: obj.currentPayout.map(item => new BN(item)),
      mediansFulfilled: obj.mediansFulfilled,
      errorsFulfilled: obj.errorsFulfilled,
    });
  }

  toEncodable() {
    return AggregatorRound.toEncodable(this);
  }
}
