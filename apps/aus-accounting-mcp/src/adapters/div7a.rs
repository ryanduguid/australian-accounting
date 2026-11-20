// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

use div7a_loan_review::{
    api,
    facts::optional_years,
    gate::{GateFacts, GateResult, complying_loan_gate, maximum_term},
    myr::{GateSummary, MyrFacts, repayment_within},
    raw::Raw,
    years::parse_year,
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
const LOAN_FIELDS: &[&str] = &[
    "ok",
    "engine",
    "engine_version",
    "law_content_date",
    "law_compilation",
    "disclaimer",
];
const GATE_FIELDS: &[&str] = &[
    "verdict",
    "loan_id",
    "benchmark_year_used",
    "benchmark_rate",
    "maximum_term_years_allowed",
    "reasons",
    "reason_codes",
    "caveats",
];
const REPAYMENT_FIELDS: &[&str] = &[
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

pub fn loan(input: &Value) -> Result<Value, String> {
    let concise = summary(input)?;
    let (gate, repayment) = loan_parts(input)?;
    let mut payload = json!({"gate":gate,"minimum_yearly_repayment":repayment});
    metadata(&mut payload);
    if concise {
        Ok(loan_summary(&gate, &repayment, &payload))
    } else {
        Ok(payload)
    }
}

fn loan_parts(input: &Value) -> Result<(Value, Value), String> {
    let mut mapping = input.clone();
    if mapping.get("loan_id").is_none() {
        mapping["loan_id"] = json!("mcp-div7a-1");
    }
    let raw = Raw::from_value(&mapping);
    let facts = GateFacts::from_mapping(&raw, "MCP review_div7a_loan")?;
    let gate = complying_loan_gate(&facts, None, None).map_err(|e| e.to_string())?;
    let (maximum, _, _) = maximum_term(&facts).map_err(|e| format!("decimal.{}", e.name()))?;
    let myr = repayment_within(
        &repayment_facts(input, &raw, &facts, &gate)?,
        maximum.as_ref(),
        facts.maximum_term_years.as_ref(),
        None,
        None,
    )
    .map_err(|e| e.to_string())?;
    Ok((
        gate.to_json(),
        myr.to_json().map_err(|e| format!("decimal.{}", e.name()))?,
    ))
}

fn repayment_facts(
    input: &Value,
    raw: &Raw,
    facts: &GateFacts,
    gate: &GateResult,
) -> Result<MyrFacts, String> {
    let year = parse_year(&Raw::from_value(&input["year_of_income"]), "year of income")?;
    Ok(MyrFacts {
        loan_id: facts.loan_id.clone(),
        year_of_income: Some(year),
        year_loan_made: facts.year_loan_made,
        gate_result: Some(GateSummary::from(gate)),
        remaining_term_years: optional_years(
            raw.get("remaining_term_years"),
            "MCP review_div7a_loan remaining_term_years",
        )?,
        amalgamated_loan_unpaid_at_end_of_previous_year: super::money::optional(
            input.get("amalgamated_loan_unpaid_at_end_of_previous_year"),
            "amalgamated_loan_unpaid_at_end_of_previous_year",
        )?,
        payments_applied_during_the_year: super::money::optional(
            input.get("payments_applied_during_the_year"),
            "payments_applied_during_the_year",
        )?,
    })
}

fn loan_summary(gate: &Value, repayment: &Value, payload: &Value) -> Value {
    let mut result = selected(payload, LOAN_FIELDS);
    result["response_detail"] = json!("summary");
    result["source"] = loan_source(gate, repayment);
    result["gate"] = selected(gate, GATE_FIELDS);
    result["minimum_yearly_repayment"] = selected(repayment, REPAYMENT_FIELDS);
    result
}

fn loan_source(gate: &Value, repayment: &Value) -> Value {
    let provenance = if gate["benchmark_provenance"].is_null() {
        &repayment["benchmark_provenance"]
    } else {
        &gate["benchmark_provenance"]
    };
    let uris = if gate["manifest"]["rate_table_uris"]
        .as_array()
        .is_none_or(|v| v.is_empty())
    {
        &repayment["manifest"]["rate_table_uris"]
    } else {
        &gate["manifest"]["rate_table_uris"]
    };
    json!({"verify_at":provenance.get("verify_at"),"rate_table_uris":uris})
}
