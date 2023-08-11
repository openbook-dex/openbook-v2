import {
  type AnchorProvider,
  BN,
  type Program,
  type IdlTypes,
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
import { type OpenOrdersAccount, Market } from './accounts';
import { Side } from './utils/utils';

export type IdsSource = 'api' | 'static' | 'get-program-accounts';
export type PlaceOrderArgs = IdlTypes<OpenbookV2>['PlaceOrderArgs'];
export type OracleConfigParams = IdlTypes<OpenbookV2>['OracleConfigParams'];

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

  public async getMarketFromPublicKey(
    publicKey: PublicKey,
  ): Promise<Market | null> {
    const account = await this.connection.getAccountInfo(publicKey);
    if (account != null) {
      const market = this.program.coder.accounts.decode('Market', account.data);
      return new Market(
        publicKey,
        market.asks,
        market.bids,
        market.eventQueue,
        market.baseVault,
        market.quoteVault,
        market.oracleA,
        market.oracleB,
      );
    }
    return null;
  }

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
    openOrdersAccount: OpenOrdersAccount,
    userBaseAccount: PublicKey,
    userQuoteAccount: PublicKey,
    baseAmount: BN,
    quoteAmount: BN,
  ): Promise<TransactionSignature> {
    const ix = await this.program.methods
      .deposit(baseAmount, quoteAmount)
      .accounts({
        owner: openOrdersAccount.owner,
        market: openOrdersAccount.market.publicKey,
        openOrdersAccount: openOrdersAccount.publicKey,
        userBaseAccount,
        userQuoteAccount,
        marketBaseVault: openOrdersAccount.market.baseVault,
        marketQuoteVault: openOrdersAccount.market.quoteVault,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .instruction();

    return await this.sendAndConfirmTransaction([ix]);
  }

  public async depositNative(
    openOrdersAccount: OpenOrdersAccount,
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
        market: openOrdersAccount.market.publicKey,
        openOrdersAccount: openOrdersAccount.publicKey,
        userBaseAccount,
        userQuoteAccount,
        marketBaseVault: openOrdersAccount.market.baseVault,
        marketQuoteVault: openOrdersAccount.market.quoteVault,
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
    openOrdersAccount: OpenOrdersAccount,
    userTokenAccount: PublicKey,
    openOrdersAdmin: PublicKey | null,
    args: PlaceOrderArgs,
  ): Promise<TransactionSignature> {
    const marketVault =
      args.side === Side.Bid
        ? openOrdersAccount.market.quoteVault
        : openOrdersAccount.market.baseVault;
    const ix = await this.program.methods
      .placeOrder(args)
      .accounts({
        signer: openOrdersAccount.owner,
        asks: openOrdersAccount.market.asks,
        bids: openOrdersAccount.market.bids,
        marketVault,
        eventQueue: openOrdersAccount.market.eventQueue,
        market: openOrdersAccount.market.publicKey,
        openOrdersAccount: openOrdersAccount.publicKey,
        oracleA: openOrdersAccount.market.oracleA,
        oracleB: openOrdersAccount.market.oracleB,
        userTokenAccount,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        openOrdersAdmin,
      })
      .instruction();
    const signers: Signer[] = [];
    if (openOrdersAccount.ownerOrDelegateKeypair != null) {
      signers.push(openOrdersAccount.ownerOrDelegateKeypair);
    }
    return await this.sendAndConfirmTransaction([ix], { signers });
  }

  public async cancelAndPlaceOrders(
    openOrdersAccount: OpenOrdersAccount,
    userBaseAccount: PublicKey,
    userQuoteAccount: PublicKey,
    openOrdersAdmin: PublicKey | null,
    cancelClientOrdersIds: BN[],
    placeOrders: PlaceOrderArgs[],
  ): Promise<TransactionSignature> {
    const ix = await this.program.methods
      .cancelAndPlaceOrders(cancelClientOrdersIds, placeOrders)
      .accounts({
        signer: openOrdersAccount.owner,
        asks: openOrdersAccount.market.asks,
        bids: openOrdersAccount.market.bids,
        marketQuoteVault: openOrdersAccount.market.quoteVault,
        marketBaseVault: openOrdersAccount.market.baseVault,
        eventQueue: openOrdersAccount.market.eventQueue,
        market: openOrdersAccount.market.publicKey,
        openOrdersAccount: openOrdersAccount.publicKey,
        oracleA: openOrdersAccount.market.oracleA,
        oracleB: openOrdersAccount.market.oracleB,
        userBaseAccount,
        userQuoteAccount,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        openOrdersAdmin,
      })
      .instruction();
    const signers: Signer[] = [];
    if (openOrdersAccount.ownerOrDelegateKeypair != null) {
      signers.push(openOrdersAccount.ownerOrDelegateKeypair);
    }
    return await this.sendAndConfirmTransaction([ix], { signers });
  }

  public async cancelOrderById(
    openOrdersAccount: OpenOrdersAccount,
    market: Market,
    orderId: BN,
  ): Promise<TransactionSignature> {
    const ix = await this.program.methods
      .cancelOrder(orderId)
      .accounts({
        signer: openOrdersAccount.owner,
        asks: market.asks,
        bids: market.bids,
        market: market.publicKey,
        openOrdersAccount: openOrdersAccount.publicKey,
      })
      .instruction();
    const signers: Signer[] = [];
    if (openOrdersAccount.ownerOrDelegateKeypair != null) {
      signers.push(openOrdersAccount.ownerOrDelegateKeypair);
    }
    return await this.sendAndConfirmTransaction([ix], { signers });
  }

  public async cancelOrderByClientId(
    openOrdersAccount: OpenOrdersAccount,
    market: Market,
    clientOrderId: BN,
  ): Promise<TransactionSignature> {
    const ix = await this.program.methods
      .cancelOrderByClientOrderId(clientOrderId)
      .accounts({
        signer: openOrdersAccount.owner,
        asks: market.asks,
        bids: market.bids,
        market: market.publicKey,
        openOrdersAccount: openOrdersAccount.publicKey,
      })
      .instruction();
    const signers: Signer[] = [];
    if (openOrdersAccount.ownerOrDelegateKeypair != null) {
      signers.push(openOrdersAccount.ownerOrDelegateKeypair);
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
