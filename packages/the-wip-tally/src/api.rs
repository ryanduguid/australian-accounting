// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! JSON boundary for the removed Python import API (`measure`, `parse_money`).
//! Amounts are decimal strings; README.md documents both schemas.
use crate::decimal::Decimal;
use crate::engine::{self, ContractInput, ScheduleError};
use crate::money::{self, AmountError};
use serde_json::{Value, json};

pub const COMMANDS: &[&str] = &["measure", "parse-money"];

type Result<T> = std::result::Result<T, String>;

fn text(v: &Value, key: &str) -> Result<String> {
    v.get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("{key} must be a string"))
}

fn decimal(v: &Value, key: &str) -> Result<Decimal> {
    let t = text(v, key)?;
    if t.len() > 8192 {
        return Err(format!("{key}: decimal input exceeds 8192 bytes"));
    }
    // `Decimal(text)` raised InvalidOperation for text it could not read.
    Decimal::parse(&t).map_err(|s| format!("decimal.{}", s.name()))
}

/// A JSON integer of any size, as Python's `int` prints it.
fn integer(v: &Value, key: &str) -> Result<String> {
    let text = match v.get(key) {
        Some(Value::Number(n)) => n.to_string(),
        _ => String::new(),
    };
    let digits = text.strip_prefix('-').unwrap_or(&text);
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("{key} must be an integer"));
    }
    Ok(if digits.bytes().all(|b| b == b'0') {
        "0".into()
    } else {
        text
    })
}

/// `str()` of the Python object `json.loads` makes from `value`, as an
/// f-string prints it. A JSON object is refused: serde_json here does not keep
/// its key order, which Python's `str()` of a dict would show.
fn py_str(value: &Value) -> Result<String> {
    match value {
        Value::String(s) => Ok(s.clone()),
        other => py_repr(other),
    }
}

/// `repr()` of the Python object `json.loads` makes from `value`.
fn py_repr(value: &Value) -> Result<String> {
    Ok(match value {
        Value::Null => "None".into(),
        Value::Bool(true) => "True".into(),
        Value::Bool(false) => "False".into(),
        Value::Number(n) => {
            let text = n.to_string();
            let digits = text.strip_prefix('-').unwrap_or(&text);
            if digits.bytes().all(|b| b.is_ascii_digit()) {
                // `int("-0")` is 0.
                if digits.bytes().all(|b| b == b'0') {
                    "0".into()
                } else {
                    text
                }
            } else {
                // float(text): an overflowing literal such as 1e400 is infinity.
                float_repr(text.parse().unwrap_or(f64::NAN))
            }
        }
        Value::String(s) => crate::pytext::repr(s),
        Value::Array(items) => format!(
            "[{}]",
            items
                .iter()
                .map(py_repr)
                .collect::<Result<Vec<_>>>()?
                .join(", ")
        ),
        Value::Object(_) => return Err("where must not be a JSON object".into()),
    })
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

fn optional_decimal(v: &Value, key: &str) -> Result<Option<Decimal>> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(_) => decimal(v, key).map(Some),
    }
}

fn contract(v: &Value) -> Result<ContractInput> {
    let measurable = match v.get("outcome_reasonably_measurable") {
        None | Some(Value::Null) => None,
        Some(Value::Bool(b)) => Some(*b),
        Some(_) => return Err("outcome_reasonably_measurable must be true, false or null".into()),
    };
    Ok(ContractInput {
        contract_id: text(v, "contract_id")?,
        line_number: integer(v, "line_number")?,
        customer: text(v, "customer")?,
        description: text(v, "description")?,
        original_contract_sum: decimal(v, "original_contract_sum")?,
        approved_variations: decimal(v, "approved_variations")?,
        unapproved_variations_estimate: decimal(v, "unapproved_variations_estimate")?,
        constraint_include_ratio: decimal(v, "constraint_include_ratio")?,
        costs_incurred: decimal(v, "costs_incurred")?,
        inefficiency_rework_wastage: decimal(v, "inefficiency_rework_wastage")?,
        uninstalled_materials: decimal(v, "uninstalled_materials")?,
        estimated_cost_to_complete: decimal(v, "estimated_cost_to_complete")?,
        certified_billings: decimal(v, "certified_billings")?,
        uncertified_claims: decimal(v, "uncertified_claims")?,
        retention_withheld: decimal(v, "retention_withheld")?,
        retention_classification: text(v, "retention_classification")?,
        committed_outstanding: decimal(v, "committed_outstanding")?,
        outcome_reasonably_measurable: measurable,
        recoverable_costs: optional_decimal(v, "recoverable_costs")?,
        progress_method: text(v, "progress_method")?,
        output_percent: optional_decimal(v, "output_percent")?,
        prior_transaction_price: optional_decimal(v, "prior_transaction_price")?,
        prior_estimated_cost_at_completion: optional_decimal(
            v,
            "prior_estimated_cost_at_completion",
        )?,
        prior_costs_incurred: optional_decimal(v, "prior_costs_incurred")?,
        prior_estimated_cost_to_complete: optional_decimal(v, "prior_estimated_cost_to_complete")?,
        prior_revenue_to_date: optional_decimal(v, "prior_revenue_to_date")?,
        gst_rate: decimal(v, "gst_rate")?,
        assets_used_carrying: optional_decimal(v, "assets_used_carrying")?,
    })
}

fn opt(value: Option<&Decimal>) -> Value {
    value.map_or(Value::Null, |d| json!(d.to_string()))
}

/// The position as JSON: every computed field, plus `needs_review`.
pub fn position_json(p: &engine::ContractPosition) -> Value {
    json!({
        "approved_price": p.approved_price.to_string(),
        "variable_consideration_included": p.variable_consideration_included.to_string(),
        "variable_consideration_excluded": p.variable_consideration_excluded.to_string(),
        "transaction_price": p.transaction_price.to_string(),
        "progress_cost": p.progress_cost.to_string(),
        "estimated_cost_at_completion": p.estimated_cost_at_completion.to_string(),
        "progress_eac": p.progress_eac.to_string(),
        "percent_complete": p.percent_complete.to_string(),
        "revenue_to_date": p.revenue_to_date.to_string(),
        "contract_asset": p.contract_asset.to_string(),
        "contract_liability": p.contract_liability.to_string(),
        "gross_profit_at_completion": p.gross_profit_at_completion.to_string(),
        "margin_at_completion": opt(p.margin_at_completion.as_ref()),
        "prior_margin_at_completion": opt(p.prior_margin_at_completion.as_ref()),
        "profit_fade_points": opt(p.profit_fade_points.as_ref()),
        "period_revenue": opt(p.period_revenue.as_ref()),
        "gst_on_certified_billings": p.gst_on_certified_billings.to_string(),
        "gst_on_retention": p.gst_on_retention.to_string(),
        "flags": p.flags,
        "needs_review": p.needs_review(),
    })
}

/// Run one JSON stdin command: the result value, or the refusal message.
/// `json.loads` refuses an integer token of more than 4,300 digits, as
/// `int(text)` does under Python's default limit.
fn int_limit(value: &Value) -> Result<()> {
    match value {
        Value::Number(n) => {
            let text = n.to_string();
            let digits = text.strip_prefix('-').unwrap_or(&text);
            if digits.len() > 4300 && digits.bytes().all(|b| b.is_ascii_digit()) {
                return Err(format!(
                    "ValueError: Exceeds the limit (4300 digits) for integer string conversion: \
                     value has {} digits; use sys.set_int_max_str_digits() to increase the limit",
                    digits.len()
                ));
            }
            Ok(())
        }
        Value::Array(items) => items.iter().try_for_each(int_limit),
        Value::Object(items) => items.values().try_for_each(int_limit),
        _ => Ok(()),
    }
}

pub fn evaluate(command: &str, v: &Value) -> Result<Value> {
    if !v.is_object() {
        return Err("input must be a JSON object".into());
    }
    int_limit(v)?;
    match command {
        "measure" => match engine::measure(&contract(v)?) {
            Ok(position) => Ok(position_json(&position)),
            Err(ScheduleError::Message(m)) => Err(m),
            Err(ScheduleError::Decimal(s)) => Err(format!("decimal.{}", s.name())),
        },
        "parse-money" => {
            // `parse_money(raw, where="amount")`: an explicit null `where`
            // is printed as None, and a null `raw` is "no amount given".
            let where_ = match v.get("where") {
                None => "amount".to_string(),
                Some(value) => py_str(value)?,
            };
            if matches!(v.get("raw"), Some(Value::Null)) {
                return Err(format!("{where_}: no amount given"));
            }
            match money::parse_money(&text(v, "raw")?, &where_) {
                Ok(value) => Ok(json!(value.to_string())),
                Err(AmountError::Message(m)) => Err(m),
                Err(AmountError::Decimal(s)) => Err(format!("decimal.{}", s.name())),
            }
        }
        _ => Err(format!("unknown command {command}")),
    }
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
        assert_eq!(wrong, Vec::<String>::new());
    }

    #[test]
    fn a_closer_spelling_that_does_not_read_back_yields_to_the_shortest() {
        // 2**-1017: the 16-digit spelling nearest the value reads back as a
        // different double, so CPython prints the shortest round-trip form.
        assert_eq!(float_repr(2f64.powi(-1017)), "7.120236347223045e-307");
    }

    fn base() -> Value {
        json!({"contract_id": "J1", "line_number": 2, "customer": "", "description": ""})
    }

    fn with(mut v: Value, key: &str, value: Value) -> Value {
        v[key] = value;
        v
    }

    #[test]
    fn measure_refuses_malformed_fields_by_name() {
        let long = "1".repeat(8193);
        for (input, want) in [
            (json!([1]), "input must be a JSON object"),
            (
                with(base(), "line_number", json!(1.5)),
                "line_number must be an integer",
            ),
            (
                with(base(), "line_number", json!("2")),
                "line_number must be an integer",
            ),
            (
                with(base(), "original_contract_sum", json!(long)),
                "original_contract_sum: decimal input exceeds 8192 bytes",
            ),
            (
                with(base(), "outcome_reasonably_measurable", json!("yes")),
                "outcome_reasonably_measurable must be true, false or null",
            ),
        ] {
            assert_eq!(
                evaluate("measure", &input),
                Err(want.to_string()),
                "{input}"
            );
        }
    }

    #[test]
    fn an_optional_prior_figure_that_is_not_a_decimal_is_refused() {
        let mut v = base();
        for key in [
            "original_contract_sum",
            "approved_variations",
            "unapproved_variations_estimate",
            "constraint_include_ratio",
            "costs_incurred",
            "inefficiency_rework_wastage",
            "uninstalled_materials",
            "estimated_cost_to_complete",
            "certified_billings",
            "uncertified_claims",
            "retention_withheld",
            "committed_outstanding",
        ] {
            v[key] = json!("0");
        }
        v["retention_classification"] = json!("review");
        v["progress_method"] = json!("cost_to_cost");
        v["prior_estimated_cost_at_completion"] = json!("ten");
        assert_eq!(
            evaluate("measure", &v),
            Err("decimal.InvalidOperation".to_string())
        );
    }

    #[test]
    fn where_prints_as_python_str_and_an_object_is_refused() {
        let missing =
            |where_: Value| evaluate("parse-money", &json!({"raw": null, "where": where_}));
        assert_eq!(missing(json!(false)), Err("False: no amount given".into()));
        assert_eq!(missing(json!(-0)), Err("0: no amount given".into()));
        assert_eq!(
            missing(json!({"a": 1})),
            Err("where must not be a JSON object".into())
        );
    }

    #[test]
    fn an_unknown_command_is_named_in_the_refusal() {
        assert_eq!(
            evaluate("close-job", &json!({})),
            Err("unknown command close-job".into())
        );
    }

    #[test]
    fn variable_consideration_too_large_for_cents_raises_invalid_operation() {
        // quantize(Decimal("0.01")) of 1E+27 needs 30 digits, past the 28-digit
        // context, so Python raised decimal.InvalidOperation.
        let mut v = base();
        for key in [
            "original_contract_sum",
            "approved_variations",
            "costs_incurred",
            "inefficiency_rework_wastage",
            "uninstalled_materials",
            "estimated_cost_to_complete",
            "certified_billings",
            "uncertified_claims",
            "retention_withheld",
            "committed_outstanding",
        ] {
            v[key] = json!("0");
        }
        v["original_contract_sum"] = json!("1000");
        v["unapproved_variations_estimate"] = json!("1E27");
        v["constraint_include_ratio"] = json!("1");
        v["gst_rate"] = json!("0.1");
        v["retention_classification"] = json!("review");
        v["progress_method"] = json!("cost_to_cost");
        assert_eq!(
            evaluate("measure", &v),
            Err("decimal.InvalidOperation".to_string())
        );
    }
}
