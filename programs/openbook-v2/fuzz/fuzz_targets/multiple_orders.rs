#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::{fuzz_target, Corpus};
use log::info;
use openbook_v2::instructions::MAX_EVENTS_CONSUME;
use openbook_v2_fuzz::{
    processor::TestSyscallStubs, FuzzContext, ReferrerId, UserId, INITIAL_BALANCE,
};
use std::{collections::HashSet, sync::Once};

#[derive(Debug, Arbitrary, Clone)]
struct FuzzData {
    market: openbook_v2::instruction::CreateMarket,
    instructions: Vec<FuzzInstruction>,
}

impl FuzzData {
    fn contains_place_order_ixs(&self) -> bool {
        self.instructions.iter().any(|ix| {
            matches!(
                ix,
                FuzzInstruction::PlaceOrder { .. }
                    | FuzzInstruction::PlaceOrderPegged { .. }
                    | FuzzInstruction::PlaceTakeOrder { .. }
            )
        })
    }
}

#[derive(Debug, Arbitrary, Clone)]
enum FuzzInstruction {
    Deposit {
        user_id: UserId,
        data: openbook_v2::instruction::Deposit,
    },
    PlaceOrder {
        user_id: UserId,
        data: openbook_v2::instruction::PlaceOrder,
        makers: Option<HashSet<UserId>>,
    },
    PlaceOrderPegged {
        user_id: UserId,
        data: openbook_v2::instruction::PlaceOrderPegged,
        makers: Option<HashSet<UserId>>,
    },
    PlaceTakeOrder {
        user_id: UserId,
        data: openbook_v2::instruction::PlaceTakeOrder,
        referrer_id: Option<ReferrerId>,
        makers: Option<HashSet<UserId>>,
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
        referrer_id: Option<ReferrerId>,
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
            FuzzInstruction::Deposit { user_id, data } => self
                .deposit(user_id, data)
                .map_or_else(error_parser::deposit, keep),

            FuzzInstruction::PlaceOrder {
                user_id,
                data,
                makers,
            } => self
                .place_order(user_id, data, makers.as_ref())
                .map_or_else(error_parser::place_order, keep),

            FuzzInstruction::PlaceOrderPegged {
                user_id,
                data,
                makers,
            } => self
                .place_order_pegged(user_id, data, makers.as_ref())
                .map_or_else(error_parser::place_order_pegged, keep),

            FuzzInstruction::PlaceTakeOrder {
                user_id,
                data,
                referrer_id,
                makers,
            } => self
                .place_take_order(user_id, data, referrer_id.as_ref(), makers.as_ref())
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

            FuzzInstruction::SettleFunds {
                user_id,
                data,
                referrer_id,
            } => self
                .settle_funds(user_id, data, referrer_id.as_ref())
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
    if !fuzz_data.contains_place_order_ixs() {
        return Corpus::Reject;
    }

    info!("initializing");
    info!("{:#?}", fuzz_data.market);

    let mut ctx = FuzzContext::new(fuzz_data.market.market_index);
    if matches!(
        ctx.initialize()
            .create_market(fuzz_data.market)
            .map_or_else(error_parser::create_market, |_| Corpus::Keep),
        Corpus::Reject
    ) {
        return Corpus::Reject;
    }

    info!("fuzzing");
    if fuzz_data
        .instructions
        .iter()
        .any(|ix| matches!(ctx.run(ix), Corpus::Reject))
    {
        return Corpus::Reject;
    };

    info!("validating");
    {
        let referrer_rebates: u64 = ctx
            .users
            .values()
            .map(|user| {
                let oo = ctx
                    .state
                    .get_account::<openbook_v2::state::OpenOrdersAccount>(&user.open_orders)
                    .unwrap();
                oo.position.referrer_rebates_available
            })
            .sum();

        let base_amount = ctx.state.get_balance(&ctx.base_vault);
        let quote_amount = ctx.state.get_balance(&ctx.quote_vault);

        let market = ctx
            .state
            .get_account::<openbook_v2::state::Market>(&ctx.market)
            .unwrap();

        assert_eq!(market.base_deposit_total, base_amount);
        assert_eq!(market.quote_deposit_total, quote_amount);
        assert_eq!(market.referrer_rebates_accrued, referrer_rebates);
    }

    {
        info!("cleaning event_queue");
        let consume_events_fuzz = FuzzInstruction::ConsumeEvents {
            user_ids: HashSet::from_iter(ctx.users.keys().cloned()),
            data: openbook_v2::instruction::ConsumeEvents {
                limit: MAX_EVENTS_CONSUME,
            },
        };

        let event_queue_len = |ctx: &FuzzContext| -> usize {
            let event_queue = ctx
                .state
                .get_account::<openbook_v2::state::EventQueue>(&ctx.event_queue)
                .unwrap();
            event_queue.len()
        };

        for _ in (0..event_queue_len(&ctx)).step_by(MAX_EVENTS_CONSUME) {
            ctx.run(&consume_events_fuzz);
        }

        assert_eq!(event_queue_len(&ctx), 0);
    }

    {
        let positions = ctx
            .users
            .values()
            .map(|user| {
                let oo = ctx
                    .state
                    .get_account::<openbook_v2::state::OpenOrdersAccount>(&user.open_orders)
                    .unwrap();
                oo.position
            })
            .collect::<Vec<_>>();

        let maker_volume_in_oo: u64 = positions.iter().map(|pos| pos.maker_volume).sum();
        let taker_volume_in_oo: u64 = positions.iter().map(|pos| pos.taker_volume).sum();

        let market = ctx
            .state
            .get_account::<openbook_v2::state::Market>(&ctx.market)
            .unwrap();

        assert_eq!(
            maker_volume_in_oo,
            taker_volume_in_oo + market.taker_volume_wo_oo
        );
    }

    ctx.users
        .keys()
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .for_each(|user_id| {
            info!("cleaning {:?}", user_id);
            ctx.run(&FuzzInstruction::CancelAllOrders {
                user_id,
                data: openbook_v2::instruction::CancelAllOrders {
                    limit: u8::MAX,
                    side_option: None,
                },
            });
            ctx.run(&FuzzInstruction::SettleFunds {
                user_id,
                data: openbook_v2::instruction::SettleFunds {},
                referrer_id: None,
            });

            let position = {
                let user = ctx.users.get(&user_id).unwrap();
                let open_orders = ctx
                    .state
                    .get_account::<openbook_v2::state::OpenOrdersAccount>(&user.open_orders)
                    .unwrap();
                open_orders.position
            };

            assert_eq!(position.bids_base_lots, 0);
            assert_eq!(position.asks_base_lots, 0);
            assert_eq!(position.base_free_native, 0);
            assert_eq!(position.quote_free_native, 0);
            assert_eq!(position.locked_maker_fees, 0);
            assert_eq!(position.referrer_rebates_available, 0);
        });

    {
        let is_empty = |pubkey| -> bool {
            let book_side = ctx
                .state
                .get_account::<openbook_v2::state::BookSide>(pubkey)
                .unwrap();
            book_side.is_empty()
        };

        assert!(is_empty(&ctx.asks));
        assert!(is_empty(&ctx.bids));
    }

    let referrers_balances: u64 = ctx
        .referrers
        .values()
        .map(|quote_vault| ctx.state.get_balance(quote_vault))
        .sum();

    {
        info!("cleaning market");
        ctx.run(&FuzzInstruction::SweepFees {
            data: openbook_v2::instruction::SweepFees {},
        });

        let market = ctx
            .state
            .get_account::<openbook_v2::state::Market>(&ctx.market)
            .unwrap();

        assert_eq!(ctx.state.get_balance(&ctx.base_vault), 0);
        assert_eq!(ctx.state.get_balance(&ctx.quote_vault), 0);
        assert_eq!(market.base_deposit_total, 0);
        assert_eq!(market.quote_deposit_total, 0);
        assert_eq!(market.fees_available, 0);
        assert_eq!(market.fees_to_referrers, referrers_balances);
    }

    {
        let base_balances: u64 = ctx
            .users
            .values()
            .map(|user| ctx.state.get_balance(&user.base_vault))
            .sum();

        let quote_balances: u64 = ctx
            .users
            .values()
            .map(|user| ctx.state.get_balance(&user.quote_vault))
            .sum();

        let n_users = ctx.users.len() as u64;
        assert_eq!(INITIAL_BALANCE * n_users, base_balances);
        assert_eq!(
            INITIAL_BALANCE * n_users,
            quote_balances
                + referrers_balances
                + ctx.state.get_balance(&ctx.collect_fee_admin_quote_vault)
        );
    }

    Corpus::Keep
}

mod error_parser {
    use anchor_spl::token::spl_token::error::TokenError;
    use libfuzzer_sys::Corpus;
    use openbook_v2::error::OpenBookError;
    use solana_program::program_error::ProgramError;

    pub fn create_market(err: ProgramError) -> Corpus {
        match err {
            e if e == OpenBookError::InvalidInputLots.into() => Corpus::Reject,
            e if e == OpenBookError::InvalidInputNameLength.into() => Corpus::Reject,
            e if e == OpenBookError::InvalidInputMarketExpired.into() => Corpus::Reject,
            e if e == OpenBookError::InvalidInputMarketFees.into() => Corpus::Reject,
            _ => panic!("{}", err),
        }
    }

    pub fn deposit(err: ProgramError) -> Corpus {
        match err {
            e if e == TokenError::InsufficientFunds.into() => Corpus::Keep,
            _ => panic!("{}", err),
        }
    }

    pub fn place_order(err: ProgramError) -> Corpus {
        match err {
            e if e == OpenBookError::InvalidInputLots.into() => Corpus::Reject,
            e if e == OpenBookError::InvalidInputLotsSize.into() => Corpus::Reject,
            e if e == OpenBookError::InvalidInputPriceLots.into() => Corpus::Reject,
            e if e == OpenBookError::InvalidOraclePrice.into() => Corpus::Keep,
            e if e == OpenBookError::InvalidPostAmount.into() => Corpus::Keep,
            e if e == OpenBookError::InvalidPriceLots.into() => Corpus::Keep,
            e if e == OpenBookError::OpenOrdersFull.into() => Corpus::Keep,
            e if e == OpenBookError::WouldSelfTrade.into() => Corpus::Keep,
            e if e == TokenError::InsufficientFunds.into() => Corpus::Keep,
            _ => panic!("{}", err),
        }
    }

    pub fn place_order_pegged(err: ProgramError) -> Corpus {
        match err {
            e if e == OpenBookError::InvalidInputLots.into() => Corpus::Reject,
            e if e == OpenBookError::InvalidInputLotsSize.into() => Corpus::Reject,
            e if e == OpenBookError::InvalidInputPegLimit.into() => Corpus::Reject,
            e if e == OpenBookError::InvalidInputPriceLots.into() => Corpus::Reject,
            e if e == OpenBookError::InvalidInputStaleness.into() => Corpus::Reject,
            e if e == OpenBookError::InvalidOraclePrice.into() => Corpus::Keep,
            e if e == OpenBookError::InvalidOrderPostIOC.into() => Corpus::Keep,
            e if e == OpenBookError::InvalidOrderPostMarket.into() => Corpus::Keep,
            e if e == OpenBookError::InvalidPostAmount.into() => Corpus::Keep,
            e if e == OpenBookError::InvalidPriceLots.into() => Corpus::Keep,
            e if e == OpenBookError::WouldSelfTrade.into() => Corpus::Keep,
            e if e == TokenError::InsufficientFunds.into() => Corpus::Keep,
            _ => panic!("{}", err),
        }
    }

    pub fn place_take_order(err: ProgramError) -> Corpus {
        match err {
            e if e == OpenBookError::InvalidInputLots.into() => Corpus::Reject,
            e if e == OpenBookError::InvalidInputLotsSize.into() => Corpus::Reject,
            e if e == OpenBookError::InvalidInputOrderType.into() => Corpus::Reject,
            e if e == OpenBookError::InvalidInputPriceLots.into() => Corpus::Reject,
            e if e == OpenBookError::InvalidOraclePrice.into() => Corpus::Keep,
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
            e if e == OpenBookError::OrderIdNotFound.into() => Corpus::Keep,
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
