import { PublicKey } from '@solana/web3.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh';

export interface ModeRoundResolutionJSON {
  kind: 'ModeRoundResolution';
}

export class ModeRoundResolution {
  static readonly discriminator = 0;
  static readonly kind = 'ModeRoundResolution';
  readonly discriminator = 0;
  readonly kind = 'ModeRoundResolution';

  toJSON(): ModeRoundResolutionJSON {
    return {
      kind: 'ModeRoundResolution',
    };
  }

  toEncodable() {
    return {
      ModeRoundResolution: {},
    };
  }
}

export interface ModeSlidingResolutionJSON {
  kind: 'ModeSlidingResolution';
}

export class ModeSlidingResolution {
  static readonly discriminator = 1;
  static readonly kind = 'ModeSlidingResolution';
  readonly discriminator = 1;
  readonly kind = 'ModeSlidingResolution';

  toJSON(): ModeSlidingResolutionJSON {
    return {
      kind: 'ModeSlidingResolution',
    };
  }

  toEncodable() {
    return {
      ModeSlidingResolution: {},
    };
  }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function fromDecoded(obj: any): types.AggregatorResolutionModeKind {
  if (typeof obj !== 'object') {
    throw new Error('Invalid enum object');
  }

  if ('ModeRoundResolution' in obj) {
    return new ModeRoundResolution();
  }
  if ('ModeSlidingResolution' in obj) {
    return new ModeSlidingResolution();
  }

  throw new Error('Invalid enum object');
}

export function fromJSON(
  obj: types.AggregatorResolutionModeJSON
): types.AggregatorResolutionModeKind {
  switch (obj.kind) {
    case 'ModeRoundResolution': {
      return new ModeRoundResolution();
    }
    case 'ModeSlidingResolution': {
      return new ModeSlidingResolution();
    }
  }
}

export function layout(property?: string) {
  const ret = borsh.rustEnum([
    borsh.struct([], 'ModeRoundResolution'),
    borsh.struct([], 'ModeSlidingResolution'),
  ]);
  if (property !== undefined) {
    return ret.replicate(property);
  }
  return ret;
}
