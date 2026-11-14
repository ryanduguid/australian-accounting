// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Translation to the linked checker's public assessment API.
use payday_super_checker::{
    self as engine, Failure,
    assess::{self, Assessed},
    calendar, csv_io,
    deadlines::ContribLine,
    decimal::Decimal,
    py::Date,
    rates, report,
};
use regex::Regex;
use serde_json::{Map, Value, json};
use std::sync::LazyLock;

pub const DISCLAIMER: &str = "Experimental review aid. Not a compliance determination, an ATO assessment or professional advice. payday-super-checker refuses or marks UNKNOWN where the supplied facts do not establish the statutory test (SGAA 1992 s 18C). This MCP does not model clearing-house latency. Fund receipt must be supplied before a contribution can be ON_TIME.";
const SINGLE: &str = "This review covers one contribution. It does not review related contributions, allocate receipts across QE days or apply s 18C(2) item 4 deadline alignment. Those facts can change the deadline or shortfall. Use payday-super-checker's full contribution review with the related evidence before drawing a conclusion. The supplied sg_amount is not a calculation of SG entitlement.";
/// Carried by every ON_TIME verdict or horizon candidate, because the server's
/// assessment takes no fund return. It applies no rule of its own.
const RETURNED: &str = "aus-accounting-mcp: returned contributions were not assessed, so ON_TIME assumes the fund kept every receipt. Check fund records before relying on it.";
/// A caller that knows of a return gets a refusal rather than a review that reads the
/// receipt as kept, as the Python server does until the MCP passes returns through.
const RETURNED_REFUSAL: &str = "the fund returned some or all of this contribution, and the payday-super-checker this server links cannot assess that, so it would read the receipt as kept. Review the row with the native payday-super-check command, using its returned_by_fund column and --events file, or remove the row once the fund's records settle it.";
static NUMERIC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d{1,2})[/-](\d{1,2})[/-]\d{2,4}").unwrap());

fn failure(error: Failure) -> String {
    if matches!(error, Failure::Overflow) {
        "A date is too far in the future to work with. Check for placeholder dates such as 9999-12-31.".into()
    } else {
        error.message().replace(
            "--confirm-transition-allocation",
            "a human reconciliation of June-quarter balances; this MCP cannot confirm that",
        )
    }
}

fn date(text: &str, field: &str) -> Result<Date, String> {
    if let Some(capture) = NUMERIC.captures(text)
        && text[capture[0].len()..]
            .chars()
            .next()
            .is_none_or(engine::py::is_space)
    {
        let number = |part: &str| {
            part.chars()
                .fold(0, |n, c| n * 10 + engine::py::decimal_value(c).unwrap())
        };
        let first = number(&capture[1]);
        let second = number(&capture[2]);
        if first != second && first <= 12 && second <= 12 {
            return Err(format!(
                "{field}: {} is ambiguous. A numeric date like this is read day first, so it means day {first} of month {second}, but nothing in the text rules out day {second} of month {first}, and the difference decides the verdict. Supply it as YYYY-MM-DD.",
                div7a_loan_review::raw::Raw::text(text).py_repr()
            ));
        }
    }
    csv_io::parse_date_text(text).map_err(|e|format!("{field}: {}",e.message()))?.ok_or_else(||format!("{field}: {} is not a date this tool reads. Use YYYY-MM-DD; a day-first numeric date, a form like '9 Jul 2026', and a date-time with no timezone marker are also read.",div7a_loan_review::raw::Raw::text(text).py_repr()))
}
fn required_date(text: &str, field: &str) -> Result<Date, String> {
    let text = super::money::strip(text);
    if text.is_empty() {
        return Err(format!("{field} is required (YYYY-MM-DD)"));
    }
    date(text, field)
}
fn optional_date(input: &Value, field: &str) -> Result<Option<Date>, String> {
    match input[field]
        .as_str()
        .map(super::money::strip)
        .filter(|s| !s.is_empty())
    {
        None => Ok(None),
        Some(text) => date(text, field).map(Some),
    }
}
fn amount(text: &str, field: &str) -> Result<Decimal, String> {
    super::money::amount(text, field)?;
    Decimal::parse(super::money::strip(text)).map_err(|e| e.name().into())
}
fn optional_amount(input: &Value, field: &str) -> Result<Option<Decimal>, String> {
    super::money::optional(input.get(field), field)?
        .map(|d| Decimal::parse(&d.to_string()).map_err(|e| e.name().into()))
        .transpose()
}
fn line(input: &Value, row: i64) -> Result<ContribLine, String> {
    if input["returned_by_fund"] == true {
        return Err(format!("row {row}: {RETURNED_REFUSAL}"));
    }
    let qe_day = required_date(input["qe_day"].as_str().unwrap(), "qe_day")?;
    let sg_amount = amount(input["sg_amount"].as_str().unwrap(), "sg_amount")?;
    let remitted = optional_date(input, "remitted")?;
    let remitted_amount = optional_amount(input, "remitted_amount")?;
    let matched_amount = optional_amount(input, "matched_amount")?;
    let received = optional_date(input, "received")?;
    let next_standard_qe_day = optional_date(input, "next_standard_qe_day")?;
    Ok(ContribLine {
        employee_id: input["employee_id"].as_str().unwrap_or("mcp-1").into(),
        qe_day,
        sg_amount,
        remitted,
        remitted_amount,
        matched_amount,
        received,
        next_standard_qe_day,
        first_to_fund: input["first_to_fund"].as_bool().unwrap_or(false),
        out_of_cycle: input["out_of_cycle"].as_bool().unwrap_or(false),
        db_interest: input["db_interest"].as_bool().unwrap_or(false),
        row,
        duplicate_note: String::new(),
        join_caveats: vec![],
        returned_by_fund: false,
        events: vec![],
        return_not_shown: false,
        // These tools take no receipt_evidence, mvr_status or fund_error_ref
        // argument and refuse unknown ones, so the line states no evidence.
        evidence: Default::default(),
    })
}
fn relies_on_kept_receipts(result: &Assessed) -> bool {
    result.verdict == assess::ON_TIME
        || result
            .horizon_verdicts
            .as_ref()
            .is_some_and(|(a, b)| a == assess::ON_TIME || b == assess::ON_TIME)
}
fn qualification() -> Value {
    json!({"code":"RETURNED_CONTRIBUTIONS_NOT_ASSESSED","returned_contributions_assessed":false,"assumes_fund_kept_every_receipt":true})
}
fn money(value: Option<&Decimal>) -> Value {
    value
        .map(|d| json!(d.to_sci_string()))
        .unwrap_or(Value::Null)
}
fn serialise(result: &Assessed) -> Result<Value, String> {
    let figures = report::rounded_figures(result).map_err(failure)?;
    let uplift = if let Some(rows) = &result.uplift {
        let mut output = Map::new();
        for (name, scenarios) in rows {
            let mut items = Map::new();
            for (key, value) in scenarios {
                items.insert(
                    key.clone(),
                    json!(csv_io::cents(Some(value)).map_err(failure)?.to_sci_string()),
                );
            }
            output.insert(name.clone(), Value::Object(items));
        }
        Value::Object(output)
    } else {
        Value::Null
    };
    let mut caveats = result.caveats.clone();
    caveats.push(SINGLE.into());
    let line = &result.line;
    let qualified = relies_on_kept_receipts(result);
    let mut output = json!({"employee_id":line.employee_id,"qe_day":line.qe_day.isoformat(),"sg_amount":line.sg_amount.to_sci_string(),
        "remitted_amount":money(line.remitted_amount.as_ref()),"matched_amount":money(line.matched_amount.as_ref()),
        "remitted":line.remitted.map(|d|d.isoformat()),"received":line.received.map(|d|d.isoformat()),"due":result.deadline.due.map(|d|d.isoformat()),
        "pathway":result.deadline.pathway,"verdict":result.verdict,"days_late":result.days_late,
        "lateness_basis":if result.lateness_basis.is_empty() {None}else{Some(&result.lateness_basis)},
        "base_shortfall":money(result.base_shortfall.as_ref()),"final_shortfall":money(result.final_shortfall.as_ref()),
        "notional_earnings":money(figures.nec.as_ref()),"experimental_sgc_low":money(figures.low.as_ref()),"experimental_sgc_high":money(figures.high.as_ref()),
        "uplift":uplift,"notes":result.notes,"caveats":caveats,"horizon_verdicts":result.horizon_verdicts.as_ref().map(|(a,b)|json!([a,b]))});
    if qualified {
        output["verdict_qualification"] = qualification();
    }
    Ok(output)
}
pub fn evaluate(input: &Value) -> Result<Value, String> {
    let lines = vec![line(input, 1)?];
    let as_at = required_date(input["as_at"].as_str().unwrap(), "as_at")?;
    let gic = rates::load_gic().map_err(failure)?;
    let calendar = calendar::load_calendar(None).map_err(failure)?;
    let mut results = assess::assess(lines, &calendar, &gic, as_at, assess::Options::default())
        .map_err(failure)?;
    for result in &mut results {
        if relies_on_kept_receipts(result) {
            result.caveats.push(RETURNED.into());
        }
    }
    let mut output = json!({"ok":true,"engine":"payday-super-checker","engine_version":engine::VERSION,"law_content_date":engine::LAW_CONTENT_DATE,"as_at":as_at.isoformat(),"disclaimer":DISCLAIMER});
    output["assessment_scope"] = json!("single_contribution");
    output["result"] = serialise(&results[0])?;
    Ok(output)
}
