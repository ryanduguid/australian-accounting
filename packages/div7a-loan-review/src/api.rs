//! JSON stdin boundary for the removed Python import API. Each command takes
//! one JSON object: decimals as strings, years of income as labels such as
//! "2026-27", facts as JSON booleans or null, and rate tables and overrides
//! as file paths. The result is the object the Python `to_json_dict()` gave.

use serde_json::{Value, json};

use crate::Failure;
use crate::decimal::Decimal;
use crate::facts::{
    optional_money, optional_rate, optional_year_of_income, optional_years, parse_tristate,
};
use crate::gate::{GateFacts, complying_loan_gate};
use crate::money::{parse_money, parse_rate, parse_ratio};
use crate::myr::{
    FormulaError, GateSummary, MyrFacts, minimum_yearly_repayment, minimum_yearly_repayment_amount,
    statutory_remaining_term,
};
use crate::py::PyPath;
use crate::rates::{
    BenchmarkTable, RateOverride, TableSource, benchmark_rate, load_override, load_table,
};
use crate::raw::Raw;
use crate::register::{Mode, load_rows, require_columns, review_register, review_register_file};
use crate::years::{YearOfIncome, parse_year};

type Result<T> = std::result::Result<T, String>;

pub const COMMANDS: [&str; 21] = [
    "benchmark-rate",
    "complying-loan-gate",
    "minimum-yearly-repayment",
    "minimum-yearly-repayment-amount",
    "review-register",
    "review-register-file",
    "parse-year",
    "parse-money",
    "parse-rate",
    "parse-ratio",
    "load-table",
    "load-override",
    "parse-tristate",
    "optional-money",
    "optional-rate",
    "optional-years",
    "optional-year-of-income",
    "statutory-remaining-term",
    "year-of-income",
    "load-rows",
    "require-columns",
];

fn failure(f: Failure) -> String {
    f.to_string()
}

fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{key} must be a string"))
}

fn optional_text<'a>(v: &'a Value, key: &str) -> Result<Option<&'a str>> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(s)),
        Some(_) => Err(format!("{key} must be a string or null")),
    }
}

fn decimal(v: &Value, key: &str) -> Result<Option<Decimal>> {
    match optional_text(v, key)? {
        None => Ok(None),
        Some(text) => Decimal::parse(text)
            .map(Some)
            .map_err(|s| format!("decimal.{}", s.name())),
    }
}

fn boolean(v: &Value, key: &str) -> Result<Option<bool>> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(b)) => Ok(Some(*b)),
        Some(_) => Err(format!("{key} must be a boolean or null")),
    }
}

fn year(v: &Value, key: &str) -> Result<Option<YearOfIncome>> {
    match optional_text(v, key)? {
        None => Ok(None),
        Some(label) => parse_year(&Raw::text(label), key).map(Some),
    }
}

fn table(v: &Value) -> Result<Option<BenchmarkTable>> {
    optional_text(v, "table")?
        .map(|p| load_table(&PyPath::new(p)).map_err(failure))
        .transpose()
}

fn over(v: &Value) -> Result<Option<RateOverride>> {
    optional_text(v, "override")?
        .map(|p| load_override(&PyPath::new(p)).map_err(failure))
        .transpose()
}

/// The `where` keyword: its default when absent, otherwise `str()` of the
/// value given, `None` included, as the f-strings that quote it print it.
fn where_(v: &Value, default: &str) -> Result<String> {
    Ok(match v.get("where") {
        None => default.to_string(),
        Some(value) => Raw::from_value(value).py_str(),
    })
}

/// A register row as `from_mapping` reads it: anything but a dict fails on
/// its first `row.get`, as Python's did.
fn mapping_of(value: &Value) -> Result<Raw> {
    match Raw::from_value(value) {
        row @ Raw::Dict(_) => Ok(row),
        other => Err(format!(
            "AttributeError: '{}' object has no attribute 'get'",
            other.type_name()
        )),
    }
}

fn gate_facts(v: &Value) -> Result<GateFacts> {
    if let Some(mapping) = v.get("mapping") {
        return GateFacts::from_mapping(&mapping_of(mapping)?, &where_(v, "row")?);
    }
    let f = v.get("facts").ok_or("facts or mapping is required")?;
    Ok(GateFacts {
        maximum_term_years: decimal(f, "maximum_term_years")?,
        written_agreement: boolean(f, "written_agreement")?,
        terms_in_place_before_lodgment_day: boolean(f, "terms_in_place_before_lodgment_day")?,
        secured_by_registered_mortgage_over_real_property: boolean(
            f,
            "secured_by_registered_mortgage_over_real_property",
        )?,
        security_coverage_at_first_made: decimal(f, "security_coverage_at_first_made")?,
        interest_rate_for_years_after_year_loan_made: decimal(
            f,
            "interest_rate_for_years_after_year_loan_made",
        )?,
        year_of_income_being_tested: year(f, "year_of_income_being_tested")?,
        year_loan_made: year(f, "year_loan_made")?,
        loan_id: optional_text(f, "loan_id")?.unwrap_or_default().to_string(),
    })
}

fn strings(v: &Value, key: &str) -> Result<Vec<String>> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(vec![]),
        Some(Value::Array(items)) => items
            .iter()
            .map(|i| {
                i.as_str()
                    .map(str::to_string)
                    .ok_or(format!("{key} must hold strings"))
            })
            .collect(),
        Some(_) => Err(format!("{key} must be a list of strings")),
    }
}

/// `for name in value`, as Python iterated a required argument: a string by
/// character, a dict by key, a list by item; `None` and numbers cannot be.
fn iterable(v: &Value, key: &str, none_is_empty: bool) -> Result<Vec<String>> {
    match v.get(key) {
        None => Err(format!("{key} is required")),
        Some(Value::Null) if none_is_empty => Ok(vec![]),
        Some(Value::String(s)) => Ok(s.chars().map(String::from).collect()),
        Some(Value::Object(items)) => Ok(items.keys().cloned().collect()),
        Some(Value::Array(items)) => items
            .iter()
            .map(|i| {
                i.as_str()
                    .map(str::to_string)
                    .ok_or(format!("{key} must hold strings"))
            })
            .collect(),
        Some(other) => Err(format!(
            "TypeError: '{}' object is not iterable",
            Raw::from_value(other).type_name()
        )),
    }
}

fn gate_summary(v: &Value) -> Result<Option<GateSummary>> {
    match v.get("gate_result") {
        None | Some(Value::Null) => Ok(None),
        Some(g) => Ok(Some(GateSummary {
            // `GateVerdict(value)` accepts its three members only.
            verdict: match text(g, "verdict")? {
                v @ ("COMPLYING" | "NOT_COMPLYING" | "UNKNOWN") => v.to_string(),
                other => {
                    return Err(format!(
                        "{} is not a valid GateVerdict",
                        crate::pytext::repr(other)
                    ));
                }
            },
            benchmark_year_used: optional_text(g, "benchmark_year_used")?
                .unwrap_or_default()
                .to_string(),
            reasons: strings(g, "reasons")?,
            caveats: strings(g, "caveats")?,
        })),
    }
}

fn myr_facts(v: &Value) -> Result<MyrFacts> {
    if let Some(mapping) = v.get("mapping") {
        let year = year(v, "year_of_income")?.ok_or("year_of_income is required")?;
        return MyrFacts::from_mapping(
            &mapping_of(mapping)?,
            year,
            gate_summary(v)?,
            &where_(v, "row")?,
        );
    }
    let f = v.get("facts").ok_or("facts or mapping is required")?;
    Ok(MyrFacts {
        year_of_income: year(f, "year_of_income")?,
        amalgamated_loan_unpaid_at_end_of_previous_year: decimal(
            f,
            "amalgamated_loan_unpaid_at_end_of_previous_year",
        )?,
        remaining_term_years: decimal(f, "remaining_term_years")?,
        payments_applied_during_the_year: decimal(f, "payments_applied_during_the_year")?,
        gate_result: gate_summary(f)?,
        year_loan_made: year(f, "year_loan_made")?,
        loan_id: optional_text(f, "loan_id")?.unwrap_or_default().to_string(),
    })
}

fn mode(v: &Value) -> Result<Mode> {
    let flag = |key: &str| match v.get(key) {
        None => Ok(false),
        Some(Value::Bool(b)) => Ok(*b),
        Some(_) => Err(format!("{key} must be a boolean")),
    };
    let mode = Mode {
        gate_only: flag("gate_only")?,
        myr_only: flag("myr_only")?,
    };
    if mode.gate_only && mode.myr_only {
        return Err("gate_only and myr_only cannot both be True".into());
    }
    Ok(mode)
}

/// A register row as `[[column, value], ...]`, in header order.
fn row_json(row: &Raw) -> Value {
    let Raw::Dict(items) = row else {
        return Value::Null;
    };
    json!(
        items
            .iter()
            .map(|(k, v)| json!([
                k,
                if let Raw::Str(s) = v {
                    json!(s)
                } else {
                    Value::Null
                }
            ]))
            .collect::<Vec<_>>()
    )
}

fn sources(sources: &[TableSource]) -> Value {
    json!(sources.iter().map(TableSource::to_json).collect::<Vec<_>>())
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
        "benchmark-rate" => {
            // The caller loads the table and override before the call parses the year.
            let (table, over) = (table(v)?, over(v)?);
            let raw = Raw::from_value(v.get("year_of_income").unwrap_or(&Value::Null));
            let year = parse_year(&raw, "year of income")?;
            Ok(benchmark_rate(year, table.as_ref(), over.as_ref()).to_json())
        }
        "complying-loan-gate" => {
            let facts = gate_facts(v)?;
            let (table, over) = (table(v)?, over(v)?);
            complying_loan_gate(&facts, table.as_ref(), over.as_ref())
                .map(|g| g.to_json())
                .map_err(failure)
        }
        "minimum-yearly-repayment" => {
            let facts = myr_facts(v)?;
            let (table, over) = (table(v)?, over(v)?);
            let result =
                minimum_yearly_repayment(&facts, table.as_ref(), over.as_ref()).map_err(failure)?;
            result
                .to_json()
                .map_err(|s| format!("decimal.{}", s.name()))
        }
        "minimum-yearly-repayment-amount" => {
            let get = |key: &str| decimal(v, key)?.ok_or(format!("{key} is required"));
            let (p, r, t) = (get("principal")?, get("rate")?, get("remaining_term")?);
            match minimum_yearly_repayment_amount(&p, &r, &t) {
                Ok(amount) => Ok(json!(amount.to_sci_string())),
                Err(FormulaError::Value(message)) => Err(message.into()),
                Err(FormulaError::Signal(s)) => Err(format!("decimal.{}", s.name())),
            }
        }
        "review-register" | "review-register-file" => {
            let label = year(v, "year_of_income")?.ok_or("year_of_income is required")?;
            let gate_year = year(v, "gate_benchmark_year")?;
            let (table, over) = (table(v)?, over(v)?);
            let mode = mode(v)?;
            let report = if command == "review-register" {
                let rows = match v.get("rows") {
                    Some(Value::Array(rows)) => {
                        rows.iter().map(Raw::from_value).collect::<Vec<_>>()
                    }
                    _ => return Err("rows must be a list".into()),
                };
                review_register(&rows, label, mode, gate_year, table.as_ref(), over.as_ref())
            } else {
                review_register_file(
                    text(v, "path")?,
                    label,
                    mode,
                    gate_year,
                    table.as_ref(),
                    over.as_ref(),
                )
            }
            .map_err(failure)?;
            report
                .to_json()
                .map_err(|s| format!("decimal.{}", s.name()))
        }
        "parse-year" => {
            let raw = Raw::from_value(v.get("raw").unwrap_or(&Value::Null));
            Ok(json!(
                parse_year(&raw, &where_(v, "year of income")?)?.label()
            ))
        }
        "parse-money" | "parse-rate" | "parse-ratio" => {
            let raw = Raw::from_value(v.get("raw").unwrap_or(&Value::Null));
            let at = where_(v, "value")?;
            let value = match command {
                "parse-money" => parse_money(&raw, &at),
                "parse-rate" => parse_rate(&raw, &at),
                _ => parse_ratio(&raw, &at),
            }?;
            Ok(json!(value.to_sci_string()))
        }
        "load-table" => {
            // `load_table(path=None)` reads the frozen table.
            let table = match optional_text(v, "path")? {
                None => crate::rates::frozen_table(),
                Some(path) => load_table(&PyPath::new(path)).map_err(failure)?,
            };
            Ok(json!({
                "reviewed_until": table.reviewed_until.label(),
                "reviewed_on": table.reviewed_on,
                "rates": table.entries.values().map(|e| json!([e.year.label(), e.rate.to_sci_string()])).collect::<Vec<_>>(),
                "sources": sources(&table.sources),
            }))
        }
        "load-override" => {
            let over = load_override(&PyPath::new(text(v, "path")?)).map_err(failure)?;
            Ok(json!({
                "verified_until": over.verified_until.label(),
                "citation": over.citation,
                "rates": over.entries.iter().map(|e| json!([e.year.label(), e.rate.to_sci_string(), e.rba_table, e.rba_series, e.rba_month, e.source, e.verify_at, e.seen])).collect::<Vec<_>>(),
                "sources": sources(std::slice::from_ref(&over.source)),
            }))
        }
        "parse-tristate" => {
            let raw = Raw::from_value(v.get("raw").unwrap_or(&Value::Null));
            Ok(json!(parse_tristate(Some(&raw), &where_(v, "value")?)?))
        }
        "optional-money" | "optional-rate" | "optional-years" => {
            let raw = Raw::from_value(v.get("raw").unwrap_or(&Value::Null));
            let at = where_(v, "value")?;
            let value = match command {
                "optional-money" => optional_money(Some(&raw), &at),
                "optional-rate" => optional_rate(Some(&raw), &at),
                _ => optional_years(Some(&raw), &at),
            }?;
            Ok(json!(value.map(|d| d.to_sci_string())))
        }
        "optional-year-of-income" => {
            let raw = Raw::from_value(v.get("raw").unwrap_or(&Value::Null));
            let year = optional_year_of_income(Some(&raw), &where_(v, "value")?)?;
            Ok(json!(year.map(|y| y.label())))
        }
        "statutory-remaining-term" => {
            let supplied = decimal(v, "supplied")?.ok_or("supplied is required")?;
            statutory_remaining_term(&supplied)
                .map(|d| json!(d.to_sci_string()))
                .map_err(|s| format!("decimal.{}", s.name()))
        }
        "year-of-income" => match v.get("start_year") {
            Some(Value::Number(n)) if n.is_i64() => {
                YearOfIncome::new(n.as_i64().expect("i64")).map(|y| json!(y.label()))
            }
            // An int beyond 64 bits is still an int, and out of range.
            Some(Value::Number(n))
                if matches!(Raw::from_value(&Value::Number(n.clone())), Raw::Int(_)) =>
            {
                Err(format!(
                    "year of income {n}-.. is outside 1900-2100; this looks like a typo rather than a year"
                ))
            }
            other => {
                let raw = Raw::from_value(other.unwrap_or(&Value::Null));
                Err(format!(
                    "year of income start {} must be a whole year",
                    raw.py_repr()
                ))
            }
        },
        "load-rows" => {
            let needed = iterable(v, "needed", false)?;
            let needed: Vec<&str> = needed.iter().map(String::as_str).collect();
            let rows = load_rows(&PyPath::new(text(v, "path")?), &needed).map_err(failure)?;
            Ok(json!(rows.iter().map(row_json).collect::<Vec<_>>()))
        }
        "require-columns" => {
            // `set(fieldnames or [])`, but `needed` is iterated as given.
            let fieldnames = iterable(v, "fieldnames", true)?;
            let needed = iterable(v, "needed", false)?;
            let needed: Vec<&str> = needed.iter().map(String::as_str).collect();
            require_columns(&fieldnames, &needed, text(v, "where")?)?;
            Ok(Value::Null)
        }
        _ => Err(format!("unknown command {command}")),
    }
}
