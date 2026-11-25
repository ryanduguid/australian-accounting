// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Victoria's vacant residential land tax, commercial and industrial property
//! tax and windfall gains tax against the Acts and the State Revenue Office's
//! examples, read on 23 and 24 November 2026
//! (docs/calculation-evidence.md#vic-property-taxes). Every owner and
//! value here is fabricated.
use australian_tax_calculators::api;
use serde_json::{Value, json};

fn run(input: &Value) -> Value {
    let result = api::evaluate("vic-property-taxes", input).unwrap();
    let envelope = json!({"command": "vic-property-taxes", "input": input});
    let doc = api::evaluate("derive", &envelope).unwrap();
    assert_eq!(doc["result"], result, "derive {input}");
    result
}

fn code(input: &Value) -> &'static str {
    api::evaluate_coded("vic-property-taxes", input)
        .unwrap_err()
        .code
}

fn amount(result: &Value, key: &str) -> String {
    result["amounts"][key].as_str().unwrap().to_string()
}

fn with(mut v: Value, changes: Value) -> Value {
    for (key, value) in changes.as_object().unwrap() {
        if value.is_null() {
            v.as_object_mut().unwrap().remove(key);
        } else {
            v[key] = value.clone();
        }
    }
    v
}

/// A vacant home worth $800,000, never liable before, in 2026.
fn vacant() -> Value {
    json!({"tax": "vacant_residential_land", "capital_improved_value": "800000",
           "residential_land": "vacant_home", "new_residential_concession": false,
           "liable_previous_tax_year": false, "liable_tax_year_before": false,
           "year": "2026", "scope_confirmed": true})
}

fn cipt(entry: &str) -> Value {
    json!({"tax": "commercial_industrial", "entry_date": entry, "year": "2026",
           "scope_confirmed": true})
}

/// Windfall gains tax on titles of (CIV1, CIV2), rezoned in 2026.
fn wgt(parcels: &[(&str, &str)]) -> Value {
    let parcels: Vec<Value> = parcels
        .iter()
        .map(|(before, after)| json!({"civ_before": before, "civ_after": after}))
        .collect();
    json!({"tax": "windfall_gains", "parcels": parcels, "year": "2026", "scope_confirmed": true})
}

#[test]
fn vacant_residential_land_tax_rises_with_each_liable_year() {
    // s 35(3)(b): 1% when not liable the year before, 2% after one liable
    // year, 3% after 2; a gap in the earlier year does not matter.
    for (previous, before, want) in [
        (false, false, "8000.00"),
        (false, true, "8000.00"),
        (true, false, "16000.00"),
        (true, true, "24000.00"),
    ] {
        let changes =
            json!({"liable_previous_tax_year": previous, "liable_tax_year_before": before});
        for year in ["2025", "2026"] {
            let input = with(vacant(), changes.clone());
            let result = run(&with(input, json!({"year": year})));
            assert_eq!(amount(&result, "vrlt"), want);
            assert_eq!(amount(&result, "total"), want);
        }
    }
    for land in ["construction", "uninhabitable"] {
        let input = with(
            vacant(),
            json!({"residential_land": land, "liable_previous_tax_year": true}),
        );
        assert_eq!(amount(&run(&input), "total"), "16000.00");
    }
    // A $1 value gives exact cents at each rate.
    for (previous, before, want) in [
        (false, false, "0.01"),
        (true, false, "0.02"),
        (true, true, "0.03"),
    ] {
        let input = with(
            vacant(),
            json!({"capital_improved_value": "1", "liable_previous_tax_year": previous,
                   "liable_tax_year_before": before}),
        );
        assert_eq!(amount(&run(&input), "total"), want);
    }
}

#[test]
fn undeveloped_land_and_the_concession_pay_one_percent() {
    // s 35(3)(a): undeveloped land in a residential zone, from 2026, at 1%
    // whatever its history, which is not asked.
    let undeveloped = with(
        vacant(),
        json!({"residential_land": "undeveloped", "capital_improved_value": "1200000",
               "new_residential_concession": null, "liable_previous_tax_year": null,
               "liable_tax_year_before": null}),
    );
    let result = run(&undeveloped);
    assert_eq!(amount(&result, "total"), "12000.00");
    assert_eq!(result["rates"]["vrlt_rate"], "0.01");
    // s 88EB: the concessional 1%, history not asked, whatever the land's
    // class now: s 88EB(a) asks only that it became s 34B(1) land before.
    for land in ["vacant_home", "construction", "uninhabitable"] {
        let concession = with(
            vacant(),
            json!({"capital_improved_value": "650000", "new_residential_concession": true,
                   "residential_land": land, "liable_previous_tax_year": null,
                   "liable_tax_year_before": null}),
        );
        let result = run(&concession);
        assert_eq!(amount(&result, "total"), "6500.00");
        assert_eq!(result["rates"]["vrlt_rate"], "0.01");
    }
}

#[test]
fn commercial_and_industrial_property_tax_is_nil_until_its_first_year() {
    // SRO: a 1 July 2024 entry first pays in 2035, a 1 August 2034 entry in
    // 2045. The period leaves out the entry date (Interpretation of
    // Legislation Act 1984 (Vic) s 44(1)) and ends on the tenth anniversary,
    // so a 1 January entry is still within it at the midnight before that
    // anniversary's year.
    for (entry, ends, first) in [
        ("2024-07-01", "2034-07-01", "2035"),
        ("2034-08-01", "2044-08-01", "2045"),
        ("2024-12-30", "2034-12-30", "2035"),
        ("2025-01-01", "2035-01-01", "2036"),
        ("2025-01-02", "2035-01-02", "2036"),
        ("2026-01-31", "2036-01-31", "2037"),
        ("2026-02-28", "2036-02-28", "2037"),
        ("2026-03-31", "2036-03-31", "2037"),
        ("2026-04-30", "2036-04-30", "2037"),
        ("2028-02-28", "2038-02-28", "2039"),
        ("2028-02-29", "2038-02-28", "2039"),
    ] {
        for year in ["2025", "2026"] {
            let result = run(&with(cipt(entry), json!({"year": year})));
            assert_eq!(amount(&result, "cipt"), "0.00");
            assert_eq!(amount(&result, "total"), "0.00");
            assert_eq!(result["rates"]["transition_period_ends"], ends);
            assert_eq!(result["rates"]["earliest_possible_tax_year"], first);
            assert_eq!(result["rates"]["cipt_rate"], "0.01");
            assert_eq!(result["rates"]["btr_cipt_rate"], "0.005");
        }
    }
}

#[test]
fn windfall_gains_tax_follows_the_sros_examples_and_edges() {
    for (before, after, want) in [
        // SRO's examples: $187,500 on $400,000; $300,000 on $600,000.
        ("500000", "900000", "187500.00"),
        ("1000000", "1600000", "300000.00"),
        ("0", "100000", "0.00"),
        ("0", "100001", "0.63"),
        ("0", "499999", "249999.38"),
        ("0", "500000", "250000.00"),
        ("700000", "650000", "0.00"),
        ("300000", "300000", "0.00"),
    ] {
        for year in ["2025", "2026"] {
            let result = run(&with(wgt(&[(before, after)]), json!({"year": year})));
            assert_eq!(amount(&result, "wgt"), want, "{before} to {after}");
            assert_eq!(amount(&result, "total"), want);
        }
    }
}

#[test]
fn windfall_gains_tax_aggregates_titles_and_ignores_falls() {
    // s 16(1) and (3): +$300,000, -$50,000 and +$150,000 give $450,000.
    let titles = [
        ("600000", "900000"),
        ("400000", "350000"),
        ("250000", "400000"),
    ];
    let result = run(&wgt(&titles));
    assert_eq!(amount(&result, "value_uplift_2"), "-50000.00");
    assert_eq!(amount(&result, "taxable_value_uplift"), "450000.00");
    assert_eq!(amount(&result, "wgt"), "218750.00");
    // 2 titles of $300,000 each pass $500,000 together: 50% of the whole.
    let result = run(&wgt(&[("0", "300000"), ("100000", "400000")]));
    assert_eq!(amount(&result, "total"), "300000.00");
    // The threshold and the rounding apply once, to the aggregate: $50,000
    // and $50,001 give $100,001, so $0.625, shown $0.63.
    let result = run(&wgt(&[("0", "50000"), ("0", "50001")]));
    assert_eq!(amount(&result, "taxable_value_uplift"), "100001.00");
    assert_eq!(amount(&result, "wgt"), "0.63");
    // All falls: nothing taxable.
    let result = run(&wgt(&[("400000", "300000"), ("200000", "100000")]));
    assert_eq!(amount(&result, "taxable_value_uplift"), "0.00");
    assert_eq!(amount(&result, "total"), "0.00");
    // 100 titles are taken; 101 are refused below.
    let hundred: Vec<(&str, &str)> = vec![("0", "1000"); 100];
    assert_eq!(amount(&run(&wgt(&hundred)), "total"), "0.00");
    let hundred: Vec<(&str, &str)> = vec![("0", "6000"); 100];
    assert_eq!(amount(&run(&wgt(&hundred)), "total"), "300000.00");
}

fn refused(cases: &[(Value, Value, &str)]) {
    for (base, changes, want) in cases {
        let input = with(base.clone(), changes.clone());
        assert_eq!(code(&input), *want, "{input}");
    }
}

#[test]
fn refuses_other_taxes_years_and_land() {
    refused(&[
        (
            vacant(),
            json!({"tax": "land"}),
            "vic_property_taxes.tax_invalid",
        ),
        (vacant(), json!({"year": "2027"}), "period.unsupported"),
        (vacant(), json!({"year": "2024"}), "period.unsupported"),
        (
            vacant(),
            json!({"scope_confirmed": false}),
            "scope.not_confirmed",
        ),
        (
            with(
                vacant(),
                json!({"liable_previous_tax_year": null, "liable_tax_year_before": null, "new_residential_concession": null}),
            ),
            json!({"residential_land": "undeveloped", "year": "2025"}),
            "vic_property_taxes.undeveloped_before_2026",
        ),
        (
            vacant(),
            json!({"residential_land": "shop"}),
            "vic_property_taxes.residential_land_invalid",
        ),
        (
            vacant(),
            json!({"residential_land": null}),
            "input.missing_field",
        ),
        (
            vacant(),
            json!({"year": "2025", "new_residential_concession": true, "liable_previous_tax_year": null, "liable_tax_year_before": null}),
            "vic_property_taxes.concession_unavailable",
        ),
        (
            vacant(),
            json!({"capital_improved_value": "800000.50"}),
            "input.amount_not_whole_dollars",
        ),
        (
            vacant(),
            json!({"capital_improved_value": null}),
            "input.missing_field",
        ),
    ]);
}

#[test]
fn refuses_bad_entry_dates() {
    refused(&[
        (
            cipt("2024-06-30"),
            json!({}),
            "vic_property_taxes.entry_date_before_reform",
        ),
        (cipt("2024-13-01"), json!({}), "input.date_invalid"),
        (
            cipt("2024-12-31"),
            json!({}),
            "vic_property_taxes.transition_end_unsettled",
        ),
        (
            cipt("2030-12-31"),
            json!({"year": "2025"}),
            "vic_property_taxes.transition_end_unsettled",
        ),
        (
            cipt("2025-01-01"),
            json!({"entry_date": null}),
            "input.missing_field",
        ),
    ]);
}

#[test]
fn refuses_bad_parcels() {
    refused(&[
        (wgt(&[]), json!({}), "vic_property_taxes.parcels_invalid"),
        (
            wgt(&[("1", "2")]),
            json!({"parcels": [1]}),
            "vic_property_taxes.parcels_invalid",
        ),
        (
            wgt(&[("1", "2")]),
            json!({"parcels": [{"civ_before": "1"}]}),
            "vic_property_taxes.parcels_invalid",
        ),
        (
            wgt(&[("-1", "2")]),
            json!({}),
            "vic_property_taxes.parcels_invalid",
        ),
        (
            wgt(&[("1.50", "2")]),
            json!({}),
            "vic_property_taxes.parcels_invalid",
        ),
        (
            wgt(&[("1", "2")]),
            json!({"parcels": [{"civ_before": "1", "civ_after": "2", "area": "3"}]}),
            "input.field_unknown",
        ),
        (
            wgt(&[("1", "2")]),
            json!({"parcels": null}),
            "input.missing_field",
        ),
    ]);
}

#[test]
fn refuses_inputs_out_of_place() {
    let too_many: Vec<(&str, &str)> = vec![("1", "2"); 101];
    refused(&[
        (
            wgt(&too_many),
            json!({}),
            "vic_property_taxes.parcels_invalid",
        ),
        (
            vacant(),
            json!({"entry_date": "2025-01-01"}),
            "input.field_unknown",
        ),
        (vacant(), json!({"parcels": []}), "input.field_unknown"),
        (
            cipt("2025-01-01"),
            json!({"capital_improved_value": "1"}),
            "input.field_unknown",
        ),
        (
            wgt(&[("1", "2")]),
            json!({"residential_land": "vacant_home"}),
            "input.field_unknown",
        ),
        // History is not asked where it does not change the rate.
        (
            vacant(),
            json!({"new_residential_concession": true}),
            "input.field_unknown",
        ),
        (
            with(
                vacant(),
                json!({"new_residential_concession": null, "liable_tax_year_before": null}),
            ),
            json!({"residential_land": "undeveloped"}),
            "input.field_unknown",
        ),
        (
            vacant(),
            json!({"liable_tax_year_before": null}),
            "input.missing_field",
        ),
        (
            vacant(),
            json!({"new_residential_concession": "yes"}),
            "input.flag_invalid",
        ),
    ]);
}

#[test]
fn the_steps_show_their_working() {
    let input = with(
        vacant(),
        json!({"liable_previous_tax_year": true, "liable_tax_year_before": true}),
    );
    let envelope = json!({"command": "vic-property-taxes", "input": input});
    let doc = api::evaluate("derive", &envelope).unwrap();
    let vrlt = &doc["steps"][0];
    assert_eq!(
        vrlt["working"],
        "capital_improved_value x vrlt_rate = 800000 x 0.03 = 24000.00"
    );
    assert!(
        vrlt["authority"]
            .as_str()
            .unwrap()
            .contains("35(3)(b)(iii)")
    );
    // 2025's text numbered the same rates s 35(3)(a) to (c).
    let input = with(input, json!({"year": "2025"}));
    let envelope = json!({"command": "vic-property-taxes", "input": input});
    let doc = api::evaluate("derive", &envelope).unwrap();
    let authority = doc["steps"][0]["authority"].as_str().unwrap();
    assert!(
        authority.contains("35(3)(c) as in force for 2025"),
        "{authority}"
    );
    let envelope = json!({"command": "vic-property-taxes", "input": wgt(&[("500000", "900000")])});
    let doc = api::evaluate("derive", &envelope).unwrap();
    assert_eq!(
        doc["steps"][2]["working"],
        "(taxable_value_uplift - wgt_threshold) x wgt_lower_rate = (400000 - 100000) x 0.625 = 187500.000"
    );
}
