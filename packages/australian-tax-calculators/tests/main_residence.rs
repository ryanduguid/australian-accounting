// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The main residence worksheet against the ATO's and the Library's worked
//! examples (docs/calculation-evidence.md#main-residence), its day-counting
//! boundaries and its refusals. Where an example gives only a month or a year,
//! the day used is stated; it does not change the published result.
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn run(overrides: Value) -> Result<Value, String> {
    let mut input = json!({
        "capital_gain": "0.00", "capital_loss": "0.00", "market_value_basis": false,
        "lived_periods": [], "income_use_periods": [], "absence_choice_periods": [],
        "new_home_acquired": null, "year": "2025-26", "scope_confirmed": true,
    });
    for (k, v) in overrides.as_object().unwrap() {
        input[k] = v.clone();
    }
    api::evaluate("main-residence", &input)
}

fn lived(start: &str, end: &str) -> Value {
    json!({"start": start, "end": end})
}

fn used(start: &str, end: &str, floor_area: &str) -> Value {
    json!({"start": start, "end": end, "floor_area": floor_area})
}

fn assessable(result: &Value) -> &str {
    result["amounts"]["assessable_capital_gain"]
        .as_str()
        .unwrap()
}

fn refusal(overrides: Value) -> String {
    run(overrides).unwrap_err()
}

/// Stella, Library ¶2-060: lived from February 2010 (1 February used), rented
/// under the choice from 1 July 2016 to the sale on 30 June 2026.
fn stella(rented_to: &str) -> Value {
    json!({
        "capital_gain": "600000.00", "market_value_basis": true,
        "ownership_start": "2010-02-01", "ownership_end": "2026-06-30",
        "lived_periods": [lived("2010-02-01", "2016-06-30")],
        "income_use_periods": [used("2016-07-01", rented_to, "1")],
        "absence_choice_periods": [lived("2016-07-01", "2026-06-30")],
    })
}

#[test]
fn rented_before_moving_in_matches_peter() {
    // QC66033, Peter: rented 1,004 of 4,564 days from acquisition, so the
    // market value rule does not apply; the ATO prints $50,595.
    let r = run(json!({
        "capital_gain": "230000.00", "ownership_start": "2013-10-01", "ownership_end": "2026-03-30",
        "lived_periods": [lived("2016-07-01", "2026-03-30")],
        "income_use_periods": [used("2013-10-01", "2016-06-30", "1")],
    }))
    .unwrap();
    assert_eq!(
        r["amounts"],
        json!({"assessable_capital_gain": "50595.97", "exempt_capital_gain": "179404.03",
               "assessable_capital_loss": "0.00", "exempt_capital_loss": "0.00"})
    );
    assert_eq!(r["rates"]["ownership_period_days"], "4564");
    assert_eq!(r["rates"]["non_main_residence_days"], "1004");
    assert_eq!(
        r["rates"]["home_first_used_to_produce_income"],
        "not applied"
    );
    assert_eq!(r["rates"]["changing_main_residence_overlap"], "not claimed");
}

#[test]
fn contract_dates_match_farnaz() {
    // QC66033, Farnaz counts contract dates: 757 of 2,355 days; $173,097.
    let r = run(json!({
        "capital_gain": "538500.00", "ownership_start": "2019-10-21", "ownership_end": "2026-04-01",
        "lived_periods": [lived("2021-11-16", "2026-04-01")],
        "income_use_periods": [used("2019-10-21", "2021-11-15", "1")],
    }))
    .unwrap();
    assert_eq!(assessable(&r), "173097.45");
}

#[test]
fn part_rented_throughout_matches_thomas() {
    // QC66033, Thomas: 35% of the floor area rented for the whole ownership.
    let r = run(json!({
        "capital_gain": "400000.00", "ownership_start": "2003-07-01", "ownership_end": "2026-06-30",
        "lived_periods": [lived("2003-07-01", "2026-06-30")],
        "income_use_periods": [used("2003-07-01", "2026-06-30", "0.35")],
    }))
    .unwrap();
    assert_eq!(assessable(&r), "140000.00");
    assert_eq!(
        r["rates"]["home_first_used_to_produce_income"],
        "not applied"
    );
}

#[test]
fn later_business_use_matches_fatima() {
    // QC66033, Fatima: bought December 1997 (1 December used); 40% business
    // use 1 November 2018 to 1 August 2022 restarts the ownership period at
    // market value; 1,370 of 2,739 days; the ATO prints $20,007.
    let facts = json!({
        "capital_gain": "100000.00", "market_value_basis": true,
        "ownership_start": "1997-12-01", "ownership_end": "2026-05-01",
        "lived_periods": [lived("1997-12-01", "2026-05-01")],
        "income_use_periods": [used("2018-11-01", "2022-08-01", "0.4")],
    });
    let r = run(facts.clone()).unwrap();
    assert_eq!(assessable(&r), "20007.30");
    assert_eq!(r["rates"]["ownership_period_from"], "2018-11-01");
    assert_eq!(r["rates"]["ownership_period_days"], "2739");
    assert_eq!(r["rates"]["home_first_used_to_produce_income"], "applied");
    // The rule names the market value date when the gain is on cost.
    let mut cost = facts;
    cost["market_value_basis"] = json!(false);
    assert!(refusal(cost).contains("market value on 2018-11-01"));
}

#[test]
fn rented_after_moving_out_without_the_choice_matches_erin() {
    // QC66033, Erin: rented from 2 August 2025 with no choice; deemed
    // acquired that day at $650,000, so all of the $46,000 gain is taxable.
    let r = run(json!({
        "capital_gain": "46000.00", "market_value_basis": true,
        "ownership_start": "2014-07-25", "ownership_end": "2026-06-10",
        "lived_periods": [lived("2014-07-25", "2025-08-01")],
        "income_use_periods": [used("2025-08-02", "2026-06-10", "1")],
    }))
    .unwrap();
    assert_eq!(assessable(&r), "46000.00");
    assert_eq!(r["rates"]["ownership_period_from"], "2025-08-02");
}

#[test]
fn rented_past_six_years_matches_roya() {
    // QC66030, Roya: purchase date not given (1 March 1995 used); rented
    // from 29 September 1999; covered to 29 September 2005 inclusive;
    // 6,940 of 9,133 days; the ATO prints $243,162.
    let r = run(json!({
        "capital_gain": "320000.00", "market_value_basis": true, "year": "2024-25",
        "ownership_start": "1995-03-01", "ownership_end": "2024-09-29",
        "lived_periods": [lived("1995-03-01", "1999-09-28")],
        "income_use_periods": [used("1999-09-29", "2024-09-29", "1")],
        "absence_choice_periods": [lived("1999-09-29", "2024-09-29")],
    }))
    .unwrap();
    assert_eq!(assessable(&r), "243162.16");
    assert_eq!(r["rates"]["ownership_period_days"], "9133");
    assert_eq!(r["rates"]["non_main_residence_days"], "6940");
}

#[test]
fn rented_past_six_years_differs_from_library_stella_only_by_rounded_years() {
    // ¶2-060 apportions 6 of 10 years ($240,000); by days it is 1,460 of 3,652.
    let r = run(stella("2026-06-30")).unwrap();
    assert_eq!(assessable(&r), "239868.57");
    assert_eq!(r["rates"]["non_main_residence_days"], "1460");
}

#[test]
fn six_years_end_on_the_sixth_anniversary() {
    // Rental from 1 July 2016 is covered to 1 July 2022 inclusive, as the ATO
    // counts Roya; one more day of rental is taxable. Within the cap the
    // dwelling is fully exempt, so the market value rule does not apply.
    let mut covered = stella("2022-07-01");
    covered["market_value_basis"] = json!(false);
    let r = run(covered).unwrap();
    assert_eq!(assessable(&r), "0.00");
    let r = run(stella("2022-07-02")).unwrap();
    assert_eq!(r["rates"]["non_main_residence_days"], "1");
}

#[test]
fn part_used_before_moving_out_stays_taxable_like_helen() {
    // QC66030, Helen: years only (1 March 2008, 30 June 2020 and 1 March 2026
    // used); 25% surgery while living there, then rented under the choice.
    let r = run(json!({
        "capital_gain": "400000.00", "ownership_start": "2008-03-01", "ownership_end": "2026-03-01",
        "lived_periods": [lived("2008-03-01", "2020-06-30")],
        "income_use_periods": [used("2008-03-01", "2020-06-30", "0.25"), used("2020-07-01", "2026-03-01", "1")],
        "absence_choice_periods": [lived("2020-07-01", "2026-03-01")],
    }))
    .unwrap();
    assert_eq!(assessable(&r), "100000.00");
}

#[test]
fn broken_and_repeated_absences_match_lisa_and_jez() {
    // QC66030, Lisa: 3 years rented, 2 vacant, 3 rented, 2 vacant in one
    // absence: 6 years of rental, fully exempt (dates assumed).
    let lisa = run(json!({
        "capital_gain": "500000.00", "ownership_start": "2003-06-01", "ownership_end": "2025-06-30",
        "year": "2024-25",
        "lived_periods": [lived("2003-06-01", "2014-06-30")],
        "income_use_periods": [used("2014-07-01", "2017-06-30", "1"), used("2019-07-01", "2022-06-30", "1")],
        "absence_choice_periods": [lived("2014-07-01", "2025-06-30")],
    }))
    .unwrap();
    assert_eq!(assessable(&lisa), "0.00");
    // Jez: 5 years rented, 2 lived, 4 rented: each absence has its own 6 years.
    let jez = run(json!({
        "capital_gain": "500000.00", "ownership_start": "2005-06-01", "ownership_end": "2025-06-30",
        "year": "2024-25",
        "lived_periods": [lived("2005-06-01", "2014-06-30"), lived("2019-07-01", "2021-06-30")],
        "income_use_periods": [used("2014-07-01", "2019-06-30", "1"), used("2021-07-01", "2025-06-30", "1")],
        "absence_choice_periods": [lived("2014-07-01", "2019-06-30"), lived("2021-07-01", "2025-06-30")],
    }))
    .unwrap();
    assert_eq!(assessable(&jez), "0.00");
}

#[test]
fn changing_homes_matches_library_medford() {
    // ¶2-075: new home 1 October 2025, old one settled 1 May 2026; both are
    // main residences from 1 November 2025, leaving 31 taxable days of 4,749.
    let r = run(json!({
        "capital_gain": "500000.00", "ownership_start": "2013-05-01", "ownership_end": "2026-05-01",
        "lived_periods": [lived("2013-05-01", "2025-09-30")], "new_home_acquired": "2025-10-01",
    }))
    .unwrap();
    assert_eq!(assessable(&r), "3263.85");
    assert_eq!(
        r["rates"]["changing_main_residence_overlap"],
        "from 2025-11-01"
    );
    assert_eq!(r["rates"]["non_main_residence_days"], "31");
}

#[test]
fn changing_homes_needs_three_months_and_no_uncovered_income() {
    // Lived only 2 months of the last 12: no overlap.
    let r = run(json!({
        "capital_gain": "100000.00", "ownership_start": "2025-07-01", "ownership_end": "2026-05-01",
        "lived_periods": [lived("2025-07-01", "2025-08-31")], "new_home_acquired": "2025-09-01",
    }))
    .unwrap();
    assert!(
        r["rates"]["changing_main_residence_overlap"]
            .as_str()
            .unwrap()
            .starts_with("not applied: no continuous 3 months")
    );
    // Rental past the 6-year cap within the last 12 months is income while not
    // the main residence, though inside a choice period.
    let r = run(json!({
        "capital_gain": "100000.00", "market_value_basis": true,
        "ownership_start": "2010-02-01", "ownership_end": "2026-06-30",
        "lived_periods": [lived("2010-02-01", "2016-06-30"), lived("2025-10-01", "2025-12-31")],
        "income_use_periods": [used("2016-07-01", "2025-09-30", "1"), used("2026-01-01", "2026-06-30", "1")],
        "absence_choice_periods": [lived("2016-07-01", "2025-09-30"), lived("2026-01-01", "2026-06-30")],
        "new_home_acquired": "2026-01-01",
    }))
    .unwrap();
    assert_eq!(
        r["rates"]["changing_main_residence_overlap"],
        "not applied: used to produce income in those 12 months while not the main residence"
    );
}

#[test]
fn a_loss_is_apportioned_like_a_gain() {
    // s 118-185: Peter's facts with a $10,000 loss.
    let r = run(json!({
        "capital_loss": "10000.00", "ownership_start": "2013-10-01", "ownership_end": "2026-03-30",
        "lived_periods": [lived("2016-07-01", "2026-03-30")],
        "income_use_periods": [used("2013-10-01", "2016-06-30", "1")],
    }))
    .unwrap();
    assert_eq!(r["amounts"]["assessable_capital_loss"], "2199.82");
    assert_eq!(r["amounts"]["exempt_capital_loss"], "7800.18");
    assert_eq!(r["amounts"]["assessable_capital_gain"], "0.00");
}

#[test]
fn never_lived_in_is_fully_taxable_and_always_lived_in_fully_exempt() {
    let base = json!({"capital_gain": "1000.00", "ownership_start": "2020-01-01", "ownership_end": "2025-12-31"});
    assert_eq!(assessable(&run(base.clone()).unwrap()), "1000.00");
    let mut home = base;
    home["lived_periods"] = json!([lived("2020-01-01", "2025-12-31")]);
    let r = run(home).unwrap();
    assert_eq!(assessable(&r), "0.00");
    assert_eq!(r["amounts"]["exempt_capital_gain"], "1000.00");
}

#[test]
fn refuses_what_it_cannot_establish() {
    let base = json!({"capital_gain": "1000.00", "ownership_start": "2020-01-01", "ownership_end": "2025-12-31"});
    let with = |extra: Value| {
        let mut input = base.clone();
        for (k, v) in extra.as_object().unwrap() {
            input[k] = v.clone();
        }
        refusal(input)
    };
    assert!(with(json!({"capital_loss": "1.00"})).contains("not both"));
    assert!(with(json!({"market_value_basis": true})).contains("does not apply"));
    assert!(with(json!({"ownership_start": "1985-09-19"})).contains("20 September 1985"));
    assert!(with(json!({"ownership_end": "2025-06-30"})).contains("2025-26 income year"));
    assert!(with(json!({"ownership_start": "2020-02-30"})).contains("YYYY-MM-DD"));
    assert!(with(json!({"year": "2027-28"})).contains("Unsupported period"));
    assert!(with(json!({"scope_confirmed": false})).contains("scope"));
    assert!(
        with(json!({"lived_periods": [lived("2020-01-01", "2021-01-01"), lived("2021-01-01", "2022-01-01")]}))
            .contains("must not overlap")
    );
    assert!(
        with(json!({"lived_periods": [lived("2019-12-31", "2021-01-01")]}))
            .contains("inside the ownership period")
    );
    assert!(with(json!({"lived_periods": "2020-01-01"})).contains("list of periods"));
    assert!(
        with(json!({"income_use_periods": [used("2020-01-01", "2021-01-01", "1.5")]}))
            .contains("floor_area")
    );
    assert!(
        with(json!({"income_use_periods": [lived("2020-01-01", "2021-01-01")]}))
            .contains("exactly start, end, floor_area")
    );
    // A choice period must follow a lived period.
    assert!(
        with(json!({"absence_choice_periods": [lived("2021-01-01", "2021-06-30")]}))
            .contains("day after a lived period")
    );
    assert!(with(json!({"new_home_acquired": "2026-01-01"})).contains("new_home_acquired"));
    // Part use that stopped before the move that starts a choice period.
    assert!(
        with(json!({
            "lived_periods": [lived("2020-01-01", "2022-12-31")],
            "income_use_periods": [used("2020-01-01", "2021-12-31", "0.2")],
            "absence_choice_periods": [lived("2023-01-01", "2025-12-31")],
        }))
        .contains("changed or stopped")
    );
    // Rented after moving out, then vacant with no choice: the market value
    // rule's 'only because of income use' cannot be established.
    assert!(
        with(json!({
            "market_value_basis": true,
            "lived_periods": [lived("2020-01-01", "2022-12-31")],
            "income_use_periods": [used("2023-01-01", "2023-12-31", "1")],
        }))
        .contains("besides income use")
    );
}

#[test]
fn the_catalogue_lists_the_worksheet_with_a_runnable_example() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let entry = &catalogue["main_residence"];
    let mut facts = entry["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    let r = api::evaluate("main-residence", &facts).unwrap();
    assert_eq!(assessable(&r), "50595.97");
    assert_eq!(entry["supported_periods"][0]["id"], "2024-25");
}

#[test]
fn the_exempt_share_adds_back_to_the_gain_on_a_half_cent() {
    // 1 taxable day of 8 on $1.00 is $0.125: the taxable share rounds to
    // $0.13 and the exempt share is what is left, not $0.875 rounded up.
    let r = run(json!({
        "capital_gain": "1.00", "ownership_start": "2025-07-01", "ownership_end": "2025-07-08",
        "lived_periods": [lived("2025-07-02", "2025-07-08")],
    }))
    .unwrap();
    assert_eq!(r["amounts"]["assessable_capital_gain"], "0.13");
    assert_eq!(r["amounts"]["exempt_capital_gain"], "0.87");
}

#[test]
fn part_use_that_paused_before_the_move_is_refused() {
    // 25% in 2020 and 2022 but none in 2021, then a choice period: whether
    // the paused use is 'before you stop living in it' is unsettled.
    let message = refusal(json!({
        "capital_gain": "1000.00", "ownership_start": "2020-01-01", "ownership_end": "2025-12-31",
        "lived_periods": [lived("2020-01-01", "2022-12-31")],
        "income_use_periods": [used("2020-01-01", "2020-12-31", "0.25"), used("2022-01-01", "2022-12-31", "0.25")],
        "absence_choice_periods": [lived("2023-01-01", "2025-12-31")],
    }));
    assert!(message.contains("changed or stopped"), "{message}");
}

#[test]
fn acquisition_after_the_event_year_is_refused() {
    let message = refusal(json!({
        "capital_gain": "1000.00", "year": "2024-25",
        "ownership_start": "2025-07-01", "ownership_end": "2025-12-31",
    }));
    assert!(message.contains("ownership_start"), "{message}");
}

#[test]
fn a_loss_on_the_wrong_basis_is_told_to_use_the_reduced_cost_base() {
    let message = refusal(json!({
        "capital_loss": "1000.00", "market_value_basis": true,
        "ownership_start": "2020-01-01", "ownership_end": "2025-12-31",
    }));
    assert!(message.contains("reduced cost base"), "{message}");
}

#[test]
fn the_12_months_before_sale_start_the_day_after_the_date_a_year_earlier() {
    // Sold 30 June 2026: the 12 months run from 1 July 2025, so living there
    // from 30 June to 29 September 2025 is not 3 continuous months inside them.
    let r = run(json!({
        "capital_gain": "100000.00", "ownership_start": "2020-01-01", "ownership_end": "2026-06-30",
        "lived_periods": [lived("2025-06-30", "2025-09-29")], "new_home_acquired": "2026-01-01",
    }))
    .unwrap();
    assert!(
        r["rates"]["changing_main_residence_overlap"]
            .as_str()
            .unwrap()
            .starts_with("not applied: no continuous 3 months"),
        "{}",
        r["rates"]
    );
}

#[test]
fn six_months_before_a_month_end_sale_never_exceed_six_months() {
    // Sold 31 August 2026: 28 February plus 6 months is only 28 August, so
    // both homes are main residences from 1 March, not 28 February.
    let r = run(json!({
        "capital_gain": "100000.00", "ownership_start": "2020-01-01", "ownership_end": "2026-08-31",
        "lived_periods": [lived("2020-01-01", "2026-02-27")], "new_home_acquired": "2026-02-28",
        "year": "2026-27",
    }))
    .unwrap();
    assert_eq!(
        r["rates"]["changing_main_residence_overlap"],
        "from 2026-03-01"
    );
}
