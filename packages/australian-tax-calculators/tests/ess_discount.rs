// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The employee share scheme discount (ITAA 1997 Division 83A, compilations
//! 210, 234 and 268, and the Income Tax Assessment (1997 Act) Regulations 2021
//! ss 83A-315.01 to 83A-315.09, compilation 16, read on 22 November 2026)
//! against the ATO's published examples, each table entry of the regulations
//! and hand-worked cases at each edge
//! (docs/calculation-evidence.md#ess-discount).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn run(input: &Value) -> Value {
    api::evaluate("ess-discount", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("ess-discount", input).unwrap_err().code
}

fn call(year: &str, interests: Value, income_test: Value) -> Value {
    json!({
        "year": year, "scope_confirmed": true, "interests": interests,
        "income_test": income_test, "foreign_employment": false,
    })
}

/// Adjusted taxable income of `taxable`, nothing else added.
fn income(taxable: &str) -> Value {
    json!({
        "taxable_income": taxable, "reportable_fringe_benefits_total": "0.00",
        "reportable_super_contributions": "0.00", "total_net_investment_loss": "0.00",
    })
}

/// Every condition met, no real risk of forfeiture: taxed upfront and
/// eligible for the reduction.
fn conditions(kind: &str) -> Value {
    let mut c = json!({
        "employed": true, "ordinary_shares_only": true, "not_share_trading_company": true,
        "minimum_holding_period": true, "ten_percent_limit": true, "non_discriminatory": true,
        "real_risk_of_forfeiture": false, "salary_sacrifice": false,
    });
    if kind == "share" {
        c["broad_availability"] = json!(true);
    } else {
        c["deferral_stated"] = json!(false);
    }
    c
}

fn share(acquired: &str, consideration: &str, market_value: &str) -> Value {
    json!({
        "kind": "share", "acquired": acquired, "consideration": consideration,
        "market_value": market_value, "conditions": conditions("share"), "start_up": null,
    })
}

/// Rights valued under the regulations, no consideration paid.
fn rights(acquired: &str, exercise: &str, shares: &str, last: &str) -> Value {
    json!({
        "kind": "right", "acquired": acquired, "consideration": "0.00",
        "exercise_price": exercise, "share_market_value": shares,
        "last_exercise_day": last, "valuation": "regulations",
        "conditions": conditions("right"), "start_up": null,
    })
}

fn set(mut base: Value, changes: Value) -> Value {
    for (key, value) in changes.as_object().unwrap() {
        if value == "remove" {
            base.as_object_mut().unwrap().remove(key);
        } else {
            base[key] = value.clone();
        }
    }
    base
}

fn with_conditions(mut interest: Value, changes: Value) -> Value {
    let merged = set(interest["conditions"].clone(), changes);
    interest["conditions"] = merged;
    interest
}

fn amount<'a>(result: &'a Value, key: &str) -> &'a str {
    result["amounts"][key].as_str().unwrap()
}

fn row(result: &Value, index: usize) -> &Value {
    &result["interests"][index]
}

/// Lester (the ATO's '$1,000 reduction' example 1): a $1,500 discount and
/// $91,000 of adjusted taxable income leave $500; the date is moved into
/// 2025-26.
#[test]
fn lester_keeps_500_after_the_reduction() {
    let interests = json!([share("2025-07-29", "1500.00", "3000.00")]);
    let r = run(&call("2025-26", interests, income("91000.00")));
    assert_eq!(row(&r, 0)["treatment"], "upfront_eligible");
    assert_eq!(amount(&r, "upfront_eligible_discounts"), "1500.00");
    assert_eq!(amount(&r, "reduction"), "1000.00");
    assert_eq!(amount(&r, "assessable_discount"), "500.00");
    assert_eq!(amount(&r, "adjusted_taxable_income"), "91000.00");
}

/// Jane (example 2): $200,000 of adjusted taxable income, no reduction.
#[test]
fn jane_over_the_income_test_keeps_the_whole_discount() {
    let interests = json!([share("2025-07-03", "2700.00", "4000.00")]);
    let r = run(&call("2025-26", interests, income("200000.00")));
    assert_eq!(amount(&r, "reduction"), "0.00");
    assert_eq!(amount(&r, "assessable_discount"), "1300.00");
}

/// s 83A-35(2)(b): $180,000 or less; each of the 4 amounts counts.
#[test]
fn the_income_test_boundary_and_its_parts() {
    let interests = json!([share("2025-07-03", "2700.00", "4000.00")]);
    let at = run(&call("2025-26", interests.clone(), income("180000.00")));
    assert_eq!(amount(&at, "reduction"), "1000.00");
    let over = run(&call("2025-26", interests.clone(), income("180000.01")));
    assert_eq!(amount(&over, "reduction"), "0.00");
    let parts = [
        "reportable_fringe_benefits_total",
        "reportable_super_contributions",
        "total_net_investment_loss",
    ];
    for key in parts {
        let mut test = income("179999.99");
        test[key] = json!("0.02");
        let r = run(&call("2025-26", interests.clone(), test));
        assert_eq!(amount(&r, "reduction"), "0.00", "{key}");
    }
}

/// s 83A-35(2)(a): the year's eligible discounts together, at most $1,000;
/// an interest failing s 83A-35(6) goes to label E without reduction, and an
/// acquisition in another year adds nothing.
#[test]
fn the_reduction_is_one_1000_for_the_year() {
    let other = with_conditions(
        share("2025-09-01", "0.00", "400.00"),
        json!({"non_discriminatory": false}),
    );
    let interests = json!([
        share("2025-08-01", "400.00", "1000.00"),
        share("2026-02-01", "300.00", "1000.00"),
        other,
        share("2024-08-01", "0.00", "900.00"),
    ]);
    let r = run(&call("2025-26", interests, income("90000.00")));
    assert_eq!(amount(&r, "upfront_eligible_discounts"), "1300.00");
    assert_eq!(amount(&r, "upfront_other_discounts"), "400.00");
    assert_eq!(amount(&r, "reduction"), "1000.00");
    assert_eq!(amount(&r, "assessable_discount"), "700.00");
    assert_eq!(row(&r, 2)["treatment"], "upfront");
    assert_eq!(row(&r, 3)["income_year"], "2024-25");
    assert_eq!(row(&r, 3)["amount_in_year"], "0.00");
}

/// No eligible discount needs no income test; one does.
#[test]
fn the_income_test_is_needed_only_for_the_reduction() {
    let other = with_conditions(
        share("2025-09-01", "0.00", "400.00"),
        json!({"minimum_holding_period": false}),
    );
    let r = run(&call("2025-26", json!([other]), Value::Null));
    assert_eq!(amount(&r, "assessable_discount"), "400.00");
    let eligible = json!([share("2025-09-01", "0.00", "400.00")]);
    let missing = call("2025-26", eligible.clone(), Value::Null);
    assert_eq!(code(&missing), "input.missing_field");
    let bad = call("2025-26", eligible, json!("90000"));
    assert_eq!(code(&bad), "ess_discount.income_test_invalid");
}

fn start_up(turnover: &str) -> Value {
    json!({
        "unlisted": true, "incorporated_under_10_years": true,
        "aggregated_turnover": turnover, "employer_australian_resident": true,
    })
}

/// Sarah (the ATO's start-up concession example): 2,500 shares worth $1.00
/// bought at $0.85, a 15% discount, are reduced to nil; a cent more is over
/// 15% (s 83A-33(5)(a)).
#[test]
fn sarah_at_15_percent_is_a_start_up() {
    let sarah = set(
        share("2025-08-01", "2125.00", "2500.00"),
        json!({"start_up": start_up("5900000.00")}),
    );
    let r = run(&call("2025-26", json!([sarah.clone()]), Value::Null));
    assert_eq!(row(&r, 0)["treatment"], "start_up");
    assert_eq!(amount(&r, "start_up_discounts"), "375.00");
    assert_eq!(amount(&r, "assessable_discount"), "0.00");
    let over = set(sarah, json!({"consideration": "2124.99"}));
    let r = run(&call("2025-26", json!([over]), income("80000.00")));
    assert_eq!(row(&r, 0)["treatment"], "upfront_eligible");
    assert_eq!(amount(&r, "upfront_eligible_discounts"), "375.01");
}

fn treatment(interest: Value) -> Value {
    let r = run(&call("2025-26", json!([interest]), income("1.00")));
    r["interests"][0]["treatment"].clone()
}

/// s 83A-33(4) at $50 million, (2), (3), (6), s 83A-45 and, for a share,
/// s 83A-105(2) each decide.
#[test]
fn each_start_up_condition_decides() {
    let base = set(
        share("2025-08-01", "900.00", "1000.00"),
        json!({"start_up": start_up("50000000.00")}),
    );
    assert_eq!(treatment(base.clone()), "start_up");
    let over = set(base.clone(), json!({"start_up": start_up("50000000.01")}));
    assert_eq!(treatment(over), "upfront_eligible");
    let facts = [
        "unlisted",
        "incorporated_under_10_years",
        "employer_australian_resident",
    ];
    for key in facts {
        let mut failed = start_up("1.00");
        failed[key] = json!(false);
        let interest = set(base.clone(), json!({"start_up": failed}));
        assert_eq!(treatment(interest), "upfront_eligible", "{key}");
    }
    let held = with_conditions(base.clone(), json!({"minimum_holding_period": false}));
    assert_eq!(treatment(held), "upfront");
    let narrow = with_conditions(base, json!({"broad_availability": false}));
    assert_eq!(treatment(narrow), "upfront_eligible");
}

/// s 83A-33(5)(b): a right's exercise price at least the share's market value.
#[test]
fn a_start_up_right_needs_an_exercise_price_at_market_value() {
    let right = |exercise: &str| {
        json!({
            "kind": "right", "acquired": "2025-08-01", "consideration": "0.00",
            "exercise_price": exercise, "share_market_value": "10000.00",
            "last_exercise_day": "2030-08-01", "valuation": "market_value",
            "market_value": "800.00", "conditions": conditions("right"),
            "start_up": start_up("1000000.00"),
        })
    };
    assert_eq!(treatment(right("10000.00")), "start_up");
    let under = run(&call("2025-26", json!([right("9999.99")]), income("1.00")));
    assert_eq!(row(&under, 0)["treatment"], "upfront_eligible");
    assert_eq!(amount(&under, "upfront_eligible_discounts"), "800.00");
}

/// Barry (the ATO's unlisted rights example): 200,000 rights at 10 cents over
/// shares worth 12 cents, 48 months to the last exercise day: 120%, the 36 to
/// 48 month row (the lower range), 11.4% + 10 x 0.8% = 19.4%, $3,880 under the
/// tables and $4,000 intrinsic. A day more is the 48 to 60 month row.
#[test]
fn barry_takes_the_intrinsic_value() {
    let barry = rights("2023-03-26", "20000.00", "24000.00", "2027-03-26");
    let r = run(&call(
        "2022-23",
        json!([barry.clone()]),
        income("100000.00"),
    ));
    let v = &row(&r, 0)["valuation"];
    assert_eq!(v["rule"], "s 83A-315.09");
    assert_eq!(v["exercise_period_months_up_to"], 48);
    assert_eq!(v["table_value"], "3880.00");
    assert_eq!(v["value"], "4000.00");
    assert_eq!(amount(&r, "upfront_eligible_discounts"), "4000.00");
    assert_eq!(amount(&r, "assessable_discount"), "3000.00");
    let later = set(barry, json!({"last_exercise_day": "2027-03-27"}));
    let r = run(&call("2022-23", json!([later]), income("100000.00")));
    // 12.1% + 10 x 0.7% = 19.1% of $20,000.
    assert_eq!(row(&r, 0)["valuation"]["table_value"], "3820.00");
}

fn valued(exercise: &str, shares: &str, last: &str) -> Value {
    let interests = json!([rights("2023-01-10", exercise, shares, last)]);
    let r = run(&call("2022-23", interests, income("1.00")));
    r["interests"][0]["valuation"].clone()
}

/// The lower range on both edges (s 83A-315.08(2)), 50% and 110% exactly,
/// under 50%, a fraction of excess and a nil exercise price (s 83A-315.03).
#[test]
fn the_regulations_edges() {
    // 100% and exactly 24 months: item 33 (18 to 24), column 97.5 to 100.
    assert_eq!(
        valued("10000.00", "10000.00", "2025-01-10")["table_value"],
        "440.00"
    );
    // 110%: s 83A-315.09 with no excess, 9.9% of $10,000; intrinsic $1,000.
    let at = valued("10000.00", "11000.00", "2025-01-10");
    assert_eq!(at["rule"], "s 83A-315.09");
    assert_eq!(at["table_value"], "990.00");
    assert_eq!(at["value"], "1000.00");
    // 50%: the first column, 0.0% at 24 months; under 50%: nil.
    assert_eq!(
        valued("10000.00", "5000.00", "2025-01-10")["rule"],
        "s 83A-315.08"
    );
    let under = valued("10000.00", "4999.99", "2025-01-10");
    assert_eq!(under["rule"], "s 83A-315.05 step 2");
    // 125.5%: the excess disregards the fraction, 9.9% + 15 x 0.8% = 21.9%.
    assert_eq!(
        valued("10000.00", "12550.00", "2025-01-10")["table_value"],
        "2190.00"
    );
    assert_eq!(valued("0.00", "7000.00", "2025-01-10")["value"], "7000.00");
}

/// The same day `months` later, for days that exist in every month.
fn months_after(day: &str, months: i64) -> String {
    let (y, m): (i64, i64) = (day[..4].parse().unwrap(), day[5..7].parse().unwrap());
    let index = y * 12 + (m - 1) + months;
    format!("{}-{:02}-{}", index / 12, index % 12 + 1, &day[8..])
}

/// s 83A-315.08(1): every entry of items 1 to 38, in the regulation's own
/// layout, at a calculation percentage inside each column and a period inside
/// each row.
#[test]
fn every_table_entry_of_section_83a_315_08() {
    let lower = [55.0, 65.0, 72.5, 77.5, 82.5, 87.5, 91.25];
    let upper = [93.75, 96.25, 98.75, 101.25, 103.75, 106.25, 108.75];
    for (item, period, entries) in ITEMS {
        let top: i64 = period.split(" to ").nth(1).unwrap().parse().unwrap();
        let last = months_after("2023-01-10", top - 1);
        let middles = if item <= 19 { lower } else { upper };
        for (middle, entry) in middles.iter().zip(entries) {
            let shares = format!("{:.2}", middle * 1000.0);
            let v = valued("100000.00", &shares, &last);
            let expected = format!("{:.2}", entry.parse::<f64>().unwrap() * 1000.0);
            assert_eq!(v["table_value"], expected, "item {item} at {middle}%");
        }
    }
}

/// s 83A-315.09(2): each row's base and additional percentages at 120%.
#[test]
fn every_base_percentage_of_section_83a_315_09() {
    for (period, base, additional) in BASE {
        let top: i64 = period.split(" to ").nth(1).unwrap().parse().unwrap();
        let last = months_after("2023-01-10", top - 1);
        let v = valued("100000.00", "120000.00", &last);
        let rate = base.parse::<f64>().unwrap() + 10.0 * additional.parse::<f64>().unwrap();
        assert_eq!(
            v["table_value"],
            format!("{:.2}", rate * 1000.0),
            "{period}"
        );
    }
}

/// The regulations value a right only from 1 July 2021 and within 15 years
/// of acquisition (s 83A-315.01(1), s 1000-1.02).
#[test]
fn the_regulations_refuse_what_they_do_not_cover() {
    let early = json!([rights("2021-06-30", "100.00", "120.00", "2024-06-30")]);
    assert_eq!(
        code(&call("2020-21", early, Value::Null)),
        "ess_discount.valuation_unavailable"
    );
    let long = json!([rights("2025-07-01", "100.00", "120.00", "2040-07-02")]);
    assert_eq!(
        code(&call("2025-26", long, Value::Null)),
        "ess_discount.valuation_unavailable"
    );
    let at = json!([rights("2025-07-01", "100.00", "120.00", "2040-07-01")]);
    let r = run(&call("2025-26", at, income("1.00")));
    assert_eq!(row(&r, 0)["valuation"]["exercise_period_months_up_to"], 180);
}

/// Shares at a real risk of forfeiture (s 83A-105(3)), acquired after
/// 30 June 2022, with the deferral facts given.
fn deferred_share(lifted: &str, disposed: Value, value: Value) -> Value {
    let mut s = with_conditions(
        share("2022-07-23", "0.00", "1000.00"),
        json!({"real_risk_of_forfeiture": true}),
    );
    s["restrictions_lifted"] = json!(lifted);
    s["disposed"] = disposed;
    s["cost_base"] = json!("0.00");
    s["taxing_point_value"] = value;
    s
}

/// The 30-day rule (s 83A-115(3)): restrictions lift on 23 June and the
/// shares are sold for $1,518 on 20 July, within 30 days, so the taxing point
/// is the sale, in the next income year.
#[test]
fn a_sale_within_30_days_moves_the_taxing_point() {
    let value = json!({"date": "2025-07-20", "market_value": "1518.00"});
    let sold = deferred_share("2025-06-23", json!("2025-07-20"), value);
    let r = run(&call("2025-26", json!([sold.clone()]), Value::Null));
    let w = row(&r, 0);
    assert_eq!(w["treatment"], "deferred");
    assert_eq!(w["rule"], "s 83A-105(3)");
    assert_eq!(w["deferred_taxing_point"], "2025-07-20");
    assert_eq!(w["taxing_point_rule"], "s 83A-115(3)");
    assert_eq!(amount(&r, "deferred_amounts"), "1518.00");
    assert_eq!(amount(&r, "assessable_discount"), "1518.00");
    let before = run(&call("2024-25", json!([sold]), Value::Null));
    assert_eq!(amount(&before, "deferred_amounts"), "0.00");
    assert_eq!(row(&before, 0)["income_year"], "2025-26");
}

/// 30 days after is within the rule; 31 is not, and the value must then be
/// the one on the day restrictions lifted.
#[test]
fn the_30_day_rule_edge() {
    let at = json!({"date": "2025-07-23", "market_value": "1500.00"});
    let r = run(&call(
        "2025-26",
        json!([deferred_share("2025-06-23", json!("2025-07-23"), at)]),
        Value::Null,
    ));
    assert_eq!(row(&r, 0)["deferred_taxing_point"], "2025-07-23");
    let stale = json!({"date": "2025-07-24", "market_value": "1500.00"});
    let late = call(
        "2025-26",
        json!([deferred_share("2025-06-23", json!("2025-07-24"), stale)]),
        Value::Null,
    );
    assert_eq!(code(&late), "ess_discount.taxing_point_value");
    let lifted = json!({"date": "2025-06-23", "market_value": "1400.00"});
    let r = run(&call(
        "2024-25",
        json!([deferred_share("2025-06-23", json!("2025-07-24"), lifted)]),
        Value::Null,
    ));
    assert_eq!(row(&r, 0)["taxing_point_rule"], "s 83A-115(4)");
    assert_eq!(amount(&r, "deferred_amounts"), "1400.00");
}

/// s 83A-110(1): the cost base comes off, not below nil; a taxing point in
/// the year needs its value.
#[test]
fn the_deferred_amount_less_cost_base() {
    let value = json!({"date": "2025-06-23", "market_value": "1400.00"});
    let base = set(
        deferred_share("2025-06-23", Value::Null, value.clone()),
        json!({"cost_base": "400.00"}),
    );
    let r = run(&call("2024-25", json!([base]), Value::Null));
    assert_eq!(amount(&r, "deferred_amounts"), "1000.00");
    let under = set(
        deferred_share("2025-06-23", Value::Null, value),
        json!({"cost_base": "1500.00"}),
    );
    let r = run(&call("2024-25", json!([under]), Value::Null));
    assert_eq!(amount(&r, "deferred_amounts"), "0.00");
    let missing = deferred_share("2025-06-23", Value::Null, Value::Null);
    assert_eq!(
        code(&call("2024-25", json!([missing.clone()]), Value::Null)),
        "ess_discount.taxing_point_value"
    );
    let r = run(&call("2023-24", json!([missing]), Value::Null));
    assert_eq!(row(&r, 0)["income_year"], "2024-25");
}

/// Rights at a real risk of forfeiture acquired on 1 August 2020, for the
/// cessation of employment rule, valued on `value_on`.
fn early_right(ended: Value, value_on: &str) -> Value {
    let mut c = conditions("right");
    c["real_risk_of_forfeiture"] = json!(true);
    json!({
        "kind": "right", "acquired": "2020-08-01", "consideration": "0.00",
        "exercise_price": "0.00", "share_market_value": "3000.00",
        "last_exercise_day": "2030-08-01", "valuation": "market_value",
        "market_value": "1000.00", "conditions": c, "start_up": null,
        "restrictions_lifted": "2023-08-01", "exercised": null,
        "share_restrictions_lifted": null, "employment_ended": ended, "disposed": null,
        "cost_base": "0.00",
        "taxing_point_value": {"date": value_on, "market_value": "2500.00"},
    })
}

/// Ending employment before 1 July 2022 was a taxing point (former
/// s 83A-120(5)); from then it is not (Act No. 8, 2022 Sch 10 item 9).
#[test]
fn cessation_counts_only_before_1_july_2022() {
    let r = run(&call(
        "2021-22",
        json!([early_right(json!("2022-03-01"), "2022-03-01")]),
        Value::Null,
    ));
    assert_eq!(row(&r, 0)["taxing_point_rule"], "former s 83A-120(5)");
    assert_eq!(amount(&r, "deferred_amounts"), "2500.00");
    let r = run(&call(
        "2023-24",
        json!([early_right(json!("2022-07-01"), "2023-08-01")]),
        Value::Null,
    ));
    assert_eq!(row(&r, 0)["taxing_point_rule"], "s 83A-120(4)");
    assert_eq!(amount(&r, "deferred_amounts"), "2500.00");
    let mut silent = early_right(Value::Null, "2023-08-01");
    silent.as_object_mut().unwrap().remove("employment_ended");
    let refused = call("2023-24", json!([silent]), Value::Null);
    assert_eq!(code(&refused), "ess_discount.employment_end_not_stated");
}

/// From 1 July 2022 employment_ended is not a fact the command takes.
#[test]
fn a_later_interest_takes_no_cessation_date() {
    let value = json!({"date": "2025-06-23", "market_value": "1400.00"});
    let s = set(
        deferred_share("2025-06-23", Value::Null, value),
        json!({"employment_ended": null}),
    );
    assert_eq!(
        code(&call("2024-25", json!([s]), Value::Null)),
        "input.field_unknown"
    );
}

/// s 83A-120(7): a right exercised before its restrictions lift is taxed
/// when the share's lift; (4) needs the right unexercised.
#[test]
fn an_exercised_right_waits_for_the_share() {
    let mut r = early_right(Value::Null, "2024-01-01");
    r["restrictions_lifted"] = json!("2023-06-01");
    r["exercised"] = json!("2023-01-01");
    r["share_restrictions_lifted"] = json!("2024-01-01");
    let out = run(&call("2023-24", json!([r]), Value::Null));
    assert_eq!(row(&out, 0)["deferred_taxing_point"], "2024-01-01");
    assert_eq!(row(&out, 0)["taxing_point_rule"], "s 83A-120(7)");
}

/// s 83A-120(6): with nothing else, 15 years from acquisition, ending the day
/// before the anniversary.
#[test]
fn fifteen_years_is_the_last_point() {
    let mut r = early_right(Value::Null, "2035-07-31");
    r["restrictions_lifted"] = Value::Null;
    r["taxing_point_value"] = Value::Null;
    let out = run(&call("2025-26", json!([r]), Value::Null));
    assert_eq!(row(&out, 0)["deferred_taxing_point"], "2035-07-31");
    assert_eq!(row(&out, 0)["income_year"], "2035-36");
    assert_eq!(amount(&out, "deferred_amounts"), "0.00");
}

/// s 83A-105(6): a right whose rules state Subdivision 83A-C applies and
/// restrict disposal is deferred; the regulations may value it at the taxing
/// point but not on a disposal (s 83A-315.01(2)).
#[test]
fn a_stated_deferral_right_valued_at_the_taxing_point() {
    let mut r = rights("2022-08-01", "10000.00", "8000.00", "2030-08-01");
    r["conditions"]["deferral_stated"] = json!(true);
    r["restrictions_lifted"] = json!("2024-08-01");
    r["exercised"] = Value::Null;
    r["share_restrictions_lifted"] = Value::Null;
    r["disposed"] = Value::Null;
    r["cost_base"] = json!("0.00");
    r["taxing_point_value"] = json!({"date": "2024-08-01", "share_market_value": "12000.00"});
    let out = run(&call("2024-25", json!([r.clone()]), Value::Null));
    assert_eq!(row(&out, 0)["rule"], "s 83A-105(6)");
    // 120%, 72 months (60 to 72): 12.5% + 10 x 0.7% = 19.5% of $10,000.
    assert_eq!(
        row(&out, 0)["valuation_at_taxing_point"]["table_value"],
        "1950.00"
    );
    assert_eq!(amount(&out, "deferred_amounts"), "2000.00");
    r["disposed"] = json!("2024-08-10");
    r["taxing_point_value"] = json!({"date": "2024-08-10", "share_market_value": "12000.00"});
    assert_eq!(
        code(&call("2024-25", json!([r]), Value::Null)),
        "ess_discount.valuation_unavailable"
    );
}

/// Salary sacrificed shares with no real risk: deferred under s 83A-105(4)
/// up to $5,000 of market value in the year (s 83A-105(4)(c)), taxed upfront
/// above it.
#[test]
fn salary_sacrifice_up_to_5000() {
    let sacrificed = |value: &str| {
        let mut s = with_conditions(
            share("2025-08-01", "0.00", value),
            json!({"salary_sacrifice": true}),
        );
        s["restrictions_lifted"] = json!("2027-08-01");
        s["disposed"] = Value::Null;
        s["cost_base"] = json!("0.00");
        s["taxing_point_value"] = Value::Null;
        s
    };
    let r = run(&call(
        "2025-26",
        json!([sacrificed("2500.00"), sacrificed("2500.00")]),
        Value::Null,
    ));
    assert_eq!(row(&r, 0)["rule"], "s 83A-105(4)");
    assert_eq!(amount(&r, "assessable_discount"), "0.00");
    let mut upfront = sacrificed("2500.01");
    for key in [
        "restrictions_lifted",
        "disposed",
        "cost_base",
        "taxing_point_value",
    ] {
        upfront.as_object_mut().unwrap().remove(key);
    }
    let mut first = sacrificed("2500.00");
    for key in [
        "restrictions_lifted",
        "disposed",
        "cost_base",
        "taxing_point_value",
    ] {
        first.as_object_mut().unwrap().remove(key);
    }
    let r = run(&call("2025-26", json!([first, upfront]), income("1.00")));
    assert_eq!(row(&r, 1)["treatment"], "upfront_eligible");
    assert_eq!(amount(&r, "upfront_eligible_discounts"), "5000.01");
    let mut risky = with_conditions(
        sacrificed("100.00"),
        json!({"real_risk_of_forfeiture": true}),
    );
    risky["acquired"] = json!("2025-09-01");
    let mixed = call(
        "2025-26",
        json!([sacrificed("2500.00"), risky]),
        Value::Null,
    );
    assert_eq!(code(&mixed), "ess_discount.salary_sacrifice_unclear");
}

fn forfeited(kind: &str, cause: &str, protects: bool) -> Value {
    json!({
        "kind": kind, "acquired": "2022-09-01",
        "forfeiture": {
            "cause": cause, "protects_against_fall": protects,
            "consideration_received": false, "amount_included": "2400.00", "year_included": "2024-25",
        },
    })
}

/// s 83A-310 and the ATO's forfeiture examples 1 (Conner, a hurdle not met),
/// 7 and 9 (rights left to lapse, from 1 July 2015) and 10 (Remy, a market
/// value condition).
#[test]
fn forfeiture_excludes_the_amount_included() {
    let interests = json!([
        forfeited("share", "scheme_condition", false),
        forfeited("right", "lapsed_unexercised", false),
        forfeited("right", "scheme_condition", true),
        forfeited("share", "other_choice", false),
        forfeited("right", "cancellation_allowed", false),
        forfeited("share", "ceased_employment", false),
    ]);
    let r = run(&call("2025-26", interests, Value::Null));
    let excluded: Vec<bool> = (0..6)
        .map(|i| row(&r, i)["excluded"].as_bool().unwrap())
        .collect();
    assert_eq!(excluded, [true, true, false, false, true, true]);
    assert_eq!(r["forfeiture_exclusions"]["2024-25"], "9600.00");
    assert_eq!(amount(&r, "assessable_discount"), "0.00");
    let lapsed_share = call(
        "2025-26",
        json!([forfeited("share", "lapsed_unexercised", false)]),
        Value::Null,
    );
    assert_eq!(code(&lapsed_share), "ess_discount.interests_invalid");
    // Example 11: rights cancelled for their market value are disposed of,
    // not forfeited.
    let mut cashed = forfeited("right", "cancellation_allowed", false);
    cashed["forfeiture"]["consideration_received"] = json!(true);
    let r = run(&call("2025-26", json!([cashed]), Value::Null));
    assert_eq!(row(&r, 0)["excluded"], false);
    let mut early = forfeited("share", "scheme_condition", false);
    early["forfeiture"]["year_included"] = json!("2021-22");
    assert_eq!(
        code(&call("2025-26", json!([early]), Value::Null)),
        "ess_discount.interests_invalid"
    );
}

/// The refusals the map requires and the input checks.
#[test]
fn refusals() {
    let lester = json!([share("2025-07-29", "1500.00", "3000.00")]);
    let mut foreign = call("2025-26", lester.clone(), income("1.00"));
    foreign["foreign_employment"] = json!(true);
    assert_eq!(code(&foreign), "ess_discount.foreign_employment");
    assert_eq!(
        code(&call("2019-20", lester.clone(), income("1.00"))),
        "period.unsupported"
    );
    assert_eq!(
        code(&call("2027-28", lester.clone(), income("1.00"))),
        "period.unsupported"
    );
    let old = json!([share("2015-06-30", "0.00", "100.00")]);
    assert_eq!(
        code(&call("2025-26", old, Value::Null)),
        "ess_discount.acquired_before_2015"
    );
    let first = run(&call(
        "2025-26",
        json!([share("2015-07-01", "0.00", "100.00")]),
        income("1.00"),
    ));
    assert_eq!(row(&first, 0)["treatment"], "upfront_eligible");
    assert_eq!(row(&first, 0)["income_year"], "2015-16");
    let deferral_fact = set(
        share("2025-07-29", "1500.00", "3000.00"),
        json!({"cost_base": "0.00"}),
    );
    assert_eq!(
        code(&call("2025-26", json!([deferral_fact]), income("1.00"))),
        "input.field_unknown"
    );
    let before = deferred_share("2025-06-23", json!("2025-06-01"), Value::Null);
    assert_eq!(
        code(&call("2025-26", json!([before]), Value::Null)),
        "ess_discount.dates_inconsistent"
    );
    assert_eq!(
        code(&call("2025-26", json!([]), Value::Null)),
        "ess_discount.interests_invalid"
    );
    let unknown = with_conditions(
        share("2025-07-29", "1500.00", "3000.00"),
        json!({"deferral_stated": true}),
    );
    assert_eq!(
        code(&call("2025-26", json!([unknown]), income("1.00"))),
        "input.field_unknown"
    );
}

/// Each malformed fact is refused with what it must be.
#[test]
fn malformed_facts_say_what_they_must_be() {
    let refused = |interests: Value| {
        let error =
            api::evaluate_coded("ess-discount", &call("2025-26", interests, income("1.00")))
                .unwrap_err();
        assert_eq!(
            error.code, "ess_discount.interests_invalid",
            "{}",
            error.message
        );
        error.message
    };
    let lester = || share("2025-07-29", "1500.00", "3000.00");
    assert!(refused(json!(5)).contains("interests must be a list of 1 to 50 objects."));
    assert!(refused(json!([1])).ends_with(" must be an object."));
    let not_bool = with_conditions(lester(), json!({"employed": "yes"}));
    assert!(refused(json!([not_bool])).contains("conditions: employed must be a boolean."));
    let bad_date = set(lester(), json!({"acquired": "2025-02-30"}));
    assert!(refused(json!([bad_date])).contains("acquired must be a YYYY-MM-DD date."));
    let bad_lift = deferred_share("2025-06-31", Value::Null, Value::Null);
    assert!(
        refused(json!([bad_lift]))
            .contains("restrictions_lifted must be a YYYY-MM-DD date or null.")
    );
    let bad_kind = set(lester(), json!({"kind": "option"}));
    assert!(refused(json!([bad_kind])).contains("kind must be one of share, right."));
    let no_conditions = set(lester(), json!({"conditions": null}));
    assert!(refused(json!([no_conditions])).contains("conditions must be an object."));
    let no_forfeiture = json!({"kind": "share", "acquired": "2022-09-01", "forfeiture": null});
    assert!(refused(json!([no_forfeiture])).contains("forfeiture must be an object."));

    let reversed = rights("2025-07-01", "1.00", "2.00", "2025-06-30");
    let error = api::evaluate_coded(
        "ess-discount",
        &call("2025-26", json!([reversed]), income("1.00")),
    )
    .unwrap_err();
    assert_eq!(error.code, "ess_discount.dates_inconsistent");
    assert!(
        error
            .message
            .ends_with("last_exercise_day is before acquired.")
    );
}

/// s 83A-20(1): no discount, nothing included, and no deferral facts taken.
#[test]
fn no_discount_includes_nothing() {
    let r = run(&call(
        "2025-26",
        json!([share("2025-07-29", "3000.00", "3000.00")]),
        Value::Null,
    ));
    assert_eq!(row(&r, 0)["treatment"], "no_discount");
    assert_eq!(amount(&r, "assessable_discount"), "0.00");
}

/// s 83A-315.08(1), items 1 to 38 as the regulation sets them out: the
/// exercise period, then the 7 calculation percentage columns (50 to 92.5 for
/// items 1 to 19, 92.5 to 110 for items 20 to 38).
const ITEMS: [(u32, &str, [&str; 7]); 38] = [
    (
        1,
        "168 to 180",
        ["0.5", "1.3", "2.6", "3.5", "4.6", "5.8", "7.1"],
    ),
    (
        2,
        "156 to 168",
        ["0.4", "1.2", "2.5", "3.4", "4.4", "5.7", "7.1"],
    ),
    (
        3,
        "144 to 156",
        ["0.4", "1.0", "2.3", "3.2", "4.3", "5.5", "7.0"],
    ),
    (
        4,
        "132 to 144",
        ["0.3", "0.9", "2.2", "3.0", "4.1", "5.4", "6.8"],
    ),
    (
        5,
        "120 to 132",
        ["0.2", "0.8", "2.0", "2.8", "3.9", "5.2", "6.6"],
    ),
    (
        6,
        "108 to 120",
        ["0.2", "0.7", "1.8", "2.6", "3.7", "4.9", "6.4"],
    ),
    (
        7,
        "96 to 108",
        ["0.1", "0.6", "1.6", "2.4", "3.4", "4.6", "6.1"],
    ),
    (
        8,
        "84 to 96",
        ["0.1", "0.4", "1.3", "2.1", "3.0", "4.3", "5.8"],
    ),
    (
        9,
        "72 to 84",
        ["0.1", "0.3", "1.1", "1.7", "2.7", "3.9", "5.4"],
    ),
    (
        10,
        "60 to 72",
        ["0.0", "0.2", "0.8", "1.4", "2.2", "3.4", "4.9"],
    ),
    (
        11,
        "48 to 60",
        ["0.0", "0.1", "0.5", "1.0", "1.7", "2.8", "4.2"],
    ),
    (
        12,
        "36 to 48",
        ["0.0", "0.0", "0.3", "0.6", "1.2", "2.1", "3.4"],
    ),
    (
        13,
        "24 to 36",
        ["0.0", "0.0", "0.1", "0.3", "0.6", "1.3", "2.4"],
    ),
    (
        14,
        "18 to 24",
        ["0.0", "0.0", "0.0", "0.1", "0.3", "0.9", "1.8"],
    ),
    (
        15,
        "12 to 18",
        ["0.0", "0.0", "0.0", "0.0", "0.1", "0.4", "1.1"],
    ),
    (
        16,
        "9 to 12",
        ["0.0", "0.0", "0.0", "0.0", "0.1", "0.2", "0.8"],
    ),
    (
        17,
        "6 to 9",
        ["0.0", "0.0", "0.0", "0.0", "0.0", "0.1", "0.4"],
    ),
    (
        18,
        "3 to 6",
        ["0.0", "0.0", "0.0", "0.0", "0.0", "0.0", "0.1"],
    ),
    (
        19,
        "0 to 3",
        ["0.0", "0.0", "0.0", "0.0", "0.0", "0.0", "0.0"],
    ),
    (
        20,
        "168 to 180",
        ["7.9", "8.6", "9.4", "10.3", "11.2", "12.2", "13.3"],
    ),
    (
        21,
        "156 to 168",
        ["7.8", "8.6", "9.4", "10.3", "11.2", "12.2", "13.3"],
    ),
    (
        22,
        "144 to 156",
        ["7.7", "8.5", "9.4", "10.3", "11.2", "12.2", "13.3"],
    ),
    (
        23,
        "132 to 144",
        ["7.6", "8.4", "9.3", "10.2", "11.2", "12.2", "13.3"],
    ),
    (
        24,
        "120 to 132",
        ["7.5", "8.3", "9.2", "10.2", "11.2", "12.2", "13.3"],
    ),
    (
        25,
        "108 to 120",
        ["7.2", "8.1", "9.1", "10.0", "11.1", "12.1", "13.3"],
    ),
    (
        26,
        "96 to 108",
        ["7.0", "7.9", "8.8", "9.8", "10.9", "12.0", "13.2"],
    ),
    (
        27,
        "84 to 96",
        ["6.6", "7.6", "8.5", "9.6", "10.7", "11.8", "13.0"],
    ),
    (
        28,
        "72 to 84",
        ["6.2", "7.2", "8.2", "9.2", "10.4", "11.6", "12.8"],
    ),
    (
        29,
        "60 to 72",
        ["5.7", "6.7", "7.7", "8.8", "9.9", "11.2", "12.5"],
    ),
    (
        30,
        "48 to 60",
        ["5.1", "6.0", "7.0", "8.2", "9.4", "10.7", "12.1"],
    ),
    (
        31,
        "36 to 48",
        ["4.2", "5.2", "6.2", "7.4", "8.6", "10.0", "11.4"],
    ),
    (
        32,
        "24 to 36",
        ["3.2", "4.1", "5.1", "6.3", "7.6", "9.0", "10.5"],
    ),
    (
        33,
        "18 to 24",
        ["2.5", "3.4", "4.4", "5.5", "6.8", "8.3", "9.9"],
    ),
    (
        34,
        "12 to 18",
        ["1.7", "2.5", "3.4", "4.6", "6.0", "7.5", "9.2"],
    ),
    (
        35,
        "9 to 12",
        ["1.3", "2.0", "2.9", "4.0", "5.4", "7.0", "8.8"],
    ),
    (
        36,
        "6 to 9",
        ["0.8", "1.4", "2.2", "3.3", "4.7", "6.4", "8.3"],
    ),
    (
        37,
        "3 to 6",
        ["0.3", "0.6", "1.3", "2.4", "3.8", "5.7", "7.8"],
    ),
    (
        38,
        "0 to 3",
        ["0.0", "0.1", "0.5", "1.4", "3.0", "5.1", "7.5"],
    ),
];

/// s 83A-315.09(2), items 1 to 19: exercise period, base and additional
/// percentages.
const BASE: [(&str, &str, &str); 19] = [
    ("168 to 180", "13.3", "0.5"),
    ("156 to 168", "13.3", "0.5"),
    ("144 to 156", "13.3", "0.5"),
    ("132 to 144", "13.3", "0.6"),
    ("120 to 132", "13.3", "0.6"),
    ("108 to 120", "13.3", "0.6"),
    ("96 to 108", "13.2", "0.6"),
    ("84 to 96", "13.0", "0.6"),
    ("72 to 84", "12.8", "0.7"),
    ("60 to 72", "12.5", "0.7"),
    ("48 to 60", "12.1", "0.7"),
    ("36 to 48", "11.4", "0.8"),
    ("24 to 36", "10.5", "0.8"),
    ("18 to 24", "9.9", "0.8"),
    ("12 to 18", "9.2", "0.9"),
    ("9 to 12", "8.8", "0.9"),
    ("6 to 9", "8.3", "0.9"),
    ("3 to 6", "7.8", "0.9"),
    ("0 to 3", "7.5", "1.0"),
];

/// Act No. 8, 2022 Sch 10 item 9 looks at the taxing point after the 30-day
/// rule: employment ending on 30 June 2022 with a sale on 20 July 2022 gives
/// a point after 30 June 2022, so ending employment no longer counts and the
/// sale before restrictions lift is refused; a sale on 20 June 2022 keeps it.
#[test]
fn cessation_then_a_sale_after_30_june_2022() {
    let share = |disposed: &str| {
        let value = json!({"date": disposed, "market_value": "2500.00"});
        set(
            deferred_share("2023-08-01", json!(disposed), value),
            json!({"acquired": "2020-08-01", "employment_ended": "2022-06-30"}),
        )
    };
    let right = |disposed: &str| {
        let mut r = early_right(json!("2022-06-30"), disposed);
        r["disposed"] = json!(disposed);
        r
    };
    for interest in [share("2022-07-20"), right("2022-07-20")] {
        let late = call("2022-23", json!([interest]), Value::Null);
        assert_eq!(code(&late), "ess_discount.dates_inconsistent");
    }
    let mut early_sale = right("2022-07-20");
    early_sale["employment_ended"] = json!("2022-06-01");
    early_sale["disposed"] = json!("2022-06-20");
    early_sale["taxing_point_value"]["date"] = json!("2022-06-20");
    for (interest, rule) in [
        (
            set(
                share("2022-06-20"),
                json!({"employment_ended": "2022-06-01"}),
            ),
            "s 83A-115(3)",
        ),
        (early_sale, "s 83A-120(3)"),
    ] {
        let r = run(&call("2021-22", json!([interest]), Value::Null));
        assert_eq!(row(&r, 0)["deferred_taxing_point"], "2022-06-20");
        assert_eq!(row(&r, 0)["taxing_point_rule"], rule);
        assert_eq!(amount(&r, "deferred_amounts"), "2500.00");
    }
}

/// A right is exercised by its last exercise day and not after it is
/// disposed of.
#[test]
fn impossible_right_dates_are_refused() {
    let mut expired = early_right(Value::Null, "2030-08-02");
    expired["exercised"] = json!("2030-08-02");
    expired["share_restrictions_lifted"] = json!("2030-08-02");
    let mut sold_first = early_right(Value::Null, "2023-01-01");
    sold_first["disposed"] = json!("2023-09-01");
    sold_first["exercised"] = json!("2023-09-02");
    sold_first["share_restrictions_lifted"] = json!("2023-09-02");
    for interest in [expired, sold_first] {
        let refused = call("2023-24", json!([interest]), Value::Null);
        assert_eq!(code(&refused), "ess_discount.dates_inconsistent");
    }
}

/// s 83A-105(4)(c): a salary sacrificed right at a real risk of forfeiture
/// beside a share relying on s 83A-105(4) is refused as unsettled; a
/// salary sacrificed share whose discount is less than its market value is
/// not one that could count.
#[test]
fn salary_sacrificed_rights_and_the_5000() {
    let sacrificed = || {
        let mut s = with_conditions(
            share("2025-08-01", "0.00", "3000.00"),
            json!({"salary_sacrifice": true}),
        );
        s["restrictions_lifted"] = json!("2027-08-01");
        s["disposed"] = Value::Null;
        s["cost_base"] = json!("0.00");
        s["taxing_point_value"] = Value::Null;
        s
    };
    let mut right = early_right(Value::Null, "2027-08-01");
    right["acquired"] = json!("2025-08-01");
    right["market_value"] = json!("3000.00");
    right["conditions"]["salary_sacrifice"] = json!(true);
    right["restrictions_lifted"] = json!("2027-08-01");
    right["taxing_point_value"] = Value::Null;
    right.as_object_mut().unwrap().remove("employment_ended");
    let mixed = call("2025-26", json!([sacrificed(), right]), Value::Null);
    assert_eq!(code(&mixed), "ess_discount.salary_sacrifice_unclear");
    let mut paid = with_conditions(sacrificed(), json!({"real_risk_of_forfeiture": true}));
    paid["consideration"] = json!("1.00");
    let r = run(&call("2025-26", json!([sacrificed(), paid]), Value::Null));
    assert_eq!(row(&r, 0)["rule"], "s 83A-105(4)");
    assert_eq!(row(&r, 1)["rule"], "s 83A-105(3)");
}

/// s 83A-105(4)(b)(ii): a scheme offering a right without a real risk of
/// forfeiture is not one s 83A-105(4) applies to, so a right stated as
/// salary sacrificed without that risk contradicts itself. Beside a $3,000
/// share relying on s 83A-105(4) it is refused, not left out of the $5,000.
#[test]
fn a_salary_sacrificed_right_needs_a_real_risk() {
    let mut share = with_conditions(
        share("2025-08-01", "0.00", "3000.00"),
        json!({"salary_sacrifice": true}),
    );
    share["restrictions_lifted"] = json!("2027-08-01");
    share["disposed"] = Value::Null;
    share["cost_base"] = json!("0.00");
    share["taxing_point_value"] = Value::Null;
    let mut right = early_right(Value::Null, "2027-08-01");
    right["acquired"] = json!("2025-08-01");
    right["market_value"] = json!("3000.00");
    right["conditions"]["salary_sacrifice"] = json!(true);
    right["conditions"]["real_risk_of_forfeiture"] = json!(false);
    right["conditions"]["deferral_stated"] = json!(true);
    right["restrictions_lifted"] = json!("2027-08-01");
    right["taxing_point_value"] = Value::Null;
    right.as_object_mut().unwrap().remove("employment_ended");
    for stated in [true, false] {
        right["conditions"]["deferral_stated"] = json!(stated);
        for interests in [
            json!([share.clone(), right.clone()]),
            json!([right.clone()]),
        ] {
            let refused = call("2025-26", interests, Value::Null);
            assert_eq!(code(&refused), "ess_discount.interests_invalid");
        }
    }
}

/// A $3,000 share relying on s 83A-105(4) and, from the same income year, a
/// forfeited interest: a forfeited parcel states no conditions, so whether it
/// counted toward the $5,000 is unknown and the pair is refused.
#[test]
fn a_forfeited_interest_beside_a_salary_sacrificed_share() {
    let mut share = with_conditions(
        share("2025-08-01", "0.00", "3000.00"),
        json!({"salary_sacrifice": true}),
    );
    share["restrictions_lifted"] = json!("2027-08-01");
    share["disposed"] = Value::Null;
    share["cost_base"] = json!("0.00");
    share["taxing_point_value"] = Value::Null;
    let mut lost = forfeited("right", "cancellation_allowed", false);
    lost["acquired"] = json!("2025-08-01");
    lost["forfeiture"]["consideration_received"] = json!(true);
    lost["forfeiture"]["year_included"] = json!("2025-26");
    let mixed = call("2025-26", json!([share.clone(), lost.clone()]), Value::Null);
    assert_eq!(code(&mixed), "ess_discount.salary_sacrifice_unclear");
    lost["acquired"] = json!("2024-08-01");
    let r = run(&call("2025-26", json!([share, lost]), Value::Null));
    assert_eq!(row(&r, 0)["rule"], "s 83A-105(4)");
}

/// s 83A-105(4) defers only the salary sacrificed share: a share beside it in
/// the same year, not salary sacrificed and at no real risk, is taxed upfront.
#[test]
fn only_the_salary_sacrificed_share_relies_on_section_83a_105_4() {
    let mut sacrificed = with_conditions(
        share("2025-08-01", "0.00", "3000.00"),
        json!({"salary_sacrifice": true}),
    );
    sacrificed["restrictions_lifted"] = json!("2027-08-01");
    sacrificed["disposed"] = Value::Null;
    sacrificed["cost_base"] = json!("0.00");
    sacrificed["taxing_point_value"] = Value::Null;
    let plain = share("2025-09-01", "500.00", "1500.00");
    let r = run(&call("2025-26", json!([sacrificed, plain]), income("1.00")));
    assert_eq!(row(&r, 0)["rule"], "s 83A-105(4)");
    assert_eq!(row(&r, 1)["treatment"], "upfront_eligible");
    assert_eq!(amount(&r, "upfront_eligible_discounts"), "1000.00");
}

/// s 83A-315.08(2): a calculation percentage on each column's upper edge
/// and a period of exactly each row's months take the lower range.
#[test]
fn every_table_boundary_takes_the_lower_range() {
    let lower = [60.0, 70.0, 75.0, 80.0, 85.0, 90.0, 92.5];
    let upper = [95.0, 97.5, 100.0, 102.5, 105.0, 107.5];
    for (item, period, entries) in ITEMS {
        let top: i64 = period.split(" to ").nth(1).unwrap().parse().unwrap();
        let last = months_after("2023-01-10", top);
        let edges: &[f64] = if item <= 19 { &lower } else { &upper };
        for (edge, entry) in edges.iter().zip(entries) {
            let shares = format!("{:.2}", edge * 1000.0);
            let v = valued("100000.00", &shares, &last);
            let expected = format!("{:.2}", entry.parse::<f64>().unwrap() * 1000.0);
            assert_eq!(v["table_value"], expected, "item {item} at {edge}%");
        }
    }
}

/// s 83A-115(6) for a share and s 83A-120(3) for a right.
#[test]
fn a_share_at_15_years_and_a_right_sold_within_30_days() {
    let mut s = deferred_share("2022-07-23", Value::Null, Value::Null);
    s["restrictions_lifted"] = Value::Null;
    let r = run(&call("2025-26", json!([s]), Value::Null));
    assert_eq!(row(&r, 0)["deferred_taxing_point"], "2037-07-22");
    assert_eq!(row(&r, 0)["taxing_point_rule"], "s 83A-115(6)");
    let mut right = early_right(Value::Null, "2023-08-31");
    right["disposed"] = json!("2023-08-31");
    let r = run(&call("2023-24", json!([right]), Value::Null));
    assert_eq!(row(&r, 0)["deferred_taxing_point"], "2023-08-31");
    assert_eq!(row(&r, 0)["taxing_point_rule"], "s 83A-120(3)");
    assert_eq!(amount(&r, "deferred_amounts"), "2500.00");
}

/// s 83A-20(1): consideration above market value is no discount, shown as
/// nil rather than negative.
#[test]
fn consideration_above_market_value_shows_a_nil_discount() {
    let r = run(&call(
        "2025-26",
        json!([share("2025-07-29", "3500.00", "3000.00")]),
        Value::Null,
    ));
    assert_eq!(row(&r, 0)["discount"], "0.00");
    assert_eq!(row(&r, 0)["treatment"], "no_discount");
    assert_eq!(amount(&r, "assessable_discount"), "0.00");
}

/// s 83A-105(1)(b): an interest not acquired as an employee fails s 83A-45(1),
/// so even at a real risk of forfeiture nothing defers it and the discount is
/// taxed upfront without the reduction.
#[test]
fn an_interest_outside_the_general_conditions_is_not_deferred() {
    let interest = with_conditions(
        share("2025-07-29", "1500.00", "3000.00"),
        json!({"employed": false, "real_risk_of_forfeiture": true}),
    );
    let r = run(&call("2025-26", json!([interest]), Value::Null));
    assert_eq!(row(&r, 0)["treatment"], "upfront");
    assert_eq!(amount(&r, "upfront_other_discounts"), "1500.00");
    assert_eq!(amount(&r, "assessable_discount"), "1500.00");
}
