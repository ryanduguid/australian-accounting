// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

use div7a_loan_review::{
    api,
    decimal::{Calc, Decimal, Signal},
    facts::optional_years,
    gate::{
        DEFAULT_MAXIMUM_TERM, GateFacts, GateResult, REQUIRED_SECURITY_COVERAGE,
        SECURED_MAXIMUM_TERM, complying_loan_gate,
    },
    money::d,
    myr::{GateSummary, MyrFacts, MyrResult, minimum_yearly_repayment, statutory_remaining_term},
    raw::Raw,
    years::{YearOfIncome, parse_year},
};
use serde_json::{Value, json};

const DISCLAIMER: &str = "Experimental review aid. Not a Division 7A determination, an ATO assessment or professional advice. The operator supplies the statutory facts. Verify the result against the compiled Act and obtain human review before acting.";

fn metadata(value: &mut Value) {
    for (key, v) in json!({"ok":true,"engine":"div7a-loan-review",
        "engine_version":div7a_loan_review::VERSION,
        "law_content_date":div7a_loan_review::LAW_CONTENT_DATE,
        "law_compilation":div7a_loan_review::LAW_COMPILATION,"disclaimer":DISCLAIMER})
    .as_object()
    .unwrap()
    {
        value
            .as_object_mut()
            .unwrap()
            .insert(key.clone(), v.clone());
    }
}
fn selected(value: &Value, keys: &[&str]) -> Value {
    Value::Object(
        keys.iter()
            .map(|k| ((*k).into(), value[*k].clone()))
            .collect(),
    )
}
fn summary(input: &Value) -> Result<bool, String> {
    match input["response_detail"].as_str().unwrap_or("summary") {
        "summary" => Ok(true),
        "full" => Ok(false),
        _ => Err("response_detail must be 'summary' or 'full'".into()),
    }
}
pub fn rate(input: &Value) -> Result<Value, String> {
    let concise = summary(input)?;
    let mut payload = api::evaluate("benchmark-rate", input)?;
    metadata(&mut payload);
    if !concise {
        return Ok(payload);
    }
    let mut result = selected(
        &payload,
        &[
            "ok",
            "engine",
            "engine_version",
            "law_content_date",
            "law_compilation",
            "disclaimer",
            "year_of_income",
            "verdict",
            "benchmark_rate",
            "reason",
        ],
    );
    result["response_detail"] = json!("summary");
    result["source"] = json!({"verify_at":payload["provenance"]["verify_at"],
        "rate_table_uris":payload["manifest"]["rate_table_uris"]});
    Ok(result)
}
// The engine's public minimum_yearly_repayment takes no loan term, so it checks
// no remaining-term limit; repayment_within, which does, is crate-private.
// These refusals copy the Python server's on main
// (aus_accounting_mcp/adapters/div7a.py: _remaining_term_refusal,
// _term_left_refusal and _refuse), which copy repayment_within's conditions,
// codes and wording. A code the engine gave is not repeated.

/// The s 109N(3) maximum term the security facts give, before any s 109N(3A)
/// to (3D) reduction; None where they do not establish one. Not the gate's
/// maximum_term_years_allowed, which is 7 years whilst the security is unknown.
fn maximum_term(facts: &GateFacts) -> Calc<Option<Decimal>> {
    let secured = facts.secured_by_registered_mortgage_over_real_property;
    Ok(match (secured, &facts.security_coverage_at_first_made) {
        (Some(false), _) => Some(d(DEFAULT_MAXIMUM_TERM)),
        (Some(true), Some(coverage)) if !coverage.lt(&d(REQUIRED_SECURITY_COVERAGE))? => {
            Some(d(SECURED_MAXIMUM_TERM))
        }
        (Some(true), Some(_)) => Some(d(DEFAULT_MAXIMUM_TERM)),
        _ => None,
    })
}

const ABOVE_MAXIMUM_TERM: &str = "-year maximum term this row's security facts give under s 109N(3) before any s 109N(3A) to (3D) reduction, which can only shorten it, yet the gate found the loan on s 109N terms. Both cannot hold: each constituent loan is one s 109N saves (s 109E(3)(b)), so its term is within its maximum term (s 109N(1)(c)); the constituent loans share one maximum term (s 109E(3)(c)); and the remaining term is the longest constituent term less the years since the loan year (s 109E(6)). Correct the remaining term or the security facts. Enter a loan secured and extended under s 109E(3A) as the new amalgamated loan s 109E(3B) creates, with its 25-year security facts and, in maximum_term_years, its agreed term counted from when the loan was first made (at most 25 years, s 109E(3A)(e)) less the time it had already run before the extension, which is how the ATO works the term of such a loan. No MYR figure is emitted.";
const ABOVE_LOAN_TERM: &str = "-year term of the loan in maximum_term_years. The remaining term is the longest constituent-loan term less the years from the end of the loan year to the end of the previous year of income (s 109E(6)), so it cannot be longer than that term, and a longer one gives a lower minimum yearly repayment than the Act requires. Correct remaining_term_years or, where a constituent loan of this amalgamated loan has a longer term, enter that longest term in maximum_term_years. No MYR figure is emitted.";

/// A refusal of a remaining term that cannot hold on a loan the gate passed.
fn remaining_term_refusal(
    gate: &GateResult,
    facts: &GateFacts,
    supplied: Option<&Decimal>,
    year: YearOfIncome,
) -> Calc<Option<(&'static str, String)>> {
    let Some(supplied) = supplied.filter(|_| gate.verdict == "COMPLYING") else {
        return Ok(None);
    };
    let used = statutory_remaining_term(supplied)?;
    let head = format!(
        "remaining_term_years is {supplied}, a remaining term of {used} under s 109E(6), longer than the "
    );
    if let Some(maximum) = maximum_term(facts)?
        && used.gt(&maximum)?
    {
        let reason = format!("{head}{maximum}{ABOVE_MAXIMUM_TERM}");
        return Ok(Some(("REFUSED_REMAINING_TERM_ABOVE_MAXIMUM_TERM", reason)));
    }
    let Some(term) = &facts.maximum_term_years else {
        return Ok(None);
    };
    if used.gt(&statutory_remaining_term(term)?)? {
        let reason = format!("{head}{term}{ABOVE_LOAN_TERM}");
        return Ok(Some(("REFUSED_REMAINING_TERM_ABOVE_LOAN_TERM", reason)));
    }
    match facts.year_loan_made {
        Some(made) if made < year => term_left_refusal(supplied, &used, term, made, year),
        _ => Ok(None),
    }
}

fn term_left_refusal(
    supplied: &Decimal,
    used: &Decimal,
    term: &Decimal,
    made: YearOfIncome,
    year: YearOfIncome,
) -> Calc<Option<(&'static str, String)>> {
    // s 109E(6)(b): the whole years from the end of the loan year to the end of
    // the year before this one, and the loan's term less them, rounded up.
    let elapsed = year.start_year - made.start_year - 1;
    let left = statutory_remaining_term(term)?.sub(&d(&elapsed.to_string()))?;
    let zero = d("0");
    if used.le(&zero)? || used.le(&left)? {
        return Ok(None);
    }
    let previous = YearOfIncome {
        start_year: year.start_year - 1,
    };
    let years = if elapsed == 1 { "year" } else { "years" };
    let limit = if left.gt(&zero)? {
        format!("so the remaining term is at most {left}")
    } else {
        format!(
            "which leaves a remaining term of {left}, so the loan should have been repaid in full by the end of the previous year of income"
        )
    };
    let (from, to) = (made.label(), previous.label());
    Ok(Some((
        "REFUSED_REMAINING_TERM_ABOVE_TERM_LESS_YEARS_ELAPSED",
        format!(
            "remaining_term_years is {supplied}, a remaining term of {used} under s 109E(6), longer than the {term}-year term in maximum_term_years less the years since the loan year. s 109E(6) makes the remaining term the difference between (a) 'the number of years in the longest term of any of the constituent loans that the amalgamated loan takes account of' and (b) 'the number of years between the end of the private company's year of income in which the loan was made and the end of the private company's year of income before the year of income for which the minimum yearly repayment is being worked out', rounded up if not already a whole number. From the end of {from} to the end of {to} is {elapsed} {years}, {limit}; a longer remaining term gives a lower minimum yearly repayment than the Act requires. Correct remaining_term_years, year_loan_made or maximum_term_years. A loan secured and extended under s 109E(3A) is the new amalgamated loan s 109E(3B) treats as made just before the start of the later year of income, so its year_loan_made is the year before that later year, and its maximum_term_years, as the ATO works it, is its agreed term counted from when the loan was first made (at most 25 years, s 109E(3A)(e)) less the time it had already run before the extension. No MYR figure is emitted."
        ),
    )))
}

/// A refusal joins the engine's refusals and replaces its unknowns.
fn refuse(myr: &mut MyrResult, code: &str, reason: String) {
    if myr.reason_codes.iter().any(|given| given == code) {
        return;
    }
    if myr.verdict != "REFUSED" {
        myr.reasons.clear();
        myr.reason_codes.clear();
    }
    myr.verdict = "REFUSED";
    myr.myr_required = None;
    myr.shortfall = None;
    myr.experimental_deemed_dividend_exposure = None;
    myr.reasons.push(reason);
    myr.reason_codes.push(code.to_owned());
    // The engine's refused result keeps only its s 109E(1) and (5) lines.
    myr.statutory_trace.truncate(2);
}

pub fn loan(input: &Value) -> Result<Value, String> {
    let concise = summary(input)?;
    let mut mapping = input.clone();
    if mapping.get("loan_id").is_none() {
        mapping["loan_id"] = json!("mcp-div7a-1");
    }
    let raw = Raw::from_value(&mapping);
    let facts = GateFacts::from_mapping(&raw, "MCP review_div7a_loan")?;
    let gate = complying_loan_gate(&facts, None, None).map_err(|e| e.to_string())?;
    let year = parse_year(&Raw::from_value(&input["year_of_income"]), "year of income")?;
    let term = optional_years(
        raw.get("remaining_term_years"),
        "MCP review_div7a_loan remaining_term_years",
    )?;
    let mut myr = minimum_yearly_repayment(
        &MyrFacts {
            loan_id: facts.loan_id.clone(),
            year_of_income: Some(year),
            year_loan_made: facts.year_loan_made,
            gate_result: Some(GateSummary::from(&gate)),
            remaining_term_years: term.clone(),
            amalgamated_loan_unpaid_at_end_of_previous_year: super::money::optional(
                input.get("amalgamated_loan_unpaid_at_end_of_previous_year"),
                "amalgamated_loan_unpaid_at_end_of_previous_year",
            )?,
            payments_applied_during_the_year: super::money::optional(
                input.get("payments_applied_during_the_year"),
                "payments_applied_during_the_year",
            )?,
        },
        None,
        None,
    )
    .map_err(|e| e.to_string())?;
    let signal = |e: Signal| format!("decimal.{}", e.name());
    if let Some((code, reason)) =
        remaining_term_refusal(&gate, &facts, term.as_ref(), year).map_err(signal)?
    {
        refuse(&mut myr, code, reason);
    }
    let g = gate.to_json();
    let m = myr.to_json().map_err(|e| format!("decimal.{}", e.name()))?;
    let mut payload = json!({"gate":g,"minimum_yearly_repayment":m});
    metadata(&mut payload);
    Ok(if concise {
        loan_summary(&payload)
    } else {
        payload
    })
}

const LOAN_SUMMARY: [&str; 6] = [
    "ok",
    "engine",
    "engine_version",
    "law_content_date",
    "law_compilation",
    "disclaimer",
];
const GATE_SUMMARY: [&str; 8] = [
    "verdict",
    "loan_id",
    "benchmark_year_used",
    "benchmark_rate",
    "maximum_term_years_allowed",
    "reasons",
    "reason_codes",
    "caveats",
];
const REPAYMENT_SUMMARY: [&str; 15] = [
    "verdict",
    "loan_id",
    "year_of_income",
    "gate_verdict",
    "benchmark_rate",
    "amalgamated_loan_unpaid_at_end_of_previous_year",
    "remaining_term_years_used",
    "myr_required",
    "payments_applied",
    "shortfall",
    "experimental_deemed_dividend_exposure",
    "rounding",
    "reasons",
    "reason_codes",
    "caveats",
];

/// The `summary` detail of a full `review_div7a_loan` payload.
fn loan_summary(payload: &Value) -> Value {
    let (g, m) = (&payload["gate"], &payload["minimum_yearly_repayment"]);
    let mut result = selected(payload, &LOAN_SUMMARY);
    result["response_detail"] = json!("summary");
    let provenance = if g["benchmark_provenance"].is_null() {
        &m["benchmark_provenance"]
    } else {
        &g["benchmark_provenance"]
    };
    let uris = if g["manifest"]["rate_table_uris"]
        .as_array()
        .is_none_or(|v| v.is_empty())
    {
        &m["manifest"]["rate_table_uris"]
    } else {
        &g["manifest"]["rate_table_uris"]
    };
    result["source"] = json!({"verify_at":provenance.get("verify_at"),"rate_table_uris":uris});
    result["gate"] = selected(g, &GATE_SUMMARY);
    result["minimum_yearly_repayment"] = selected(m, &REPAYMENT_SUMMARY);
    result
}
