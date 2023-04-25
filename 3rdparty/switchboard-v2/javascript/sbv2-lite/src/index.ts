import * as anchor from "@project-serum/anchor";
import Big from "big.js";

export class AnchorWallet implements anchor.Wallet {
  constructor(readonly payer: anchor.web3.Keypair) {
    this.payer = payer;
  }

  async signTransaction(
    tx: anchor.web3.Transaction
  ): Promise<anchor.web3.Transaction> {
    tx.partialSign(this.payer);
    return tx;
  }

  async signAllTransactions(
    txs: anchor.web3.Transaction[]
  ): Promise<anchor.web3.Transaction[]> {
    return txs.map((t) => {
      t.partialSign(this.payer);
      return t;
    });
  }

  get publicKey(): anchor.web3.PublicKey {
    return this.payer.publicKey;
  }
}

/** A Switchboard V2 wrapper to assist in decoding onchain accounts */
export default class SwitchboardProgram {
  /**
   * Switchboard Devnet Program ID
   * 2TfB33aLaneQb5TNVwyDz3jSZXS6jdW2ARw1Dgf84XCG
   */
  public static devnetPid = new anchor.web3.PublicKey(
    "2TfB33aLaneQb5TNVwyDz3jSZXS6jdW2ARw1Dgf84XCG"
  );

  /**
   * Switchboard Mainnet Program ID
   * SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f
   */
  public static mainnetPid = new anchor.web3.PublicKey(
    "SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f"
  );

  /**
   * Default confirmation options for fetching Solana data
   */
  public static defaultConfirmOptions: anchor.web3.ConfirmOptions = {
    commitment: "confirmed",
  };

  /**
   * Switchboard Anchor program object
   */
  public program: anchor.Program;

  /**
   * Selected Solana cluster
   */
  public cluster: "devnet" | "mainnet-beta";

  constructor(program: anchor.Program, cluster: "devnet" | "mainnet-beta") {
    this.program = program;
    this.cluster = cluster;
  }

  /**
   * Return the Switchboard devnet program
   * @param connection optional connection object if not using the default endpoints
   * @param confirmOptions optional confirmation options. defaults to commitment level 'confirmed'
   */
  public static async loadDevnet(
    connection = new anchor.web3.Connection(
      anchor.web3.clusterApiUrl("devnet")
    ),
    confirmOptions = SwitchboardProgram.defaultConfirmOptions
  ): Promise<SwitchboardProgram> {
    const provider = new anchor.AnchorProvider(
      connection,
      new AnchorWallet(
        anchor.web3.Keypair.fromSeed(new Uint8Array(32).fill(1))
      ),
      confirmOptions
    );

    const anchorIdl = await anchor.Program.fetchIdl(
      SwitchboardProgram.devnetPid,
      provider
    );
    if (!anchorIdl) {
      throw new Error(
        `failed to read devnet idl for ${SwitchboardProgram.devnetPid}`
      );
    }

    const program = new anchor.Program(
      anchorIdl,
      SwitchboardProgram.devnetPid,
      provider
    );

    return new SwitchboardProgram(program, "devnet");
  }

  /**
   * Return the Switchboard mainnet-beta program
   * @param connection optional connection object if not using the default endpoints
   * @param confirmOptions optional confirmation options. defaults to commitment level 'confirmed'
   */
  public static async loadMainnet(
    connection = new anchor.web3.Connection(
      anchor.web3.clusterApiUrl("mainnet-beta")
    ),
    confirmOptions = SwitchboardProgram.defaultConfirmOptions
  ): Promise<SwitchboardProgram> {
    const provider = new anchor.AnchorProvider(
      connection,
      new AnchorWallet(
        anchor.web3.Keypair.fromSeed(new Uint8Array(32).fill(1))
      ),
      confirmOptions
    );

    const anchorIdl = await anchor.Program.fetchIdl(
      SwitchboardProgram.mainnetPid,
      provider
    );
    if (!anchorIdl) {
      throw new Error(
        `failed to read devnet idl for ${SwitchboardProgram.mainnetPid}`
      );
    }

    const program = new anchor.Program(
      anchorIdl,
      SwitchboardProgram.mainnetPid,
      provider
    );

    return new SwitchboardProgram(program, "mainnet-beta");
  }

  /** Parse an aggregators account data and return the latest confirmed result if valid
   * @param aggregator an aggregators deserialized account data
   * @param maxStaleness the maximum duration in seconds before a result is considered invalid. Defaults to 0 which ignores any checks
   * @returns latest confirmed result as a big.js or null if the latest confirmed round has insufficient oracle responses or data is too stale
   */
  private getLatestAggregatorValue(
    aggregator: any,
    maxStaleness = 0
  ): Big | null {
    if ((aggregator.latestConfirmedRound?.numSuccess ?? 0) === 0) {
      return null;
    }
    if (maxStaleness !== 0) {
      const now = new anchor.BN(Date.now() / 1000);
      const latestRoundTimestamp: anchor.BN =
        aggregator.latestConfirmedRound.roundOpenTimestamp;
      const staleness = now.sub(latestRoundTimestamp);
      if (staleness.gt(new anchor.BN(maxStaleness))) {
        return null;
      }
    }

    const mantissa = new Big(
      aggregator.latestConfirmedRound.result.mantissa.toString()
    );
    const scale = aggregator.latestConfirmedRound.result.scale;
    const oldDp = Big.DP;
    Big.DP = 20;
    const result: Big = mantissa.div(new Big(10).pow(scale));
    Big.DP = oldDp;
    return result;
  }

  /** Fetch and decode an aggregator account
   * @param aggregatorPubkey the aggregator's public key
   * @param commitment optional connection commitment level
   * @returns deserialized aggregator account, as specified by the Switchboard IDL
   */
  public async fetchAggregator(
    aggregatorPubkey: anchor.web3.PublicKey,
    commitment?: anchor.web3.Commitment
  ): Promise<any> {
    const aggregator: any =
      await this.program.account.aggregatorAccountData?.fetch(
        aggregatorPubkey,
        commitment
      );
    aggregator.ebuf = undefined;
    return aggregator;
  }

  /** Fetch and decode an aggregator's latest confirmed value if valid
   * @param aggregatorPubkey the aggregator's public key
   * @param commitment optional connection commitment level
   * @param maxStaleness the maximum duration in seconds before a result is considered invalid. Defaults to 0 which ignores any checks
   * @returns latest confirmed result as a big.js or null if the latest confirmed round has insufficient oracle responses or data is too stale
   */
  public async fetchAggregatorLatestValue(
    aggregatorPubkey: anchor.web3.PublicKey,
    commitment?: anchor.web3.Commitment,
    maxStaleness = 0
  ): Promise<Big | null> {
    const aggregator = await this.fetchAggregator(aggregatorPubkey, commitment);
    return this.getLatestAggregatorValue(aggregator, maxStaleness);
  }

  /** Decode an aggregator's account info
   * @param accountInfo the aggregatror's account info
   * @returns deserialized aggregator account, as specified by the Switchboard IDL
   */
  public decodeAggregator(accountInfo: anchor.web3.AccountInfo<Buffer>): any {
    const coder = new anchor.BorshAccountsCoder(this.program.idl);
    const aggregator: any = coder.decode(
      "AggregatorAccountData",
      accountInfo?.data
    );
    aggregator.ebuf = undefined;
    return aggregator;
  }

  /** Decode an aggregator and get the latest confirmed round
   * @param accountInfo the aggregator's account info
   * @param maxStaleness the maximum duration in seconds before a result is considered invalid. Defaults to 0 which ignores any checks
   * @returns latest confirmed result as a big.js or null if the latest confirmed round has insufficient oracle responses or data is too stale
   */
  public decodeLatestAggregatorValue(
    accountInfo: anchor.web3.AccountInfo<Buffer>,
    maxStaleness = 0
  ): Big | null {
    const aggregator = this.decodeAggregator(accountInfo);
    return this.getLatestAggregatorValue(aggregator, maxStaleness);
  }
}
