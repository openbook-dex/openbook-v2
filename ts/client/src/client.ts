import { type AnchorProvider, BN, type Program, Provider } from '@coral-xyz/anchor';
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  AccountLayout,
  MintLayout,
  NATIVE_MINT,
  type RawAccount,
  type RawMint,
  TOKEN_PROGRAM_ID,
  createCloseAccountInstruction,
  createInitializeAccount3Instruction,
  getAssociatedTokenAddress,
  getOrCreateAssociatedTokenAccount,
} from '@solana/spl-token';
import {
  type AccountInfo,
  AccountMeta,
  AddressLookupTableAccount,
  type Cluster,
  type Commitment,
  type Connection,
  Keypair,
  MemcmpFilter,
  PublicKey,
  SYSVAR_INSTRUCTIONS_PUBKEY,
  SYSVAR_RENT_PUBKEY,
  type Signer,
  SystemProgram,
  type TransactionInstruction,
  type TransactionSignature,
} from '@solana/web3.js';
import { IDL, type OpenbookV2 } from './openbook_v2';
import { sendTransaction } from './utils/rpc';
import { type OpenOrdersAccount } from './accounts/openOrdersAccount';

export type IdsSource = 'api' | 'static' | 'get-program-accounts';

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
    this.idsSource = opts?.idsSource || 'get-program-accounts';
    this.prioritizationFee = opts?.prioritizationFee || 0;
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

  public async createMarket(
    marketIndex: number,
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
  ): Promise<TransactionSignature> {
    const bids = Keypair.generate().publicKey;
    let space = 123712;
    const ix0 = SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: bids,
      lamports: await this.connection.getMinimumBalanceForRentExemption(space),
      space,
      programId: SystemProgram.programId,
    });
    const asks = Keypair.generate().publicKey;
    const ix1 = SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: asks,
      lamports: await this.connection.getMinimumBalanceForRentExemption(space),
      space,
      programId: SystemProgram.programId,
    });
    const eventQueue = Keypair.generate().publicKey;
    space = 97680;
    const ix2 = SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: eventQueue,
      lamports: await this.connection.getMinimumBalanceForRentExemption(space),
      space,
      programId: SystemProgram.programId,
    });

    const market = Keypair.generate();

    const [marketAuthority, _tmp2] = PublicKey.findProgramAddressSync(
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
      market.publicKey,
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
      market.publicKey,
    );

    const ix = await this.program.methods
      .createMarket(
        name,
        {
          confFilter: 0.1,
          maxStalenessSlots: 100,
        },
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
        baseVault: baseVault.address,
        quoteVault: quoteVault.address,
        baseMint,
        quoteMint,
        systemProgram: SystemProgram.programId,
      })
      .instruction();

    return await this.sendAndConfirmTransaction([ix0, ix1, ix2, ix]);
  }

  public async deposit(
    openOrdersAccount: OpenOrdersAccount,
    tokenBaseAccount: PublicKey,
    tokenQuoteAccount: PublicKey,
    baseAmount: BN,
    quoteAmount: BN,
  ): Promise<TransactionSignature> {
    const ix = await this.program.methods
      .deposit(baseAmount, quoteAmount)
      .accounts({
        owner: openOrdersAccount.owner,
        market: openOrdersAccount.market,
        openOrdersAccount: openOrdersAccount.publicKey,
        tokenBaseAccount,
        tokenQuoteAccount,
        baseVault: openOrdersAccount.baseVault,
        quoteVault: openOrdersAccount.quoteVault,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .instruction();

    return await this.sendAndConfirmTransaction([ix]);
  }

  public async depositNative(
    openOrdersAccount: OpenOrdersAccount,
    market: PublicKey,
    tokenBaseAccount: PublicKey,
    tokenQuoteAccount: PublicKey,
    baseAmount: BN,
    quoteAmount: BN,
  ): Promise<TransactionSignature> {
    let wrappedSolAccount: Keypair | undefined;
    let preInstructions: TransactionInstruction[] = [];
    let postInstructions: TransactionInstruction[] = [];
    const additionalSigners: Signer[] = [];

    wrappedSolAccount = new Keypair();
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
        market,
        openOrdersAccount: openOrdersAccount.publicKey,
        tokenBaseAccount,
        tokenQuoteAccount,
        baseVault: openOrdersAccount.baseVault,
        quoteVault: openOrdersAccount.quoteVault,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .instruction();

    return await this.sendAndConfirmTransaction(
      [...preInstructions, ix, ...postInstructions],
      { additionalSigners },
    );
  }

  public decodeMarket(program, data: Buffer): any {
    return this.program.coder.accounts.decode('Market', data);
  }
}

export function decodeMint(data: Buffer): RawMint {
  return MintLayout.decode(data);
}

export function decodeAccount(data: Buffer): RawAccount {
  return AccountLayout.decode(data);
}

export async function getFilteredProgramAccounts(
  connection: Connection,
  programId: PublicKey,
  filters,
): Promise<Array<{ publicKey: PublicKey; accountInfo: AccountInfo<Buffer> }>> {
  // @ts-expect-error
  const resp = await connection._rpcRequest('getProgramAccounts', [
    programId.toBase58(),
    {
      commitment: connection.commitment,
      filters,
      encoding: 'base64',
    },
  ]);
  if (resp.error) {
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
