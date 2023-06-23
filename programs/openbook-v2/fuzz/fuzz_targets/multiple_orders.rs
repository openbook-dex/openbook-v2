#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::{fuzz_target, Corpus};
use log::info;
use openbook_v2_fuzz::{processor::TestSyscallStubs, FuzzContext, UserId};
use std::{collections::HashSet, sync::Once};

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
        user_ids: HashSet<UserId>,
        data: openbook_v2::instruction::ConsumeEvents,
    },
    ConsumeGivenEvents {
        user_ids: HashSet<UserId>,
        data: openbook_v2::instruction::ConsumeGivenEvents,
    },
    CancelOrder {
        user_id: UserId,
        data: openbook_v2::instruction::CancelOrder,
    },
    CancelOrderByClientOrderId {
        user_id: UserId,
        data: openbook_v2::instruction::CancelOrderByClientOrderId,
    },
    CancelAllOrders {
        user_id: UserId,
        data: openbook_v2::instruction::CancelAllOrders,
    },
    SettleFunds {
        user_id: UserId,
        data: openbook_v2::instruction::SettleFunds,
    },
    SweepFees {
        data: openbook_v2::instruction::SweepFees,
    },
    StubOracleSet {
        data: openbook_v2::instruction::StubOracleSet,
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

            FuzzInstruction::ConsumeEvents { user_ids, data } => ctx
                .consume_events(user_ids, data)
                .map_or_else(error_filter::consume_events, |_| true),

            FuzzInstruction::ConsumeGivenEvents { user_ids, data } => ctx
                .consume_given_events(user_ids, data)
                .map_or_else(error_filter::consume_given_events, |_| true),

            FuzzInstruction::CancelOrder { user_id, data } => ctx
                .cancel_order(user_id, data)
                .map_or_else(error_filter::cancel_order, |_| true),

            FuzzInstruction::CancelOrderByClientOrderId { user_id, data } => ctx
                .cancel_order_by_client_order_id(user_id, data)
                .map_or_else(error_filter::cancel_order_by_client_order_id, |_| true),

            FuzzInstruction::CancelAllOrders { user_id, data } => ctx
                .cancel_all_orders(user_id, data)
                .map_or_else(error_filter::cancel_all_orders, |_| true),

            FuzzInstruction::SettleFunds { user_id, data } => ctx
                .settle_funds(user_id, data)
                .map_or_else(error_filter::settle_funds, |_| true),

            FuzzInstruction::SweepFees { data } => ctx
                .sweep_fees(data)
                .map_or_else(error_filter::sweep_fees, |_| true),

            FuzzInstruction::StubOracleSet { data } => ctx
                .stub_oracle_set(data)
                .map_or_else(error_filter::stub_oracle_set, |_| true),
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
            e if e == OpenBookError::InvalidOraclePrice.into() => true,
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
            e if e == OpenBookError::InvalidOraclePrice.into() => true,
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
            e if e == OpenBookError::InvalidOraclePrice.into() => true,
            e if e == OpenBookError::InvalidOrderSize.into() => true,
            e if e == TokenError::InsufficientFunds.into() => true,
            _ => panic!("{}", err),
        }
    }

    pub fn consume_events(err: ProgramError) -> bool {
        panic!("{}", err);
    }

    pub fn consume_given_events(err: ProgramError) -> bool {
        match err {
            e if e == OpenBookError::InvalidInputQueueSlots.into() => false,
            _ => panic!("{}", err),
        }
    }

    pub fn cancel_order(err: ProgramError) -> bool {
        match err {
            e if e == OpenBookError::InvalidInputOrderId.into() => false,
            e if e == OpenBookError::OpenOrdersOrderNotFound.into() => true,
            _ => panic!("{}", err),
        }
    }

    pub fn cancel_order_by_client_order_id(err: ProgramError) -> bool {
        match err {
            e if e == OpenBookError::OpenOrdersOrderNotFound.into() => true,
            _ => panic!("{}", err),
        }
    }

    pub fn cancel_all_orders(err: ProgramError) -> bool {
        panic!("{}", err);
    }

    pub fn settle_funds(err: ProgramError) -> bool {
        panic!("{}", err);
    }

    pub fn sweep_fees(err: ProgramError) -> bool {
        panic!("{}", err);
    }

    pub fn stub_oracle_set(err: ProgramError) -> bool {
        panic!("{}", err);
    }
}
