//! The payday super due date of one contribution as a saved derivation: the
//! input, one step per statutory question with the provision it applies, and
//! the result. `verify` rederives a saved derivation from its input, offline,
//! and names the first step whose saved working differs from the rederived one.
//!
//! Only the bundled business-day calendar is read, so a derivation never
//! depends on a file the person verifying it may not have. Item 4 alignment
//! needs the employee's earlier paydays, so a single contribution does not
//! apply it; the register check does.

use serde_json::{Value, json};

use crate::api;
use crate::calendar::{self, BusinessCalendar};
use crate::deadlines::{
    self, ContribLine, EXTENDED_20BD, OUT_OF_CYCLE, SKIP_DB, USUAL_7BD, pathway_words,
};
use crate::decimal::Decimal;
use crate::py::Date;

pub const FORMAT: &str = "payday-super-checker/derivation/1";
const CALCULATION: &str = "due-date";
const INPUT_KEYS: [&str; 5] = [
    "qe_day",
    "first_to_fund",
    "out_of_cycle",
    "next_standard_qe_day",
    "db_interest",
];

fn step(id: &str, authority: &str, working: String, value: Value) -> Value {
    json!({"id": id, "authority": authority, "working": working, "value": value})
}

fn iso(d: Option<Date>) -> Value {
    d.map_or(Value::Null, |d| Value::String(d.isoformat()))
}

fn line_of(input: &Value) -> Result<ContribLine, String> {
    let Some(object) = input.as_object() else {
        return Err("the JSON input must be an object".into());
    };
    if let Some(key) = object.keys().find(|k| !INPUT_KEYS.contains(&k.as_str())) {
        return Err(format!(
            "{key}: a derivation takes {} only, and reads only the bundled calendar so it \
             can be rederived offline",
            INPUT_KEYS.join(", ")
        ));
    }
    Ok(ContribLine {
        employee_id: String::new(),
        qe_day: api::date(input, "qe_day")?,
        sg_amount: Decimal::from_int(0),
        remitted: None,
        remitted_amount: None,
        received: None,
        first_to_fund: api::flag(input, "first_to_fund")?,
        out_of_cycle: api::flag(input, "out_of_cycle")?,
        next_standard_qe_day: api::optional_date(input, "next_standard_qe_day")?,
        db_interest: api::flag(input, "db_interest")?,
        row: 1,
        duplicate_note: String::new(),
        matched_amount: None,
        join_caveats: Vec::new(),
        returned_by_fund: false,
    })
}

/// Weekend days and confirmed holidays after `start`, up to and including `end`.
fn not_counted(cal: &BusinessCalendar, start: Date, end: Date) -> (usize, Vec<String>) {
    let holidays: Vec<String> = cal
        .holidays()
        .filter(|h| start < h.day && h.day <= end && !h.provisional && h.day.weekday() < 5)
        .map(|h| format!("{} {}", h.day.isoformat(), h.name))
        .collect();
    let days = (end.ordinal() - start.ordinal()).max(0);
    let weekends = (1..=days)
        .filter_map(|n| Date::from_ordinal(start.ordinal() + n))
        .filter(|d| d.weekday() >= 5)
        .count();
    (weekends, holidays)
}

fn bd(cal: &BusinessCalendar, d: Date, n: i64) -> Result<Date, String> {
    cal.add_business_days(d, n).map_err(api::failure)
}

/// The derivation of one contribution's due date.
pub fn derive(input: &Value) -> Result<Value, String> {
    let line = line_of(input)?;
    let cal = calendar::load_calendar(None).map_err(api::failure)?;
    let mut pairs = [(
        line.clone(),
        deadlines::compute_due(&line, &cal).map_err(api::failure)?,
    )];
    deadlines::annotate_missing_flag(&mut pairs);
    deadlines::annotate_calendar_risk(&mut pairs, &cal);
    let deadline = &pairs[0].1;
    let qe = line.qe_day.isoformat();
    let skipped = "Not worked out: a defined-benefit interest is not lateness tested.";
    let mut steps = vec![
        step(
            "payday-super",
            "SGAA 1992 s 18C, as amended by the Treasury Laws Amendment (Payday \
             Superannuation) Act 2025 (No 57 of 2025)",
            format!(
                "The QE day, on which the qualifying earnings were paid, is {qe}. Payday \
                 super applies to QE days from 1 July 2026."
            ),
            json!(qe),
        ),
        step(
            "defined-benefit",
            "SGAA 1992 s 18A(3)",
            if line.db_interest {
                "A defined-benefit interest: the notional contribution is treated as \
                 received on the QE day, so there is no due date to test."
                    .into()
            } else {
                "Not a defined-benefit interest.".into()
            },
            json!(line.db_interest),
        ),
    ];
    if deadline.pathway == SKIP_DB {
        for (id, authority) in [
            (
                "usual-period",
                "SGAA 1992 s 6(1) 'usual period'; s 18C(1)(c)",
            ),
            ("first-contribution", "SGAA 1992 s 18C(2) item 1"),
            ("out-of-cycle", "SGAA 1992 s 18C(2) item 2; F2026L00784 s 5"),
            ("item-4", "SGAA 1992 s 18C(2) item 4"),
            ("business-days", "SGAA 1992 s 6(1) 'business day'"),
            ("due-date", "SGAA 1992 s 18A(3)"),
        ] {
            steps.push(step(id, authority, skipped.into(), Value::Null));
        }
    } else {
        let usual = bd(&cal, line.qe_day, 7)?;
        let first = line
            .first_to_fund
            .then(|| bd(&cal, line.qe_day, 20))
            .transpose()?;
        let next = line.next_standard_qe_day;
        let cycle = match (line.out_of_cycle, next) {
            (true, Some(next)) => Some(bd(&cal, next, 7)?),
            _ => None,
        };
        let due = deadline.due.ok_or("the engine gave no due date")?;
        // The steps restate the engine's own arithmetic; refuse if they disagree.
        let latest = [Some(usual), first, cycle].into_iter().flatten().max();
        if latest != Some(due) {
            return Err(format!(
                "the derivation's latest period {} differs from the engine's due date {}",
                iso(latest),
                due.isoformat()
            ));
        }
        let count_start = if deadline.pathway == OUT_OF_CYCLE {
            next.ok_or("out-of-cycle deadline has no next standard QE day")?
        } else {
            line.qe_day
        };
        let (weekends, holidays) = not_counted(&cal, count_start, due);
        steps.extend([
            step(
                "usual-period",
                "SGAA 1992 s 6(1) 'usual period'; s 18C(1)(c)",
                format!("The 7th business day after the QE day {qe}."),
                iso(Some(usual)),
            ),
            step(
                "first-contribution",
                "SGAA 1992 s 18C(2) item 1",
                match first {
                    Some(_) => format!(
                        "The first eligible contribution to this fund: the 20th business \
                         day after the QE day {qe}."
                    ),
                    None => "Not claimed: not the first eligible contribution to this fund.".into(),
                },
                iso(first),
            ),
            step(
                "out-of-cycle",
                "SGAA 1992 s 18C(2) item 2; F2026L00784 s 5",
                match (cycle, next) {
                    (Some(_), Some(next)) => format!(
                        "Out-of-cycle earnings: the usual period of the next subsequent \
                         non-out-of-cycle QE payment on {}, its 7th business day.",
                        next.isoformat()
                    ),
                    _ => "Not claimed: the out-of-cycle flag is not set.".into(),
                },
                iso(cycle),
            ),
            step(
                "item-4",
                "SGAA 1992 s 18C(2) item 4",
                "Not applied: aligning to an earlier eligible contribution's later period \
                 needs the employee's earlier paydays and fund receipts, which the register \
                 check reads."
                    .into(),
                Value::Null,
            ),
            step(
                "business-days",
                "SGAA 1992 s 6(1) 'business day'",
                format!(
                    "Not counted for the {} period after {} up to {}: weekend days, \
                     {weekends}; weekday public holidays for the whole of a State or \
                     Territory, {}. A holiday not yet confirmed is counted as a business day.",
                    pathway_words(&deadline.pathway),
                    count_start.isoformat(),
                    due.isoformat(),
                    holidays.len()
                ),
                json!(holidays),
            ),
            step(
                "due-date",
                match deadline.pathway.as_str() {
                    EXTENDED_20BD => "SGAA 1992 s 18C(2) item 1",
                    OUT_OF_CYCLE => "SGAA 1992 s 18C(2) item 2; F2026L00784 s 5",
                    USUAL_7BD => "SGAA 1992 s 18C(1)(c); s 6(1)",
                    _ => "SGAA 1992 s 18C(2)",
                },
                format!(
                    "The latest period that applies, the {} period: the fund must receive \
                     the contribution by the end of this day.",
                    pathway_words(&deadline.pathway)
                ),
                iso(Some(due)),
            ),
        ]);
    }
    Ok(json!({
        "format": FORMAT,
        "engine": "payday-super-checker",
        "engine_version": crate::VERSION,
        "calculation": CALCULATION,
        "input": input,
        "steps": steps,
        "result": api::deadline_json(deadline),
    }))
}

/// The first field, in `fresh`'s key order then `saved`'s, whose values differ,
/// when both sides are objects; otherwise the values are compared whole.
fn first_field(saved: Option<&Value>, fresh: Option<&Value>) -> Option<String> {
    let (s, f) = (saved?.as_object()?, fresh?.as_object()?);
    f.keys()
        .chain(s.keys())
        .find(|k| s.get(*k) != f.get(*k))
        .cloned()
}

/// One side's value of `field`, or the whole value when no field is named.
fn pick<'a>(v: Option<&'a Value>, field: &Option<String>) -> Option<&'a Value> {
    match field {
        Some(k) => v.and_then(|v| v.get(k)),
        None => v,
    }
}

/// One difference. `missing` names the side that has no such step or field,
/// which a JSON null alone cannot distinguish from a present null.
fn difference(step: Value, id: Value, saved: Option<&Value>, fresh: Option<&Value>) -> Value {
    let field = first_field(saved, fresh);
    let (s, f) = (pick(saved, &field), pick(fresh, &field));
    let mut d = json!({"step": step, "id": id, "field": field, "saved": s, "rederived": f});
    match (s, f) {
        (None, Some(_)) => d["missing"] = json!("saved"),
        (Some(_), None) => d["missing"] = json!("rederived"),
        _ => {}
    }
    d
}

/// Where a saved derivation and its rederivation part: a step, then the result.
fn first_difference(saved: &Value, fresh: &Value) -> (Option<Value>, usize) {
    for key in ["engine", "calculation"] {
        let (s, f) = (saved.get(key), fresh.get(key));
        if s != f {
            let mut d = difference(Value::Null, json!("metadata"), s, f);
            d["field"] = json!(key);
            return (Some(d), 0);
        }
    }
    let none = Vec::new();
    let saved_steps = saved["steps"].as_array().unwrap_or(&none);
    let fresh_steps = fresh["steps"].as_array().unwrap_or(&none);
    for i in 0..saved_steps.len().max(fresh_steps.len()) {
        let (s, f) = (saved_steps.get(i), fresh_steps.get(i));
        if s != f {
            let id = f
                .and_then(|f| f.get("id"))
                .or_else(|| s.and_then(|s| s.get("id")))
                .cloned()
                .unwrap_or(Value::Null);
            return (Some(difference(json!(i + 1), id, s, f)), i + 1);
        }
    }
    let (s, f) = (saved.get("result"), fresh.get("result"));
    let differs = (s != f).then(|| difference(Value::Null, json!("result"), s, f));
    (differs, fresh_steps.len())
}

/// Rederive a saved derivation from its input and compare it step by step.
pub fn verify(saved: &Value) -> Result<Value, String> {
    if saved.get("format").and_then(Value::as_str) != Some(FORMAT) {
        return Err(format!("not a {FORMAT} document"));
    }
    let input = saved.get("input").ok_or("input is missing")?;
    let fresh = derive(input)?;
    let (differs, checked) = first_difference(saved, &fresh);
    Ok(json!({
        "verified": differs.is_none(),
        "differs_at": differs,
        "steps_checked": checked,
        "saved_engine_version": saved.get("engine_version"),
        "engine_version": crate::VERSION,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(d: &Value, id: &str) -> Value {
        d["steps"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["id"] == id)
            .unwrap()["value"]
            .clone()
    }

    #[test]
    fn the_ato_first_payday_example_is_derived_step_by_step() {
        // The ATO's worked example: a first payday of 9 July 2026, due 7 August.
        let d = derive(&json!({"qe_day": "2026-07-09", "first_to_fund": true})).unwrap();
        assert_eq!(value(&d, "first-contribution"), "2026-08-07");
        assert_eq!(value(&d, "usual-period"), "2026-07-20");
        assert_eq!(d["result"]["due"], value(&d, "due-date"));
        assert_eq!(d["result"]["pathway"], EXTENDED_20BD);
        for s in d["steps"].as_array().unwrap() {
            let authority = s["authority"].as_str().unwrap();
            assert!(authority.starts_with("SGAA 1992 s "), "{authority}");
        }
        assert_eq!(verify(&d).unwrap()["verified"], true);
    }

    #[test]
    fn excluded_days_belong_to_the_selected_deadline_window() {
        for (next, first, due, pathway, dates, weekends) in [
            ("2027-01-04", false, "2027-01-13", OUT_OF_CYCLE, vec![], 2),
            (
                "2027-01-04",
                true,
                "2027-01-25",
                EXTENDED_20BD,
                vec!["2026-12-25", "2026-12-28", "2027-01-01"],
                10,
            ),
            (
                "2027-01-25",
                true,
                "2027-02-04",
                OUT_OF_CYCLE,
                vec!["2027-01-26"],
                2,
            ),
            ("2027-01-14", true, "2027-01-25", OUT_OF_CYCLE, vec![], 4),
        ] {
            let d = derive(&json!({"qe_day": "2026-12-23", "out_of_cycle": true,
                "next_standard_qe_day": next, "first_to_fund": first}))
            .unwrap();
            assert_eq!(d["result"]["due"], due);
            assert_eq!(d["result"]["pathway"], pathway);
            let step = d["steps"]
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s["id"] == "business-days")
                .unwrap();
            let actual: Vec<_> = step["value"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().split_once(' ').unwrap().0)
                .collect();
            assert_eq!(actual, dates);
            let start = if pathway == OUT_OF_CYCLE {
                next
            } else {
                "2026-12-23"
            };
            let working = step["working"].as_str().unwrap();
            assert!(
                working.contains(&format!("after {start} up to {due}")),
                "{working}"
            );
            assert!(
                working.contains(&format!("weekend days, {weekends};")),
                "{working}"
            );
        }
    }

    #[test]
    fn the_result_is_the_engines_compute_due() {
        for input in [
            json!({"qe_day": "2026-07-09"}),
            json!({"qe_day": "2026-12-23"}),
            json!({"qe_day": "2026-09-30", "out_of_cycle": true, "next_standard_qe_day": "2026-10-08"}),
            json!({"qe_day": "2026-07-09", "db_interest": true}),
        ] {
            let d = derive(&input).unwrap();
            let mut line = input.clone();
            line["employee_id"] = json!("");
            line["sg_amount"] = json!("0");
            line["row"] = json!(1);
            let engine = api::evaluate("compute-due", &json!({"line": line})).unwrap();
            assert_eq!(d["result"]["due"], engine["due"], "{input}");
            assert_eq!(d["result"]["pathway"], engine["pathway"], "{input}");
            assert_eq!(d["result"]["notes"], engine["notes"], "{input}");
        }
    }

    #[test]
    fn verify_names_the_step_that_differs() {
        let mut d = derive(&json!({"qe_day": "2026-12-23"})).unwrap();
        d["steps"][2]["value"] = json!("2026-12-31");
        let v = verify(&d).unwrap();
        assert_eq!(v["verified"], false);
        assert_eq!(v["differs_at"]["id"], "usual-period");
        assert_eq!(v["differs_at"]["step"], 3);
        let mut d = derive(&json!({"qe_day": "2026-12-23"})).unwrap();
        d["result"]["due"] = json!("2027-01-15");
        assert_eq!(verify(&d).unwrap()["differs_at"]["field"], "due");
    }

    #[test]
    fn an_absent_step_or_field_is_named_as_missing() {
        let input = json!({"qe_day": "2026-12-23"});
        let mut d = derive(&input).unwrap();
        d["steps"].as_array_mut().unwrap().push(Value::Null);
        let at = &verify(&d).unwrap()["differs_at"];
        assert_eq!(
            at,
            &json!({"step": 9, "id": null, "field": null, "saved": null,
                "rederived": null, "missing": "rederived"})
        );
        let mut d = derive(&input).unwrap();
        d["steps"][0]["note"] = Value::Null;
        let at = &verify(&d).unwrap()["differs_at"];
        assert_eq!(
            (&at["field"], &at["missing"]),
            (&json!("note"), &json!("rederived"))
        );
        let mut d = derive(&input).unwrap();
        d.as_object_mut().unwrap().remove("engine");
        let at = &verify(&d).unwrap()["differs_at"];
        assert_eq!(
            (&at["field"], &at["missing"]),
            (&json!("engine"), &json!("saved"))
        );
        assert_eq!(verify(&d).unwrap()["steps_checked"], 0);
    }

    #[test]
    fn holidays_not_counted_are_named() {
        let d = derive(&json!({"qe_day": "2026-12-23"})).unwrap();
        let holidays = value(&d, "business-days");
        let names: Vec<&str> = holidays
            .as_array()
            .unwrap()
            .iter()
            .map(|h| h.as_str().unwrap())
            .collect();
        assert!(
            names.iter().any(|h| h.starts_with("2026-12-25 ")),
            "{names:?}"
        );
    }

    #[test]
    fn foreign_input_and_documents_are_refused() {
        assert!(
            derive(&json!({"qe_day": "2026-07-09", "calendar": {}}))
                .unwrap_err()
                .starts_with("calendar: ")
        );
        assert!(derive(&json!({"qe_day": "2026-06-30"})).is_err());
        assert_eq!(
            verify(&json!({})).unwrap_err(),
            "not a payday-super-checker/derivation/1 document"
        );
    }
}
