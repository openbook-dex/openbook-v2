// Copyright © 2018–2021 Trevor Spiteri

// This library is free software: you can redistribute it and/or
// modify it under the terms of either
//
//   * the Apache License, Version 2.0 or
//   * the MIT License
//
// at your option.
//
// You should have recieved copies of the Apache License and the MIT
// License along with the library. If not, see
// <https://www.apache.org/licenses/LICENSE-2.0> and
// <https://opensource.org/licenses/MIT>.

macro_rules! if_true {
    (True; $($rem:tt)+) => {
        $($rem)+
    };
    (False; $($rem:tt)+) => {};
}

macro_rules! if_signed {
    (Signed; $($rem:tt)+) => {
        $($rem)+
    };
    (Unsigned; $($rem:tt)+) => {};
}

macro_rules! if_unsigned {
    (Signed; $($rem:tt)+) => {};
    (Unsigned; $($rem:tt)+) => {
        $($rem)+
    };
}

macro_rules! if_signed_unsigned {
    (Signed, $signed:expr, $unsigned:expr $(,)?) => {
        $signed
    };
    (Unsigned, $signed:expr, $unsigned:expr $(,)?) => {
        $unsigned
    };
}

macro_rules! if_signed_else_empty_str {
    (Signed; $($signed:tt)*) => {
        concat!($($signed)*)
    };
    (Unsigned; $($signed:tt)*) => {
        ""
    };
}
macro_rules! if_unsigned_else_empty_str {
    (Signed; $($unsigned:tt)*) => {
        ""
    };
    (Unsigned; $($unsigned:tt)*) => {
        concat!($($unsigned)*)
    };
}

macro_rules! doc_comment {
    ($comment:expr; $($tt:tt)*) => {
        #[doc = $comment]
        $($tt)*
    };
}

macro_rules! comment {
    ($($comment:expr),* $(,)?; $($tt:tt)*) => {
        doc_comment! {
            concat!($($comment),*);
            $($tt)*
        }
    };
}

macro_rules! maybe_assert {
    ($($arg:tt)*) => {
#[cfg(not(feature = "debug-assert-in-release"))]
        debug_assert!($($arg)*);
#[cfg(feature = "debug-assert-in-release")]
        assert!($($arg)*);
    }
}
