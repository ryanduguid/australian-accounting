// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Translation only. All statutory calculation and provenance belong to engines.
mod div7a;
pub mod freshness;
mod money;
mod payday;
pub mod tax;

use serde_json::{Value, json};

pub fn evaluate(name: &str, input: &Value) -> Result<Value, String> {
    match name {
        "refuse_div7a" => Ok(json!({"ok":false,"available":false,"reviewed_engine":true,
            "code":"ERR_POLICY_DIV7A_SCOPE_REFUSED",
            "reason":"The reviewed div7a-loan-review engine covers s 109N loan terms, s 109N(2) benchmark rates and s 109E minimum yearly repayments for an operator-supplied amalgamated loan. It does not form amalgamated loans, classify repayments under s 109R, model unpaid present entitlements, distributable surplus, interposed entities, debt forgiveness or the Commissioner's discretion. Those matters remain refused."})),
        "get_div7a_benchmark_rate" => div7a::rate(input),
        "review_div7a_loan" => div7a::loan(input),
        "calculate_tax_worksheet" => tax::calculate(input),
        "get_data_freshness" => freshness::report(input),
        "calc_payday_super_deadline" => payday::evaluate(input),
        _ => Err(format!(
            "{name}: native migration is incomplete; use the existing Python server"
        )),
    }
}

pub(super) fn valid_day(day: &str, field: &str) -> Result<(), String> {
    chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d")
        .ok()
        .filter(|date| chrono::Datelike::year(date) > 0)
        .map(|_| ())
        .ok_or_else(|| {
            format!(
                "{field}: {} is not a YYYY-MM-DD date",
                div7a_loan_review::raw::Raw::text(day).py_repr()
            )
        })
}
