import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface VrfBuilderFields {
  /** The OracleAccountData that is producing the randomness. */
  producer: PublicKey;
  /** The current status of the VRF verification. */
  status: types.VrfStatusKind;
  /** The VRF proof sourced from the producer. */
  reprProof: Array<number>;
  proof: types.EcvrfProofZCFields;
  yPoint: PublicKey;
  stage: number;
  stage1Out: types.EcvrfIntermediateFields;
  r1: types.EdwardsPointZCFields;
  r2: types.EdwardsPointZCFields;
  stage3Out: types.EcvrfIntermediateFields;
  hPoint: types.EdwardsPointZCFields;
  sReduced: types.ScalarFields;
  yPointBuilder: Array<types.FieldElementZCFields>;
  yRistrettoPoint: types.EdwardsPointZCFields;
  mulRound: number;
  hashPointsRound: number;
  mulTmp1: types.CompletedPointZCFields;
  uPoint1: types.EdwardsPointZCFields;
  uPoint2: types.EdwardsPointZCFields;
  vPoint1: types.EdwardsPointZCFields;
  vPoint2: types.EdwardsPointZCFields;
  uPoint: types.EdwardsPointZCFields;
  vPoint: types.EdwardsPointZCFields;
  u1: types.FieldElementZCFields;
  u2: types.FieldElementZCFields;
  invertee: types.FieldElementZCFields;
  y: types.FieldElementZCFields;
  z: types.FieldElementZCFields;
  p1Bytes: Array<number>;
  p2Bytes: Array<number>;
  p3Bytes: Array<number>;
  p4Bytes: Array<number>;
  cPrimeHashbuf: Array<number>;
  m1: types.FieldElementZCFields;
  m2: types.FieldElementZCFields;
  /** The number of transactions remaining to verify the VRF proof. */
  txRemaining: number;
  /** Whether the VRF proof has been verified on-chain. */
  verified: boolean;
  /** The VRF proof verification result. Will be zeroized if still awaiting fulfillment. */
  result: Array<number>;
}

export interface VrfBuilderJSON {
  /** The OracleAccountData that is producing the randomness. */
  producer: string;
  /** The current status of the VRF verification. */
  status: types.VrfStatusJSON;
  /** The VRF proof sourced from the producer. */
  reprProof: Array<number>;
  proof: types.EcvrfProofZCJSON;
  yPoint: string;
  stage: number;
  stage1Out: types.EcvrfIntermediateJSON;
  r1: types.EdwardsPointZCJSON;
  r2: types.EdwardsPointZCJSON;
  stage3Out: types.EcvrfIntermediateJSON;
  hPoint: types.EdwardsPointZCJSON;
  sReduced: types.ScalarJSON;
  yPointBuilder: Array<types.FieldElementZCJSON>;
  yRistrettoPoint: types.EdwardsPointZCJSON;
  mulRound: number;
  hashPointsRound: number;
  mulTmp1: types.CompletedPointZCJSON;
  uPoint1: types.EdwardsPointZCJSON;
  uPoint2: types.EdwardsPointZCJSON;
  vPoint1: types.EdwardsPointZCJSON;
  vPoint2: types.EdwardsPointZCJSON;
  uPoint: types.EdwardsPointZCJSON;
  vPoint: types.EdwardsPointZCJSON;
  u1: types.FieldElementZCJSON;
  u2: types.FieldElementZCJSON;
  invertee: types.FieldElementZCJSON;
  y: types.FieldElementZCJSON;
  z: types.FieldElementZCJSON;
  p1Bytes: Array<number>;
  p2Bytes: Array<number>;
  p3Bytes: Array<number>;
  p4Bytes: Array<number>;
  cPrimeHashbuf: Array<number>;
  m1: types.FieldElementZCJSON;
  m2: types.FieldElementZCJSON;
  /** The number of transactions remaining to verify the VRF proof. */
  txRemaining: number;
  /** Whether the VRF proof has been verified on-chain. */
  verified: boolean;
  /** The VRF proof verification result. Will be zeroized if still awaiting fulfillment. */
  result: Array<number>;
}

export class VrfBuilder {
  /** The OracleAccountData that is producing the randomness. */
  readonly producer: PublicKey;
  /** The current status of the VRF verification. */
  readonly status: types.VrfStatusKind;
  /** The VRF proof sourced from the producer. */
  readonly reprProof: Array<number>;
  readonly proof: types.EcvrfProofZC;
  readonly yPoint: PublicKey;
  readonly stage: number;
  readonly stage1Out: types.EcvrfIntermediate;
  readonly r1: types.EdwardsPointZC;
  readonly r2: types.EdwardsPointZC;
  readonly stage3Out: types.EcvrfIntermediate;
  readonly hPoint: types.EdwardsPointZC;
  readonly sReduced: types.Scalar;
  readonly yPointBuilder: Array<types.FieldElementZC>;
  readonly yRistrettoPoint: types.EdwardsPointZC;
  readonly mulRound: number;
  readonly hashPointsRound: number;
  readonly mulTmp1: types.CompletedPointZC;
  readonly uPoint1: types.EdwardsPointZC;
  readonly uPoint2: types.EdwardsPointZC;
  readonly vPoint1: types.EdwardsPointZC;
  readonly vPoint2: types.EdwardsPointZC;
  readonly uPoint: types.EdwardsPointZC;
  readonly vPoint: types.EdwardsPointZC;
  readonly u1: types.FieldElementZC;
  readonly u2: types.FieldElementZC;
  readonly invertee: types.FieldElementZC;
  readonly y: types.FieldElementZC;
  readonly z: types.FieldElementZC;
  readonly p1Bytes: Array<number>;
  readonly p2Bytes: Array<number>;
  readonly p3Bytes: Array<number>;
  readonly p4Bytes: Array<number>;
  readonly cPrimeHashbuf: Array<number>;
  readonly m1: types.FieldElementZC;
  readonly m2: types.FieldElementZC;
  /** The number of transactions remaining to verify the VRF proof. */
  readonly txRemaining: number;
  /** Whether the VRF proof has been verified on-chain. */
  readonly verified: boolean;
  /** The VRF proof verification result. Will be zeroized if still awaiting fulfillment. */
  readonly result: Array<number>;

  constructor(fields: VrfBuilderFields) {
    this.producer = fields.producer;
    this.status = fields.status;
    this.reprProof = fields.reprProof;
    this.proof = new types.EcvrfProofZC({ ...fields.proof });
    this.yPoint = fields.yPoint;
    this.stage = fields.stage;
    this.stage1Out = new types.EcvrfIntermediate({ ...fields.stage1Out });
    this.r1 = new types.EdwardsPointZC({ ...fields.r1 });
    this.r2 = new types.EdwardsPointZC({ ...fields.r2 });
    this.stage3Out = new types.EcvrfIntermediate({ ...fields.stage3Out });
    this.hPoint = new types.EdwardsPointZC({ ...fields.hPoint });
    this.sReduced = new types.Scalar({ ...fields.sReduced });
    this.yPointBuilder = fields.yPointBuilder.map(
      item => new types.FieldElementZC({ ...item })
    );
    this.yRistrettoPoint = new types.EdwardsPointZC({
      ...fields.yRistrettoPoint,
    });
    this.mulRound = fields.mulRound;
    this.hashPointsRound = fields.hashPointsRound;
    this.mulTmp1 = new types.CompletedPointZC({ ...fields.mulTmp1 });
    this.uPoint1 = new types.EdwardsPointZC({ ...fields.uPoint1 });
    this.uPoint2 = new types.EdwardsPointZC({ ...fields.uPoint2 });
    this.vPoint1 = new types.EdwardsPointZC({ ...fields.vPoint1 });
    this.vPoint2 = new types.EdwardsPointZC({ ...fields.vPoint2 });
    this.uPoint = new types.EdwardsPointZC({ ...fields.uPoint });
    this.vPoint = new types.EdwardsPointZC({ ...fields.vPoint });
    this.u1 = new types.FieldElementZC({ ...fields.u1 });
    this.u2 = new types.FieldElementZC({ ...fields.u2 });
    this.invertee = new types.FieldElementZC({ ...fields.invertee });
    this.y = new types.FieldElementZC({ ...fields.y });
    this.z = new types.FieldElementZC({ ...fields.z });
    this.p1Bytes = fields.p1Bytes;
    this.p2Bytes = fields.p2Bytes;
    this.p3Bytes = fields.p3Bytes;
    this.p4Bytes = fields.p4Bytes;
    this.cPrimeHashbuf = fields.cPrimeHashbuf;
    this.m1 = new types.FieldElementZC({ ...fields.m1 });
    this.m2 = new types.FieldElementZC({ ...fields.m2 });
    this.txRemaining = fields.txRemaining;
    this.verified = fields.verified;
    this.result = fields.result;
  }

  static layout(property?: string) {
    return borsh.struct(
      [
        borsh.publicKey('producer'),
        types.VrfStatus.layout('status'),
        borsh.array(borsh.u8(), 80, 'reprProof'),
        types.EcvrfProofZC.layout('proof'),
        borsh.publicKey('yPoint'),
        borsh.u32('stage'),
        types.EcvrfIntermediate.layout('stage1Out'),
        types.EdwardsPointZC.layout('r1'),
        types.EdwardsPointZC.layout('r2'),
        types.EcvrfIntermediate.layout('stage3Out'),
        types.EdwardsPointZC.layout('hPoint'),
        types.Scalar.layout('sReduced'),
        borsh.array(types.FieldElementZC.layout(), 3, 'yPointBuilder'),
        types.EdwardsPointZC.layout('yRistrettoPoint'),
        borsh.u8('mulRound'),
        borsh.u8('hashPointsRound'),
        types.CompletedPointZC.layout('mulTmp1'),
        types.EdwardsPointZC.layout('uPoint1'),
        types.EdwardsPointZC.layout('uPoint2'),
        types.EdwardsPointZC.layout('vPoint1'),
        types.EdwardsPointZC.layout('vPoint2'),
        types.EdwardsPointZC.layout('uPoint'),
        types.EdwardsPointZC.layout('vPoint'),
        types.FieldElementZC.layout('u1'),
        types.FieldElementZC.layout('u2'),
        types.FieldElementZC.layout('invertee'),
        types.FieldElementZC.layout('y'),
        types.FieldElementZC.layout('z'),
        borsh.array(borsh.u8(), 32, 'p1Bytes'),
        borsh.array(borsh.u8(), 32, 'p2Bytes'),
        borsh.array(borsh.u8(), 32, 'p3Bytes'),
        borsh.array(borsh.u8(), 32, 'p4Bytes'),
        borsh.array(borsh.u8(), 16, 'cPrimeHashbuf'),
        types.FieldElementZC.layout('m1'),
        types.FieldElementZC.layout('m2'),
        borsh.u32('txRemaining'),
        borsh.bool('verified'),
        borsh.array(borsh.u8(), 32, 'result'),
      ],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new VrfBuilder({
      producer: obj.producer,
      status: types.VrfStatus.fromDecoded(obj.status),
      reprProof: obj.reprProof,
      proof: types.EcvrfProofZC.fromDecoded(obj.proof),
      yPoint: obj.yPoint,
      stage: obj.stage,
      stage1Out: types.EcvrfIntermediate.fromDecoded(obj.stage1Out),
      r1: types.EdwardsPointZC.fromDecoded(obj.r1),
      r2: types.EdwardsPointZC.fromDecoded(obj.r2),
      stage3Out: types.EcvrfIntermediate.fromDecoded(obj.stage3Out),
      hPoint: types.EdwardsPointZC.fromDecoded(obj.hPoint),
      sReduced: types.Scalar.fromDecoded(obj.sReduced),
      yPointBuilder: obj.yPointBuilder.map(
        (
          item: any /* eslint-disable-line @typescript-eslint/no-explicit-any */
        ) => types.FieldElementZC.fromDecoded(item)
      ),
      yRistrettoPoint: types.EdwardsPointZC.fromDecoded(obj.yRistrettoPoint),
      mulRound: obj.mulRound,
      hashPointsRound: obj.hashPointsRound,
      mulTmp1: types.CompletedPointZC.fromDecoded(obj.mulTmp1),
      uPoint1: types.EdwardsPointZC.fromDecoded(obj.uPoint1),
      uPoint2: types.EdwardsPointZC.fromDecoded(obj.uPoint2),
      vPoint1: types.EdwardsPointZC.fromDecoded(obj.vPoint1),
      vPoint2: types.EdwardsPointZC.fromDecoded(obj.vPoint2),
      uPoint: types.EdwardsPointZC.fromDecoded(obj.uPoint),
      vPoint: types.EdwardsPointZC.fromDecoded(obj.vPoint),
      u1: types.FieldElementZC.fromDecoded(obj.u1),
      u2: types.FieldElementZC.fromDecoded(obj.u2),
      invertee: types.FieldElementZC.fromDecoded(obj.invertee),
      y: types.FieldElementZC.fromDecoded(obj.y),
      z: types.FieldElementZC.fromDecoded(obj.z),
      p1Bytes: obj.p1Bytes,
      p2Bytes: obj.p2Bytes,
      p3Bytes: obj.p3Bytes,
      p4Bytes: obj.p4Bytes,
      cPrimeHashbuf: obj.cPrimeHashbuf,
      m1: types.FieldElementZC.fromDecoded(obj.m1),
      m2: types.FieldElementZC.fromDecoded(obj.m2),
      txRemaining: obj.txRemaining,
      verified: obj.verified,
      result: obj.result,
    });
  }

  static toEncodable(fields: VrfBuilderFields) {
    return {
      producer: fields.producer,
      status: fields.status.toEncodable(),
      reprProof: fields.reprProof,
      proof: types.EcvrfProofZC.toEncodable(fields.proof),
      yPoint: fields.yPoint,
      stage: fields.stage,
      stage1Out: types.EcvrfIntermediate.toEncodable(fields.stage1Out),
      r1: types.EdwardsPointZC.toEncodable(fields.r1),
      r2: types.EdwardsPointZC.toEncodable(fields.r2),
      stage3Out: types.EcvrfIntermediate.toEncodable(fields.stage3Out),
      hPoint: types.EdwardsPointZC.toEncodable(fields.hPoint),
      sReduced: types.Scalar.toEncodable(fields.sReduced),
      yPointBuilder: fields.yPointBuilder.map(item =>
        types.FieldElementZC.toEncodable(item)
      ),
      yRistrettoPoint: types.EdwardsPointZC.toEncodable(fields.yRistrettoPoint),
      mulRound: fields.mulRound,
      hashPointsRound: fields.hashPointsRound,
      mulTmp1: types.CompletedPointZC.toEncodable(fields.mulTmp1),
      uPoint1: types.EdwardsPointZC.toEncodable(fields.uPoint1),
      uPoint2: types.EdwardsPointZC.toEncodable(fields.uPoint2),
      vPoint1: types.EdwardsPointZC.toEncodable(fields.vPoint1),
      vPoint2: types.EdwardsPointZC.toEncodable(fields.vPoint2),
      uPoint: types.EdwardsPointZC.toEncodable(fields.uPoint),
      vPoint: types.EdwardsPointZC.toEncodable(fields.vPoint),
      u1: types.FieldElementZC.toEncodable(fields.u1),
      u2: types.FieldElementZC.toEncodable(fields.u2),
      invertee: types.FieldElementZC.toEncodable(fields.invertee),
      y: types.FieldElementZC.toEncodable(fields.y),
      z: types.FieldElementZC.toEncodable(fields.z),
      p1Bytes: fields.p1Bytes,
      p2Bytes: fields.p2Bytes,
      p3Bytes: fields.p3Bytes,
      p4Bytes: fields.p4Bytes,
      cPrimeHashbuf: fields.cPrimeHashbuf,
      m1: types.FieldElementZC.toEncodable(fields.m1),
      m2: types.FieldElementZC.toEncodable(fields.m2),
      txRemaining: fields.txRemaining,
      verified: fields.verified,
      result: fields.result,
    };
  }

  toJSON(): VrfBuilderJSON {
    return {
      producer: this.producer.toString(),
      status: this.status.toJSON(),
      reprProof: this.reprProof,
      proof: this.proof.toJSON(),
      yPoint: this.yPoint.toString(),
      stage: this.stage,
      stage1Out: this.stage1Out.toJSON(),
      r1: this.r1.toJSON(),
      r2: this.r2.toJSON(),
      stage3Out: this.stage3Out.toJSON(),
      hPoint: this.hPoint.toJSON(),
      sReduced: this.sReduced.toJSON(),
      yPointBuilder: this.yPointBuilder.map(item => item.toJSON()),
      yRistrettoPoint: this.yRistrettoPoint.toJSON(),
      mulRound: this.mulRound,
      hashPointsRound: this.hashPointsRound,
      mulTmp1: this.mulTmp1.toJSON(),
      uPoint1: this.uPoint1.toJSON(),
      uPoint2: this.uPoint2.toJSON(),
      vPoint1: this.vPoint1.toJSON(),
      vPoint2: this.vPoint2.toJSON(),
      uPoint: this.uPoint.toJSON(),
      vPoint: this.vPoint.toJSON(),
      u1: this.u1.toJSON(),
      u2: this.u2.toJSON(),
      invertee: this.invertee.toJSON(),
      y: this.y.toJSON(),
      z: this.z.toJSON(),
      p1Bytes: this.p1Bytes,
      p2Bytes: this.p2Bytes,
      p3Bytes: this.p3Bytes,
      p4Bytes: this.p4Bytes,
      cPrimeHashbuf: this.cPrimeHashbuf,
      m1: this.m1.toJSON(),
      m2: this.m2.toJSON(),
      txRemaining: this.txRemaining,
      verified: this.verified,
      result: this.result,
    };
  }

  static fromJSON(obj: VrfBuilderJSON): VrfBuilder {
    return new VrfBuilder({
      producer: new PublicKey(obj.producer),
      status: types.VrfStatus.fromJSON(obj.status),
      reprProof: obj.reprProof,
      proof: types.EcvrfProofZC.fromJSON(obj.proof),
      yPoint: new PublicKey(obj.yPoint),
      stage: obj.stage,
      stage1Out: types.EcvrfIntermediate.fromJSON(obj.stage1Out),
      r1: types.EdwardsPointZC.fromJSON(obj.r1),
      r2: types.EdwardsPointZC.fromJSON(obj.r2),
      stage3Out: types.EcvrfIntermediate.fromJSON(obj.stage3Out),
      hPoint: types.EdwardsPointZC.fromJSON(obj.hPoint),
      sReduced: types.Scalar.fromJSON(obj.sReduced),
      yPointBuilder: obj.yPointBuilder.map(item =>
        types.FieldElementZC.fromJSON(item)
      ),
      yRistrettoPoint: types.EdwardsPointZC.fromJSON(obj.yRistrettoPoint),
      mulRound: obj.mulRound,
      hashPointsRound: obj.hashPointsRound,
      mulTmp1: types.CompletedPointZC.fromJSON(obj.mulTmp1),
      uPoint1: types.EdwardsPointZC.fromJSON(obj.uPoint1),
      uPoint2: types.EdwardsPointZC.fromJSON(obj.uPoint2),
      vPoint1: types.EdwardsPointZC.fromJSON(obj.vPoint1),
      vPoint2: types.EdwardsPointZC.fromJSON(obj.vPoint2),
      uPoint: types.EdwardsPointZC.fromJSON(obj.uPoint),
      vPoint: types.EdwardsPointZC.fromJSON(obj.vPoint),
      u1: types.FieldElementZC.fromJSON(obj.u1),
      u2: types.FieldElementZC.fromJSON(obj.u2),
      invertee: types.FieldElementZC.fromJSON(obj.invertee),
      y: types.FieldElementZC.fromJSON(obj.y),
      z: types.FieldElementZC.fromJSON(obj.z),
      p1Bytes: obj.p1Bytes,
      p2Bytes: obj.p2Bytes,
      p3Bytes: obj.p3Bytes,
      p4Bytes: obj.p4Bytes,
      cPrimeHashbuf: obj.cPrimeHashbuf,
      m1: types.FieldElementZC.fromJSON(obj.m1),
      m2: types.FieldElementZC.fromJSON(obj.m2),
      txRemaining: obj.txRemaining,
      verified: obj.verified,
      result: obj.result,
    });
  }

  toEncodable() {
    return VrfBuilder.toEncodable(this);
  }
}
