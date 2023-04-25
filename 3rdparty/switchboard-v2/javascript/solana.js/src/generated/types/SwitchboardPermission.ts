import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface PermitOracleHeartbeatJSON {
  kind: 'PermitOracleHeartbeat';
}

export class PermitOracleHeartbeat {
  static readonly discriminator = 0;
  static readonly kind = 'PermitOracleHeartbeat';
  readonly discriminator = 0;
  readonly kind = 'PermitOracleHeartbeat';

  toJSON(): PermitOracleHeartbeatJSON {
    return {
      kind: 'PermitOracleHeartbeat',
    };
  }

  toEncodable() {
    return {
      PermitOracleHeartbeat: {},
    };
  }
}

export interface PermitOracleQueueUsageJSON {
  kind: 'PermitOracleQueueUsage';
}

export class PermitOracleQueueUsage {
  static readonly discriminator = 1;
  static readonly kind = 'PermitOracleQueueUsage';
  readonly discriminator = 1;
  readonly kind = 'PermitOracleQueueUsage';

  toJSON(): PermitOracleQueueUsageJSON {
    return {
      kind: 'PermitOracleQueueUsage',
    };
  }

  toEncodable() {
    return {
      PermitOracleQueueUsage: {},
    };
  }
}

export interface PermitVrfRequestsJSON {
  kind: 'PermitVrfRequests';
}

export class PermitVrfRequests {
  static readonly discriminator = 2;
  static readonly kind = 'PermitVrfRequests';
  readonly discriminator = 2;
  readonly kind = 'PermitVrfRequests';

  toJSON(): PermitVrfRequestsJSON {
    return {
      kind: 'PermitVrfRequests',
    };
  }

  toEncodable() {
    return {
      PermitVrfRequests: {},
    };
  }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function fromDecoded(obj: any): types.SwitchboardPermissionKind {
  if (typeof obj !== 'object') {
    throw new Error('Invalid enum object');
  }

  if ('PermitOracleHeartbeat' in obj) {
    return new PermitOracleHeartbeat();
  }
  if ('PermitOracleQueueUsage' in obj) {
    return new PermitOracleQueueUsage();
  }
  if ('PermitVrfRequests' in obj) {
    return new PermitVrfRequests();
  }

  throw new Error('Invalid enum object');
}

export function fromJSON(
  obj: types.SwitchboardPermissionJSON
): types.SwitchboardPermissionKind {
  switch (obj.kind) {
    case 'PermitOracleHeartbeat': {
      return new PermitOracleHeartbeat();
    }
    case 'PermitOracleQueueUsage': {
      return new PermitOracleQueueUsage();
    }
    case 'PermitVrfRequests': {
      return new PermitVrfRequests();
    }
  }
}

export function layout(property?: string) {
  const ret = borsh.rustEnum([
    borsh.struct([], 'PermitOracleHeartbeat'),
    borsh.struct([], 'PermitOracleQueueUsage'),
    borsh.struct([], 'PermitVrfRequests'),
  ]);
  if (property !== undefined) {
    return ret.replicate(property);
  }
  return ret;
}
