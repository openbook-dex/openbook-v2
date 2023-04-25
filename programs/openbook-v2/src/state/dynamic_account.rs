use std::cell::{Ref, RefMut};

use anchor_lang::prelude::*;

/// Header is created by scanning and parsing the dynamic portion of the account.
/// This stores useful information e.g. offsets to easily seek into dynamic content.
pub trait DynamicHeader: Sized {
    /// Builds header by scanning and parsing the dynamic portion of the account.
    fn from_bytes(dynamic_data: &[u8]) -> Result<Self>;

    /// initializes a header on the dynamic portion of a new account
    fn initialize(dynamic_data: &mut [u8]) -> Result<()>;
}

#[derive(Clone)]
pub struct DynamicAccount<Header, Fixed, Dynamic> {
    pub header: Header,
    pub fixed: Fixed,
    pub dynamic: Dynamic,
}

// Want to generalize over:
// - T (which is Borrow<T>)
// - &T (which is Borrow<T> and Deref<Target=T>)
// - Ref<T> (which is Deref<T>)
pub trait DerefOrBorrow<T: ?Sized> {
    fn deref_or_borrow(&self) -> &T;
}

impl<T: ?Sized> DerefOrBorrow<T> for T {
    fn deref_or_borrow(&self) -> &T {
        self
    }
}

impl<T: ?Sized> DerefOrBorrow<T> for &T {
    fn deref_or_borrow(&self) -> &T {
        self
    }
}

impl<T: Sized> DerefOrBorrow<[T]> for Vec<T> {
    fn deref_or_borrow(&self) -> &[T] {
        self
    }
}

impl<T: ?Sized> DerefOrBorrow<T> for &mut T {
    fn deref_or_borrow(&self) -> &T {
        self
    }
}

impl<'a, T: ?Sized> DerefOrBorrow<T> for Ref<'a, T> {
    fn deref_or_borrow(&self) -> &T {
        self
    }
}

impl<'a, T: ?Sized> DerefOrBorrow<T> for RefMut<'a, T> {
    fn deref_or_borrow(&self) -> &T {
        self
    }
}

pub trait DerefOrBorrowMut<T: ?Sized> {
    fn deref_or_borrow_mut(&mut self) -> &mut T;
}

impl<T: ?Sized> DerefOrBorrowMut<T> for T {
    fn deref_or_borrow_mut(&mut self) -> &mut T {
        self
    }
}

impl<T: ?Sized> DerefOrBorrowMut<T> for &mut T {
    fn deref_or_borrow_mut(&mut self) -> &mut T {
        self
    }
}

impl<'a, T: ?Sized> DerefOrBorrowMut<T> for RefMut<'a, T> {
    fn deref_or_borrow_mut(&mut self) -> &mut T {
        self
    }
}

impl<T: Sized> DerefOrBorrowMut<[T]> for Vec<T> {
    fn deref_or_borrow_mut(&mut self) -> &mut [T] {
        self
    }
}
