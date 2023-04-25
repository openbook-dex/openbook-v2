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

use core::{
    cell::Cell,
    fmt::{Debug, Formatter, Result as FmtResult, Write},
};

// This is an ugly hack to check whether a `Formatter` has `debug_lower_hex` or
// `debug_upper_hex`.
//
// We do a dummy write with format string "{:x?}" to get `debug_lower_hex`, and
// a dummy write with format string "{:X?}" to get `debug_upper_hex`. Each time,
// we get the flags using the deprecated `Formatter::flags`.

fn get_flags(f: &Formatter) -> u32 {
    #[allow(deprecated)]
    f.flags()
}

struct StoreFlags(Cell<u32>);

impl Debug for StoreFlags {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.0.set(get_flags(f));
        Ok(())
    }
}

struct Discard;

impl Write for Discard {
    fn write_str(&mut self, _s: &str) -> FmtResult {
        Ok(())
    }
}

pub enum IsDebugHex {
    No,
    Lower,
    Upper,
}

pub fn is_debug_hex(f: &Formatter) -> IsDebugHex {
    let flags = get_flags(f);
    // avoid doing unnecessary work if flags is zero
    if flags == 0 {
        return IsDebugHex::No;
    }

    let store_flags = StoreFlags(Cell::new(0));
    if write!(Discard, "{:x?}", store_flags).is_err() {
        return IsDebugHex::No;
    }
    let lower_flags = store_flags.0.get();
    if write!(Discard, "{:X?}", store_flags).is_err() {
        return IsDebugHex::No;
    }
    let upper_flags = store_flags.0.get();
    let lower_mask = lower_flags & !upper_flags;
    let upper_mask = upper_flags & !lower_flags;

    if flags & lower_mask != 0 {
        IsDebugHex::Lower
    } else if flags & upper_mask != 0 {
        IsDebugHex::Upper
    } else {
        IsDebugHex::No
    }
}
