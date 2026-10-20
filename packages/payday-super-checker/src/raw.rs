//! A value as the Python engine received it: a CSV cell, a JSON value from an
//! override file or the JSON API, or nothing at all. The engine's messages
//! quote `str()` and `repr()` of these, so both are reproduced.

use crate::pyjson::Json;
use crate::pytext;

#[derive(PartialEq)]
enum Number {
    Exact(num_bigint::BigInt),
    Inexact(f64),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Raw {
    None,
    Bool(bool),
    /// An int, as its decimal digits.
    Int(String),
    Float(f64),
    Str(String),
    List(Vec<Raw>),
    Dict(Vec<(String, Raw)>),
}

impl Raw {
    pub fn text(text: &str) -> Raw {
        Raw::Str(text.to_string())
    }

    /// `str(value)`.
    pub fn py_str(&self) -> String {
        match self {
            Raw::Str(s) => s.clone(),
            other => other.py_repr(),
        }
    }

    /// `repr(value)`.
    pub fn py_repr(&self) -> String {
        match self {
            Raw::None => "None".into(),
            Raw::Bool(true) => "True".into(),
            Raw::Bool(false) => "False".into(),
            Raw::Int(digits) => digits.clone(),
            Raw::Float(f) => float_repr(*f),
            Raw::Str(s) => pytext::repr(s),
            Raw::List(items) => format!(
                "[{}]",
                items
                    .iter()
                    .map(Raw::py_repr)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Raw::Dict(items) => format!(
                "{{{}}}",
                items
                    .iter()
                    .map(|(k, v)| format!("{}: {}", pytext::repr(k), v.py_repr()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }

    /// `type(value).__name__`.
    pub fn type_name(&self) -> &'static str {
        match self {
            Raw::None => "NoneType",
            Raw::Bool(_) => "bool",
            Raw::Int(_) => "int",
            Raw::Float(_) => "float",
            Raw::Str(_) => "str",
            Raw::List(_) => "list",
            Raw::Dict(_) => "dict",
        }
    }

    /// `bool(value)`.
    pub fn truthy(&self) -> bool {
        match self {
            Raw::None => false,
            Raw::Bool(b) => *b,
            Raw::Int(digits) => !digits
                .trim_start_matches('-')
                .trim_start_matches('0')
                .is_empty(),
            Raw::Float(f) => *f != 0.0,
            Raw::Str(s) => !s.is_empty(),
            Raw::List(items) => !items.is_empty(),
            Raw::Dict(items) => !items.is_empty(),
        }
    }

    /// `self == other` as Python compares the values `json.loads` builds:
    /// `True == 1 == 1.0`, and dicts compare by their keys, not their order.
    pub fn py_eq(&self, other: &Raw) -> bool {
        self.eq_at(other, false)
    }

    /// Inside a container Python compares items by identity first, and
    /// `json.loads` gives every `NaN` the same float object, so a nested NaN
    /// equals a nested NaN while a bare one does not.
    fn eq_at(&self, other: &Raw, nested: bool) -> bool {
        match (self, other) {
            (Raw::None, Raw::None) => true,
            (Raw::Float(a), Raw::Float(b)) if nested && a.is_nan() && b.is_nan() => true,
            (Raw::Str(a), Raw::Str(b)) => a == b,
            (Raw::List(a), Raw::List(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.eq_at(y, true))
            }
            (Raw::Dict(_), Raw::Dict(_)) => {
                let (left, right) = (self.keys(), other.keys());
                left.len() == right.len()
                    && left.iter().all(|k| {
                        other
                            .get(k)
                            .is_some_and(|v| self.get(k).is_some_and(|w| w.eq_at(v, true)))
                    })
            }
            _ => match (self.number(), other.number()) {
                (Some(a), Some(b)) => a == b,
                _ => false,
            },
        }
    }

    /// A dict's distinct keys.
    fn keys(&self) -> Vec<&String> {
        let mut keys: Vec<&String> = Vec::new();
        if let Raw::Dict(items) = self {
            for (k, _) in items {
                if !keys.contains(&k) {
                    keys.push(k);
                }
            }
        }
        keys
    }

    /// A bool, int or float as an exact integer where it is one.
    fn number(&self) -> Option<Number> {
        use num_traits::FromPrimitive;
        match self {
            Raw::Bool(b) => Some(Number::Exact(num_bigint::BigInt::from(u8::from(*b)))),
            Raw::Int(digits) => digits.parse().ok().map(Number::Exact),
            Raw::Float(f) if f.is_finite() && f.fract() == 0.0 => {
                num_bigint::BigInt::from_f64(*f).map(Number::Exact)
            }
            Raw::Float(f) => Some(Number::Inexact(*f)),
            _ => None,
        }
    }

    /// `mapping.get(key)` for a dict; None otherwise.
    pub fn get(&self, key: &str) -> Option<&Raw> {
        match self {
            Raw::Dict(items) => items.iter().rev().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn from_json(value: &Json) -> Raw {
        match value {
            Json::Null => Raw::None,
            Json::Bool(b) => Raw::Bool(*b),
            Json::Int(i) => Raw::Int(i.to_string()),
            Json::Float(f) => Raw::Float(*f),
            Json::Str(s) => Raw::Str(s.clone()),
            Json::Array(items) => Raw::List(items.iter().map(Raw::from_json).collect()),
            Json::Object(items) => Raw::Dict(
                items
                    .iter()
                    .map(|(k, v)| (k.clone(), Raw::from_json(v)))
                    .collect(),
            ),
        }
    }

    pub fn from_value(value: &serde_json::Value) -> Raw {
        use serde_json::Value;
        match value {
            Value::Null => Raw::None,
            Value::Bool(b) => Raw::Bool(*b),
            // An integer of any size keeps its digits, as Python's int does.
            Value::Number(n) if n.is_i64() || n.is_u64() || is_integer_lexeme(&n.to_string()) => {
                Raw::Int(n.to_string())
            }
            // float(text): an overflowing literal such as 1e400 is infinity.
            Value::Number(n) => Raw::Float(n.to_string().parse().unwrap_or(f64::NAN)),
            Value::String(s) => Raw::Str(s.clone()),
            Value::Array(items) => Raw::List(items.iter().map(Raw::from_value).collect()),
            Value::Object(items) => Raw::Dict(
                items
                    .iter()
                    .map(|(k, v)| (k.clone(), Raw::from_value(v)))
                    .collect(),
            ),
        }
    }
}

/// `repr(float)`: the shortest round-trip digits, positional from 1e-4 up to
/// 1e16 and scientific with a signed two-digit exponent outside that.
pub fn float_repr(value: f64) -> String {
    if value.is_nan() {
        return "nan".into();
    }
    if value.is_infinite() {
        return if value > 0.0 { "inf" } else { "-inf" }.into();
    }
    let sci = format!("{value:e}");
    let (mantissa, exponent) = sci.split_once('e').expect("exponent form");
    let exponent: i32 = exponent.parse().expect("exponent digits");
    let (sign, mantissa) = match mantissa.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", mantissa),
    };
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    let decpt = exponent + 1;
    if !(-4 < decpt && decpt <= 16) {
        let rest = &digits[1..];
        let dot = if rest.is_empty() {
            String::new()
        } else {
            format!(".{rest}")
        };
        let esign = if exponent < 0 { '-' } else { '+' };
        return format!(
            "{sign}{}{dot}e{esign}{:02}",
            &digits[..1],
            exponent.unsigned_abs()
        );
    }
    let body = if decpt <= 0 {
        format!("0.{}{digits}", "0".repeat((-decpt) as usize))
    } else if decpt as usize >= digits.len() {
        format!("{digits}{}.0", "0".repeat(decpt as usize - digits.len()))
    } else {
        format!(
            "{}.{}",
            &digits[..decpt as usize],
            &digits[decpt as usize..]
        )
    };
    format!("{sign}{body}")
}

fn is_integer_lexeme(text: &str) -> bool {
    let digits = text.strip_prefix('-').unwrap_or(text);
    !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn float_repr_matches_python() {
        for (value, text) in [
            (0.0877, "0.0877"),
            (1.0, "1.0"),
            (1e16, "1e+16"),
            (1e15, "1000000000000000.0"),
            (1e-5, "1e-05"),
            (0.0001, "0.0001"),
            (-1.5e-7, "-1.5e-07"),
            (123456789.125, "123456789.125"),
            (0.0, "0.0"),
            (1e300, "1e+300"),
        ] {
            assert_eq!(float_repr(value), text);
        }
    }

    #[test]
    fn repr_of_containers_matches_python() {
        let value = Raw::Dict(vec![
            ("a".into(), Raw::List(vec![Raw::Int("1".into()), Raw::None])),
            ("b".into(), Raw::Str("it's".into())),
        ]);
        assert_eq!(value.py_repr(), "{'a': [1, None], 'b': \"it's\"}");
        assert_eq!(value.py_str(), value.py_repr());
    }
}
