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
    PlaceOrderPegged {
        user_id: UserId,
        data: openbook_v2::instruction::PlaceOrderPegged,
    },
    PlaceTakeOrder {
        user_id: UserId,
        data: openbook_v2::instruction::PlaceTakeOrder,
    },
}

fuzz_target!(|fuzz_data: FuzzData| { run_fuzz(fuzz_data) });

fn run_fuzz(fuzz_data: FuzzData) {
    if fuzz_data.instructions.len() == 0 {
        return;
    }

    let mut ctx = FuzzContext::new();
    ctx.initialize();

    for fuzz_instruction in fuzz_data.instructions {
        match fuzz_instruction {
            FuzzInstruction::PlaceOrder { user_id, data } => {
                ctx.place_order(user_id, data).unwrap()
            }

            FuzzInstruction::PlaceOrderPegged { user_id, data } => {
                ctx.place_order_pegged(user_id, data).unwrap()
            }

            FuzzInstruction::PlaceTakeOrder { user_id, data } => {
                ctx.place_take_order(user_id, data).unwrap()
            }
        };
    }
}