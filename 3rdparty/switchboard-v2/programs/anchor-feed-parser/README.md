# Anchor Feed Parser

## Generate the Client

```bash
anchor build
npx anchor-client-gen target/idl/anchor_feed_parser.json client --program-id "$(solana-keygen pubkey target/deploy/anchor_feed_parser-keypair.json)"
```
