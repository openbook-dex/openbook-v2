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
  ASSOCIATED_TOKEN_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  createCloseAccountInstruction,
  createInitializeAccount3Instruction,
  getAssociatedTokenAddressSync,
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
  type AccountMeta,
} from '@solana/web3.js';
import { IDL, type OpenbookV2 } from './openbook_v2';
import { sendTransaction } from './utils/rpc';
import { Side } from './utils/utils';

export type IdsSource = 'api' | 'static' | 'get-program-accounts';
export type PlaceOrderArgs = IdlTypes<OpenbookV2>['PlaceOrderArgs'];
export type OracleConfigParams = IdlTypes<OpenbookV2>['OracleConfigParams'];
export type OracleConfig = IdlTypes<OpenbookV2>['OracleConfig'];
export type MarketAccount = IdlAccounts<OpenbookV2>['market'];
export type OpenOrdersAccount = IdlAccounts<OpenbookV2>['openOrdersAccount'];
export type OpenOrdersIndexerAccount =
  IdlAccounts<OpenbookV2>['openOrdersIndexer'];
export type EventHeapAccount = IdlAccounts<OpenbookV2>['eventHeap'];
export type BookSideAccount = IdlAccounts<OpenbookV2>['bookSide'];
export type LeafNode = IdlTypes<OpenbookV2>['LeafNode'];
export type AnyNode = IdlTypes<OpenbookV2>['AnyNode'];
export type FillEvent = IdlTypes<OpenbookV2>['FillEvent'];
export type OutEvent = IdlTypes<OpenbookV2>['OutEvent'];

export interface OpenBookClientOptions {
  idsSource?: IdsSource;
  postSendTxCallback?: ({ txid }: { txid: string }) => void;
  prioritizationFee?: number;
  txConfirmationCommitment?: Commitment;
}

export function nameToString(name: number[]): string {
  return utf8.decode(new Uint8Array(name)).split('\x00')[0];
}

const BooksideSpace = 90944 + 8;
const EventHeapSpace = 91280 + 8;

export class OpenBookV2Client {
  public program: Program<OpenbookV2>;

  private readonly idsSource: IdsSource;
  private readonly postSendTxCallback?: ({ txid }) => void;
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

  public async getOpenOrdersIndexer(
    publicKey: PublicKey,
  ): Promise<OpenOrdersIndexerAccount | null> {
    try {
      return await this.program.account.openOrdersIndexer.fetch(publicKey);
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
    quoteLotSize: BN,
    baseLotSize: BN,
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
    market = Keypair.generate(),
    collectFeeAdmin?: PublicKey,
  ): Promise<PublicKey> {
    const bids = await this.createProgramAccount(payer, BooksideSpace);
    const asks = await this.createProgramAccount(payer, BooksideSpace);
    const eventHeap = await this.createProgramAccount(payer, EventHeapSpace);

    const [marketAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from('Market'), market.publicKey.toBuffer()],
      this.program.programId,
    );

    const baseVault = getAssociatedTokenAddressSync(
      baseMint,
      marketAuthority,
      true,
    );

    const quoteVault = getAssociatedTokenAddressSync(
      quoteMint,
      marketAuthority,
      true,
    );

    const [eventAuthority] = PublicKey.findProgramAddressSync(
      [Buffer.from('__event_authority')],
      this.program.programId,
    );

    const ix = await this.program.methods
      .createMarket(
        name,
        oracleConfigParams,
        quoteLotSize,
        baseLotSize,
        makerFee,
        takerFee,
        timeExpiry,
      )
      .accounts({
        market: market.publicKey,
        marketAuthority,
        bids,
        asks,
        eventHeap,
        payer: payer.publicKey,
        marketBaseVault: baseVault,
        marketQuoteVault: quoteVault,
        baseMint,
        quoteMint,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        oracleA,
        oracleB,
        collectFeeAdmin:
          collectFeeAdmin != null ? collectFeeAdmin : payer.publicKey,
        openOrdersAdmin,
        consumeEventsAdmin,
        closeMarketAdmin,
        eventAuthority,
        program: this.programId,
      })
      .instruction();

    await this.sendAndConfirmTransaction([ix], {
      additionalSigners: [payer, market],
    });

    return market.publicKey;
  }

  public findOpenOrdersIndexer(): PublicKey {
    const [openOrdersIndexer] = PublicKey.findProgramAddressSync(
      [Buffer.from('OpenOrdersIndexer'), this.walletPk.toBuffer()],
      this.programId,
    );
    return openOrdersIndexer;
  }

  public async createOpenOrdersIndexer(
    openOrdersIndexer: PublicKey,
  ): Promise<TransactionSignature> {
    const ix = await this.program.methods
      .createOpenOrdersIndexer()
      .accounts({
        openOrdersIndexer,
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
        accountIndex.toArrayLike(Buffer, 'le', 4),
      ],
      this.programId,
    );
    return openOrders;
  }

  public async createOpenOrders(
    market: PublicKey,
    accountIndex: BN,
    openOrdersIndexer?: PublicKey,
  ): Promise<PublicKey> {
    if (openOrdersIndexer == null) {
      openOrdersIndexer = this.findOpenOrdersIndexer();
      try {
        const acc = await this.connection.getAccountInfo(openOrdersIndexer);
        if (acc == null) {
          const tx = await this.createOpenOrdersIndexer(openOrdersIndexer);
          console.log('Created open orders indexer', tx);
        }
      } catch {
        const tx = await this.createOpenOrdersIndexer(openOrdersIndexer);
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

    await this.sendAndConfirmTransaction([ix]);

    return openOrders;
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
    remainingAccounts: PublicKey[],
    openOrdersDelegate?: Keypair,
  ): Promise<TransactionSignature> {
    const marketVault =
      args.side === Side.Bid ? market.marketQuoteVault : market.marketBaseVault;
    const accountsMeta: AccountMeta[] = remainingAccounts.map((remaining) => ({
      pubkey: remaining,
      isSigner: false,
      isWritable: true,
    }));

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
      .remainingAccounts(accountsMeta)
      .instruction();
    const signers: Signer[] = [];
    if (openOrdersDelegate != null) {
      signers.push(openOrdersDelegate);
    }
    return await this.sendAndConfirmTransaction([ix], { signers });
  }

  public async placeTakeOrder(
    marketPublicKey: PublicKey,
    market: MarketAccount,
    userBaseAccount: PublicKey,
    userQuoteAccount: PublicKey,
    openOrdersAdmin: PublicKey | null,
    args: PlaceOrderArgs,
    referrerAccount: PublicKey | null,
    remainingAccounts: PublicKey[],
    openOrdersDelegate?: Keypair,
  ): Promise<TransactionSignature> {
    const accountsMeta: AccountMeta[] = remainingAccounts.map((remaining) => ({
      pubkey: remaining,
      isSigner: false,
      isWritable: true,
    }));
    const ix = await this.program.methods
      .placeTakeOrder(args)
      .accounts({
        signer:
          openOrdersDelegate != null
            ? openOrdersDelegate.publicKey
            : this.walletPk,
        asks: market.asks,
        bids: market.bids,
        eventHeap: market.eventHeap,
        market: marketPublicKey,
        oracleA: market.oracleA.key,
        oracleB: market.oracleB.key,
        userBaseAccount,
        userQuoteAccount,
        marketBaseVault: market.marketBaseVault,
        marketQuoteVault: market.marketQuoteVault,
        marketAuthority: market.marketAuthority,
        referrerAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        openOrdersAdmin,
      })
      .remainingAccounts(accountsMeta)
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
        marketBaseVault: market.marketBaseVault,
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

  // Use getAccountsToConsume as a helper
  public async consumeEvents(
    marketPublicKey: PublicKey,
    market: MarketAccount,
    limit: BN,
    remainingAccounts: PublicKey[],
  ): Promise<TransactionSignature> {
    const accountsMeta: AccountMeta[] = remainingAccounts.map((remaining) => ({
      pubkey: remaining,
      isSigner: false,
      isWritable: true,
    }));
    const ix = await this.program.methods
      .consumeEvents(limit)
      .accounts({
        eventHeap: market.eventHeap,
        market: marketPublicKey,
        consumeEventsAdmin: market.consumeEventsAdmin.key,
      })
      .remainingAccounts(accountsMeta)
      .instruction();
    return await this.sendAndConfirmTransaction([ix]);
  }

  // In order to get slots for certain key use getSlotsToConsume and include the key in the remainingAccounts
  public async consumeGivenEvents(
    marketPublicKey: PublicKey,
    market: MarketAccount,
    slots: BN[],
    remainingAccounts: PublicKey[],
  ): Promise<TransactionSignature> {
    const accountsMeta: AccountMeta[] = remainingAccounts.map((remaining) => ({
      pubkey: remaining,
      isSigner: false,
      isWritable: true,
    }));
    const ix = await this.program.methods
      .consumeGivenEvents(slots)
      .accounts({
        eventHeap: market.eventHeap,
        market: marketPublicKey,
        consumeEventsAdmin: market.consumeEventsAdmin.key,
      })
      .remainingAccounts(accountsMeta)
      .instruction();
    return await this.sendAndConfirmTransaction([ix]);
  }

  public async getAccountsToConsume(
    market: MarketAccount,
  ): Promise<PublicKey[]> {
    let accounts: PublicKey[] = new Array<PublicKey>();
    const eventHeap = await this.getEventHeap(market.eventHeap);
    if (eventHeap != null) {
      for (const node of eventHeap.nodes) {
        if (node.event.eventType === 0) {
          const fillEvent: FillEvent = this.program.coder.types.decode(
            'FillEvent',
            Buffer.from([0, ...node.event.padding]),
          );
          accounts = accounts
            .filter((a) => a !== fillEvent.maker)
            .concat([fillEvent.maker]);
        } else {
          const outEvent: OutEvent = this.program.coder.types.decode(
            'OutEvent',
            Buffer.from([0, ...node.event.padding]),
          );
          accounts = accounts
            .filter((a) => a !== outEvent.owner)
            .concat([outEvent.owner]);
        }
        // Tx would be too big, do not add more accounts
        if (accounts.length > 20) return accounts;
      }
    }
    return accounts;
  }

  public async getSlotsToConsume(
    key: PublicKey,
    market: MarketAccount,
  ): Promise<BN[]> {
    const slots: BN[] = new Array<BN>();

    const eventHeap = await this.getEventHeap(market.eventHeap);
    if (eventHeap != null) {
      for (const [i, node] of eventHeap.nodes.entries()) {
        if (node.event.eventType === 0) {
          const fillEvent: FillEvent = this.program.coder.types.decode(
            'FillEvent',
            Buffer.from([0, ...node.event.padding]),
          );
          if (key === fillEvent.maker) slots.push(new BN(i));
        } else {
          const outEvent: OutEvent = this.program.coder.types.decode(
            'OutEvent',
            Buffer.from([0, ...node.event.padding]),
          );
          if (key === outEvent.owner) slots.push(new BN(i));
        }
      }
    }
    return slots;
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
