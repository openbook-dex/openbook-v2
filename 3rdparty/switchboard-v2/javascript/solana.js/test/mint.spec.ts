import 'mocha';

import * as anchor from '@project-serum/anchor';
import { setupTest, TestContext } from './utilts';
import { Keypair, PublicKey } from '@solana/web3.js';

describe('Mint Tests', () => {
  let ctx: TestContext;

  before(async () => {
    ctx = await setupTest();
  });

  const user = Keypair.generate();
  let userTokenAddress: PublicKey;

  it('Creates a user token account', async () => {
    const airdropTxn = await ctx.program.connection.requestAirdrop(
      user.publicKey,
      1 * anchor.web3.LAMPORTS_PER_SOL
    );
    await ctx.program.connection.confirmTransaction(airdropTxn);

    const [tokenAddress] = await ctx.program.mint.createAssocatedUser(
      ctx.payer.publicKey,
      user.publicKey
    );
    userTokenAddress = tokenAddress;

    const userTokenBalance =
      (await ctx.program.mint.getAssociatedBalance(user.publicKey)) ?? 0;

    if (userTokenBalance !== 0) {
      throw new Error(
        `Incorrect user token balance, expected 0, received ${userTokenBalance}`
      );
    }
  });

  it('Wraps SOL', async () => {
    if (!userTokenAddress) {
      throw new Error(`User token address does not exist`);
    }

    await ctx.program.mint.wrap(ctx.payer.publicKey, { amount: 0.25 }, user);

    const userTokenBalance =
      (await ctx.program.mint.getAssociatedBalance(user.publicKey)) ?? 0;
    if (userTokenBalance !== 0.25) {
      throw new Error(
        `Incorrect user token balance, expected 0.25, received ${userTokenBalance}`
      );
    }
  });

  it('Unwraps SOL', async () => {
    if (!userTokenAddress) {
      throw new Error(`User token address does not exist`);
    }

    const initialUserTokenBalance =
      (await ctx.program.mint.getAssociatedBalance(user.publicKey)) ?? 0;
    const expectedFinalBalance = initialUserTokenBalance - 0.1;
    if (expectedFinalBalance < 0) {
      throw new Error(`Final user token address would be negative`);
    }

    await ctx.program.mint.unwrap(ctx.payer.publicKey, 0.1, user);

    const userTokenBalance = await ctx.program.mint.getAssociatedBalance(
      user.publicKey
    );
    if (userTokenBalance !== expectedFinalBalance) {
      throw new Error(
        `Incorrect user token balance, expected ${expectedFinalBalance}, received ${userTokenBalance}`
      );
    }
  });

  it('Closes associated token account', async () => {
    if (!userTokenAddress) {
      throw new Error(`User token address does not exist`);
    }

    await ctx.program.mint.getAssociatedBalance(user.publicKey);

    await ctx.program.mint.unwrap(ctx.payer.publicKey, undefined, user);

    const userTokenAccount = await ctx.program.connection.getAccountInfo(
      userTokenAddress
    );
    if (userTokenAccount !== null) {
      throw new Error(`Failed to close associated token account`);
    }
  });
});
