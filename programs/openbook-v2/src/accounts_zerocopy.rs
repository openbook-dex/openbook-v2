use anchor_lang::prelude::*;
use anchor_lang::ZeroCopy;
use arrayref::array_ref;
use std::cell::RefMut;
use std::{cell::Ref, mem};

/// Functions should prefer to work with AccountReader where possible, to abstract over
/// AccountInfo and AccountSharedData. That way the functions become usable in the program
/// and in client code.
// NOTE: would love to use solana's ReadableAccount, but that's in solana_sdk -- unavailable for programs
pub trait AccountReader {
    fn owner(&self) -> &Pubkey;
    fn data(&self) -> &[u8];
}

/// Like AccountReader, but can also get the account pubkey
pub trait KeyedAccountReader: AccountReader {
    fn key(&self) -> &Pubkey;
}

/// A Ref to an AccountInfo - makes AccountInfo compatible with AccountReader
pub struct AccountInfoRef<'a, 'info: 'a> {
    pub key: &'info Pubkey,
    pub owner: &'info Pubkey,
    pub data: Ref<'a, &'info mut [u8]>,
}

impl<'a, 'info: 'a> AccountInfoRef<'a, 'info> {
    pub fn borrow(account_info: &'a AccountInfo<'info>) -> Result<Self> {
        Ok(Self {
            key: account_info.key,
            owner: account_info.owner,
            data: account_info
                .data
                .try_borrow()
                .map_err(|_| ProgramError::AccountBorrowFailed)?,
            // Why is the following not acceptable?
            //data: account_info.try_borrow_data()?,
        })
    }

    pub fn borrow_slice(ais: &'a [AccountInfo<'info>]) -> Result<Vec<Self>> {
        ais.iter().map(Self::borrow).collect()
    }
}

pub struct AccountInfoRefMut<'a, 'info: 'a> {
    pub key: &'info Pubkey,
    pub owner: &'info Pubkey,
    pub data: RefMut<'a, &'info mut [u8]>,
}

impl<'a, 'info: 'a> AccountInfoRefMut<'a, 'info> {
    pub fn borrow(account_info: &'a AccountInfo<'info>) -> Result<Self> {
        Ok(Self {
            key: account_info.key,
            owner: account_info.owner,
            data: account_info
                .data
                .try_borrow_mut()
                .map_err(|_| ProgramError::AccountBorrowFailed)?,
        })
    }

    pub fn borrow_slice(ais: &'a [AccountInfo<'info>]) -> Result<Vec<Self>> {
        ais.iter().map(Self::borrow).collect()
    }
}

impl<'info, 'a> AccountReader for AccountInfoRef<'info, 'a> {
    fn owner(&self) -> &Pubkey {
        self.owner
    }

    fn data(&self) -> &[u8] {
        &self.data
    }
}

impl<'info, 'a> AccountReader for AccountInfoRefMut<'info, 'a> {
    fn owner(&self) -> &Pubkey {
        self.owner
    }

    fn data(&self) -> &[u8] {
        &self.data
    }
}

impl<'info, 'a> KeyedAccountReader for AccountInfoRef<'info, 'a> {
    fn key(&self) -> &Pubkey {
        self.key
    }
}

impl<'info, 'a> KeyedAccountReader for AccountInfoRefMut<'info, 'a> {
    fn key(&self) -> &Pubkey {
        self.key
    }
}

#[cfg(feature = "solana-sdk")]
impl<T: solana_sdk::account::ReadableAccount> AccountReader for T {
    fn owner(&self) -> &Pubkey {
        self.owner()
    }

    fn data(&self) -> &[u8] {
        self.data()
    }
}

#[cfg(feature = "solana-sdk")]
#[derive(Clone)]
pub struct KeyedAccount {
    pub key: Pubkey,
    pub account: solana_sdk::account::Account,
}

#[cfg(feature = "solana-sdk")]
impl AccountReader for KeyedAccount {
    fn owner(&self) -> &Pubkey {
        self.account.owner()
    }

    fn data(&self) -> &[u8] {
        self.account.data()
    }
}

#[cfg(feature = "solana-sdk")]
impl KeyedAccountReader for KeyedAccount {
    fn key(&self) -> &Pubkey {
        &self.key
    }
}

#[cfg(feature = "solana-sdk")]
#[derive(Clone)]
pub struct KeyedAccountSharedData {
    pub key: Pubkey,
    pub data: solana_sdk::account::AccountSharedData,
}

#[cfg(feature = "solana-sdk")]
impl KeyedAccountSharedData {
    pub fn new(key: Pubkey, data: solana_sdk::account::AccountSharedData) -> Self {
        Self { key, data }
    }
}

#[cfg(feature = "solana-sdk")]
impl AccountReader for KeyedAccountSharedData {
    fn owner(&self) -> &Pubkey {
        AccountReader::owner(&self.data)
    }

    fn data(&self) -> &[u8] {
        AccountReader::data(&self.data)
    }
}

#[cfg(feature = "solana-sdk")]
impl KeyedAccountReader for KeyedAccountSharedData {
    fn key(&self) -> &Pubkey {
        &self.key
    }
}

//
// Common traits for loading from account data.
//

pub trait LoadZeroCopy {
    /// Using AccountLoader forces a AccountInfo.clone() and then binds the loaded
    /// lifetime to the AccountLoader's lifetime. This function avoids both.
    /// It checks the account owner and discriminator, then casts the data.
    fn load<T: ZeroCopy + Owner>(&self) -> Result<&T>;

    /// Same as load(), but doesn't check the discriminator or owner.
    fn load_fully_unchecked<T: ZeroCopy + Owner>(&self) -> Result<&T>;
}

pub trait LoadMutZeroCopy {
    /// Same as load(), but mut
    fn load_mut<T: ZeroCopy + Owner>(&mut self) -> Result<&mut T>;

    /// Same as load_fully_unchecked(), but mut
    fn load_mut_fully_unchecked<T: ZeroCopy + Owner>(&mut self) -> Result<&mut T>;
}

pub trait LoadZeroCopyRef {
    /// Using AccountLoader forces a AccountInfo.clone() and then binds the loaded
    /// lifetime to the AccountLoader's lifetime. This function avoids both.
    /// It checks the account owner and discriminator, then casts the data.
    fn load<T: ZeroCopy + Owner>(&self) -> Result<Ref<T>>;

    /// Same as load(), but doesn't check the discriminator or owner.
    fn load_fully_unchecked<T: ZeroCopy + Owner>(&self) -> Result<Ref<T>>;
}

pub trait LoadMutZeroCopyRef {
    /// Same as load(), but mut
    fn load_mut<T: ZeroCopy + Owner>(&self) -> Result<RefMut<T>>;

    /// Same as load_fully_unchecked(), but mut
    fn load_mut_fully_unchecked<T: ZeroCopy + Owner>(&self) -> Result<RefMut<T>>;
}

impl<A: AccountReader> LoadZeroCopy for A {
    fn load<T: ZeroCopy + Owner>(&self) -> Result<&T> {
        if self.owner() != &T::owner() {
            return Err(ErrorCode::AccountOwnedByWrongProgram.into());
        }

        let data = self.data();
        if data.len() < 8 {
            return Err(ErrorCode::AccountDiscriminatorNotFound.into());
        }
        let disc_bytes = array_ref![data, 0, 8];
        if disc_bytes != &T::discriminator() {
            return Err(ErrorCode::AccountDiscriminatorMismatch.into());
        }

        Ok(bytemuck::from_bytes(&data[8..mem::size_of::<T>() + 8]))
    }

    fn load_fully_unchecked<T: ZeroCopy + Owner>(&self) -> Result<&T> {
        Ok(bytemuck::from_bytes(
            &self.data()[8..mem::size_of::<T>() + 8],
        ))
    }
}

impl<'info, 'a> LoadMutZeroCopy for AccountInfoRefMut<'info, 'a> {
    fn load_mut<T: ZeroCopy + Owner>(&mut self) -> Result<&mut T> {
        if self.owner != &T::owner() {
            return Err(ErrorCode::AccountOwnedByWrongProgram.into());
        }

        if self.data.len() < 8 {
            return Err(ErrorCode::AccountDiscriminatorNotFound.into());
        }
        let disc_bytes = array_ref![self.data, 0, 8];
        if disc_bytes != &T::discriminator() {
            return Err(ErrorCode::AccountDiscriminatorMismatch.into());
        }

        Ok(bytemuck::from_bytes_mut(
            &mut self.data[8..mem::size_of::<T>() + 8],
        ))
    }

    fn load_mut_fully_unchecked<T: ZeroCopy + Owner>(&mut self) -> Result<&mut T> {
        Ok(bytemuck::from_bytes_mut(
            &mut self.data[8..mem::size_of::<T>() + 8],
        ))
    }
}

impl<'info> LoadZeroCopyRef for AccountInfo<'info> {
    fn load<T: ZeroCopy + Owner>(&self) -> Result<Ref<T>> {
        if self.owner != &T::owner() {
            return Err(ErrorCode::AccountOwnedByWrongProgram.into());
        }

        let data = self.try_borrow_data()?;
        if data.len() < 8 {
            return Err(ErrorCode::AccountDiscriminatorNotFound.into());
        }

        let disc_bytes = array_ref![data, 0, 8];
        if disc_bytes != &T::discriminator() {
            return Err(ErrorCode::AccountDiscriminatorMismatch.into());
        }

        Ok(Ref::map(data, |data| {
            bytemuck::from_bytes(&data[8..mem::size_of::<T>() + 8])
        }))
    }

    fn load_fully_unchecked<T: ZeroCopy + Owner>(&self) -> Result<Ref<T>> {
        let data = self.try_borrow_data()?;
        Ok(Ref::map(data, |data| {
            bytemuck::from_bytes(&data[8..mem::size_of::<T>() + 8])
        }))
    }
}

impl<'info> LoadMutZeroCopyRef for AccountInfo<'info> {
    fn load_mut<T: ZeroCopy + Owner>(&self) -> Result<RefMut<T>> {
        if self.owner != &T::owner() {
            return Err(ErrorCode::AccountOwnedByWrongProgram.into());
        }

        let data = self.try_borrow_mut_data()?;
        if data.len() < 8 {
            return Err(ErrorCode::AccountDiscriminatorNotFound.into());
        }

        let disc_bytes = array_ref![data, 0, 8];
        if disc_bytes != &T::discriminator() {
            return Err(ErrorCode::AccountDiscriminatorMismatch.into());
        }

        Ok(RefMut::map(data, |data| {
            bytemuck::from_bytes_mut(&mut data[8..mem::size_of::<T>() + 8])
        }))
    }

    fn load_mut_fully_unchecked<T: ZeroCopy + Owner>(&self) -> Result<RefMut<T>> {
        let data = self.try_borrow_mut_data()?;
        Ok(RefMut::map(data, |data| {
            bytemuck::from_bytes_mut(&mut data[8..mem::size_of::<T>() + 8])
        }))
    }
}
