import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { OpenbookV2 } from '../target/types/openbook_v2';

describe('openbook-v2', () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.OpenbookV2 as Program<OpenbookV2>;

  it('Is initialized!', async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log('Your transaction signature', tx);
  });
});
