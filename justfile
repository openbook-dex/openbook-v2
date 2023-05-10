build:
    cargo build-sbf --features enable-gpl

lint:
    cargo clippy --no-deps --tests --features enable-gpl --features test-bpf -- --allow=clippy::result-large-err

test TEST_NAME:
    cargo test {{ TEST_NAME }} --features enable-gpl --features test-bpf -- --nocapture

test-all:
    (cd ./programs/openbook-v2 && RUST_LOG=ERROR cargo test --features enable-gpl --features test-bpf)

idl:
    anchor build -- --features enable-gpl
    bash ./{{justfile_directory()}}/idl-fixup.sh
