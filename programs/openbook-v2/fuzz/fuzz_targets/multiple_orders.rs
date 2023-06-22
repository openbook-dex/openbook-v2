#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::{fuzz_target, Corpus};
use log::info;
use openbook_v2_fuzz::{processor::TestSyscallStubs, FuzzContext, UserId};
use std::sync::Once;

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
    ConsumeEvents {
        data: openbook_v2::instruction::ConsumeEvents,
    },
    ConsumeGivenEvents {
        data: openbook_v2::instruction::ConsumeGivenEvents,
    },
}

fuzz_target!(|fuzz_data: FuzzData| -> Corpus {
    static ONCE: Once = Once::new();
    ONCE.call_once(env_logger::init);
    solana_program::program_stubs::set_syscall_stubs(Box::new(TestSyscallStubs {}));
    run_fuzz(fuzz_data)
});

fn run_fuzz(fuzz_data: FuzzData) -> Corpus {
    let mut corpus = Corpus::Keep;
    if fuzz_data.instructions.is_empty() {
        return Corpus::Reject;
    }

    let mut ctx = FuzzContext::new();
    ctx.initialize();

    for fuzz_instruction in fuzz_data.instructions {
        info!("{:#?}", fuzz_instruction);

        let has_valid_inputs = match fuzz_instruction {
            FuzzInstruction::PlaceOrder { user_id, data } => ctx
                .place_order(user_id, data)
                .map_or_else(error_filter::place_order, |_| true),

            FuzzInstruction::PlaceOrderPegged { user_id, data } => ctx
                .place_order_pegged(user_id, data)
                .map_or_else(error_filter::place_order_pegged, |_| true),

            FuzzInstruction::PlaceTakeOrder { user_id, data } => ctx
                .place_take_order(user_id, data)
                .map_or_else(error_filter::place_take_order, |_| true),

            FuzzInstruction::ConsumeEvents { data } => ctx
                .consume_events(data)
                .map_or_else(error_filter::consume_events, |_| true),

            FuzzInstruction::ConsumeGivenEvents { data } => ctx
                .consume_given_events(data)
                .map_or_else(error_filter::consume_given_events, |_| true),
        };

        if !has_valid_inputs {
            corpus = Corpus::Reject;
        };
    }

    corpus
}

mod error_filter {
    use openbook_v2::error::OpenBookError;
    use solana_program::program_error::ProgramError;
    use spl_token::error::TokenError;

    pub fn place_order(err: ProgramError) -> bool {
        match err {
            e if e == OpenBookError::InvalidInputLots.into() => false,
            e if e == OpenBookError::InvalidInputPriceLots.into() => false,
            e if e == OpenBookError::InvalidOrderSize.into() => true,
            e if e == OpenBookError::OpenOrdersFull.into() => true,
            e if e == OpenBookError::WouldSelfTrade.into() => true,
            e if e == TokenError::InsufficientFunds.into() => true,
            _ => panic!("{}", err),
        }
    }

    pub fn place_order_pegged(err: ProgramError) -> bool {
        match err {
            e if e == OpenBookError::InvalidInputLots.into() => false,
            e if e == OpenBookError::InvalidInputPegLimit.into() => false,
            e if e == OpenBookError::InvalidInputPriceLots.into() => false,
            e if e == OpenBookError::InvalidInputStaleness.into() => false,
            e if e == OpenBookError::InvalidOrderPostIOC.into() => true,
            e if e == OpenBookError::InvalidOrderPostMarket.into() => true,
            e if e == OpenBookError::InvalidOrderSize.into() => true,
            e if e == OpenBookError::InvalidPriceLots.into() => true,
            e if e == OpenBookError::WouldSelfTrade.into() => true,
            e if e == TokenError::InsufficientFunds.into() => true,
            _ => panic!("{}", err),
        }
    }

    pub fn place_take_order(err: ProgramError) -> bool {
        match err {
            e if e == OpenBookError::InvalidInputLots.into() => false,
            e if e == OpenBookError::InvalidInputOrderType.into() => false,
            e if e == OpenBookError::InvalidInputPriceLots.into() => false,
            e if e == OpenBookError::InvalidOrderSize.into() => true,
            e if e == TokenError::InsufficientFunds.into() => true,
            _ => panic!("{}", err),
        }
    }

    pub fn consume_events(err: ProgramError) -> bool {
        match err {
            _ => panic!("{}", err),
        }
    }

    pub fn consume_given_events(err: ProgramError) -> bool {
        match err {
            e if e == OpenBookError::InvalidInputQueueSlots.into() => false,
            _ => panic!("{}", err),
        }
    }
}
