import { Keypair, PublicKey } from "@solana/web3.js";

export const DEFAULT_PUBKEY = new PublicKey("11111111111111111111111111111111");

export const DEFAULT_KEYPAIR = Keypair.fromSeed(new Uint8Array(32).fill(1));
