import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface PermissionSetParamsFields {
  permission: types.SwitchboardPermissionKind;
  enable: boolean;
}

export interface PermissionSetParamsJSON {
  permission: types.SwitchboardPermissionJSON;
  enable: boolean;
}

export class PermissionSetParams {
  readonly permission: types.SwitchboardPermissionKind;
  readonly enable: boolean;

  constructor(fields: PermissionSetParamsFields) {
    this.permission = fields.permission;
    this.enable = fields.enable;
  }

  static layout(property?: string) {
    return borsh.struct(
      [types.SwitchboardPermission.layout('permission'), borsh.bool('enable')],
      property
    );
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  static fromDecoded(obj: any) {
    return new PermissionSetParams({
      permission: types.SwitchboardPermission.fromDecoded(obj.permission),
      enable: obj.enable,
    });
  }

  static toEncodable(fields: PermissionSetParamsFields) {
    return {
      permission: fields.permission.toEncodable(),
      enable: fields.enable,
    };
  }

  toJSON(): PermissionSetParamsJSON {
    return {
      permission: this.permission.toJSON(),
      enable: this.enable,
    };
  }

  static fromJSON(obj: PermissionSetParamsJSON): PermissionSetParams {
    return new PermissionSetParams({
      permission: types.SwitchboardPermission.fromJSON(obj.permission),
      enable: obj.enable,
    });
  }

  toEncodable() {
    return PermissionSetParams.toEncodable(this);
  }
}
