import * as anchor from '@project-serum/anchor';
import { AccountInfo, Commitment, PublicKey } from '@solana/web3.js';
import * as errors from '../errors';
import * as types from '../generated';
import { SwitchboardProgram } from '../program';
import {
  Account,
  BUFFER_DISCRIMINATOR,
  OnAccountChangeCallback,
} from './account';

/**
 * Account holding a priority queue of aggregators and their next available update time.
 *
 * Data: Array<{@linkcode types.CrankRow}>
 */
export class CrankDataBuffer extends Account<Array<types.CrankRow>> {
  static accountName = 'CrankDataBuffer';

  public size = 40;

  /**
   * Invoke a callback each time a crank's buffer has changed on-chain. The buffer stores a list of {@linkcode AggregatorAccount} public keys along with their next available update time.
   * @param callback - the callback invoked when the crank's buffer changes
   * @param commitment - optional, the desired transaction finality. defaults to 'confirmed'
   * @returns the websocket subscription id
   */
  onChange(
    callback: OnAccountChangeCallback<Array<types.CrankRow>>,
    commitment: Commitment = 'confirmed'
  ): number {
    if (this.publicKey.equals(PublicKey.default)) {
      throw new Error(
        `No crank dataBuffer provided. Call crankAccount.loadData() or pass it to this function in order to watch the account for changes`
      );
    }
    return this.program.connection.onAccountChange(
      this.publicKey,
      accountInfo => callback(CrankDataBuffer.decode(accountInfo)),
      commitment
    );
  }

  /**
   * Retrieve and decode the {@linkcode types.CrankAccountData} stored in this account.
   */
  public async loadData(): Promise<Array<types.CrankRow>> {
    if (this.publicKey.equals(PublicKey.default)) {
      return [];
    }
    const accountInfo = await this.program.connection.getAccountInfo(
      this.publicKey
    );
    if (accountInfo === null)
      throw new errors.AccountNotFoundError(
        'Crank Data Buffer',
        this.publicKey
      );
    const data = CrankDataBuffer.decode(accountInfo);
    return data;
  }

  public static decode(
    bufferAccountInfo: AccountInfo<Buffer>
  ): Array<types.CrankRow> {
    const buffer = bufferAccountInfo.data.slice(8) ?? Buffer.from('');
    const maxRows = Math.floor(buffer.byteLength / 40);

    const pqData: Array<types.CrankRow> = [];

    for (let i = 0; i < maxRows * 40; i += 40) {
      if (buffer.byteLength - i < 40) {
        break;
      }

      const rowBuf = buffer.slice(i, i + 40);
      const pubkey = new PublicKey(rowBuf.slice(0, 32));
      if (pubkey.equals(PublicKey.default)) {
        break;
      }

      const nextTimestamp = new anchor.BN(rowBuf.slice(32, 40), 'le');
      pqData.push(new types.CrankRow({ pubkey, nextTimestamp }));
    }

    return pqData;
  }

  public static getAccountSize(size: number): number {
    return 8 + size * 40;
  }

  public static default(size = 100): Buffer {
    const buffer = Buffer.alloc(CrankDataBuffer.getAccountSize(size), 0);
    BUFFER_DISCRIMINATOR.copy(buffer, 0);
    return buffer;
  }

  /**
   * Return a crank's dataBuffer
   *
   * @throws {string} if dataBuffer is equal to default publicKey
   */
  static fromCrank(
    program: SwitchboardProgram,
    crank: types.CrankAccountData
  ): CrankDataBuffer {
    if (crank.dataBuffer.equals(PublicKey.default)) {
      throw new Error(`Failed to find crank data buffer`);
    }

    return new CrankDataBuffer(program, crank.dataBuffer);
  }
}
