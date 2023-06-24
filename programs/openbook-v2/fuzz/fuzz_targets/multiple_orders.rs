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

trait FuzzRunner {
    fn run(&mut self, fuzz_ix: &FuzzInstruction) -> Corpus;
}

impl FuzzRunner for FuzzContext {
    fn run(&mut self, fuzz_ix: &FuzzInstruction) -> Corpus {
        info!("{:#?}", fuzz_ix);
        let keep = |_| Corpus::Keep;

        match fuzz_ix {
            FuzzInstruction::PlaceOrder { user_id, data } => self
                .place_order(user_id, data)
                .map_or_else(error_parser::place_order, keep),

            FuzzInstruction::PlaceOrderPegged { user_id, data } => self
                .place_order_pegged(user_id, data)
                .map_or_else(error_parser::place_order_pegged, keep),

            FuzzInstruction::PlaceTakeOrder { user_id, data } => self
                .place_take_order(user_id, data)
                .map_or_else(error_parser::place_take_order, keep),

            FuzzInstruction::ConsumeEvents { user_ids, data } => self
                .consume_events(user_ids, data)
                .map_or_else(error_parser::consume_events, keep),

            FuzzInstruction::ConsumeGivenEvents { user_ids, data } => self
                .consume_given_events(user_ids, data)
                .map_or_else(error_parser::consume_given_events, keep),

            FuzzInstruction::CancelOrder { user_id, data } => self
                .cancel_order(user_id, data)
                .map_or_else(error_parser::cancel_order, keep),

            FuzzInstruction::CancelOrderByClientOrderId { user_id, data } => self
                .cancel_order_by_client_order_id(user_id, data)
                .map_or_else(error_parser::cancel_order_by_client_order_id, keep),

            FuzzInstruction::CancelAllOrders { user_id, data } => self
                .cancel_all_orders(user_id, data)
                .map_or_else(error_parser::cancel_all_orders, keep),

            FuzzInstruction::SettleFunds { user_id, data } => self
                .settle_funds(user_id, data)
                .map_or_else(error_parser::settle_funds, keep),

            FuzzInstruction::SweepFees { data } => self
                .sweep_fees(data)
                .map_or_else(error_parser::sweep_fees, keep),

            FuzzInstruction::StubOracleSet { data } => self
                .stub_oracle_set(data)
                .map_or_else(error_parser::stub_oracle_set, keep),
        }
    }
}

fuzz_target!(|fuzz_data: FuzzData| -> Corpus {
    static ONCE: Once = Once::new();
    ONCE.call_once(env_logger::init);
    solana_program::program_stubs::set_syscall_stubs(Box::new(TestSyscallStubs {}));
    run_fuzz(fuzz_data)
});

fn run_fuzz(fuzz_data: FuzzData) -> Corpus {
    if fuzz_data.instructions.is_empty() {
        return Corpus::Reject;
    }

    let mut ctx = FuzzContext::new();
    ctx.initialize();

    if fuzz_data.instructions.iter().any(|ix| match ctx.run(ix) {
        Corpus::Keep => false,
        Corpus::Reject => true,
    }) {
        return Corpus::Reject;
    };

    Corpus::Keep
}

mod error_parser {
    use libfuzzer_sys::Corpus;
    use openbook_v2::error::OpenBookError;
    use solana_program::program_error::ProgramError;
    use spl_token::error::TokenError;

    pub fn place_order(err: ProgramError) -> Corpus {
        match err {
            e if e == OpenBookError::InvalidInputLots.into() => Corpus::Reject,
            e if e == OpenBookError::InvalidInputPriceLots.into() => Corpus::Reject,
            e if e == OpenBookError::InvalidOraclePrice.into() => Corpus::Keep,
            e if e == OpenBookError::InvalidOrderSize.into() => Corpus::Keep,
            e if e == OpenBookError::OpenOrdersFull.into() => Corpus::Keep,
            e if e == OpenBookError::WouldSelfTrade.into() => Corpus::Keep,
            e if e == TokenError::InsufficientFunds.into() => Corpus::Keep,
            _ => panic!("{}", err),
        }
    }

    pub fn place_order_pegged(err: ProgramError) -> Corpus {
        match err {
            e if e == OpenBookError::InvalidInputLots.into() => Corpus::Reject,
            e if e == OpenBookError::InvalidInputPegLimit.into() => Corpus::Reject,
            e if e == OpenBookError::InvalidInputPriceLots.into() => Corpus::Reject,
            e if e == OpenBookError::InvalidInputStaleness.into() => Corpus::Reject,
            e if e == OpenBookError::InvalidOraclePrice.into() => Corpus::Keep,
            e if e == OpenBookError::InvalidOrderPostIOC.into() => Corpus::Keep,
            e if e == OpenBookError::InvalidOrderPostMarket.into() => Corpus::Keep,
            e if e == OpenBookError::InvalidOrderSize.into() => Corpus::Keep,
            e if e == OpenBookError::InvalidPriceLots.into() => Corpus::Keep,
            e if e == OpenBookError::WouldSelfTrade.into() => Corpus::Keep,
            e if e == TokenError::InsufficientFunds.into() => Corpus::Keep,
            _ => panic!("{}", err),
        }
    }

    pub fn place_take_order(err: ProgramError) -> Corpus {
        match err {
            e if e == OpenBookError::InvalidInputLots.into() => Corpus::Reject,
            e if e == OpenBookError::InvalidInputOrderType.into() => Corpus::Reject,
            e if e == OpenBookError::InvalidInputPriceLots.into() => Corpus::Reject,
            e if e == OpenBookError::InvalidOraclePrice.into() => Corpus::Keep,
            e if e == OpenBookError::InvalidOrderSize.into() => Corpus::Keep,
            e if e == TokenError::InsufficientFunds.into() => Corpus::Keep,
            _ => panic!("{}", err),
        }
    }

    pub fn consume_events(err: ProgramError) -> Corpus {
        panic!("{}", err);
    }

    pub fn consume_given_events(err: ProgramError) -> Corpus {
        match err {
            e if e == OpenBookError::InvalidInputQueueSlots.into() => Corpus::Reject,
            _ => panic!("{}", err),
        }
    }

    pub fn cancel_order(err: ProgramError) -> Corpus {
        match err {
            e if e == OpenBookError::InvalidInputOrderId.into() => Corpus::Reject,
            e if e == OpenBookError::OpenOrdersOrderNotFound.into() => Corpus::Keep,
            _ => panic!("{}", err),
        }
    }

    pub fn cancel_order_by_client_order_id(err: ProgramError) -> Corpus {
        match err {
            e if e == OpenBookError::OpenOrdersOrderNotFound.into() => Corpus::Keep,
            _ => panic!("{}", err),
        }
    }

    pub fn cancel_all_orders(err: ProgramError) -> Corpus {
        panic!("{}", err);
    }

    pub fn settle_funds(err: ProgramError) -> Corpus {
        panic!("{}", err);
    }

    pub fn sweep_fees(err: ProgramError) -> Corpus {
        panic!("{}", err);
    }

    pub fn stub_oracle_set(err: ProgramError) -> Corpus {
        panic!("{}", err);
    }
}
