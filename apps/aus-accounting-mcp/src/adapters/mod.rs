//! Translation only. All statutory calculation and provenance belong to engines.
mod benchmarks;
mod div7a;
mod freshness;
mod gst;
mod money;
mod payday;
mod synthetic;
pub(crate) mod tax;

use serde_json::{Value, json};

pub fn evaluate(name: &str, input: &Value) -> Result<Value, String> {
    match name {
        "refuse_div7a" => Ok(json!({"ok":false,"available":false,"reviewed_engine":true,
            "code":"ERR_POLICY_DIV7A_SCOPE_REFUSED",
            "reason":"The reviewed div7a-loan-review engine covers s 109N loan terms, s 109N(2) benchmark rates and s 109E minimum yearly repayments for an operator-supplied amalgamated loan. It does not form amalgamated loans, classify repayments under s 109R, model unpaid present entitlements, distributable surplus, interposed entities, debt forgiveness or the Commissioner's discretion. Those matters remain refused."})),
        "get_div7a_benchmark_rate" => div7a::rate(input),
        "review_div7a_loan" => div7a::loan(input),
        "get_lodgment_due_dates" => lodgment(input),
        "list_ato_benchmark_industries" => benchmarks::list(input),
        "get_ato_benchmarks" => benchmarks::compare(input),
        "calculate_tax_worksheet" => tax::calculate(input),
        "get_data_freshness" => freshness::report(input),
        "review_gst_coding" => gst::coding(input),
        "reconcile_bas" => gst::bas(input),
        "generate_synthetic_sbr_fixture" => synthetic::generate(input),
        "calc_payday_super_deadline"
        | "review_payday_super_contributions"
        | "build_payday_super_evidence_pack" => payday::evaluate(name, input),
        "read_accounting_library"
        | "search_accounting_library"
        | "search_tax_legislation"
        | "read_tax_legislation_section"
        | "define_tax_term"
        | "search_tax_rates"
        | "search_ato_rulings"
        | "read_ato_ruling" => crate::retrieval::evaluate(name, input),
        _ => Err(format!(
            "{name}: native migration is incomplete; use the existing Python server"
        )),
    }
}

fn lodgment(input: &Value) -> Result<Value, String> {
    let mut input = input.clone();
    // Native engine expects the established holiday map, not MCP's typed list.
    let mut holidays = serde_json::Map::new();
    if let Some(items) = input["extra_holidays"].as_array() {
        for item in items {
            let day = item["date"]
                .as_str()
                .ok_or("extra_holidays.date must be a string")?;
            valid_day(day, "extra_holidays.date")?;
            let name = item["name"]
                .as_str()
                .ok_or("extra_holidays.name must be a string")?;
            if holidays.insert(day.into(), json!(name.trim())).is_some() {
                return Err(format!("extra_holidays: {day} appears more than once"));
            }
        }
    }
    input
        .as_object_mut()
        .unwrap()
        .insert("extra_holidays".into(), Value::Object(holidays));
    if let Some(day) = input["holidays_complete_until"].as_str() {
        valid_day(day, "holidays_complete_until")?;
    }
    let mut result = lodgment_calendar::api::evaluate("due-dates", &input)?;
    result["disclaimer"] = json!(
        "Preparation aid, not a lodgement or a determination of any due date. The operator supplies the facts, and none of them is verified. The ATO calls its program list a guide only: confirm each client's dates in Online services for agents, the practitioner lodgment service or on the statement before acting."
    );
    Ok(result)
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
