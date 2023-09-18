import {
  type AnchorProvider,
  BN,
  Program,
  type IdlTypes,
  type IdlAccounts,
} from '@coral-xyz/anchor';
import { utf8 } from '@coral-xyz/anchor/dist/cjs/utils/bytes';
import {
  MintLayout,
  NATIVE_MINT,
  type RawMint,
  TOKEN_PROGRAM_ID,
  createCloseAccountInstruction,
  createInitializeAccount3Instruction,
} from '@solana/spl-token';
import {
  type AccountInfo,
  type Commitment,
  type Connection,
  Keypair,
  PublicKey,
  type Signer,
  SystemProgram,
  type TransactionInstruction,
  type TransactionSignature,
  Transaction,
} from '@solana/web3.js';
import * as splToken from '@solana/spl-token';
import { IDL, type OpenbookV2 } from './openbook_v2';
import { sendTransaction } from './utils/rpc';
import { Side } from './utils/utils';

export type IdsSource = 'api' | 'static' | 'get-program-accounts';
export type PlaceOrderArgs = IdlTypes<OpenbookV2>['PlaceOrderArgs'];
export type OracleConfigParams = IdlTypes<OpenbookV2>['OracleConfigParams'];
export type OracleConfig = IdlTypes<OpenbookV2>['OracleConfig'];
export type MarketAccount = IdlAccounts<OpenbookV2>['market'];
export type OpenOrdersAccount = IdlAccounts<OpenbookV2>['openOrdersAccount'];
export type EventHeapAccount = IdlAccounts<OpenbookV2>['eventHeap'];
export type BookSideAccount = IdlAccounts<OpenbookV2>['bookSide'];
export type LeafNode = IdlTypes<OpenbookV2>['LeafNode'];
export type AnyNode = IdlTypes<OpenbookV2>['AnyNode'];

export interface OpenBookClientOptions {
  idsSource?: IdsSource;
  postSendTxCallback?: ({ txid }: { txid: string }) => void;
  prioritizationFee?: number;
  txConfirmationCommitment?: Commitment;
}

export interface Filter {
  memcmp: {
    offset: number;
    bytes: string;
  };
  // Add other filter properties as needed
}

export function nameToString(name: number[]): string {
  return utf8.decode(new Uint8Array(name)).split('\x00')[0];
}

const BooksideSpace = 90944 + 8;
const EventHeapSpace = 91280 + 8;

export class OpenBookV2Client {
  public program: Program<OpenbookV2>;

  private readonly idsSource: IdsSource;
  private readonly postSendTxCallback?: ({ txid }: { txid: string }) => void;
  private readonly prioritizationFee: number;
  private readonly txConfirmationCommitment: Commitment;

  constructor(
    public programId: PublicKey,
    public provider: AnchorProvider,
    public opts: OpenBookClientOptions = {},
  ) {
    this.program = new Program<OpenbookV2>(IDL, programId, provider);
    this.idsSource = opts.idsSource ?? 'get-program-accounts';
    this.prioritizationFee = opts?.prioritizationFee ?? 0;
    this.postSendTxCallback = opts?.postSendTxCallback;
    this.txConfirmationCommitment =
      opts?.txConfirmationCommitment ??
      (this.program.provider as AnchorProvider).opts.commitment ??
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

  public async createProgramAccount(
    authority: Keypair,
    size: number,
  ): Promise<PublicKey> {
    const lamports = await this.connection.getMinimumBalanceForRentExemption(
      size,
    );
    const address = Keypair.generate();

    const tx = new Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: authority.publicKey,
        newAccountPubkey: address.publicKey,
        lamports,
        space: size,
        programId: this.programId,
      }),
    ).instructions;

    await this.sendAndConfirmTransaction(tx, {
      additionalSigners: [authority, address],
    });
    return address.publicKey;
  }

  public async getMarket(publicKey: PublicKey): Promise<MarketAccount | null> {
    try {
      return await this.program.account.market.fetch(publicKey);
    } catch {
      return null;
    }
  }

  public async getOpenOrders(
    publicKey: PublicKey,
  ): Promise<OpenOrdersAccount | null> {
    try {
      return await this.program.account.openOrdersAccount.fetch(publicKey);
    } catch {
      return null;
    }
  }

  public async getEventHeap(
    publicKey: PublicKey,
  ): Promise<EventHeapAccount | null> {
    try {
      return await this.program.account.eventHeap.fetch(publicKey);
    } catch {
      return null;
    }
  }

  public async getBookSide(
    publicKey: PublicKey,
  ): Promise<BookSideAccount | null> {
    try {
      return await this.program.account.bookSide.fetch(publicKey);
    } catch {
      return null;
    }
  }

  public priceData(key: BN): number {
    const shiftedValue = key.shrn(64); // Shift right by 64 bits
    return shiftedValue.toNumber(); // Convert BN to a regular number
  }

  public getLeafNodes(bookside: BookSideAccount): LeafNode[] {
    const leafNodesData = bookside.nodes.nodes.filter(
      (x: AnyNode) => x.tag === 2,
    );
    const leafNodes: LeafNode[] = [];
    for (const x of leafNodesData) {
      const leafNode: LeafNode = this.program.coder.types.decode(
        'LeafNode',
        Buffer.from([0, ...x.data]),
      );
      leafNodes.push(leafNode);
    }
    return leafNodes;
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
    oracleA: PublicKey | null,
    oracleB: PublicKey | null,
    openOrdersAdmin: PublicKey | null,
    consumeEventsAdmin: PublicKey | null,
    closeMarketAdmin: PublicKey | null,
    oracleConfigParams: OracleConfigParams = {
      confFilter: 0.1,
      maxStalenessSlots: 100,
    },
    collectFeeAdmin?: PublicKey,
  ): Promise<string> {
    const bids = await this.createProgramAccount(payer, BooksideSpace);
    const asks = await this.createProgramAccount(payer, BooksideSpace);
    const eventHeap = await this.createProgramAccount(payer, EventHeapSpace);

    const market = Keypair.generate();
    const [marketAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from('Market'), market.publicKey.toBuffer()],
      this.program.programId,
    );

    const baseVault = await splToken.createAccount(
      this.connection,
      payer,
      baseMint,
      marketAuthority,
      Keypair.generate(),
    );

    const quoteVault = await splToken.createAccount(
      this.connection,
      payer,
      quoteMint,
      marketAuthority,
      Keypair.generate(),
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
        eventHeap,
        payer: payer.publicKey,
        marketBaseVault: baseVault,
        marketQuoteVault: quoteVault,
        baseMint,
        quoteMint,
        systemProgram: SystemProgram.programId,
        eventAuthority,
        program: this.programId,
        collectFeeAdmin:
          collectFeeAdmin != null ? collectFeeAdmin : payer.publicKey,
        openOrdersAdmin,
        consumeEventsAdmin,
        closeMarketAdmin,
      })
      .instruction();

    return await this.sendAndConfirmTransaction([ix], {
      additionalSigners: [payer, market],
    });
  }

  public findOpenOrdersIndexer(market: PublicKey): PublicKey {
    const [openOrdersIndexer] = PublicKey.findProgramAddressSync(
      [Buffer.from('OpenOrdersIndexer'), this.walletPk.toBuffer()],
      this.programId,
    );
    return openOrdersIndexer;
  }

  public async createOpenOrdersIndexer(
    market: PublicKey,
    openOrdersIndexer: PublicKey,
  ): Promise<TransactionSignature> {
    const ix = await this.program.methods
      .createOpenOrdersIndexer()
      .accounts({
        openOrdersIndexer,
        market,
        owner: this.walletPk,
        payer: this.walletPk,
        systemProgram: SystemProgram.programId,
      })
      .instruction();

    return await this.sendAndConfirmTransaction([ix]);
  }

  public findOpenOrders(market: PublicKey, accountIndex: BN): PublicKey {
    const [openOrders] = PublicKey.findProgramAddressSync(
      [
        Buffer.from('OpenOrders'),
        this.walletPk.toBuffer(),
        market.toBuffer(),
        accountIndex.toBuffer('le', 4),
      ],
      this.programId,
    );
    return openOrders;
  }

  public async createOpenOrders(
    market: PublicKey,
    accountIndex: BN,
    openOrdersIndexer?: PublicKey,
  ): Promise<TransactionSignature> {
    if (openOrdersIndexer == null) {
      openOrdersIndexer = this.findOpenOrdersIndexer(market);
      const acc = await this.connection.getAccountInfo(openOrdersIndexer);

      if (acc == null) {
        const tx = await this.createOpenOrdersIndexer(
          market,
          openOrdersIndexer,
        );
        console.log('Created open orders indexer', tx);
      }
    }
    if (accountIndex.toNumber() === 0) {
      throw Object.assign(new Error('accountIndex can not be 0'), {
        code: 403,
      });
    }
    const openOrders = this.findOpenOrders(market, accountIndex);

    const ix = await this.program.methods
      .createOpenOrdersAccount()
      .accounts({
        openOrdersIndexer,
        openOrdersAccount: openOrders,
        market,
        owner: this.walletPk,
        delegateAccount: null,
        payer: this.walletPk,
        systemProgram: SystemProgram.programId,
      })
      .instruction();

    return await this.sendAndConfirmTransaction([ix]);
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
    marketPublicKey: PublicKey,
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
        signer:
          openOrdersDelegate != null
            ? openOrdersDelegate.publicKey
            : this.walletPk,
        asks: market.asks,
        bids: market.bids,
        marketVault,
        eventHeap: market.eventHeap,
        market: marketPublicKey,
        openOrdersAccount: openOrdersPublicKey,
        oracleA: market.oracleA.key,
        oracleB: market.oracleB.key,
        userTokenAccount,
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
    marketPublicKey: PublicKey,
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
        signer:
          openOrdersDelegate != null
            ? openOrdersDelegate.publicKey
            : this.walletPk,
        asks: market.asks,
        bids: market.bids,
        marketQuoteVault: market.marketQuoteVault,
        marketBaseVault: market.marketQuoteVault,
        eventHeap: market.eventHeap,
        market: marketPublicKey,
        openOrdersAccount: openOrdersPublicKey,
        oracleA: market.oracleA.key,
        oracleB: market.oracleB.key,
        userBaseAccount,
        userQuoteAccount,
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
  filters: Filter[], // Specify the type of 'filters' as an array of 'Filter'
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
    ({ pubkey, account: { data, executable, owner, lamports } }: any) => ({
      publicKey: new PublicKey(pubkey),
      accountInfo: {
        data: Buffer.from(data[0], 'base64'),
        executable,
        owner: new PublicKey(owner),
        lamports,
      },
    })
  );
}

