// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use crate::{CHECKED, Payment, Rule, data, holidays};
use serde_json::{Value, json};

pub(super) struct Pending<'a> {
    pub(super) payment: &'a str,
    pub(super) source: &'a str,
}

pub(super) fn make_rows(
    label: &str,
    period: &str,
    rule: Option<Rule>,
    missing: &[String],
    family: &str,
    pending: &Pending<'_>,
    cal: &holidays::Calendar,
) -> Result<Vec<Value>, String> {
    if rule.is_none() || !missing.is_empty() {
        return Ok(vec![pending_row(
            label, period, missing, family, pending, cal,
        )?]);
    }
    let rule = rule.expect("present rule");
    let payment = match &rule.payment {
        Payment::Mode(mode) => mode.as_str(),
        Payment::Separate(_) => "separate",
    };
    let mut result = vec![row(
        label,
        period,
        "lodgment",
        &rule,
        &[],
        Some(payment),
        cal,
    )?];
    if let Payment::Separate(payment) = &rule.payment {
        result.push(row(label, period, "payment", payment, &[], None, cal)?);
    }
    Ok(result)
}

fn pending_row(
    label: &str,
    period: &str,
    missing: &[String],
    family: &str,
    pending: &Pending<'_>,
    cal: &holidays::Calendar,
) -> Result<Value, String> {
    let meanings: Vec<_> = missing
        .iter()
        .map(|n| data()["fact_meanings"][n].as_str().expect("fact meaning"))
        .collect();
    let rule = Rule::new(
        family,
        format!(
            "The date depends on facts not supplied: {}.",
            meanings.join("; ")
        ),
        pending.source,
        None,
    );
    row(
        label,
        period,
        "lodgment",
        &rule,
        missing,
        Some(pending.payment),
        cal,
    )
}

pub(super) fn row(
    label: &str,
    period: &str,
    kind: &str,
    rule: &Rule,
    missing: &[String],
    payment: Option<&str>,
    cal: &holidays::Calendar,
) -> Result<Value, String> {
    let source = &data()["sources"][&rule.source];
    let mut needed = fact_lines(missing);
    let (due, holiday_roll) = schedule(kind, rule, &mut needed, cal)?;
    let status = if needed.is_empty() {
        "DETERMINED"
    } else {
        "UNKNOWN"
    };
    let mut result = json!({"obligation":label,"period":period,"kind":kind,"status":status,
        "due_date":due.map(|d|d.to_string()),"nominal_due_date":rule.nominal.map(|d|d.to_string()),"rule_id":rule.id,"summary":rule.summary,
        "source_url":source[0],"source_provision":source[1],"source_checked":CHECKED,"holiday_roll":holiday_roll,"missing":needed});
    if kind == "lodgment" {
        result["payment"] = json!(payment);
    }
    Ok(result)
}

fn fact_lines(missing: &[String]) -> Vec<String> {
    missing
        .iter()
        .map(|n| {
            format!(
                "{n}: {}",
                data()["fact_meanings"][n].as_str().expect("fact meaning")
            )
        })
        .collect()
}

fn schedule(
    kind: &str,
    rule: &Rule,
    needed: &mut Vec<String>,
    cal: &holidays::Calendar,
) -> Result<(Option<jiff::civil::Date>, Value), String> {
    if let Some(unresolved) = &rule.unresolved {
        needed.push(unresolved.clone());
        return Ok((None, Value::Null));
    }
    let Some(nominal) = rule.nominal.filter(|_| needed.is_empty()) else {
        return Ok((None, Value::Null));
    };
    let (mut due, skipped, question) = holidays::roll_day(nominal, cal)?;
    let holiday_roll = if !skipped.is_empty() || due.is_none() {
        json!({"skipped":skipped,"source_url":data()["holiday_source"]})
    } else {
        Value::Null
    };
    if let Some(question) = question {
        needed.push(question);
    } else if let Some(question) = concession_question(kind, &holiday_roll, nominal) {
        needed.push(question);
        due = None;
    }
    if !needed.is_empty() {
        due = None;
    }
    Ok((due, holiday_roll))
}

fn concession_question(
    kind: &str,
    holiday_roll: &Value,
    nominal: jiff::civil::Date,
) -> Option<String> {
    if kind == "concession"
        && holiday_roll["skipped"]
            .as_array()
            .is_some_and(|days| !days.is_empty())
    {
        Some(format!(
            "whether the next-business-day rule extends this concession: {nominal} is not a business day, and the ATO pages call it both a concessional due date and not a lodgment due date"
        ))
    } else {
        None
    }
}
