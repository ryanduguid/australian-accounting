// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! JSON boundary for the removed Python import API. Each command takes the
//! original function's keyword arguments as one JSON object: amounts and
//! ratios as decimal strings, the rest as JSON booleans, integers or strings.
//! Values of another type reach the engine, which refuses them as the
//! original's own type checks did.
use crate::decimal::Decimal;
use crate::engine::{self, Arg};
use serde_json::Value;
use std::collections::HashSet;

/// A refusal at the JSON boundary: its stable code and the message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal {
    pub code: &'static str,
    pub message: String,
}

impl Refusal {
    pub fn new(code: &'static str, message: impl Into<String>) -> Refusal {
        Refusal {
            code,
            message: message.into(),
        }
    }
}

/// The first key an object in `bytes` repeats, which decoding into `Value`
/// resolves silently by keeping the last value. `bytes` must already have
/// decoded as JSON; each key is compared decoded, so escapes cannot hide a repeat.
/// This command and `au-accounting tax` both refuse on it, before the worksheet
/// sees either value, so a repeat is refused whatever the values are.
pub fn repeated_key(bytes: &[u8]) -> Option<String> {
    // One entry per open container: an object's keys so far, or None for an array.
    let mut open: Vec<Option<HashSet<String>>> = Vec::new();
    let mut key_next = false;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                let start = i;
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    i += if bytes[i] == b'\\' { 2 } else { 1 };
                }
                if key_next && let Some(Some(keys)) = open.last_mut() {
                    let literal = bytes.get(start..=i).unwrap_or_default();
                    if let Ok(key) = serde_json::from_slice::<String>(literal)
                        && !keys.insert(key.clone())
                    {
                        return Some(key);
                    }
                }
                key_next = false;
            }
            b'{' => {
                open.push(Some(HashSet::new()));
                key_next = true;
            }
            b'[' => {
                open.push(None);
                key_next = false;
            }
            b'}' | b']' => {
                open.pop();
                key_next = false;
            }
            b',' => key_next = matches!(open.last(), Some(Some(_))),
            _ => {}
        }
        i += 1;
    }
    None
}

/// Every refusal code and the failure it names.
pub fn refusal_codes() -> &'static [(&'static str, &'static str)] {
    crate::texts::REFUSAL_CODES
}

fn refusal<T>(code: &'static str, message: impl Into<String>) -> Result<T> {
    Err(Refusal::new(code, message))
}

type Result<T> = std::result::Result<T, Refusal>;

/// Argument kinds: a decimal string, a JSON value the engine reads as JSON
/// (lists of periods), or any other JSON value.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Dec,
    Json,
    Raw,
}
use Kind::{Dec, Json, Raw};

/// Command, the worksheet's parameters in the original order.
const COMMAND_ARGS: &[(&str, &[(&str, Kind)])] = &[
    (
        "gst",
        &[
            ("amount", Dec),
            ("gst_inclusive", Raw),
            ("scope_confirmed", Raw),
            ("year", Raw),
        ],
    ),
    (
        "resident-tax",
        &[
            ("taxable_income", Dec),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "capital-gains",
        &[
            ("other_gains", Dec),
            ("discount_gains", Dec),
            ("current_losses", Dec),
            ("prior_losses", Dec),
            ("scope_confirmed", Raw),
            ("year", Raw),
        ],
    ),
    (
        "main-residence",
        &[
            ("capital_gain", Dec),
            ("capital_loss", Dec),
            ("market_value_basis", Raw),
            ("ownership_start", Raw),
            ("ownership_end", Raw),
            ("lived_periods", Json),
            ("income_use_periods", Json),
            ("absence_choice_periods", Json),
            ("new_home_acquired", Json),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "borrowing-expenses",
        &[
            ("borrowing_expenses", Dec),
            ("other_borrowing_expenses_in_year", Dec),
            ("income_use", Dec),
            ("loan_start", Raw),
            ("loan_term_end", Json),
            ("repaid_on", Json),
            ("section_26_50_applies", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "fbt",
        &[
            ("type_one_value", Dec),
            ("type_two_value", Dec),
            ("year_ended", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "fbt-car-statutory",
        &[
            ("cost_price", Dec),
            ("later_accessories", Dec),
            ("one_third_reduction", Raw),
            ("days_available", Raw),
            ("employee_contribution", Dec),
            ("electric_car_exempt", Raw),
            ("year_ended", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "fbt-car-operating-cost",
        &[
            ("running_costs", Dec),
            ("lease_charges", Dec),
            ("depreciated_value", Dec),
            ("effective_life_years", Dec),
            ("days_held", Raw),
            ("business_use", Dec),
            ("employee_contribution", Dec),
            ("statutory_formula_taxable_value", Dec),
            ("electric_car_exempt", Raw),
            ("year_ended", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "depreciation",
        &[
            ("cost", Dec),
            ("effective_life", Dec),
            ("days", Raw),
            ("taxable_use", Dec),
            ("method", Raw),
            ("scope_confirmed", Raw),
            ("year", Raw),
        ],
    ),
    (
        "quarterly-sg",
        &[
            ("ordinary_time_earnings", Dec),
            ("qualifying_contributions", Dec),
            ("year", Raw),
            ("quarter", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "payg-withholding",
        &[
            ("earnings", Dec),
            ("pay_period", Raw),
            ("scale", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "study-loan-withholding",
        &[
            ("earnings", Dec),
            ("pay_period", Raw),
            ("scale", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "medicare-levy",
        &[
            ("taxable_income", Dec),
            ("section_9a_adjustment_applies", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "low-income-tax-offset",
        &[
            ("taxable_income", Dec),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "division-293",
        &[
            ("division_293_income", Dec),
            ("low_tax_contributions", Dec),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "division-296",
        &[
            ("total_super_balance_end_of_year", Dec),
            ("total_super_earnings", Dec),
            ("child_recipient", Raw),
            ("structured_settlement_contribution", Raw),
            ("died_in_year", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "study-loan-repayment",
        &[
            ("repayment_income", Dec),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "contribution-caps",
        &[
            ("total_super_balance", Dec),
            ("concessional_contributions", Dec),
            ("unused_concessional_cap", Dec),
            ("non_concessional_contributions", Dec),
            ("under_75_in_year", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
    (
        "pension-minimum",
        &[
            ("account_balance", Dec),
            ("age", Raw),
            ("days", Raw),
            ("year", Raw),
            ("scope_confirmed", Raw),
        ],
    ),
];

pub fn commands() -> Vec<&'static str> {
    let mut out: Vec<&'static str> = COMMAND_ARGS.iter().map(|(c, _)| *c).collect();
    out.push("worksheet-catalogue");
    out.push("periods");
    out
}

/// A JSON value as the Python object the original received.
pub fn arg(value: &Value) -> Arg {
    match value {
        Value::Bool(b) => Arg::Bool(*b),
        Value::Number(n) => match n.as_i64() {
            Some(i) => Arg::Int(i),
            None if is_integer(&n.to_string()) => Arg::BigInt(n.to_string()),
            None => Arg::Other(py_repr(value)),
        },
        Value::String(s) => Arg::Text(s.clone()),
        other => Arg::Other(py_repr(other)),
    }
}

/// An integer token of any size, which `json.loads` makes an `int`.
fn is_integer(text: &str) -> bool {
    let digits = text.strip_prefix('-').unwrap_or(text);
    !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
}

/// `repr()` of the Python object `json.loads` makes from `value`.
fn py_repr(value: &Value) -> String {
    match value {
        Value::Null => "None".into(),
        Value::Bool(true) => "True".into(),
        Value::Bool(false) => "False".into(),
        Value::Number(n) if is_integer(&n.to_string()) => {
            let text = n.to_string();
            // `int("-0")` is 0.
            if text.trim_start_matches('-').bytes().all(|b| b == b'0') {
                "0".into()
            } else {
                text
            }
        }
        // float(text): an overflowing literal such as 1e400 is infinity.
        Value::Number(n) => float_repr(n.to_string().parse().unwrap_or(f64::NAN)),
        Value::String(s) => crate::pytext::repr(s),
        Value::Array(items) => format!(
            "[{}]",
            items.iter().map(py_repr).collect::<Vec<_>>().join(", ")
        ),
        Value::Object(items) => format!(
            "{{{}}}",
            items
                .iter()
                .map(|(k, v)| format!("{}: {}", crate::pytext::repr(k), py_repr(v)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

/// `format!("{value:e}")`, but where two spellings are equally short, the one
/// CPython's `repr` prints: the nearer to `value`, an exact tie going to the even
/// digit. Rust's shortest formatting can break that tie upwards: `1 + 2**-17` is
/// `1.0000076293945312` in Python, not `…313`. `{:.Ne}` rounds exactly with ties
/// to even, so its spelling is used whenever it still reads back as `value`.
fn shortest_exp(value: f64) -> String {
    let shortest = format!("{value:e}");
    let (mantissa, _) = shortest.split_once('e').expect("exponent form");
    let precision = mantissa.bytes().filter(u8::is_ascii_digit).count() - 1;
    let nearest = format!("{value:.precision$e}");
    if nearest.parse::<f64>() == Ok(value) {
        nearest
    } else {
        shortest
    }
}

/// `repr(float)`: the shortest text that reads back as the same double.
fn float_repr(value: f64) -> String {
    if value.is_nan() {
        return "nan".into();
    }
    if value.is_infinite() {
        return if value > 0.0 { "inf" } else { "-inf" }.into();
    }
    let sci = shortest_exp(value);
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

fn argument(v: &Value, key: &str, kind: Kind) -> Result<Arg> {
    let Some(value) = v.get(key) else {
        return refusal("input.missing_field", format!("{key} is required"));
    };
    match (kind, value) {
        (Dec, Value::String(text)) => {
            if text.len() > 8192 {
                return refusal(
                    "input.decimal_too_long",
                    format!("{key}: decimal input exceeds 8192 bytes"),
                );
            }
            Decimal::parse(text)
                .map(Arg::Decimal)
                .or_else(|s| refusal("input.decimal_unreadable", format!("decimal.{}", s.name())))
        }
        (Json, _) => Ok(Arg::Json(value.clone())),
        _ => Ok(arg(value)),
    }
}

/// `json.loads` refuses an integer token of more than 4,300 digits, as
/// `int(text)` does under Python's default limit.
fn int_limit(value: &Value) -> Result<()> {
    match value {
        Value::Number(n) => {
            let text = n.to_string();
            let digits = text.strip_prefix('-').unwrap_or(&text);
            if digits.len() > 4300 && digits.bytes().all(|b| b.is_ascii_digit()) {
                return refusal(
                    "input.integer_too_long",
                    format!(
                        "ValueError: Exceeds the limit (4300 digits) for integer string conversion: \
                         value has {} digits; use sys.set_int_max_str_digits() to increase the limit",
                        digits.len()
                    ),
                );
            }
            Ok(())
        }
        Value::Array(items) => items.iter().try_for_each(int_limit),
        Value::Object(items) => items.values().try_for_each(int_limit),
        _ => Ok(()),
    }
}

/// Run one JSON stdin command: the result value, or the refusal message.
pub fn evaluate(command: &str, v: &Value) -> std::result::Result<Value, String> {
    evaluate_coded(command, v).map_err(|refusal| refusal.message)
}

/// [`evaluate`], keeping each refusal's stable code beside its message.
pub fn evaluate_coded(command: &str, v: &Value) -> Result<Value> {
    if !v.is_object() {
        return refusal("input.not_object", "input must be a JSON object");
    }
    int_limit(v)?;
    match command {
        "worksheet-catalogue" => return Ok(engine::worksheet_catalogue()),
        "periods" => {
            let kind = match v.get("kind") {
                None => return refusal("input.missing_field", "kind is required"),
                Some(Value::String(kind)) => kind.as_str(),
                Some(_) => return refusal("input.kind_invalid", "kind must be a string"),
            };
            // Python raised KeyError for a worksheet it does not have.
            if crate::metadata::supported_periods(kind).is_empty() {
                return refusal(
                    "worksheet.unknown",
                    format!("unknown worksheet {}", crate::pytext::repr(kind)),
                );
            }
            return Ok(engine::periods(kind));
        }
        _ => {}
    }
    let Some((_, params)) = COMMAND_ARGS.iter().find(|(c, _)| *c == command) else {
        return refusal("command.unknown", format!("unknown command {command}"));
    };
    // Division 296 has no Python original whose lenience to keep, and a misspelt
    // optional depreciation input would silently give the start year's result, so
    // a field either does not take, such as an opening balance, is refused rather
    // than ignored.
    if matches!(command, "division-296" | "depreciation") {
        let optional = crate::metadata::optional_inputs(command);
        let taken = |key: &String| {
            params.iter().any(|(p, _)| p == key) || optional.iter().any(|(o, _)| o == key)
        };
        if let Some(key) = v
            .as_object()
            .into_iter()
            .flat_map(|o| o.keys())
            .find(|k| !taken(k))
        {
            return refusal(
                "input.field_unknown",
                format!("{key} is not an input of {command}"),
            );
        }
    }
    let a: Vec<Arg> = params
        .iter()
        .map(|(key, kind)| argument(v, key, *kind))
        .collect::<Result<_>>()?;
    let result = match command {
        "gst" => engine::gst(&a[0], &a[1], &a[2], &a[3]),
        "resident-tax" => engine::resident_tax(&a[0], &a[1], &a[2]),
        "capital-gains" => engine::capital_gains(&a[0], &a[1], &a[2], &a[3], &a[4], &a[5]),
        "main-residence" => engine::main_residence(
            &a[0], &a[1], &a[2], &a[3], &a[4], &a[5], &a[6], &a[7], &a[8], &a[9], &a[10],
        ),
        "borrowing-expenses" => {
            let loan = engine::LoanDates {
                start: &a[3],
                term_end: &a[4],
                repaid_on: &a[5],
            };
            engine::borrowing_expenses(&a[0], &a[1], &a[2], loan, &a[6], &a[7], &a[8])
        }
        "fbt" => engine::fbt(&a[0], &a[1], &a[2], &a[3]),
        "fbt-car-statutory" => {
            engine::fbt_car_statutory(&a[0], &a[1], &a[2], &a[3], &a[4], &a[5], &a[6], &a[7])
        }
        "fbt-car-operating-cost" => engine::fbt_car_operating_cost(
            &a[0], &a[1], &a[2], &a[3], &a[4], &a[5], &a[6], &a[7], &a[8], &a[9], &a[10],
        ),
        "depreciation" => {
            // Optional: present only for an income year after the asset's start time.
            let key = "opening_adjustable_value";
            let opening = match v.get(key) {
                None => None,
                Some(_) => Some(argument(v, key, Dec)?),
            };
            engine::depreciation(
                &a[0],
                &a[1],
                &a[2],
                &a[3],
                &a[4],
                &a[5],
                &a[6],
                opening.as_ref(),
            )
        }
        "quarterly-sg" => engine::quarterly_sg(&a[0], &a[1], &a[2], &a[3], &a[4]),
        "payg-withholding" => engine::payg_withholding(&a[0], &a[1], &a[2], &a[3], &a[4]),
        "study-loan-withholding" => {
            engine::study_loan_withholding(&a[0], &a[1], &a[2], &a[3], &a[4])
        }
        "medicare-levy" => engine::medicare_levy(&a[0], &a[1], &a[2], &a[3]),
        "low-income-tax-offset" => engine::low_income_tax_offset(&a[0], &a[1], &a[2]),
        "division-293" => engine::division_293(&a[0], &a[1], &a[2], &a[3]),
        "division-296" => engine::division_296(&a[0], &a[1], [&a[2], &a[3], &a[4]], &a[5], &a[6]),
        "study-loan-repayment" => engine::study_loan_repayment(&a[0], &a[1], &a[2]),
        "contribution-caps" => {
            engine::contribution_caps(&a[0], &a[1], &a[2], &a[3], &a[4], &a[5], &a[6])
        }
        "pension-minimum" => engine::pension_minimum(&a[0], &a[1], &a[2], &a[3], &a[4]),
        _ => unreachable!("listed command"),
    };
    result.or_else(|e| refusal(e.code(), e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn float_repr_matches_cpython_corpus() {
        // CPython 3.15's repr for doubles chosen to exercise ties between two
        // equally short spellings: bits in hex, then the repr.
        assert_eq!(float_repr(1.0 + 2f64.powi(-17)), "1.0000076293945312");
        let wrong: Vec<String> = include_str!("../tests/data/float_repr.txt")
            .lines()
            .filter_map(|line| {
                let (bits, expected) = line.split_once(' ').expect("bits and repr");
                let actual = float_repr(f64::from_bits(u64::from_str_radix(bits, 16).unwrap()));
                (actual != expected).then(|| format!("{bits}: python {expected}, rust {actual}"))
            })
            .collect();
        assert!(
            wrong.is_empty(),
            "{} wrong: {:?}",
            wrong.len(),
            &wrong[..wrong.len().min(10)]
        );
    }
}
