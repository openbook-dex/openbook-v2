import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface InvalidPublicKeyJSON {
  kind: 'InvalidPublicKey';
}

export class InvalidPublicKey {
  static readonly discriminator = 0;
  static readonly kind = 'InvalidPublicKey';
  readonly discriminator = 0;
  readonly kind = 'InvalidPublicKey';

  toJSON(): InvalidPublicKeyJSON {
    return {
      kind: 'InvalidPublicKey',
    };
  }

  toEncodable() {
    return {
      InvalidPublicKey: {},
    };
  }
}

export interface SerializationErrorJSON {
  kind: 'SerializationError';
}

export class SerializationError {
  static readonly discriminator = 1;
  static readonly kind = 'SerializationError';
  readonly discriminator = 1;
  readonly kind = 'SerializationError';

  toJSON(): SerializationErrorJSON {
    return {
      kind: 'SerializationError',
    };
  }

  toEncodable() {
    return {
      SerializationError: {},
    };
  }
}

export interface DeserializationErrorJSON {
  kind: 'DeserializationError';
}

export class DeserializationError {
  static readonly discriminator = 2;
  static readonly kind = 'DeserializationError';
  readonly discriminator = 2;
  readonly kind = 'DeserializationError';

  toJSON(): DeserializationErrorJSON {
    return {
      kind: 'DeserializationError',
    };
  }

  toEncodable() {
    return {
      DeserializationError: {},
    };
  }
}

export interface InvalidDataErrorJSON {
  kind: 'InvalidDataError';
}

export class InvalidDataError {
  static readonly discriminator = 3;
  static readonly kind = 'InvalidDataError';
  readonly discriminator = 3;
  readonly kind = 'InvalidDataError';

  toJSON(): InvalidDataErrorJSON {
    return {
      kind: 'InvalidDataError',
    };
  }

  toEncodable() {
    return {
      InvalidDataError: {},
    };
  }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function fromDecoded(obj: any): types.ErrorKind {
  if (typeof obj !== 'object') {
    throw new Error('Invalid enum object');
  }

  if ('InvalidPublicKey' in obj) {
    return new InvalidPublicKey();
  }
  if ('SerializationError' in obj) {
    return new SerializationError();
  }
  if ('DeserializationError' in obj) {
    return new DeserializationError();
  }
  if ('InvalidDataError' in obj) {
    return new InvalidDataError();
  }

  throw new Error('Invalid enum object');
}

export function fromJSON(obj: types.ErrorJSON): types.ErrorKind {
  switch (obj.kind) {
    case 'InvalidPublicKey': {
      return new InvalidPublicKey();
    }
    case 'SerializationError': {
      return new SerializationError();
    }
    case 'DeserializationError': {
      return new DeserializationError();
    }
    case 'InvalidDataError': {
      return new InvalidDataError();
    }
  }
}

export function layout(property?: string) {
  const ret = borsh.rustEnum([
    borsh.struct([], 'InvalidPublicKey'),
    borsh.struct([], 'SerializationError'),
    borsh.struct([], 'DeserializationError'),
    borsh.struct([], 'InvalidDataError'),
  ]);
  if (property !== undefined) {
    return ret.replicate(property);
  }
  return ret;
}
