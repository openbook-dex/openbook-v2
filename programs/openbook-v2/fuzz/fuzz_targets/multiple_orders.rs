#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use openbook_v2_fuzz::{FuzzContext, UserId};

#[derive(Debug, Arbitrary, Clone)]
struct FuzzData {
    instructions: Vec<FuzzInstruction>,
}

#[derive(Debug, Arbitrary, Clone)]
enum FuzzInstruction {
    PlaceOrder {
        user_id: UserId,
        data: openbook_v2::instruction::PlaceOrder,
    },
}

fuzz_target!(|fuzz_data: FuzzData| { run_fuzz(fuzz_data) });

fn run_fuzz(fuzz_data: FuzzData) {
    let mut ctx = FuzzContext::new();
    ctx.initialize();

    for fuzz_instruction in fuzz_data.instructions {
        match fuzz_instruction {
            FuzzInstruction::PlaceOrder { user_id, data } => {
                ctx.place_order(user_id, data).unwrap()
            }
        };
    }
}
