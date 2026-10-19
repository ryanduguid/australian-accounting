//! Read the contract CSV and the optional heading map, as `wiptally.csv_io` did.

use crate::csvread::{ReadError, Reader};
use crate::decimal::{Decimal, Signal};
use crate::engine::{
    ContractInput, PROGRESS_COST_TO_COST, PROGRESS_METHODS, RETENTION_CLASSES, RETENTION_REVIEW,
};
use crate::money::{AmountError, d, parse_bool, parse_money, parse_ratio};
use crate::py::{self, OsStyle, PyPath};
use crate::pyjson::{self, Json};
use crate::pytext;

pub const CANONICAL_FIELDS: [&str; 27] = [
    "contract_id",
    "customer",
    "description",
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
    "retention_classification",
    "committed_outstanding",
    "outcome_reasonably_measurable",
    "recoverable_costs",
    "progress_method",
    "output_percent",
    "prior_transaction_price",
    "prior_estimated_cost_at_completion",
    "prior_costs_incurred",
    "prior_estimated_cost_to_complete",
    "prior_revenue_to_date",
    "gst_rate",
    "assets_used_carrying",
];

const REQUIRED_FIELDS: [&str; 5] = [
    "contract_id",
    "original_contract_sum",
    "costs_incurred",
    "estimated_cost_to_complete",
    "certified_billings",
];

/// A run that cannot produce a schedule: the one-line `error:` message, or a
/// decimal signal the Python code raised uncaught.
#[derive(Debug, PartialEq, Eq)]
pub enum Failure {
    Message(String),
    Decimal(Signal),
}

impl From<Signal> for Failure {
    fn from(signal: Signal) -> Self {
        Failure::Decimal(signal)
    }
}

impl From<AmountError> for Failure {
    fn from(error: AmountError) -> Self {
        match error {
            AmountError::Message(m) => Failure::Message(m),
            AmountError::Decimal(s) => Failure::Decimal(s),
        }
    }
}

impl From<crate::engine::ScheduleError> for Failure {
    fn from(error: crate::engine::ScheduleError) -> Self {
        match error {
            crate::engine::ScheduleError::Message(m) => Failure::Message(m),
            crate::engine::ScheduleError::Decimal(s) => Failure::Decimal(s),
        }
    }
}

fn fail<T>(message: String) -> Result<T, Failure> {
    Err(Failure::Message(message))
}

/// Canonical field to source heading, in canonical order.
pub type Mapping = Vec<(&'static str, String)>;

pub fn heading<'m>(mapping: &'m Mapping, field: &str) -> &'m str {
    &mapping
        .iter()
        .find(|(f, _)| *f == field)
        .expect("canonical field")
        .1
}

/// `load_mapping`: canonical field to source heading, defaulting to identity.
pub fn load_mapping(path: Option<&PyPath>) -> Result<Mapping, Failure> {
    let mut mapping: Mapping = CANONICAL_FIELDS
        .iter()
        .map(|f| (*f, f.to_string()))
        .collect();
    let Some(path) = path else {
        return Ok(mapping);
    };
    let shown = path.display();
    let text = match py::read_text(&path.fs()) {
        Ok(text) => text,
        Err(py::ReadError::Os(e)) => {
            return fail(format!(
                "cannot read mapping file {shown}: {}",
                py::oserror_str(&e, &shown, OsStyle::Errno)
            ));
        }
        Err(py::ReadError::Decode(e)) => {
            return fail(format!("mapping file {shown} is not UTF-8: {e}"));
        }
    };
    let payload = match pyjson::loads(&text) {
        Ok(payload) => payload,
        Err(e) => return fail(format!("mapping file {shown} is not JSON: {e}")),
    };
    let Json::Object(pairs) = payload else {
        return fail(format!("mapping file {shown} must be a JSON object"));
    };
    for (key, value) in pairs {
        let Some(slot) = mapping.iter_mut().find(|(f, _)| *f == key) else {
            return fail(format!(
                "mapping file names unknown field {}",
                pytext::repr(&key)
            ));
        };
        let heading = match &value {
            Json::Str(s) if !pytext::strip(s).is_empty() => pytext::strip(s).to_string(),
            _ => {
                return fail(format!(
                    "mapping for {key} must be a non-empty column heading"
                ));
            }
        };
        slot.1 = heading;
    }
    Ok(mapping)
}

/// Heading (casefolded) to column position; a repeated heading is refused.
fn heading_index(header: &[String]) -> Result<Vec<(String, usize)>, Failure> {
    let mut index: Vec<(String, usize)> = Vec::new();
    for (position, raw) in header.iter().enumerate() {
        let name = pytext::strip(raw);
        if name.is_empty() {
            continue;
        }
        let key = py::casefold(name);
        if index.iter().any(|(k, _)| *k == key) {
            return fail(format!("duplicated column heading {}", pytext::repr(name)));
        }
        index.push((key, position));
    }
    Ok(index)
}

fn lookup(index: &[(String, usize)], heading: &str) -> Option<usize> {
    let key = py::casefold(heading);
    index.iter().find(|(k, _)| *k == key).map(|(_, p)| *p)
}

/// `_cell`: the stripped value, or None where absent or blank.
fn cell(row: &[String], index: &[(String, usize)], heading: &str) -> Option<String> {
    let position = lookup(index, heading)?;
    let value = pytext::strip(row.get(position)?);
    (!value.is_empty()).then(|| value.to_string())
}

/// `read_contracts`. `shown` is `str(path)`; `bytes` the file or the snapshot.
pub fn read_contracts(
    shown: &str,
    bytes: &[u8],
    mapping: &Mapping,
) -> Result<Vec<ContractInput>, Failure> {
    let mut reader = Reader::new(bytes);
    let next = |reader: &mut Reader| -> Result<Option<Vec<String>>, Failure> {
        reader.next_row().map_err(|e| {
            Failure::Message(match e {
                ReadError::Decode { start, reason } => format!(
                    "cannot read {shown}: it is not UTF-8 at byte {start} ({reason}); re-export it as UTF-8"
                ),
                ReadError::Csv(message) => format!("cannot read {shown} as CSV: {message}"),
            })
        })
    };
    let Some(header) = next(&mut reader)? else {
        return fail(format!("{shown} is empty"));
    };
    let index = heading_index(&header)?;
    let missing: Vec<&str> = REQUIRED_FIELDS
        .iter()
        .map(|f| heading(mapping, f))
        .filter(|h| lookup(&index, h).is_none())
        .collect();
    if !missing.is_empty() {
        return fail(format!(
            "{shown} is missing required column(s): {}",
            missing.join(", ")
        ));
    }

    let mut contracts: Vec<ContractInput> = Vec::new();
    let mut seen: Vec<(String, i64)> = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    let mut line_number: i64 = 1;
    while let Some(row) = next(&mut reader)? {
        line_number += 1;
        if row.is_empty() || row.iter().all(|c| pytext::strip(c).is_empty()) {
            continue;
        }
        let parsed = if row.len() != header.len() {
            Err(Failure::Message(format!(
                "row {line_number}: has {} field(s) but the header has {}; a short row would \
                 read its missing trailing columns as absent",
                row.len(),
                header.len()
            )))
        } else {
            parse_row(&row, &index, mapping, line_number)
        };
        let contract = match parsed {
            Ok(contract) => contract,
            Err(Failure::Message(message)) => {
                errors.push(message);
                // One past the cap, so the "... and more" suffix can fire.
                if errors.len() > 20 {
                    break;
                }
                continue;
            }
            Err(signal) => return Err(signal),
        };
        if let Some((_, previous)) = seen.iter().find(|(id, _)| *id == contract.contract_id) {
            errors.push(format!(
                "row {line_number}: contract_id {} already used on row {previous}",
                pytext::repr(&contract.contract_id)
            ));
            continue;
        }
        seen.push((contract.contract_id.clone(), line_number));
        contracts.push(contract);
    }

    if !errors.is_empty() {
        let listed = errors[..errors.len().min(20)].join("\n  ");
        let more = if errors.len() <= 20 {
            ""
        } else {
            "\n  ... and more"
        };
        return fail(format!("{shown} has unreadable cells:\n  {listed}{more}"));
    }
    if contracts.is_empty() {
        return fail(format!("{shown} has a header but no contract rows"));
    }
    Ok(contracts)
}

/// `_parse_row`, evaluating cells in the original order so the first refusal matches.
fn parse_row(
    row: &[String],
    index: &[(String, usize)],
    mapping: &Mapping,
    line: i64,
) -> Result<ContractInput, Failure> {
    let get = |field: &str| cell(row, index, heading(mapping, field));
    let Some(contract_id) = get("contract_id") else {
        return fail(format!("row {line}: contract_id is blank"));
    };
    let where_ = |field: &str| format!("row {line} ({contract_id}): {field}");
    let money_req = |field: &str| -> Result<Decimal, Failure> {
        match get(field) {
            None => fail(format!("{} is blank", where_(field))),
            Some(raw) => Ok(parse_money(&raw, &where_(field))?),
        }
    };
    let money_zero = |field: &str| -> Result<Decimal, Failure> {
        match get(field) {
            None => Ok(d("0.00")),
            Some(raw) => Ok(parse_money(&raw, &where_(field))?),
        }
    };
    let money_none = |field: &str| -> Result<Option<Decimal>, Failure> {
        match get(field) {
            None => Ok(None),
            Some(raw) => Ok(Some(parse_money(&raw, &where_(field))?)),
        }
    };
    let ratio = |field: &str, default: Option<&str>| -> Result<Option<Decimal>, Failure> {
        match get(field) {
            None => Ok(default.map(d)),
            Some(raw) => Ok(Some(parse_ratio(&raw, &where_(field))?)),
        }
    };

    let constraint = ratio("constraint_include_ratio", Some("0"))?.expect("default");
    let measurable = match get("outcome_reasonably_measurable") {
        None => None,
        Some(raw) => Some(parse_bool(&raw, &where_("outcome_reasonably_measurable"))?),
    };
    let method = py::casefold(pytext::strip(
        &get("progress_method").unwrap_or_else(|| PROGRESS_COST_TO_COST.into()),
    ));
    if !PROGRESS_METHODS.contains(&method.as_str()) {
        return fail(format!(
            "{} {} is not one of {}",
            where_("progress_method"),
            pytext::repr(&method),
            list_repr(&PROGRESS_METHODS)
        ));
    }
    let output_percent = ratio("output_percent", None)?;
    let retention = py::casefold(pytext::strip(
        &get("retention_classification").unwrap_or_else(|| RETENTION_REVIEW.into()),
    ));
    if !RETENTION_CLASSES.contains(&retention.as_str()) {
        return fail(format!(
            "{} {} is not one of {}",
            where_("retention_classification"),
            pytext::repr(&retention),
            list_repr(&RETENTION_CLASSES)
        ));
    }
    let gst_rate = ratio("gst_rate", Some("0.10"))?.expect("default");

    Ok(ContractInput {
        contract_id: contract_id.clone(),
        line_number: line,
        customer: get("customer").unwrap_or_default(),
        description: get("description").unwrap_or_default(),
        original_contract_sum: money_req("original_contract_sum")?,
        approved_variations: money_zero("approved_variations")?,
        unapproved_variations_estimate: money_zero("unapproved_variations_estimate")?,
        constraint_include_ratio: constraint,
        costs_incurred: money_req("costs_incurred")?,
        inefficiency_rework_wastage: money_zero("inefficiency_rework_wastage")?,
        uninstalled_materials: money_zero("uninstalled_materials")?,
        estimated_cost_to_complete: money_req("estimated_cost_to_complete")?,
        certified_billings: money_req("certified_billings")?,
        uncertified_claims: money_zero("uncertified_claims")?,
        retention_withheld: money_zero("retention_withheld")?,
        retention_classification: retention,
        committed_outstanding: money_zero("committed_outstanding")?,
        outcome_reasonably_measurable: measurable,
        recoverable_costs: money_none("recoverable_costs")?,
        progress_method: method,
        output_percent,
        prior_transaction_price: money_none("prior_transaction_price")?,
        prior_estimated_cost_at_completion: money_none("prior_estimated_cost_at_completion")?,
        prior_costs_incurred: money_none("prior_costs_incurred")?,
        prior_estimated_cost_to_complete: money_none("prior_estimated_cost_to_complete")?,
        prior_revenue_to_date: money_none("prior_revenue_to_date")?,
        gst_rate,
        assets_used_carrying: money_none("assets_used_carrying")?,
    })
}

/// `repr` of a list of strings.
fn list_repr(items: &[&str]) -> String {
    let parts: Vec<String> = items.iter().map(|s| pytext::repr(s)).collect();
    format!("[{}]", parts.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cell_past_the_end_of_its_row_reads_as_absent() {
        // read_contracts refuses short rows first; the helper still answers None.
        let index = vec![("customer".to_string(), 3)];
        assert_eq!(cell(&["JOB-1".to_string()], &index, "Customer"), None);
    }
}
