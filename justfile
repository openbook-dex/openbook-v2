fuzz-toolchain := if `arch` == "arm64" { "+nightly-x86_64-apple-darwin" } else { "+nightly" }

build:
    cargo build-sbf --features enable-gpl

lint:
    cargo clippy --no-deps --tests --features enable-gpl --features test-bpf -- --allow=clippy::result-large-err

test TEST_NAME:
    cargo test-sbf --features enable-gpl --  {{ TEST_NAME }}

test-all:
    (cd ./programs/openbook-v2 && RUST_LOG=ERROR cargo test-sbf --features enable-gpl)

test-dev:
    (find programs) | entr -s 'just test-all'

idl:
    anchor build --arch sbf -- --features enable-gpl
    bash {{ justfile_directory() }}/idl-fixup.sh

fuzz:
  cd ./programs/openbook-v2/fuzz && cargo {{ fuzz-toolchain }} fuzz run multiple_orders

fuzz-reproduce CASE:
  cd ./programs/openbook-v2/fuzz && RUST_LOG=debug cargo {{ fuzz-toolchain }} fuzz run multiple_orders {{ CASE }}
