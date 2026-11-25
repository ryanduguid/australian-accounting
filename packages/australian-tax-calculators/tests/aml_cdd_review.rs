// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `aml-cdd-review`: deterministic Australian AML/CTF Tranche 2 CDD checks for
//! 2026-27 with zero personal data (docs/calculation-evidence.md#aml-cdd-review).
use australian_tax_calculators::{api, engine};
use serde_json::{Value, json};

fn run(input: &Value) -> Value {
    api::evaluate("aml-cdd-review", input).unwrap()
}

fn code(input: &Value) -> String {
    api::evaluate_coded("aml-cdd-review", input)
        .unwrap_err()
        .code
        .to_string()
}

fn country_base(score: &str) -> Value {
    json!({
        "question": "country_risk",
        "basel_aml_score": score,
        "fatf_call_for_action": false,
        "fatf_increased_monitoring": false,
        "australian_sanctioned_jurisdiction": false,
        "year": "2026-27",
        "scope_confirmed": true
    })
}

fn bo_body_base(chains: Value) -> Value {
    json!({
        "question": "beneficial_ownership",
        "entity_kind": "body_corporate",
        "customer_exemption": "none",
        "ownership_chains": chains,
        "voting_power_over_50": false,
        "issued_share_capital_over_50": false,
        "board_control": false,
        "financial_operating_policy_control": false,
        "year": "2026-27",
        "scope_confirmed": true
    })
}

fn cdd_base() -> Value {
    json!({
        "question": "cdd_review",
        "last_cdd_date": "2026-07-01",
        "customer_risk": "low",
        "simplified_cdd_policy_in_place": true,
        "pep_status": "none",
        "acting_representative_foreign_pep": false,
        "smr_continue_relationship": false,
        "unusual_smr_behaviour": false,
        "fatf_call_for_action_jurisdiction": false,
        "nested_services_relationship": false,
        "unusual_service_request": false,
        "virtual_asset_physical_currency": false,
        "kyc_veracity_doubt": false,
        "significant_risk_change": false,
        "new_pep_status_change": false,
        "cadence_basis": "austrac_accountant_starter_kit",
        "year": "2026-27",
        "scope_confirmed": true
    })
}

#[test]
fn country_risk_matches_austrac_examples_and_conditional_s32d_notice() {
    for (score, band) in [
        ("0", "low"),
        ("0.00", "low"),
        ("4.04", "low"),
        ("5.00", "low"),
        ("5.01", "medium"),
        ("5.75", "medium"),
        ("6.00", "medium"),
        ("6.01", "high"),
        ("10.00", "high"),
    ] {
        let res = run(&country_base(score));
        assert_eq!(res["question"], "country_risk", "{score}");
        assert_eq!(res["basel_band"], band, "{score}");
        assert_eq!(res["country_risk_rating"], band, "{score}");
        assert_eq!(res["override_applied"], false, "{score}");
        assert_eq!(
            res["fatf_call_for_action_ecdd_applies_if_covered_connection"], false,
            "{score}"
        );
    }

    // AUSTRAC Country B: Basel 5.25 (medium) overridden to high by FATF grey list or sanctions.
    let mut grey = country_base("5.25");
    grey["fatf_increased_monitoring"] = json!(true);
    let res = run(&grey);
    assert_eq!(res["basel_band"], "medium");
    assert_eq!(res["override_applied"], true);
    assert_eq!(res["country_risk_rating"], "high");
    assert_eq!(
        res["fatf_call_for_action_ecdd_applies_if_covered_connection"],
        false
    );

    let mut sanctioned = country_base("5.25");
    sanctioned["australian_sanctioned_jurisdiction"] = json!(true);
    let res = run(&sanctioned);
    assert_eq!(res["override_applied"], true);
    assert_eq!(res["country_risk_rating"], "high");

    // S5 regression: FATF Call for Action does not assert unconditional statutory ECDD
    // without physical presence / formation connection under s 32(d).
    let mut black = country_base("4.50");
    black["fatf_call_for_action"] = json!(true);
    let res = run(&black);
    assert_eq!(res["basel_band"], "low");
    assert_eq!(res["override_applied"], true);
    assert_eq!(res["country_risk_rating"], "high");
    assert!(res.get("statutory_ecdd_triggered").is_none());
    assert_eq!(
        res["fatf_call_for_action_ecdd_applies_if_covered_connection"],
        true
    );
    let provs = res["statutory_provisions"].as_array().unwrap();
    assert_eq!(provs.len(), 2);
    assert!(provs[0].as_str().unwrap().contains("physically present in"));
    assert!(provs[1].as_str().unwrap().contains("where relevant"));
}

#[test]
fn country_risk_refuses_invalid_scores_extraneous_and_duplicate_fields() {
    for bad in [
        json!(""),
        json!("-0.01"),
        json!("10.01"),
        json!("5.005"),
        json!("5e0"),
        json!("abc"),
        json!(5.75),
    ] {
        let mut inp = country_base("5.00");
        inp["basel_aml_score"] = bad.clone();
        assert_eq!(code(&inp), "aml_cdd_review.basel_score_invalid", "{bad}");
    }
    let mut bad_flag = country_base("5.00");
    bad_flag["fatf_call_for_action"] = json!("yes");
    assert_eq!(code(&bad_flag), "input.flag_invalid");

    let mut extra = country_base("5.00");
    extra["entity_kind"] = json!("body_corporate");
    assert_eq!(code(&extra), "input.field_unknown");

    let mut missing = country_base("5.00");
    missing.as_object_mut().unwrap().remove("basel_aml_score");
    assert_eq!(code(&missing), "input.missing_field");

    // S8 regression: direct engine call with duplicate named key must refuse, not silently pick first.
    let named = vec![
        (
            "basel_aml_score".to_string(),
            engine::Arg::Text("4.00".into()),
        ),
        ("fatf_call_for_action".to_string(), engine::Arg::Bool(false)),
        ("fatf_call_for_action".to_string(), engine::Arg::Bool(true)),
        (
            "fatf_increased_monitoring".to_string(),
            engine::Arg::Bool(false),
        ),
        (
            "australian_sanctioned_jurisdiction".to_string(),
            engine::Arg::Bool(false),
        ),
    ];
    let err = engine::aml_cdd_review(
        &engine::Arg::Text("country_risk".into()),
        &engine::Arg::Text("2026-27".into()),
        &engine::Arg::Bool(true),
        &named,
    )
    .unwrap_err();
    assert_eq!(err.code(), "aml_cdd_review.duplicate_field");
}

#[test]
fn beneficial_ownership_traces_exact_scaled_chains_control_and_s6_7_exemptions() {
    // 50% x 50% = 25.00% exactly meets s 5(a).
    let res = run(&bo_body_base(json!([["50", "50"]])));
    assert_eq!(res["rates"]["effective_ownership_percent"], "25.00");
    assert_eq!(res["owns_25_or_more"], true);
    assert_eq!(res["controls_customer"], false);
    assert_eq!(res["is_beneficial_owner"], true);
    assert_eq!(res["identification_required"], true);

    // Multi-chain aggregation: 30% x 50% + 10% = 25.00%.
    let res = run(&bo_body_base(json!([["30", "50"], ["10"]])));
    assert_eq!(res["rates"]["effective_ownership_percent"], "25.00");
    assert_eq!(res["owns_25_or_more"], true);

    // S2 regression: exact below-threshold totals must never round up to 25%.
    let res = run(&bo_body_base(json!([["33.33", "75"]])));
    assert_eq!(res["rates"]["effective_ownership_percent"], "24.9975");
    assert_eq!(res["owns_25_or_more"], false);
    assert_eq!(res["is_beneficial_owner"], false);

    let sub_25_deep = run(&bo_body_base(json!([
        ["24.9999"],
        [
            "0.0001", "99.9999", "99.9999", "99.9999", "99.9999", "99.9999", "99.9999", "99.9999"
        ]
    ])));
    assert_eq!(sub_25_deep["owns_25_or_more"], false);
    assert!(
        sub_25_deep["rates"]["effective_ownership_percent"]
            .as_str()
            .unwrap()
            .starts_with("24.9999")
    );

    // S2 regression: 100 + 10^-46 must refuse as ownership_above_100 (zero 40-digit rounding).
    assert_eq!(
        code(&bo_body_base(json!([
            ["100"],
            [
                "0.0001", "0.0001", "0.0001", "0.0001", "0.0001", "0.0001", "0.0001", "0.0001"
            ]
        ]))),
        "aml_cdd_review.ownership_above_100"
    );

    // Formatting: 1 decimal place, trailing zeros, and tiny < 1% sub-unit chain.
    let res = run(&bo_body_base(json!([["50", "10.2"]])));
    assert_eq!(res["rates"]["effective_ownership_percent"], "5.10");
    let res = run(&bo_body_base(json!([["25.0000"]])));
    assert_eq!(res["rates"]["effective_ownership_percent"], "25.00");
    let res = run(&bo_body_base(json!([["0.0001", "0.0001"]])));
    assert_eq!(res["rates"]["effective_ownership_percent"], "0.0000000001");
    let res = run(&bo_body_base(json!([])));
    assert_eq!(res["rates"]["effective_ownership_percent"], "0.00");

    // Body corporate control limbs under s 11(1)(a)-(d) with 0% ownership.
    for key in [
        "voting_power_over_50",
        "issued_share_capital_over_50",
        "board_control",
        "financial_operating_policy_control",
    ] {
        let mut inp = bo_body_base(json!([]));
        inp[key] = json!(true);
        let res = run(&inp);
        assert_eq!(res["owns_25_or_more"], false, "{key}");
        assert_eq!(res["controls_customer"], true, "{key}");
        assert_eq!(res["is_beneficial_owner"], true, "{key}");
        assert_eq!(res["control_grounds"].as_array().unwrap().len(), 1, "{key}");
    }

    // Non-body corporate control limbs under s 11(2)(a)-(b).
    let non_body = json!({
        "question": "beneficial_ownership",
        "entity_kind": "non_body_corporate",
        "customer_exemption": "none",
        "ownership_chains": [],
        "board_control": true,
        "financial_operating_policy_control": true,
        "year": "2026-27",
        "scope_confirmed": true
    });
    let res = run(&non_body);
    assert_eq!(res["controls_customer"], true);
    assert_eq!(res["control_grounds"].as_array().unwrap().len(), 2);

    // S7 regression: all 4 Rules s 6-7(1), (1A), (1B), (1C) exemptions.
    for (ex, cite) in [
        ("listed_public_company", "s 6-7(1):"),
        ("government_body", "s 6-7(1A):"),
        ("controlled_by_listed_or_government", "s 6-7(1B):"),
        ("owned_through_listed_or_government_only", "s 6-7(1C):"),
    ] {
        let mut inp = bo_body_base(json!([["40"]]));
        inp["customer_exemption"] = json!(ex);
        let res = run(&inp);
        assert_eq!(res["is_beneficial_owner"], true, "{ex}");
        assert_eq!(res["beneficial_owner_cdd_deemed_established"], true, "{ex}");
        assert_eq!(res["identification_required"], false, "{ex}");
        assert!(
            res["exemption_authority"].as_str().unwrap().contains(cite),
            "{ex}"
        );
    }
}

#[test]
fn beneficial_ownership_refuses_malformed_and_inconsistent_inputs() {
    let mut bad_kind = bo_body_base(json!([]));
    bad_kind["entity_kind"] = json!("trust");
    assert_eq!(code(&bad_kind), "aml_cdd_review.entity_kind_invalid");

    let mut bad_ex = bo_body_base(json!([]));
    bad_ex["customer_exemption"] = json!("charity");
    assert_eq!(code(&bad_ex), "aml_cdd_review.exemption_invalid");

    let non_body_listed = json!({
        "question": "beneficial_ownership",
        "entity_kind": "non_body_corporate",
        "customer_exemption": "listed_public_company",
        "ownership_chains": [],
        "board_control": false,
        "financial_operating_policy_control": false,
        "year": "2026-27",
        "scope_confirmed": true
    });
    assert_eq!(
        code(&non_body_listed),
        "aml_cdd_review.exemption_inconsistent"
    );

    let non_body_vote = json!({
        "question": "beneficial_ownership",
        "entity_kind": "non_body_corporate",
        "customer_exemption": "none",
        "ownership_chains": [],
        "voting_power_over_50": true,
        "board_control": false,
        "financial_operating_policy_control": false,
        "year": "2026-27",
        "scope_confirmed": true
    });
    assert_eq!(code(&non_body_vote), "input.field_unknown");

    assert_eq!(
        code(&bo_body_base(json!([["60"], ["50"]]))),
        "aml_cdd_review.ownership_above_100"
    );

    for bad_chains in [
        json!("50"),
        json!(vec![json!(["1"]); 17]),
        json!(["50"]),
        json!([[]]),
        json!([["50", "50", "50", "50", "50", "50", "50", "50", "50"]]),
        json!([["0"]]),
        json!([["100.01"]]),
        json!([["101"]]),
        json!([["12.34567"]]),
        json!([["12."]]),
        json!([[".5"]]),
        json!([[""]]),
        json!([["1e1"]]),
        json!([[50]]),
    ] {
        assert_eq!(
            code(&bo_body_base(bad_chains.clone())),
            "aml_cdd_review.chains_invalid",
            "{bad_chains}"
        );
    }
}

#[test]
fn cdd_review_evaluates_statutory_tiers_triggers_and_s1_s3_s4_s6_regressions() {
    // Low risk + simplified policy in place -> simplified_permitted, 36 months.
    let res = run(&cdd_base());
    assert_eq!(res["cdd_tier"], "simplified_permitted");
    assert_eq!(
        res["source_of_wealth_and_funds_status"],
        "not_mandated_by_rules"
    );
    assert_eq!(res["immediate_review_required"], false);
    assert_eq!(res["cadence_is_statutory"], false);
    assert_eq!(res["review_interval_months"], 36);
    assert_eq!(res["next_periodic_review_date"], "2029-07-01");

    // S6 regression: low risk, statutory_only, no simplified CDD policy -> standard (NOT simplified_permitted).
    let mut no_simp_pol = cdd_base();
    no_simp_pol["simplified_cdd_policy_in_place"] = json!(false);
    no_simp_pol["cadence_basis"] = json!("statutory_only");
    let res = run(&no_simp_pol);
    assert_eq!(res["cdd_tier"], "standard");

    // S1 regression: pending-SMR unusual behaviour (s 30(2)(b)(ii)) without smr_continue_relationship
    // triggers immediate review without forcing s 32(b) ECDD.
    let mut pending_smr = cdd_base();
    pending_smr["unusual_smr_behaviour"] = json!(true);
    pending_smr["cadence_basis"] = json!("statutory_only");
    let res = run(&pending_smr);
    assert_eq!(res["cdd_tier"], "simplified_permitted");
    assert_eq!(res["immediate_review_required"], true);
    assert!(
        res["immediate_review_triggers"][0]
            .as_str()
            .unwrap()
            .contains("s 30(2)(b)(ii)")
    );

    // S3 regression: acting representative foreign PEP triggers s 32(c) ECDD WITHOUT triggering Rules 6-23/6-24.
    let mut acting_pep = cdd_base();
    acting_pep["acting_representative_foreign_pep"] = json!(true);
    acting_pep["cadence_basis"] = json!("statutory_only");
    let res = run(&acting_pep);
    assert_eq!(res["cdd_tier"], "enhanced_mandatory");
    assert!(
        res["ecdd_triggers"][0]
            .as_str()
            .unwrap()
            .contains("s 32(c)")
    );
    assert_eq!(
        res["source_of_wealth_and_funds_status"],
        "not_mandated_by_rules"
    );
    assert_eq!(res["sow_sof_triggers"].as_array().unwrap().len(), 0);
    assert_eq!(res["immediate_review_required"], false);

    // S4 regression: high risk alone (non-PEP, non-VA-cash) yields conditional_on_relevance under Rule 6-21,
    // not unconditional mandatory SoW + SoF.
    let mut high_only = cdd_base();
    high_only["customer_risk"] = json!("high");
    let res = run(&high_only);
    assert_eq!(res["cdd_tier"], "enhanced_mandatory");
    assert_eq!(
        res["source_of_wealth_and_funds_status"],
        "conditional_on_relevance"
    );
    assert_eq!(res["review_interval_months"], 12);
    assert_eq!(res["next_periodic_review_date"], "2027-07-01");

    // Medium risk under starter kit -> standard, 24 months.
    let mut med = cdd_base();
    med["customer_risk"] = json!("medium");
    med["last_cdd_date"] = json!("2026-08-31");
    let res = run(&med);
    assert_eq!(res["cdd_tier"], "standard");
    assert_eq!(res["review_interval_months"], 24);
    assert_eq!(res["next_periodic_review_date"], "2028-08-31");

    // High risk + domestic/IO PEP + new_pep triggers Rules 6-23 (mandatory) and 6-24(1)(b)/(c).
    let mut high_dom = high_only.clone();
    high_dom["pep_status"] = json!("domestic");
    high_dom["new_pep_status_change"] = json!(true);
    let res = run(&high_dom);
    assert_eq!(res["source_of_wealth_and_funds_status"], "mandatory");
    assert_eq!(res["sow_sof_triggers"].as_array().unwrap().len(), 2);
    assert_eq!(res["immediate_review_required"], true);

    let mut io_pep = high_dom.clone();
    io_pep["pep_status"] = json!("international_organisation");
    let res = run(&io_pep);
    assert!(
        res["immediate_review_triggers"][0]
            .as_str()
            .unwrap()
            .contains("s 6-24(1)(c)")
    );

    // Medium-risk domestic PEP newly acquired does NOT trigger s 6-24(1)(b) or s 6-23(1)(b).
    let mut med_pep = med.clone();
    med_pep["pep_status"] = json!("domestic");
    med_pep["new_pep_status_change"] = json!(true);
    let res = run(&med_pep);
    assert_eq!(res["cdd_tier"], "standard");
    assert_eq!(
        res["source_of_wealth_and_funds_status"],
        "not_mandated_by_rules"
    );
    assert_eq!(res["immediate_review_required"], false);

    // Custom policy (6 months, month-end clamp 2026-08-31 -> 2027-02-28) with all 7 ECDD triggers.
    let mut custom = cdd_base();
    custom["last_cdd_date"] = json!("2026-08-31");
    custom["customer_risk"] = json!("high");
    custom["pep_status"] = json!("foreign");
    custom["smr_continue_relationship"] = json!(true);
    custom["fatf_call_for_action_jurisdiction"] = json!(true);
    custom["nested_services_relationship"] = json!(true);
    custom["unusual_service_request"] = json!(true);
    custom["virtual_asset_physical_currency"] = json!(true);
    custom["kyc_veracity_doubt"] = json!(true);
    custom["significant_risk_change"] = json!(true);
    custom["new_pep_status_change"] = json!(true);
    custom["cadence_basis"] = json!("custom_policy");
    custom["policy_review_months"] = json!(6);
    let res = run(&custom);
    assert_eq!(res["cdd_tier"], "enhanced_mandatory");
    assert_eq!(res["ecdd_triggers"].as_array().unwrap().len(), 7);
    assert_eq!(res["source_of_wealth_and_funds_status"], "mandatory");
    assert_eq!(res["sow_sof_triggers"].as_array().unwrap().len(), 3);
    assert_eq!(
        res["immediate_review_triggers"].as_array().unwrap().len(),
        4
    );
    assert_eq!(res["cadence_is_statutory"], false);
    assert_eq!(res["review_interval_months"], 6);
    assert_eq!(res["next_periodic_review_date"], "2027-02-28");

    // Statutory-only mode returns null interval and date with cadence_is_statutory = true.
    let mut stat = cdd_base();
    stat["cadence_basis"] = json!("statutory_only");
    stat["nested_services_relationship"] = json!(true);
    let res = run(&stat);
    assert_eq!(res["cdd_tier"], "enhanced_mandatory");
    assert_eq!(
        res["source_of_wealth_and_funds_status"],
        "not_mandated_by_rules"
    );
    assert_eq!(res["cadence_is_statutory"], true);
    assert_eq!(res["review_interval_months"], Value::Null);
    assert_eq!(res["next_periodic_review_date"], Value::Null);
}

#[test]
fn cdd_review_scope_and_top_level_refuse_invalid_and_inconsistent_inputs() {
    // S9 regression: catalogue scope states business-relationship prerequisite and excludes occasional-only.
    let cat = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let scope = cat["aml_cdd_review"]["scope"].as_str().unwrap();
    assert!(scope.contains("business relationship"));
    assert!(scope.contains("occasional-only"));

    // E1 regression: starter kit safeguard states worksheet safeguard accurately.
    let mut conflict = cdd_base();
    conflict["pep_status"] = json!("foreign");
    let err = api::evaluate_coded("aml-cdd-review", &conflict).unwrap_err();
    assert_eq!(err.code, "aml_cdd_review.starter_kit_risk_inconsistent");
    assert!(err.message.starts_with("Worksheet safeguard:"));

    let mut pep_conflict = cdd_base();
    pep_conflict["new_pep_status_change"] = json!(true);
    assert_eq!(
        code(&pep_conflict),
        "aml_cdd_review.pep_change_inconsistent"
    );

    for d in ["2026-06-30", "2027-07-01"] {
        let mut inp = cdd_base();
        inp["last_cdd_date"] = json!(d);
        assert_eq!(code(&inp), "aml_cdd_review.date_outside_year", "{d}");
    }
    let mut bad_date = cdd_base();
    bad_date["last_cdd_date"] = json!("2026-02-29");
    assert_eq!(code(&bad_date), "input.date_invalid");

    let mut bad_risk = cdd_base();
    bad_risk["customer_risk"] = json!("extreme");
    assert_eq!(code(&bad_risk), "aml_cdd_review.risk_invalid");

    let mut bad_pep = cdd_base();
    bad_pep["pep_status"] = json!("local");
    assert_eq!(code(&bad_pep), "aml_cdd_review.pep_invalid");

    let mut bad_basis = cdd_base();
    bad_basis["cadence_basis"] = json!("monthly");
    assert_eq!(code(&bad_basis), "aml_cdd_review.cadence_basis_invalid");

    let mut extra_months = cdd_base();
    extra_months["policy_review_months"] = json!(12);
    assert_eq!(code(&extra_months), "input.field_unknown");

    let mut missing_months = cdd_base();
    missing_months["cadence_basis"] = json!("custom_policy");
    assert_eq!(code(&missing_months), "input.missing_field");

    for bad_m in [json!(0), json!(61), json!("12")] {
        let mut inp = cdd_base();
        inp["cadence_basis"] = json!("custom_policy");
        inp["policy_review_months"] = bad_m.clone();
        assert_eq!(code(&inp), "aml_cdd_review.months_invalid", "{bad_m}");
    }

    let mut bad_q = country_base("4.00");
    bad_q["question"] = json!("sanctions_list");
    assert_eq!(code(&bad_q), "aml_cdd_review.question_invalid");

    let mut unconfirmed = country_base("4.00");
    unconfirmed["scope_confirmed"] = json!(false);
    assert_eq!(code(&unconfirmed), "scope.not_confirmed");

    for yr in ["2025-26", "2027-28"] {
        let mut inp = country_base("4.00");
        inp["year"] = json!(yr);
        assert_eq!(code(&inp), "period.unsupported", "{yr}");
    }
    let mut yr2728 = country_base("4.00");
    yr2728["year"] = json!("2027-28");
    let err = api::evaluate_coded("aml-cdd-review", &yr2728).unwrap_err();
    assert!(err.message.contains("2027-28 amendments"));
}
