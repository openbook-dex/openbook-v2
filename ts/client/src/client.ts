import {
  type AnchorProvider,
  BN,
  type Program,
  type IdlTypes,
  type IdlAccounts,
} from '@coral-xyz/anchor';
import {
  MintLayout,
  NATIVE_MINT,
  type RawMint,
  TOKEN_PROGRAM_ID,
  createCloseAccountInstruction,
  createInitializeAccount3Instruction,
  getOrCreateAssociatedTokenAccount,
} from '@solana/spl-token';
import {
  type AccountInfo,
  type Cluster,
  type Commitment,
  type Connection,
  Keypair,
  PublicKey,
  type Signer,
  SystemProgram,
  type TransactionInstruction,
  type TransactionSignature,
} from '@solana/web3.js';
import { type OpenbookV2 } from './openbook_v2';
import { sendTransaction } from './utils/rpc';
import { type OpenOrders, Market } from './accounts';
import { Side } from './utils/utils';

export type IdsSource = 'api' | 'static' | 'get-program-accounts';
export type PlaceOrderArgs = IdlTypes<OpenbookV2>['PlaceOrderArgs'];
export type OracleConfigParams = IdlTypes<OpenbookV2>['OracleConfigParams'];
export type OracleConfig = IdlTypes<OpenbookV2>['OracleConfig'];
export type MarketAccount = IdlAccounts<OpenbookV2>['market'];
export type OpenOrdersAccount = IdlAccounts<OpenbookV2>['openOrdersAccount'];

export interface OpenBookClientOptions {
  idsSource?: IdsSource;
  postSendTxCallback?: ({ txid }: { txid: string }) => void;
  prioritizationFee?: number;
  txConfirmationCommitment?: Commitment;
}

export class OpenBookV2Client {
  private readonly idsSource: IdsSource;
  private readonly postSendTxCallback?: ({ txid }) => void;
  private readonly prioritizationFee: number;
  private readonly txConfirmationCommitment: Commitment;

  constructor(
    public program: Program<OpenbookV2>,
    public programId: PublicKey,
    public cluster: Cluster,
    public opts: OpenBookClientOptions = {},
  ) {
    this.idsSource = opts.idsSource ?? 'get-program-accounts';
    this.prioritizationFee = opts?.prioritizationFee ?? 0;
    this.postSendTxCallback = opts?.postSendTxCallback;
    this.txConfirmationCommitment =
      opts?.txConfirmationCommitment ??
      (program.provider as AnchorProvider).opts.commitment ??
      'processed';
    // TODO: evil side effect, but limited backtraces are a nightmare
    Error.stackTraceLimit = 1000;
  }

  /// Convenience accessors
  public get connection(): Connection {
    return this.program.provider.connection;
  }

  public get walletPk(): PublicKey {
    return (this.program.provider as AnchorProvider).wallet.publicKey;
  }

  /// Transactions
  public async sendAndConfirmTransaction(
    ixs: TransactionInstruction[],
    opts: any = {},
  ): Promise<string> {
    return await sendTransaction(
      this.program.provider as AnchorProvider,
      ixs,
      opts.alts ?? [],
      {
        postSendTxCallback: this.postSendTxCallback,
        prioritizationFee: this.prioritizationFee,
        txConfirmationCommitment: this.txConfirmationCommitment,
        ...opts,
      },
    );
  }

  public async getMarket(publicKey: PublicKey): Promise<MarketAccount> {
    return await this.program.account.market.fetch(publicKey);
  }

  public async getOpenOrders(publicKey: PublicKey): Promise<OpenOrdersAccount> {
    return await this.program.account.openOrdersAccount.fetch(publicKey);
  }

  // public async getOpenOrdersAccountFromPublicKey(
  //   publicKey: PublicKey,
  // ): Promise<Market | null> {
  //   const account = await this.connection.getAccountInfo(publicKey);
  //   if (account != null) {

  //   }
  //   return null;
  // }

  public async createMarket(
    payer: Keypair,
    name: string,
    quoteMint: PublicKey,
    baseMint: PublicKey,
    quoteLoteSize: BN,
    baseLoteSize: BN,
    makerFee: BN,
    takerFee: BN,
    timeExpiry: BN,
    oracleA: PublicKey,
    oracleB: PublicKey,
    oracleConfigParams: OracleConfigParams = {
      confFilter: 0.1,
      maxStalenessSlots: 100,
    },
  ): Promise<TransactionSignature> {
    const bids = Keypair.generate().publicKey;
    const booksideSpace = 123720;
    const ix0 = SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: bids,
      lamports: await this.connection.getMinimumBalanceForRentExemption(
        booksideSpace,
      ),
      space: booksideSpace,
      programId: SystemProgram.programId,
    });
    const asks = Keypair.generate().publicKey;
    const ix1 = SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: asks,
      lamports: await this.connection.getMinimumBalanceForRentExemption(
        booksideSpace,
      ),
      space: booksideSpace,
      programId: SystemProgram.programId,
    });
    const eventQueue = Keypair.generate().publicKey;
    const eventQueueSpace = 91288;
    const ix2 = SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: eventQueue,
      lamports: await this.connection.getMinimumBalanceForRentExemption(
        eventQueueSpace,
      ),
      space: eventQueueSpace,
      programId: SystemProgram.programId,
    });

    const market = Keypair.generate();

    const [marketAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from('Market'), market.publicKey.toBuffer()],
      this.program.programId,
    );
    // Usage
    const baseVault = await getOrCreateAssociatedTokenAccount(
      this.connection,
      payer,
      baseMint,
      this.programId,
      true,
      undefined,
      undefined,
      TOKEN_PROGRAM_ID,
      marketAuthority,
    );
    const quoteVault = await getOrCreateAssociatedTokenAccount(
      this.connection,
      payer,
      quoteMint,
      this.programId,
      true,
      undefined,
      undefined,
      TOKEN_PROGRAM_ID,
      marketAuthority,
    );
    const [eventAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from('__event_authority')],
      this.program.programId,
    );

    const ix = await this.program.methods
      .createMarket(
        name,
        oracleConfigParams,
        quoteLoteSize,
        baseLoteSize,
        makerFee,
        takerFee,
        timeExpiry,
      )
      .accounts({
        market: market.publicKey,
        marketAuthority,
        oracleA,
        oracleB,
        bids,
        asks,
        eventQueue,
        payer: payer.publicKey,
        marketBaseVault: baseVault.address,
        marketQuoteVault: quoteVault.address,
        baseMint,
        quoteMint,
        systemProgram: SystemProgram.programId,
        eventAuthority,
        program: this.programId,
      })
      .instruction();

    return await this.sendAndConfirmTransaction([ix0, ix1, ix2, ix]);
  }

  public async deposit(
    openOrdersPublicKey: PublicKey,
    openOrdersAccount: OpenOrdersAccount,
    market: MarketAccount,
    userBaseAccount: PublicKey,
    userQuoteAccount: PublicKey,
    baseAmount: BN,
    quoteAmount: BN,
  ): Promise<TransactionSignature> {
    const ix = await this.program.methods
      .deposit(baseAmount, quoteAmount)
      .accounts({
        owner: openOrdersAccount.owner,
        market: openOrdersAccount.market,
        openOrdersAccount: openOrdersPublicKey,
        userBaseAccount,
        userQuoteAccount,
        marketBaseVault: market.marketBaseVault,
        marketQuoteVault: market.marketQuoteVault,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .instruction();

    return await this.sendAndConfirmTransaction([ix]);
  }

  public async depositNative(
    openOrdersPublicKey: PublicKey,
    openOrdersAccount: OpenOrdersAccount,
    market: MarketAccount,
    userBaseAccount: PublicKey,
    userQuoteAccount: PublicKey,
    baseAmount: BN,
    quoteAmount: BN,
  ): Promise<TransactionSignature> {
    const wrappedSolAccount: Keypair | undefined = new Keypair();
    let preInstructions: TransactionInstruction[] = [];
    let postInstructions: TransactionInstruction[] = [];
    const additionalSigners: Signer[] = [];

    const lamports = baseAmount.add(new BN(1e7));

    preInstructions = [
      SystemProgram.createAccount({
        fromPubkey: openOrdersAccount.owner,
        newAccountPubkey: wrappedSolAccount.publicKey,
        lamports: lamports.toNumber(),
        space: 165,
        programId: TOKEN_PROGRAM_ID,
      }),
      createInitializeAccount3Instruction(
        wrappedSolAccount.publicKey,
        NATIVE_MINT,
        openOrdersAccount.owner,
      ),
    ];
    postInstructions = [
      createCloseAccountInstruction(
        wrappedSolAccount.publicKey,
        openOrdersAccount.owner,
        openOrdersAccount.owner,
      ),
    ];
    additionalSigners.push(wrappedSolAccount);

    const ix = await this.program.methods
      .deposit(baseAmount, quoteAmount)
      .accounts({
        owner: openOrdersAccount.owner,
        market: openOrdersAccount.market,
        openOrdersAccount: openOrdersPublicKey,
        userBaseAccount,
        userQuoteAccount,
        marketBaseVault: market.marketBaseVault,
        marketQuoteVault: market.marketBaseVault,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .instruction();

    return await this.sendAndConfirmTransaction(
      [...preInstructions, ix, ...postInstructions],
      { additionalSigners },
    );
  }

  public decodeMarket(data: Buffer): any {
    return this.program.coder.accounts.decode('Market', data);
  }

  public async placeOrder(
    openOrdersPublicKey: PublicKey,
    openOrdersAccount: OpenOrdersAccount,
    market: MarketAccount,
    userTokenAccount: PublicKey,
    openOrdersAdmin: PublicKey | null,
    args: PlaceOrderArgs,
    openOrdersDelegate?: Keypair,
  ): Promise<TransactionSignature> {
    const marketVault =
      args.side === Side.Bid ? market.marketQuoteVault : market.marketBaseVault;
    const ix = await this.program.methods
      .placeOrder(args)
      .accounts({
        signer: openOrdersAccount.owner,
        asks: market.asks,
        bids: market.bids,
        marketVault,
        eventQueue: market.eventQueue,
        market: openOrdersAccount.market,
        openOrdersAccount: openOrdersPublicKey,
        oracleA: market.oracleA.key,
        oracleB: market.oracleB.key,
        userTokenAccount,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        openOrdersAdmin,
      })
      .instruction();
    const signers: Signer[] = [];
    if (openOrdersDelegate != null) {
      signers.push(openOrdersDelegate);
    }
    return await this.sendAndConfirmTransaction([ix], { signers });
  }

  public async cancelAndPlaceOrders(
    openOrdersPublicKey: PublicKey,
    openOrdersAccount: OpenOrdersAccount,
    market: MarketAccount,
    userBaseAccount: PublicKey,
    userQuoteAccount: PublicKey,
    openOrdersAdmin: PublicKey | null,
    cancelClientOrdersIds: BN[],
    placeOrders: PlaceOrderArgs[],
    openOrdersDelegate?: Keypair,
  ): Promise<TransactionSignature> {
    const ix = await this.program.methods
      .cancelAndPlaceOrders(cancelClientOrdersIds, placeOrders)
      .accounts({
        signer: openOrdersAccount.owner,
        asks: market.asks,
        bids: market.bids,
        marketQuoteVault: market.marketQuoteVault,
        marketBaseVault: market.marketQuoteVault,
        eventQueue: market.eventQueue,
        market: openOrdersAccount.market,
        openOrdersAccount: openOrdersPublicKey,
        oracleA: market.oracleA.key,
        oracleB: market.oracleB.key,
        userBaseAccount,
        userQuoteAccount,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        openOrdersAdmin,
      })
      .instruction();
    const signers: Signer[] = [];
    if (openOrdersDelegate != null) {
      signers.push(openOrdersDelegate);
    }
    return await this.sendAndConfirmTransaction([ix], { signers });
  }

  public async cancelOrderById(
    openOrdersPublicKey: PublicKey,
    openOrdersAccount: OpenOrdersAccount,
    market: MarketAccount,
    orderId: BN,
    openOrdersDelegate?: Keypair,
  ): Promise<TransactionSignature> {
    const ix = await this.program.methods
      .cancelOrder(orderId)
      .accounts({
        signer: openOrdersAccount.owner,
        asks: market.asks,
        bids: market.bids,
        market: openOrdersAccount.market,
        openOrdersAccount: openOrdersPublicKey,
      })
      .instruction();
    const signers: Signer[] = [];
    if (openOrdersDelegate != null) {
      signers.push(openOrdersDelegate);
    }
    return await this.sendAndConfirmTransaction([ix], { signers });
  }

  public async cancelOrderByClientId(
    openOrdersPublicKey: PublicKey,
    openOrdersAccount: OpenOrdersAccount,
    market: MarketAccount,
    clientOrderId: BN,
    openOrdersDelegate?: Keypair,
  ): Promise<TransactionSignature> {
    const ix = await this.program.methods
      .cancelOrderByClientOrderId(clientOrderId)
      .accounts({
        signer: openOrdersAccount.owner,
        asks: market.asks,
        bids: market.bids,
        market: openOrdersAccount.market,
        openOrdersAccount: openOrdersPublicKey,
      })
      .instruction();
    const signers: Signer[] = [];
    if (openOrdersDelegate != null) {
      signers.push(openOrdersDelegate);
    }
    return await this.sendAndConfirmTransaction([ix], { signers });
  }
}

export function decodeMint(data: Buffer): RawMint {
  return MintLayout.decode(data);
}

export async function getFilteredProgramAccounts(
  connection: Connection,
  programId: PublicKey,
  filters,
): Promise<Array<{ publicKey: PublicKey; accountInfo: AccountInfo<Buffer> }>> {
  // @ts-expect-error not need check
  const resp = await connection._rpcRequest('getProgramAccounts', [
    programId.toBase58(),
    {
      commitment: connection.commitment,
      filters,
      encoding: 'base64',
    },
  ]);
  if (resp.error !== null) {
    throw new Error(resp.error.message);
  }
  return resp.result.map(
    ({ pubkey, account: { data, executable, owner, lamports } }) => ({
      publicKey: new PublicKey(pubkey),
      accountInfo: {
        data: Buffer.from(data[0], 'base64'),
        executable,
        owner: new PublicKey(owner),
        lamports,
      },
    }),
  );
}
