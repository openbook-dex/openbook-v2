fuzz-toolchain := if `arch` == "arm64" { "+nightly-2023-04-01-x86_64-apple-darwin" } else { "+nightly-2023-04-01" }

# only the program: cargo build-sbf would also try to build the janitor CLI for sbf
build:
    (cd ./programs/openbook-v2 && cargo build-sbf --features enable-gpl)

lint:
    cargo clippy --no-deps --tests --features enable-gpl --features test-bpf -- --allow=clippy::result-large-err

# test-janitor: the tests sign with a public test key instead of the mainnet janitor
test TEST_NAME:
    cargo test-sbf --features enable-gpl --features test-janitor --  {{ TEST_NAME }}

test-all:
    (cd ./programs/openbook-v2 && RUST_LOG=ERROR cargo test-sbf --features enable-gpl --features test-janitor)

test-dev:
    (find programs) | entr -s 'just test-all'

idl:
    anchor build --arch sbf -- --features enable-gpl
    bash {{ justfile_directory() }}/idl-fixup.sh
    cp -v ./target/types/openbook_v2.ts ./ts/client/src/openbook_v2.ts
    cp -v ./target/idl/openbook_v2.json ./idl/openbook_v2.json
    yarn lint --fix
    yarn format

fuzz:
  cd ./programs/openbook-v2/fuzz && cargo {{ fuzz-toolchain }} fuzz run --strip-dead-code multiple_orders

fuzz-reproduce CASE:
  cd ./programs/openbook-v2/fuzz && RUST_LOG=debug cargo {{ fuzz-toolchain }} fuzz run --strip-dead-code multiple_orders {{ CASE }}

publish:
    yarn format && yarn lint && yarn build && npm publish
