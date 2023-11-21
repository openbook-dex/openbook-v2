import { IDL } from '../client/src/openbook_v2';
import { BorshCoder } from '@coral-xyz/anchor';

function valuesToString(obj: object) {
  for (const key of Object.keys(obj)) {
    const value = obj[key];
    if (value.toString !== Object.prototype.toString) {
      obj[key] = value.toString();
    } else {
      obj[key] = valuesToString(value);
    }
  }

  return obj;
}

function main() {
  if (process.argv.length != 3) {
    throw new Error('missing event b64 string');
  }

  const event = process.argv[2];
  const borshCoder = new BorshCoder(IDL);
  const decoded = borshCoder.events.decode(event);

  if (!decoded) {
    throw new Error(`Cannot decode ${event}`);
  } else {
    console.log(JSON.stringify(valuesToString(decoded), null, 4));
  }
}

main();
