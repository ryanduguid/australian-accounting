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

/// `str()` of a scalar JSON value, as an f-string prints the Python object.
fn py_str(value: &Value) -> Result<String> {
    Ok(match value {
        Value::Null => "None".into(),
        Value::Bool(true) => "True".into(),
        Value::Bool(false) => "False".into(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        _ => return Err("where must be a string, number, boolean or null".into()),
    })
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

fn opt(value: &Option<Decimal>) -> Value {
    value.as_ref().map_or(Value::Null, |d| json!(d.to_string()))
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
        "margin_at_completion": opt(&p.margin_at_completion),
        "prior_margin_at_completion": opt(&p.prior_margin_at_completion),
        "profit_fade_points": opt(&p.profit_fade_points),
        "period_revenue": opt(&p.period_revenue),
        "gst_on_certified_billings": p.gst_on_certified_billings.to_string(),
        "gst_on_retention": p.gst_on_retention.to_string(),
        "flags": p.flags,
        "needs_review": p.needs_review(),
    })
}

/// Run one JSON stdin command: the result value, or the refusal message.
pub fn evaluate(command: &str, v: &Value) -> Result<Value> {
    if !v.is_object() {
        return Err("input must be a JSON object".into());
    }
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
