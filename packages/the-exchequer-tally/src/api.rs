//! JSON boundary for the removed Python import API. Monetary values are strings.
use crate::date::Date;
use crate::decimal::Decimal;
use crate::engine::{self, Account, Bre, Entry, EntryType, Event, Result, Statement};
use serde_json::{Value, json};

pub const COMMANDS: &[&str] = &[
    "corporate-tax",
    "bre-properties",
    "turnover-threshold",
    "max-franking-rate",
    "statement",
    "distribution-event",
    "benchmark",
    "franking-account",
    "validate-value",
];
pub fn text<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{key} must be a string"))
}
pub fn number(v: &Value, key: &str) -> Result<i64> {
    v.get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("{key} must be an integer"))
}
pub fn decimal(v: &Value, key: &str) -> Result<Decimal> {
    let t = text(v, key)?;
    if t.len() > 8192 {
        return Err(format!("{key}: decimal input exceeds 8192 bytes"));
    }
    let d = Decimal::parse(t).map_err(|_| format!("{key}: not a decimal amount"))?;
    if d.is_finite()
        && (d.exponent().unsigned_abs() > 10_000 || d.adjusted().unsigned_abs() > 10_000)
    {
        return Err(format!(
            "{key}: decimal exponent or adjusted exponent exceeds 10000"
        ));
    }
    Ok(d)
}
fn optional_decimal(v: &Value, key: &str) -> Result<Option<Decimal>> {
    if v.get(key).is_none_or(Value::is_null) {
        Ok(None)
    } else {
        decimal(v, key).map(Some)
    }
}
fn optional_text(v: &Value, key: &str) -> Result<Option<String>> {
    if v.get(key).is_none_or(Value::is_null) {
        Ok(None)
    } else {
        Ok(Some(text(v, key)?.into()))
    }
}
fn date(v: &Value, key: &str) -> Result<Date> {
    Date::parse(text(v, key)?).map_err(|_| format!("{key} must be YYYY-MM-DD"))
}
/// Omitted means "" and an explicit null stays null, as `v.get(key, "")` did.
fn reference(v: &Value) -> Result<Option<String>> {
    match v.get("statutory_reference") {
        None => Ok(Some(String::new())),
        Some(Value::Null) => Ok(None),
        Some(_) => Ok(Some(text(v, "statutory_reference")?.into())),
    }
}
fn array<'a>(v: &'a Value, key: &str) -> Result<&'a [Value]> {
    match v.get(key) {
        None => Ok(&[]),
        Some(Value::Array(a)) => Ok(a),
        _ => Err(format!("{key} must be an array")),
    }
}
pub fn bre(v: &Value) -> Result<Bre> {
    let b = Bre {
        fy: number(v, "financial_year")?,
        turnover: decimal(v, "aggregated_turnover")?,
        assessable: decimal(v, "assessable_income")?,
        passive: decimal(v, "passive_income")?,
    };
    b.validate()?;
    Ok(b)
}
fn event(v: &Value) -> Result<Event> {
    let e = Event {
        date: date(v, "event_date")?,
        recipient: text(v, "recipient_name")?.into(),
        amount: decimal(v, "distribution_amount")?,
        credit: decimal(v, "franking_credit")?,
        rate: optional_decimal(v, "corporate_tax_rate")?,
    };
    e.validate()?;
    Ok(e)
}
fn account(v: &Value) -> Result<Value> {
    let mut a = Account {
        fy: number(v, "financial_year")?,
        opening: optional_decimal(v, "opening_balance")?,
        entries: Vec::new(),
    };
    for e in array(v, "entries")? {
        a.entries.push(Entry::new(
            date(e, "entry_date")?,
            EntryType::parse(text(e, "entry_type")?)?,
            decimal(e, "amount")?,
            text(e, "description")?.into(),
            reference(e)?,
        )?);
    }
    a.validate()?;
    for e in array(v, "records")? {
        let classification = match e.get("includes_r_and_d_offset") {
            None | Some(Value::Null) => None,
            Some(Value::Bool(b)) => Some(*b),
            _ => return Err("includes_r_and_d_offset must be a boolean".into()),
        };
        a.record(
            date(e, "entry_date")?,
            EntryType::parse(text(e, "entry_type")?)?,
            decimal(e, "amount")?,
            optional_text(e, "description")?,
            classification,
        )?;
    }
    // Appending direct entries remains testable: every calculation revalidates periods.
    for e in array(v, "append_entries")? {
        a.entries.push(Entry::new(
            date(e, "entry_date")?,
            EntryType::parse(text(e, "entry_type")?)?,
            decimal(e, "amount")?,
            text(e, "description")?.into(),
            reference(e)?,
        )?);
    }
    a.json()
}
pub fn evaluate(command: &str, v: &Value) -> Result<Value> {
    if !v.is_object() {
        return Err("input must be a JSON object".into());
    }
    match command {
        "corporate-tax" => Ok(bre(v)?.corporate_tax()?.json()),
        "bre-properties" => {
            let b = bre(v)?;
            Ok(
                json!({"passive_income_percentage":b.passive_pct()?.map(|d| d.to_string()),"is_aggregated_turnover_eligible":b.turnover_eligible()?,"is_brepi_eligible":b.brepi(),"is_base_rate_entity":b.is_bre()?}),
            )
        }
        "turnover-threshold" => Ok(json!(
            engine::threshold(number(v, "financial_year")?)?.to_string()
        )),
        "max-franking-rate" => {
            let prior = v
                .get("prior_year_test")
                .filter(|v| !v.is_null())
                .map(bre)
                .transpose()?;
            Ok(json!(
                engine::max_franking_rate(number(v, "current_fy")?, prior)?.to_string()
            ))
        }
        "statement" => Ok(Statement::new(
            text(v, "entity_name")?.into(),
            text(v, "abn_or_acn")?.into(),
            text(v, "recipient_name")?.into(),
            date(v, "payment_date")?,
            decimal(v, "total_distribution")?,
            decimal(v, "franking_percentage")?,
            optional_decimal(v, "corporate_tax_rate")?,
        )?
        .json()),
        "distribution-event" => event(v)?.json(),
        "benchmark" => {
            // Python validates the validator's rate before constructing/adding events.
            let tax_rate = engine::rate(
                optional_decimal(v, "corporate_tax_rate")?.as_ref(),
                "BenchmarkRuleValidator",
            )?;
            let events = array(v, "distributions")?
                .iter()
                .map(event)
                .collect::<Result<Vec<_>>>()?;
            engine::benchmark(tax_rate, events)
        }
        "franking-account" => account(v),
        "validate-value" => {
            let t = text(v, "value")?;
            match text(v, "kind")? {
                "money" => Ok(json!(crate::cli::money(t)?.to_string())),
                "percentage" => Ok(json!(crate::cli::percentage(t)?.to_string())),
                "rate" => Ok(json!(crate::cli::tax_rate(t)?.to_string())),
                "date" => Ok(json!(Date::parse(t)?.to_string())),
                _ => Err("kind must be money, percentage, rate or date".into()),
            }
        }
        _ => Err(format!("unknown API command: {command}")),
    }
}
