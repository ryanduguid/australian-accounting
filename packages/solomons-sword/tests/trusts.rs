// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `partnership-net-income`, `trustee-assessment` and `trust-loss-tests`: worked
//! examples from the ATO's instructions and the Acts, each rate table entry, each
//! year boundary and each refusal.
use serde_json::{Value, json};
use solomons_sword::api::evaluate;

fn ok(command: &str, v: Value) -> Value {
    evaluate(command, &v).unwrap_or_else(|e| panic!("{command} refused {v}: {e}"))
}
fn err(command: &str, v: Value) -> String {
    evaluate(command, &v).expect_err("expected a refusal")
}
fn merge(mut base: Value, extra: Value) -> Value {
    for (k, x) in extra.as_object().unwrap() {
        base[k] = x.clone();
    }
    base
}

// ---- partnership-net-income ----

fn partner(name: &str, pct: &str) -> Value {
    json!({"name": name, "residency": "resident", "interest_percentage": pct})
}
fn partnership(extra: Value) -> Value {
    let base = json!({"financial_year": 2025, "assessable_income": "1000", "deductions": "0",
        "partners": [partner("A", "50"), partner("B", "50")]});
    ok("partnership-net-income", merge(base, extra))
}
fn partnership_err(extra: Value) -> String {
    let base = json!({"financial_year": 2025, "assessable_income": "1000", "deductions": "0",
        "partners": [partner("A", "50"), partner("B", "50")]});
    err("partnership-net-income", merge(base, extra))
}
fn column(r: &Value, key: &str) -> Vec<String> {
    r["partners"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p[key].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn partnership_ato_example_11_equal_partners() {
    // ATO Partnership tax return 2025 instructions, Example 11: net income $1,000,
    // 2 Australian partners with equal interests each include $500.
    let r = partnership(json!({}));
    assert_eq!(r["net_income"], "1000.00");
    assert_eq!(r["partnership_loss"], "0.00");
    assert_eq!(column(&r, "assessable_income"), ["500.00", "500.00"]);
    assert_eq!(column(&r, "deductible_loss"), ["0.00", "0.00"]);
    assert!(r["steps"][0].as_str().unwrap().contains("s 91"));
}

#[test]
fn partnership_loss_is_shared_as_deductions() {
    let r = partnership(json!({"assessable_income": "40000", "deductions": "70000",
        "partners": [partner("A", "60"), partner("B", "40")]}));
    assert_eq!(r["net_income"], "0.00");
    assert_eq!(r["partnership_loss"], "30000.00");
    assert_eq!(column(&r, "deductible_loss"), ["18000.00", "12000.00"]);
    assert_eq!(column(&r, "assessable_income"), ["0.00", "0.00"]);
}

#[test]
fn partnership_excludes_s_290_150_and_division_36_deductions() {
    let r = partnership(json!({"assessable_income": "100000", "deductions": "30000",
        "excluded_deductions": "5000"}));
    assert_eq!(r["deductions_counted"], "25000.00");
    assert_eq!(r["net_income"], "75000.00");
    // Assessable income equal to counted deductions: neither net income nor loss.
    let r = partnership(json!({"assessable_income": "25000", "deductions": "30000",
        "excluded_deductions": "5000"}));
    assert_eq!(
        (r["net_income"].as_str(), r["partnership_loss"].as_str()),
        (Some("0.00"), Some("0.00"))
    );
    assert!(
        partnership_err(json!({"deductions": "10", "excluded_deductions": "11"}))
            .contains("cannot exceed")
    );
}

#[test]
fn partnership_rounding_residual_goes_to_the_largest_share() {
    let r = partnership(json!({"assessable_income": "0.10",
        "partners": [partner("A", "33.33"), partner("B", "33.33"), partner("C", "33.34")]}));
    assert_eq!(column(&r, "assessable_income"), ["0.03", "0.03", "0.04"]);
    // An overdrawn cent comes off the largest share first.
    let r = partnership(json!({"assessable_income": "0.05",
        "partners": [partner("A", "50"), partner("B", "50")]}));
    assert_eq!(column(&r, "assessable_income"), ["0.02", "0.03"]);
    let r = partnership(json!({"assessable_income": "0.03",
        "partners": [partner("A", "50"), partner("B", "25"), partner("C", "25")]}));
    assert_eq!(column(&r, "assessable_income"), ["0.01", "0.01", "0.01"]);
    let r = partnership(json!({"assessable_income": "0.02",
        "partners": [partner("A", "25"), partner("B", "25"), partner("C", "25"), partner("D", "25")]}));
    assert_eq!(
        column(&r, "assessable_income"),
        ["0.00", "0.00", "0.01", "0.01"]
    );
}

#[test]
fn partnership_exempt_and_nane_income_follow_the_interests() {
    let r = partnership(
        json!({"exempt_income": "900", "non_assessable_non_exempt_income": "300",
        "partners": [partner("A", "66.67"), partner("B", "33.33")]}),
    );
    assert_eq!(column(&r, "exempt_income"), ["600.03", "299.97"]);
    assert_eq!(
        column(&r, "non_assessable_non_exempt_income"),
        ["200.01", "99.99"]
    );
    assert_eq!(r["exempt_income"], "900.00");
}

#[test]
fn partnership_non_resident_partner_only_with_australian_income() {
    let nr = json!({"name": "N", "residency": "non_resident", "interest_percentage": "50"});
    let r = partnership(json!({"partners": [partner("A", "50"), nr.clone()],
        "all_income_from_australian_sources": true}));
    assert_eq!(column(&r, "assessable_income"), ["500.00", "500.00"]);
    assert_eq!(r["partners"][1]["residency"], "non_resident");
    for stated in [json!(false), Value::Null] {
        let e = partnership_err(json!({"partners": [partner("A", "50"), nr.clone()],
            "all_income_from_australian_sources": stated}));
        assert!(e.starts_with("refused:") && e.contains("s 92(1)(b)"), "{e}");
    }
    let py = json!({"name": "P", "residency": "part_year", "interest_percentage": "50"});
    assert!(
        partnership_err(json!({"partners": [partner("A", "50"), py]}))
            .contains("s 92(1) ITAA 1936")
    );
    let bad = json!({"name": "P", "residency": "abroad", "interest_percentage": "50"});
    assert!(
        partnership_err(json!({"partners": [partner("A", "50"), bad]}))
            .contains("residency must be")
    );
}

#[test]
fn partnership_refusals_name_their_provisions() {
    for (flag, law) in [
        ("corporate_limited_partnership", "Div 5A"),
        ("venture_capital_partnership", "92(2AA)"),
        ("foreign_hybrid", "830-45"),
        ("uncontrolled_partnership_income", "s 94"),
        ("segregated_or_pension_asset_interest", "92(2A)"),
        ("partner_salaries_or_interest", "s 92"),
    ] {
        let e = partnership_err(json!({ flag: true }));
        assert!(e.starts_with("refused:") && e.contains(law), "{flag}: {e}");
        partnership(json!({ flag: false }));
    }
    assert!(partnership_err(json!({"foreign_hybrid": "yes"})).contains("true or false"));
}

#[test]
fn partnership_interests_and_amounts_are_checked() {
    let e = partnership_err(json!({"partners": [partner("A", "50"), partner("B", "49.99")]}));
    assert!(e.contains("must total 100"), "{e}");
    assert!(partnership_err(json!({"partners": []})).contains("must total 100"));
    assert!(partnership_err(json!({"partners": [partner("A", "101")]})).contains("from 0 to 100"));
    for amount in ["-1", "1.001", "1E+15", "NaN"] {
        let e = partnership_err(json!({ "assessable_income": amount }));
        assert!(e.contains("non-negative amount"), "{amount}: {e}");
    }
    assert_eq!(
        partnership_err(json!({"assessable_income": "abc"})),
        "decimal.InvalidOperation"
    );
}

#[test]
fn partnership_years_2025_to_2027_only() {
    for fy in [2025, 2026, 2027] {
        assert_eq!(
            partnership(json!({"financial_year": fy}))["financial_year"],
            fy
        );
    }
    for fy in [2024, 2028] {
        let e = partnership_err(json!({"financial_year": fy}));
        assert!(
            e.starts_with("refused:") && e.contains("outside 2025"),
            "{e}"
        );
    }
}

// ---- trustee-assessment ----

fn ben(name: &str, entitlement: &str, extra: Value) -> Value {
    merge(
        json!({"name": name, "kind": "individual", "residency": "resident", "entitlement": entitlement,
            "under_18": false, "under_legal_disability": false}),
        extra,
    )
}
fn trust_base() -> Value {
    json!({"financial_year": 2025, "trust_residency": "resident", "net_income": "100000",
        "income_of_trust_estate": "80000", "beneficiaries": []})
}
fn trust(extra: Value) -> Value {
    ok("trustee-assessment", merge(trust_base(), extra))
}
fn trust_err(extra: Value) -> String {
    err("trustee-assessment", merge(trust_base(), extra))
}
/// The trustee's tax on one beneficiary entitled to all the trust income.
fn one(fy: i64, net: &str, b: Value, extra: Value) -> Value {
    let r = trust(merge(
        json!({"financial_year": fy, "net_income": net, "income_of_trust_estate": "1",
            "beneficiaries": [merge(ben("B", "1", json!({})), b)]}),
        extra,
    ));
    r["beneficiaries"][0].clone()
}
/// The trustee's tax on net income to which no beneficiary is entitled.
fn unallocated(fy: i64, net: &str, extra: Value) -> Value {
    let r = trust(merge(
        json!({"financial_year": fy, "net_income": net, "income_of_trust_estate": "0"}),
        extra,
    ));
    r["no_beneficiary_presently_entitled"].clone()
}
fn bankrupt() -> Value {
    json!({"section_99a_2_kind": "bankrupt_property", "commissioner_opinion_99a_unreasonable": true})
}

#[test]
fn trustee_worked_example_t1() {
    let r = trust(json!({"beneficiaries": [ben("A", "40000", json!({})),
        ben("B", "20000", json!({"under_legal_disability": true}))]}));
    let a = &r["beneficiaries"][0];
    assert_eq!(a["division_6_percentage"], "50.0000");
    assert_eq!(a["share_of_net_income"], "50000.00");
    assert_eq!(a["assessed_to"], "beneficiary");
    assert_eq!(a["provision"], "s 97(1)(a)(i) ITAA 1936");
    assert_eq!(a["trustee_tax"], "0.00");
    let b = &r["beneficiaries"][1];
    assert_eq!(b["share_of_net_income"], "25000.00");
    assert_eq!(b["provision"], "s 98(1)(a) ITAA 1936");
    assert_eq!(b["trustee_tax"], "1088.00");
    let t = &r["no_beneficiary_presently_entitled"];
    assert_eq!(t["division_6_percentage"], "25.0000");
    assert_eq!(t["share_of_net_income"], "25000.00");
    assert_eq!(t["provision"], "s 99A(4A) ITAA 1936");
    assert_eq!(t["trustee_tax"], "11250.00");
    assert_eq!(r["total_trustee_tax"], "12338.00");
    assert_eq!(r["not_computed"], "Medicare levy, tax offsets and credits");
}

#[test]
fn trustee_resident_scale_each_band_and_year() {
    let disabled = json!({"under_legal_disability": true});
    for (fy, net, tax) in [
        (2025, "18200", "0.00"),
        (2025, "45000", "4288.00"),
        (2025, "50000", "5788.00"),
        (2025, "135000", "31288.00"),
        (2025, "190000", "51638.00"),
        (2025, "200000", "56138.00"),
        (2026, "50000", "5788.00"),
        (2027, "45000", "4020.00"),
        (2027, "50000", "5520.00"),
        (2027, "200000", "55870.00"),
    ] {
        assert_eq!(
            one(fy, net, disabled.clone(), json!({}))["trustee_tax"],
            tax,
            "{fy} {net}"
        );
    }
}

#[test]
fn trustee_non_resident_scale_each_band() {
    let aus = json!({"all_net_income_from_australian_sources": true});
    let nr = json!({"residency": "non_resident"});
    for (net, tax) in [
        ("100000", "30000.00"),
        ("135000", "40500.00"),
        ("150000", "46050.00"),
        ("200000", "65350.00"),
    ] {
        let b = one(2025, net, nr.clone(), aus.clone());
        assert_eq!(b["provision"], "s 98(2A), (3)(a) ITAA 1936");
        assert_eq!(b["trustee_tax"], tax, "{net}");
    }
    // Under a legal disability and non-resident: s 98(1)(b), same scale.
    let b = one(
        2027,
        "150000",
        merge(nr, json!({"under_legal_disability": true})),
        aus,
    );
    assert_eq!(b["provision"], "s 98(1)(b) ITAA 1936");
    assert_eq!(b["trustee_tax"], "46050.00");
}

#[test]
fn trustee_company_and_trustee_beneficiaries() {
    let aus = json!({"all_net_income_from_australian_sources": true});
    let co = json!({"kind": "company", "residency": "non_resident", "under_18": null, "under_legal_disability": null});
    let b = one(
        2025,
        "10000",
        merge(co.clone(), json!({"base_rate_entity": true})),
        aus.clone(),
    );
    assert_eq!(
        (b["provision"].as_str(), b["trustee_tax"].as_str()),
        (Some("s 98(2A), (3)(b) ITAA 1936"), Some("2500.00"))
    );
    let b = one(
        2025,
        "10000",
        merge(co.clone(), json!({"base_rate_entity": false})),
        aus.clone(),
    );
    assert_eq!(b["trustee_tax"], "3000.00");
    let e = trust_err(
        json!({"beneficiaries": [merge(ben("C", "1", json!({})), co.clone())], "all_net_income_from_australian_sources": true}),
    );
    assert!(e.contains("base rate entity status is not stated"), "{e}");
    let tb = json!({"kind": "trustee", "residency": "non_resident"});
    let b = one(2025, "10000", tb, aus.clone());
    assert_eq!(
        (b["provision"].as_str(), b["trustee_tax"].as_str()),
        (Some("s 98(4) ITAA 1936"), Some("4500.00"))
    );
    for kind in ["company", "trustee"] {
        let b = one(2025, "10000", json!({"kind": kind}), json!({}));
        assert_eq!(
            (b["assessed_to"].as_str(), b["trustee_tax"].as_str()),
            (Some("beneficiary"), Some("0.00"))
        );
        let e = trust_err(
            json!({"beneficiaries": [ben("X", "1", json!({"kind": kind, "under_legal_disability": true}))]}),
        );
        assert!(e.contains("s 95B"), "{e}");
    }
    assert!(
        trust_err(json!({"beneficiaries": [ben("X", "1", json!({"kind": "club"}))]}))
            .contains("kind must be")
    );
}

#[test]
fn trustee_section_99a_rate_and_section_99_discretion() {
    let t = unallocated(2025, "1000", json!({}));
    assert_eq!(
        (t["provision"].as_str(), t["trustee_tax"].as_str()),
        (Some("s 99A(4) ITAA 1936"), Some("450.00"))
    );
    // Opinion stated false: s 99A still applies.
    let t = unallocated(
        2025,
        "1000",
        json!({"section_99a_2_kind": "bankruptcy_part_xi", "commissioner_opinion_99a_unreasonable": false}),
    );
    assert_eq!(t["trustee_tax"], "450.00");
    // T5: s 99(2), cl 2, beyond the phase-out limit.
    let t = unallocated(2025, "5000", bankrupt());
    assert_eq!(
        (t["provision"].as_str(), t["trustee_tax"].as_str()),
        (Some("s 99(2) ITAA 1936"), Some("800.00"))
    );
    let t = unallocated(
        2025,
        "50000",
        json!({"section_99a_2_kind": "section_102ag_2_c", "commissioner_opinion_99a_unreasonable": true}),
    );
    assert_eq!(t["trustee_tax"], "8700.00");
    // T6: deceased estate within 3 years keeps the tax-free threshold.
    let estate = json!({"section_99a_2_kind": "will_or_intestacy", "commissioner_opinion_99a_unreasonable": true, "deceased_estate_less_than_3_years": true});
    let t = unallocated(2026, "30000", estate.clone());
    assert_eq!(t["trustee_tax"], "1888.00");
    let old = merge(estate, json!({"deceased_estate_less_than_3_years": false}));
    assert_eq!(unallocated(2026, "30000", old)["trustee_tax"], "4800.00");
    // A part, not the whole: s 99(3).
    let r = trust(merge(
        bankrupt(),
        json!({"beneficiaries": [ben("A", "40000", json!({}))]}),
    ));
    assert_eq!(
        r["no_beneficiary_presently_entitled"]["provision"],
        "s 99(3) ITAA 1936"
    );
}

#[test]
fn trustee_section_14_limitation_and_phase_out() {
    for (fy, net, tax) in [
        (2025, "416", "0.00"),
        (2025, "416.01", "0.01"),
        (2025, "500", "42.00"),
        (2025, "611", "97.50"),
        (2025, "612", "97.92"),
        (2026, "611", "97.50"),
        (2027, "594", "89.00"),
        (2027, "595", "89.25"),
    ] {
        assert_eq!(
            unallocated(fy, net, bankrupt())["trustee_tax"],
            tax,
            "{fy} {net}"
        );
    }
}

#[test]
fn trustee_s_99a_2_facts_must_be_stated() {
    let e = trust_err(json!({"section_99a_2_kind": "bankrupt_property"}));
    assert!(e.contains("Commissioner's opinion is not stated"), "{e}");
    let e = trust_err(
        json!({"section_99a_2_kind": "will_or_intestacy", "commissioner_opinion_99a_unreasonable": true}),
    );
    assert!(
        e.contains("less than 3 years") && e.contains("cl 1(b)"),
        "{e}"
    );
    assert!(
        trust_err(json!({"section_99a_2_kind": "charity"})).contains("section_99a_2_kind must be")
    );
    assert!(trust_err(json!({"section_99a_2_kind": 3})).contains("must be a string"));
    // A null kind is no kind.
    assert_eq!(
        trust(json!({"section_99a_2_kind": null}))["no_beneficiary_presently_entitled"]["trustee_tax"],
        "45000.00"
    );
    // No unallocated part: the s 99A(2) facts are not needed.
    let r = trust(
        json!({"section_99a_2_kind": "bankrupt_property", "income_of_trust_estate": "10",
        "beneficiaries": [ben("A", "10", json!({}))]}),
    );
    let t = &r["no_beneficiary_presently_entitled"];
    assert_eq!(
        (t["share_of_net_income"].as_str(), t["provision"].clone()),
        (Some("0.00"), Value::Null)
    );
}

#[test]
fn trustee_nil_trust_income_and_nil_net_income() {
    // Nil income of the trust estate: the trustee has 100% (s 95(1)).
    let r =
        trust(json!({"income_of_trust_estate": "0", "beneficiaries": [ben("A", "0", json!({}))]}));
    assert_eq!(r["beneficiaries"][0]["division_6_percentage"], "0.0000");
    assert_eq!(
        r["no_beneficiary_presently_entitled"]["division_6_percentage"],
        "100.0000"
    );
    assert_eq!(
        r["no_beneficiary_presently_entitled"]["provision"],
        "s 99A(4) ITAA 1936"
    );
    let e = trust_err(
        json!({"income_of_trust_estate": "0", "beneficiaries": [ben("A", "1", json!({}))]}),
    );
    assert!(e.contains("more than the income"), "{e}");
    let r = trust(json!({"net_income": "0", "beneficiaries": [ben("A", "40000", json!({}))]}));
    assert_eq!(r["total_trustee_tax"], "0.00");
    assert_eq!(
        r["no_beneficiary_presently_entitled"]["provision"],
        Value::Null
    );
}

#[test]
fn trustee_thirds_reconcile_to_net_income() {
    let r = trust(json!({"net_income": "0.10", "income_of_trust_estate": "3",
        "beneficiaries": [ben("A", "1", json!({})), ben("B", "1", json!({})), ben("C", "1", json!({}))]}));
    let shares: Vec<&str> = r["beneficiaries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["share_of_net_income"].as_str().unwrap())
        .collect();
    assert_eq!(shares, ["0.04", "0.03", "0.03"]);
    assert_eq!(
        r["no_beneficiary_presently_entitled"]["share_of_net_income"],
        "0.00"
    );
    assert_eq!(r["beneficiaries"][0]["division_6_percentage"], "33.3333");
}

#[test]
fn trustee_beneficiary_refusals() {
    for (extra, law) in [
        (json!({"under_18": true}), "Div 6AA"),
        (
            json!({"under_18": null}),
            "whether the beneficiary is under 18 is not stated",
        ),
        (
            json!({"under_legal_disability": null}),
            "legal disability is not stated",
        ),
        (
            json!({"residency": "part_year"}),
            "parts attributable to periods",
        ),
        (json!({"residency": "non_resident"}), "Australian sources"),
        (json!({"deemed_entitlement_s95a2": true}), "98(2)"),
        (json!({"farm_management_deposit_owner": true}), "s 97A"),
        (json!({"exempt_non_resident_entity": true}), "97(3)"),
    ] {
        let e = trust_err(json!({"beneficiaries": [ben("A", "1", extra.clone())]}));
        assert!(e.starts_with("refused:") && e.contains(law), "{extra}: {e}");
    }
    let e = trust_err(json!({"beneficiaries": [ben("A", "1", json!({"residency": "abroad"}))]}));
    assert!(e.contains("residency must be"), "{e}");
    let e = trust_err(json!({"beneficiaries": [ben("A", "1", json!({"under_18": "no"}))]}));
    assert!(e.contains("true, false or null"), "{e}");
}

#[test]
fn trustee_trust_refusals_and_years() {
    for (flag, law) in [
        ("capital_gain_or_franked_distribution", "Div 6E"),
        ("special_disability_trust", "95AB"),
        ("managed_investment_trust", "275"),
        ("public_trading_or_corporate_unit_trust", "6C"),
        ("uncontrolled_partnership_income", "s 94"),
    ] {
        let e = trust_err(json!({ flag: true }));
        assert!(e.starts_with("refused:") && e.contains(law), "{flag}: {e}");
    }
    let e = trust_err(json!({"trust_residency": "non_resident"}));
    assert!(e.starts_with("refused:") && e.contains("99A(4B)"), "{e}");
    assert!(trust_err(json!({"trust_residency": "offshore"})).contains("trust_residency must be"));
    for fy in [2024, 2028] {
        assert!(trust_err(json!({"financial_year": fy})).contains("outside 2025"));
    }
    assert!(trust_err(json!({"net_income": "-5"})).contains("non-negative"));
}

// ---- trust-loss-tests ----

fn loss_base() -> Value {
    json!({"financial_year": 2027, "loss_year": 2025, "trust_kind": "non_fixed",
        "deduction": "prior_year_tax_loss"})
}
fn losses(extra: Value) -> Value {
    ok("trust-loss-tests", merge(loss_base(), extra))
}
fn losses_err(extra: Value) -> String {
    err("trust-loss-tests", merge(loss_base(), extra))
}
fn test<'a>(r: &'a Value, name: &str) -> &'a Value {
    r["tests"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["test"] == name)
        .unwrap()
}
/// Every non-fixed fact stated so that each test passes.
fn clean() -> Value {
    json!({"income_distributed_in_income_year_or_2_months_after": true,
        "income_distributed_in_6_earlier_years": true,
        "capital_distributed_in_income_year_or_2_months_after": false,
        "capital_distributed_in_6_earlier_years": false,
        "pattern_of_distributions_test_passed": true, "previously_failed_pattern_test": false,
        "individuals_had_over_50_percent_stake": false, "group_began_to_control": false,
        "income_injection_scheme": false})
}

#[test]
fn trust_loss_ato_ray_non_fixed_trust_fails_pattern_test() {
    // ATO 'How we apply the trust loss test': Ray Non-fixed Trust, minimum total 40%.
    let r = losses(merge(
        clean(),
        json!({"pattern_of_distributions_test_passed": null,
        "test_year_distributions": {"income": [
            {"Mum": "80", "Dad": "10", "Ray": "10"},
            {"Mum": "20", "Dad": "50", "Ray": "30"}]}}),
    ));
    let t = test(&r, "pattern_of_distributions");
    assert_eq!(
        (t["applies"].clone(), t["passed"].clone()),
        (json!(true), json!(false))
    );
    let w = &r["pattern_of_distributions_working"]["income"];
    assert_eq!(w["total"], "40");
    assert_eq!(
        w["minimum_percentages"],
        json!({"Mum": "20", "Dad": "10", "Ray": "10"})
    );
    assert_eq!(r["deductible"], false);
    assert_eq!(r["not_established"], json!([]));
}

#[test]
fn trust_loss_pattern_passes_only_with_both_kinds() {
    let income =
        json!([{"Mum": "60", "Dad": "10", "Ray": "30"}, {"Mum": "70", "Dad": "10", "Ray": "20"}]);
    let r = losses(merge(
        clean(),
        json!({"test_year_distributions": {"income": income}}),
    ));
    assert_eq!(
        r["pattern_of_distributions_working"]["income"]["total"],
        "90"
    );
    assert_eq!(test(&r, "pattern_of_distributions")["passed"], Value::Null);
    assert_eq!(r["deductible"], Value::Null);
    assert_eq!(r["not_established"], json!(["pattern_of_distributions"]));
    let capital = json!([{"Mum": "100"}]);
    let r = losses(merge(
        clean(),
        json!({"test_year_distributions": {"income": income, "capital": capital}}),
    ));
    assert_eq!(test(&r, "pattern_of_distributions")["passed"], true);
    assert_eq!(r["deductible"], true);
    // Exactly 50% is not greater than 50%; an individual absent from one distribution counts 0%.
    let r = losses(merge(
        clean(),
        json!({"test_year_distributions": {"income": income,
        "capital": [{"Mum": "50", "Dad": "50"}, {"Mum": "50"}]}}),
    ));
    assert_eq!(
        r["pattern_of_distributions_working"]["capital"]["total"],
        "50"
    );
    assert_eq!(r["deductible"], false);
}

#[test]
fn trust_loss_pattern_distribution_inputs_are_checked() {
    let e = losses_err(json!({"test_year_distributions": {"income": [{"A": "60", "B": "50"}]}}));
    assert!(e.contains("more than 100%"), "{e}");
    let e = losses_err(json!({"test_year_distributions": {"income": [{"A": "160"}]}}));
    assert!(e.contains("to A must be a percentage"), "{e}");
    assert!(
        losses_err(json!({"test_year_distributions": {"income": ["A"]}}))
            .contains("must be an object")
    );
    assert!(losses_err(json!({"test_year_distributions": {}})).contains("no income or capital"));
}

#[test]
fn trust_loss_pattern_applies_only_after_earlier_distributions() {
    let r = losses(merge(
        clean(),
        json!({"income_distributed_in_6_earlier_years": false,
        "pattern_of_distributions_test_passed": false}),
    ));
    assert_eq!(test(&r, "pattern_of_distributions")["applies"], false);
    assert_eq!(r["deductible"], true);
    let r = losses(merge(
        clean(),
        json!({"income_distributed_in_6_earlier_years": null,
        "pattern_of_distributions_test_passed": false}),
    ));
    assert_eq!(test(&r, "pattern_of_distributions")["applies"], Value::Null);
    assert_eq!(r["deductible"], Value::Null);
    assert_eq!(r["not_established"], json!(["pattern_of_distributions"]));
    // Capital distributed in both periods brings the test in on its own.
    let r = losses(merge(
        clean(),
        json!({"income_distributed_in_6_earlier_years": false,
        "capital_distributed_in_income_year_or_2_months_after": true,
        "capital_distributed_in_6_earlier_years": true}),
    ));
    assert_eq!(test(&r, "pattern_of_distributions")["applies"], true);
}

#[test]
fn trust_loss_non_fixed_conditions() {
    assert_eq!(losses(clean())["deductible"], true);
    let r = losses(merge(clean(), json!({"group_began_to_control": true})));
    assert_eq!(
        (
            test(&r, "control")["passed"].clone(),
            r["deductible"].clone()
        ),
        (json!(false), json!(false))
    );
    let r = losses(merge(
        clean(),
        json!({"previously_failed_pattern_test": true}),
    ));
    assert_eq!(r["deductible"], false);
    let r = losses(merge(
        clean(),
        json!({"individuals_had_over_50_percent_stake": true, "stake_maintained": false}),
    ));
    assert_eq!(r["deductible"], false);
    let r = losses(merge(
        clean(),
        json!({"individuals_had_over_50_percent_stake": true, "stake_maintained": true}),
    ));
    assert_eq!(r["deductible"], true);
    let r = losses(merge(
        clean(),
        json!({"individuals_had_over_50_percent_stake": true}),
    ));
    assert_eq!(r["not_established"], json!(["fifty_percent_stake"]));
    let r = losses(json!({}));
    assert_eq!(r["deductible"], Value::Null);
    assert_eq!(r["not_established"].as_array().unwrap().len(), 5);
    assert!(losses_err(json!({"group_began_to_control": "no"})).contains("true, false or null"));
    assert!(
        losses_err(json!({"capital_distributed_in_6_earlier_years": 1}))
            .contains("true, false or null")
    );
}

#[test]
fn trust_loss_fixed_trust_stake_or_alternative() {
    let fixed = json!({"trust_kind": "fixed", "income_injection_scheme": false});
    let r = losses(merge(
        fixed.clone(),
        json!({"fifty_percent_stake_test_passed": true}),
    ));
    assert_eq!(r["deductible"], true);
    assert_eq!(test(&r, "control")["applies"], false);
    assert_eq!(test(&r, "pattern_of_distributions")["applies"], false);
    for (alt, out) in [
        (json!(true), json!(true)),
        (json!(false), json!(false)),
        (Value::Null, Value::Null),
    ] {
        let r = losses(merge(
            fixed.clone(),
            json!({"fifty_percent_stake_test_passed": false, "alternative_stake_test_passed": alt}),
        ));
        assert_eq!(r["deductible"], out);
    }
    let r = losses(merge(fixed, json!({"alternative_stake_test_passed": true})));
    assert_eq!(r["deductible"], true);
}

#[test]
fn trust_loss_excepted_trusts() {
    let r = losses(json!({"excepted_trust": "family_trust", "income_injection_scheme": false}));
    assert_eq!(r["deductible"], true);
    assert_eq!(
        r["excepted_trust"],
        json!({"kind": "family_trust", "provision": "s 272-100(a)"})
    );
    for name in ["fifty_percent_stake", "pattern_of_distributions", "control"] {
        assert_eq!(test(&r, name)["applies"], false);
    }
    assert_eq!(test(&r, "income_injection")["applies"], true);
    let r = losses(json!({"excepted_trust": "family_trust", "income_injection_scheme": true}));
    assert_eq!(r["deductible"], false);
    let r = losses(json!({"excepted_trust": "designated_infrastructure_project_entity"}));
    assert_eq!(r["not_established"], json!(["income_injection"]));
    for kind in [
        "complying_superannuation_fund",
        "deceased_estate",
        "exempt_entity_unit_trust",
    ] {
        let r = losses(
            json!({"excepted_trust": kind, "trust_kind": "fixed", "income_injection_scheme": true}),
        );
        assert_eq!(test(&r, "income_injection")["applies"], false, "{kind}");
        assert_eq!(r["deductible"], true, "{kind}");
    }
    assert!(losses_err(json!({"excepted_trust": "charity"})).contains("excepted_trust must be"));
}

#[test]
fn trust_loss_refusals_and_years() {
    for kind in [
        "unlisted_widely_held",
        "listed_widely_held",
        "unlisted_very_widely_held",
        "wholesale_widely_held",
    ] {
        let e = losses_err(json!({"trust_kind": kind}));
        assert!(e.starts_with("refused:") && e.contains("266-C"), "{e}");
    }
    assert!(losses_err(json!({"trust_kind": "hybrid"})).contains("trust_kind must be"));
    assert!(losses_err(json!({"deduction": "bad_debt"})).contains("267-25"));
    assert!(losses_err(json!({"deduction": "current_year_loss"})).contains("Div 268"));
    assert!(losses_err(json!({"deduction": "other"})).contains("deduction must be"));
    assert!(losses_err(json!({"loss_year": 2027})).contains("must be earlier"));
    for fy in [2024, 2028] {
        assert!(
            losses_err(json!({"financial_year": fy, "loss_year": 2020})).contains("outside 2025")
        );
    }
    let r = losses(json!({"financial_year": 2025, "loss_year": 2024}));
    assert_eq!(r["not_decided"].as_array().unwrap().len(), 3);
}

#[test]
fn api_help_lists_the_new_commands() {
    for c in [
        "partnership-net-income",
        "trustee-assessment",
        "trust-loss-tests",
    ] {
        assert!(solomons_sword::api::COMMANDS.contains(&c));
    }
}
