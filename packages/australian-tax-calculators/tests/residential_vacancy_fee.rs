// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Tests for `residential-vacancy-fee` against the ATO's 'Vacancy fee return
//! for foreign owners' and 'Residential fees for a foreign person' examples,
//! Treasury Guidance Notes 6 (v5) and 10 (v12), Schedule of Fees (v8), FATA
//! 1975 Part 6A, FATR 2015 Division 4A, Fees Imposition Act 2015 ss 5 to 9
//! and Fees Imposition Regulations 2020 Part 3.

use australian_tax_calculators::api;
use serde_json::{Value, json};

fn base(year: &str, start: &str) -> Value {
    json!({
        "year": year,
        "scope_confirmed": true,
        "vacancy_year_start": start,
        "held_full_vacancy_year": true,
        "residentially_occupied_days": 100,
        "return_lodged_on_time": true,
        "exemption": null,
        "acquisition_basis": "notifiable_action",
        "ownership": "sole",
        "application_fee_waiver": "none",
        "vacancy_fee_waiver_or_remission": false,
    })
}

fn eval(input: &Value) -> Value {
    api::evaluate("residential-vacancy-fee", input)
        .unwrap_or_else(|e| panic!("unexpected refusal: {e} for {input}"))
}

fn refuse_code(input: &Value) -> &'static str {
    api::evaluate_coded("residential-vacancy-fee", input)
        .expect_err("expected refusal")
        .code
}

#[test]
fn ato_myeong_pre_and_post_9_april_2024_doubling() {
    // ATO 'Vacancy fee return for foreign owners' — Myeong:
    // $850,000 new townhouse settled 1 August 2022, $13,200 fee.
    for (year, start, end, due, expected_mult, expected_fee) in [
        (
            "2022-23",
            "2022-08-01",
            "2023-07-31",
            "2023-08-30",
            "1",
            "13200.00",
        ),
        // Starts before 9 April 2024 (ends after): still 1x per Regs s 67(2).
        (
            "2023-24",
            "2023-08-01",
            "2024-07-31",
            "2024-08-30",
            "1",
            "13200.00",
        ),
        // Starts on or after 9 April 2024: doubled to $26,400.
        (
            "2024-25",
            "2024-08-01",
            "2025-07-31",
            "2025-08-30",
            "2",
            "26400.00",
        ),
    ] {
        let mut input = base(year, start);
        input["application_fee_payable"] = json!("13200.00");
        let out = eval(&input);
        assert_eq!(out["vacancy_year_end"], end);
        assert_eq!(out["return_due_date"], due);
        assert_eq!(out["rates"]["vacancy_fee_multiplier"], expected_mult);
        assert_eq!(out["amounts"]["vacancy_fee_payable"], expected_fee);
        assert_eq!(out["liable"], true);
        assert_eq!(out["liability_status"], "liable_vacant");
    }

    // Same example derived from the 29 July 2022 schedule ($850,000 new dwelling).
    let mut derived = base("2024-25", "2024-08-01");
    derived["consideration"] = json!("850000.00");
    derived["dwelling_kind"] = json!("new_or_near_new");
    derived["application_date"] = json!("2022-08-01");
    let out = eval(&derived);
    assert_eq!(out["amounts"]["application_fee_base"], "13200.00");
    assert_eq!(out["amounts"]["vacancy_fee_payable"], "26400.00");
}

#[test]
fn ato_edmond_vacancy_year_and_due_date_and_retention() {
    // ATO 'Vacancy fee return for foreign owners' — Edmond:
    // Occupied from 5 October 2022 -> year ends 4 October 2023, due 3 November 2023.
    let mut input = base("2022-23", "2022-10-05");
    input["residentially_occupied_days"] = json!(200);
    input["application_fee_payable"] = json!("13200.00");
    let out = eval(&input);
    assert_eq!(out["vacancy_year_start"], "2022-10-05");
    assert_eq!(out["vacancy_year_end"], "2023-10-04");
    assert_eq!(out["days_in_vacancy_year"], 365);
    assert_eq!(out["return_due_date"], "2023-11-03");
    assert_eq!(out["record_retention_until"], "2028-10-04");
    assert_eq!(out["return_required"], true);
    assert_eq!(out["liable"], false);
    assert_eq!(out["liability_status"], "not_liable_occupied");
    assert_eq!(out["amounts"]["vacancy_fee_payable"], "0.00");
}

#[test]
fn ato_rishi_2026_27_exemption_certificate_doubled_vacancy_fee() {
    // ATO 'Residential fees for a foreign person' — Rishi:
    // Exemption certificate in Sept 2026 ($31,300), settled 2 November 2026,
    // year ends 1 November 2027, return due 1 December 2027, vacancy fee $62,600.
    let mut input = base("2026-27", "2026-11-02");
    input["acquisition_basis"] = json!("other_exemption_certificate");
    input["application_fee_payable"] = json!("31300.00");
    let out = eval(&input);
    assert_eq!(out["vacancy_year_end"], "2027-11-01");
    assert_eq!(out["return_due_date"], "2027-12-01");
    assert_eq!(out["amounts"]["application_fee_base"], "31300.00");
    assert_eq!(out["amounts"]["vacancy_fee_payable"], "62600.00");
    assert_eq!(out["rates"]["statutory_fee_cap_floor"], "7791000");
}

#[test]
fn treasury_gn10_example_34_established_dwelling_tripled_and_doubled() {
    // Treasury GN10 v12 Example 34: $1.5m established dwelling bought 2 July 2024
    // (2024-25 schedule: 3 x $29,500 = $88,500), settled 15 August 2024 -> $177,000.
    let mut input = base("2024-25", "2024-08-15");
    input["residentially_occupied_days"] = json!(79);
    input["consideration"] = json!("1500000.00");
    input["dwelling_kind"] = json!("established");
    input["application_date"] = json!("2024-07-02");
    let out = eval(&input);
    assert_eq!(out["vacancy_year_end"], "2025-08-14");
    assert_eq!(out["amounts"]["whole_property_application_fee"], "88500.00");
    assert_eq!(out["amounts"]["application_fee_base"], "88500.00");
    assert_eq!(out["amounts"]["vacancy_fee_payable"], "177000.00");
    assert!(out["established_dwelling_ban_note"].is_null());

    // Matching stated-fee path produces the exact same vacancy fee.
    let mut stated = base("2024-25", "2024-08-15");
    stated["residentially_occupied_days"] = json!(79);
    stated["application_fee_payable"] = json!("88500.00");
    assert_eq!(eval(&stated)["amounts"]["vacancy_fee_payable"], "177000.00");
}

#[test]
fn ato_sara_tenants_in_common_proportionate_share() {
    // ATO 'Residential fees for a foreign person' — Sara:
    // 25% tenant in common of $1.5m new dwelling on 1 August 2026:
    // whole fee $31,300 -> 25% share = $7,825 -> doubled vacancy fee = $15,650.
    let mut input = base("2026-27", "2026-08-01");
    input["ownership"] = json!("tenants_in_common");
    input["consideration"] = json!("1500000.00");
    input["dwelling_kind"] = json!("new_or_near_new");
    input["application_date"] = json!("2026-08-01");
    input["ownership_share"] = json!("0.25");
    let out = eval(&input);
    assert_eq!(out["amounts"]["whole_property_application_fee"], "31300.00");
    assert_eq!(out["amounts"]["application_fee_base"], "7825.00");
    assert_eq!(out["amounts"]["vacancy_fee_payable"], "15650.00");

    // Fractional share rounding (1/3 of $15,600 = $5,200; 0.3333 -> $5,199.48).
    let mut frac = base("2026-27", "2026-08-01");
    frac["ownership"] = json!("tenants_in_common");
    frac["consideration"] = json!("800000.00");
    frac["dwelling_kind"] = json!("new_or_near_new");
    frac["application_date"] = json!("2026-08-01");
    frac["ownership_share"] = json!("0.3333");
    let fout = eval(&frac);
    assert_eq!(fout["amounts"]["application_fee_base"], "5199.48");
    assert_eq!(fout["amounts"]["vacancy_fee_payable"], "10398.96");
}

#[test]
fn gn6_example_27_late_return_deems_liability_overriding_occupancy_and_exemption() {
    // Nikolai occupied 250 days but failed to lodge within 30 days -> FATA s 115D(3).
    let mut nikolai = base("2025-26", "2025-07-01");
    nikolai["residentially_occupied_days"] = json!(250);
    nikolai["return_lodged_on_time"] = json!(false);
    nikolai["application_fee_payable"] = json!("15100.00");
    let out = eval(&nikolai);
    assert_eq!(out["liable"], true);
    assert_eq!(out["liability_status"], "deemed_liable_late_return");
    assert_eq!(out["amounts"]["vacancy_fee_payable"], "30200.00");

    // FATR 2015 s 43D(4): late return ALSO overrides a s 43D exemption.
    let mut exempt_late = base("2025-26", "2025-07-01");
    exempt_late["residentially_occupied_days"] = json!(40);
    exempt_late["return_lodged_on_time"] = json!(false);
    exempt_late["exemption"] = json!("damaged_unsafe_or_unsuitable");
    exempt_late["application_fee_payable"] = json!("15100.00");
    let out2 = eval(&exempt_late);
    assert_eq!(out2["liable"], true);
    assert_eq!(out2["liability_status"], "deemed_liable_late_return");
    assert!(out2["exemption_applied"].is_null());
    assert_eq!(out2["amounts"]["vacancy_fee_payable"], "30200.00");
}

#[test]
fn occupancy_183_day_boundary_including_leap_year() {
    // 2023-08-01 to 2024-07-31 includes 29 Feb 2024 -> 366 days.
    // FATA s 115C(1): "fewer than 183 days", so 183 occupied days is NOT liable
    // even though 183 days were vacant in a 366-day leap year.
    let mut leap_183 = base("2023-24", "2023-08-01");
    leap_183["residentially_occupied_days"] = json!(183);
    leap_183["application_fee_payable"] = json!("14100.00");
    let out = eval(&leap_183);
    assert_eq!(out["days_in_vacancy_year"], 366);
    assert_eq!(out["liable"], false);
    assert_eq!(out["liability_status"], "not_liable_occupied");
    assert_eq!(out["amounts"]["vacancy_fee_payable"], "0.00");

    // 182 days IS liable.
    leap_183["residentially_occupied_days"] = json!(182);
    let out_182 = eval(&leap_183);
    assert_eq!(out_182["liable"], true);
    assert_eq!(out_182["amounts"]["vacancy_fee_payable"], "14100.00");

    // Leap-day occupation start: 2024-02-29 to 2025-02-28 (366 days, AIA s 2G(2)).
    let mut feb29 = base("2023-24", "2024-02-29");
    feb29["residentially_occupied_days"] = json!(366);
    feb29["application_fee_payable"] = json!("14100.00");
    let out_feb = eval(&feb29);
    assert_eq!(out_feb["vacancy_year_end"], "2025-02-28");
    assert_eq!(out_feb["days_in_vacancy_year"], 366);
    assert_eq!(out_feb["liable"], false);
}

#[test]
fn nine_april_2024_boundary_and_established_tripling_and_ban_note() {
    // 8 April 2024 start -> 1x; 9 April 2024 start -> 2x.
    let mut apr8 = base("2023-24", "2024-04-08");
    apr8["application_fee_payable"] = json!("14100.00");
    assert_eq!(eval(&apr8)["amounts"]["vacancy_fee_payable"], "14100.00");

    let mut apr9 = base("2023-24", "2024-04-09");
    apr9["application_fee_payable"] = json!("14100.00");
    assert_eq!(eval(&apr9)["amounts"]["vacancy_fee_payable"], "28200.00");

    // Established dwelling application: 8 April 2024 (1x = $14,100) vs 9 April 2024 (3x = $42,300).
    let mut est8 = base("2024-25", "2024-07-01");
    est8["consideration"] = json!("900000.00");
    est8["dwelling_kind"] = json!("established");
    est8["application_date"] = json!("2024-04-08");
    assert_eq!(eval(&est8)["amounts"]["application_fee_base"], "14100.00");
    assert_eq!(eval(&est8)["amounts"]["vacancy_fee_payable"], "28200.00");

    let mut est9 = base("2024-25", "2024-07-01");
    est9["consideration"] = json!("900000.00");
    est9["dwelling_kind"] = json!("established");
    est9["application_date"] = json!("2024-04-09");
    assert_eq!(eval(&est9)["amounts"]["application_fee_base"], "42300.00");
    assert_eq!(eval(&est9)["amounts"]["vacancy_fee_payable"], "84600.00");

    // Established dwelling ban note active from 1 April 2025.
    let mut ban = base("2025-26", "2025-07-01");
    ban["consideration"] = json!("900000.00");
    ban["dwelling_kind"] = json!("established");
    ban["application_date"] = json!("2025-04-01");
    assert!(eval(&ban)["established_dwelling_ban_note"].is_string());
}

#[test]
fn all_2026_27_residential_tier_boundaries_match_ato_tables_1_and_2() {
    let cases = [
        // (consideration, new_fee, established_fee)
        ("74999.99", "4600.00", "13800.00"),
        ("75000.00", "15600.00", "46800.00"),
        ("1000000.00", "15600.00", "46800.00"),
        ("1000000.01", "31300.00", "93900.00"),
        ("2000000.00", "31300.00", "93900.00"),
        ("2000000.01", "62600.00", "187800.00"),
        ("40000000.00", "1220700.00", "3662100.00"),
        ("40000000.01", "1245500.00", "3736500.00"),
        ("100000000.00", "1245500.00", "3736500.00"),
    ];
    for (cons, new_fee, est_fee) in cases {
        let mut n = base("2026-27", "2026-07-01");
        n["consideration"] = json!(cons);
        n["dwelling_kind"] = json!("new_or_near_new");
        n["application_date"] = json!("2026-07-01");
        let nout = eval(&n);
        assert_eq!(
            nout["amounts"]["application_fee_base"], new_fee,
            "new {cons}"
        );

        let mut e = base("2026-27", "2026-07-01");
        e["consideration"] = json!(cons);
        e["dwelling_kind"] = json!("established");
        e["application_date"] = json!("2026-07-01");
        let eout = eval(&e);
        assert_eq!(
            eout["amounts"]["application_fee_base"], est_fee,
            "est {cons}"
        );
    }

    // Verify 2026-27 maximum established vacancy fee ($7,473,000) stays under cap floor ($7,791,000).
    let mut max_est = base("2026-27", "2026-07-01");
    max_est["consideration"] = json!("50000000.00");
    max_est["dwelling_kind"] = json!("established");
    max_est["application_date"] = json!("2026-07-01");
    let mout = eval(&max_est);
    assert_eq!(mout["amounts"]["vacancy_fee_payable"], "7473000.00");
}

#[test]
fn historical_schedules_2021_to_2026_match_statutory_indexation() {
    let checks = [
        (
            "2021-01-01",
            "2020-21",
            "2021-01-15",
            "6350.00",
            "500000.00",
        ),
        (
            "2021-07-01",
            "2021-22",
            "2021-07-15",
            "6350.00",
            "503000.00",
        ),
        (
            "2022-07-01",
            "2022-23",
            "2022-07-15",
            "6600.00",
            "522500.00",
        ),
        (
            "2022-07-29",
            "2022-23",
            "2022-08-01",
            "13200.00",
            "1045000.00",
        ),
        (
            "2023-07-01",
            "2023-24",
            "2023-07-15",
            "14100.00",
            "1119100.00",
        ),
        (
            "2024-07-01",
            "2024-25",
            "2024-07-15",
            "14700.00",
            "1171600.00",
        ),
        (
            "2025-07-01",
            "2025-26",
            "2025-07-15",
            "15100.00",
            "1205200.00",
        ),
    ];
    for (app_date, year, vac_start, tier1, max_cap) in checks {
        let mut t1 = base(year, vac_start);
        t1["consideration"] = json!("500000.00");
        t1["dwelling_kind"] = json!("new_or_near_new");
        t1["application_date"] = json!(app_date);
        assert_eq!(
            eval(&t1)["amounts"]["application_fee_base"],
            tier1,
            "{app_date} t1"
        );

        let mut mc = base(year, vac_start);
        mc["consideration"] = json!("50000000.00");
        mc["dwelling_kind"] = json!("new_or_near_new");
        mc["application_date"] = json!(app_date);
        assert_eq!(
            eval(&mc)["amounts"]["application_fee_base"],
            max_cap,
            "{app_date} cap"
        );
    }
}

#[test]
fn exemptions_not_held_full_year_and_full_waivers_and_cap() {
    // All 4 FATR 2015 s 43D exemptions give nil when return is lodged on time.
    for ex in [
        "damaged_unsafe_or_unsuitable",
        "substantial_repairs_or_renovations",
        "legal_restriction_or_court_order",
        "medical_or_residential_care",
    ] {
        let mut input = base("2025-26", "2025-08-01");
        input["exemption"] = json!(ex);
        input["application_fee_payable"] = json!("15100.00");
        let out = eval(&input);
        assert_eq!(out["liable"], false, "{ex}");
        assert_eq!(out["liability_status"], "exempt", "{ex}");
        assert_eq!(out["exemption_applied"], ex);
        assert_eq!(out["return_required"], true);
        assert_eq!(out["amounts"]["vacancy_fee_payable"], "0.00");
    }

    // Disposed / ceased to be foreign person during the vacancy year.
    let mut disposed = base("2025-26", "2025-08-01");
    disposed["held_full_vacancy_year"] = json!(false);
    disposed["return_lodged_on_time"] = json!(false);
    disposed["application_fee_payable"] = json!("15100.00");
    let dout = eval(&disposed);
    assert_eq!(dout["liable"], false);
    assert_eq!(dout["return_required"], false);
    assert_eq!(dout["liability_status"], "not_required");
    assert!(dout["return_due_date"].is_null());
    assert!(dout["record_retention_until"].is_null());
    assert_eq!(dout["amounts"]["vacancy_fee_payable"], "0.00");

    // Post-9 April 2024 full waiver (s 43(b)) — established dwelling (3x) and new (1x).
    let mut w_new = base("2026-27", "2026-08-01");
    w_new["application_fee_waiver"] = json!("full");
    w_new["dwelling_kind"] = json!("new_or_near_new");
    w_new["application_date"] = json!("2026-07-10");
    assert_eq!(eval(&w_new)["amounts"]["vacancy_fee_payable"], "31200.00");

    let mut w_est = base("2026-27", "2026-08-01");
    w_est["application_fee_waiver"] = json!("full");
    w_est["dwelling_kind"] = json!("established");
    w_est["application_date"] = json!("2026-07-10");
    assert_eq!(eval(&w_est)["amounts"]["vacancy_fee_payable"], "93600.00");

    // Pre-9 April 2024 full waiver (s 46(b)) in a stable window.
    let mut w_pre = base("2022-23", "2022-08-01");
    w_pre["acquisition_basis"] = json!("no_notice_order");
    w_pre["application_fee_waiver"] = json!("full");
    assert_eq!(eval(&w_pre)["amounts"]["vacancy_fee_payable"], "14100.00");

    // Statutory s 6(3) cap binds on a large custom stated application fee
    // whose payable window lies cleanly inside 2025-26 ($7,539,000 cap).
    let mut capped = base("2024-25", "2024-08-01");
    capped["acquisition_basis"] = json!("other_exemption_certificate");
    capped["application_fee_payable"] = json!("4000000.00");
    let cout = eval(&capped);
    assert_eq!(cout["amounts"]["uncapped_vacancy_fee"], "8000000.00");
    assert_eq!(cout["amounts"]["vacancy_fee_payable"], "7539000.00");
    assert_eq!(cout["rates"]["statutory_fee_cap"], "7539000");

    // Straddling the 1 July 2025 cap change ($7,329,000 -> $7,539,000) refuses cleanly.
    let mut straddle = base("2024-25", "2024-07-01");
    straddle["acquisition_basis"] = json!("other_exemption_certificate");
    straddle["application_fee_payable"] = json!("4000000.00");
    assert_eq!(
        refuse_code(&straddle),
        "residential_vacancy_fee.cap_unsettled"
    );
}

#[test]
fn every_refusal_code_is_triggered_cleanly() {
    // 1. acquisition_basis_invalid
    let mut i = base("2025-26", "2025-08-01");
    i["acquisition_basis"] = json!("commercial");
    i["application_fee_payable"] = json!("15100.00");
    assert_eq!(
        refuse_code(&i),
        "residential_vacancy_fee.acquisition_basis_invalid"
    );

    // 2. build_to_rent_unsupported
    let mut i = base("2025-26", "2025-08-01");
    i["acquisition_basis"] = json!("build_to_rent");
    i["application_fee_payable"] = json!("15100.00");
    assert_eq!(
        refuse_code(&i),
        "residential_vacancy_fee.build_to_rent_unsupported"
    );

    // 3. application_date_unsupported (pre-2021, post-2027-06-30, after vacancy start)
    let mut i = base("2025-26", "2025-08-01");
    i["consideration"] = json!("800000.00");
    i["dwelling_kind"] = json!("new_or_near_new");
    i["application_date"] = json!("2020-12-31");
    assert_eq!(
        refuse_code(&i),
        "residential_vacancy_fee.application_date_unsupported"
    );
    i["application_date"] = json!("2027-07-01");
    assert_eq!(
        refuse_code(&i),
        "residential_vacancy_fee.application_date_unsupported"
    );
    i["application_date"] = json!("2025-09-01");
    assert_eq!(
        refuse_code(&i),
        "residential_vacancy_fee.application_date_unsupported"
    );

    // 4. application_fee_invalid (nil, conflicting with consideration, developer_ec, other_ec missing)
    let mut i = base("2025-26", "2025-08-01");
    i["application_fee_payable"] = json!("0.00");
    assert_eq!(
        refuse_code(&i),
        "residential_vacancy_fee.application_fee_invalid"
    );
    i["application_fee_payable"] = json!("15100.00");
    i["consideration"] = json!("800000.00");
    assert_eq!(
        refuse_code(&i),
        "residential_vacancy_fee.application_fee_invalid"
    );
    let mut dev = base("2025-26", "2025-08-01");
    dev["acquisition_basis"] = json!("developer_exemption_certificate");
    dev["application_fee_payable"] = json!("15100.00");
    assert_eq!(
        refuse_code(&dev),
        "residential_vacancy_fee.application_fee_invalid"
    );
    let mut other_ec = base("2025-26", "2025-08-01");
    other_ec["acquisition_basis"] = json!("other_exemption_certificate");
    other_ec["consideration"] = json!("800000.00");
    assert_eq!(
        refuse_code(&other_ec),
        "residential_vacancy_fee.application_fee_invalid"
    );

    // 5. cap_unsettled (payable window enters unindexed 2027-28 and uncapped > $7,791,000)
    let mut i = base("2026-27", "2026-11-02");
    i["acquisition_basis"] = json!("other_exemption_certificate");
    i["application_fee_payable"] = json!("4000000.00");
    assert_eq!(refuse_code(&i), "residential_vacancy_fee.cap_unsettled");

    // 6. consideration_invalid (missing, nil, or given with full waiver)
    let mut i = base("2025-26", "2025-08-01");
    assert_eq!(
        refuse_code(&i),
        "residential_vacancy_fee.consideration_invalid"
    );
    i["consideration"] = json!("0.00");
    assert_eq!(
        refuse_code(&i),
        "residential_vacancy_fee.consideration_invalid"
    );
    let mut wf = base("2025-26", "2025-08-01");
    wf["application_fee_waiver"] = json!("full");
    wf["consideration"] = json!("800000.00");
    assert_eq!(
        refuse_code(&wf),
        "residential_vacancy_fee.consideration_invalid"
    );

    // 7. date_outside_year
    let mut i = base("2025-26", "2024-08-01");
    i["application_fee_payable"] = json!("15100.00");
    assert_eq!(refuse_code(&i), "residential_vacancy_fee.date_outside_year");

    // 8. days_invalid
    let mut i = base("2025-26", "2025-08-01");
    i["residentially_occupied_days"] = json!(366);
    i["application_fee_payable"] = json!("15100.00");
    assert_eq!(refuse_code(&i), "residential_vacancy_fee.days_invalid");
    i["residentially_occupied_days"] = json!(-1);
    assert_eq!(refuse_code(&i), "residential_vacancy_fee.days_invalid");

    // 9. de_minimis_developer_ec_unsettled
    let mut i = base("2026-27", "2026-08-01");
    i["acquisition_basis"] = json!("developer_exemption_certificate");
    i["consideration"] = json!("50000.00");
    i["dwelling_kind"] = json!("new_or_near_new");
    i["application_date"] = json!("2026-08-01");
    assert_eq!(
        refuse_code(&i),
        "residential_vacancy_fee.de_minimis_developer_ec_unsettled"
    );

    // 10. discretionary_waiver_unsupported
    let mut i = base("2025-26", "2025-08-01");
    i["vacancy_fee_waiver_or_remission"] = json!(true);
    i["application_fee_payable"] = json!("15100.00");
    assert_eq!(
        refuse_code(&i),
        "residential_vacancy_fee.discretionary_waiver_unsupported"
    );

    // 11. dwelling_kind_invalid (bad value or established under developer_ec)
    let mut i = base("2026-27", "2026-08-01");
    i["acquisition_basis"] = json!("developer_exemption_certificate");
    i["consideration"] = json!("800000.00");
    i["dwelling_kind"] = json!("established");
    i["application_date"] = json!("2026-08-01");
    assert_eq!(
        refuse_code(&i),
        "residential_vacancy_fee.dwelling_kind_invalid"
    );

    // 12. exemption_invalid
    let mut i = base("2025-26", "2025-08-01");
    i["exemption"] = json!("holiday");
    i["application_fee_payable"] = json!("15100.00");
    assert_eq!(refuse_code(&i), "residential_vacancy_fee.exemption_invalid");

    // 13. other_ec_waiver_unsupported
    let mut i = base("2025-26", "2025-08-01");
    i["acquisition_basis"] = json!("other_exemption_certificate");
    i["application_fee_waiver"] = json!("full");
    assert_eq!(
        refuse_code(&i),
        "residential_vacancy_fee.other_ec_waiver_unsupported"
    );

    // 14. ownership_invalid (bad kind, missing share, share >= 1, share on sole)
    let mut i = base("2025-26", "2025-08-01");
    i["ownership"] = json!("trust");
    i["application_fee_payable"] = json!("15100.00");
    assert_eq!(refuse_code(&i), "residential_vacancy_fee.ownership_invalid");
    let mut tic = base("2026-27", "2026-08-01");
    tic["ownership"] = json!("tenants_in_common");
    tic["consideration"] = json!("800000.00");
    tic["dwelling_kind"] = json!("new_or_near_new");
    tic["application_date"] = json!("2026-08-01");
    assert_eq!(
        refuse_code(&tic),
        "residential_vacancy_fee.ownership_invalid"
    );
    tic["ownership_share"] = json!("1.0");
    assert_eq!(
        refuse_code(&tic),
        "residential_vacancy_fee.ownership_invalid"
    );
    let mut sole_share = base("2026-27", "2026-08-01");
    sole_share["consideration"] = json!("800000.00");
    sole_share["dwelling_kind"] = json!("new_or_near_new");
    sole_share["application_date"] = json!("2026-08-01");
    sole_share["ownership_share"] = json!("0.5");
    assert_eq!(
        refuse_code(&sole_share),
        "residential_vacancy_fee.ownership_invalid"
    );

    // 15. partial_waiver_unsupported
    let mut i = base("2025-26", "2025-08-01");
    i["application_fee_waiver"] = json!("partial");
    i["application_fee_payable"] = json!("15100.00");
    assert_eq!(
        refuse_code(&i),
        "residential_vacancy_fee.partial_waiver_unsupported"
    );

    // 16. pre_2024_waiver_boundary_unsettled (window crosses 1 July 2023 $13,200 -> $14,100)
    let mut i = base("2021-22", "2022-06-15");
    i["application_fee_waiver"] = json!("full");
    assert_eq!(
        refuse_code(&i),
        "residential_vacancy_fee.pre_2024_waiver_boundary_unsettled"
    );

    // 17. waiver_invalid (unknown waiver or full on developer_ec)
    let mut i = base("2025-26", "2025-08-01");
    i["application_fee_waiver"] = json!("maybe");
    i["application_fee_payable"] = json!("15100.00");
    assert_eq!(refuse_code(&i), "residential_vacancy_fee.waiver_invalid");
    let mut dev_full = base("2025-26", "2025-08-01");
    dev_full["acquisition_basis"] = json!("developer_exemption_certificate");
    dev_full["application_fee_waiver"] = json!("full");
    assert_eq!(
        refuse_code(&dev_full),
        "residential_vacancy_fee.waiver_invalid"
    );
}

#[test]
fn unknown_field_and_unsupported_year_are_refused() {
    let mut extra = base("2025-26", "2025-08-01");
    extra["application_fee_payable"] = json!("15100.00");
    extra["unexpected_field"] = json!(true);
    assert_eq!(refuse_code(&extra), "input.field_unknown");

    let mut future = base("2027-28", "2027-08-01");
    future["application_fee_payable"] = json!("15600.00");
    assert_eq!(refuse_code(&future), "period.unsupported");
}
