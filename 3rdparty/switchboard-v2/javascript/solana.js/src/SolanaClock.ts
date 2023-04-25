import * as anchor from '@project-serum/anchor';
import * as borsh from '@project-serum/borsh';
import { Connection, SYSVAR_CLOCK_PUBKEY } from '@solana/web3.js';

export interface SolanaClockDataFields {
  slot: anchor.BN;
  epochStartTimestamp: anchor.BN;
  epoch: anchor.BN;
  leaderScheduleEpoch: anchor.BN;
  unixTimestamp: anchor.BN;
}

export class SolanaClock {
  slot: anchor.BN;
  epochStartTimestamp: anchor.BN;
  epoch: anchor.BN;
  leaderScheduleEpoch: anchor.BN;
  unixTimestamp: anchor.BN;

  static readonly layout = borsh.struct([
    borsh.u64('slot'),
    borsh.i64('epochStartTimestamp'),
    borsh.u64('epoch'),
    borsh.u64('leaderScheduleEpoch'),
    borsh.i64('unixTimestamp'),
  ]);

  constructor(fields: SolanaClockDataFields) {
    this.slot = fields.slot;
    this.epochStartTimestamp = fields.epochStartTimestamp;
    this.epoch = fields.epoch;
    this.leaderScheduleEpoch = fields.epochStartTimestamp;
    this.unixTimestamp = fields.unixTimestamp;
  }

  static decode(data: Buffer): SolanaClock {
    const dec = SolanaClock.layout.decode(data) as SolanaClockDataFields;

    return new SolanaClock({
      slot: dec.slot,
      epochStartTimestamp: dec.epochStartTimestamp,
      epoch: dec.epoch,
      leaderScheduleEpoch: dec.leaderScheduleEpoch,
      unixTimestamp: dec.unixTimestamp,
    });
  }

  static decodeUnixTimestamp(data: Buffer): anchor.BN {
    return borsh.u64('unixTimestamp').decode(data, data.byteLength - 8);
  }

  static async fetch(connection: Connection): Promise<SolanaClock> {
    const sysclockInfo = await connection.getAccountInfo(SYSVAR_CLOCK_PUBKEY);
    if (!sysclockInfo) {
      throw new Error(`Failed to fetch SYSVAR_CLOCK AccountInfo`);
    }
    const clock = SolanaClock.decode(sysclockInfo.data);
    return clock;
  }
}
