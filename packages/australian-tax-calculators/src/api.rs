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
            None if n.is_u64() => Arg::BigInt(n.to_string()),
            None => Arg::Other(n.to_string()),
        },
        Value::String(s) => Arg::Text(s.clone()),
        Value::Null => Arg::Other("None".into()),
        other => Arg::Other(other.to_string()),
    }
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
            let kind = v
                .get("kind")
                .and_then(Value::as_str)
                .ok_or("kind must be a string")?;
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
