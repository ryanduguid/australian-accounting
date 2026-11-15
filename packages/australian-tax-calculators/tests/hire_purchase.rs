// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The hire purchase schedule (ITAA 1997 s 240-60): agreements whose implicit
//! rate is exactly 10% a month, worked by hand, and fabricated agreements
//! checked against an independent calculation, with the worksheet's refusals
//! (docs/calculation-evidence.md#hire-purchase-schedule).
use australian_tax_calculators::api;
use australian_tax_calculators::decimal::{Decimal, Rounding};
use serde_json::{Value, json};

fn run(overrides: Value) -> Result<Value, String> {
    let mut input = json!({
        "amount_financed": "45000.00", "instalment": "850.00", "instalments": 60,
        "balloon": "5000.00", "timing": "in_arrears", "start_date": "2026-07-15",
        "year": "2026-27", "scope_confirmed": true,
    });
    for (k, v) in overrides.as_object().unwrap() {
        input[k] = v.clone();
    }
    api::evaluate("hire-purchase-schedule", &input)
}

fn row(result: &Value, instalment: usize) -> [&str; 5] {
    let row = &result["schedule"][instalment - 1];
    ["opening", "interest", "payment", "principal", "closing"].map(|k| row[k].as_str().unwrap())
}

fn dates(result: &Value, instalment: usize) -> [&str; 3] {
    let row = &result["schedule"][instalment - 1];
    ["period_start", "period_end", "payment_date"].map(|k| row[k].as_str().unwrap())
}

/// $1,000 financed, $100 a month for 2 months and a $1,000 balloon: at 10% a
/// month the interest is exactly each $100 instalment, so the balance stays at
/// $1,000 until the balloon clears it.
#[test]
fn an_agreement_at_exactly_10_percent_a_month_in_arrears() {
    let r = run(json!({"amount_financed": "1000.00", "instalment": "100.00", "instalments": 2, "balloon": "1000.00"})).unwrap();
    assert_eq!(r["rates"]["implicit_monthly_rate"], "0.100000000000");
    assert_eq!(r["rates"]["nominal_annual_rate"], "1.2000000000");
    assert_eq!(
        row(&r, 1),
        ["1000.00", "100.00", "100.00", "0.00", "1000.00"]
    );
    assert_eq!(
        row(&r, 2),
        ["1000.00", "100.00", "1100.00", "1000.00", "0.00"]
    );
    assert_eq!(r["amounts"]["total_payments"], "1200.00");
    assert_eq!(r["amounts"]["total_interest"], "200.00");
}

/// In advance, step 2 of s 240-60(1) takes the instalment due at the month's
/// start off the principal first: (1,000 - 100) x 10% = 90, then
/// (990 - 100) x 10% = 89, and 890 + 89 = 979 is the balloon at the end.
#[test]
fn an_agreement_at_exactly_10_percent_a_month_in_advance() {
    let r = run(json!({"amount_financed": "1000.00", "instalment": "100.00", "instalments": 2, "balloon": "979.00", "timing": "in_advance"})).unwrap();
    assert_eq!(r["rates"]["implicit_monthly_rate"], "0.100000000000");
    assert_eq!(
        row(&r, 1),
        ["1000.00", "90.00", "100.00", "10.00", "990.00"]
    );
    assert_eq!(row(&r, 2), ["990.00", "89.00", "1079.00", "990.00", "0.00"]);
    assert_eq!(dates(&r, 1), ["2026-07-15", "2026-08-15", "2026-07-15"]);
    assert_eq!(dates(&r, 2), ["2026-08-15", "2026-09-15", "2026-08-15"]);
    assert_eq!(r["balloon_date"], "2026-09-15");
}

/// In advance the last instalment is paid at the start of the last month and
/// the balloon at its end, so the row shows them apart: the instalment as
/// `payment` on `payment_date`, the balloon as `balloon` on `balloon_date`.
#[test]
fn an_in_advance_balloon_is_shown_apart_from_the_last_instalment() {
    let r = run(json!({"amount_financed": "1000.00", "instalment": "100.00", "instalments": 2, "balloon": "979.00", "timing": "in_advance"})).unwrap();
    let (first, last) = (&r["schedule"][0], &r["schedule"][1]);
    assert_eq!(first["balloon"], "0.00");
    assert_eq!(last["payment"], "100.00");
    assert_eq!(last["payment_date"], "2026-08-15");
    assert_eq!(last["balloon"], "979.00");
    assert_eq!(r["balloon_date"], "2026-09-15");
    assert_eq!(last["principal"], "990.00");
}

/// The rates shown reproduce the interest at the largest amount: on
/// $1,000,000,000,000 the first month's interest is the amount financed times
/// the monthly rate, or a twelfth of the annual rate, to the cent.
#[test]
fn the_rates_shown_reproduce_the_first_month_s_interest_to_the_cent() {
    let r = run(json!({"amount_financed": "1000000000000.00", "instalment": "9000000000.00", "instalments": 360, "balloon": "0.00"})).unwrap();
    let dec = |s: &str| Decimal::parse(s).unwrap();
    let rate = |key: &str| dec(r["rates"][key].as_str().unwrap());
    let first = dec(row(&r, 1)[1]);
    let annual = rate("nominal_annual_rate");
    for (monthly, what) in [
        (rate("implicit_monthly_rate"), "monthly"),
        (annual.div(&Decimal::from_int(12)).unwrap(), "annual"),
    ] {
        let gap = dec("1000000000000")
            .mul(&monthly)
            .unwrap()
            .sub(&first)
            .unwrap();
        assert!(
            dec("-0.01").le(&gap).unwrap() && gap.le(&dec("0.01")).unwrap(),
            "{what}: {gap}"
        );
    }
}

/// A fabricated 5-year agreement, against an independent calculation that
/// finds the rate from the payments' present value to 60 digits and rounds
/// the interest to date to cents each month.
#[test]
fn a_5_year_agreement_with_a_balloon_in_arrears() {
    let r = run(json!({})).unwrap();
    assert_eq!(r["rates"]["implicit_monthly_rate"], "0.006866976696");
    assert_eq!(r["rates"]["nominal_annual_rate"], "0.0824037204");
    let schedule = r["schedule"].as_array().unwrap();
    assert_eq!(schedule.len(), 60);
    assert_eq!(
        row(&r, 1),
        ["45000.00", "309.01", "850.00", "540.99", "44459.01"]
    );
    assert_eq!(
        row(&r, 60),
        ["5810.10", "39.90", "5850.00", "5810.10", "0.00"]
    );
    assert_eq!(dates(&r, 1), ["2026-07-15", "2026-08-15", "2026-08-15"]);
    assert_eq!(dates(&r, 60), ["2031-06-15", "2031-07-15", "2031-07-15"]);
    assert_eq!(r["balloon_date"], "2031-07-15");
    assert_eq!(r["amounts"]["total_payments"], "56000.00");
    assert_eq!(r["amounts"]["total_interest"], "11000.00");
}

#[test]
fn the_same_agreement_in_advance() {
    let r = run(json!({"timing": "in_advance"})).unwrap();
    assert_eq!(r["rates"]["implicit_monthly_rate"], "0.007081241014");
    assert_eq!(r["rates"]["nominal_annual_rate"], "0.0849748922");
    assert_eq!(
        row(&r, 1),
        ["45000.00", "312.64", "850.00", "537.36", "44462.64"]
    );
    assert_eq!(
        row(&r, 60),
        ["5814.84", "35.16", "5850.00", "5814.84", "0.00"]
    );
    assert_eq!(dates(&r, 60)[2], "2031-06-15");
    assert_eq!(r["balloon_date"], "2031-07-15");
}

#[test]
fn a_2_year_agreement_without_a_balloon() {
    let r = run(json!({"amount_financed": "10000.00", "instalment": "500.00", "instalments": 24, "balloon": "0.00"})).unwrap();
    assert_eq!(r["rates"]["implicit_monthly_rate"], "0.015130843902");
    assert_eq!(
        row(&r, 1),
        ["10000.00", "151.31", "500.00", "348.69", "9651.31"]
    );
    assert_eq!(
        row(&r, 23),
        ["977.75", "14.80", "500.00", "485.20", "492.55"]
    );
    assert_eq!(row(&r, 24), ["492.55", "7.45", "500.00", "492.55", "0.00"]);
    assert_eq!(r["amounts"]["total_interest"], "2000.00");
}

/// The shortest term, worked by hand at 5% a month: $1,050 in arrears repays
/// $1,000 with $50 interest; in advance, ($1,000 - $100) x 5% = $45, and
/// $900 + $45 = $945 is the balloon at the month's end.
#[test]
fn a_1_month_agreement_in_arrears_and_in_advance() {
    let r = run(json!({"amount_financed": "1000.00", "instalment": "1050.00", "instalments": 1, "balloon": "0.00"})).unwrap();
    assert_eq!(r["rates"]["implicit_monthly_rate"], "0.050000000000");
    assert_eq!(
        row(&r, 1),
        ["1000.00", "50.00", "1050.00", "1000.00", "0.00"]
    );
    let r = run(json!({"amount_financed": "1000.00", "instalment": "100.00", "instalments": 1, "balloon": "945.00", "timing": "in_advance"})).unwrap();
    assert_eq!(r["rates"]["implicit_monthly_rate"], "0.050000000000");
    assert_eq!(
        row(&r, 1),
        ["1000.00", "45.00", "1045.00", "1000.00", "0.00"]
    );
    assert_eq!(dates(&r, 1)[2], "2026-07-15");
    assert_eq!(r["balloon_date"], "2026-08-15");
}

/// LodgeiT Labs publishes, in its public repository, schedules from LodgeiT's
/// printed hire purchase reports at a stated nominal rate, each leaving a
/// residual instead of closing at nil (`tests/fixtures/hp/anchors_v1_4.json`
/// of `lodgeit-labs/clawdog-calculator-api` at e00f6f87, read on
/// 15 November 2026). On the same payments the implicit rate rounds to the
/// stated rate, the first and last months' interest are within 2 cents of the
/// report's, and the total interest is the report's less its residual: the
/// payments less the amount financed. The start dates are moved into a year
/// the worksheet takes; dates do not change the amounts.
#[test]
fn lodgeit_s_published_stated_rate_schedules_close_at_nil_here() {
    let dec = |v: &Value| Decimal::parse(v.as_str().unwrap()).unwrap();
    let text = |s: &str| Decimal::parse(s).unwrap();
    // hp/235 pays a $53,893.16 balloon in place of its 60th $3,893.76
    // instalment: here a 60th instalment and a $49,999.40 balloon.
    for (financed, instalment, count, balloon, stated, first, last, total, residual) in [
        (
            "180630.00",
            "3346.99",
            60,
            "0.00",
            "0.0425",
            "639.73",
            "11.81",
            "20189.64",
            "0.24",
        ),
        (
            "252450.00",
            "3893.76",
            60,
            "49999.40",
            "0.0399",
            "839.40",
            "178.60",
            "31175.69",
            "0.69",
        ),
    ] {
        let r = run(json!({"amount_financed": financed, "instalment": instalment, "instalments": count, "balloon": balloon, "start_date": "2024-07-26", "year": "2024-25"})).unwrap();
        let case = format!("{financed} at {stated}");
        let rate = dec(&r["rates"]["nominal_annual_rate"]);
        let rounded = rate.quantize(-4, Rounding::HalfUp).unwrap();
        assert!(rounded.equals(&text(stated)).unwrap(), "{case}: {rate}");
        for (month, report) in [(1, first), (count as usize, last)] {
            let gap = text(row(&r, month)[1]).sub(&text(report)).unwrap();
            assert!(
                text("-0.02").le(&gap).unwrap() && gap.le(&text("0.02")).unwrap(),
                "{case} month {month}: {gap}"
            );
        }
        let expected = text(total).sub(&text(residual)).unwrap();
        assert!(
            dec(&r["amounts"]["total_interest"])
                .equals(&expected)
                .unwrap(),
            "{case}"
        );
        assert_eq!(row(&r, count as usize)[4], "0.00", "{case}");
    }
}

#[test]
fn payments_that_only_repay_the_amount_financed_carry_no_interest() {
    let r = run(json!({"amount_financed": "1200.00", "instalment": "100.00", "instalments": 12, "balloon": "0.00"})).unwrap();
    assert_eq!(r["rates"]["implicit_monthly_rate"], "0.000000000000");
    assert_eq!(r["amounts"]["total_interest"], "0.00");
    assert!(
        r["schedule"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["interest"] == "0.00")
    );
}

/// A month runs to the same day of the next month, or its last day when that
/// month is shorter, each counted from the start date.
#[test]
fn months_from_the_31st_end_on_each_month_s_last_day() {
    let r = run(json!({"start_date": "2027-01-31", "instalments": 3, "instalment": "15000.00", "balloon": "0.00"})).unwrap();
    assert_eq!(dates(&r, 1), ["2027-01-31", "2027-02-28", "2027-02-28"]);
    assert_eq!(dates(&r, 2), ["2027-02-28", "2027-03-31", "2027-03-31"]);
    assert_eq!(dates(&r, 3), ["2027-03-31", "2027-04-30", "2027-04-30"]);
}

#[test]
fn a_rate_or_term_outside_the_worksheet_is_refused_with_its_code() {
    let code = |overrides: Value| {
        let mut input = json!({
            "amount_financed": "45000.00", "instalment": "850.00", "instalments": 60,
            "balloon": "5000.00", "timing": "in_arrears", "start_date": "2026-07-15",
            "year": "2026-27", "scope_confirmed": true,
        });
        for (k, v) in overrides.as_object().unwrap() {
            input[k] = v.clone();
        }
        api::evaluate_coded("hire-purchase-schedule", &input)
            .unwrap_err()
            .code
    };
    // Payments totalling less than the amount financed imply a negative rate.
    assert_eq!(
        code(json!({"instalment": "10.00", "balloon": "0.00"})),
        "hire_purchase.rate_unsupported"
    );
    // $3,000 a month after one month on $1,000 is 200% a month; $2,000 is exactly 100%.
    for instalment in ["3000.00", "2000.00"] {
        let overrides = json!({"amount_financed": "1000.00", "instalment": instalment, "instalments": 1, "balloon": "0.00"});
        assert_eq!(code(overrides), "hire_purchase.rate_unsupported");
    }
    // About 50% a month for 30 years: the last halving moves the balance by far more than a cent.
    let overrides = json!({"amount_financed": "1000.00", "instalment": "500.00", "instalments": 360, "balloon": "0.00"});
    assert_eq!(code(overrides), "hire_purchase.rate_imprecise");
    for instalments in [json!(0), json!(361), json!("60"), json!(60.5), json!(true)] {
        assert_eq!(
            code(json!({"instalments": instalments})),
            "hire_purchase.instalments_unsupported"
        );
    }
    assert_eq!(
        code(json!({"amount_financed": "0.00"})),
        "hire_purchase.amount_nil"
    );
    assert_eq!(
        code(json!({"instalment": "0.00"})),
        "hire_purchase.amount_nil"
    );
    assert_eq!(code(json!({"timing": "monthly"})), "input.timing_invalid");
    for start in ["2026-06-30", "2027-07-01"] {
        assert_eq!(
            code(json!({"start_date": start})),
            "hire_purchase.start_outside_year"
        );
    }
    assert_eq!(
        code(json!({"start_date": "15/07/2026"})),
        "input.date_invalid"
    );
    assert_eq!(code(json!({"balloon": "-1.00"})), "input.amount_invalid");
    assert_eq!(code(json!({"year": "2023-24"})), "period.unsupported");
    assert_eq!(
        code(json!({"scope_confirmed": false})),
        "scope.not_confirmed"
    );
    assert_eq!(
        code(json!({"interest_rate": "0.08"})),
        "input.field_unknown"
    );
}

#[test]
fn the_longest_term_at_the_largest_amount_still_closes_at_nil() {
    let r = run(json!({"amount_financed": "1000000000000.00", "instalment": "9000000000.00", "instalments": 360, "balloon": "0.00"})).unwrap();
    assert_eq!(r["schedule"].as_array().unwrap().len(), 360);
    assert_eq!(r["rates"]["implicit_monthly_rate"], "0.008585344600");
    assert_eq!(row(&r, 1)[1], "8585344599.77");
    assert_eq!(
        row(&r, 360),
        [
            "8923389625.07",
            "76610374.93",
            "9000000000.00",
            "8923389625.07",
            "0.00"
        ]
    );
    assert_eq!(r["amounts"]["total_interest"], "2240000000000.00");
}

#[test]
fn the_catalogue_lists_the_worksheet_with_a_runnable_example() {
    let catalogue = api::evaluate("worksheet-catalogue", &json!({})).unwrap();
    let entry = &catalogue["hire_purchase_schedule"];
    let mut facts = entry["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    let r = api::evaluate("hire-purchase-schedule", &facts).unwrap();
    assert_eq!(r, run(json!({})).unwrap());
    assert_eq!(
        entry["example_evidence"]["note"],
        "docs/calculation-evidence.md#hire-purchase-schedule"
    );
}
