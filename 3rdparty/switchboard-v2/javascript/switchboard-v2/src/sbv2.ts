/* eslint-disable @typescript-eslint/no-non-null-assertion */
/* eslint-disable @typescript-eslint/no-non-null-asserted-optional-chain */
import * as anchor from "@project-serum/anchor";
import * as spl from "@solana/spl-token-v3";
import {
  AccountInfo,
  AccountMeta,
  clusterApiUrl,
  ConfirmOptions,
  Connection,
  Keypair,
  PublicKey,
  sendAndConfirmTransaction,
  Signer,
  SystemProgram,
  SYSVAR_INSTRUCTIONS_PUBKEY,
  SYSVAR_RECENT_BLOCKHASHES_PUBKEY,
  Transaction,
  TransactionSignature,
} from "@solana/web3.js";
import { OracleJob, SwitchboardDecimal, toUtf8 } from "@switchboard-xyz/common";
import assert from "assert";
import Big from "big.js";
import * as crypto from "crypto";
import lodash from "lodash";

export type SwitchboardProgram = anchor.Program;

export { SwitchboardDecimal } from "@switchboard-xyz/common";

/**
 * Switchboard Devnet Program ID
 * 2TfB33aLaneQb5TNVwyDz3jSZXS6jdW2ARw1Dgf84XCG
 */
export const SBV2_DEVNET_PID = new PublicKey(
  "2TfB33aLaneQb5TNVwyDz3jSZXS6jdW2ARw1Dgf84XCG"
);
/**
 * Switchboard Mainnet Program ID
 * SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f
 */
export const SBV2_MAINNET_PID = new PublicKey(
  "SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f"
);

export const GOVERNANCE_PID = new PublicKey(
  "GovER5Lthms3bLBqWub97yVrMmEogzX7xNjdXpPPCVZw"
  // "2iNnEMZuLk2TysefLvXtS6kyvCFC7CDUTLLeatVgRend"
);

/**
 * Load the Switchboard Program ID for a given cluster
 * @param cluster solana cluster to fetch program ID for
 * @return Switchboard Program ID Public Key
 */
export function getSwitchboardPid(
  cluster: "devnet" | "mainnet-beta"
): PublicKey {
  switch (cluster) {
    case "devnet":
      return SBV2_DEVNET_PID;
    case "mainnet-beta":
      return SBV2_MAINNET_PID;
    default:
      throw new Error(`no Switchboard PID associated with cluster ${cluster}`);
  }
}

/**
 * Load the Switchboard Program for a given cluster
 * @param cluster solana cluster to interact with
 * @param connection optional Connection object to use for rpc request
 * @param payerKeypair optional Keypair to use for onchain txns. If ommited, a dummy keypair will be used and onchain txns will fail
 * @param confirmOptions optional confirmation options for rpc request
 * @return Switchboard Program
 */
export async function loadSwitchboardProgram(
  cluster: "devnet" | "mainnet-beta",
  connection = new Connection(clusterApiUrl(cluster)),
  payerKeypair?: Keypair,
  confirmOptions: ConfirmOptions = {
    commitment: "confirmed",
  }
): Promise<SwitchboardProgram> {
  const DEFAULT_KEYPAIR = Keypair.fromSeed(new Uint8Array(32).fill(1));
  const programId = getSwitchboardPid(cluster);
  const wallet: AnchorWallet = payerKeypair
    ? new AnchorWallet(payerKeypair)
    : new AnchorWallet(DEFAULT_KEYPAIR);
  const provider = new anchor.AnchorProvider(
    connection,
    wallet,
    confirmOptions
  );

  const anchorIdl = await anchor.Program.fetchIdl(programId, provider);
  if (!anchorIdl) {
    throw new Error(`failed to read idl for ${cluster} ${programId}`);
  }

  return new anchor.Program(anchorIdl, programId, provider);
}

// should also check if pubkey is a token account
export const findAccountName = (
  program: SwitchboardProgram,
  accountInfo: AccountInfo<Buffer>
): string => {
  const accountDiscriminator = accountInfo.data.slice(
    0,
    anchor.ACCOUNT_DISCRIMINATOR_SIZE
  );

  for (const accountDef of program.idl.accounts) {
    const typeDiscriminator = anchor.BorshAccountsCoder.accountDiscriminator(
      accountDef.name
    );
    if (Buffer.compare(accountDiscriminator, typeDiscriminator) === 0) {
      return accountDef.name;
    }
  }

  throw new Error("failed to match account type by discriminator");
};

/** Callback to pass deserialized account data when updated on-chain */
export type OnAccountChangeCallback = (accountData: any) => void;

export function watchSwitchboardAccount(
  program: SwitchboardProgram,
  publicKey: PublicKey,
  accountName: string,
  callback: OnAccountChangeCallback
): number {
  // const accountName = await findAccountName(program, publicKey);
  const accountDef = program.idl.accounts.find((a) => a.name === accountName);
  if (!accountDef) {
    throw new Error(`Failed to find account ${accountName} in switchboard IDL`);
  }
  const coder = new anchor.BorshAccountsCoder(program.idl);

  return program.provider.connection.onAccountChange(
    publicKey,
    (accountInfo, context) => {
      const data = coder.decode(accountName, accountInfo?.data);
      callback(data);
    }
  );
}

/**
 * Input parameters for constructing wrapped representations of Switchboard accounts.
 */
export interface AccountParams {
  /**
   * program referencing the Switchboard program and IDL.
   */
  program: SwitchboardProgram;
  /**
   * Public key of the account being referenced. This will always be populated
   * within the account wrapper.
   */
  publicKey?: PublicKey;
  /**
   * Keypair of the account being referenced. This may not always be populated.
   */
  keypair?: Keypair;
}

/**
 * Input parameters initializing program state.
 */
export interface ProgramInitParams {
  mint?: PublicKey;
  daoMint?: PublicKey;
}
export interface ProgramConfigParams {
  mint?: PublicKey;
  daoMint?: PublicKey;
}

/**
 * Input parameters for transferring from Switchboard token vault.
 */
export interface VaultTransferParams {
  amount: anchor.BN;
}

/**
 * Account type representing Switchboard global program state.
 */
export class ProgramStateAccount {
  static accountName = "SbState";

  program: SwitchboardProgram;

  publicKey: PublicKey;

  keypair?: Keypair;

  /**
   * ProgramStateAccount constructor
   * @param params initialization params.
   */
  public constructor(params: AccountParams) {
    if (params.keypair === undefined && params.publicKey === undefined) {
      throw new Error(
        `${this.constructor.name}: User must provide either a publicKey or keypair for account use.`
      );
    }
    if (params.keypair !== undefined && params.publicKey !== undefined) {
      if (!params.publicKey.equals(params.keypair.publicKey)) {
        throw new Error(
          `${this.constructor.name}: provided pubkey and keypair mismatch.`
        );
      }
    }
    this.program = params.program;
    this.keypair = params.keypair;
    this.publicKey = params.publicKey ?? this.keypair.publicKey;
  }

  /**
   * Constructs ProgramStateAccount from the static seed from which it was generated.
   * @return ProgramStateAccount and PDA bump tuple.
   */
  static fromSeed(program: SwitchboardProgram): [ProgramStateAccount, number] {
    const [statePubkey, stateBump] =
      anchor.utils.publicKey.findProgramAddressSync(
        [Buffer.from("STATE")],
        program.programId
      );
    return [
      new ProgramStateAccount({ program, publicKey: statePubkey }),
      stateBump,
    ];
  }

  /**
   * Load and parse ProgramStateAccount state based on the program IDL.
   * @return ProgramStateAccount data parsed in accordance with the
   * Switchboard IDL.
   */
  async loadData(): Promise<any> {
    const state: any = await this.program.account.sbState.fetch(this.publicKey);
    state.ebuf = undefined;
    return state;
  }

  /**
   * Fetch the Switchboard token mint specified in the program state account.
   * @return Switchboard token mint.
   */
  async getTokenMint(): Promise<spl.Mint> {
    const state = await this.loadData();
    const switchTokenMint = spl.getMint(
      this.program.provider.connection,
      state.tokenMint
    );
    return switchTokenMint;
  }

  /**
   * @return account size of the global ProgramStateAccount.
   */
  size(): number {
    return this.program.account.sbState.size;
  }

  static async getOrCreate(
    program: SwitchboardProgram,
    params: ProgramInitParams
  ): Promise<[ProgramStateAccount, number]> {
    const [account, seed] = ProgramStateAccount.fromSeed(program);
    try {
      await account.loadData();
    } catch (e) {
      try {
        await ProgramStateAccount.create(program, params);
      } catch {}
    }
    return [account, seed];
  }

  /**
   * Create and initialize the ProgramStateAccount.
   * @param program Switchboard program representation holding connection and IDL.
   * @param params.
   * @return newly generated ProgramStateAccount.
   */
  static async create(
    program: SwitchboardProgram,
    params: ProgramInitParams
  ): Promise<ProgramStateAccount> {
    const payerKeypair = programWallet(program);
    const [stateAccount, stateBump] = ProgramStateAccount.fromSeed(program);
    const psa = new ProgramStateAccount({
      program,
      publicKey: stateAccount.publicKey,
    });
    // Short circuit if already created.
    try {
      await psa.loadData();
      return psa;
    } catch (e) {}
    let mint = null;
    let vault = null;
    if (params.mint === undefined) {
      const decimals = 9;
      mint = await spl.createMint(
        program.provider.connection,
        payerKeypair,
        payerKeypair.publicKey,
        null,
        decimals
      );
      const tokenVault = await spl.createAccount(
        program.provider.connection,
        payerKeypair,
        mint,
        Keypair.generate().publicKey
      );
      await spl.mintTo(
        program.provider.connection,
        payerKeypair,
        mint,
        tokenVault,
        payerKeypair.publicKey,
        100_000_000
      );
      vault = tokenVault;
    } else {
      mint = params.mint;
      vault = await spl.createAccount(
        program.provider.connection,
        payerKeypair,
        mint,
        payerKeypair.publicKey
      );
    }
    await program.methods
      .programInit({
        stateBump,
      })
      .accounts({
        state: stateAccount.publicKey,
        authority: payerKeypair.publicKey,
        tokenMint: mint,
        vault,
        payer: payerKeypair.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: spl.TOKEN_PROGRAM_ID,
        daoMint: params.daoMint ?? mint,
      })
      .rpc();
    return psa;
  }

  /**
   * Transfer N tokens from the program vault to a specified account.
   * @param to The recipient of the vault tokens.
   * @param authority The vault authority required to sign the transfer tx.
   * @param params specifies the amount to transfer.
   * @return TransactionSignature
   */
  async vaultTransfer(
    to: PublicKey,
    authority: Keypair,
    params: VaultTransferParams
  ): Promise<TransactionSignature> {
    const [statePubkey, stateBump] =
      anchor.utils.publicKey.findProgramAddressSync(
        [Buffer.from("STATE")],
        this.program.programId
      );
    const vault = (await this.loadData()).tokenVault;
    return this.program.methods
      .vaultTransfer({
        stateBump,
        amount: params.amount,
      })
      .accounts({
        state: statePubkey,
        to,
        vault,
        authority: authority.publicKey,
        tokenProgram: spl.TOKEN_PROGRAM_ID,
      })
      .signers([authority])
      .rpc();
  }
}

/**
 * Parameters to initialize an aggregator account.
 */
export interface AggregatorInitParams {
  /**
   *  Name of the aggregator to store on-chain.
   */
  name?: Buffer;
  /**
   *  Metadata of the aggregator to store on-chain.
   */
  metadata?: Buffer;
  /**
   *  Number of oracles to request on aggregator update.
   */
  batchSize: number;
  /**
   *  Minimum number of oracle responses required before a round is validated.
   */
  minRequiredOracleResults: number;
  /**
   *  Minimum number of feed jobs suggested to be successful before an oracle
   *  sends a response.
   */
  minRequiredJobResults: number;
  /**
   *  Minimum number of seconds required between aggregator rounds.
   */
  minUpdateDelaySeconds: number;
  /**
   *  unix_timestamp for which no feed update will occur before.
   */
  startAfter?: anchor.BN;
  /**
   *  Change percentage required between a previous round and the current round.
   *  If variance percentage is not met, reject new oracle responses.
   */
  varianceThreshold?: number;
  /**
   *  Number of seconds for which, even if the variance threshold is not passed,
   *  accept new responses from oracles.
   */
  forceReportPeriod?: anchor.BN;
  /**
   *  unix_timestamp after which funds may be withdrawn from the aggregator.
   *  null/undefined/0 means the feed has no expiration.
   */
  expiration?: anchor.BN;
  /**
   *  If true, this aggregator is disallowed from being updated by a crank on the queue.
   */
  disableCrank?: boolean;
  /**
   *  Optional pre-existing keypair to use for aggregator initialization.
   */
  keypair?: Keypair;
  /**
   *  An optional wallet for receiving kickbacks from job usage in feeds.
   *  Defaults to token vault.
   */
  authorWallet?: PublicKey;
  /**
   *  If included, this keypair will be the aggregator authority rather than
   *  the aggregator keypair.
   */
  authority?: PublicKey;
  /**
   *  The queue to which this aggregator will be linked
   */
  queueAccount: OracleQueueAccount;
}

/**
 * Parameters for which oracles must submit for responding to update requests.
 */
export interface AggregatorSaveResultParams {
  /**
   *  Index in the list of oracles in the aggregator assigned to this round update.
   */
  oracleIdx: number;
  /**
   *  Reports that an error occured and the oracle could not send a value.
   */
  error: boolean;
  /**
   *  Value the oracle is responding with for this update.
   */
  value: Big;
  /**
   *  The minimum value this oracle has seen this round for the jobs listed in the
   *  aggregator.
   */
  minResponse: Big;
  /**
   *  The maximum value this oracle has seen this round for the jobs listed in the
   *  aggregator.
   */
  maxResponse: Big;
  /**
   *  List of OracleJobs that were performed to produce this result.
   */
  jobs: Array<OracleJob>;
  /**
   *  Authority of the queue the aggregator is attached to.
   */
  queueAuthority: PublicKey;
  /**
   *  Program token mint.
   */
  tokenMint: PublicKey;
  /**
   *  List of parsed oracles.
   */
  oracles: Array<any>;
  /**
   *  List of oracle results relative to the job idx. null if failed
   */
  jobValues: Array<SwitchboardDecimal | null>;
}

/**
 * Parameters for creating and setting a history buffer for an aggregator
 */
export interface AggregatorSetHistoryBufferParams {
  /*
   * Authority keypair for the aggregator.
   */
  authority?: Keypair;
  /*
   * Number of elements for the history buffer to fit.
   */
  size: number;
}

/**
 * Parameters required to open an aggregator round
 */
export interface AggregatorOpenRoundParams {
  /**
   *  The oracle queue from which oracles are assigned this update.
   */
  oracleQueueAccount: OracleQueueAccount;
  /**
   *  The token wallet which will receive rewards for calling update on this feed.
   */
  payoutWallet: PublicKey;
}

/**
 * Switchboard wrapper for anchor program errors.
 */
export class SwitchboardError {
  /**
   *  The program containing the Switchboard IDL specifying error codes.
   */
  program: SwitchboardProgram;

  /**
   *  Stringified name of the error type.
   */
  name: string;

  /**
   *  Numerical SwitchboardError representation.
   */
  code: number;

  /**
   *  Message describing this error in detail.
   */
  msg?: string;

  /**
   * Converts a numerical error code to a SwitchboardError based on the program
   * IDL.
   * @param program the Switchboard program object containing the program IDL.
   * @param code Error code to convert to a SwitchboardError object.
   * @return SwitchboardError
   */
  static fromCode(program: SwitchboardProgram, code: number): SwitchboardError {
    for (const e of program.idl.errors ?? []) {
      if (code === e.code) {
        const r = new SwitchboardError();
        r.program = program;
        r.name = e.name;
        r.code = e.code;
        r.msg = e.msg;
        return r;
      }
    }
    throw new Error(`Could not find SwitchboardError for error code ${code}`);
  }
}

export type TxnResponse<T = unknown> = {
  created?: T;
  ixns: anchor.web3.TransactionInstruction[];
  signers: anchor.web3.Signer[];
};

/**
 * Row structure of elements in the aggregator history buffer.
 */
export class AggregatorHistoryRow {
  /**
   *  Timestamp of the aggregator result.
   */
  timestamp: anchor.BN;

  /**
   *  Aggregator value at timestamp.
   */
  value: Big;

  static from(buf: Buffer): AggregatorHistoryRow {
    const timestamp = new anchor.BN(buf.slice(0, 8), "le");
    // TODO(mgild): does this work for negative???
    const mantissa = new anchor.BN(buf.slice(8, 24), "le");
    const scale = buf.readUInt32LE(24);
    const decimal = new SwitchboardDecimal(mantissa, scale);
    const res = new AggregatorHistoryRow();
    res.timestamp = timestamp;
    res.value = decimal.toBig();
    return res;
  }
}

export type AggregatorSetConfigParams = Partial<{
  name?: Buffer;
  metadata?: Buffer;
  batchSize?: number;
  minOracleResults?: number;
  minJobResults?: number;
  minUpdateDelaySeconds?: number;
  forceReportPeriod?: number;
  varianceThreshold?: number;
  basePriorityFee?: number;
  priorityFeeBump?: number;
  priorityFeeBumpPeriod?: number;
  maxPriorityFeeMultiplier?: number;
}>;

export interface AggregatorSetQueueParams {
  queueAccount: OracleQueueAccount;
  authority?: Keypair;
}

/**
 * Account type representing an aggregator (data feed).
 */
export class AggregatorAccount {
  static accountName = "AggregatorAccountData";

  program: SwitchboardProgram;

  publicKey: PublicKey;

  keypair?: Keypair;

  /**
   * AggregatorAccount constructor
   * @param params initialization params.
   */
  public constructor(params: AccountParams) {
    if (params.keypair === undefined && params.publicKey === undefined) {
      throw new Error(
        `${this.constructor.name}: User must provide either a publicKey or keypair for account use.`
      );
    }
    if (params.keypair !== undefined && params.publicKey !== undefined) {
      if (!params.publicKey.equals(params.keypair.publicKey)) {
        throw new Error(
          `${this.constructor.name}: provided pubkey and keypair mismatch.`
        );
      }
    }
    this.program = params.program;
    this.keypair = params.keypair;
    this.publicKey = params.publicKey ?? this.keypair.publicKey;
  }

  static decode(
    program: SwitchboardProgram,
    accountInfo: AccountInfo<Buffer>
  ): any {
    const coder = new anchor.BorshAccountsCoder(program.idl);
    const aggregator = coder.decode(
      AggregatorAccount.accountName,
      accountInfo?.data!
    );
    return aggregator;
  }

  /**
   * Returns the aggregator's ID buffer in a stringified format.
   * @param aggregator A preloaded aggregator object.
   * @return The name of the aggregator.
   */
  static getName = (aggregator: any) => toUtf8(aggregator.name);

  /**
   * Returns the aggregator's metadata buffer in a stringified format.
   * @param aggregator A preloaded aggregator object.
   * @return The stringified metadata of the aggregator.
   */
  static getMetadata = (aggregator: any) => toUtf8(aggregator.metadata);

  /**
   * Load and parse AggregatorAccount state based on the program IDL.
   * @return AggregatorAccount data parsed in accordance with the
   * Switchboard IDL.
   */
  async loadData(): Promise<any> {
    const aggregator: any =
      await this.program.account.aggregatorAccountData.fetch(this.publicKey);
    aggregator.ebuf = undefined;
    return aggregator;
  }

  onChange(callback: OnAccountChangeCallback): number {
    const coder = new anchor.BorshAccountsCoder(this.program.idl);
    return this.program.provider.connection.onAccountChange(
      this.publicKey,
      (accountInfo, context) => {
        const aggregator = coder.decode(
          AggregatorAccount.accountName,
          accountInfo?.data
        );
        callback(aggregator);
      }
    );
  }

  async onResult(
    callback: (result: {
      feedPubkey: PublicKey;
      result: Big;
      slot: anchor.BN;
      timestamp: anchor.BN;
      oracleValues: Big[];
    }) => Promise<void>
  ): Promise<number> {
    return this.program.addEventListener(
      "AggregatorValueUpdateEvent",
      (event, slot) => {
        const result = SwitchboardDecimal.from(
          event.value as { mantissa: anchor.BN; scale: number }
        ).toBig();
        const oracleValues: Big[] = (
          event.oracleValues as { mantissa: anchor.BN; scale: number }[]
        ).map((v) => SwitchboardDecimal.from(v).toBig());
        callback({
          feedPubkey: event.feedPubkey as PublicKey,
          result,
          slot: event.slot as anchor.BN,
          timestamp: event.timestamp as anchor.BN,
          oracleValues,
        });
      }
    );
  }

  async loadHistory(aggregator?: any): Promise<Array<AggregatorHistoryRow>> {
    aggregator = aggregator ?? (await this.loadData());
    if (aggregator.historyBuffer == PublicKey.default) {
      return [];
    }
    const ROW_SIZE = 28;
    let buffer =
      (
        await this.program.provider.connection.getAccountInfo(
          aggregator.historyBuffer
        )
      )?.data ?? Buffer.from("");
    if (buffer.length < 12) {
      return [];
    }
    const insertIdx = buffer.readUInt32LE(8) * ROW_SIZE;
    // console.log(insertIdx);
    buffer = buffer.slice(12);
    const front = [];
    const tail = [];
    for (let i = 0; i < buffer.length; i += ROW_SIZE) {
      if (i + ROW_SIZE > buffer.length) {
        break;
      }
      const row = AggregatorHistoryRow.from(buffer.slice(i, i + ROW_SIZE));
      if (row.timestamp.eq(new anchor.BN(0))) {
        break;
      }
      if (i <= insertIdx) {
        tail.push(row);
      } else {
        front.push(row);
      }
    }
    return front.concat(tail);
  }

  /**
   * Get the latest confirmed value stored in the aggregator account.
   * @param aggregator Optional parameter representing the already loaded
   * aggregator info.
   * @return latest feed value
   */
  async getLatestValue(aggregator?: any, decimals = 20): Promise<Big | null> {
    aggregator = aggregator ?? (await this.loadData());
    if ((aggregator.latestConfirmedRound?.numSuccess ?? 0) === 0) {
      return null;
    }
    const mantissa = new Big(
      aggregator.latestConfirmedRound.result.mantissa.toString()
    );
    const scale = aggregator.latestConfirmedRound.result.scale;
    const oldDp = Big.DP;
    Big.DP = decimals;
    const result: Big = mantissa.div(new Big(10).pow(scale));
    Big.DP = oldDp;
    return result;
  }

  /**
   * Get the timestamp latest confirmed round stored in the aggregator account.
   * @param aggregator Optional parameter representing the already loaded
   * aggregator info.
   * @return latest feed timestamp
   */
  async getLatestFeedTimestamp(aggregator?: any): Promise<anchor.BN> {
    aggregator = aggregator ?? (await this.loadData());
    if ((aggregator.latestConfirmedRound?.numSuccess ?? 0) === 0) {
      throw new Error("Aggregator currently holds no value.");
    }
    return aggregator.latestConfirmedRound.roundOpenTimestamp;
  }

  /**
   * Speciifies if the aggregator settings recommend reporting a new value
   * @param value The value which we are evaluating
   * @param aggregator The loaded aggegator schema
   * @returns boolean
   */
  static shouldReportValue(value: Big, aggregator: any): boolean {
    if ((aggregator.latestConfirmedRound?.numSuccess ?? 0) === 0) {
      return true;
    }
    const timestamp: anchor.BN = new anchor.BN(Math.round(Date.now() / 1000));
    if (aggregator.startAfter.gt(timestamp)) {
      return false;
    }
    const varianceThreshold: Big = SwitchboardDecimal.from(
      aggregator.varianceThreshold
    ).toBig();
    const latestResult: Big = SwitchboardDecimal.from(
      aggregator.latestConfirmedRound.result
    ).toBig();
    const forceReportPeriod: anchor.BN = aggregator.forceReportPeriod;
    const lastTimestamp: anchor.BN =
      aggregator.latestConfirmedRound.roundOpenTimestamp;
    if (lastTimestamp.add(aggregator.forceReportPeriod).lt(timestamp)) {
      return true;
    }
    let diff = safeDiv(latestResult, value);
    if (diff.abs().gt(1)) {
      diff = safeDiv(value, latestResult);
    }
    // I dont want to think about variance percentage when values cross 0.
    // Changes the scale of what we consider a "percentage".
    if (diff.lt(0)) {
      return true;
    }
    const changePercent = new Big(1).minus(diff).mul(100);
    return changePercent.gt(varianceThreshold);
  }

  /**
   * Get the individual oracle results of the latest confirmed round.
   * @param aggregator Optional parameter representing the already loaded
   * aggregator info.
   * @return latest results by oracle pubkey
   */
  async getConfirmedRoundResults(
    aggregator?: any
  ): Promise<Array<{ oracleAccount: OracleAccount; value: Big }>> {
    aggregator = aggregator ?? (await this.loadData());
    if ((aggregator.latestConfirmedRound?.numSuccess ?? 0) === 0) {
      throw new Error("Aggregator currently holds no value.");
    }
    const results: Array<{ oracleAccount: OracleAccount; value: Big }> = [];
    for (let i = 0; i < aggregator.oracleRequestBatchSize; ++i) {
      if (aggregator.latestConfirmedRound.mediansFulfilled[i] === true) {
        results.push({
          oracleAccount: new OracleAccount({
            program: this.program,
            publicKey: aggregator.latestConfirmedRound.oraclePubkeysData[i],
          }),
          value: SwitchboardDecimal.from(
            aggregator.latestConfirmedRound.mediansData[i]
          ).toBig(),
        });
      }
    }
    return results;
  }

  /**
   * Produces a hash of all the jobs currently in the aggregator
   * @return hash of all the feed jobs.
   */
  produceJobsHash(jobs: Array<OracleJob>): crypto.Hash {
    const hash = crypto.createHash("sha256");
    for (const job of jobs) {
      const jobHasher = crypto.createHash("sha256");
      jobHasher.update(OracleJob.encodeDelimited(job).finish());
      hash.update(jobHasher.digest());
    }
    return hash;
  }

  async loadCurrentRoundOracles(aggregator?: any): Promise<Array<any>> {
    const coder = new anchor.BorshAccountsCoder(this.program.idl);

    aggregator = aggregator ?? (await this.loadData());

    const oracleAccountDatas = await anchor.utils.rpc.getMultipleAccounts(
      this.program.provider.connection,
      aggregator.currentRound?.oraclePubkeysData?.slice(
        0,
        aggregator.oracleRequestBatchSize
      )
    );
    if (oracleAccountDatas === null) {
      throw new Error("Failed to load aggregator oracles");
    }
    return oracleAccountDatas.map((item) =>
      coder.decode("OracleAccountData", item.account.data)
    );
  }

  async loadJobAccounts(aggregator?: any): Promise<Array<any>> {
    const coder = new anchor.BorshAccountsCoder(this.program.idl);

    aggregator = aggregator ?? (await this.loadData());

    const jobAccountDatas = await anchor.utils.rpc.getMultipleAccounts(
      this.program.provider.connection,
      aggregator.jobPubkeysData.slice(0, aggregator.jobPubkeysSize)
    );
    if (jobAccountDatas === null) {
      throw new Error("Failed to load feed jobs.");
    }
    const jobs = jobAccountDatas.map((item) => {
      return coder.decode(JobAccount.accountName, item.account.data);
    });
    return jobs;
  }

  /**
   * Load and deserialize all jobs stored in this aggregator
   * @return Array<OracleJob>
   */
  async loadJobs(aggregator?: any): Promise<Array<OracleJob>> {
    const coder = new anchor.BorshAccountsCoder(this.program.idl);

    aggregator = aggregator ?? (await this.loadData());

    const jobAccountDatas = await anchor.utils.rpc.getMultipleAccounts(
      this.program.provider.connection,
      aggregator.jobPubkeysData.slice(0, aggregator.jobPubkeysSize)
    );
    if (jobAccountDatas === null) {
      throw new Error("Failed to load feed jobs.");
    }
    const jobs = jobAccountDatas.map((item) => {
      const decoded = coder.decode(JobAccount.accountName, item.account.data);
      return OracleJob.decodeDelimited(decoded.data);
    });
    return jobs;
  }

  async loadHashes(aggregator?: any): Promise<Array<Buffer>> {
    const coder = new anchor.BorshAccountsCoder(this.program.idl);

    aggregator = aggregator ?? (await this.loadData());

    const jobAccountDatas = await anchor.utils.rpc.getMultipleAccounts(
      this.program.provider.connection,
      aggregator.jobPubkeysData.slice(0, aggregator.jobPubkeysSize)
    );
    if (jobAccountDatas === null) {
      throw new Error("Failed to load feed jobs.");
    }
    const jobs = jobAccountDatas.map((item) => {
      const decoded = coder.decode(JobAccount.accountName, item.account.data);
      return decoded.hash;
    });
    return jobs;
  }

  /**
   * Get the size of an AggregatorAccount on chain.
   * @return size.
   */
  size(): number {
    return this.program.account.aggregatorAccountData.size;
  }

  /**
   * Create and initialize the AggregatorAccount.
   * @param program Switchboard program representation holding connection and IDL.
   * @param params.
   * @return newly generated AggregatorAccount.
   */
  static async create(
    program: SwitchboardProgram,
    params: AggregatorInitParams
  ): Promise<AggregatorAccount> {
    const aggregatorAccount = params.keypair ?? anchor.web3.Keypair.generate();
    const authority = params.authority ?? aggregatorAccount.publicKey;
    const size = program.account.aggregatorAccountData.size;
    const [stateAccount, stateBump] = ProgramStateAccount.fromSeed(program);
    const state = await stateAccount.loadData();
    await program.methods
      .aggregatorInit({
        name: (params.name ?? Buffer.from("")).slice(0, 32),
        metadata: (params.metadata ?? Buffer.from("")).slice(0, 128),
        batchSize: params.batchSize,
        minOracleResults: params.minRequiredOracleResults,
        minJobResults: params.minRequiredJobResults,
        minUpdateDelaySeconds: params.minUpdateDelaySeconds,
        startAfter: params.startAfter,
        varianceThreshold: SwitchboardDecimal.fromBig(
          new Big(params.varianceThreshold ?? 0)
        ),
        forceReportPeriod: params.forceReportPeriod ?? new anchor.BN(0),
        expiration: params.expiration ?? new anchor.BN(0),
        disableCrank: params.disableCrank,
        stateBump,
      })
      .accounts({
        aggregator: aggregatorAccount.publicKey,
        authority,
        queue: params.queueAccount.publicKey,
        authorWallet: params.authorWallet ?? state.tokenVault,
        programState: stateAccount.publicKey,
      })
      .signers([aggregatorAccount])
      .preInstructions([
        anchor.web3.SystemProgram.createAccount({
          fromPubkey: programWallet(program).publicKey,
          newAccountPubkey: aggregatorAccount.publicKey,
          space: size,
          lamports:
            await program.provider.connection.getMinimumBalanceForRentExemption(
              size
            ),
          programId: program.programId,
        }),
      ])
      .rpc();

    return new AggregatorAccount({ program, keypair: aggregatorAccount });
  }

  async setConfigTxn(
    params: AggregatorSetConfigParams & { authority?: Keypair }
  ): Promise<anchor.web3.Transaction> {
    const program = this.program;
    const authority =
      params.authority ?? this.keypair ?? programWallet(this.program);
    return program.methods
      .aggregatorSetConfig({
        name: params.name ?? null,
        metadata: params.metadata ?? null,
        batchSize: params.batchSize ?? null,
        minOracleResults: params.minOracleResults ?? null,
        minUpdateDelaySeconds: params.minUpdateDelaySeconds ?? null,
        minJobResults: params.minJobResults ?? null,
        forceReportPeriod: params.forceReportPeriod ?? null,
        varianceThreshold: lodash.isFinite(params.varianceThreshold)
          ? SwitchboardDecimal.fromBig(new Big(params.varianceThreshold))
          : null,
        basePriorityFee: params.basePriorityFee ?? null,
        priorityFeeBump: params.priorityFeeBump ?? null,
        priorityFeeBumpPeriod: params.priorityFeeBumpPeriod ?? null,
        maxPriorityFeeMultiplier: params.maxPriorityFeeMultiplier ?? null,
      })
      .accounts({
        aggregator: this.publicKey,
        authority: authority.publicKey,
      })
      .signers([authority])
      .transaction();
  }

  async setConfig(
    params: AggregatorSetConfigParams
  ): Promise<TransactionSignature> {
    const provider = this.program.provider as anchor.AnchorProvider;
    return provider.sendAndConfirm(await this.setConfigTxn(params), [
      this.keypair ?? programWallet(this.program),
    ]);
  }

  async setHistoryBuffer(
    params: AggregatorSetHistoryBufferParams
  ): Promise<TransactionSignature> {
    const buffer = Keypair.generate();
    const program = this.program;
    const authority =
      params.authority ?? this.keypair ?? programWallet(this.program);
    const HISTORY_ROW_SIZE = 28;
    const INSERT_IDX_SIZE = 4;
    const DISCRIMINATOR_SIZE = 8;
    const size =
      params.size * HISTORY_ROW_SIZE + INSERT_IDX_SIZE + DISCRIMINATOR_SIZE;
    return program.methods
      .aggregatorSetHistoryBuffer({})
      .accounts({
        aggregator: this.publicKey,
        authority: authority.publicKey,
        buffer: buffer.publicKey,
      })
      .signers([authority, buffer])
      .preInstructions([
        anchor.web3.SystemProgram.createAccount({
          fromPubkey: programWallet(program).publicKey,
          newAccountPubkey: buffer.publicKey,
          space: size,
          lamports:
            await program.provider.connection.getMinimumBalanceForRentExemption(
              size
            ),
          programId: program.programId,
        }),
      ])
      .rpc();
  }

  async setResolutionMode(params: {
    authority: Keypair;
    mode: number;
  }): Promise<TransactionSignature> {
    const slidingWindow = anchor.utils.publicKey.findProgramAddressSync(
      [Buffer.from("SlidingResultAccountData"), this.publicKey.toBytes()],
      this.program.programId
    )[0];
    return this.program.methods
      .aggregatorSetResolutionMode({ mode: params.mode })
      .accounts({
        aggregator: this.publicKey,
        authority: params.authority.publicKey,
        slidingWindow,
        payer: programWallet(this.program).publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([params.authority])
      .rpc();
  }

  async setQueue(
    params: AggregatorSetQueueParams
  ): Promise<TransactionSignature> {
    const authority =
      params.authority ?? this.keypair ?? programWallet(this.program);
    return this.program.methods
      .aggregatorSetQueue({})
      .accounts({
        aggregator: this.publicKey,
        authority: authority.publicKey,
        queue: params.queueAccount.publicKey,
      })
      .signers([authority])
      .rpc();
  }

  /**
   * RPC to add a new job to an aggregtor to be performed on feed updates.
   * @param job JobAccount specifying another job for this aggregator to fulfill on update
   * @return TransactionSignature
   */
  async addJob(
    job: JobAccount,
    authority?: Keypair,
    weight = 1
  ): Promise<TransactionSignature> {
    authority = authority ?? this.keypair ?? programWallet(this.program);
    return this.program.methods
      .aggregatorAddJob({
        weight,
      })
      .accounts({
        aggregator: this.publicKey,
        authority: authority.publicKey,
        job: job.publicKey,
      })
      .signers([authority])
      .rpc();
  }

  /**
   * Prevent new jobs from being added to the feed.
   * @param authority The current authroity keypair
   * @return TransactionSignature
   */
  async lock(authority?: Keypair): Promise<TransactionSignature> {
    authority = authority ?? this.keypair ?? programWallet(this.program);
    return this.program.methods
      .aggregatorLock({})
      .accounts({
        aggregator: this.publicKey,
        authority: authority.publicKey,
      })
      .signers([authority])
      .rpc();
  }

  /**
   * Change the aggregator authority.
   * @param currentAuthority The current authroity keypair
   * @param newAuthority The new authority to set.
   * @return TransactionSignature
   */
  async setAuthority(
    newAuthority: PublicKey,
    currentAuthority?: Keypair
  ): Promise<TransactionSignature> {
    currentAuthority =
      currentAuthority ?? this.keypair ?? programWallet(this.program);
    return this.program.methods
      .aggregatorSetAuthority({})
      .accounts({
        aggregator: this.publicKey,
        newAuthority,
        authority: currentAuthority.publicKey,
      })
      .signers([currentAuthority])
      .rpc();
  }

  /**
   * RPC to remove a job from an aggregtor.
   * @param job JobAccount to be removed from the aggregator
   * @return TransactionSignature
   */
  async removeJob(
    job: JobAccount,
    authority?: Keypair
  ): Promise<TransactionSignature> {
    authority = authority ?? this.keypair ?? programWallet(this.program);
    return this.program.methods
      .aggregatorRemoveJob({})
      .accounts({
        aggregator: this.publicKey,
        authority: authority.publicKey,
        job: job.publicKey,
      })
      .signers([authority])
      .rpc();
  }

  /**
   * Opens a new round for the aggregator and will provide an incentivize reward
   * to the caller
   * @param params
   * @return TransactionSignature
   */
  async openRound(
    params: AggregatorOpenRoundParams
  ): Promise<TransactionSignature> {
    const [stateAccount, stateBump] = ProgramStateAccount.fromSeed(
      this.program
    );

    const [leaseAccount, leaseBump] = LeaseAccount.fromSeed(
      this.program,
      params.oracleQueueAccount,
      this
    );
    try {
      await leaseAccount.loadData();
    } catch (_) {
      throw new Error(
        "A requested lease pda account has not been initialized."
      );
    }

    const escrowPubkey = (await leaseAccount.loadData()).escrow;
    const queue = await params.oracleQueueAccount.loadData();
    const queueAuthority = queue.authority;

    const [permissionAccount, permissionBump] = PermissionAccount.fromSeed(
      this.program,
      queueAuthority,
      params.oracleQueueAccount.publicKey,
      this.publicKey
    );
    try {
      await permissionAccount.loadData();
    } catch (_) {
      throw new Error(
        "A requested permission pda account has not been initialized."
      );
    }

    return this.program.methods
      .aggregatorOpenRound({
        stateBump,
        leaseBump,
        permissionBump,
      })
      .accounts({
        aggregator: this.publicKey,
        lease: leaseAccount.publicKey,
        oracleQueue: params.oracleQueueAccount.publicKey,
        queueAuthority,
        permission: permissionAccount.publicKey,
        escrow: escrowPubkey,
        programState: stateAccount.publicKey,
        payoutWallet: params.payoutWallet,
        tokenProgram: spl.TOKEN_PROGRAM_ID,
        dataBuffer: queue.dataBuffer,
        mint: (await params.oracleQueueAccount.loadMint()).address,
      })
      .rpc();
  }

  async getOracleIndex(oraclePubkey: PublicKey): Promise<number> {
    const aggregator = await this.loadData();
    for (let i = 0; i < aggregator.oracleRequestBatchSize; i++) {
      if (aggregator.currentRound.oraclePubkeysData[i].equals(oraclePubkey)) {
        return i;
      }
    }
    return -1;
  }

  async saveResult(
    aggregator: any,
    oracleAccount: OracleAccount,
    params: AggregatorSaveResultParams
  ): Promise<TransactionSignature> {
    return (
      await this.program.provider.sendAll([
        {
          tx: await this.saveResultTxn(aggregator, oracleAccount, params),
          signers: [programWallet(this.program)],
        },
      ])
    )[0];
  }

  /**
   * RPC for an oracle to save a result to an aggregator round.
   * @param oracleAccount The oracle account submitting a result.
   * @param params
   * @return TransactionSignature
   */
  async saveResultTxn(
    aggregator: any,
    oracleAccount: OracleAccount, // TODO: move to params.
    params: AggregatorSaveResultParams
  ): Promise<Transaction> {
    let oracles = params.oracles ?? [];
    if (oracles.length === 0) {
      oracles = await this.loadCurrentRoundOracles(aggregator);
    }
    const payerKeypair = programWallet(this.program);
    const remainingAccounts: Array<PublicKey> = [];
    for (let i = 0; i < aggregator.oracleRequestBatchSize; ++i) {
      remainingAccounts.push(aggregator.currentRound.oraclePubkeysData[i]);
    }
    for (const oracle of oracles) {
      remainingAccounts.push(oracle.tokenAccount);
    }
    const queuePubkey = aggregator.queuePubkey;
    const queueAccount = new OracleQueueAccount({
      program: this.program,
      publicKey: queuePubkey,
    });
    const [leaseAccount, leaseBump] = LeaseAccount.fromSeed(
      this.program,
      queueAccount,
      this
    );
    const escrow = await spl.getAssociatedTokenAddress(
      params.tokenMint,
      leaseAccount.publicKey,
      true
    );
    const [feedPermissionAccount, feedPermissionBump] =
      PermissionAccount.fromSeed(
        this.program,
        params.queueAuthority,
        queueAccount.publicKey,
        this.publicKey
      );
    const [oraclePermissionAccount, oraclePermissionBump] =
      PermissionAccount.fromSeed(
        this.program,
        params.queueAuthority,
        queueAccount.publicKey,
        oracleAccount.publicKey
      );
    const [programStateAccount, stateBump] = ProgramStateAccount.fromSeed(
      this.program
    );
    const digest = this.produceJobsHash(params.jobs).digest();
    let historyBuffer = aggregator.historyBuffer;
    if (historyBuffer.equals(PublicKey.default)) {
      historyBuffer = this.publicKey;
    }

    remainingAccounts.push(
      anchor.utils.publicKey.findProgramAddressSync(
        [Buffer.from("SlidingResultAccountData"), this.publicKey.toBytes()],
        this.program.programId
      )[0]
    );
    return this.program.methods
      .aggregatorSaveResultV2({
        oracleIdx: params.oracleIdx,
        error: params.error,
        value: SwitchboardDecimal.fromBig(params.value),
        jobsChecksum: digest,
        minResponse: SwitchboardDecimal.fromBig(params.minResponse),
        maxResponse: SwitchboardDecimal.fromBig(params.maxResponse),
        feedPermissionBump,
        oraclePermissionBump,
        leaseBump,
        stateBump,
        jobValues: params.jobValues,
      })
      .accounts({
        aggregator: this.publicKey,
        oracle: oracleAccount.publicKey,
        oracleAuthority: payerKeypair.publicKey,
        oracleQueue: queueAccount.publicKey,
        queueAuthority: params.queueAuthority,
        feedPermission: feedPermissionAccount.publicKey,
        oraclePermission: oraclePermissionAccount.publicKey,
        lease: leaseAccount.publicKey,
        escrow,
        tokenProgram: spl.TOKEN_PROGRAM_ID,
        programState: programStateAccount.publicKey,
        historyBuffer,
        mint: params.tokenMint,
      })
      .remainingAccounts(
        remainingAccounts.map((pubkey: PublicKey): AccountMeta => {
          return { isSigner: false, isWritable: true, pubkey };
        })
      )
      .transaction();
  }
}

/**
 * Parameters for initializing JobAccount
 */
export interface JobInitParams {
  /**
   *  An optional name to apply to the job account.
   */
  name?: Buffer;
  /**
   *  unix_timestamp of when funds can be withdrawn from this account.
   */
  expiration?: anchor.BN;
  /**
   *  A serialized protocol buffer holding the schema of the job.
   */
  data: Buffer;
  /**
   *  A required variables oracles must fill to complete the job.
   */
  variables?: Array<string>;
  /**
   *  A pre-generated keypair to use.
   */
  keypair?: Keypair;
  authority: PublicKey;
}

/**
 * A Switchboard account representing a job for an oracle to perform, stored as
 * a protocol buffer.
 */
export class JobAccount {
  static accountName = "JobAccountData";

  program: SwitchboardProgram;

  publicKey: PublicKey;

  keypair?: Keypair;

  /**
   * JobAccount constructor
   * @param params initialization params.
   */
  public constructor(params: AccountParams) {
    if (params.keypair === undefined && params.publicKey === undefined) {
      throw new Error(
        `${this.constructor.name}: User must provide either a publicKey or keypair for account use.`
      );
    }
    if (params.keypair !== undefined && params.publicKey !== undefined) {
      if (!params.publicKey.equals(params.keypair.publicKey)) {
        throw new Error(
          `${this.constructor.name}: provided pubkey and keypair mismatch.`
        );
      }
    }
    this.program = params.program;
    this.keypair = params.keypair;
    this.publicKey = params.publicKey ?? this.keypair.publicKey;
  }

  /**
   * Load and parse JobAccount data based on the program IDL.
   * @return JobAccount data parsed in accordance with the
   * Switchboard IDL.
   */
  async loadData(): Promise<any> {
    const job = await this.program.account.jobAccountData.fetch(this.publicKey);
    return job;
  }

  /**
   * Load and parse the protobuf from the raw buffer stored in the JobAccount.
   * @return OracleJob
   */
  async loadJob(): Promise<OracleJob> {
    const job = await this.loadData();
    return OracleJob.decodeDelimited(job.data);
  }

  /**
   * Create and initialize the JobAccount.
   * @param program Switchboard program representation holding connection and IDL.
   * @param params.
   * @return newly generated JobAccount.
   */
  static async create(
    program: SwitchboardProgram,
    params: JobInitParams
  ): Promise<JobAccount> {
    const CHUNK_SIZE = 800;
    const payerKeypair = programWallet(program);
    const jobKeypair = params.keypair ?? anchor.web3.Keypair.generate();
    const [stateAccount, stateBump] = await ProgramStateAccount.getOrCreate(
      program,
      {}
    );
    const state = await stateAccount.loadData();

    if (params.data.byteLength <= CHUNK_SIZE) {
      await program.methods
        .jobInit({
          name: params.name ?? Buffer.from(""),
          expiration: params.expiration ?? new anchor.BN(0),
          stateBump,
          data: params.data,
          size: null,
        })
        .accounts({
          job: jobKeypair.publicKey,
          authority: params.authority,
          programState: stateAccount.publicKey,
          payer: payerKeypair.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([payerKeypair, jobKeypair])
        .rpc();
    } else {
      const chunks: Buffer[] = [];
      for (let i = 0; i < params.data.byteLength; ) {
        const end =
          i + CHUNK_SIZE >= params.data.byteLength
            ? params.data.byteLength
            : i + CHUNK_SIZE;
        chunks.push(params.data.slice(i, end));
        i = end;
      }

      const txns: string[] = [];

      txns.push(
        await program.methods
          .jobInit({
            name: [],
            expiration: new anchor.BN(0),
            stateBump,
            data: Buffer.from(""),
            size: params.data.byteLength,
          })
          .accounts({
            job: jobKeypair.publicKey,
            authority: payerKeypair.publicKey,
            programState: stateAccount.publicKey,
            payer: payerKeypair.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([payerKeypair, jobKeypair])
          .rpc()
      );

      for await (const [n, chunk] of chunks.entries()) {
        txns.push(
          await program.methods
            .jobSetData({
              data: chunk,
              size: params.data.byteLength,
              chunkIdx: n,
            })
            .accounts({
              job: jobKeypair.publicKey,
              authority: payerKeypair.publicKey,
            })
            .rpc()
        );
      }
    }

    return new JobAccount({ program, keypair: jobKeypair });
  }

  static decode(
    program: SwitchboardProgram,
    accountInfo: AccountInfo<Buffer>
  ): any {
    const coder = new anchor.BorshAccountsCoder(program.idl);
    const data = coder.decode(JobAccount.accountName, accountInfo?.data!);
    return data;
  }

  static decodeJob(
    program: SwitchboardProgram,
    accountInfo: AccountInfo<Buffer>
  ): OracleJob {
    return OracleJob.decodeDelimited(
      JobAccount.decode(program, accountInfo).data!
    );
  }
}

/**
 * Parameters for initializing PermissionAccount
 */
export interface PermissionInitParams {
  /**
   *  Pubkey of the account granting the permission.
   */
  granter: PublicKey;
  /**
   *  The receiving account of a permission.
   */
  grantee: PublicKey;
  /**
   *  The authority that is allowed to set permissions for this account.
   */
  authority: PublicKey;
}

/**
 * Parameters for setting a permission in a PermissionAccount
 */
export interface PermissionSetParams {
  /**
   *  The permssion to set
   */
  permission: SwitchboardPermission;
  /**
   *  The authority controlling this permission.
   */
  //authority: Keypair | PublicKey;
  authority: Keypair | PublicKey;
  /**
   *  Specifies whether to enable or disable the permission.
   */
  enable: boolean;
}

export interface PermissionSetVoterWeightParams {
  govProgram: PublicKey;
  pubkeySigner?: PublicKey;
  addinProgram: SwitchboardProgram;
  realm: PublicKey;
}

/**
 * An enum representing all known permission types for Switchboard.
 */
export enum SwitchboardPermission {
  PERMIT_ORACLE_HEARTBEAT = "permitOracleHeartbeat",
  PERMIT_ORACLE_QUEUE_USAGE = "permitOracleQueueUsage",
  PERMIT_VRF_REQUESTS = "permitVrfRequests",
}
export enum SwitchboardPermissionValue {
  PERMIT_ORACLE_HEARTBEAT = 1 << 0,
  PERMIT_ORACLE_QUEUE_USAGE = 1 << 1,
  PERMIT_VRF_REQUESTS = 1 << 2,
}
/**
 * A Switchboard account representing a permission or privilege granted by one
 * account signer to another account.
 */
export class PermissionAccount {
  static accountName = "PermissionAccountData";

  program: SwitchboardProgram;

  publicKey: PublicKey;

  keypair?: Keypair;

  /**
   * PermissionAccount constructor
   * @param params initialization params.
   */
  public constructor(params: AccountParams) {
    if (params.keypair === undefined && params.publicKey === undefined) {
      throw new Error(
        `${this.constructor.name}: User must provide either a publicKey or keypair for account use.`
      );
    }
    if (params.keypair !== undefined && params.publicKey !== undefined) {
      if (!params.publicKey.equals(params.keypair.publicKey)) {
        throw new Error(
          `${this.constructor.name}: provided pubkey and keypair mismatch.`
        );
      }
    }
    this.program = params.program;
    this.keypair = params.keypair;
    this.publicKey = params.publicKey ?? this.keypair.publicKey;
  }

  /**
   * Check if a specific permission is enabled on this permission account
   */
  async isPermissionEnabled(
    permission: SwitchboardPermissionValue
  ): Promise<boolean> {
    const permissions = (await this.loadData()).permissions;
    return (permissions & (permission as number)) != 0;
  }

  /**
   * Load and parse PermissionAccount data based on the program IDL.
   * @return PermissionAccount data parsed in accordance with the
   * Switchboard IDL.
   */
  async loadData(): Promise<any> {
    const permission: any =
      await this.program.account.permissionAccountData.fetch(this.publicKey);
    permission.ebuf = undefined;
    return permission;
  }

  /**
   * Get the size of a PermissionAccount on chain.
   * @return size.
   */
  size(): number {
    return this.program.account.permissionAccountData.size;
  }

  /**
   * Create and initialize the PermissionAccount.
   * @param program Switchboard program representation holding connection and IDL.
   * @param params.
   * @return newly generated PermissionAccount.
   */
  static async create(
    program: SwitchboardProgram,
    params: PermissionInitParams
  ): Promise<PermissionAccount> {
    const authorityInfo = await program.provider.connection.getAccountInfo(
      params.authority
    );

    const [permissionAccount, permissionBump] = PermissionAccount.fromSeed(
      program,
      params.authority,
      params.granter,
      params.grantee
    );
    const payerKeypair = programWallet(program);
    await program.methods
      .permissionInit({})
      .accounts({
        permission: permissionAccount.publicKey,
        authority: params.authority,
        granter: params.granter,
        grantee: params.grantee,
        payer: programWallet(program).publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([payerKeypair])
      .rpc();

    return new PermissionAccount({
      program,
      publicKey: permissionAccount.publicKey,
    });
  }

  /**
   * Loads a PermissionAccount from the expected PDA seed format.
   * @param authority The authority pubkey to be incorporated into the account seed.
   * @param granter The granter pubkey to be incorporated into the account seed.
   * @param grantee The grantee pubkey to be incorporated into the account seed.
   * @return PermissionAccount and PDA bump.
   */
  static fromSeed(
    program: SwitchboardProgram,
    authority: PublicKey,
    granter: PublicKey,
    grantee: PublicKey
  ): [PermissionAccount, number] {
    const [pubkey, bump] = anchor.utils.publicKey.findProgramAddressSync(
      [
        Buffer.from("PermissionAccountData"),
        authority.toBytes(),
        granter.toBytes(),
        grantee.toBytes(),
      ],
      program.programId
    );
    return [new PermissionAccount({ program, publicKey: pubkey }), bump];
  }

  /**
   * Sets the permission in the PermissionAccount
   * @param params.
   * @return TransactionSignature.
   */
  async set(params: PermissionSetParams): Promise<TransactionSignature> {
    if (!("publicKey" in params.authority)) {
      throw new Error(
        "Authority cannot be a PublicKey for the set RPC method."
      );
    }
    const permissionData = await this.loadData();
    const authorityInfo = await this.program.provider.connection.getAccountInfo(
      permissionData.authority
    );

    const permission = new Map<string, null>();
    permission.set(params.permission.toString(), null);
    return this.program.methods
      .permissionSet({
        permission: Object.fromEntries(permission),
        enable: params.enable,
      })
      .accounts({
        permission: this.publicKey,
        authority: params.authority.publicKey,
      })
      .signers([params.authority])
      .rpc();
  }

  /**
   * Sets the permission in the PermissionAccount
   * @param params.
   * @return TransactionSignature.
   */
  async setTx(params: PermissionSetParams): Promise<Transaction> {
    const permissionData = await this.loadData();

    let authPk: PublicKey;
    const signers: Array<Keypair> = [];
    if ("publicKey" in params.authority) {
      authPk = params.authority.publicKey;
      signers.push(params.authority as Keypair);
    } else {
      authPk = params.authority;
    }

    const authorityInfo = await this.program.provider.connection.getAccountInfo(
      permissionData.authority
    );

    const permission = new Map<string, null>();
    permission.set(params.permission.toString(), null);
    console.log("authority:");
    console.log(authPk);
    return this.program.methods
      .permissionSet({
        permission: Object.fromEntries(permission),
        enable: params.enable,
      })
      .accounts({
        permission: this.publicKey,
        authority: authPk,
      })
      .signers(signers)
      .transaction();
  }

  async setVoterWeightTx(params: PermissionSetVoterWeightParams) {
    const permissionData = await this.loadData();
    const oracleData = await this.program.account.oracleAccountData.fetch(
      permissionData.grantee
    );

    let payerKeypair;
    if (params.pubkeySigner == undefined) {
      payerKeypair = programWallet(this.program);
    }

    const [programStateAccount, stateBump] = ProgramStateAccount.fromSeed(
      this.program
    );
    const psData = await programStateAccount.loadData();

    // eslint-disable-next-line @typescript-eslint/naming-convention
    const [addinState, _] = await PublicKey.findProgramAddress(
      [Buffer.from("state")],
      params.addinProgram.programId
    );

    const [realmSpawnRecord] = anchor.utils.publicKey.findProgramAddressSync(
      [Buffer.from("RealmSpawnRecord"), params.realm.toBytes()],
      params.addinProgram.programId
    );

    const [voterWeightRecord] = anchor.utils.publicKey.findProgramAddressSync(
      [Buffer.from("VoterWeightRecord"), permissionData.grantee.toBytes()],
      params.addinProgram.programId
    );

    const [tokenOwnerRecord] = anchor.utils.publicKey.findProgramAddressSync(
      [
        Buffer.from("governance"),
        params.realm.toBytes(),
        psData.daoMint.toBytes(),
        (oracleData.oracleAuthority as PublicKey).toBytes(),
      ],
      params.govProgram
    );

    return params.addinProgram.methods
      .permissionSetVoterWeight()
      .accounts({
        permission: this.publicKey,
        permissionAuthority: permissionData.authority,
        oracle: permissionData.grantee,
        oracleAuthority: oracleData.oracleAuthority as PublicKey,
        payer: params.pubkeySigner,
        systemProgram: SystemProgram.programId,
        sbState: programStateAccount.publicKey,
        programState: addinState,
        govProgram: params.govProgram,
        daoMint: psData.daoMint,
        spawnRecord: realmSpawnRecord,
        voterWeight: voterWeightRecord,
        tokenOwnerRecord: tokenOwnerRecord,
        realm: params.realm,
      })
      .transaction();
  }
}

/**
 * Parameters for initializing OracleQueueAccount
 */
export interface OracleQueueInitParams {
  /**
   *  The account to delegate authority to for creating permissions targeted
   *  at the queue.
   */
  authority: PublicKey;
  /**
   *  A name to assign to this OracleQueue
   */
  name?: Buffer;
  /**
   *  Buffer for queue metadata
   */
  metadata?: Buffer;
  /**
   *  Rewards to provide oracles and round openers on this queue.
   */
  reward: anchor.BN;
  /**
   *  The minimum amount of stake oracles must present to remain on the queue.
   */
  minStake: anchor.BN;
  /**
   *  After a feed lease is funded or re-funded, it must consecutively succeed
   *  N amount of times or its authorization to use the queue is auto-revoked.
   */
  feedProbationPeriod?: number;
  /**
   *  Time period (in seconds) we should remove an oracle after if no response.
   */
  oracleTimeout?: number;
  /**
   *  The tolerated variance amount oracle results can have from the
   *  accepted round result before being slashed.
   *  slashBound = varianceToleranceMultiplier * stdDeviation
   *  Default: 2
   */
  varianceToleranceMultiplier?: number;
  /**
   *  Consecutive failure limit for a feed before feed permission is revoked.
   */
  consecutiveFeedFailureLimit?: anchor.BN;
  /**
   *  TODO: implement
   *  Consecutive failure limit for an oracle before oracle permission is revoked.
   */
  consecutiveOracleFailureLimit?: anchor.BN;
  /**
   * the minimum update delay time for Aggregators
   */
  minimumDelaySeconds?: number;
  /**
   * Optionally set the size of the queue.
   */
  queueSize?: number;
  /**
   *  Whether slashing is enabled on this queue.
   */
  slashingEnabled?: boolean;
  /**
   * Enabling this setting means data feeds do not need explicit permission
   * to join the queue.
   */
  unpermissionedFeeds?: boolean;
  /**
   * Enabling this setting means data feeds do not need explicit permission
   * to request VRF proofs and verifications from this queue.
   */
  unpermissionedVrf?: boolean;
  /**
   * Enabling this setting will allow buffer relayer accounts to call openRound.
   */
  enableBufferRelayers?: boolean;
  mint: PublicKey;
}

export type OracleQueueSetConfigParams = Partial<{
  name: Buffer;
  metadata: Buffer;
  unpermissionedFeedsEnabled: boolean;
  unpermissionedVrfEnabled: boolean;
  enableBufferRelayers: boolean;
  slashingEnabled: boolean;
  varianceToleranceMultiplier: number;
  oracleTimeout: number;
  reward: anchor.BN;
  minStake: anchor.BN;
  consecutiveFeedFailureLimit: anchor.BN;
  consecutiveOracleFailureLimit: anchor.BN;
}>;

/**
 * A Switchboard account representing a queue for distributing oracles to
 * permitted data feeds.
 */
export class OracleQueueAccount {
  static accountName = "OracleQueueAccountData";

  program: SwitchboardProgram;

  publicKey: PublicKey;

  keypair?: Keypair;

  /**
   * OracleQueueAccount constructor
   * @param params initialization params.
   */
  public constructor(params: AccountParams) {
    if (params.keypair === undefined && params.publicKey === undefined) {
      throw new Error(
        `${this.constructor.name}: User must provide either a publicKey or keypair for account use.`
      );
    }
    if (params.keypair !== undefined && params.publicKey !== undefined) {
      if (!params.publicKey.equals(params.keypair.publicKey)) {
        throw new Error(
          `${this.constructor.name}: provided pubkey and keypair mismatch.`
        );
      }
    }
    this.program = params.program;
    this.keypair = params.keypair;
    this.publicKey = params.publicKey ?? this.keypair.publicKey;
  }

  /**
   * Returns the queue's name buffer in a stringified format.
   * @param queue A preloaded queue object.
   * @return The name of the queue.
   */
  static getName = (queue: any) => toUtf8(queue.name);

  /**
   * Returns the queue's metadata buffer in a stringified format.
   * @param queue A preloaded queue object.
   * @return The stringified metadata of the queue.
   */
  static getMetadata = (queue: any) => toUtf8(queue.metadata);

  async loadMint(): Promise<spl.Mint> {
    const queue = await this.loadData();
    let mintKey = queue.mint ?? PublicKey.default;
    if (mintKey.equals(PublicKey.default)) {
      mintKey = spl.NATIVE_MINT;
    }
    return spl.getMint(this.program.provider.connection, mintKey);
  }

  /**
   * Load and parse OracleQueueAccount data based on the program IDL.
   * @return OracleQueueAccount data parsed in accordance with the
   * Switchboard IDL.
   */
  async loadData(): Promise<any> {
    const queue: any = await this.program.account.oracleQueueAccountData.fetch(
      this.publicKey
    );
    if (
      !("mint" in queue) ||
      queue.mint === undefined ||
      queue.mint === PublicKey.default
    ) {
      queue.mint = spl.NATIVE_MINT;
    }
    const queueData = [];
    const buffer =
      (
        await this.program.provider.connection.getAccountInfo(queue.dataBuffer)
      )?.data.slice(8) ?? Buffer.from("");
    const rowSize = 32;
    for (let i = 0; i < queue.size * rowSize; i += rowSize) {
      if (buffer.length - i < rowSize) {
        break;
      }
      const pubkeyBuf = buffer.slice(i, i + rowSize);
      const key = new PublicKey(pubkeyBuf);
      if (key === PublicKey.default) {
        break;
      }
      queueData.push(key);
    }
    queue.queue = queueData;
    queue.ebuf = undefined;
    return queue;
  }

  /**
   * Get the size of an OracleQueueAccount on chain.
   * @return size.
   */
  size(): number {
    return this.program.account.oracleQueueAccountData.size;
  }

  /**
   * Create and initialize the OracleQueueAccount.
   * @param program Switchboard program representation holding connection and IDL.
   * @param params.
   * @return newly generated OracleQueueAccount.
   */
  static async create(
    program: SwitchboardProgram,
    params: OracleQueueInitParams
  ): Promise<OracleQueueAccount> {
    const payerKeypair = programWallet(program);
    const [stateAccount, stateBump] = ProgramStateAccount.fromSeed(program);
    /*const mint = (await stateAccount.getTokenMint()).publicKey;*/
    const mint = params.mint;
    const oracleQueueAccount = anchor.web3.Keypair.generate();
    const buffer = anchor.web3.Keypair.generate();
    const size = program.account.oracleQueueAccountData.size;
    params.queueSize = params.queueSize ?? 500;
    const queueSize = params.queueSize * 32 + 8;
    await program.methods
      .oracleQueueInit({
        name: (params.name ?? Buffer.from("")).slice(0, 32),
        metadata: (params.metadata ?? Buffer.from("")).slice(0, 64),
        reward: params.reward ?? new anchor.BN(0),
        minStake: params.minStake ?? new anchor.BN(0),
        feedProbationPeriod: params.feedProbationPeriod ?? 0,
        oracleTimeout: params.oracleTimeout ?? 180,
        slashingEnabled: params.slashingEnabled ?? false,
        varianceToleranceMultiplier: SwitchboardDecimal.fromBig(
          new Big(params.varianceToleranceMultiplier ?? 2)
        ),
        authority: params.authority,
        consecutiveFeedFailureLimit:
          params.consecutiveFeedFailureLimit ?? new anchor.BN(1000),
        consecutiveOracleFailureLimit:
          params.consecutiveOracleFailureLimit ?? new anchor.BN(1000),
        minimumDelaySeconds: params.minimumDelaySeconds ?? 5,
        queueSize: params.queueSize,
        unpermissionedFeeds: params.unpermissionedFeeds ?? false,
        enableBufferRelayers: params.enableBufferRelayers ?? false,
      })
      .accounts({
        oracleQueue: oracleQueueAccount.publicKey,
        authority: params.authority,
        buffer: buffer.publicKey,
        systemProgram: SystemProgram.programId,
        payer: programWallet(program).publicKey,
        mint,
      })
      .signers([oracleQueueAccount, buffer])
      .preInstructions([
        anchor.web3.SystemProgram.createAccount({
          fromPubkey: programWallet(program).publicKey,
          newAccountPubkey: buffer.publicKey,
          space: queueSize,
          lamports:
            await program.provider.connection.getMinimumBalanceForRentExemption(
              queueSize
            ),
          programId: program.programId,
        }),
      ])
      .rpc();

    return new OracleQueueAccount({ program, keypair: oracleQueueAccount });
  }

  async setConfigTxn(
    params: OracleQueueSetConfigParams & { authority?: Keypair }
  ): Promise<anchor.web3.Transaction> {
    const program = this.program;
    const authority =
      params.authority ?? this.keypair ?? programWallet(this.program);
    return program.methods
      .oracleQueueSetConfig({
        name: params.name ?? null,
        metadata: params.metadata ?? null,
        unpermissionedFeedsEnabled: params.unpermissionedFeedsEnabled ?? null,
        unpermissionedVrfEnabled: params.unpermissionedVrfEnabled ?? null,
        enableBufferRelayers: params.enableBufferRelayers ?? null,
        slashingEnabled: params.slashingEnabled ?? null,
        reward: params.reward ?? null,
        minStake: params.minStake ?? null,
        oracleTimeout: params.oracleTimeout ?? null,
        consecutiveFeedFailureLimit: params.consecutiveFeedFailureLimit ?? null,
        consecutiveOracleFailureLimit:
          params.consecutiveOracleFailureLimit ?? null,
        varianceToleranceMultiplier: lodash.isFinite(
          params.varianceToleranceMultiplier
        )
          ? SwitchboardDecimal.fromBig(
              new Big(params.varianceToleranceMultiplier)
            )
          : null,
      })
      .accounts({ queue: this.publicKey, authority: authority.publicKey })
      .signers([authority])
      .transaction();
  }

  async setConfig(
    params: OracleQueueSetConfigParams & { authority?: Keypair }
  ): Promise<TransactionSignature> {
    const provider = this.program.provider as anchor.AnchorProvider;
    const authority =
      params.authority ?? this.keypair ?? programWallet(this.program);
    return provider.sendAndConfirm(await this.setConfigTxn(params), [
      authority,
    ]);
  }
}

/**
 * Parameters for initializing a LeaseAccount
 */
export interface LeaseInitParams {
  /**
   *  Token amount to load into the lease escrow
   */
  loadAmount: anchor.BN;
  /**
   *  The funding wallet of the lease.
   */
  funder: PublicKey;
  /**
   *  The authority of the funding wallet
   */
  funderAuthority: Keypair;
  /**
   *  The target to which this lease is applied.
   */
  oracleQueueAccount: OracleQueueAccount;
  /**
   *  The feed which the lease grants permission.
   */
  aggregatorAccount: AggregatorAccount;
  /**
   *  This authority will be permitted to withdraw funds from this lease.
   */
  withdrawAuthority?: PublicKey;
}

/**
 * Parameters for extending a LeaseAccount
 */
export interface LeaseExtendParams {
  /**
   *  Token amount to load into the lease escrow
   */
  loadAmount: anchor.BN;
  /**
   *  The funding wallet of the lease.
   */
  funder: PublicKey;
  /**
   *  The authority of the funding wallet
   */
  funderAuthority: Keypair;
}

/**
 * Parameters for withdrawing from a LeaseAccount
 */
export interface LeaseWithdrawParams {
  /**
   *  Token amount to withdraw from the lease escrow
   */
  amount: anchor.BN;
  /**
   *  The wallet to withdraw to.
   */
  withdrawWallet: PublicKey;
  /**
   *  The withdraw authority of the lease
   */
  withdrawAuthority: Keypair;
}

/**
 * A Switchboard account representing a lease for managing funds for oracle payouts
 * for fulfilling feed updates.
 */
export class LeaseAccount {
  static accountName = "LeaseAccountData";

  program: SwitchboardProgram;

  publicKey: PublicKey;

  keypair?: Keypair;

  /**
   * LeaseAccount constructor
   * @param params initialization params.
   */
  public constructor(params: AccountParams) {
    if (params.keypair === undefined && params.publicKey === undefined) {
      throw new Error(
        `${this.constructor.name}: User must provide either a publicKey or keypair for account use.`
      );
    }
    if (params.keypair !== undefined && params.publicKey !== undefined) {
      if (!params.publicKey.equals(params.keypair.publicKey)) {
        throw new Error(
          `${this.constructor.name}: provided pubkey and keypair mismatch.`
        );
      }
    }
    this.program = params.program;
    this.keypair = params.keypair;
    this.publicKey = params.publicKey ?? this.keypair.publicKey;
  }

  /**
   * Loads a LeaseAccount from the expected PDA seed format.
   * @param leaser The leaser pubkey to be incorporated into the account seed.
   * @param target The target pubkey to be incorporated into the account seed.
   * @return LeaseAccount and PDA bump.
   */
  static fromSeed(
    program: SwitchboardProgram,
    queueAccount: OracleQueueAccount,
    aggregatorAccount: AggregatorAccount
  ): [LeaseAccount, number] {
    const [pubkey, bump] = anchor.utils.publicKey.findProgramAddressSync(
      [
        Buffer.from("LeaseAccountData"),
        queueAccount.publicKey.toBytes(),
        aggregatorAccount.publicKey.toBytes(),
      ],
      program.programId
    );
    return [new LeaseAccount({ program, publicKey: pubkey }), bump];
  }

  /**
   * Load and parse LeaseAccount data based on the program IDL.
   * @return LeaseAccount data parsed in accordance with the
   * Switchboard IDL.
   */
  async loadData(): Promise<any> {
    const lease: any = await this.program.account.leaseAccountData.fetch(
      this.publicKey
    );
    lease.ebuf = undefined;
    return lease;
  }

  /**
   * Get the size of a LeaseAccount on chain.
   * @return size.
   */
  size(): number {
    return this.program.account.leaseAccountData.size;
  }

  /**
   * Create and initialize the LeaseAccount.
   * @param program Switchboard program representation holding connection and IDL.
   * @param params.
   * @return newly generated LeaseAccount.
   */
  static async create(
    program: SwitchboardProgram,
    params: LeaseInitParams
  ): Promise<LeaseAccount> {
    const payerKeypair = programWallet(program);
    const [programStateAccount, stateBump] =
      ProgramStateAccount.fromSeed(program);
    const switchTokenMint = await params.oracleQueueAccount.loadMint();
    const [leaseAccount, leaseBump] = LeaseAccount.fromSeed(
      program,
      params.oracleQueueAccount,
      params.aggregatorAccount
    );
    const escrow = await spl.getAssociatedTokenAddress(
      switchTokenMint.address,
      leaseAccount.publicKey,
      true
    );

    const jobAccountDatas = await params.aggregatorAccount.loadJobAccounts();
    const aggregatorData = await params.aggregatorAccount.loadData();
    const jobPubkeys = aggregatorData.jobPubkeysData.slice(
      0,
      aggregatorData.jobPubkeysSize
    );
    const jobWallets: Array<PublicKey> = [];
    const walletBumps: Array<number> = [];
    for (const idx in jobAccountDatas) {
      const jobAccountData = jobAccountDatas[idx];
      const authority = jobAccountData.authority ?? PublicKey.default;
      const [jobWallet, bump] = await PublicKey.findProgramAddress(
        [
          authority.toBuffer(),
          spl.TOKEN_PROGRAM_ID.toBuffer(),
          switchTokenMint.address.toBuffer(),
        ],
        spl.ASSOCIATED_TOKEN_PROGRAM_ID
      );
      jobWallets.push(jobWallet);
      walletBumps.push(bump);
    }

    await program.methods
      .leaseInit({
        loadAmount: params.loadAmount,
        stateBump,
        leaseBump,
        withdrawAuthority: params.withdrawAuthority ?? PublicKey.default,
        walletBumps: Buffer.from(walletBumps),
      })
      .accounts({
        programState: programStateAccount.publicKey,
        lease: leaseAccount.publicKey,
        queue: params.oracleQueueAccount.publicKey,
        aggregator: params.aggregatorAccount.publicKey,
        systemProgram: SystemProgram.programId,
        funder: params.funder,
        payer: programWallet(program).publicKey,
        tokenProgram: spl.TOKEN_PROGRAM_ID,
        escrow,
        owner: params.funderAuthority.publicKey,
        mint: switchTokenMint.address,
      })
      .preInstructions([
        spl.createAssociatedTokenAccountInstruction(
          payerKeypair.publicKey,
          escrow,
          leaseAccount.publicKey,
          switchTokenMint.address
        ),
      ])
      .signers([params.funderAuthority])
      .remainingAccounts(
        jobPubkeys.concat(jobWallets).map((pubkey: PublicKey) => {
          return { isSigner: false, isWritable: true, pubkey };
        })
      )
      .rpc();

    return new LeaseAccount({ program, publicKey: leaseAccount.publicKey });
  }

  async getBalance(): Promise<number> {
    const lease = await this.loadData();
    const escrow = await spl.getAccount(
      this.program.provider.connection,
      lease.escrow
    );
    return Number(escrow.amount);
  }

  /**
   * Estimate the time remaining on a given lease
   * @params void
   * @returns number milliseconds left in lease (estimate)
   */
  async estimatedLeaseTimeRemaining(): Promise<number> {
    // get lease data for escrow + aggregator pubkeys
    const lease = await this.loadData();
    const aggregatorAccount = new AggregatorAccount({
      program: this.program,
      publicKey: lease.aggregator,
    });
    // get aggregator data for minUpdateDelaySeconds + batchSize + queue pubkey
    const aggregator = await aggregatorAccount.loadData();
    const queueAccount = new OracleQueueAccount({
      program: this.program,
      publicKey: aggregator.queuePubkey,
    });
    const queue = await queueAccount.loadData();
    const batchSize = aggregator.oracleRequestBatchSize + 1;
    const minUpdateDelaySeconds = aggregator.minUpdateDelaySeconds * 1.5; // account for jitters with * 1.5
    const updatesPerDay = (60 * 60 * 24) / minUpdateDelaySeconds;
    const costPerDay = batchSize * queue.reward * updatesPerDay;
    const oneDay = 24 * 60 * 60 * 1000; // ms in a day
    const balance = await this.getBalance();
    const endDate = new Date();
    endDate.setTime(endDate.getTime() + (balance * oneDay) / costPerDay);
    const timeLeft = endDate.getTime() - new Date().getTime();
    return timeLeft;
  }

  /**
   * Adds fund to a LeaseAccount. Note that funds can always be withdrawn by
   * the withdraw authority if one was set on lease initialization.
   * @param program Switchboard program representation holding connection and IDL.
   * @param params.
   */
  async extend(params: LeaseExtendParams): Promise<TransactionSignature> {
    const program = this.program;
    const lease = await this.loadData();
    const escrow = lease.escrow;
    const queue = lease.queue;
    const queueAccount = new OracleQueueAccount({ program, publicKey: queue });
    const aggregator = lease.aggregator;
    const aggregatorAccount = new AggregatorAccount({
      program,
      publicKey: aggregator,
    });
    const [programStateAccount, stateBump] =
      ProgramStateAccount.fromSeed(program);
    const switchTokenMint = await queueAccount.loadMint();

    const [leaseAccount, leaseBump] = LeaseAccount.fromSeed(
      program,
      queueAccount,
      aggregatorAccount
    );
    const aggregatorData = await aggregatorAccount.loadData();
    const jobPubkeys = aggregatorData.jobPubkeysData.slice(
      0,
      aggregatorData.jobPubkeysSize
    );
    const jobAccountDatas = await aggregatorAccount.loadJobAccounts();
    const jobWallets: Array<PublicKey> = [];
    const walletBumps: Array<number> = [];
    for (const idx in jobAccountDatas) {
      const jobAccountData = jobAccountDatas[idx];
      const authority = jobAccountData.authority ?? PublicKey.default;
      const [jobWallet, bump] = await PublicKey.findProgramAddress(
        [
          authority.toBuffer(),
          spl.TOKEN_PROGRAM_ID.toBuffer(),
          switchTokenMint.address.toBuffer(),
        ],
        spl.ASSOCIATED_TOKEN_PROGRAM_ID
      );
      jobWallets.push(jobWallet);
      walletBumps.push(bump);
    }
    return program.methods
      .leaseExtend({
        loadAmount: params.loadAmount,
        stateBump,
        leaseBump,
        walletBumps: Buffer.from(walletBumps),
      })
      .accounts({
        lease: leaseAccount.publicKey,
        aggregator,
        queue,
        funder: params.funder,
        owner: params.funderAuthority.publicKey,
        tokenProgram: spl.TOKEN_PROGRAM_ID,
        escrow,
        programState: programStateAccount.publicKey,
        mint: (await queueAccount.loadMint()).address,
      })
      .signers([params.funderAuthority])
      .remainingAccounts(
        jobPubkeys.concat(jobWallets).map((pubkey: PublicKey) => {
          return { isSigner: false, isWritable: true, pubkey };
        })
      )
      .rpc();
  }

  /**
   * Withdraw funds from a LeaseAccount.
   * @param program Switchboard program representation holding connection and IDL.
   * @param params.
   */
  async withdraw(params: LeaseWithdrawParams): Promise<TransactionSignature> {
    const program = this.program;
    const lease = await this.loadData();
    const escrow = lease.escrow;
    const queue = lease.queue;
    const queueAccount = new OracleQueueAccount({ program, publicKey: queue });
    const aggregator = lease.aggregator;
    const [programStateAccount, stateBump] =
      ProgramStateAccount.fromSeed(program);
    const switchTokenMint = await queueAccount.loadMint();
    const [leaseAccount, leaseBump] = LeaseAccount.fromSeed(
      program,
      queueAccount,
      new AggregatorAccount({ program, publicKey: aggregator })
    );
    return program.methods
      .leaseWithdraw({
        amount: params.amount,
        stateBump,
        leaseBump,
      })
      .accounts({
        lease: leaseAccount.publicKey,
        escrow,
        aggregator,
        queue,
        withdrawAuthority: params.withdrawAuthority.publicKey,
        withdrawAccount: params.withdrawWallet,
        tokenProgram: spl.TOKEN_PROGRAM_ID,
        programState: programStateAccount.publicKey,
        mint: (await queueAccount.loadMint()).address,
      })
      .signers([params.withdrawAuthority])
      .rpc();
  }
}

/**
 * Parameters for initializing a CrankAccount
 */
export interface CrankInitParams {
  /**
   *  Buffer specifying crank name
   */
  name?: Buffer;
  /**
   *  Buffer specifying crank metadata
   */
  metadata?: Buffer;
  /**
   *  OracleQueueAccount for which this crank is associated
   */
  queueAccount: OracleQueueAccount;
  /**
   * Optional max number of rows
   */
  maxRows?: number;
}

/**
 * Parameters for popping an element from a CrankAccount.
 */
export interface CrankPopParams {
  /**
   * Specifies the wallet to reward for turning the crank.
   */
  payoutWallet: PublicKey;
  /**
   * The pubkey of the linked oracle queue.
   */
  queuePubkey: PublicKey;
  /**
   * The pubkey of the linked oracle queue authority.
   */
  queueAuthority: PublicKey;
  /**
   * Array of pubkeys to attempt to pop. If discluded, this will be loaded
   * from the crank upon calling.
   */
  readyPubkeys?: Array<PublicKey>;
  /**
   * Nonce to allow consecutive crank pops with the same blockhash.
   */
  nonce?: number;
  crank: any;
  queue: any;
  tokenMint: PublicKey;
  failOpenOnMismatch?: boolean;
  popIdx?: number;
}

/**
 * Parameters for pushing an element into a CrankAccount.
 */
export interface CrankPushParams {
  /**
   * Specifies the aggregator to push onto the crank.
   */
  aggregatorAccount: AggregatorAccount;
}

/**
 * Row structure of elements in the crank.
 */
export class CrankRow {
  /**
   *  Aggregator account pubkey
   */
  pubkey: PublicKey;

  /**
   *  Next aggregator update timestamp to order the crank by
   */
  nextTimestamp: anchor.BN;

  static from(buf: Buffer): CrankRow {
    const pubkey = new PublicKey(buf.slice(0, 32));
    const nextTimestamp = new anchor.BN(buf.slice(32, 40), "le");
    const res = new CrankRow();
    res.pubkey = pubkey;
    res.nextTimestamp = nextTimestamp;
    return res;
  }
}

/**
 * A Switchboard account representing a crank of aggregators ordered by next update time.
 */
export class CrankAccount {
  static accountName = "CrankAccountData";

  program: SwitchboardProgram;

  publicKey: PublicKey;

  keypair?: Keypair;

  /**
   * CrankAccount constructor
   * @param params initialization params.
   */
  public constructor(params: AccountParams) {
    if (params.keypair === undefined && params.publicKey === undefined) {
      throw new Error(
        `${this.constructor.name}: User must provide either a publicKey or keypair for account use.`
      );
    }
    if (params.keypair !== undefined && params.publicKey !== undefined) {
      if (!params.publicKey.equals(params.keypair.publicKey)) {
        throw new Error(
          `${this.constructor.name}: provided pubkey and keypair mismatch.`
        );
      }
    }
    this.program = params.program;
    this.keypair = params.keypair;
    this.publicKey = params.publicKey ?? this.keypair.publicKey;
  }

  /**
   * Load and parse CrankAccount data based on the program IDL.
   * @return CrankAccount data parsed in accordance with the
   * Switchboard IDL.
   */
  async loadData(): Promise<any> {
    const crank: any = await this.program.account.crankAccountData.fetch(
      this.publicKey
    );
    const pqData = [];
    const buffer =
      (
        await this.program.provider.connection.getAccountInfo(crank.dataBuffer)
      )?.data.slice(8) ?? Buffer.from("");
    const rowSize = 40;
    for (let i = 0; i < crank.pqSize * rowSize; i += rowSize) {
      if (buffer.length - i < rowSize) {
        break;
      }
      const rowBuf = buffer.slice(i, i + rowSize);
      pqData.push(CrankRow.from(rowBuf));
    }
    crank.pqData = pqData;
    crank.ebuf = undefined;
    return crank;
  }

  /**
   * Get the size of a CrankAccount on chain.
   * @return size.
   */
  size(): number {
    return this.program.account.crankAccountData.size;
  }

  /**
   * Create and initialize the CrankAccount.
   * @param program Switchboard program representation holding connection and IDL.
   * @param params.
   * @return newly generated CrankAccount.
   */
  static async create(
    program: SwitchboardProgram,
    params: CrankInitParams
  ): Promise<CrankAccount> {
    const payerKeypair = programWallet(program);
    const crankAccount = anchor.web3.Keypair.generate();
    const buffer = anchor.web3.Keypair.generate();
    const size = program.account.crankAccountData.size;
    params.maxRows = params.maxRows ?? 500;
    const crankSize = params.maxRows * 40 + 8;
    await program.methods
      .crankInit({
        name: (params.name ?? Buffer.from("")).slice(0, 32),
        metadata: (params.metadata ?? Buffer.from("")).slice(0, 64),
        crankSize: params.maxRows,
      })
      .accounts({
        crank: crankAccount.publicKey,
        queue: params.queueAccount.publicKey,
        buffer: buffer.publicKey,
        systemProgram: SystemProgram.programId,
        payer: programWallet(program).publicKey,
      })
      .signers([crankAccount, buffer])
      .preInstructions([
        anchor.web3.SystemProgram.createAccount({
          fromPubkey: programWallet(program).publicKey,
          newAccountPubkey: buffer.publicKey,
          space: crankSize,
          lamports:
            await program.provider.connection.getMinimumBalanceForRentExemption(
              crankSize
            ),
          programId: program.programId,
        }),
      ])
      .rpc();
    return new CrankAccount({ program, keypair: crankAccount });
  }

  /**
   * Pushes a new aggregator onto the crank.
   * @param aggregator The Aggregator account to push on the crank.
   * @return TransactionSignature
   */
  async push(params: CrankPushParams): Promise<TransactionSignature> {
    const aggregatorAccount: AggregatorAccount = params.aggregatorAccount;
    const crank = await this.loadData();
    const queueAccount = new OracleQueueAccount({
      program: this.program,
      publicKey: crank.queuePubkey,
    });
    const queue = await queueAccount.loadData();
    const queueAuthority = queue.authority;
    const [leaseAccount, leaseBump] = LeaseAccount.fromSeed(
      this.program,
      queueAccount,
      aggregatorAccount
    );
    let lease = null;
    try {
      lease = await leaseAccount.loadData();
    } catch (_) {
      throw new Error(
        "A requested lease pda account has not been initialized."
      );
    }

    const [permissionAccount, permissionBump] = PermissionAccount.fromSeed(
      this.program,
      queueAuthority,
      queueAccount.publicKey,
      aggregatorAccount.publicKey
    );
    try {
      await permissionAccount.loadData();
    } catch (_) {
      throw new Error(
        "A requested permission pda account has not been initialized."
      );
    }
    const [programStateAccount, stateBump] = ProgramStateAccount.fromSeed(
      this.program
    );
    return this.program.methods
      .crankPush({
        stateBump,
        permissionBump,
        nofitiRef: null,
        notifiRef: null,
      })
      .accounts({
        crank: this.publicKey,
        aggregator: aggregatorAccount.publicKey,
        oracleQueue: queueAccount.publicKey,
        queueAuthority,
        permission: permissionAccount.publicKey,
        lease: leaseAccount.publicKey,
        escrow: lease.escrow,
        programState: programStateAccount.publicKey,
        dataBuffer: crank.dataBuffer,
      })
      .rpc();
  }

  /**
   * Pops an aggregator from the crank.
   * @param params
   * @return TransactionSignature
   */
  async popTxn(params: CrankPopParams): Promise<Transaction> {
    const failOpenOnAccountMismatch = params.failOpenOnMismatch ?? false;
    const next = params.readyPubkeys ?? (await this.peakNextReady(5));
    if (next.length === 0) {
      throw new Error("Crank is not ready to be turned.");
    }
    const remainingAccounts: Array<PublicKey> = [];
    const leaseBumpsMap: Map<string, number> = new Map();
    const permissionBumpsMap: Map<string, number> = new Map();
    const queueAccount = new OracleQueueAccount({
      program: this.program,
      publicKey: params.queuePubkey,
    });

    for (const row of next) {
      const aggregatorAccount = new AggregatorAccount({
        program: this.program,
        publicKey: row,
      });
      const [leaseAccount, leaseBump] = LeaseAccount.fromSeed(
        this.program,
        queueAccount,
        aggregatorAccount
      );
      const [permissionAccount, permissionBump] = PermissionAccount.fromSeed(
        this.program,
        params.queueAuthority,
        params.queuePubkey,
        row
      );
      const escrow = await spl.getAssociatedTokenAddress(
        params.tokenMint,
        leaseAccount.publicKey,
        true
      );
      remainingAccounts.push(aggregatorAccount.publicKey);
      remainingAccounts.push(leaseAccount.publicKey);
      remainingAccounts.push(escrow);
      remainingAccounts.push(permissionAccount.publicKey);
      leaseBumpsMap.set(row.toBase58(), leaseBump);
      permissionBumpsMap.set(row.toBase58(), permissionBump);
    }
    remainingAccounts.sort((a: PublicKey, b: PublicKey) =>
      a.toBuffer().compare(b.toBuffer())
    );
    const crank = params.crank;
    const queue = params.queue;
    const leaseBumps: Array<number> = [];
    const permissionBumps: Array<number> = [];
    // Map bumps to the index of their corresponding feeds.
    for (const key of remainingAccounts) {
      leaseBumps.push(leaseBumpsMap.get(key.toBase58()) ?? 0);
      permissionBumps.push(permissionBumpsMap.get(key.toBase58()) ?? 0);
    }
    const [programStateAccount, stateBump] = ProgramStateAccount.fromSeed(
      this.program
    );
    const payerKeypair = programWallet(this.program);
    let mint: PublicKey = queue.mint;
    if (!mint || mint.equals(PublicKey.default)) {
      mint = spl.NATIVE_MINT;
    }
    // const promises: Array<Promise<TransactionSignature>> = [];
    return this.program.methods
      .crankPopV2({
        stateBump,
        leaseBumps: Buffer.from(leaseBumps),
        permissionBumps: Buffer.from(permissionBumps),
        nonce: params.nonce ?? null,
        failOpenOnAccountMismatch: failOpenOnAccountMismatch ?? true,
        popIdx: params.popIdx ?? 0,
      })
      .accounts({
        crank: this.publicKey,
        oracleQueue: params.queuePubkey,
        queueAuthority: params.queueAuthority,
        programState: programStateAccount.publicKey,
        payoutWallet: params.payoutWallet,
        tokenProgram: spl.TOKEN_PROGRAM_ID,
        crankDataBuffer: crank.dataBuffer,
        queueDataBuffer: queue.dataBuffer,
        mint,
      })
      .remainingAccounts(
        remainingAccounts.map((pubkey: PublicKey) => {
          return { isSigner: false, isWritable: true, pubkey };
        })
      )
      .signers([payerKeypair])
      .transaction();
  }

  /**
   * Pops an aggregator from the crank.
   * @param params
   * @return TransactionSignature
   */
  async pop(params: CrankPopParams): Promise<TransactionSignature> {
    const payerKeypair = programWallet(this.program);
    return sendAndConfirmTransaction(
      this.program.provider.connection,
      await this.popTxn(params),
      [payerKeypair]
    );
  }

  /**
   * Get an array of the next aggregator pubkeys to be popped from the crank, limited by n
   * @param n The limit of pubkeys to return.
   * @return Pubkey list of Aggregators and next timestamp to be popped, ordered by timestamp.
   */
  async peakNextWithTime(n: number): Promise<Array<CrankRow>> {
    const crank = await this.loadData();
    const items = crank.pqData
      .slice(0, crank.pqSize)
      .sort((a: CrankRow, b: CrankRow) => a.nextTimestamp.sub(b.nextTimestamp))
      .slice(0, n);
    return items;
  }

  /**
   * Get an array of the next readily updateable aggregator pubkeys to be popped
   * from the crank, limited by n
   * @param n The limit of pubkeys to return.
   * @return Pubkey list of Aggregator pubkeys.
   */
  async peakNextReady(n?: number): Promise<Array<PublicKey>> {
    const now = Math.floor(+new Date() / 1000);
    const crank = await this.loadData();
    n = n ?? crank.pqSize;
    const items = crank.pqData
      .slice(0, crank.pqSize)
      .filter((row: CrankRow) => now >= row.nextTimestamp.toNumber())
      .sort((a: CrankRow, b: CrankRow) => a.nextTimestamp.sub(b.nextTimestamp))
      .slice(0, n)
      .map((item: CrankRow) => item.pubkey);
    return items;
  }

  /**
   * Get an array of the next aggregator pubkeys to be popped from the crank, limited by n
   * @param n The limit of pubkeys to return.
   * @return Pubkey list of Aggregators next up to be popped.
   */
  async peakNext(n: number): Promise<Array<PublicKey>> {
    const crank = await this.loadData();
    const items = crank.pqData
      .slice(0, crank.pqSize)
      .sort((a: CrankRow, b: CrankRow) => a.nextTimestamp.sub(b.nextTimestamp))
      .map((item: CrankRow) => item.pubkey)
      .slice(0, n);
    return items;
  }
}

/**
 * Parameters for an OracleInit request.
 */
export interface OracleInitParams {
  /**
   *  Buffer specifying oracle name
   */
  name?: Buffer;
  /**
   *  Buffer specifying oracle metadata
   */
  metadata?: Buffer;
  /**
   * If included, this keypair will be the oracle authority.
   */
  oracleAuthority?: Keypair;
  /**
   * Specifies the oracle queue to associate with this OracleAccount.
   */
  queueAccount: OracleQueueAccount;
}

/**
 * Parameters for an OracleWithdraw request.
 */
export interface OracleWithdrawParams {
  /**
   *  Amount to withdraw
   */
  amount: anchor.BN;
  /**
   * Token Account to withdraw to
   */
  withdrawAccount: PublicKey;
  /**
   * Oracle authority keypair.
   */
  oracleAuthority: Keypair;
}

/**
 * A Switchboard account representing an oracle account and its associated queue
 * and escrow account.
 */
export class OracleAccount {
  static accountName = "OracleAccountData";

  program: SwitchboardProgram;

  publicKey: PublicKey;

  keypair?: Keypair;

  /**
   * OracleAccount constructor
   * @param params initialization params.
   */
  public constructor(params: AccountParams) {
    if (params.keypair === undefined && params.publicKey === undefined) {
      throw new Error(
        `${this.constructor.name}: User must provide either a publicKey or keypair for account use.`
      );
    }
    if (params.keypair !== undefined && params.publicKey !== undefined) {
      if (!params.publicKey.equals(params.keypair.publicKey)) {
        throw new Error(
          `${this.constructor.name}: provided pubkey and keypair mismatch.`
        );
      }
    }
    this.program = params.program;
    this.keypair = params.keypair;
    this.publicKey = params.publicKey ?? this.keypair.publicKey;
  }

  /**
   * Load and parse OracleAccount data based on the program IDL.
   * @return OracleAccount data parsed in accordance with the
   * Switchboard IDL.
   */
  async loadData(): Promise<any> {
    const item: any = await this.program.account.oracleAccountData.fetch(
      this.publicKey
    );
    item.ebuf = undefined;
    return item;
  }

  /**
   * Get the size of an OracleAccount on chain.
   * @return size.
   */
  size(): number {
    return this.program.account.oracleAccountData.size;
  }

  /**
   * Create and initialize the OracleAccount.
   * @param program Switchboard program representation holding connection and IDL.
   * @param params.
   * @return newly generated OracleAccount.
   */
  static async create(
    program: SwitchboardProgram,
    params: OracleInitParams
  ): Promise<OracleAccount> {
    const payerKeypair = programWallet(program);
    const authorityKeypair = params.oracleAuthority ?? payerKeypair;
    const size = program.account.oracleAccountData.size;
    const [programStateAccount, stateBump] =
      ProgramStateAccount.fromSeed(program);

    const mint = await params.queueAccount.loadMint();

    const walletKeypair = Keypair.generate();

    const [oracleAccount, oracleBump] = OracleAccount.fromSeed(
      program,
      params.queueAccount,
      walletKeypair.publicKey
    );

    const tokenRent =
      await program.provider.connection.getMinimumBalanceForRentExemption(
        spl.ACCOUNT_SIZE
      );

    await program.methods
      .oracleInit({
        name: (params.name ?? Buffer.from("")).slice(0, 32),
        metadata: (params.metadata ?? Buffer.from("")).slice(0, 128),
        stateBump,
        oracleBump,
      })
      .accounts({
        oracle: oracleAccount.publicKey,
        oracleAuthority: authorityKeypair.publicKey,
        queue: params.queueAccount.publicKey,
        wallet: walletKeypair.publicKey,
        programState: programStateAccount.publicKey,
        systemProgram: SystemProgram.programId,
        payer: programWallet(program).publicKey,
      })
      .preInstructions([
        SystemProgram.createAccount({
          fromPubkey: payerKeypair.publicKey,
          newAccountPubkey: walletKeypair.publicKey,
          space: spl.ACCOUNT_SIZE,
          lamports: tokenRent,
          programId: spl.TOKEN_PROGRAM_ID,
        }),
        spl.createInitializeAccountInstruction(
          walletKeypair.publicKey,
          mint.address,
          programWallet(program).publicKey
        ),
        spl.createSetAuthorityInstruction(
          walletKeypair.publicKey,
          programWallet(program).publicKey,
          spl.AuthorityType.AccountOwner,
          programStateAccount.publicKey,
          [programWallet(program), walletKeypair],
          spl.TOKEN_PROGRAM_ID
        ),
      ])
      .signers([walletKeypair])
      .rpc();

    return new OracleAccount({ program, publicKey: oracleAccount.publicKey });
  }

  static decode(
    program: SwitchboardProgram,
    accountInfo: AccountInfo<Buffer>
  ): any {
    const coder = new anchor.BorshAccountsCoder(program.idl);
    const key = "OracleAccountData";
    const data = coder.decode(key, accountInfo?.data!);
    return data;
  }

  /**
   * Constructs OracleAccount from the static seed from which it was generated.
   * @return OracleAccount and PDA bump tuple.
   */
  static fromSeed(
    program: SwitchboardProgram,
    queueAccount: OracleQueueAccount,
    wallet: PublicKey
  ): [OracleAccount, number] {
    const [oraclePubkey, oracleBump] =
      anchor.utils.publicKey.findProgramAddressSync(
        [
          Buffer.from("OracleAccountData"),
          queueAccount.publicKey.toBuffer(),
          wallet.toBuffer(),
        ],
        program.programId
      );
    return [
      new OracleAccount({ program, publicKey: oraclePubkey }),
      oracleBump,
    ];
  }

  /**
   * Inititates a heartbeat for an OracleAccount, signifying oracle is still healthy.
   * @return TransactionSignature.
   */
  async heartbeat(authority: Keypair): Promise<TransactionSignature> {
    const payerKeypair = programWallet(this.program);
    const queueAccount = new OracleQueueAccount({
      program: this.program,
      publicKey: (await this.loadData()).queuePubkey,
    });
    const queue = await queueAccount.loadData();
    let lastPubkey = this.publicKey;
    if (queue.size !== 0) {
      lastPubkey = queue.queue[queue.gcIdx];
    }
    const [permissionAccount, permissionBump] = PermissionAccount.fromSeed(
      this.program,
      queue.authority,
      queueAccount.publicKey,
      this.publicKey
    );
    try {
      await permissionAccount.loadData();
    } catch (_) {
      throw new Error(
        "A requested permission pda account has not been initialized."
      );
    }
    const oracle = await this.loadData();

    assert(this.publicKey !== undefined);
    assert(payerKeypair.publicKey !== undefined);
    assert(oracle.tokenAccount !== undefined);
    assert(lastPubkey !== undefined);
    assert(queueAccount.publicKey !== undefined);
    assert(queueAccount.publicKey !== undefined);
    assert(permissionAccount.publicKey !== undefined);
    assert(queue.dataBuffer !== undefined);

    return this.program.methods
      .oracleHeartbeat({
        permissionBump,
      })
      .accounts({
        oracle: this.publicKey,
        oracleAuthority: payerKeypair.publicKey,
        tokenAccount: oracle.tokenAccount,
        gcOracle: lastPubkey,
        oracleQueue: queueAccount.publicKey,
        permission: permissionAccount.publicKey,
        dataBuffer: queue.dataBuffer,
      })
      .signers([authority])
      .rpc();
  }

  /**
  /**
   * Inititates a heartbeat for an OracleAccount, signifying oracle is still healthy.
   * @return TransactionSignature.
   */
  async heartbeatTx(): Promise<Transaction> {
    const payerKeypair = programWallet(this.program);
    const queueAccount = new OracleQueueAccount({
      program: this.program,
      publicKey: (await this.loadData()).queuePubkey,
    });
    const queue = await queueAccount.loadData();
    let lastPubkey = this.publicKey;
    if (queue.size !== 0) {
      lastPubkey = queue.queue[queue.gcIdx];
    }
    const [permissionAccount, permissionBump] = PermissionAccount.fromSeed(
      this.program,
      queue.authority,
      queueAccount.publicKey,
      this.publicKey
    );
    try {
      await permissionAccount.loadData();
    } catch (_) {
      throw new Error(
        "A requested permission pda account has not been initialized."
      );
    }
    const oracle = await this.loadData();

    return this.program.methods
      .oracleHeartbeat({
        permissionBump,
      })
      .accounts({
        oracle: this.publicKey,
        oracleAuthority: payerKeypair.publicKey,
        tokenAccount: oracle.tokenAccount,
        gcOracle: lastPubkey,
        oracleQueue: queueAccount.publicKey,
        permission: permissionAccount.publicKey,
        dataBuffer: queue.dataBuffer,
      })
      .signers([this.keypair])
      .transaction();
  }

  /**
   * Withdraw stake and/or rewards from an OracleAccount.
   */
  async withdraw(params: OracleWithdrawParams): Promise<TransactionSignature> {
    const payerKeypair = programWallet(this.program);
    const oracle = await this.loadData();
    const queuePubkey = oracle.queuePubkey;
    const queueAccount = new OracleQueueAccount({
      program: this.program,
      publicKey: queuePubkey,
    });
    const queueAuthority = (await queueAccount.loadData()).authority;
    const [stateAccount, stateBump] = ProgramStateAccount.fromSeed(
      this.program
    );
    const [permissionAccount, permissionBump] = PermissionAccount.fromSeed(
      this.program,
      queueAuthority,
      queueAccount.publicKey,
      this.publicKey
    );

    return this.program.methods
      .oracleWithdraw({
        permissionBump,
        stateBump,
        amount: params.amount,
      })
      .accounts({
        oracle: this.publicKey,
        oracleAuthority: params.oracleAuthority.publicKey,
        tokenAccount: oracle.tokenAccount,
        withdrawAccount: params.withdrawAccount,
        oracleQueue: queueAccount.publicKey,
        permission: permissionAccount.publicKey,
        tokenProgram: spl.TOKEN_PROGRAM_ID,
        programState: stateAccount.publicKey,
        systemProgram: SystemProgram.programId,
        payer: programWallet(this.program).publicKey,
      })
      .signers([params.oracleAuthority])
      .rpc();
  }

  async getBalance(): Promise<number> {
    const oracle = await this.loadData();
    const escrow = await spl.getAccount(
      this.program.provider.connection,
      oracle.tokenAccount
    );
    return Number(escrow.amount);
  }
}

export interface Callback {
  programId: PublicKey;
  accounts: Array<AccountMeta>;
  ixData: Buffer;
}

/**
 * Parameters for a VrfInit request.
 */
export interface VrfInitParams {
  /**
   *  Vrf account authority to configure the account
   */
  authority: PublicKey;
  queue: OracleQueueAccount;
  callback: Callback;
  /**
   *  Keypair to use for the vrf account.
   */
  keypair: Keypair;
}
/**
 * Parameters for a VrfSetCallback request.
 */
export interface VrfSetCallbackParams {
  authority: Keypair;
  cpiProgramId: PublicKey;
  accountList: Array<AccountMeta>;
  instruction: Buffer;
}

export interface VrfProveAndVerifyParams {
  proof: Buffer;
  oracleAccount: OracleAccount;
  oracleAuthority: Keypair;
  skipPreflight: boolean;
}

export interface VrfRequestRandomnessParams {
  authority: Keypair;
  payer: PublicKey;
  payerAuthority: Keypair;
}

export interface VrfProveParams {
  proof: Buffer;
  oracleAccount: OracleAccount;
  oracleAuthority: Keypair;
}

/**
 * A Switchboard VRF account.
 */
export class VrfAccount {
  static accountName = "VrfAccountData";

  program: SwitchboardProgram;

  publicKey: PublicKey;

  keypair?: Keypair;

  /**
   * CrankAccount constructor
   * @param params initialization params.
   */
  public constructor(params: AccountParams) {
    if (params.keypair === undefined && params.publicKey === undefined) {
      throw new Error(
        `${this.constructor.name}: User must provide either a publicKey or keypair for account use.`
      );
    }
    if (params.keypair !== undefined && params.publicKey !== undefined) {
      if (!params.publicKey.equals(params.keypair.publicKey)) {
        throw new Error(
          `${this.constructor.name}: provided pubkey and keypair mismatch.`
        );
      }
    }
    this.program = params.program;
    this.keypair = params.keypair;
    this.publicKey = params.publicKey ?? this.keypair.publicKey;
  }

  /**
   * Load and parse VrfAccount data based on the program IDL.
   * @return VrfAccount data parsed in accordance with the
   * Switchboard IDL.
   */
  async loadData(): Promise<any> {
    const vrf: any = await this.program.account.vrfAccountData.fetch(
      this.publicKey
    );
    vrf.ebuf = undefined;
    vrf.builders = vrf.builders.slice(0, vrf.buildersLen);
    return vrf;
  }

  onChange(callback: OnAccountChangeCallback): number {
    const coder = new anchor.BorshAccountsCoder(this.program.idl);
    return this.program.provider.connection.onAccountChange(
      this.publicKey,
      (accountInfo, context) => {
        const vrf = coder.decode(VrfAccount.accountName, accountInfo?.data);
        callback(vrf);
      }
    );
  }

  /**
   * Get the size of a VrfAccount on chain.
   * @return size.
   */
  size(): number {
    return this.program.account.vrfAccountData.size;
  }

  /**
   * Create and initialize the VrfAccount.
   * @param program Switchboard program representation holding connection and IDL.
   * @param params.
   * @return newly generated VrfAccount.
   */
  static async create(
    program: SwitchboardProgram,
    params: VrfInitParams
  ): Promise<VrfAccount> {
    const payerKeypair = programWallet(program);
    const [programStateAccount, stateBump] =
      ProgramStateAccount.fromSeed(program);
    const keypair = params.keypair;
    const size = program.account.vrfAccountData.size;
    const switchTokenMint = await params.queue.loadMint();

    const escrow = await spl.getAssociatedTokenAddress(
      switchTokenMint.address,
      keypair.publicKey,
      true
    );

    await program.methods
      .vrfInit({
        stateBump,
        callback: params.callback,
      })
      .accounts({
        vrf: keypair.publicKey,
        escrow,
        authority: params.authority ?? keypair.publicKey,
        oracleQueue: params.queue.publicKey,
        programState: programStateAccount.publicKey,
        tokenProgram: spl.TOKEN_PROGRAM_ID,
      })
      .preInstructions([
        spl.createAssociatedTokenAccountInstruction(
          payerKeypair.publicKey,
          escrow,
          keypair.publicKey,
          switchTokenMint.address
        ),
        spl.createSetAuthorityInstruction(
          escrow,
          keypair.publicKey,
          spl.AuthorityType.AccountOwner,
          programStateAccount.publicKey,
          [payerKeypair, keypair]
        ),
        anchor.web3.SystemProgram.createAccount({
          fromPubkey: payerKeypair.publicKey,
          newAccountPubkey: keypair.publicKey,
          space: size,
          lamports:
            await program.provider.connection.getMinimumBalanceForRentExemption(
              size
            ),
          programId: program.programId,
        }),
      ])
      .signers([payerKeypair, keypair])
      .rpc();

    return new VrfAccount({ program, keypair, publicKey: keypair.publicKey });
  }

  /**
   * Trigger new randomness production on the vrf account
   */
  async requestRandomness(params: VrfRequestRandomnessParams) {
    const vrf = await this.loadData();
    const queueAccount = new OracleQueueAccount({
      program: this.program,
      publicKey: vrf.oracleQueue,
    });
    const queue = await queueAccount.loadData();
    const queueAuthority = queue.authority;
    const dataBuffer = queue.dataBuffer;
    const escrow = vrf.escrow;
    const payer = params.payer;
    const [stateAccount, stateBump] = ProgramStateAccount.fromSeed(
      this.program
    );
    const [permissionAccount, permissionBump] = PermissionAccount.fromSeed(
      this.program,
      queueAuthority,
      queueAccount.publicKey,
      this.publicKey
    );
    try {
      await permissionAccount.loadData();
    } catch (_) {
      throw new Error(
        "A requested permission pda account has not been initialized."
      );
    }
    const tokenProgram = spl.TOKEN_PROGRAM_ID;
    const recentBlockhashes = SYSVAR_RECENT_BLOCKHASHES_PUBKEY;
    await this.program.methods
      .vrfRequestRandomness({
        stateBump,
        permissionBump,
      })
      .accounts({
        authority: params.authority.publicKey,
        vrf: this.publicKey,
        oracleQueue: queueAccount.publicKey,
        queueAuthority,
        dataBuffer,
        permission: permissionAccount.publicKey,
        escrow,
        payerWallet: payer,
        payerAuthority: params.payerAuthority.publicKey,
        recentBlockhashes,
        programState: stateAccount.publicKey,
        tokenProgram,
      })
      .signers([params.authority, params.payerAuthority])
      .rpc();
  }

  async prove(params: VrfProveParams): Promise<TransactionSignature> {
    const vrf = await this.loadData();
    let idx = -1;
    let producerKey = PublicKey.default;
    for (idx = 0; idx < vrf.buildersLen; ++idx) {
      const builder = vrf.builders[idx];
      producerKey = builder.producer;
      if (producerKey.equals(params.oracleAccount.publicKey)) {
        break;
      }
    }
    if (idx === vrf.buildersLen) {
      throw new Error("OracleProofRequestNotFoundError");
    }
    return this.program.methods
      .vrfProve({
        proof: params.proof,
        idx,
      })
      .accounts({
        vrf: this.publicKey,
        oracle: producerKey,
        randomnessProducer: params.oracleAuthority.publicKey,
      })
      .signers([params.oracleAuthority])
      .rpc();
  }

  async verify(
    oracle: OracleAccount,
    tryCount = 278
  ): Promise<Array<TransactionSignature>> {
    const skipPreflight = true;
    const txs: Array<any> = [];
    const vrf = await this.loadData();
    const idx = vrf.builders.find((builder: any) =>
      oracle.publicKey.equals(builder.producer)
    );
    if (idx === -1) {
      throw new Error("OracleNotFoundError");
    }
    const counter = 0;
    const remainingAccounts = vrf.callback.accounts.slice(
      0,
      vrf.callback.accountsLen
    );
    const [programStateAccount, stateBump] = ProgramStateAccount.fromSeed(
      this.program
    );
    const oracleData = await oracle.loadData();
    const oracleWallet = oracleData.tokenAccount;
    const oracleAuthority: PublicKey = oracleData.oracleAuthority;

    const instructions = [];
    const tx = new Transaction();
    for (let i = 0; i < tryCount; ++i) {
      txs.push({
        tx: await this.program.methods
          .vrfProveAndVerify({
            nonce: i,
            stateBump,
            idx,
            proof: Buffer.from(""),
          })
          .accounts({
            vrf: this.publicKey,
            callbackPid: vrf.callback.programId,
            tokenProgram: spl.TOKEN_PROGRAM_ID,
            escrow: vrf.escrow,
            programState: programStateAccount.publicKey,
            oracle: oracle.publicKey,
            oracleAuthority,
            oracleWallet,
            instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
          })
          .remainingAccounts(remainingAccounts)
          .transaction(),
      });
      // try {
      // tx.add(newTx);
      // } catch (e) {
      // txs.push({ tx });
      // tx = newTx;
      // }
      // txs.push(newTx);
    }
    // txs.push({ tx });
    return sendAll(this.program.provider, txs, [], skipPreflight);
  }

  /**
   * Attempt the maximum amount of turns remaining on the vrf verify crank.
   * This will automatically call the vrf callback (if set) when completed.
   */
  async proveAndVerify(
    params: VrfProveAndVerifyParams,
    tryCount = 278
  ): Promise<Array<TransactionSignature>> {
    const skipPreflight = params.skipPreflight;
    const oracle = params.oracleAccount;
    const txs: Array<any> = [];
    const vrf = await this.loadData();
    const idx = vrf.builders.find((builder: any) =>
      oracle.publicKey.equals(builder.producer)
    );
    if (idx === -1) {
      throw new Error("OracleNotFoundError");
    }
    const counter = 0;
    const remainingAccounts = vrf.callback.accounts.slice(
      0,
      vrf.callback.accountsLen
    );
    const [programStateAccount, stateBump] = ProgramStateAccount.fromSeed(
      this.program
    );
    const oracleData = await oracle.loadData();
    const oracleWallet = oracleData.tokenAccount;
    const oracleAuthority: PublicKey = oracleData.oracleAuthority;

    const instructions = [];
    const tx = new Transaction();
    for (let i = 0; i < tryCount; ++i) {
      txs.push({
        tx: await this.program.methods
          .vrfProveAndVerify({
            nonce: i,
            stateBump,
            idx,
            proof: params.proof,
          })
          .accounts({
            vrf: this.publicKey,
            callbackPid: vrf.callback.programId,
            tokenProgram: spl.TOKEN_PROGRAM_ID,
            escrow: vrf.escrow,
            programState: programStateAccount.publicKey,
            oracle: oracle.publicKey,
            oracleAuthority,
            oracleWallet,
            instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
          })
          .remainingAccounts(remainingAccounts)
          .signers([params.oracleAuthority])
          .transaction(),
      });
      // try {
      // tx.add(newTx);
      // } catch (e) {
      // txs.push({ tx });
      // tx = newTx;
      // }
      // txs.push(newTx);
    }
    // txs.push({ tx });
    return sendAll(
      this.program.provider,
      txs,
      [params.oracleAuthority],
      skipPreflight
    );
  }
}

export class BufferRelayerAccount {
  static accountName = "BufferRelayerAccountData";

  program: SwitchboardProgram;

  publicKey: PublicKey;

  keypair?: Keypair;

  /**
   * CrankAccount constructor
   * @param params initialization params.
   */
  public constructor(params: AccountParams) {
    if (params.keypair === undefined && params.publicKey === undefined) {
      throw new Error(
        `${this.constructor.name}: User must provide either a publicKey or keypair for account use.`
      );
    }
    if (params.keypair !== undefined && params.publicKey !== undefined) {
      if (!params.publicKey.equals(params.keypair.publicKey)) {
        throw new Error(
          `${this.constructor.name}: provided pubkey and keypair mismatch.`
        );
      }
    }
    this.program = params.program;
    this.keypair = params.keypair;
    this.publicKey = params.publicKey ?? this.keypair.publicKey;
  }

  /**
   * Load and parse BufferRelayerAccount data based on the program IDL.
   * @return BufferRelayerAccount data parsed in accordance with the
   * Switchboard IDL.
   */
  async loadData(): Promise<any> {
    const data: any = await this.program.account.bufferRelayerAccountData.fetch(
      this.publicKey
    );
    data.ebuf = undefined;
    return data;
  }

  size(): number {
    return 4092;
  }

  static async create(
    program: SwitchboardProgram,
    params: {
      name: Buffer;
      minUpdateDelaySeconds: number;
      queueAccount: OracleQueueAccount;
      authority: PublicKey;
      jobAccount: JobAccount;
    }
  ): Promise<BufferRelayerAccount> {
    const [programStateAccount, stateBump] =
      ProgramStateAccount.fromSeed(program);
    const switchTokenMint = await params.queueAccount.loadMint();
    const keypair = Keypair.generate();
    const escrow = await spl.getAssociatedTokenAddress(
      switchTokenMint.address,
      keypair.publicKey
    );
    const size = 2048;
    const payer = programWallet(program);
    await program.rpc.bufferRelayerInit(
      {
        name: params.name.slice(0, 32),
        minUpdateDelaySeconds: params.minUpdateDelaySeconds,
        stateBump,
      },
      {
        accounts: {
          bufferRelayer: keypair.publicKey,
          escrow,
          authority: params.authority,
          queue: params.queueAccount.publicKey,
          job: params.jobAccount.publicKey,
          programState: programStateAccount.publicKey,
          mint: switchTokenMint.address,
          payer: payer.publicKey,
          tokenProgram: spl.TOKEN_PROGRAM_ID,
          associatedTokenProgram: spl.ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: new PublicKey("SysvarRent111111111111111111111111111111111"),
        },
        instructions: [
          anchor.web3.SystemProgram.createAccount({
            fromPubkey: programWallet(program).publicKey,
            newAccountPubkey: keypair.publicKey,
            space: size,
            lamports:
              await program.provider.connection.getMinimumBalanceForRentExemption(
                size
              ),
            programId: program.programId,
          }),
        ],
        signers: [keypair],
      }
    );
    return new BufferRelayerAccount({ program, keypair });
  }

  async openRound(): Promise<TransactionSignature> {
    const [programStateAccount, stateBump] = ProgramStateAccount.fromSeed(
      this.program
    );
    const relayerData = await this.loadData();
    const queue = relayerData.queuePubkey;
    const queueAccount = new OracleQueueAccount({
      program: this.program,
      publicKey: queue,
    });
    const switchTokenMint = await queueAccount.loadMint();
    const source = (
      await spl.getOrCreateAssociatedTokenAccount(
        this.program.provider.connection,
        programWallet(this.program),
        switchTokenMint.address,
        programWallet(this.program).publicKey,
        true
      )
    ).address;
    const bufferRelayer = this.publicKey;
    const escrow = relayerData.escrow;
    const queueData = await queueAccount.loadData();
    const queueAuthority = queueData.authority;
    const [permissionAccount, permissionBump] = PermissionAccount.fromSeed(
      this.program,
      queueAuthority,
      queueAccount.publicKey,
      this.publicKey
    );
    const payer = programWallet(this.program);
    const transferIx = spl.createTransferInstruction(
      source,
      escrow,
      programWallet(this.program).publicKey,
      queueData.reward.toNumber()
    );
    const openRoundIx = this.program.instruction.bufferRelayerOpenRound(
      {
        stateBump,
        permissionBump,
      },
      {
        accounts: {
          bufferRelayer,
          oracleQueue: queueAccount.publicKey,
          dataBuffer: queueData.dataBuffer,
          queueAuthority: queueData.authority,
          permission: permissionAccount.publicKey,
          escrow,
          programState: programStateAccount.publicKey,
          job: relayerData.jobPubkey,
        },
      }
    );
    const tx = new Transaction();
    tx.add(transferIx);
    tx.add(openRoundIx);
    const connection = (this.program.provider as anchor.AnchorProvider)
      .connection;
    return sendAndConfirmTransaction(connection, tx, [
      programWallet(this.program),
    ]);
  }

  async saveResult(params: {
    oracleAuthority: Keypair;
    result: Buffer;
    success: boolean;
  }): Promise<TransactionSignature> {
    const [programStateAccount, stateBump] = ProgramStateAccount.fromSeed(
      this.program
    );
    const relayerData = await this.loadData();
    const queue = new PublicKey(relayerData.queuePubkey);
    const queueAccount = new OracleQueueAccount({
      program: this.program,
      publicKey: queue!,
    });
    const bufferRelayer = this.publicKey;
    const escrow = relayerData.escrow;
    const queueData = await queueAccount.loadData();
    const queueAuthority = queueData.authority;
    const [permissionAccount, permissionBump] = PermissionAccount.fromSeed(
      this.program,
      queueAuthority,
      queueAccount.publicKey,
      this.publicKey
    );
    const oracleAccount = new OracleAccount({
      program: this.program,
      publicKey: relayerData.currentRound.oraclePubkey,
    });
    const oracleData = await oracleAccount.loadData();
    console.log("!!!!");
    return this.program.rpc.bufferRelayerSaveResult(
      {
        stateBump,
        permissionBump,
        result: params.result,
        success: params.success,
      },
      {
        accounts: {
          bufferRelayer,
          oracleAuthority: params.oracleAuthority.publicKey,
          oracle: relayerData.currentRound.oraclePubkey,
          oracleQueue: queueAccount.publicKey,
          dataBuffer: queueData.dataBuffer,
          queueAuthority: queueData.authority,
          permission: permissionAccount.publicKey,
          escrow,
          programState: programStateAccount.publicKey,
          oracleWallet: oracleData.tokenAccount,
          tokenProgram: spl.TOKEN_PROGRAM_ID,
        },
      }
    );
  }
}

export async function sendAll(
  provider: anchor.Provider,
  reqs: Array<any>,
  signers: Array<Keypair>,
  skipPreflight: boolean
): Promise<Array<TransactionSignature>> {
  const res: Array<TransactionSignature> = [];
  try {
    const opts = (provider as anchor.AnchorProvider).opts;
    // TODO: maybe finalized
    const blockhash = await provider.connection.getLatestBlockhash("confirmed");

    let txs = reqs.map((r: any) => {
      if (r === null || r === undefined) return new Transaction();
      const tx = r.tx;
      let rSigners = r.signers;

      if (rSigners === undefined) {
        rSigners = [];
      }

      tx.feePayer = (provider as anchor.AnchorProvider).wallet.publicKey;
      tx.recentBlockhash = blockhash.blockhash;

      rSigners
        .filter((s: any): s is Signer => s !== undefined)
        .forEach((kp: any) => {
          tx.partialSign(kp);
        });

      return tx;
    });
    txs = await packTransactions(
      provider.connection,
      txs,
      signers,
      (provider as anchor.AnchorProvider).wallet.publicKey
    );

    const signedTxs = await (
      provider as anchor.AnchorProvider
    ).wallet.signAllTransactions(txs);
    const promises = [];
    for (let k = 0; k < txs.length; k += 1) {
      const tx = signedTxs[k];
      const rawTx = tx.serialize();
      promises.push(
        provider.connection.sendRawTransaction(rawTx, {
          skipPreflight,
          maxRetries: 10,
        })
      );
    }
    return await Promise.all(promises);
  } catch (e) {
    console.log(e);
  }
  return res;
}

/**
 * Pack instructions into transactions as tightly as possible
 * @param instructions Instructions or Grouping of Instructions to pack down into transactions.
 * Arrays of instructions will be grouped into the same tx.
 * NOTE: this will break if grouping is too large for a single tx
 * @param feePayer Optional feepayer
 * @param recentBlockhash Optional blockhash
 * @returns Transaction[]
 */
export function packInstructions(
  instructions: (
    | anchor.web3.TransactionInstruction
    | anchor.web3.TransactionInstruction[]
  )[],
  feePayer = anchor.web3.PublicKey.default,
  recentBlockhash = anchor.web3.PublicKey.default.toBase58()
): anchor.web3.Transaction[] {
  // Constructs a new Transaction.
  const buildNewTransaction = (ixns: anchor.web3.TransactionInstruction[]) => {
    const txn = new anchor.web3.Transaction();
    txn.recentBlockhash = recentBlockhash;
    txn.feePayer = feePayer;
    return ixns.length ? txn.add(...ixns) : txn;
  };

  const getTxnSize = (transaction: anchor.web3.Transaction) => {
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

    try {
      return (
        transaction.serializeMessage().length +
        transaction.signatures.length * 64 +
        encodeLength(transaction.signatures.length).length
      );
    } catch (err) {
      return Number.MAX_SAFE_INTEGER;
    }
  };

  const packed: anchor.web3.Transaction[] = [];
  let currentTransaction = buildNewTransaction([]);
  const emptyTxSize = getTxnSize(currentTransaction);
  instructions
    .map((ixGroup) => (Array.isArray(ixGroup) ? ixGroup : [ixGroup]))
    .forEach((ixGroup) => {
      // Build a new transaction with this ixGroup for comparison.
      const newTransaction = buildNewTransaction(ixGroup);
      // Size of the new TXN - size of an empty TXN should ~= size of the ixGroup data.
      const newIxGroupSize = getTxnSize(newTransaction) - emptyTxSize;

      if (
        anchor.web3.PACKET_DATA_SIZE >=
        getTxnSize(currentTransaction) + newIxGroupSize
      ) {
        // If `newTransaction` can be added to current transaction, do so.
        currentTransaction.add(...newTransaction.instructions);
      } else if (anchor.web3.PACKET_DATA_SIZE <= getTxnSize(newTransaction)) {
        // If `newTransaction` is too large to fit in a transaction, throw an error.
        throw new Error(
          "Instruction packing error: a grouping of instructions must be able to fit into a single transaction"
        );
      } else {
        // If `newTransaction` cannot be added to `currentTransaction`, push `currentTransaction` and move forward.
        if (currentTransaction.instructions.length > 0) {
          packed.push(currentTransaction);
        }
        currentTransaction = newTransaction;
      }
    });
  // If the final transaction has at least 1 instruction, add it to the pack.
  if (currentTransaction.instructions.length > 0) {
    packed.push(currentTransaction);
  }
  return packed;
}

/**
 * Repack Transactions and sign them
 * @param connection Web3.js Connection
 * @param transactions Transactions to repack
 * @param signers Signers for each transaction
 */
export async function packTransactions(
  connection: anchor.web3.Connection,
  transactions: Transaction[],
  signers: Keypair[],
  feePayer: PublicKey
): Promise<Transaction[]> {
  const instructions = transactions.map((t) => t.instructions).flat();
  const txs = packInstructions(instructions, feePayer);
  const { blockhash } = await connection.getLatestBlockhash();
  txs.forEach((t) => {
    t.recentBlockhash = blockhash;
  });
  return signTransactions(txs, signers);
}

/**
 * Sign transactions with correct signers
 * @param transactions array of transactions to sign
 * @param signers array of keypairs to sign the array of transactions with
 * @returns transactions signed
 */
export function signTransactions(
  transactions: Transaction[],
  signers: Keypair[]
): Transaction[] {
  // Sign with all the appropriate signers
  for (const transaction of transactions) {
    // Get pubkeys of signers needed
    const sigsNeeded = transaction.instructions
      .map((instruction) => {
        const ixnSigners = instruction.keys.filter((meta) => meta.isSigner);
        return ixnSigners.map((signer) => signer.pubkey);
      })
      .flat();

    // Get matching signers in our supplied array
    const currentSigners = signers.filter((signer) =>
      Boolean(sigsNeeded.find((sig) => sig.equals(signer.publicKey)))
    );

    // Sign all transactions
    for (const signer of currentSigners) {
      transaction.partialSign(signer);
    }
  }
  return transactions;
}

export function programWallet(program: SwitchboardProgram): Keypair {
  return ((program.provider as anchor.AnchorProvider).wallet as AnchorWallet)
    .payer;
}

function safeDiv(number_: Big, denominator: Big, decimals = 20): Big {
  const oldDp = Big.DP;
  Big.DP = decimals;
  const result = number_.div(denominator);
  Big.DP = oldDp;
  return result;
}

export class AnchorWallet implements anchor.Wallet {
  constructor(readonly payer: Keypair) {
    this.payer = payer;
  }

  async signTransaction(tx: Transaction): Promise<Transaction> {
    tx.partialSign(this.payer);
    return tx;
  }

  async signAllTransactions(txs: Transaction[]): Promise<Transaction[]> {
    return txs.map((t) => {
      t.partialSign(this.payer);
      return t;
    });
  }

  get publicKey(): PublicKey {
    return this.payer.publicKey;
  }
}
