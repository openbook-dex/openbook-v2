#![allow(dead_code)]

use bytemuck::{bytes_of, Contiguous};
use fixed::types::I80F48;
use solana_program::instruction::InstructionError;
use solana_program::program_error::ProgramError;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::transaction::TransactionError;
use solana_sdk::transport::TransportError;

pub fn gen_signer_seeds<'a>(nonce: &'a u64, acc_pk: &'a Pubkey) -> [&'a [u8]; 2] {
    [acc_pk.as_ref(), bytes_of(nonce)]
}

pub fn gen_signer_key(
    nonce: u64,
    acc_pk: &Pubkey,
    program_id: &Pubkey,
) -> Result<Pubkey, ProgramError> {
    let seeds = gen_signer_seeds(&nonce, acc_pk);
    Ok(Pubkey::create_program_address(&seeds, program_id)?)
}

pub fn create_signer_key_and_nonce(program_id: &Pubkey, acc_pk: &Pubkey) -> (Pubkey, u64) {
    for i in 0..=u64::MAX_VALUE {
        if let Ok(pk) = gen_signer_key(i, acc_pk, program_id) {
            return (pk, i);
        }
    }
    panic!("Could not generate signer key");
}

pub fn clone_keypair(keypair: &Keypair) -> Keypair {
    Keypair::from_base58_string(&keypair.to_base58_string())
}

// Add clone() to Keypair, totally safe in tests
pub trait ClonableKeypair {
    fn clone(&self) -> Self;
}
impl ClonableKeypair for Keypair {
    fn clone(&self) -> Self {
        clone_keypair(self)
    }
}

/// A Keypair-like struct that's Clone and Copy and can be into()ed to a Keypair
///
/// The regular Keypair is neither Clone nor Copy because the key data is sensitive
/// and should not be copied needlessly. That just makes things difficult for tests.
#[derive(Clone, Copy, Debug)]
pub struct TestKeypair([u8; 64]);
impl TestKeypair {
    pub fn new() -> Self {
        Keypair::new().into()
    }

    pub fn to_keypair(&self) -> Keypair {
        Keypair::from_bytes(&self.0).unwrap()
    }

    pub fn pubkey(&self) -> Pubkey {
        solana_sdk::signature::Signer::pubkey(&self.to_keypair())
    }
}
impl Default for TestKeypair {
    fn default() -> Self {
        Self([0; 64])
    }
}
impl<T: std::borrow::Borrow<Keypair>> From<T> for TestKeypair {
    fn from(k: T) -> Self {
        Self(k.borrow().to_bytes())
    }
}
#[allow(clippy::from_over_into)]
impl Into<Keypair> for &TestKeypair {
    fn into(self) -> Keypair {
        self.to_keypair()
    }
}

pub fn assert_openbook_error<T>(
    result: &Result<T, TransportError>,
    expected_error: u32,
    comment: String,
) {
    #[allow(clippy::collapsible_match)]
    match result {
        Ok(_) => panic!("No error returned"),
        Err(TransportError::TransactionError(tx_err)) => match tx_err {
            TransactionError::InstructionError(_, err) => match err {
                InstructionError::Custom(err_num) => {
                    assert_eq!(*err_num, expected_error, "{}", comment);
                }
                _ => panic!("Not an openbook error"),
            },
            _ => panic!("Not an openbook error"),
        },
        _ => panic!("Not an openbook error"),
    }
}

pub fn assert_equal_fixed_f64(value: I80F48, expected: f64, max_error: f64) -> bool {
    let ok = (value.to_num::<f64>() - expected).abs() < max_error;
    if !ok {
        println!("comparison failed: value: {value}, expected: {expected}");
    }
    ok
}

pub fn assert_equal_f64_f64(value: f64, expected: f64, max_error: f64) -> bool {
    let ok = (value - expected).abs() < max_error;
    if !ok {
        println!("comparison failed: value: {value}, expected: {expected}");
    }
    ok
}
