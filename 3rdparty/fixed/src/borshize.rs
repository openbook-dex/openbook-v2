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

use crate::{
    FixedI128, FixedI16, FixedI32, FixedI64, FixedI8, FixedU128, FixedU16, FixedU32, FixedU64,
    FixedU8,
};
use borsh::maybestd::io::{Result, Write};
use borsh::{BorshDeserialize, BorshSerialize};

macro_rules! borsh_fixed {
    ($Fixed:ident is $TBits:ident) => {
        impl<Frac> BorshSerialize for $Fixed<Frac> {
            #[inline]
            fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
                <$TBits as BorshSerialize>::serialize(&self.bits, writer)
            }
        }

        impl<Frac> BorshDeserialize for $Fixed<Frac> {
            #[inline]
            fn deserialize(buf: &mut &[u8]) -> Result<Self> {
                <$TBits as BorshDeserialize>::deserialize(buf).map($Fixed::from_bits)
            }
        }
    };
}

borsh_fixed! { FixedI8 is i8 }
borsh_fixed! { FixedI16 is i16 }
borsh_fixed! { FixedI32 is i32 }
borsh_fixed! { FixedI64 is i64 }
borsh_fixed! { FixedI128 is i128 }
borsh_fixed! { FixedU8 is u8 }
borsh_fixed! { FixedU16 is u16 }
borsh_fixed! { FixedU32 is u32 }
borsh_fixed! { FixedU64 is u64 }
borsh_fixed! { FixedU128 is u128 }
