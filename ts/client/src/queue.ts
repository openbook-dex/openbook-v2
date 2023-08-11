import { BorshCoder } from '@coral-xyz/anchor';
import { IDL } from './openbook_v2';

const coder = new BorshCoder(IDL);

export function decodeQueue(data: Buffer): any {
  const eventQueue = coder.accounts.decode('eventQueue', data);
  return eventQueue;
}
