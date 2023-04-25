import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface AAAAJSON {
  kind: 'AAAA';
}

export class AAAA {
  static readonly discriminator = 0;
  static readonly kind = 'AAAA';
  readonly discriminator = 0;
  readonly kind = 'AAAA';

  toJSON(): AAAAJSON {
    return {
      kind: 'AAAA',
    };
  }

  toEncodable() {
    return {
      AAAA: {},
    };
  }
}

export interface BBBBJSON {
  kind: 'BBBB';
}

export class BBBB {
  static readonly discriminator = 1;
  static readonly kind = 'BBBB';
  readonly discriminator = 1;
  readonly kind = 'BBBB';

  toJSON(): BBBBJSON {
    return {
      kind: 'BBBB',
    };
  }

  toEncodable() {
    return {
      BBBB: {},
    };
  }
}

export interface BADCJSON {
  kind: 'BADC';
}

export class BADC {
  static readonly discriminator = 2;
  static readonly kind = 'BADC';
  readonly discriminator = 2;
  readonly kind = 'BADC';

  toJSON(): BADCJSON {
    return {
      kind: 'BADC',
    };
  }

  toEncodable() {
    return {
      BADC: {},
    };
  }
}

export interface BACDJSON {
  kind: 'BACD';
}

export class BACD {
  static readonly discriminator = 3;
  static readonly kind = 'BACD';
  readonly discriminator = 3;
  readonly kind = 'BACD';

  toJSON(): BACDJSON {
    return {
      kind: 'BACD',
    };
  }

  toEncodable() {
    return {
      BACD: {},
    };
  }
}

export interface ADDAJSON {
  kind: 'ADDA';
}

export class ADDA {
  static readonly discriminator = 4;
  static readonly kind = 'ADDA';
  readonly discriminator = 4;
  readonly kind = 'ADDA';

  toJSON(): ADDAJSON {
    return {
      kind: 'ADDA',
    };
  }

  toEncodable() {
    return {
      ADDA: {},
    };
  }
}

export interface CBCBJSON {
  kind: 'CBCB';
}

export class CBCB {
  static readonly discriminator = 5;
  static readonly kind = 'CBCB';
  readonly discriminator = 5;
  readonly kind = 'CBCB';

  toJSON(): CBCBJSON {
    return {
      kind: 'CBCB',
    };
  }

  toEncodable() {
    return {
      CBCB: {},
    };
  }
}

export interface ABDCJSON {
  kind: 'ABDC';
}

export class ABDC {
  static readonly discriminator = 6;
  static readonly kind = 'ABDC';
  readonly discriminator = 6;
  readonly kind = 'ABDC';

  toJSON(): ABDCJSON {
    return {
      kind: 'ABDC',
    };
  }

  toEncodable() {
    return {
      ABDC: {},
    };
  }
}

export interface ABABJSON {
  kind: 'ABAB';
}

export class ABAB {
  static readonly discriminator = 7;
  static readonly kind = 'ABAB';
  readonly discriminator = 7;
  readonly kind = 'ABAB';

  toJSON(): ABABJSON {
    return {
      kind: 'ABAB',
    };
  }

  toEncodable() {
    return {
      ABAB: {},
    };
  }
}

export interface DBBDJSON {
  kind: 'DBBD';
}

export class DBBD {
  static readonly discriminator = 8;
  static readonly kind = 'DBBD';
  readonly discriminator = 8;
  readonly kind = 'DBBD';

  toJSON(): DBBDJSON {
    return {
      kind: 'DBBD',
    };
  }

  toEncodable() {
    return {
      DBBD: {},
    };
  }
}

export interface CACAJSON {
  kind: 'CACA';
}

export class CACA {
  static readonly discriminator = 9;
  static readonly kind = 'CACA';
  readonly discriminator = 9;
  readonly kind = 'CACA';

  toJSON(): CACAJSON {
    return {
      kind: 'CACA',
    };
  }

  toEncodable() {
    return {
      CACA: {},
    };
  }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function fromDecoded(obj: any): types.ShuffleKind {
  if (typeof obj !== 'object') {
    throw new Error('Invalid enum object');
  }

  if ('AAAA' in obj) {
    return new AAAA();
  }
  if ('BBBB' in obj) {
    return new BBBB();
  }
  if ('BADC' in obj) {
    return new BADC();
  }
  if ('BACD' in obj) {
    return new BACD();
  }
  if ('ADDA' in obj) {
    return new ADDA();
  }
  if ('CBCB' in obj) {
    return new CBCB();
  }
  if ('ABDC' in obj) {
    return new ABDC();
  }
  if ('ABAB' in obj) {
    return new ABAB();
  }
  if ('DBBD' in obj) {
    return new DBBD();
  }
  if ('CACA' in obj) {
    return new CACA();
  }

  throw new Error('Invalid enum object');
}

export function fromJSON(obj: types.ShuffleJSON): types.ShuffleKind {
  switch (obj.kind) {
    case 'AAAA': {
      return new AAAA();
    }
    case 'BBBB': {
      return new BBBB();
    }
    case 'BADC': {
      return new BADC();
    }
    case 'BACD': {
      return new BACD();
    }
    case 'ADDA': {
      return new ADDA();
    }
    case 'CBCB': {
      return new CBCB();
    }
    case 'ABDC': {
      return new ABDC();
    }
    case 'ABAB': {
      return new ABAB();
    }
    case 'DBBD': {
      return new DBBD();
    }
    case 'CACA': {
      return new CACA();
    }
  }
}

export function layout(property?: string) {
  const ret = borsh.rustEnum([
    borsh.struct([], 'AAAA'),
    borsh.struct([], 'BBBB'),
    borsh.struct([], 'BADC'),
    borsh.struct([], 'BACD'),
    borsh.struct([], 'ADDA'),
    borsh.struct([], 'CBCB'),
    borsh.struct([], 'ABDC'),
    borsh.struct([], 'ABAB'),
    borsh.struct([], 'DBBD'),
    borsh.struct([], 'CACA'),
  ]);
  if (property !== undefined) {
    return ret.replicate(property);
  }
  return ret;
}
