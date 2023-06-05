#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use openbook_v2_fuzz::FuzzContext;

#[derive(Debug, Arbitrary, Clone)]
struct FuzzData {
    instructions: Vec<FuzzInstruction>,
}

#[derive(Debug, Arbitrary, Clone)]
enum FuzzInstruction {
    Foo,
    Bar,
}

fuzz_target!(|fuzz_data: FuzzData| { run_fuzz(fuzz_data) });

fn run_fuzz(fuzz_data: FuzzData) {
    println!("{:?}", fuzz_data);

    let mut ctx = FuzzContext::new();
    ctx.stub_oracle_create().unwrap();
    ctx.create_market().unwrap();

    panic!();
}
