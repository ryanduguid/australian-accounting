// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! More malformed facts each command refuses with its own code: names used
//! twice, a wrong type, a list where an object belongs and amounts out of
//! bounds. See also `malformed_inputs.rs`.
use australian_tax_calculators::decimal::Decimal;
use australian_tax_calculators::{api, engine};
use serde_json::{Value, json};

fn code(command: &str, input: &Value) -> &'static str {
    match api::evaluate_coded(command, input) {
        Ok(_) => "accepted",
        Err(refusal) => refusal.code,
    }
}

/// Each case's code, so one run shows every mismatch.
fn codes(command: &str, cases: &[Value]) -> Vec<&'static str> {
    cases.iter().map(|c| code(command, c)).collect()
}

fn parcel(id: &str, security: &str, acquired: &str, units: &str, cost: &str) -> Value {
    json!({"parcel": id, "security": security, "acquired": acquired, "units": units,
           "acquisition_cost": cost, "incidental_costs": "0.00"})
}

fn adjustments(parcels: Value, events: Value) -> Value {
    json!({"entity": "individual", "parcels": parcels, "events": events,
           "year": "2025-26", "scope_confirmed": true})
}

fn year_amount(extra: Value) -> Value {
    let mut event = json!({"event": "tax_deferred", "security": "F", "income_year": "2025-26",
                           "amount": "10.00", "parcels": ["P1"]});
    for (k, v) in extra.as_object().unwrap() {
        event[k] = v.clone();
    }
    event
}

#[test]
fn cost_base_adjustments_refuse_clashing_names_and_malformed_events() {
    let p1 = parcel("P1", "F", "2021-01-04", "10", "500.00");
    let p2 = parcel("P2", "F", "2026-03-02", "10", "500.00");
    let two = || json!([p1, p2]);
    let exchange = json!({"event": "scrip_for_scrip", "security": "F", "date": "2025-10-01",
                          "new_security": "LBZ", "new_units_per_unit": "1",
                          "market_value_per_new_unit": "60", "rollover": true});
    let demerger = json!({"event": "demerger", "demerger": "wesfarmers_coles_2018",
                          "security": "WES", "new_security": "COL", "rollover": true});
    let mut demerger_flag = demerger.clone();
    demerger_flag["rollover"] = json!("yes");
    let mut wes = adjustments(
        json!([
            parcel("T2", "WES", "2005-03-01", "2000", "8000.00"),
            parcel("COL from T2", "COL", "2005-03-01", "1", "1.00")
        ]),
        json!([demerger]),
    );
    wes["year"] = json!("2018-19");
    let mut wes_flag = wes.clone();
    wes_flag["parcels"] = json!([parcel("T2", "WES", "2005-03-01", "2000", "8000.00")]);
    wes_flag["events"] = json!([demerger_flag]);
    let cases = [
        adjustments(json!([p1, p1]), json!([year_amount(json!({}))])),
        adjustments(
            json!([p1, parcel("LBZ from P1", "LBZ", "2021-01-04", "1", "1.00")]),
            json!([exchange]),
        ),
        wes,
        wes_flag,
        adjustments(two(), json!([year_amount(json!({"income_year": "2025"}))])),
        adjustments(two(), json!([year_amount(json!({"parcels": []}))])),
        adjustments(
            two(),
            json!([year_amount(
                json!({"parcels": ["P2"], "before_cgt_event": "2026-01-15"})
            )]),
        ),
    ];
    assert_eq!(
        codes("cgt-cost-base-adjustments", &cases),
        [
            "cgt_parcels.fact_conflict",
            "cgt_parcels.fact_conflict",
            "cgt_parcels.fact_conflict",
            "cgt_parcels.fact_invalid",
            "cgt_parcels.fact_invalid",
            "cgt_parcels.fact_invalid",
            "cgt_parcels.fact_conflict",
        ]
    );
}

#[test]
fn a_roll_over_refuses_a_flag_that_is_not_a_boolean() {
    let land = json!({"transferor": "individual", "basis": "court_order", "event": "A1",
                      "event_date": "2025-11-03", "trading_stock": "no",
                      "transferor_acquired": "2012-05-14", "cost_base": "10000.00",
                      "reduced_cost_base": "10000.00", "asset_class": "ordinary"});
    let input = json!({"rollover": "marriage_breakdown", "facts": land, "year": "2025-26",
                       "scope_confirmed": true});
    assert_eq!(code("cgt-rollover", &input), "cgt_parcels.fact_invalid");
}

fn loan(overrides: Value) -> Value {
    let mut loan = json!({
        "borrowing_expenses": "1600.00", "loan_start": "2020-07-03",
        "loan_term_end": "2045-07-02", "repaid_on": null, "income_use": "1",
        "section_26_50_applies": false,
    });
    for (k, v) in overrides.as_object().unwrap() {
        loan[k] = v.clone();
    }
    loan
}

fn loans(loans: Value) -> Value {
    json!({"loans": loans, "year": "2026-27", "scope_confirmed": true})
}

#[test]
fn the_borrowing_schedule_refuses_malformed_loans_and_uses() {
    let uses = |third: Value| {
        json!({"2020-21": "1", "2021-22": "1", "2022-23": third, "2023-24": "0.5",
               "2024-25": "0", "2025-26": "0.5"})
    };
    let mut other = loans(json!([loan(json!({}))]));
    other["other_borrowing_expenses"] = json!("1.00");
    let cases = [
        loans(json!({"loan": loan(json!({}))})),
        loans(json!([loan(json!({"income_use": uses(json!("1.5"))}))])),
        loans(json!([loan(json!({"income_use": uses(json!(0.5))}))])),
        loans(json!([loan(json!({"loan_start": 20_200_703}))])),
        loans(json!([loan(json!({"refinances": "1"}))])),
        other,
    ];
    assert_eq!(
        codes("borrowing-expenses-schedule", &cases),
        [
            "borrowing.loans_invalid",
            "borrowing.income_use_invalid",
            "borrowing.income_use_invalid",
            "input.date_invalid",
            "borrowing.refinance_invalid",
            "borrowing.other_expenses_invalid",
        ]
    );
}

fn message(command: &str, input: &Value) -> String {
    api::evaluate_coded(command, input).unwrap_err().message
}

/// An unsupported period is quoted as Python's `repr` gives it.
#[test]
fn an_unsupported_period_is_quoted_as_python_reprs_it() {
    let rollover = |year: &str| -> Value {
        serde_json::from_str(&format!(
            r#"{{"rollover": "marriage_breakdown", "facts": {{}}, "year": {year}, "scope_confirmed": true}}"#
        ))
        .unwrap()
    };
    assert!(message("cgt-rollover", &rollover("false")).starts_with("Unsupported period False;"));
    let odd = message("cgt-rollover", &rollover(r#"[-0, 1e400, {"a": false}]"#));
    assert!(
        odd.starts_with("Unsupported period [0, inf, {'a': False}];"),
        "{odd}"
    );
    // A Decimal or JSON year reaches the engine only from Rust callers.
    let yes = engine::Arg::Bool(true);
    let year = engine::Arg::Decimal(Decimal::parse("2026").unwrap());
    let quoted = |year: &engine::Arg| engine::fbt_board(&[], year, &yes).unwrap_err().to_string();
    assert!(quoted(&year).starts_with("Unsupported period Decimal('2026');"));
    assert!(quoted(&engine::Arg::Json(json!([2026]))).starts_with("Unsupported period [2026];"));
    // fbt reads its year as Python's str() gives it: Decimal 2026 is the
    // year ended 31 March 2026.
    let value = engine::Arg::Decimal(Decimal::parse("100.00").unwrap());
    let fbt = engine::fbt(&value, &value, &year, &yes).unwrap();
    assert_eq!(fbt["period"], "year ended 31 March 2026");
}

#[test]
fn periods_and_unknown_commands_are_refused() {
    assert_eq!(code("periods", &json!({"kind": 1})), "input.kind_invalid");
    assert_eq!(code("no-such-command", &json!({})), "command.unknown");
}

#[test]
fn a_flag_or_shortfall_of_the_wrong_type_is_refused() {
    let apportionment = json!({"small_business_entity": true, "carries_on_business": true,
        "instalment_election": false, "annual_tax_period_election": false,
        "election_made": true, "gst_on_acquisition": "100.00", "creditable_purpose": "0.1",
        "car": false, "reduced_credit_acquisition": "no", "year": "2025-26",
        "scope_confirmed": true});
    assert_eq!(
        code("gst-annual-apportionment", &apportionment),
        "input.flag_invalid"
    );
    let penalty = json!({"question": "statement_penalty", "year": "2025-26",
        "scope_confirmed": true, "statement_date": "2026-03-15",
        "significant_global_entity": false,
        "parts": [{"shortfall_amount": "100.001", "behaviour": "recklessness"}]});
    assert_eq!(
        code("ato-interest-penalties", &penalty),
        "input.amount_invalid"
    );
}
