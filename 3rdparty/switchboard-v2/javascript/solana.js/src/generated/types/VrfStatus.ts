import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface StatusNoneJSON {
  kind: 'StatusNone';
}

export class StatusNone {
  static readonly discriminator = 0;
  static readonly kind = 'StatusNone';
  readonly discriminator = 0;
  readonly kind = 'StatusNone';

  toJSON(): StatusNoneJSON {
    return {
      kind: 'StatusNone',
    };
  }

  toEncodable() {
    return {
      StatusNone: {},
    };
  }
}

export interface StatusRequestingJSON {
  kind: 'StatusRequesting';
}

export class StatusRequesting {
  static readonly discriminator = 1;
  static readonly kind = 'StatusRequesting';
  readonly discriminator = 1;
  readonly kind = 'StatusRequesting';

  toJSON(): StatusRequestingJSON {
    return {
      kind: 'StatusRequesting',
    };
  }

  toEncodable() {
    return {
      StatusRequesting: {},
    };
  }
}

export interface StatusVerifyingJSON {
  kind: 'StatusVerifying';
}

export class StatusVerifying {
  static readonly discriminator = 2;
  static readonly kind = 'StatusVerifying';
  readonly discriminator = 2;
  readonly kind = 'StatusVerifying';

  toJSON(): StatusVerifyingJSON {
    return {
      kind: 'StatusVerifying',
    };
  }

  toEncodable() {
    return {
      StatusVerifying: {},
    };
  }
}

export interface StatusVerifiedJSON {
  kind: 'StatusVerified';
}

export class StatusVerified {
  static readonly discriminator = 3;
  static readonly kind = 'StatusVerified';
  readonly discriminator = 3;
  readonly kind = 'StatusVerified';

  toJSON(): StatusVerifiedJSON {
    return {
      kind: 'StatusVerified',
    };
  }

  toEncodable() {
    return {
      StatusVerified: {},
    };
  }
}

export interface StatusCallbackSuccessJSON {
  kind: 'StatusCallbackSuccess';
}

export class StatusCallbackSuccess {
  static readonly discriminator = 4;
  static readonly kind = 'StatusCallbackSuccess';
  readonly discriminator = 4;
  readonly kind = 'StatusCallbackSuccess';

  toJSON(): StatusCallbackSuccessJSON {
    return {
      kind: 'StatusCallbackSuccess',
    };
  }

  toEncodable() {
    return {
      StatusCallbackSuccess: {},
    };
  }
}

export interface StatusVerifyFailureJSON {
  kind: 'StatusVerifyFailure';
}

export class StatusVerifyFailure {
  static readonly discriminator = 5;
  static readonly kind = 'StatusVerifyFailure';
  readonly discriminator = 5;
  readonly kind = 'StatusVerifyFailure';

  toJSON(): StatusVerifyFailureJSON {
    return {
      kind: 'StatusVerifyFailure',
    };
  }

  toEncodable() {
    return {
      StatusVerifyFailure: {},
    };
  }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function fromDecoded(obj: any): types.VrfStatusKind {
  if (typeof obj !== 'object') {
    throw new Error('Invalid enum object');
  }

  if ('StatusNone' in obj) {
    return new StatusNone();
  }
  if ('StatusRequesting' in obj) {
    return new StatusRequesting();
  }
  if ('StatusVerifying' in obj) {
    return new StatusVerifying();
  }
  if ('StatusVerified' in obj) {
    return new StatusVerified();
  }
  if ('StatusCallbackSuccess' in obj) {
    return new StatusCallbackSuccess();
  }
  if ('StatusVerifyFailure' in obj) {
    return new StatusVerifyFailure();
  }

  throw new Error('Invalid enum object');
}

export function fromJSON(obj: types.VrfStatusJSON): types.VrfStatusKind {
  switch (obj.kind) {
    case 'StatusNone': {
      return new StatusNone();
    }
    case 'StatusRequesting': {
      return new StatusRequesting();
    }
    case 'StatusVerifying': {
      return new StatusVerifying();
    }
    case 'StatusVerified': {
      return new StatusVerified();
    }
    case 'StatusCallbackSuccess': {
      return new StatusCallbackSuccess();
    }
    case 'StatusVerifyFailure': {
      return new StatusVerifyFailure();
    }
  }
}

export function layout(property?: string) {
  const ret = borsh.rustEnum([
    borsh.struct([], 'StatusNone'),
    borsh.struct([], 'StatusRequesting'),
    borsh.struct([], 'StatusVerifying'),
    borsh.struct([], 'StatusVerified'),
    borsh.struct([], 'StatusCallbackSuccess'),
    borsh.struct([], 'StatusVerifyFailure'),
  ]);
  if (property !== undefined) {
    return ret.replicate(property);
  }
  return ret;
}
