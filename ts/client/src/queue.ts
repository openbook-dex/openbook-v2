import { BorshCoder } from '@coral-xyz/anchor';
import { IDL } from './openbook_v2';

const coder = new BorshCoder(IDL);

export function decodeHeap(data: Buffer): any {
  const eventHeap = coder.accounts.decode('eventHeap', data);
  return eventHeap;
}
