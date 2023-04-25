import { PublicKey } from '@solana/web3.js';

// Program ID passed with the cli --program-id flag when running the code generator. Do not edit, it will get overwritten.
export const PROGRAM_ID_CLI = new PublicKey(
  'SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f'
);

// This constant will not get overwritten on subsequent code generations and it's safe to modify it's value.
export const PROGRAM_ID: PublicKey = PROGRAM_ID_CLI;
