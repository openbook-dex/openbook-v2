#!/bin/bash

set -e

# Imports
script_dir=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
project_dir="$(dirname "${script_dir}")"
program_dir="$project_dir"/programs

cd "$program_dir"/anchor-buffer-parser
anchor build
npx anchor-client-gen target/idl/anchor_buffer_parser.json client --program-id "$(solana-keygen pubkey target/deploy/anchor_buffer_parser-keypair.json)"

cd "$program_dir"/anchor-feed-parser
anchor build
npx anchor-client-gen target/idl/anchor_feed_parser.json client --program-id "$(solana-keygen pubkey target/deploy/anchor_feed_parser-keypair.json)"

cd "$program_dir"/anchor-history-parser
anchor build
npx anchor-client-gen target/idl/anchor_history_parser.json client --program-id "$(solana-keygen pubkey target/deploy/anchor_history_parser-keypair.json)"

cd "$program_dir"/anchor-vrf-parser
anchor build
npx anchor-client-gen target/idl/anchor_vrf_parser.json client --program-id "$(solana-keygen pubkey target/deploy/anchor_vrf_parser-keypair.json)"