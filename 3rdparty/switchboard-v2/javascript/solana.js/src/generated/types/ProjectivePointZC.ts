import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface ProjectivePointZCFields {
  x: types.FieldElementZCFields;
  y: types.FieldElementZCFields;
  z: types.FieldElementZCFields;
}

export interface ProjectivePointZCJSON {
  x: types.FieldElementZCJSON;
  y: types.FieldElementZCJSON;
  z: types.FieldElementZCJSON;
}

export class ProjectivePointZC {
  readonly x: types.FieldElementZC;
  readonly y: types.FieldElementZC;
  readonly z: types.FieldElementZC;

  constructor(fields: ProjectivePointZCFields) {
    this.x = new types.FieldElementZC({ ...fields.x });
    this.y = new types.FieldElementZC({ ...fields.y });
    this.z = new types.FieldElementZC({ ...fields.z });
  }

  static layout(property?: string) {
    return borsh.struct(
      [
        types.FieldElementZC.layout('x'),
        types.FieldElementZC.layout('y'),
        types.FieldElementZC.layout('z'),
      ],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new ProjectivePointZC({
      x: types.FieldElementZC.fromDecoded(obj.x),
      y: types.FieldElementZC.fromDecoded(obj.y),
      z: types.FieldElementZC.fromDecoded(obj.z),
    });
  }

  static toEncodable(fields: ProjectivePointZCFields) {
    return {
      x: types.FieldElementZC.toEncodable(fields.x),
      y: types.FieldElementZC.toEncodable(fields.y),
      z: types.FieldElementZC.toEncodable(fields.z),
    };
  }

  toJSON(): ProjectivePointZCJSON {
    return {
      x: this.x.toJSON(),
      y: this.y.toJSON(),
      z: this.z.toJSON(),
    };
  }

  static fromJSON(obj: ProjectivePointZCJSON): ProjectivePointZC {
    return new ProjectivePointZC({
      x: types.FieldElementZC.fromJSON(obj.x),
      y: types.FieldElementZC.fromJSON(obj.y),
      z: types.FieldElementZC.fromJSON(obj.z),
    });
  }

  toEncodable() {
    return ProjectivePointZC.toEncodable(this);
  }
}
