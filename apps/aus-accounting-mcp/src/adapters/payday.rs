// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Translation to the linked checker's public assessment and pack APIs.
use payday_super_checker::{
    self as engine, Failure,
    assess::{self, Assessed},
    calendar, csv_io,
    deadlines::ContribLine,
    decimal::Decimal,
    evidence_pack,
    py::Date,
    rates, report,
};
use regex::Regex;
use serde_json::{Map, Value, json};
use std::sync::LazyLock;

pub const DISCLAIMER: &str = "Experimental review aid. Not a compliance determination, an ATO assessment or professional advice. payday-super-checker refuses or marks UNKNOWN where the supplied facts do not establish the statutory test (SGAA 1992 s 18C). This MCP does not model clearing-house latency. Fund receipt must be supplied before a contribution can be ON_TIME.";
const SINGLE: &str = "This review covers one contribution. It does not review related contributions, allocate receipts across QE days or apply s 18C(2) item 4 deadline alignment. Those facts can change the deadline or shortfall. Use payday-super-checker's full contribution review with the related evidence before drawing a conclusion. The supplied sg_amount is not a calculation of SG entitlement.";
/// Carried by every ON_TIME verdict or horizon candidate, because no tool input
/// records a contribution the fund returned. It applies no rule of its own.
const RETURNED: &str = "aus-accounting-mcp: returned contributions were not assessed, so ON_TIME assumes the fund kept every receipt. Check fund records before relying on it.";
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
    })
}
fn relies_on_kept_receipts(result: &Assessed) -> bool {
    result.verdict == assess::ON_TIME
        || result
            .horizon_verdicts
            .as_ref()
            .is_some_and(|(a, b)| a == assess::ON_TIME || b == assess::ON_TIME)
}
fn qualification(statement: bool) -> Value {
    let mut out = json!({"code":"RETURNED_CONTRIBUTIONS_NOT_ASSESSED","returned_contributions_assessed":false,"assumes_fund_kept_every_receipt":true});
    if statement {
        out["statement"] = json!(RETURNED);
    }
    out
}
/// Python's `json.loads` then `json.dumps(indent=2, ensure_ascii=True)`; the
/// engine's queue holds no floats, so any other number is a broken invariant.
fn python_json(value: &Value) -> evidence_pack::Out {
    use evidence_pack::Out;
    match value {
        Value::Null => Out::Null,
        Value::Bool(b) => Out::Bool(*b),
        Value::Number(n) => {
            assert!(
                n.is_i64() || n.is_u64(),
                "engine pack JSON holds an integer"
            );
            Out::Int(n.to_string())
        }
        Value::String(s) => Out::Str(s.clone()),
        Value::Array(items) => Out::List(items.iter().map(python_json).collect()),
        Value::Object(map) => Out::Object(
            map.iter()
                .map(|(k, v)| (k.clone(), python_json(v)))
                .collect(),
        ),
    }
}
/// State the limit in the files too, since they are saved without the envelope.
/// The note precedes the engine's status line, whose verdict it qualifies.
fn qualify_pack_files(files: &mut Map<String, Value>, exceptions: &mut Value) {
    let notice = format!(
        "> **Limit on every ON_TIME verdict or candidate.** {RETURNED}

"
    );
    for name in ["practitioner-review.md", "decision-log.md"] {
        let text = files[name].as_str().unwrap();
        files[name] = Value::String(format!("{notice}{text}"));
    }
    exceptions["verdict_qualification"] = qualification(true);
    files["exceptions.json"] = Value::String(format!(
        "{}
",
        python_json(exceptions).dumps()
    ));
}
fn money(value: Option<&Decimal>) -> Value {
    value
        .map(|d| json!(d.to_sci_string()))
        .unwrap_or(Value::Null)
}
fn serialise(result: &Assessed, single: bool) -> Result<Value, String> {
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
    if single {
        caveats.push(SINGLE.into());
    }
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
        output["verdict_qualification"] = qualification(false);
    }
    if !single {
        output["input_row"] = json!(line.row);
    }
    Ok(output)
}
pub fn evaluate(name: &str, input: &Value) -> Result<Value, String> {
    let single = name == "calc_payday_super_deadline";
    let lines = if single {
        vec![line(input, 1)?]
    } else {
        input["contributions"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .map(|(i, value)| line(value, (i + 1) as i64))
            .collect::<Result<Vec<_>, _>>()?
    };
    let as_at = required_date(input["as_at"].as_str().unwrap(), "as_at")?;
    let gic = rates::load_gic().map_err(failure)?;
    let calendar = calendar::load_calendar(None).map_err(failure)?;
    let mut results = assess::assess(lines, &calendar, &gic, as_at, assess::Options::default())
        .map_err(failure)?;
    // On the engine's results, so report.csv carries the caveat in the row and
    // the pack's hashes bind the report as returned.
    for result in &mut results {
        if relies_on_kept_receipts(result) {
            result.caveats.push(RETURNED.into());
        }
    }
    let mut output = json!({"ok":true,"engine":"payday-super-checker","engine_version":engine::VERSION,"law_content_date":engine::LAW_CONTENT_DATE,"as_at":as_at.isoformat(),"disclaimer":DISCLAIMER});
    if name == "build_payday_super_evidence_pack" {
        let provenance = gic.provenance();
        let tables: Vec<_> = gic.source.clone().into_iter().collect();
        let mut files: Map<_, _> = evidence_pack::build_evidence_pack(
            &results,
            &evidence_pack::PackFacts {
                as_at,
                assessment_date: None,
                gic_provenance: &provenance,
                remittance_only_confirmed: false,
                rate_tables: &tables,
            },
        )
        .map_err(failure)?
        .into_iter()
        .map(|(name, text)| (name, Value::String(text)))
        .collect();
        let mut exceptions: Value =
            serde_json::from_str(files["exceptions.json"].as_str().unwrap())
                .expect("engine pack JSON");
        output["review_exit_code"] =
            json!(if exceptions["exceptions"].as_array().unwrap().is_empty() {
                0
            } else {
                2
            });
        let qualified = results.iter().any(relies_on_kept_receipts);
        if qualified {
            qualify_pack_files(&mut files, &mut exceptions);
        }
        output["files"] = Value::Object(files);
        output["caveats"] = json!([
            "Private review artefacts. No files are written. Save returned strings as UTF-8 without altering newlines or the CSV's initial BOM, so the report hash matches.",
            "Source row references are one-based positions in the supplied contribution list.",
            "Supply all related rows for one employer with established receipt allocation. No ATO assessment is assumed to have issued. Transition allocation is unconfirmed.",
            "Employee identifiers are omitted from the returned pack. The calling host still receives the input references; use an approved environment and fabricated demos."
        ]);
        if qualified {
            output["caveats"]
                .as_array_mut()
                .unwrap()
                .push(json!(RETURNED));
            output["verdict_qualification"] = qualification(true);
        }
    } else if single {
        output["assessment_scope"] = json!("single_contribution");
        output["result"] = serialise(&results[0], true)?;
    } else {
        output["assessment_scope"] = json!("contribution_group");
        output["results"] = Value::Array(
            results
                .iter()
                .map(|r| serialise(r, false))
                .collect::<Result<_, _>>()?,
        );
        output["caveats"] = json!([
            "Supply all related contributions for one employer. Employee references are exact.",
            "Receipt amounts must already be allocated to rows without double counting. This tool does not allocate raw payments or calculate SG entitlement.",
            "No ATO assessment is assumed to have issued. Transition allocation is unconfirmed."
        ]);
    }
    Ok(output)
}
