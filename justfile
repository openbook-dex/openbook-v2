build:
    cargo build-sbf --features enable-gpl

lint:
    cargo clippy --no-deps --tests --features enable-gpl --features test-bpf -- --allow=clippy::result-large-err

test TEST_NAME:
    cargo test-sbf --features enable-gpl --  {{ TEST_NAME }}

test-all:
    (cd ./programs/openbook-v2 && RUST_LOG=ERROR cargo test-sbf --features enable-gpl)

idl:
    anchor build --arch sbf -- --features enable-gpl
    bash {{ justfile_directory() }}/idl-fixup.sh
