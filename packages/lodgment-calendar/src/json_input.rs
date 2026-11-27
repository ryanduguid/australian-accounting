// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Reject duplicate object fields before serde_json constructs its value map.
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use std::collections::HashSet;
use std::fmt;

struct Unique;
impl<'de> Deserialize<'de> for Unique {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(UniqueVisitor)
    }
}
struct UniqueVisitor;
impl<'de> Visitor<'de> for UniqueVisitor {
    type Value = Unique;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("JSON with unique object fields")
    }
    fn visit_bool<E: de::Error>(self, _: bool) -> Result<Unique, E> {
        Ok(Unique)
    }
    fn visit_i64<E: de::Error>(self, _: i64) -> Result<Unique, E> {
        Ok(Unique)
    }
    fn visit_u64<E: de::Error>(self, _: u64) -> Result<Unique, E> {
        Ok(Unique)
    }
    fn visit_str<E: de::Error>(self, _: &str) -> Result<Unique, E> {
        Ok(Unique)
    }
    fn visit_unit<E: de::Error>(self) -> Result<Unique, E> {
        Ok(Unique)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Unique, A::Error> {
        while seq.next_element::<Unique>()?.is_some() {}
        Ok(Unique)
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Unique, A::Error> {
        let mut keys = HashSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !keys.insert(key.clone()) {
                return Err(de::Error::custom(format!("duplicate JSON field: {key:?}")));
            }
            map.next_value::<Unique>()?;
        }
        Ok(Unique)
    }
}

pub fn parse(bytes: &[u8]) -> Result<serde_json::Value, serde_json::Error> {
    serde_json::from_slice::<Unique>(bytes)?;
    serde_json::from_slice(bytes)
}

#[cfg(test)]
mod tests {
    use super::{Unique, parse};
    use serde::Deserialize;

    /// With serde_json's `arbitrary_precision` (this crate's build) a float reaches the
    /// check as a number token inside a map, so the check needs no float arm.
    #[test]
    fn floats_are_read_and_duplicate_fields_refused() {
        let value = parse(br#"{"a": [1.5, -2.5e3], "b": {"c": 0.0}}"#).unwrap();
        assert_eq!(value["a"][0].to_string(), "1.5");
        assert_eq!(value["a"][1].as_f64(), Some(-2500.0));
        let error = parse(br#"{"a": 1.5, "a": 2}"#).unwrap_err().to_string();
        assert!(error.starts_with("duplicate JSON field: \"a\""), "{error}");
    }

    #[test]
    fn a_value_json_cannot_hold_is_described_as_the_check_expects() {
        let bytes = serde::de::value::BytesDeserializer::<serde::de::value::Error>::new(b"x");
        assert_eq!(
            Unique::deserialize(bytes).err().unwrap().to_string(),
            "invalid type: byte array, expected JSON with unique object fields"
        );
    }
}
