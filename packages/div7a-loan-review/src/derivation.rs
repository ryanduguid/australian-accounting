//! The s 109E minimum yearly repayment as a saved derivation: the input, one
//! step per statutory question with the provision it applies, and the result.
//! `verify` rederives a saved derivation from its input, offline, and names the
//! first step whose saved working differs from the rederived one.
//!
//! Only the compiled-in benchmark table is read, so a derivation never depends
//! on a file the person verifying it may not have.

use serde_json::{Map, Value, json};

use crate::Failure;
use crate::api;
use crate::decimal::{Calc, Decimal};
use crate::money::{ROUNDING, cents_str};
use crate::myr::{MyrFacts, MyrResult, minimum_yearly_repayment};

pub const FORMAT: &str = "div7a-loan-review/derivation/1";
const CALCULATION: &str = "minimum-yearly-repayment";

fn step(id: &str, authority: &str, working: String, value: Value) -> Value {
    json!({"id": id, "authority": authority, "working": working, "value": value})
}

fn money(v: &Option<Decimal>) -> Calc<Value> {
    Ok(match v {
        Some(v) => json!(cents_str(v)?),
        None => Value::Null,
    })
}

fn not_worked_out(result: &MyrResult) -> String {
    format!(
        "Not worked out: the outcome is {} (see the outcome step).",
        result.verdict
    )
}

fn steps(facts: &MyrFacts, result: &MyrResult) -> Calc<Vec<Value>> {
    let computed = result.myr_required.is_some();
    let made = facts.year_loan_made.map(|y| y.label());
    let gate = facts.gate_result.as_ref();
    let rate = result.benchmark.as_ref();
    let rate_text = rate.and_then(|r| r.rate_text());
    let entry = rate.and_then(|r| r.entry.as_ref());
    let principal = money(&result.amalgamated_loan_unpaid_at_end_of_previous_year)?;
    let term = result
        .remaining_term_years_used
        .as_ref()
        .map(Decimal::to_sci_string);
    let required = money(&result.myr_required)?;
    let applied = money(&result.payments_applied)?;
    let shortfall = money(&result.shortfall)?;
    let text = |v: &Value| v.as_str().unwrap_or("").to_string();
    Ok(vec![
        step(
            "year-of-income",
            "ITAA 1936 s 109E(1)",
            "The year of income being reviewed: the dividend, if any, is taken to be paid \
             at its end."
                .into(),
            json!(facts.year_of_income.map(|y| y.label())),
        ),
        step(
            "year-loan-made",
            "ITAA 1936 s 109E(1)(a); s 109P",
            match &made {
                Some(made) => format!(
                    "The amalgamated loan was made in {made}. s 109E(1)(a) needs an earlier \
                     year of income; s 109P puts the year of making outside s 109D."
                ),
                None => "Not supplied, so an earlier year of making is not established.".into(),
            },
            json!(made),
        ),
        step(
            "complying-terms",
            "ITAA 1936 s 109N(1); s 109E(3)(b)",
            match gate {
                Some(g) => format!(
                    "The s 109N gate verdict supplied, read against the {} benchmark. Only a \
                     loan on complying terms forms part of an amalgamated loan.",
                    if g.benchmark_year_used.is_empty() {
                        "unstated"
                    } else {
                        g.benchmark_year_used.as_str()
                    }
                ),
                None => "No s 109N gate result was supplied.".into(),
            },
            json!(gate.map(|g| g.verdict.as_str())),
        ),
        step(
            "benchmark-rate",
            "ITAA 1936 s 109N(2)",
            match (entry, &rate_text) {
                (Some(e), Some(_)) => format!(
                    "Current year's benchmark interest rate: RBA table {} series {}, {} \
                     figure, the last published before the year of income began; table \
                     reviewed until {}.",
                    e.rba_table,
                    e.rba_series,
                    e.rba_month,
                    rate.map_or("", |r| r.table_reviewed_until.as_str())
                ),
                _ => rate
                    .and_then(|r| r.reason.clone())
                    .filter(|r| !r.is_empty())
                    .unwrap_or_else(|| "No reviewed benchmark rate for this year.".into()),
            },
            json!(rate_text),
        ),
        step(
            "opening-balance",
            "ITAA 1936 s 109E(6); s 109E(3)",
            "Amount of the loan not repaid by the end of the previous year of income, as \
             the operator asserts it. This engine does not form the amalgamated loan from \
             its constituent loans."
                .into(),
            principal.clone(),
        ),
        step(
            "remaining-term",
            "ITAA 1936 s 109E(6), closing words",
            match (&facts.remaining_term_years, &term) {
                (Some(supplied), Some(used)) => format!(
                    "Remaining term supplied as {supplied} years, rounded up to the next \
                     whole number: {used}."
                ),
                _ => "Not supplied.".into(),
            },
            json!(term),
        ),
        step(
            "minimum-yearly-repayment",
            "ITAA 1936 s 109E(5) and (6)",
            if computed {
                let (p, r, n) = (
                    text(&principal),
                    rate_text.clone().unwrap_or_default(),
                    term.clone().unwrap_or_default(),
                );
                format!(
                    "MYR = (P x r) / (1 - (1 / (1 + r)) ^ n) = ({p} x {r}) / (1 - (1 / (1 + \
                     {r})) ^ {n}) = {}, worked at 60 significant digits and quantised once \
                     to cents, {ROUNDING}; the Act prescribes no rounding.",
                    text(&required)
                )
            } else {
                not_worked_out(result)
            },
            required.clone(),
        ),
        step(
            "payments-applied",
            "ITAA 1936 s 109R (not applied)",
            "Payments applied during the year, as the operator asserts them. This engine \
             does not decide which payments s 109R lets count."
                .into(),
            applied.clone(),
        ),
        step(
            "shortfall",
            "ITAA 1936 s 109E(1)(c)",
            if computed {
                format!(
                    "Shortfall = max(0, {} - {}) = {}. Any shortfall is a review aid, not a \
                     s 109E(1) determination: s 109Y and s 109Q are not modelled.",
                    text(&required),
                    text(&applied),
                    text(&shortfall)
                )
            } else {
                not_worked_out(result)
            },
            shortfall,
        ),
        step(
            "outcome",
            "ITAA 1936 s 109E",
            if result.reasons.is_empty() {
                match result.verdict {
                    "MYR_MET" => "Payments meet the minimum yearly repayment.".into(),
                    _ => "Payments fall short of the minimum yearly repayment.".into(),
                }
            } else {
                format!(
                    "{}: {}",
                    result.reason_codes.join(", "),
                    result.reasons.join(" ")
                )
            },
            json!(result.verdict),
        ),
    ])
}

fn failure(f: Failure) -> String {
    f.to_string()
}

fn signal(s: crate::decimal::Signal) -> String {
    format!("decimal.{}", s.name())
}

/// The derivation of one minimum yearly repayment. `input` is the
/// `minimum-yearly-repayment` command's input without a rate table or override.
pub fn derive(input: &Value) -> Result<Value, String> {
    if !input.is_object() {
        return Err("the JSON input must be an object".into());
    }
    for key in ["table", "override"] {
        if input.get(key).is_some_and(|v| !v.is_null()) {
            return Err(format!(
                "{key}: a derivation reads only the compiled-in benchmark table, so it can \
                 be rederived offline"
            ));
        }
    }
    let facts = api::myr_facts(input)?;
    let result = minimum_yearly_repayment(&facts, None, None).map_err(failure)?;
    Ok(json!({
        "format": FORMAT,
        "engine": "div7a-loan-review",
        "engine_version": crate::VERSION,
        "calculation": CALCULATION,
        "input": input,
        "steps": steps(&facts, &result).map_err(signal)?,
        "result": result.to_json().map_err(signal)?,
    }))
}

/// The first field, in `fresh`'s key order then `saved`'s, whose values differ.
fn first_field(saved: &Value, fresh: &Value) -> Option<String> {
    let empty = Map::new();
    let (s, f) = (
        saved.as_object().unwrap_or(&empty),
        fresh.as_object().unwrap_or(&empty),
    );
    f.keys()
        .chain(s.keys())
        .find(|k| s.get(*k) != f.get(*k))
        .cloned()
}

/// Where a saved derivation and its rederivation part: a step, then the result.
fn first_difference(saved: &Value, fresh: &Value) -> (Option<Value>, usize) {
    for key in ["engine", "calculation"] {
        if saved.get(key) != fresh.get(key) {
            return (
                Some(json!({"step": null, "id": "metadata", "field": key,
                "saved": saved.get(key), "rederived": fresh.get(key)})),
                0,
            );
        }
    }
    let none = Vec::new();
    let saved_steps = saved["steps"].as_array().unwrap_or(&none);
    let fresh_steps = fresh["steps"].as_array().unwrap_or(&none);
    for i in 0..saved_steps.len().max(fresh_steps.len()) {
        let (saved_step, fresh_step) = (saved_steps.get(i), fresh_steps.get(i));
        if saved_step != fresh_step {
            let (s, f) = (
                saved_step.unwrap_or(&Value::Null),
                fresh_step.unwrap_or(&Value::Null),
            );
            let field = first_field(s, f);
            let pick = |v: &Value| field.as_ref().map_or(v.clone(), |k| v[k].clone());
            return (
                Some(json!({
                    "step": i + 1,
                    "id": f.get("id").or_else(|| s.get("id")),
                    "field": field,
                    "saved": pick(s),
                    "rederived": pick(f),
                })),
                i + 1,
            );
        }
    }
    let (s, f) = (&saved["result"], &fresh["result"]);
    let difference = (s != f).then(|| {
        let field = first_field(s, f);
        let pick = |v: &Value| field.as_ref().map_or(v.clone(), |k| v[k].clone());
        json!({
            "step": null,
            "id": "result",
            "field": field,
            "saved": pick(s),
            "rederived": pick(f),
        })
    });
    (difference, fresh_steps.len())
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

    fn met() -> Value {
        json!({"facts": {
            "year_of_income": "2026-27", "year_loan_made": "2023-24",
            "amalgamated_loan_unpaid_at_end_of_previous_year": "100000.00",
            "remaining_term_years": "5", "payments_applied_during_the_year": "25556.00",
            "loan_id": "F1-MYR-MET",
            "gate_result": {"verdict": "COMPLYING", "benchmark_year_used": "2023-24",
                "reasons": [], "caveats": []}}})
    }

    #[test]
    fn the_hand_worked_repayment_is_derived_step_by_step() {
        let d = derive(&met()).unwrap();
        let value = |id: &str| {
            d["steps"]
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s["id"] == id)
                .unwrap()["value"]
                .clone()
        };
        assert_eq!(value("benchmark-rate"), "0.0877");
        assert_eq!(value("remaining-term"), "5");
        assert_eq!(value("minimum-yearly-repayment"), "25556.00");
        assert_eq!(value("shortfall"), "0.00");
        assert_eq!(value("outcome"), "MYR_MET");
        assert_eq!(d["result"]["myr_required"], "25556.00");
        for s in d["steps"].as_array().unwrap() {
            assert!(
                s["authority"]
                    .as_str()
                    .unwrap()
                    .starts_with("ITAA 1936 s 109")
            );
        }
        // The result is the minimum-yearly-repayment command's, unchanged.
        assert_eq!(
            d["result"],
            api::evaluate("minimum-yearly-repayment", &met()).unwrap()
        );
    }

    #[test]
    fn verify_names_the_step_that_differs() {
        let mut d = derive(&met()).unwrap();
        assert_eq!(verify(&d).unwrap()["verified"], true);
        d["steps"][3]["value"] = json!("0.0837");
        let v = verify(&d).unwrap();
        assert_eq!(v["verified"], false);
        assert_eq!(v["differs_at"]["id"], "benchmark-rate");
        assert_eq!(v["differs_at"]["step"], 4);
        assert_eq!(v["differs_at"]["field"], "value");
        assert_eq!(v["differs_at"]["rederived"], "0.0877");
    }

    #[test]
    fn verify_names_a_result_edit_and_a_missing_step() {
        let mut d = derive(&met()).unwrap();
        d["result"]["shortfall"] = json!("1.00");
        let v = verify(&d).unwrap();
        assert_eq!(v["differs_at"]["id"], "result");
        assert_eq!(v["differs_at"]["field"], "shortfall");
        let mut d = derive(&met()).unwrap();
        d["steps"].as_array_mut().unwrap().pop();
        assert_eq!(verify(&d).unwrap()["differs_at"]["id"], "outcome");
    }

    #[test]
    fn a_refusal_is_derived_up_to_its_outcome() {
        let mut input = met();
        input["facts"]["year_loan_made"] = json!("2026-27");
        input["facts"]["gate_result"]["benchmark_year_used"] = json!("2026-27");
        let d = derive(&input).unwrap();
        let steps = d["steps"].as_array().unwrap();
        let outcome = steps.last().unwrap();
        assert_eq!(outcome["value"], "REFUSED");
        assert!(
            outcome["working"]
                .as_str()
                .unwrap()
                .starts_with("REFUSED_YEAR_IS_YEAR_OF_LOAN: ")
        );
        assert_eq!(steps[6]["value"], Value::Null);
        assert_eq!(verify(&d).unwrap()["verified"], true);
    }

    #[test]
    fn files_and_foreign_documents_are_refused() {
        let mut input = met();
        input["override"] = json!("rates.json");
        assert!(derive(&input).unwrap_err().starts_with("override: "));
        assert_eq!(
            verify(&json!({"format": "x"})).unwrap_err(),
            "not a div7a-loan-review/derivation/1 document"
        );
    }
}
