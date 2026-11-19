// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The small business CGT concessions (ITAA 1997 Division 152 and s 328-110,
//! compilation 264, read on 19 November 2026) against the Library's Neil Swan
//! and Wendy Simpson examples (Tax Examples, CGT, 2-280 and 2-303) and
//! hand-worked cases at each test's edges
//! (docs/calculation-evidence.md#small-business-cgt-concessions).
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn run(input: &Value) -> Value {
    api::evaluate("sb-cgt-concessions", input).unwrap()
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("sb-cgt-concessions", input)
        .unwrap_err()
        .code
}

/// Neil Swan, ¶2-280: a resident individual, 54, who has run his business
/// since 2007 on turnover of $1.1 million and $1.2 million. The days, his
/// birthday, the turnover 2 years back and the net value are fabricated.
fn neil() -> Value {
    json!({
        "entity": "individual", "share_or_trust_interest": false,
        "acquired": "2007-07-02", "event_date": "2026-05-15",
        "partnership_asset": false, "passively_held": "no", "business": "carried_on",
        "previous_year_turnover": "1100000.00", "year_before_turnover": "1000000.00",
        "likely_turnover": "1200000.00", "year_turnover": "1200000.00",
        "net_asset_value": "3800000.00", "active_days": 6893, "business_ceased": null,
        "date_of_birth": "1972-04-10", "retirement": false, "permanently_incapacitated": false,
        "last_rollover_event": null,
        "retirement_exemption_used": "0.00", "choice_date": "2026-06-30",
        "year": "2025-26", "scope_confirmed": true,
    })
}

fn with(mut base: Value, changes: Value) -> Value {
    for (key, value) in changes.as_object().unwrap() {
        if value == "remove" {
            base.as_object_mut().unwrap().remove(key);
        } else {
            base[key] = value.clone();
        }
    }
    base
}

fn met(result: &Value, id: &str) -> bool {
    let rows = result["conditions"].as_array().unwrap();
    let row = rows.iter().find(|r| r["condition"] == id);
    row.unwrap_or_else(|| panic!("no {id} in {rows:?}"))["met"]
        .as_bool()
        .unwrap()
}

/// Each concession's availability and its unmet conditions.
fn open(result: &Value) -> Vec<(bool, Vec<String>)> {
    result["concessions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            let unmet = c["unmet"].as_array().unwrap();
            let unmet = unmet.iter().map(|u| u.as_str().unwrap().into()).collect();
            (c["available"].as_bool().unwrap(), unmet)
        })
        .collect()
}

fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

#[test]
fn neil_at_54_has_every_concession_but_the_15_year_exemption() {
    let result = run(&neil());
    assert!(met(&result, "cgt_small_business_entity"));
    assert_eq!(result["conditions"][0]["rule"], "s 328-110(1)(b)(i)");
    assert!(met(&result, "maximum_net_asset_value_test"));
    assert!(met(&result, "active_asset_test"));
    assert!(met(&result, "basic_conditions"));
    assert!(met(&result, "owned_for_15_years"));
    assert!(!met(
        &result,
        "aged_55_and_retiring_or_permanently_incapacitated"
    ));
    let concessions = open(&result);
    assert_eq!(
        concessions[0],
        (
            false,
            names(&["aged_55_and_retiring_or_permanently_incapacitated"])
        )
    );
    for available in &concessions[1..] {
        assert_eq!(*available, (true, vec![]));
    }
    assert_eq!(
        result["amounts"]["retirement_exemption_limit_left"],
        "500000.00"
    );
    // Under 55 just before the choice, he contributes the exempt amount.
    assert_eq!(result["super_contribution_required"], true);
    assert_eq!(
        result["replacement_asset_period"],
        json!({"from": "2025-05-15", "to": "2028-05-15"})
    );
    let test = &result["active_asset_test"];
    assert_eq!(test["test_period_days"], 6893);
    assert_eq!(test["owned_more_than_15_years"], true);
    // The catalogue's example is this case.
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let mut example = catalogue["sb_cgt_concessions"]["example"]["facts"].clone();
    example.as_object_mut().unwrap().remove("kind");
    assert_eq!(example, neil());
}

/// s 104-190(1A): a roll-over also obtained for a later CGT event in the
/// year moves the replacement asset period to run from that one.
#[test]
fn the_replacement_asset_period_runs_from_the_last_event_rolled_over() {
    let later = run(&with(neil(), json!({"last_rollover_event": "2026-06-30"})));
    assert_eq!(
        later["replacement_asset_period"],
        json!({"from": "2025-06-30", "to": "2028-06-30"})
    );
    let same = run(&with(neil(), json!({"last_rollover_event": "2026-05-15"})));
    assert_eq!(
        same["replacement_asset_period"],
        json!({"from": "2025-05-15", "to": "2028-05-15"})
    );
}

#[test]
fn at_55_and_retiring_the_15_year_exemption_takes_the_whole_gain() {
    let result = run(&with(
        neil(),
        json!({"date_of_birth": "1971-04-10", "retirement": true}),
    ));
    let concessions = open(&result);
    assert_eq!(concessions[0], (true, vec![]));
    for other in &concessions[1..] {
        assert_eq!(*other, (false, names(&["fifteen_year_exemption_applies"])));
    }
    assert_eq!(result["replacement_asset_period"], Value::Null);
    // Turning 55 on the day of the CGT event is 55 at that time.
    let birthday = run(&with(
        neil(),
        json!({"date_of_birth": "1971-05-15", "retirement": true}),
    ));
    assert!(open(&birthday)[0].0);
    let day_after = run(&with(
        neil(),
        json!({"date_of_birth": "1971-05-16", "retirement": true}),
    ));
    assert!(!open(&day_after)[0].0);
    // Permanent incapacity stands in for age and retirement.
    let incapacitated = run(&with(neil(), json!({"permanently_incapacitated": true})));
    assert!(open(&incapacitated)[0].0);
    // 55 before the choice: no contribution is required.
    let older = run(&with(neil(), json!({"date_of_birth": "1971-06-30"})));
    assert_eq!(older["super_contribution_required"], false);
}

/// Wendy Simpson, ¶2-303: land she has owned for 20 years, used throughout
/// by the building company she and her husband own. The dates, the
/// company's turnover and the net value are fabricated.
fn wendy() -> Value {
    with(
        neil(),
        json!({
            "acquired": "2006-01-10", "event_date": "2026-03-01",
            "passively_held": "affiliate_or_connected_entity", "carries_on_own_business": false,
            "previous_year_turnover": "1500000.00", "year_before_turnover": "1400000.00",
            "likely_turnover": "1600000.00", "year_turnover": "1700000.00",
            "net_asset_value": "7500000.00", "active_days": 7356,
            "date_of_birth": "1970-08-01",
        }),
    )
}

#[test]
fn wendys_land_used_by_her_company_meets_the_basic_conditions() {
    let result = run(&wendy());
    assert!(met(
        &result,
        "affiliate_or_connected_entity_cgt_small_business_entity"
    ));
    assert!(met(&result, "no_business_of_your_own"));
    assert!(!met(&result, "maximum_net_asset_value_test"));
    assert!(met(&result, "basic_conditions"));
    // 55 or over but not retiring.
    assert_eq!(
        open(&result)[0],
        (
            false,
            names(&["aged_55_and_retiring_or_permanently_incapacitated"])
        )
    );
}

#[test]
fn a_passively_held_asset_needs_the_business_user_to_be_a_small_business() {
    // s 152-10(1A)(a): her company's turnover of $2 million or more fails it,
    // and with net assets over $6 million no other gateway is open.
    let big = with(
        wendy(),
        json!({"previous_year_turnover": "2000000.00", "likely_turnover": "2500000.00",
               "year_turnover": "2100000.00"}),
    );
    let result = run(&big);
    assert!(!met(
        &result,
        "affiliate_or_connected_entity_cgt_small_business_entity"
    ));
    assert!(!met(&result, "small_business_or_net_assets"));
    assert!(!met(&result, "basic_conditions"));
    for concession in open(&result) {
        assert!(!concession.0);
        assert!(concession.1.contains(&"basic_conditions".to_string()));
    }
    // s 152-10(1A)(b): a business of her own outside partnership fails it.
    let own = run(&with(wendy(), json!({"carries_on_own_business": true})));
    assert!(!met(&own, "no_business_of_your_own"));
    assert!(!met(&own, "basic_conditions"));
    // The net value test still opens the gateway.
    let small = run(&with(
        wendy(),
        json!({"carries_on_own_business": true, "net_asset_value": "6000000.00"}),
    ));
    assert!(met(&small, "basic_conditions"));
    // s 152-10(1B): a partnership's business.
    let partner = run(&with(wendy(), json!({"passively_held": "partnership"})));
    assert!(met(&partner, "partnership_cgt_small_business_entity"));
}

fn turnover(previous: Value, before: Value, likely: &str, at_end: &str) -> Value {
    let result = run(&with(
        neil(),
        json!({"previous_year_turnover": previous, "year_before_turnover": before,
               "likely_turnover": likely, "year_turnover": at_end,
               "net_asset_value": "6000000.01"}),
    ));
    result["conditions"][0].clone()
}

#[test]
fn the_turnover_tests_follow_section_328_110_at_2_million() {
    let row = turnover(json!("1999999.99"), json!(null), "5000000.00", "5000000.00");
    assert_eq!(row["rule"], "s 328-110(1)(b)(i)");
    let row = turnover(json!("2000000.00"), json!(null), "1999999.99", "5000000.00");
    assert_eq!(row["rule"], "s 328-110(1)(b)(ii)");
    // s 328-110(3): after 2 years at $2 million or more the likely turnover
    // does not count, but the turnover at the year's end still can.
    let row = turnover(
        json!("2000000.00"),
        json!("2000000.00"),
        "1000000.00",
        "2000000.00",
    );
    assert_eq!(
        (row["met"].clone(), row["rule"].clone()),
        (json!(false), Value::Null)
    );
    let row = turnover(
        json!("2000000.00"),
        json!("2000000.00"),
        "1000000.00",
        "1999999.99",
    );
    assert_eq!(row["rule"], "s 328-110(4)");
    // No business carried on in the year: not met, whatever the turnover.
    let none = run(&with(neil(), json!({"business": "not_carried_on"})));
    assert!(!met(&none, "cgt_small_business_entity"));
    // s 328-110(5): a business being wound up counts if the entity was a
    // small business entity in the year it stopped.
    let winding = with(
        neil(),
        json!({"business": "winding_up", "small_business_when_stopped": true}),
    );
    assert!(met(&run(&winding), "cgt_small_business_entity"));
    let was_not = with(
        winding.clone(),
        json!({"small_business_when_stopped": false}),
    );
    assert!(!met(&run(&was_not), "cgt_small_business_entity"));
    assert_eq!(
        code(&with(winding, json!({"business": "carried_on"}))),
        "input.field_unknown"
    );
    assert_eq!(
        code(&with(neil(), json!({"business": "winding_up"}))),
        "input.missing_field"
    );
}

fn short(changes: Value) -> Value {
    // Owned 1,000 days, from 2023-08-20 to 2026-05-15, both counted.
    with(neil(), json!({"acquired": "2023-08-20"})).pipe(changes)
}

trait Pipe {
    fn pipe(self, changes: Value) -> Value;
}

impl Pipe for Value {
    fn pipe(self, changes: Value) -> Value {
        with(self, changes)
    }
}

#[test]
fn the_active_asset_test_counts_half_or_7_and_a_half_years() {
    let half = run(&short(json!({"active_days": 500})));
    assert_eq!(half["active_asset_test"]["test_period_days"], 1000);
    assert!(met(&half, "active_asset_test"));
    assert!(!met(
        &run(&short(json!({"active_days": 499}))),
        "active_asset_test"
    ));
    assert_eq!(
        code(&short(json!({"active_days": 1001}))),
        "sb_cgt_concessions.fact_conflict"
    );
    // A business that ceased in the 12 months before the event ends the
    // period at the cessation.
    let ceased = run(&short(
        json!({"active_days": 400, "business_ceased": "2025-06-01",
                                   "longer_period_allowed": false}),
    ));
    let test = &ceased["active_asset_test"];
    assert_eq!(test["test_period_to"], "2025-06-01");
    assert_eq!(test["test_period_days"], 652);
    assert!(met(&ceased, "active_asset_test"));
    // Ceased earlier than that: the period runs to the event, unless the
    // Commissioner allows longer.
    let early = short(json!({"active_days": 400, "business_ceased": "2025-05-14",
                             "longer_period_allowed": false}));
    assert_eq!(
        run(&early)["active_asset_test"]["test_period_to"],
        "2026-05-15"
    );
    assert!(!met(&run(&early), "active_asset_test"));
    let allowed = run(&early.pipe(json!({"longer_period_allowed": true})));
    assert_eq!(allowed["active_asset_test"]["test_period_to"], "2025-05-14");
    assert_eq!(
        code(&short(json!({"longer_period_allowed": false}))),
        "input.field_unknown"
    );
    // Owned more than 15 years: 7.5 years, which is 2,737 to 2,741 days.
    assert!(met(
        &run(&with(neil(), json!({"active_days": 2741}))),
        "active_asset_test"
    ));
    assert!(!met(
        &run(&with(neil(), json!({"active_days": 2736}))),
        "active_asset_test"
    ));
    for unclear in [2737, 2740] {
        assert_eq!(
            code(&with(neil(), json!({"active_days": unclear}))),
            "sb_cgt_concessions.active_period_unclear"
        );
    }
}

#[test]
fn a_company_needs_a_significant_individual() {
    let company = with(
        neil(),
        json!({"entity": "company", "retirement_exemption_used": "remove", "choice_date": "remove",
               "significant_individual_before_event": true, "significant_individual_15_years": true,
               "date_of_birth": "1960-01-01", "retirement": true}),
    );
    let result = run(&company);
    assert_eq!(open(&result)[0], (true, vec![]));
    assert_eq!(result["super_contribution_required"], Value::Null);
    let none = run(&with(
        company.clone(),
        json!({"significant_individual_before_event": false, "significant_individual_15_years": false}),
    ));
    let concessions = open(&none);
    assert_eq!(
        concessions[0],
        (
            false,
            names(&[
                "significant_individual_for_15_years",
                "significant_individual_before_event"
            ])
        )
    );
    assert_eq!(concessions[1], (true, vec![]));
    assert_eq!(
        concessions[2],
        (false, names(&["significant_individual_before_event"]))
    );
    assert_eq!(concessions[3], (true, vec![]));
    assert_eq!(
        code(&with(company.clone(), json!({"choice_date": "2026-06-30"}))),
        "input.field_unknown"
    );
    assert_eq!(
        code(&with(
            company,
            json!({"significant_individual_15_years": "remove"})
        )),
        "input.missing_field"
    );
}

#[test]
fn the_retirement_limit_used_up_closes_the_retirement_exemption() {
    let result = run(&with(
        neil(),
        json!({"retirement_exemption_used": "500000.00"}),
    ));
    assert!(!met(&result, "retirement_exemption_limit_left"));
    assert_eq!(
        open(&result)[2],
        (false, names(&["retirement_exemption_limit_left"]))
    );
    let partly = run(&with(
        neil(),
        json!({"retirement_exemption_used": "125000.50"}),
    ));
    assert_eq!(
        partly["amounts"]["retirement_exemption_limit_left"],
        "374999.50"
    );
    assert_eq!(
        code(&with(
            neil(),
            json!({"retirement_exemption_used": "500000.01"})
        )),
        "sb_cgt_concessions.fact_conflict"
    );
}

#[test]
fn refusals_name_what_is_out_of_scope_or_wrong() {
    let cases = [
        (
            json!({"share_or_trust_interest": true}),
            "sb_cgt_concessions.share_or_trust_interest",
        ),
        (
            json!({"acquired": "1985-09-19"}),
            "sb_cgt_concessions.pre_cgt_asset",
        ),
        (
            json!({"acquired": "2011-05-15"}),
            "sb_cgt_concessions.fifteen_years_same_day",
        ),
        (
            json!({"event_date": "2026-07-01"}),
            "sb_cgt_concessions.fact_conflict",
        ),
        (
            json!({"acquired": "2026-05-15"}),
            "sb_cgt_concessions.fact_conflict",
        ),
        (
            json!({"partnership_asset": true, "passively_held": "partnership",
                "carries_on_own_business": false}),
            "sb_cgt_concessions.fact_conflict",
        ),
        (
            json!({"entity": "partnership"}),
            "sb_cgt_concessions.entity_invalid",
        ),
        (
            json!({"passively_held": "yes"}),
            "sb_cgt_concessions.fact_invalid",
        ),
        (
            json!({"business": "yes"}),
            "sb_cgt_concessions.fact_invalid",
        ),
        (
            json!({"choice_date": "2026-05-14"}),
            "sb_cgt_concessions.fact_conflict",
        ),
        (
            json!({"date_of_birth": "2026-05-15"}),
            "sb_cgt_concessions.fact_conflict",
        ),
        (
            json!({"business_ceased": "2026-05-16", "longer_period_allowed": false}),
            "sb_cgt_concessions.fact_conflict",
        ),
        (
            json!({"last_rollover_event": "2026-05-14"}),
            "sb_cgt_concessions.fact_conflict",
        ),
        (
            json!({"last_rollover_event": "2026-07-01"}),
            "sb_cgt_concessions.fact_conflict",
        ),
        (
            json!({"active_days": "6893"}),
            "sb_cgt_concessions.fact_conflict",
        ),
        (json!({"year": "2027-28"}), "period.unsupported"),
        (json!({"scope_confirmed": false}), "scope.not_confirmed"),
        (json!({"retirement": "yes"}), "input.flag_invalid"),
        (
            json!({"previous_year_turnover": "-1.00"}),
            "input.amount_invalid",
        ),
    ];
    for (changes, expected) in cases {
        assert_eq!(code(&with(neil(), changes.clone())), expected, "{changes}");
    }
    // 20 September 1985 is the first post-CGT day.
    let first = with(
        neil(),
        json!({"acquired": "1985-09-20", "active_days": 2741}),
    );
    assert!(met(&run(&first), "basic_conditions"));
    // A net value under nil is allowed.
    assert!(met(
        &run(&with(neil(), json!({"net_asset_value": "-10.00"}))),
        "maximum_net_asset_value_test"
    ));
}
