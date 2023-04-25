#!/bin/bash

set -e

# Imports
project_dir=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
javascript_dir="$project_dir"/javascript
program_dir="$project_dir"/programs

cd "$javascript_dir"/solana.js
yarn install && yarn build

cd "$javascript_dir"/sbv2-utils
yarn install && yarn build

cd "$javascript_dir"/sbv2-lite
yarn install && yarn build

cd "$javascript_dir"/feed-parser
yarn install && yarn build

cd "$javascript_dir"/feed-walkthrough
yarn install && yarn build

cd "$javascript_dir"/lease-observer
yarn install && yarn build

cd "$program_dir"/anchor-buffer-parser
yarn install

cd "$program_dir"/anchor-feed-parser
yarn install

cd "$program_dir"/anchor-vrf-parser
yarn install