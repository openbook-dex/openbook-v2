use crate::error::OpenBookError;
use crate::state::Market;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey;

/// Key allowed to wind down markets after [`SUNSET_TS`], see [`janitor_active`].
#[cfg(not(feature = "test-janitor"))]
pub const JANITOR: Pubkey = pubkey!("9YfRRFqgk4KF2TF5174Q9m8t8YhyA34EteLZR5oA3kGV");
/// Test janitor, its secret key is in the program tests. Never deploy a build with this key.
#[cfg(feature = "test-janitor")]
pub const JANITOR: Pubkey = pubkey!("7W2ZXgKo6kYucLYuUZkGGEEQpNrCE1ZGuT6QXxonodoZ");
/// Receives market / book / heap / vault rent on market close: the OpenBook v2 multisig. How it
/// is distributed (market creators can only be attributed off-chain) is decided separately.
pub const ESCROW: Pubkey = pubkey!("67ESd69L3c3FHSTDpeicEutN8iNe7vqGxmxAbMxUTqj9");
/// Unix timestamp after which creation / placement instructions are refused (2026-11-15T00:00:00Z,
/// the first day of Breakpoint 2026). Tests move the Clock sysvar around it.
pub const SUNSET_TS: i64 = 1_794_700_800;

/// Rent exempt cost per byte before SIMD-0437: 3480 lamports per byte-year, two years.
pub const LEGACY_LAMPORTS_PER_BYTE: u64 = 6960;
/// Bytes of account metadata that rent is charged for on top of the data.
pub const ACCOUNT_STORAGE_OVERHEAD: u64 = 128;
const TOKEN_ACCOUNT_LEN: u64 = 165;

/// Reward for closing an open orders account, fixed in lamports at the legacy rent and not
/// following the Rent sysvar: LEGACY_LAMPORTS_PER_BYTE * (ACCOUNT_STORAGE_OVERHEAD + data bytes)
/// * multiple, here the legacy rent of 4 token accounts, 6960 * (128 + 165) * 4 = 8_157_120.
pub const JANITOR_OPEN_ORDERS_ACCOUNT_REWARD: u64 =
    LEGACY_LAMPORTS_PER_BYTE * (ACCOUNT_STORAGE_OVERHEAD + TOKEN_ACCOUNT_LEN) * 4;

pub fn is_sunset(unix_timestamp: i64) -> bool {
    unix_timestamp >= SUNSET_TS
}

/// The janitor's powers only apply after [`SUNSET_TS`], before that it is an ordinary key.
pub fn janitor_active() -> Result<bool> {
    Ok(is_sunset(Clock::get()?.unix_timestamp))
}

/// Quote a market may still count per orphaned event once every claim on it was settled. A maker
/// locks the fees of a bid rounded up per order and is released them rounded down per fill; the
/// difference normally reaches `fees_available` when the maker settles. A maker that vanished
/// with events in flight (executed against the janitor's sink, `Market::orphaned_events`) never
/// settles, so less than one native unit per fill of its orders stays behind. The allowance is a
/// generous multiple of that; a market without positive maker fees locks none and stays exact.
pub const ORPHAN_FEE_ROUNDOFF_PER_EVENT: u64 = 1_000;

pub fn orphan_fee_roundoff(market: &Market) -> u64 {
    if market.maker_fee > 0 {
        ORPHAN_FEE_ROUNDOFF_PER_EVENT * market.orphaned_events
    } else {
        0
    }
}

/// `Market::is_empty` for the janitor: no open orders account has a claim left (deposit totals,
/// up to the orphan fee roundoff) and the fees were swept
pub fn janitor_market_empty(market: &Market) -> bool {
    market.base_deposit_total == 0
        && market.quote_deposit_total <= orphan_fee_roundoff(market)
        && market.fees_available == 0
        && market.referrer_rebates_accrued == 0
}

pub fn require_not_sunset() -> Result<()> {
    require!(
        !is_sunset(Clock::get()?.unix_timestamp),
        OpenBookError::ProgramSunset
    );
    Ok(())
}

/// Close a program owned `account`: `reward` (capped at its balance) goes to the janitor, the
/// rest to `owner`. `fee_free`, if given, is closed as well and goes to `owner` in full.
///
/// `rent` is the current Rent sysvar, only used for the owner check: the runtime rejects a
/// transaction that leaves `owner` below the rent exempt minimum (wallet that does not exist yet,
/// or a rent paying wallet after a reset under SIMD-0438). A refund that falls short of it is
/// topped up out of the reward; if the whole balance does not reach it, everything goes to the
/// janitor.
///
/// Returns `(reward, refund)`.
pub fn close_with_reward<'info>(
    account: &AccountInfo<'info>,
    fee_free: Option<&AccountInfo<'info>>,
    owner: &AccountInfo<'info>,
    janitor: &AccountInfo<'info>,
    reward: u64,
    rent: &Rent,
) -> Result<(u64, u64)> {
    let lamports = account.lamports();
    let mut reward = reward.min(lamports);
    let mut refund = lamports - reward + fee_free.map_or(0, |a| a.lamports());

    let owner_lamports = owner.lamports();
    if refund > 0 && !rent.is_exempt(owner_lamports + refund, owner.data_len()) {
        let shortfall = rent.minimum_balance(owner.data_len()) - (owner_lamports + refund);
        if shortfall <= reward {
            reward -= shortfall;
            refund += shortfall;
        } else {
            reward += refund;
            refund = 0;
        }
    }

    for closed in std::iter::once(account).chain(fee_free) {
        **closed.try_borrow_mut_lamports()? = 0;
        closed.assign(&System::id());
        closed.realloc(0, false)?;
    }
    **janitor.try_borrow_mut_lamports()? += reward;
    **owner.try_borrow_mut_lamports()? += refund;

    Ok((reward, refund))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{OpenOrdersAccount, OpenOrdersIndexer};
    use anchor_lang::solana_program::program_pack::Pack;
    use anchor_spl::token::spl_token;

    const TOKEN_ACCOUNT_LEN: usize = spl_token::state::Account::LEN;
    const OPEN_ORDERS_ACCOUNT_LEN: usize = 8 + std::mem::size_of::<OpenOrdersAccount>();

    /// Rent at `lamports_per_byte` for a rent exempt account.
    /// Encoded as before SIMD-0194 (per year, two years exemption).
    fn rent(lamports_per_byte: u64) -> Rent {
        Rent {
            lamports_per_byte_year: lamports_per_byte / 2,
            exemption_threshold: 2.0,
            burn_percent: 50,
        }
    }

    /// Same rent, encoded as after SIMD-0194 (per byte, exemption threshold 1).
    fn rent_per_byte(lamports_per_byte: u64) -> Rent {
        Rent {
            lamports_per_byte_year: lamports_per_byte,
            exemption_threshold: 1.0,
            burn_percent: 50,
        }
    }

    const LEGACY: u64 = 6960;
    /// SIMD-0437 steps 1 and 2, mainnet since 2026-09
    const STEP_2: u64 = 5080;
    /// SIMD-0437 step 5, -90%
    const FINAL: u64 = 696;

    struct TestAccount {
        key: Pubkey,
        owner: Pubkey,
        lamports: u64,
        /// 8 byte length header in front of the data, `realloc` writes it
        buffer: Vec<u8>,
    }

    impl TestAccount {
        fn new(owner: Pubkey, lamports: u64, data_len: usize) -> Self {
            let mut buffer = vec![0u8; 8 + data_len];
            buffer[..8].copy_from_slice(&(data_len as u64).to_le_bytes());
            Self {
                key: Pubkey::new_unique(),
                owner,
                lamports,
                buffer,
            }
        }

        fn info(&mut self) -> AccountInfo {
            AccountInfo::new(
                &self.key,
                false,
                true,
                &mut self.lamports,
                &mut self.buffer[8..],
                &self.owner,
                false,
                0,
            )
        }
    }

    /// Close an account funded at `funded_at` lamports/byte while the rent is `rent`.
    fn close(
        data_len: usize,
        funded_at: u64,
        owner_lamports: u64,
        reward: impl Fn(&Rent) -> u64,
        rent: &Rent,
    ) -> (u64, u64, u64, u64) {
        close_with_fee_free(data_len, funded_at, 0, owner_lamports, reward, rent)
    }

    /// Same, also closing a fee free account of `fee_free` lamports (an indexer) if non-zero.
    fn close_with_fee_free(
        data_len: usize,
        funded_at: u64,
        fee_free: u64,
        owner_lamports: u64,
        reward: impl Fn(&Rent) -> u64,
        rent: &Rent,
    ) -> (u64, u64, u64, u64) {
        let funded = rent_per_byte(funded_at).minimum_balance(data_len);
        let mut account = TestAccount::new(crate::ID, funded, data_len);
        let mut indexer = TestAccount::new(crate::ID, fee_free, OpenOrdersIndexer::space(0));
        let mut owner = TestAccount::new(System::id(), owner_lamports, 0);
        let mut janitor = TestAccount::new(System::id(), 1_000_000_000, 0);

        let account_info = account.info();
        let indexer_info = indexer.info();
        let owner_info = owner.info();
        let janitor_info = janitor.info();
        let (paid_reward, refund) = close_with_reward(
            &account_info,
            (fee_free > 0).then_some(&indexer_info),
            &owner_info,
            &janitor_info,
            reward(rent),
            rent,
        )
        .unwrap();

        for closed in [&account_info, &indexer_info]
            .into_iter()
            .take(1 + (fee_free > 0) as usize)
        {
            assert_eq!(closed.lamports(), 0);
            assert_eq!(closed.data_len(), 0);
            assert_eq!(*closed.owner, System::id());
        }
        assert_eq!(
            paid_reward + refund,
            funded + fee_free,
            "lamports are conserved"
        );
        assert_eq!(janitor_info.lamports(), 1_000_000_000 + paid_reward);
        assert_eq!(owner_info.lamports(), owner_lamports + refund);
        if refund > 0 {
            assert!(rent.is_exempt(owner_info.lamports(), 0));
        }
        (funded, paid_reward, refund, owner_info.lamports())
    }

    fn ooa_reward(_: &Rent) -> u64 {
        JANITOR_OPEN_ORDERS_ACCOUNT_REWARD
    }

    #[test]
    fn test_rent_encodings_agree() {
        for per_byte in [LEGACY, STEP_2, FINAL] {
            for len in [0, TOKEN_ACCOUNT_LEN, OPEN_ORDERS_ACCOUNT_LEN] {
                assert_eq!(
                    rent(per_byte).minimum_balance(len),
                    rent_per_byte(per_byte).minimum_balance(len)
                );
            }
        }
        assert_eq!(rent(LEGACY).minimum_balance(TOKEN_ACCOUNT_LEN), 2_039_280);
        assert_eq!(rent(STEP_2).minimum_balance(TOKEN_ACCOUNT_LEN), 1_488_440);
        assert_eq!(rent(FINAL).minimum_balance(TOKEN_ACCOUNT_LEN), 203_928);
        assert_eq!(rent(FINAL).minimum_balance(0), 89_088);
    }

    #[test]
    fn test_reward_fixed_at_legacy_rent() {
        assert_eq!(
            JANITOR_OPEN_ORDERS_ACCOUNT_REWARD,
            4 * rent(LEGACY).minimum_balance(TOKEN_ACCOUNT_LEN)
        );
        assert_eq!(JANITOR_OPEN_ORDERS_ACCOUNT_REWARD, 8_157_120);
    }

    #[test]
    fn test_open_orders_account_reward_under_simd_0437() {
        // an account funded at the legacy rate, closed at every step of the reduction: the split
        // does not depend on the current rent
        for per_byte in [LEGACY, STEP_2, FINAL] {
            for rent in [rent(per_byte), rent_per_byte(per_byte)] {
                let (funded, reward, refund, _) =
                    close(OPEN_ORDERS_ACCOUNT_LEN, LEGACY, 0, ooa_reward, &rent);
                assert_eq!(funded, 9_688_320);
                assert_eq!(reward, 8_157_120);
                assert_eq!(refund, 1_531_200);
            }
        }
    }

    #[test]
    fn test_open_orders_account_reward_capped() {
        // created after the reduction, holds less than the reward
        for per_byte in [LEGACY, FINAL] {
            let (funded, reward, refund, owner_lamports) = close(
                OPEN_ORDERS_ACCOUNT_LEN,
                FINAL,
                0,
                ooa_reward,
                &rent(per_byte),
            );
            assert_eq!(funded, 968_832);
            assert_eq!((reward, refund, owner_lamports), (funded, 0, 0));
        }
    }

    #[test]
    fn test_missing_owner_topped_up_from_reward() {
        let rent = rent(FINAL);
        let len = OpenOrdersIndexer::space(1);
        let funded = rent_per_byte(LEGACY).minimum_balance(len);
        let minimum = rent.minimum_balance(0);
        // leaves 1_000 lamports, below the 0 byte minimum of 89_088
        let dust_reward = |_: &Rent| funded - 1_000;

        // owner wallet does not exist: the reward pays for creating it
        let (_, reward, refund, owner_lamports) = close(len, LEGACY, 0, dust_reward, &rent);
        assert_eq!(
            (reward, refund, owner_lamports),
            (funded - minimum, minimum, minimum)
        );

        // rent exempt owner wallet: it gets the dust
        let (_, reward, refund, owner_lamports) = close(len, LEGACY, minimum, dust_reward, &rent);
        assert_eq!(
            (reward, refund, owner_lamports),
            (funded - 1_000, 1_000, minimum + 1_000)
        );

        // the whole balance can not create the wallet: everything goes to the janitor
        let (funded, reward, refund, owner_lamports) =
            close(0, FINAL, 0, |_| 0, &rent_per_byte(LEGACY));
        assert_eq!((reward, refund, owner_lamports), (funded, 0, 0));
    }

    #[test]
    fn test_indexer_closed_with_open_orders_account() {
        // an account funded after the reduction holds less than the reward, the indexer goes to
        // the owner without a fee
        let indexer = rent_per_byte(FINAL).minimum_balance(OpenOrdersIndexer::space(1));
        let (funded, reward, refund, _) = close_with_fee_free(
            OPEN_ORDERS_ACCOUNT_LEN,
            FINAL,
            indexer,
            0,
            ooa_reward,
            &rent(FINAL),
        );
        assert_eq!((reward, refund), (funded, indexer));

        // at legacy rent the indexer alone does not create the owner wallet: the reward is
        // reduced so that it does
        let minimum = rent(LEGACY).minimum_balance(0);
        let (funded, reward, refund, owner_lamports) = close_with_fee_free(
            OPEN_ORDERS_ACCOUNT_LEN,
            FINAL,
            indexer,
            0,
            ooa_reward,
            &rent(LEGACY),
        );
        assert_eq!(
            (reward, refund, owner_lamports),
            (funded + indexer - minimum, minimum, minimum)
        );
    }

    #[test]
    fn test_rent_paying_owner_topped_up_from_reward() {
        // a wallet that is rent paying after a reset to legacy rent (SIMD-0438) may only be
        // credited if it becomes rent exempt
        let rent = rent(LEGACY);
        let minimum = rent.minimum_balance(0);
        let wallet = rent_per_byte(FINAL).minimum_balance(0);
        let (funded, reward, refund, owner_lamports) = close(
            OPEN_ORDERS_ACCOUNT_LEN,
            LEGACY,
            wallet,
            |_| 9_000_000,
            &rent,
        );
        assert_eq!(owner_lamports, minimum);
        assert_eq!(
            (reward, refund),
            (funded - (minimum - wallet), minimum - wallet)
        );

        // a large enough refund lifts it above the minimum by itself
        let (funded, reward, refund, owner_lamports) =
            close(OPEN_ORDERS_ACCOUNT_LEN, LEGACY, wallet, |_| 0, &rent);
        assert_eq!((reward, refund), (0, funded));
        assert!(rent.is_exempt(owner_lamports, 0));
    }

    #[test]
    fn test_sunset() {
        assert!(!is_sunset(SUNSET_TS - 1));
        assert!(is_sunset(SUNSET_TS));
    }
}
