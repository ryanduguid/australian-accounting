//! JSON boundary for the removed Python import API. Each command takes the
//! original function's keyword arguments as one JSON object: amounts and
//! ratios as decimal strings, the rest as JSON booleans, integers or strings.
//! Values of another type reach the engine, which refuses them as the
//! original's own type checks did.
use crate::decimal::Decimal;
use crate::engine::{self, Arg};
use serde_json::Value;

type Result<T> = std::result::Result<T, String>;

/// Argument kinds: a decimal string, or any other JSON value.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Dec,
    Raw,
}
use Kind::{Dec, Raw};

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
        "fbt",
        &[
            ("type_one_value", Dec),
            ("type_two_value", Dec),
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
        Value::Number(n) => float_repr(n.as_f64().unwrap_or(f64::NAN)),
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

/// `repr(float)`: the shortest text that reads back as the same double.
fn float_repr(value: f64) -> String {
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

fn argument(v: &Value, key: &str, kind: Kind) -> Result<Arg> {
    let Some(value) = v.get(key) else {
        return Err(format!("{key} is required"));
    };
    match (kind, value) {
        (Dec, Value::String(text)) => {
            if text.len() > 8192 {
                return Err(format!("{key}: decimal input exceeds 8192 bytes"));
            }
            Decimal::parse(text)
                .map(Arg::Decimal)
                .map_err(|s| format!("decimal.{}", s.name()))
        }
        _ => Ok(arg(value)),
    }
}

/// Run one JSON stdin command: the result value, or the refusal message.
pub fn evaluate(command: &str, v: &Value) -> Result<Value> {
    if !v.is_object() {
        return Err("input must be a JSON object".into());
    }
    match command {
        "worksheet-catalogue" => return Ok(engine::worksheet_catalogue()),
        "periods" => {
            let kind = match v.get("kind") {
                None => return Err("kind is required".into()),
                Some(Value::String(kind)) => kind.as_str(),
                Some(_) => return Err("kind must be a string".into()),
            };
            // Python raised KeyError for a worksheet it does not have.
            if crate::metadata::supported_periods(kind).is_empty() {
                return Err(format!("unknown worksheet {}", crate::pytext::repr(kind)));
            }
            return Ok(engine::periods(kind));
        }
        _ => {}
    }
    let Some((_, params)) = COMMAND_ARGS.iter().find(|(c, _)| *c == command) else {
        return Err(format!("unknown command {command}"));
    };
    let a: Vec<Arg> = params
        .iter()
        .map(|(key, kind)| argument(v, key, *kind))
        .collect::<Result<_>>()?;
    let result = match command {
        "gst" => engine::gst(&a[0], &a[1], &a[2], &a[3]),
        "resident-tax" => engine::resident_tax(&a[0], &a[1], &a[2]),
        "capital-gains" => engine::capital_gains(&a[0], &a[1], &a[2], &a[3], &a[4], &a[5]),
        "fbt" => engine::fbt(&a[0], &a[1], &a[2], &a[3]),
        "depreciation" => engine::depreciation(&a[0], &a[1], &a[2], &a[3], &a[4], &a[5], &a[6]),
        "quarterly-sg" => engine::quarterly_sg(&a[0], &a[1], &a[2], &a[3], &a[4]),
        "payg-withholding" => engine::payg_withholding(&a[0], &a[1], &a[2], &a[3], &a[4]),
        "study-loan-withholding" => {
            engine::study_loan_withholding(&a[0], &a[1], &a[2], &a[3], &a[4])
        }
        "medicare-levy" => engine::medicare_levy(&a[0], &a[1], &a[2], &a[3]),
        "low-income-tax-offset" => engine::low_income_tax_offset(&a[0], &a[1], &a[2]),
        "division-293" => engine::division_293(&a[0], &a[1], &a[2], &a[3]),
        "study-loan-repayment" => engine::study_loan_repayment(&a[0], &a[1], &a[2]),
        "contribution-caps" => {
            engine::contribution_caps(&a[0], &a[1], &a[2], &a[3], &a[4], &a[5], &a[6])
        }
        "pension-minimum" => engine::pension_minimum(&a[0], &a[1], &a[2], &a[3], &a[4]),
        _ => unreachable!("listed command"),
    };
    result.map_err(|e| e.to_string())
}
