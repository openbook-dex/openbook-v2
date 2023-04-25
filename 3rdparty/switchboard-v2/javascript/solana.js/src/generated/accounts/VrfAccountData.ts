import { SwitchboardProgram } from '../../program';
import { PublicKey, Connection } from '@solana/web3.js';
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars

export interface VrfAccountDataFields {
  /** The current status of the VRF account. */
  status: types.VrfStatusKind;
  /** Incremental counter for tracking VRF rounds. */
  counter: BN;
  /** On-chain account delegated for making account changes. */
  authority: PublicKey;
  /** The OracleQueueAccountData that is assigned to fulfill VRF update request. */
  oracleQueue: PublicKey;
  /** The token account used to hold funds for VRF update request. */
  escrow: PublicKey;
  /** The callback that is invoked when an update request is successfully verified. */
  callback: types.CallbackZCFields;
  /** The number of oracles assigned to a VRF update request. */
  batchSize: number;
  /** Struct containing the intermediate state between VRF crank actions. */
  builders: Array<types.VrfBuilderFields>;
  /** The number of builders. */
  buildersLen: number;
  testMode: boolean;
  /** Oracle results from the current round of update request that has not been accepted as valid yet */
  currentRound: types.VrfRoundFields;
  /** Reserved for future info. */
  ebuf: Array<number>;
}

export interface VrfAccountDataJSON {
  /** The current status of the VRF account. */
  status: types.VrfStatusJSON;
  /** Incremental counter for tracking VRF rounds. */
  counter: string;
  /** On-chain account delegated for making account changes. */
  authority: string;
  /** The OracleQueueAccountData that is assigned to fulfill VRF update request. */
  oracleQueue: string;
  /** The token account used to hold funds for VRF update request. */
  escrow: string;
  /** The callback that is invoked when an update request is successfully verified. */
  callback: types.CallbackZCJSON;
  /** The number of oracles assigned to a VRF update request. */
  batchSize: number;
  /** Struct containing the intermediate state between VRF crank actions. */
  builders: Array<types.VrfBuilderJSON>;
  /** The number of builders. */
  buildersLen: number;
  testMode: boolean;
  /** Oracle results from the current round of update request that has not been accepted as valid yet */
  currentRound: types.VrfRoundJSON;
  /** Reserved for future info. */
  ebuf: Array<number>;
}

export class VrfAccountData {
  /** The current status of the VRF account. */
  readonly status: types.VrfStatusKind;
  /** Incremental counter for tracking VRF rounds. */
  readonly counter: BN;
  /** On-chain account delegated for making account changes. */
  readonly authority: PublicKey;
  /** The OracleQueueAccountData that is assigned to fulfill VRF update request. */
  readonly oracleQueue: PublicKey;
  /** The token account used to hold funds for VRF update request. */
  readonly escrow: PublicKey;
  /** The callback that is invoked when an update request is successfully verified. */
  readonly callback: types.CallbackZC;
  /** The number of oracles assigned to a VRF update request. */
  readonly batchSize: number;
  /** Struct containing the intermediate state between VRF crank actions. */
  readonly builders: Array<types.VrfBuilder>;
  /** The number of builders. */
  readonly buildersLen: number;
  readonly testMode: boolean;
  /** Oracle results from the current round of update request that has not been accepted as valid yet */
  readonly currentRound: types.VrfRound;
  /** Reserved for future info. */
  readonly ebuf: Array<number>;

  static readonly discriminator = Buffer.from([
    101, 35, 62, 239, 103, 151, 6, 18,
  ]);

  static readonly layout = borsh.struct([
    types.VrfStatus.layout('status'),
    borsh.u128('counter'),
    borsh.publicKey('authority'),
    borsh.publicKey('oracleQueue'),
    borsh.publicKey('escrow'),
    types.CallbackZC.layout('callback'),
    borsh.u32('batchSize'),
    borsh.array(types.VrfBuilder.layout(), 8, 'builders'),
    borsh.u32('buildersLen'),
    borsh.bool('testMode'),
    types.VrfRound.layout('currentRound'),
    borsh.array(borsh.u8(), 1024, 'ebuf'),
  ]);

  constructor(fields: VrfAccountDataFields) {
    this.status = fields.status;
    this.counter = fields.counter;
    this.authority = fields.authority;
    this.oracleQueue = fields.oracleQueue;
    this.escrow = fields.escrow;
    this.callback = new types.CallbackZC({ ...fields.callback });
    this.batchSize = fields.batchSize;
    this.builders = fields.builders.map(
      item => new types.VrfBuilder({ ...item })
    );
    this.buildersLen = fields.buildersLen;
    this.testMode = fields.testMode;
    this.currentRound = new types.VrfRound({ ...fields.currentRound });
    this.ebuf = fields.ebuf;
  }

  static async fetch(
    program: SwitchboardProgram,
    address: PublicKey
  ): Promise<VrfAccountData | null> {
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
  ): Promise<Array<VrfAccountData | null>> {
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

  static decode(data: Buffer): VrfAccountData {
    if (!data.slice(0, 8).equals(VrfAccountData.discriminator)) {
      throw new Error('invalid account discriminator');
    }

    const dec = VrfAccountData.layout.decode(data.slice(8));

    return new VrfAccountData({
      status: types.VrfStatus.fromDecoded(dec.status),
      counter: dec.counter,
      authority: dec.authority,
      oracleQueue: dec.oracleQueue,
      escrow: dec.escrow,
      callback: types.CallbackZC.fromDecoded(dec.callback),
      batchSize: dec.batchSize,
      builders: dec.builders.map(
        (
          item: any /* eslint-disable-line @typescript-eslint/no-explicit-any */
        ) => types.VrfBuilder.fromDecoded(item)
      ),
      buildersLen: dec.buildersLen,
      testMode: dec.testMode,
      currentRound: types.VrfRound.fromDecoded(dec.currentRound),
      ebuf: dec.ebuf,
    });
  }

  toJSON(): VrfAccountDataJSON {
    return {
      status: this.status.toJSON(),
      counter: this.counter.toString(),
      authority: this.authority.toString(),
      oracleQueue: this.oracleQueue.toString(),
      escrow: this.escrow.toString(),
      callback: this.callback.toJSON(),
      batchSize: this.batchSize,
      builders: this.builders.map(item => item.toJSON()),
      buildersLen: this.buildersLen,
      testMode: this.testMode,
      currentRound: this.currentRound.toJSON(),
      ebuf: this.ebuf,
    };
  }

  static fromJSON(obj: VrfAccountDataJSON): VrfAccountData {
    return new VrfAccountData({
      status: types.VrfStatus.fromJSON(obj.status),
      counter: new BN(obj.counter),
      authority: new PublicKey(obj.authority),
      oracleQueue: new PublicKey(obj.oracleQueue),
      escrow: new PublicKey(obj.escrow),
      callback: types.CallbackZC.fromJSON(obj.callback),
      batchSize: obj.batchSize,
      builders: obj.builders.map(item => types.VrfBuilder.fromJSON(item)),
      buildersLen: obj.buildersLen,
      testMode: obj.testMode,
      currentRound: types.VrfRound.fromJSON(obj.currentRound),
      ebuf: obj.ebuf,
    });
  }
}
