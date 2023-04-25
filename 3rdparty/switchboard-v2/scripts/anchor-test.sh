#!/bin/bash

set -e

# Imports
script_dir=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
project_dir="$(dirname "${script_dir}")"
program_dir="$project_dir"/programs

cd "$program_dir"/anchor-buffer-parser
anchor test

cd "$program_dir"/anchor-feed-parser
anchor test

cd "$program_dir"/anchor-history-parser
anchor test

cd "$program_dir"/anchor-vrf-parser
anchor test