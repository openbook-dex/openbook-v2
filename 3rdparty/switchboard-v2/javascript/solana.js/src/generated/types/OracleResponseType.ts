import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface TypeSuccessJSON {
  kind: 'TypeSuccess';
}

export class TypeSuccess {
  static readonly discriminator = 0;
  static readonly kind = 'TypeSuccess';
  readonly discriminator = 0;
  readonly kind = 'TypeSuccess';

  toJSON(): TypeSuccessJSON {
    return {
      kind: 'TypeSuccess',
    };
  }

  toEncodable() {
    return {
      TypeSuccess: {},
    };
  }
}

export interface TypeErrorJSON {
  kind: 'TypeError';
}

export class TypeError {
  static readonly discriminator = 1;
  static readonly kind = 'TypeError';
  readonly discriminator = 1;
  readonly kind = 'TypeError';

  toJSON(): TypeErrorJSON {
    return {
      kind: 'TypeError',
    };
  }

  toEncodable() {
    return {
      TypeError: {},
    };
  }
}

export interface TypeDisagreementJSON {
  kind: 'TypeDisagreement';
}

export class TypeDisagreement {
  static readonly discriminator = 2;
  static readonly kind = 'TypeDisagreement';
  readonly discriminator = 2;
  readonly kind = 'TypeDisagreement';

  toJSON(): TypeDisagreementJSON {
    return {
      kind: 'TypeDisagreement',
    };
  }

  toEncodable() {
    return {
      TypeDisagreement: {},
    };
  }
}

export interface TypeNoResponseJSON {
  kind: 'TypeNoResponse';
}

export class TypeNoResponse {
  static readonly discriminator = 3;
  static readonly kind = 'TypeNoResponse';
  readonly discriminator = 3;
  readonly kind = 'TypeNoResponse';

  toJSON(): TypeNoResponseJSON {
    return {
      kind: 'TypeNoResponse',
    };
  }

  toEncodable() {
    return {
      TypeNoResponse: {},
    };
  }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function fromDecoded(obj: any): types.OracleResponseTypeKind {
  if (typeof obj !== 'object') {
    throw new Error('Invalid enum object');
  }

  if ('TypeSuccess' in obj) {
    return new TypeSuccess();
  }
  if ('TypeError' in obj) {
    return new TypeError();
  }
  if ('TypeDisagreement' in obj) {
    return new TypeDisagreement();
  }
  if ('TypeNoResponse' in obj) {
    return new TypeNoResponse();
  }

  throw new Error('Invalid enum object');
}

export function fromJSON(
  obj: types.OracleResponseTypeJSON
): types.OracleResponseTypeKind {
  switch (obj.kind) {
    case 'TypeSuccess': {
      return new TypeSuccess();
    }
    case 'TypeError': {
      return new TypeError();
    }
    case 'TypeDisagreement': {
      return new TypeDisagreement();
    }
    case 'TypeNoResponse': {
      return new TypeNoResponse();
    }
  }
}

export function layout(property?: string) {
  const ret = borsh.rustEnum([
    borsh.struct([], 'TypeSuccess'),
    borsh.struct([], 'TypeError'),
    borsh.struct([], 'TypeDisagreement'),
    borsh.struct([], 'TypeNoResponse'),
  ]);
  if (property !== undefined) {
    return ret.replicate(property);
  }
  return ret;
}
