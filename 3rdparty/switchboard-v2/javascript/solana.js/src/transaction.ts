import * as errors from './errors';
import {
  Keypair,
  PACKET_DATA_SIZE,
  PublicKey,
  Transaction,
  TransactionInstruction,
} from '@solana/web3.js';

export interface ITransactionObject {
  /** The public key of the account that will pay the transaction fees */
  payer: PublicKey;
  /** An array of TransactionInstructions that will be added to the transaction */
  ixns: Array<TransactionInstruction>;
  /** An array of signers used to sign the transaction before sending. This may not include the payer keypair for web wallet support */
  signers: Array<Keypair>;
}

export class TransactionObject implements ITransactionObject {
  payer: PublicKey;
  ixns: Array<TransactionInstruction>;
  signers: Array<Keypair>;

  constructor(
    payer: PublicKey,
    ixns: Array<TransactionInstruction>,
    signers: Array<Keypair>
  ) {
    this.payer = payer;
    this.ixns = ixns;
    this.signers = signers;

    TransactionObject.verify(payer, ixns, signers);
  }

  /**
   * Append instructions to the beginning of a TransactionObject
   */
  public unshift(
    ixn: TransactionInstruction | Array<TransactionInstruction>,
    signers?: Array<Keypair>
  ): TransactionObject {
    const newIxns = [...this.ixns];
    if (Array.isArray(ixn)) {
      newIxns.unshift(...ixn);
    } else {
      newIxns.unshift(ixn);
    }
    const newSigners = [...this.signers];
    if (signers) {
      signers.forEach(s => {
        if (
          newSigners.findIndex(signer =>
            signer.publicKey.equals(s.publicKey)
          ) === -1
        ) {
          newSigners.push(s);
        }
      });
    }
    TransactionObject.verify(this.payer, newIxns, newSigners);
    this.ixns = newIxns;
    this.signers = newSigners;
    return this;
  }

  /**
   * Append instructions to the end of a TransactionObject
   */
  public add(
    ixn: TransactionInstruction | Array<TransactionInstruction>,
    signers?: Array<Keypair>
  ): TransactionObject {
    const newIxns = [...this.ixns];
    if (Array.isArray(ixn)) {
      newIxns.push(...ixn);
    } else {
      newIxns.push(ixn);
    }
    const newSigners = [...this.signers];
    if (signers) {
      signers.forEach(s => {
        if (
          newSigners.findIndex(signer =>
            signer.publicKey.equals(s.publicKey)
          ) === -1
        ) {
          newSigners.push(s);
        }
      });
    }
    TransactionObject.verify(this.payer, newIxns, newSigners);
    this.ixns = newIxns;
    this.signers = newSigners;
    return this;
  }

  /**
   * Verify a transaction object has less than 10 instructions, less than 1232 bytes, and contains all required signers minus the payer
   * @throws if more than 10 instructions, serialized size is greater than 1232 bytes, or if object is missing a required signer minus the payer
   */
  public static verify(
    payer: PublicKey,
    ixns: Array<TransactionInstruction>,
    signers: Array<Keypair>
  ) {
    // verify payer is not default pubkey
    if (payer.equals(PublicKey.default)) {
      throw new errors.SwitchboardProgramReadOnlyError();
    }

    // if empty object, return
    if (ixns.length === 0) {
      return;
    }

    // verify num ixns
    if (ixns.length > 10) {
      throw new errors.TransactionInstructionOverflowError(ixns.length);
    }

    // verify serialized size
    const size = TransactionObject.size(ixns);
    if (size > PACKET_DATA_SIZE) {
      throw new errors.TransactionSerializationOverflowError(size);
    }

    // verify signers
    TransactionObject.verifySigners(payer, ixns, signers);
  }

  /**
   * Return the serialized size of an array of TransactionInstructions
   */
  public static size(ixns: Array<TransactionInstruction>) {
    const encodeLength = (len: number) => {
      const bytes = new Array<number>();
      let remLen = len;
      for (;;) {
        let elem = remLen & 0x7f;
        remLen >>= 7;
        if (remLen === 0) {
          bytes.push(elem);
          break;
        } else {
          elem |= 0x80;
          bytes.push(elem);
        }
      }
      return bytes;
    };

    const reqSigners = ixns.reduce((signers, ixn) => {
      ixn.keys.map(a => {
        if (a.isSigner) {
          signers.add(a.pubkey.toBase58());
        }
      });
      return signers;
    }, new Set<string>());

    const txn = new Transaction({
      feePayer: PublicKey.default,
      blockhash: '1'.repeat(32),
      lastValidBlockHeight: 200000000,
    }).add(...ixns);

    const txnSize =
      txn.serializeMessage().length +
      reqSigners.size * 64 +
      encodeLength(reqSigners.size).length;

    // console.log(`txnSize: ${txnSize}`);
    return txnSize;
  }

  get size(): number {
    return TransactionObject.size(this.ixns);
  }

  /**
   * Try to combine two {@linkcode TransactionObject}'s
   * @throws if verification fails. See TransactionObject.verify
   */
  public combine(otherObject: TransactionObject): TransactionObject {
    if (!this.payer.equals(otherObject.payer)) {
      throw new Error(`Cannot combine transactions with different payers`);
    }
    return this.add(otherObject.ixns, otherObject.signers);
  }

  private static verifySigners(
    payer: PublicKey,
    ixns: Array<TransactionInstruction>,
    signers: Array<Keypair>
  ) {
    // get all required signers
    const reqSigners = ixns.reduce((signers, ixn) => {
      ixn.keys.map(a => {
        if (a.isSigner) {
          signers.add(a.pubkey.toBase58());
        }
      });
      return signers;
    }, new Set<string>());

    if (reqSigners.has(payer.toBase58())) {
      reqSigners.delete(payer.toBase58());
    }

    signers.forEach(s => {
      if (reqSigners.has(s.publicKey.toBase58())) {
        reqSigners.delete(s.publicKey.toBase58());
      }
    });

    if (reqSigners.size > 0) {
      throw new errors.TransactionMissingSignerError(Array.from(reqSigners));
    }
  }

  /**
   * Convert the TransactionObject into a Solana Transaction
   */
  public toTxn(blockhash: {
    blockhash: string;
    lastValidBlockHeight: number;
  }): Transaction {
    const txn = new Transaction({
      feePayer: this.payer,
      blockhash: blockhash.blockhash,
      lastValidBlockHeight: blockhash.lastValidBlockHeight,
    }).add(...this.ixns);
    return txn;
  }

  /**
   * Return a Transaction signed by the provided signers
   */
  public sign(
    blockhash: { blockhash: string; lastValidBlockHeight: number },
    signers?: Array<Keypair>
  ): Transaction {
    const txn = this.toTxn(blockhash);
    const allSigners = [...this.signers];

    if (signers) {
      allSigners.push(...signers);
    }

    if (allSigners.length) {
      txn.sign(...allSigners);
    }

    return txn;
  }

  /**
   * Pack an array of TransactionObject's into as few transactions as possible.
   */
  public static pack(
    _txns: Array<TransactionObject>
  ): Array<TransactionObject> {
    const txns = [..._txns];
    if (txns.length === 0) {
      throw new Error(`No transactions to pack`);
    }

    const packed: Array<TransactionObject> = [];

    let txn = txns.shift()!;
    while (txns.length) {
      const otherTxn = txns.shift()!;
      try {
        txn = txn.combine(otherTxn);
      } catch (error) {
        packed.push(txn);
        txn = otherTxn;
      }
    }
    packed.push(txn);
    return packed;
  }

  /**
   * Pack an array of TransactionInstructions into as few transactions as possible. Assumes only a single signer
   */
  public static packIxns(
    payer: PublicKey,
    _ixns: Array<TransactionInstruction>
  ): Array<TransactionObject> {
    const ixns = [..._ixns];
    const txns: TransactionObject[] = [];

    let txn = new TransactionObject(payer, [], []);
    while (ixns.length) {
      const ixn = ixns.shift()!;
      try {
        txn.add(ixn);
      } catch {
        txns.push(txn);
        txn = new TransactionObject(payer, [ixn], []);
      }
    }

    txns.push(txn);
    return txns;
  }
}
