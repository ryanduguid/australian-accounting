// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Keep JSON integer/decimal types and exact decimal values without serde's
//! synthetic-number map representation colliding with a real user object.
use div7a_loan_review::decimal::Decimal;
use indexmap::IndexMap;
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::value::RawValue;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Json {
    Null,
    Bool(bool),
    String(String),
    Number { decimal: bool, value: Decimal },
    Array(Vec<Json>),
    Object(IndexMap<String, Json>),
}

impl PartialEq for Json {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            (
                Self::Number {
                    decimal: a,
                    value: x,
                },
                Self::Number {
                    decimal: b,
                    value: y,
                },
            ) => a == b && x.equals(y).unwrap_or(false),
            (Self::Array(a), Self::Array(b)) => a == b,
            (Self::Object(a), Self::Object(b)) => a == b,
            _ => false,
        }
    }
}

impl Json {
    pub fn object(&self) -> Option<&IndexMap<String, Json>> {
        if let Self::Object(value) = self {
            Some(value)
        } else {
            None
        }
    }
    pub fn array(&self) -> Option<&[Json]> {
        if let Self::Array(value) = self {
            Some(value)
        } else {
            None
        }
    }
    pub fn string(&self) -> Option<&str> {
        if let Self::String(value) = self {
            Some(value)
        } else {
            None
        }
    }
}

struct Members<'a>(Vec<(String, &'a RawValue)>);

impl<'de> Deserialize<'de> for Members<'de> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Unique;
        impl<'de> Visitor<'de> for Unique {
            type Value = Members<'de>;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a JSON object")
            }
            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Members<'de>, M::Error> {
                let mut result = Vec::new();
                let mut names = std::collections::BTreeSet::new();
                while let Some((key, value)) = map.next_entry::<String, &'de RawValue>()? {
                    if !names.insert(key.clone()) {
                        return Err(serde::de::Error::custom(format!(
                            "duplicate JSON key: {key}"
                        )));
                    }
                    result.push((key, value));
                }
                Ok(Members(result))
            }
        }
        deserializer.deserialize_map(Unique)
    }
}

fn charge(bytes: usize, remaining: &mut usize) -> Result<(), String> {
    *remaining = remaining
        .checked_sub(bytes)
        .ok_or("JSON exceeds 1 GiB of cumulative parsing work")?;
    Ok(())
}

fn decode(value: &RawValue, depth: usize, remaining: &mut usize) -> Result<Json, String> {
    if depth > 1_000 {
        return Err("JSON exceeds 1,000 containers".into());
    }
    // Borrow already validated tokens. Owning each intermediate subtree would
    // multiply a large string's allocation by its nesting depth.
    let raw = value.get();
    charge(raw.len(), remaining)?;
    let parse_error = |e: serde_json::Error| e.to_string();
    Ok(match raw.as_bytes()[0] {
        b'{' => Json::Object(
            serde_json::from_str::<Members>(raw)
                .map_err(parse_error)?
                .0
                .into_iter()
                .map(|(k, v)| decode(v, depth + 1, remaining).map(|v| (k, v)))
                .collect::<Result<_, _>>()?,
        ),
        b'[' => Json::Array(
            serde_json::from_str::<Vec<&RawValue>>(raw)
                .map_err(parse_error)?
                .into_iter()
                .map(|v| decode(v, depth + 1, remaining))
                .collect::<Result<_, _>>()?,
        ),
        b'"' => Json::String(serde_json::from_str(raw).map_err(parse_error)?),
        b't' => Json::Bool(true),
        b'f' => Json::Bool(false),
        b'n' => Json::Null,
        _ => {
            let decimal = raw.contains(['.', 'e', 'E']);
            if decimal && !raw.parse::<f64>().map_err(|e| e.to_string())?.is_finite() {
                return Err("non-finite JSON number".into());
            }
            if !decimal && raw.trim_start_matches('-').len() > 4_300 {
                return Err("JSON integer exceeds 4,300 digits".into());
            }
            let value = Decimal::parse(raw).map_err(|e| format!("invalid JSON number: {e:?}"))?;
            Json::Number { decimal, value }
        }
    })
}

pub fn parse(text: &str) -> Result<Json, String> {
    // Match the existing native metadata boundary's practical nesting limit.
    stacker::maybe_grow(64 * 1024, 16 * 1024 * 1024, || {
        let mut remaining = 1024 * 1024 * 1024;
        charge(text.len(), &mut remaining)?;
        let value: &RawValue = serde_json::from_str(text).map_err(|e| e.to_string())?;
        decode(value, 0, &mut remaining)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_types_precision_and_real_private_number_objects() {
        for (a, b, equal) in [
            ("true", "1", false),
            ("1", "1.0", false),
            ("1.20", "1.2", true),
            ("1e0", "1.0", true),
            ("-0.0", "0.00", true),
            ("1e-999", "0.0", false),
            ("1.0000000000000001", "1.0", false),
            (r#"{"$serde_json::private::Number":"1.2"}"#, "1.2", false),
            (r#"{"a":1,"b":2}"#, r#"{"b":2,"a":1}"#, true),
        ] {
            assert_eq!(parse(a).unwrap() == parse(b).unwrap(), equal, "{a} / {b}");
        }
        for invalid in [
            r#"{"x":1,"\u0078":2}"#,
            r#"{"a":[{"x":1,"x":2}]}"#,
            "NaN",
            "Infinity",
            "-Infinity",
            "1e999",
        ] {
            assert!(parse(invalid).is_err(), "{invalid}");
        }
    }
}
