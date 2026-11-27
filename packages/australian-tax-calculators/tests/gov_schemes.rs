// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Help to Buy, the Home Guarantee Scheme, the age pension and family tax
//! benefit Part A, read on 26 November 2026. The vectors are the sources' own
//! worked examples and published limits: the Help to Buy Program Directions
//! 2025 s 25(3) example; Services Australia's age pension cut-off points from
//! 20 September 2026; the FTB Part A income limits in 'A guide to Australian
//! Government payments' for 20 September to 31 December 2026. The rest are
//! made-up figures worked by hand from the provisions
//! (docs/calculation-evidence.md#government-schemes).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn run(cmd: &str, input: &Value) -> Value {
    api::evaluate(cmd, input).unwrap_or_else(|e| panic!("{cmd}: {e:?}"))
}

fn code(cmd: &str, input: &Value) -> &'static str {
    api::evaluate_coded(cmd, input).unwrap_err().code
}

fn with(mut base: Value, changes: Value) -> Value {
    for (k, v) in changes.as_object().unwrap() {
        base[k] = v.clone();
    }
    base
}

// ---- Help to Buy -----------------------------------------------------------

fn htb() -> Value {
    json!({"area": "vic_capital_regional", "property": "existing", "purchase_price": "700000",
           "agreed_percentage": "30", "deposit": "14000", "lender_valuation": "700000",
           "applicants": "single", "taxable_income": "95000", "application_date": "2026-08-03",
           "facts": {"citizen": true, "financial_capacity": true, "principal_residence": true,
                     "no_other_assistance": true, "no_property_interest": true, "costs_covered": true,
                     "sole_owners": true, "participating_jurisdiction": true},
           "year": "2026-27", "scope_confirmed": true})
}

#[test]
fn help_to_buy_follows_the_s_25_example() {
    // s 25(3) example: 30% of a $500,000 price, valued at $475,000 for
    // settlement, may be lifted to 31.58%, still worth $150,000; the increase
    // is Housing Australia's to make, so the agreed share stays beside it.
    let out = run(
        "help-to-buy",
        &with(
            htb(),
            json!({"purchase_price": "500000", "lender_valuation": "475000", "deposit": "10000"}),
        ),
    );
    assert_eq!(out["amounts"]["contribution"], "150000.00");
    assert_eq!(out["share_percentage_if_increased"], "31.58");
    assert_eq!(out["share_percentage"], "30");
    // s 16(2): 20% of $475,000 less the $10,000 deposit.
    assert_eq!(out["amounts"]["minimum_contribution"], "85000.00");
    assert_eq!(out["eligible"], true);
}

#[test]
fn help_to_buy_tests_each_limit() {
    let out = run("help-to-buy", &htb());
    assert_eq!(out["amounts"]["contribution"], "210000.00");
    assert_eq!(out["amounts"]["price_cap"], "950000.00");
    assert_eq!(out["amounts"]["minimum_deposit"], "14000.00");
    assert_eq!(out["share_percentage"], "30");
    assert_eq!(out["share_percentage_if_increased"], Value::Null);
    assert_eq!(out["eligible"], true);
    let failed =
        |changes: Value| run("help-to-buy", &with(htb(), changes))["conditions_not_met"].clone();
    assert_eq!(
        failed(json!({"area": "vic_other"})),
        json!(["price_within_cap"])
    );
    assert_eq!(
        failed(json!({"agreed_percentage": "30.01"})),
        json!(["percentage_within_limits"])
    );
    assert_eq!(
        failed(json!({"agreed_percentage": "40", "property": "new"})),
        json!([])
    );
    assert_eq!(
        failed(json!({"agreed_percentage": "4.99", "deposit": "200000"})),
        json!(["percentage_within_limits"])
    );
    assert_eq!(
        failed(json!({"deposit": "13999"})),
        json!(["deposit_at_least_2_percent"])
    );
    assert_eq!(
        failed(json!({"agreed_percentage": "10", "deposit": "69999"})),
        json!(["deposit_with_contribution_20_percent"])
    );
    assert_eq!(
        failed(json!({"taxable_income": "103001"})),
        json!(["income_test"])
    );
    assert_eq!(failed(json!({"taxable_income": "103000"})), json!([]));
    assert_eq!(
        failed(json!({"applicants": "single_parent", "taxable_income": "165000"})),
        json!([])
    );
    assert_eq!(
        failed(json!({"applicants": "joint", "taxable_income": "165001"})),
        json!(["income_test"])
    );
    // 2025-26: the unindexed $100,000 and $160,000 (s 8).
    let earlier = json!({"year": "2025-26", "application_date": "2026-06-30"});
    assert_eq!(
        failed(with(earlier.clone(), json!({"taxable_income": "100001"}))),
        json!(["income_test"])
    );
    assert_eq!(
        failed(with(
            earlier,
            json!({"applicants": "joint", "taxable_income": "160000"})
        )),
        json!([])
    );
}

#[test]
fn help_to_buy_refuses_what_it_cannot_work() {
    let refused = |changes: Value| code("help-to-buy", &with(htb(), changes));
    assert_eq!(
        refused(json!({"application_date": "2026-11-27"})),
        "help_to_buy.date_uncovered"
    );
    assert_eq!(
        refused(json!({"application_date": "2025-08-01"})),
        "help_to_buy.date_invalid"
    );
    assert_eq!(refused(json!({"area": "nsw"})), "help_to_buy.area_invalid");
    assert_eq!(
        refused(json!({"property": "land"})),
        "help_to_buy.property_invalid"
    );
    assert_eq!(
        refused(json!({"applicants": "couple"})),
        "help_to_buy.applicants_invalid"
    );
    assert_eq!(
        refused(json!({"agreed_percentage": "0"})),
        "help_to_buy.percentage_invalid"
    );
    assert_eq!(
        refused(json!({"purchase_price": "0"})),
        "help_to_buy.value_invalid"
    );
    assert_eq!(
        refused(json!({"deposit": "490001"})),
        "help_to_buy.deposit_exceeds_price"
    );
    assert_eq!(
        refused(json!({"facts": {"citizen": true}})),
        "help_to_buy.facts_not_stated"
    );
    assert_eq!(refused(json!({"facts": []})), "help_to_buy.facts_invalid");
    assert_eq!(
        refused(json!({"facts": {"citizen": "yes"}})),
        "input.flag_invalid"
    );
    assert_eq!(
        refused(json!({"facts": {"other": true}})),
        "input.field_unknown"
    );
}

fn review() -> Value {
    json!({"applicants": "single", "taxable_income": "110000", "previous_taxable_income": "104000",
           "share_percentage": "30", "property_value": "800000.00", "capacity_to_repay": true,
           "repayment_triggers_lmi": false,
           "year": "2026-27", "scope_confirmed": true})
}

#[test]
fn help_to_buy_review_requires_the_s_37_repayment() {
    let out = run("help-to-buy-review", &review());
    assert_eq!(out["amounts"]["commonwealth_share_value"], "240000.00");
    assert_eq!(out["amounts"]["income_threshold"], "103000.00");
    assert_eq!(out["amounts"]["previous_income_threshold"], "100000.00");
    assert_eq!(out["amounts"]["required_repayment"], "40000.00");
    assert_eq!(out["income_exceeded_both_years"], true);
    // Not over the earlier year's $100,000: no repayment for income.
    let under = run(
        "help-to-buy-review",
        &with(review(), json!({"previous_taxable_income": "100000"})),
    );
    assert_eq!(under["amounts"]["required_repayment"], "0.00");
    assert_eq!(under["amounts"]["minimum_voluntary_repayment"], "40000.00");
    // A share of 5% or less is repaid in full (s 37(8)).
    let small = run(
        "help-to-buy-review",
        &with(review(), json!({"share_percentage": "4"})),
    );
    assert_eq!(small["amounts"]["required_repayment"], "32000.00");
    // No capacity: a further assessment, nothing now (s 37(6)).
    let none = run(
        "help-to-buy-review",
        &with(review(), json!({"capacity_to_repay": false})),
    );
    assert_eq!(none["amounts"]["required_repayment"], "0.00");
    // Not required where it would bring lenders' mortgage insurance (s 37(7)
    // and (10)), for the 5% and the whole share alike.
    let lmi = run(
        "help-to-buy-review",
        &with(review(), json!({"repayment_triggers_lmi": true})),
    );
    assert_eq!(lmi["amounts"]["required_repayment"], "0.00");
    assert_eq!(lmi["amounts"]["minimum_voluntary_repayment"], "40000.00");
    assert!(
        lmi["outcome"]
            .as_str()
            .unwrap()
            .contains("s 37(7) and (10)")
    );
    let lmi_small = with(
        review(),
        json!({"repayment_triggers_lmi": true, "share_percentage": "4"}),
    );
    assert_eq!(
        run("help-to-buy-review", &lmi_small)["amounts"]["required_repayment"],
        "0.00"
    );
    // Rounded to the nearest $1,000: 5% of $812,345 is $40,617.25.
    let rounded = run(
        "help-to-buy-review",
        &with(review(), json!({"property_value": "812345.00"})),
    );
    assert_eq!(rounded["amounts"]["required_repayment"], "41000.00");
    // Joint, 2025-26: the joint threshold both years.
    let joint = json!({"applicants": "joint", "year": "2025-26", "taxable_income": "160001", "previous_taxable_income": "160001"});
    assert_eq!(
        run("help-to-buy-review", &with(review(), joint))["income_exceeded_both_years"],
        true
    );
}

#[test]
fn help_to_buy_review_refuses_what_it_cannot_work() {
    let refused = |changes: Value| code("help-to-buy-review", &with(review(), changes));
    assert_eq!(
        refused(json!({"property_value": "810000.00"})),
        "help_to_buy_review.rounding_midway"
    );
    assert_eq!(
        refused(json!({"share_percentage": "0"})),
        "help_to_buy_review.share_invalid"
    );
    assert_eq!(
        refused(json!({"share_percentage": "100.01"})),
        "help_to_buy_review.share_invalid"
    );
    assert_eq!(
        refused(json!({"property_value": "0"})),
        "help_to_buy_review.value_invalid"
    );
    assert_eq!(
        refused(json!({"applicants": "trust"})),
        "help_to_buy_review.applicants_invalid"
    );
    assert_eq!(
        refused(json!({"capacity_to_repay": "yes"})),
        "input.flag_invalid"
    );
    assert_eq!(
        refused(json!({"repayment_triggers_lmi": null})),
        "input.flag_invalid"
    );
    assert_eq!(
        refused(json!({"taxable_income": "100.50"})),
        "input.amount_not_whole_dollars"
    );
}

// ---- Home Guarantee Scheme -------------------------------------------------

fn hgs() -> Value {
    json!({"stream": "first_home", "area": "nsw_capital_regional", "issue_date": "2026-08-03",
           "property_value": "1500000.00", "loan_amount": "1425000.00", "borrowers": 2,
           "facts": {"no_property_10_years": true, "adult_citizen_or_resident": true, "eligible_lender": true,
                     "owner_occupied": true, "principal_and_interest": true, "term_30_years": true},
           "year": "2026-27", "scope_confirmed": true})
}

fn family_facts() -> Value {
    json!({"single_with_dependant": true, "property_interest_allowed": true, "no_relevant_interest": true,
           "adult_citizen_or_resident": true, "eligible_lender": true, "owner_occupied": true,
           "principal_and_interest": true, "term_30_years": true})
}

#[test]
fn home_guarantee_tests_cap_ratio_and_borrowers() {
    let out = run("home-guarantee", &hgs());
    assert_eq!(out["amounts"]["price_cap"], "1500000.00");
    assert_eq!(out["amounts"]["deposit"], "75000.00");
    assert_eq!(out["loan_to_value_ratio"], "95.00");
    assert_eq!(out["eligible"], true);
    let failed =
        |changes: Value| run("home-guarantee", &with(hgs(), changes))["conditions_not_met"].clone();
    assert_eq!(
        failed(json!({"property_value": "1500000.01", "loan_amount": "1400000.00"})),
        json!(["price_within_cap"])
    );
    assert_eq!(
        failed(json!({"loan_amount": "1200000.00"})),
        json!(["loan_to_value_ratio"])
    );
    assert_eq!(
        failed(json!({"loan_amount": "1425000.01"})),
        json!(["loan_to_value_ratio"])
    );
    assert_eq!(failed(json!({"borrowers": 3})), json!(["borrowers"]));
    let family = json!({"stream": "family_home", "facts": family_facts(), "borrowers": 1,
                        "property_value": "500000.00", "loan_amount": "490000.00", "area": "sa_other"});
    assert_eq!(failed(family.clone()), json!([]));
    assert_eq!(
        failed(with(family, json!({"borrowers": 2}))),
        json!(["borrowers"])
    );
}

#[test]
fn home_guarantee_splits_the_northern_territory_from_1_july_2026() {
    let nt = |date: &str, area: &str, year: &str| {
        with(
            hgs(),
            json!({"issue_date": date, "area": area, "year": year,
                           "property_value": "700000.00", "loan_amount": "630000.00"}),
        )
    };
    let before = run("home-guarantee", &nt("2026-06-30", "nt", "2025-26"));
    assert_eq!(before["amounts"]["price_cap"], "600000.00");
    assert_eq!(before["conditions_not_met"], json!(["price_within_cap"]));
    let after = run("home-guarantee", &nt("2026-07-01", "nt_capital", "2026-27"));
    assert_eq!(after["amounts"]["price_cap"], "750000.00");
    assert_eq!(after["eligible"], true);
    assert_eq!(
        code("home-guarantee", &nt("2026-07-01", "nt", "2026-27")),
        "home_guarantee.area_invalid"
    );
    assert_eq!(
        code("home-guarantee", &nt("2026-06-30", "nt_other", "2025-26")),
        "home_guarantee.area_invalid"
    );
}

#[test]
fn home_guarantee_refuses_what_it_cannot_work() {
    let refused = |changes: Value| code("home-guarantee", &with(hgs(), changes));
    // 'Between 80 and 95' before 1 July 2026 does not settle exactly 95.
    assert_eq!(
        refused(json!({"issue_date": "2026-03-02", "year": "2025-26"})),
        "home_guarantee.ratio_boundary"
    );
    assert_eq!(
        refused(json!({"issue_date": "2025-09-30", "year": "2025-26"})),
        "home_guarantee.date_uncovered"
    );
    assert_eq!(
        refused(json!({"issue_date": "2026-11-27"})),
        "home_guarantee.date_uncovered"
    );
    assert_eq!(
        refused(json!({"issue_date": "2025-12-01"})),
        "home_guarantee.date_invalid"
    );
    assert_eq!(
        refused(json!({"stream": "new_home"})),
        "home_guarantee.stream_invalid"
    );
    assert_eq!(
        refused(json!({"area": "nowhere"})),
        "home_guarantee.area_invalid"
    );
    assert_eq!(
        refused(json!({"property_value": "0.00"})),
        "home_guarantee.value_invalid"
    );
    assert_eq!(
        refused(json!({"borrowers": 0})),
        "home_guarantee.borrowers_invalid"
    );
    assert_eq!(
        refused(json!({"facts": {"eligible_lender": true}})),
        "home_guarantee.facts_not_stated"
    );
    assert_eq!(
        refused(json!({"facts": "yes"})),
        "home_guarantee.facts_invalid"
    );
    assert_eq!(
        refused(json!({"stream": "family_home"})),
        "input.field_unknown"
    );
}

// ---- Age pension -----------------------------------------------------------

fn pension(
    relationship: &str,
    homeowner: bool,
    income: &str,
    financial: &str,
    assets: &str,
) -> Value {
    let mut facts = json!({"in_australia": true, "not_blind": true, "not_transitional": true, "no_rent_assistance": true,
                           "underage_de_facto": false, "advance_deduction": false, "armed_services_payment": false, "remote_area": false, "pension_supplement_election": false, "quarterly_energy_supplement": false});
    if relationship == "couple" {
        facts["partner_pensioner"] = json!(true);
    }
    json!({"date": "2026-10-01", "relationship": relationship, "homeowner": homeowner,
           "ordinary_income": income, "financial_assets": financial, "assets": assets,
           "facts": facts, "year": "2026-27", "scope_confirmed": true})
}

fn rate(input: &Value) -> String {
    run("age-pension", input)["amounts"]["rate"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[test]
fn age_pension_meets_services_australias_cut_off_points() {
    // Maximum rates from 20 September 2026 (SS Guide 5.1.8.10).
    assert_eq!(
        rate(&pension("single", true, "0", "0", "333000")),
        "1237.70"
    );
    assert_eq!(rate(&pension("couple", true, "0", "0", "499000")), "933.00");
    // Assets cut-off points from 20 September 2026.
    assert_eq!(rate(&pension("single", true, "0", "0", "745750")), "0.00");
    assert_eq!(rate(&pension("single", true, "0", "0", "745500")), "0.20");
    assert_eq!(rate(&pension("single", false, "0", "0", "1012750")), "0.00");
    assert_eq!(rate(&pension("couple", true, "0", "0", "1121000")), "0.00");
    assert_eq!(rate(&pension("couple", true, "0", "0", "1120500")), "0.75");
    assert_eq!(rate(&pension("couple", false, "0", "0", "1388000")), "0.00");
    // Income cut-off points: $2,701.40 single, $4,128.00 a couple combined.
    assert_eq!(rate(&pension("single", true, "2701.40", "0", "0")), "0.00");
    assert_eq!(rate(&pension("single", true, "2701.00", "0", "0")), "0.20");
    assert_eq!(rate(&pension("couple", true, "4128.00", "0", "0")), "0.00");
    assert_eq!(rate(&pension("couple", true, "396.00", "0", "0")), "933.00");
}

#[test]
fn age_pension_deems_financial_assets() {
    // $66,800 at 1.75% and $33,200 at 3.75%: $2,414 a year, $92.846 a fortnight.
    let out = run(
        "age-pension",
        &pension("single", true, "200.00", "100000.00", "100000.00"),
    );
    assert_eq!(out["amounts"]["deemed_income"], "92.85");
    // ($200 + $92.846 - $226) x 0.5 = $33.423.
    assert_eq!(out["amounts"]["income_reduction"], "33.42");
    assert_eq!(out["amounts"]["rate"], "1204.28");
    // The 20 March 2026 rates (1.25% and 3.25%) on the 2025-26 threshold.
    let march = with(
        pension("single", true, "0", "64200.00", "64200.00"),
        json!({"date": "2026-04-01", "year": "2025-26"}),
    );
    let out = run("age-pension", &march);
    assert_eq!(out["amounts"]["deemed_income"], "30.87");
    assert_eq!(out["amounts"]["max_rate"], "1200.90");
}

#[test]
fn age_pension_deems_a_couples_combined_assets() {
    // $110,600 at 1.75% and $89,400 at 3.75%: $5,288 a year, $203.385 a
    // fortnight for the couple; with $300 ordinary income,
    // ($503.385 - $396) x 0.25 = $26.846 each.
    let out = run(
        "age-pension",
        &pension("couple", true, "300.00", "200000.00", "200000.00"),
    );
    assert_eq!(out["amounts"]["deemed_income"], "203.38");
    assert_eq!(out["amounts"]["income_reduction"], "26.85");
    assert_eq!(out["amounts"]["rate"], "906.15");
}

#[test]
fn age_pension_uses_each_periods_amounts() {
    let on = |date: &str, year: &str, assets: &str| {
        with(
            pension("single", true, "0", "0", assets),
            json!({"date": date, "year": year}),
        )
    };
    let first = run("age-pension", &on("2025-09-20", "2025-26", "321500"));
    assert_eq!(first["amounts"]["max_rate"], "1178.70");
    assert_eq!(first["amounts"]["rate"], "1178.70");
    // 2025-26's limit of $321,500: $250 over it costs $19.50 a year.
    assert_eq!(rate(&on("2026-06-30", "2025-26", "321750")), "1200.15");
    // 1 July 2026: the limit is $333,000 with the March rate.
    assert_eq!(rate(&on("2026-07-01", "2026-27", "333000")), "1200.90");
    assert_eq!(rate(&on("2027-03-19", "2026-27", "333000")), "1237.70");
}

#[test]
fn age_pension_refuses_what_it_cannot_work() {
    let base = pension("single", true, "0", "0", "0");
    let refused = |changes: Value| code("age-pension", &with(base.clone(), changes));
    assert_eq!(
        refused(json!({"date": "2025-09-19", "year": "2025-26"})),
        "age_pension.date_uncovered"
    );
    assert_eq!(
        refused(json!({"date": "2027-03-20"})),
        "age_pension.date_uncovered"
    );
    assert_eq!(
        refused(json!({"date": "2026-05-01"})),
        "age_pension.date_invalid"
    );
    assert_eq!(
        refused(json!({"relationship": "widowed"})),
        "age_pension.relationship_invalid"
    );
    assert_eq!(
        refused(json!({"financial_assets": "1.00"})),
        "age_pension.assets_inconsistent"
    );
    assert_eq!(
        refused(json!({"facts": {"in_australia": true}})),
        "age_pension.facts_not_stated"
    );
    assert_eq!(
        refused(json!({"relationship": "couple"})),
        "age_pension.facts_not_stated"
    );
    assert_eq!(refused(json!({"facts": 1})), "age_pension.facts_invalid");
    assert_eq!(refused(json!({"homeowner": "yes"})), "input.flag_invalid");
}

// ---- FTB Part A ------------------------------------------------------------

fn ftb(young: i64, older: i64, income: &str) -> Value {
    json!({"children_under_13": young, "children_13_plus": older, "adjusted_taxable_income": income,
           "income_support": false, "energy_supplement": false,
           "facts": {"whole_year": true, "no_maintenance": true, "no_rent_assistance": true,
                     "no_newborn_or_multiple_birth": true, "requirements_met": true},
           "year": "2026-27", "scope_confirmed": true})
}

fn part_a(input: &Value) -> String {
    run("ftb-part-a", input)["amounts"]["part_a_rate"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[test]
fn ftb_part_a_stops_at_the_published_income_limits() {
    // The income limits for 2026-27 in the 20 September 2026 payments guide:
    // (children under 13, aged 13 to 19, the limit).
    for (young, older, limit) in [
        (1, 0, 129_648),
        (2, 0, 136_218),
        (3, 0, 148_507),
        (0, 1, 129_648),
        (1, 1, 136_218),
        (2, 1, 154_675),
        (0, 2, 140_379),
    ] {
        let at = part_a(&ftb(young, older, &format!("{limit}.00")));
        let below = part_a(&ftb(young, older, &format!("{}.00", limit - 1)));
        assert_eq!(at, "0.00", "{young} {older} {limit}");
        assert_ne!(below, "0.00", "{young} {older} {limit}");
    }
}

#[test]
fn ftb_part_a_method_1_with_the_supplement() {
    // $6,139.30 + $970.90 less 20% of $10,869 over $69,131.
    let out = run("ftb-part-a", &ftb(1, 0, "80000.00"));
    assert_eq!(out["amounts"]["maximum_rate"], "7110.20");
    assert_eq!(out["amounts"]["base_rate"], "2941.90");
    assert_eq!(out["amounts"]["part_a_rate"], "4936.40");
    assert_eq!(out["supplement_included"], true);
    // A cent over $80,000: no supplement (clause 38A(1A)).
    assert_eq!(part_a(&ftb(1, 0, "80000.01")), "3965.50");
    // Up to the lower free area, the maximum rate.
    assert_eq!(part_a(&ftb(1, 1, "69131.00")), "16070.95");
    // Floored at the base rate below the higher free area.
    assert_eq!(part_a(&ftb(1, 0, "123078.00")), "1971.00");
    // 2025-26: $5,927.60 + $938.05.
    assert_eq!(
        part_a(&with(ftb(1, 0, "0.00"), json!({"year": "2025-26"}))),
        "6865.65"
    );
    // Energy supplement kept: + $91.25.
    assert_eq!(
        part_a(&with(ftb(1, 0, "0.00"), json!({"energy_supplement": true}))),
        "7201.45"
    );
    // A pension or benefit recipient: the maximum rate at any income.
    assert_eq!(
        part_a(&with(
            ftb(1, 0, "200000.00"),
            json!({"income_support": true})
        )),
        "6139.30"
    );
}

#[test]
fn ftb_part_a_method_2_takes_the_greater() {
    // 3 under 13 at $140,000: Method 1 continued gives $18,417.90 - $10,789.40
    // - $5,076.60 = $2,551.90; the base rate less 30% gives $5,913 - $5,076.60.
    let out = run("ftb-part-a", &ftb(3, 0, "140000.00"));
    assert_eq!(out["amounts"]["provisional_rate"], "836.40");
    assert_eq!(out["amounts"]["method_1_rate"], "2551.90");
    assert_eq!(out["amounts"]["part_a_rate"], "2551.90");
}

#[test]
fn ftb_part_a_refuses_what_it_cannot_work() {
    let refused = |input: Value| code("ftb-part-a", &input);
    assert_eq!(refused(ftb(0, 0, "0.00")), "ftb_part_a.children_invalid");
    assert_eq!(refused(ftb(21, 0, "0.00")), "ftb_part_a.children_invalid");
    assert_eq!(refused(ftb(20, 1, "0.00")), "ftb_part_a.children_invalid");
    assert_ne!(part_a(&ftb(10, 10, "0.00")), "0.00");
    assert_eq!(
        refused(with(ftb(1, 0, "0.00"), json!({"children_13_plus": "1"}))),
        "ftb_part_a.children_invalid"
    );
    assert_eq!(
        refused(with(
            ftb(1, 0, "0.00"),
            json!({"facts": {"whole_year": true}})
        )),
        "ftb_part_a.facts_not_stated"
    );
    assert_eq!(
        refused(with(ftb(1, 0, "0.00"), json!({"facts": null}))),
        "ftb_part_a.facts_invalid"
    );
    assert_eq!(
        refused(with(ftb(1, 0, "0.00"), json!({"year": "2027-28"}))),
        "period.unsupported"
    );
}
