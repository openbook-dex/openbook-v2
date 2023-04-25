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
    types::extra::{LeEqU128, LeEqU16, LeEqU32, LeEqU64, LeEqU8},
    FixedI128, FixedI16, FixedI32, FixedI64, FixedI8, FixedU128, FixedU16, FixedU32, FixedU64,
    FixedU8, Unwrapped, Wrapping,
};
use serde::{
    de::{Deserialize, Deserializer, Error as DeError},
    ser::{Serialize, Serializer},
};
#[cfg(not(feature = "serde-str"))]
use {
    core::fmt::{Formatter, Result as FmtResult},
    serde::{
        de::{MapAccess, SeqAccess, Visitor},
        ser::SerializeStruct,
    },
};

macro_rules! serde_fixed {
    ($Fixed:ident($LeEqU:ident) is $TBits:ident name $Name:expr) => {
        impl<Frac: $LeEqU> Serialize for $Fixed<Frac> {
            #[cfg(not(feature = "serde-str"))]
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let bits = self.to_bits();
                let mut state = serializer.serialize_struct($Name, 1)?;
                state.serialize_field("bits", &bits)?;
                state.end()
            }

            #[cfg(feature = "serde-str")]
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                if serializer.is_human_readable() {
                    self.to_string().serialize(serializer)
                } else {
                    self.to_bits().serialize(serializer)
                }
            }
        }

        impl<Frac: $LeEqU> Serialize for Wrapping<$Fixed<Frac>> {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                self.0.serialize(serializer)
            }
        }

        impl<Frac: $LeEqU> Serialize for Unwrapped<$Fixed<Frac>> {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                self.0.serialize(serializer)
            }
        }

        impl<'de, Frac: $LeEqU> Deserialize<'de> for $Fixed<Frac> {
            #[cfg(not(feature = "serde-str"))]
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                struct FixedVisitor;

                impl<'de> Visitor<'de> for FixedVisitor {
                    type Value = $TBits;

                    fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                        formatter.write_str("struct ")?;
                        formatter.write_str($Name)
                    }

                    fn visit_seq<V: SeqAccess<'de>>(self, mut seq: V) -> Result<$TBits, V::Error> {
                        let bits = seq
                            .next_element()?
                            .ok_or_else(|| DeError::invalid_length(0, &self))?;
                        Ok(bits)
                    }

                    fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<$TBits, V::Error> {
                        let mut bits = None;
                        while let Some(key) = map.next_key()? {
                            match key {
                                Field::Bits => {
                                    if bits.is_some() {
                                        return Err(DeError::duplicate_field("bits"));
                                    }
                                    bits = Some(map.next_value()?);
                                }
                            }
                        }
                        let bits = bits.ok_or_else(|| DeError::missing_field("bits"))?;
                        Ok(bits)
                    }
                }

                let bits = deserializer.deserialize_struct($Name, FIELDS, FixedVisitor)?;
                Ok($Fixed::from_bits(bits))
            }

            #[cfg(feature = "serde-str")]
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                if deserializer.is_human_readable() {
                    String::deserialize(deserializer)?
                        .parse()
                        .map_err(|e| DeError::custom(format_args!("parse error: {}", e)))
                } else {
                    $TBits::deserialize(deserializer).map($Fixed::from_bits)
                }
            }
        }

        impl<'de, Frac: $LeEqU> Deserialize<'de> for Wrapping<$Fixed<Frac>> {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                $Fixed::deserialize(deserializer).map(Wrapping)
            }
        }

        impl<'de, Frac: $LeEqU> Deserialize<'de> for Unwrapped<$Fixed<Frac>> {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                $Fixed::deserialize(deserializer).map(Unwrapped)
            }
        }
    };
}

serde_fixed! { FixedI8(LeEqU8) is i8 name "FixedI8" }
serde_fixed! { FixedI16(LeEqU16) is i16 name "FixedI16" }
serde_fixed! { FixedI32(LeEqU32) is i32 name "FixedI32" }
serde_fixed! { FixedI64(LeEqU64) is i64 name "FixedI64" }
serde_fixed! { FixedI128(LeEqU128) is i128 name "FixedI128" }
serde_fixed! { FixedU8(LeEqU8) is u8 name "FixedU8" }
serde_fixed! { FixedU16(LeEqU16) is u16 name "FixedU16" }
serde_fixed! { FixedU32(LeEqU32) is u32 name "FixedU32" }
serde_fixed! { FixedU64(LeEqU64) is u64 name "FixedU64" }
serde_fixed! { FixedU128(LeEqU128) is u128 name "FixedU128" }

#[cfg(not(feature = "serde-str"))]
const FIELDS: &[&str] = &["bits"];

#[cfg(not(feature = "serde-str"))]
enum Field {
    Bits,
}

#[cfg(not(feature = "serde-str"))]
impl<'de> Deserialize<'de> for Field {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Field, D::Error> {
        struct FieldVisitor;

        impl<'de> Visitor<'de> for FieldVisitor {
            type Value = Field;

            fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                formatter.write_str("`bits`")
            }

            fn visit_str<E: DeError>(self, value: &str) -> Result<Field, E> {
                match value {
                    "bits" => Ok(Field::Bits),
                    _ => Err(DeError::unknown_field(value, FIELDS)),
                }
            }
        }

        deserializer.deserialize_identifier(FieldVisitor)
    }
}
