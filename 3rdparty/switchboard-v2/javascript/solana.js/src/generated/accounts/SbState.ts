import { SwitchboardProgram } from '../../program';
import { PublicKey, Connection } from '@solana/web3.js';
import BN from 'bn.js'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as borsh from '@project-serum/borsh'; // eslint-disable-line @typescript-eslint/no-unused-vars
import * as types from '../types'; // eslint-disable-line @typescript-eslint/no-unused-vars

export interface SbStateFields {
  /** The account authority permitted to make account changes. */
  authority: PublicKey;
  /** The token mint used for oracle rewards, aggregator leases, and other reward incentives. */
  tokenMint: PublicKey;
  /** Token vault used by the program to receive kickbacks. */
  tokenVault: PublicKey;
  /** The token mint used by the DAO. */
  daoMint: PublicKey;
  /** Reserved for future info. */
  ebuf: Array<number>;
}

export interface SbStateJSON {
  /** The account authority permitted to make account changes. */
  authority: string;
  /** The token mint used for oracle rewards, aggregator leases, and other reward incentives. */
  tokenMint: string;
  /** Token vault used by the program to receive kickbacks. */
  tokenVault: string;
  /** The token mint used by the DAO. */
  daoMint: string;
  /** Reserved for future info. */
  ebuf: Array<number>;
}

export class SbState {
  /** The account authority permitted to make account changes. */
  readonly authority: PublicKey;
  /** The token mint used for oracle rewards, aggregator leases, and other reward incentives. */
  readonly tokenMint: PublicKey;
  /** Token vault used by the program to receive kickbacks. */
  readonly tokenVault: PublicKey;
  /** The token mint used by the DAO. */
  readonly daoMint: PublicKey;
  /** Reserved for future info. */
  readonly ebuf: Array<number>;

  static readonly discriminator = Buffer.from([
    159, 42, 192, 191, 139, 62, 168, 28,
  ]);

  static readonly layout = borsh.struct([
    borsh.publicKey('authority'),
    borsh.publicKey('tokenMint'),
    borsh.publicKey('tokenVault'),
    borsh.publicKey('daoMint'),
    borsh.array(borsh.u8(), 992, 'ebuf'),
  ]);

  constructor(fields: SbStateFields) {
    this.authority = fields.authority;
    this.tokenMint = fields.tokenMint;
    this.tokenVault = fields.tokenVault;
    this.daoMint = fields.daoMint;
    this.ebuf = fields.ebuf;
  }

  static async fetch(
    program: SwitchboardProgram,
    address: PublicKey
  ): Promise<SbState | null> {
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
  ): Promise<Array<SbState | null>> {
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

  static decode(data: Buffer): SbState {
    if (!data.slice(0, 8).equals(SbState.discriminator)) {
      throw new Error('invalid account discriminator');
    }

    const dec = SbState.layout.decode(data.slice(8));

    return new SbState({
      authority: dec.authority,
      tokenMint: dec.tokenMint,
      tokenVault: dec.tokenVault,
      daoMint: dec.daoMint,
      ebuf: dec.ebuf,
    });
  }

  toJSON(): SbStateJSON {
    return {
      authority: this.authority.toString(),
      tokenMint: this.tokenMint.toString(),
      tokenVault: this.tokenVault.toString(),
      daoMint: this.daoMint.toString(),
      ebuf: this.ebuf,
    };
  }

  static fromJSON(obj: SbStateJSON): SbState {
    return new SbState({
      authority: new PublicKey(obj.authority),
      tokenMint: new PublicKey(obj.tokenMint),
      tokenVault: new PublicKey(obj.tokenVault),
      daoMint: new PublicKey(obj.daoMint),
      ebuf: obj.ebuf,
    });
  }
}
