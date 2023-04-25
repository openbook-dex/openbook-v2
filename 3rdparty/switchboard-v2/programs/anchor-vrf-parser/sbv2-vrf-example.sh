#!/usr/bin/env bash

script_dir=$(dirname "$(readlink -f "$BASH_SOURCE")")
# echo "$script_dir"

# /usr/bin/env node --enable-source-maps  "$script_dir"/cli.js "$@"
# "$script_dir"/node_modules/.bin/ts-node-esm "$script_dir"/cli.ts "$@"
/usr/bin/env ts-node-esm "$script_dir"/cli.ts "$@"