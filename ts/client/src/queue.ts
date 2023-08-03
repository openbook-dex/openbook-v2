import { BorshCoder } from '@coral-xyz/anchor';
import { IDL } from './openbook_v2';

const coder = new BorshCoder(IDL);

function decodeQueue(data: Buffer) {
  const eventQueue = coder.accounts.decode('eventQueue', data);
}
