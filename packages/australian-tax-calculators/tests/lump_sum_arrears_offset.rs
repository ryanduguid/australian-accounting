// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The tax offset for lump sum payments in arrears (ITAA 1936 ss 159ZR to
//! 159ZRD, compilation 193): the ATO's Sudeep, Anthony, Heather and Yan
//! examples, ss 159ZRC and 159ZRD worked by hand, the s 159ZRD(2) and (3)
//! rounding, the earlier years' resident rates and the refusals
//! (docs/calculation-evidence.md#lump-sum-arrears-tax-offset). The examples are
//! from the ATO's "Lump sum payment in arrears" (last updated 8 June 2026),
//! read at 17:19 on 16 November 2026; the ATO labels an income year by the
//! year it ends, so its 2024 is 2023-24. Taxable incomes an example does not
//! give are made up, and the comments work each amount out.
use australian_tax_calculators::api;
use australian_tax_calculators::decimal::Decimal;
use australian_tax_calculators::engine::worksheet_catalogue;
use australian_tax_calculators::metadata::PRIOR_RESIDENT_TAX_SCALES;
use serde_json::{Value, json};

const COMMAND: &str = "lump-sum-arrears-tax-offset";

fn input(year: &str, income: &str, gain: &str, accrual_years: Value) -> Value {
    json!({"taxable_income": income, "net_capital_gain": gain, "accrual_years": accrual_years,
           "year": year, "scope_confirmed": true})
}

fn evaluate(facts: &Value) -> Value {
    let result = api::evaluate(COMMAND, facts).unwrap();
    // derive explains this command: its result is the plain one, byte for byte.
    let doc = api::evaluate("derive", &json!({"command": COMMAND, "input": facts})).unwrap();
    assert_eq!(
        doc["result"].to_string(),
        result.to_string(),
        "derive {facts}"
    );
    result
}

fn refusal(facts: &Value) -> (&'static str, String) {
    let refused = api::evaluate_coded(COMMAND, facts).unwrap_err();
    (refused.code, refused.message)
}

/// Sudeep: $3,500 received in 2025-26 with taxable income of $53,000. Threshold
/// test income is $49,500, whose 10% is $4,950, more than the arrears, so no
/// offset. The example gives no breakdown: 2023-24 is assumed.
#[test]
fn sudeep_does_not_meet_the_threshold_test() {
    let accrual = json!([{"year": "2023-24", "arrears": "3500", "taxable_income": "50000"}]);
    let result = evaluate(&input("2025-26", "53000", "0", accrual));
    assert_eq!(
        result["amounts"],
        json!({"total_arrears_amount": "3500.00", "normal_taxable_income": "53000.00",
               "threshold_test_income": "49500.00",
               "ten_percent_of_threshold_test_income": "4950.00",
               "lump_sum_arrears_tax_offset": "0.00"})
    );
    // An accrual year before the rates the worksheet holds is not needed when
    // the test is not met.
    let accrual = json!([{"year": "2005-06", "arrears": "3500", "taxable_income": "50000"}]);
    let result = evaluate(&input("2025-26", "53000", "0", accrual));
    assert_eq!(result["amounts"]["lump_sum_arrears_tax_offset"], "0.00");
}

/// Anthony: $43,979 received in 2025-26, all accrued in 2016-17, with taxable
/// income of $314,166 including $111,231 of net capital gains. Threshold test
/// income is $314,166 - $43,979 - $111,231 = $158,956, whose 10% is $15,895.60
/// (the ATO shows $15,896). The ATO taxes the arrears at 37% in 2016-17,
/// $16,272, and at 45% in 2025-26, $19,790, for an offset of $3,518. Its
/// 2016-17 taxable income is not given: $100,000 keeps it and the arrears in
/// the 37% band ($87,000 to $180,000), so 0.37 x $43,979 = $16,272.23. Both
/// $314,166 and $270,187 are above $190,000, so 0.45 x $43,979 = $19,790.55.
#[test]
fn anthony_gets_the_difference_between_45_and_37_percent() {
    let accrual = json!([{"year": "2016-17", "arrears": "43979", "taxable_income": "100000"}]);
    let result = evaluate(&input("2025-26", "314166", "111231", accrual));
    assert_eq!(
        result["amounts"],
        json!({"total_arrears_amount": "43979.00", "normal_taxable_income": "202935.00",
               "threshold_test_income": "158956.00",
               "ten_percent_of_threshold_test_income": "15895.60",
               "tax_on_arrears": "19790.55", "notional_tax_most_recent_year": "16272.23",
               "notional_tax_on_arrears": "16272.23",
               "lump_sum_arrears_tax_offset": "3518.32"})
    );
    assert_eq!(result["rates"], json!({"threshold_rate": "0.10"}));
}

/// Heather: $8,500 received in 2025-26, $4,000 for 2022-23 and $4,500 for
/// 2023-24, with taxable income of $34,000: threshold test income $25,500,
/// whose 10% is $2,550, so the test is met. The example is about the Medicare
/// levy and gives no offset. With 2023-24 and 2022-23 taxable incomes of
/// $30,000 and $28,000, the arrears fall in those years' 19% band:
/// $855 and $760. In 2025-26 they fall in the 16% band: $1,360, less than
/// $1,615, so no offset, as the ATO says is common.
#[test]
fn heather_meets_the_test_but_paid_less_tax_now() {
    let accrual = json!([
        {"year": "2022-23", "arrears": "4000", "taxable_income": "28000"},
        {"year": "2023-24", "arrears": "4500", "taxable_income": "30000"},
    ]);
    let result = evaluate(&input("2025-26", "34000", "0", accrual));
    assert_eq!(
        result["amounts"],
        json!({"total_arrears_amount": "8500.00", "normal_taxable_income": "34000.00",
               "threshold_test_income": "25500.00",
               "ten_percent_of_threshold_test_income": "2550.00",
               "tax_on_arrears": "1360.00", "notional_tax_most_recent_year": "855.00",
               "notional_tax_second_most_recent_year": "760.00",
               "notional_tax_on_arrears": "1615.00", "lump_sum_arrears_tax_offset": "0.00"})
    );
}

/// Yan's breakdown, every payer's arrears for a year added together: $3,000
/// for 2023-24, $2,500 for 2022-23, $1,400 for 2021-22, $1,700 for 2020-21,
/// $1,400 for 2019-20 and $1,000 for 2018-19, received in 2025-26. Made up:
/// taxable income of $100,000 in 2025-26, $30,000 in 2023-24 and $17,000 in
/// 2022-23. The 2 latest years are recent (s 159ZR(1)): 0.19 x $3,000 = $570,
/// and 0.19 x ($19,500 - $18,200) = $247 on $2,500, a rate of 0.0988, which
/// s 159ZRD(2) cuts to 0.098 and s 159ZRD(3) raises to 0.099 as the fourth
/// place, 8, is more than 4. The 4 distant years' $5,500 at the average,
/// (0.19 + 0.099) / 2 = 0.1445, is $794.75. Tax on arrears is 0.30 x $11,000 =
/// $3,300; notional tax $570 + $247 + $794.75 = $1,611.75; offset $1,688.25.
fn yan(order: &[usize]) -> Value {
    let years = [
        json!({"year": "2023-24", "arrears": "3000", "taxable_income": "30000"}),
        json!({"year": "2022-23", "arrears": "2500", "taxable_income": "17000"}),
        json!({"year": "2021-22", "arrears": "1400"}),
        json!({"year": "2020-21", "arrears": "1700"}),
        json!({"year": "2019-20", "arrears": "1400"}),
        json!({"year": "2018-19", "arrears": "1000"}),
    ];
    let given: Vec<Value> = order.iter().map(|&i| years[i].clone()).collect();
    input("2025-26", "100000", "0", Value::Array(given))
}

#[test]
fn yans_distant_years_take_the_average_rate_of_the_recent_ones() {
    let result = evaluate(&yan(&[0, 1, 2, 3, 4, 5]));
    assert_eq!(
        result["amounts"],
        json!({"total_arrears_amount": "11000.00", "normal_taxable_income": "100000.00",
               "threshold_test_income": "89000.00",
               "ten_percent_of_threshold_test_income": "8900.00", "tax_on_arrears": "3300.00",
               "notional_tax_most_recent_year": "570.00",
               "notional_tax_second_most_recent_year": "247.00",
               "normal_tax_on_arrears_most_recent_year": "570.00",
               "normal_tax_on_arrears_second_most_recent_year": "247.00",
               "notional_tax_distant_accrual_years": "794.75",
               "notional_tax_on_arrears": "1611.75",
               "lump_sum_arrears_tax_offset": "1688.25"})
    );
    assert_eq!(
        result["rates"],
        json!({"threshold_rate": "0.10", "tax_rate_on_arrears_most_recent_year": "0.19",
               "tax_rate_on_arrears_second_most_recent_year": "0.099",
               "average_tax_rate_on_recent_arrears": "0.1445"})
    );
    // The order the years are given in does not matter.
    let shuffled = evaluate(&yan(&[4, 1, 5, 0, 3, 2]));
    assert_eq!(shuffled["amounts"], result["amounts"]);
    // The derivation writes each year's figures into its working.
    let doc = api::evaluate(
        "derive",
        &json!({"command": COMMAND, "input": yan(&[0, 1, 2, 3, 4, 5])}),
    )
    .unwrap();
    let working = |id: &str| {
        let steps = doc["steps"].as_array().unwrap();
        let step = steps.iter().find(|s| s["id"] == id).unwrap();
        step["working"].as_str().unwrap().to_string()
    };
    assert!(
        working("total_arrears_amount").starts_with("3000 + 2500 + 1400 + 1700 + 1400 + 1000 = ")
    );
    assert!(
        working("notional_tax_distant_accrual_years").starts_with(
            "(1400 + 1700 + 1400 + 1000) x (floor(floor(normal_tax_on_arrears_most_recent_year / 3000 x 10000) / 10 + 0.5) / 1000 + floor(floor(normal_tax_on_arrears_second_most_recent_year / 2500 x 10000) / 10 + 0.5) / 1000) / 2"
        ),
        "{}",
        working("notional_tax_distant_accrual_years")
    );
}

/// s 159ZRD(2) and (3) at the fourth decimal place: in 2023-24, $3,800 of
/// arrears on $16,370 rises to $20,170, $1,970 above $18,200 at 19%, $374.30,
/// a rate of exactly 0.0985, raised to 0.099; on $16,368, $373.92 is 0.0984,
/// kept at 0.098. 2021-22's $1,000 at (0.099 + 0.098) / 2 is $98.50.
#[test]
fn a_recent_rate_is_raised_only_when_its_fourth_decimal_place_is_more_than_4() {
    let accrual = json!([
        {"year": "2023-24", "arrears": "3800", "taxable_income": "16370"},
        {"year": "2022-23", "arrears": "3800", "taxable_income": "16368"},
        {"year": "2021-22", "arrears": "1000"},
    ]);
    let result = evaluate(&input("2025-26", "40000", "0", accrual));
    let amounts = &result["amounts"];
    assert_eq!(amounts["normal_tax_on_arrears_most_recent_year"], "374.30");
    assert_eq!(
        amounts["normal_tax_on_arrears_second_most_recent_year"],
        "373.92"
    );
    assert_eq!(amounts["notional_tax_distant_accrual_years"], "98.50");
    assert_eq!(
        result["rates"]["tax_rate_on_arrears_most_recent_year"],
        "0.099"
    );
    assert_eq!(
        result["rates"]["tax_rate_on_arrears_second_most_recent_year"],
        "0.098"
    );
    assert_eq!(
        result["rates"]["average_tax_rate_on_recent_arrears"],
        "0.0985"
    );
}

/// s 159ZRD uses normal taxable income and s 159ZRC taxable income: with
/// $10,000 of net capital gain in 2023-24's $50,000, its $3,000 of arrears
/// costs 0.325 x $3,000 = $975 of rebated tax but 0.19 x $3,000 = $570 of gross
/// tax on normal taxable income ($40,000 to $43,000), a rate of 0.19. 2022-23
/// is as for Yan: $247 and 0.099. 2021-22's $1,000 at 0.1445 is $144.50. In
/// 2025-26, $6,500 on $60,000 is all in the 30% band: $1,950; notional tax
/// $975 + $247 + $144.50 = $1,366.50; offset $583.50.
#[test]
fn distant_years_use_the_recent_years_normal_taxable_income() {
    let accrual = json!([
        {"year": "2023-24", "arrears": "3000", "taxable_income": "50000",
         "net_capital_gain": "10000"},
        {"year": "2022-23", "arrears": "2500", "taxable_income": "17000"},
        {"year": "2021-22", "arrears": "1000"},
    ]);
    let result = evaluate(&input("2025-26", "60000", "0", accrual));
    let amounts = &result["amounts"];
    assert_eq!(amounts["tax_on_arrears"], "1950.00");
    assert_eq!(amounts["notional_tax_most_recent_year"], "975.00");
    assert_eq!(amounts["normal_tax_on_arrears_most_recent_year"], "570.00");
    assert_eq!(amounts["notional_tax_distant_accrual_years"], "144.50");
    assert_eq!(amounts["lump_sum_arrears_tax_offset"], "583.50");
}

/// s 159ZRC(b) and (c): 2023-24 was itself a rebate year with a $4,000 total
/// arrears amount, deducted, and $1,000 of an earlier rebate year's arrears
/// accrued in it, added: $46,000 - $4,000 + $1,000 = $43,000, so its $3,000
/// is $2,000 at 19% and $1,000 at 32.5%, $705. Without the adjustments it would
/// be 0.325 x $3,000 = $975. In 2025-26, $3,000 on $30,000 at 16% is $480, so
/// no offset.
#[test]
fn a_recent_rebate_year_is_adjusted() {
    let accrual = json!([{"year": "2023-24", "arrears": "3000", "taxable_income": "46000",
                          "rebate_year_arrears": "4000", "earlier_rebate_arrears": "1000"}]);
    let facts = input("2025-26", "30000", "0", accrual);
    let result = evaluate(&facts);
    assert_eq!(result["amounts"]["tax_on_arrears"], "480.00");
    assert_eq!(result["amounts"]["notional_tax_most_recent_year"], "705.00");
    assert_eq!(result["amounts"]["lump_sum_arrears_tax_offset"], "0.00");
    let doc = api::evaluate("derive", &json!({"command": COMMAND, "input": facts})).unwrap();
    let working = doc["steps"][5]["working"].as_str().unwrap();
    assert!(
        working.starts_with("max(0, min(46000 - 4000 + 1000 + 3000, 45000) - 18200) x 0.19"),
        "{working}"
    );
}

/// In 2014-15 to 2016-17 the temporary budget repair levy, 2% above
/// $180,000 (Income Tax (Transitional Provisions) Act 1997 s 4-11), is extra
/// income tax: $10,000 of arrears on $200,000 in 2016-17 is 0.47 x $10,000 =
/// $4,700, and in 2017-18, after the levy, 0.45 x $10,000 = $4,500.
#[test]
fn the_repair_levy_years_add_2_percent_above_180000() {
    for (year, notional) in [("2016-17", "4700.00"), ("2017-18", "4500.00")] {
        let accrual = json!([{"year": year, "arrears": "10000", "taxable_income": "200000"}]);
        let result = evaluate(&input("2025-26", "100000", "0", accrual));
        assert_eq!(
            result["amounts"]["notional_tax_most_recent_year"], notional,
            "{year}"
        );
    }
}

/// The levy also applies in a year of receipt: $30,000 of arrears received in
/// 2014-15 on $200,000 is $10,000 at 37% from $170,000 to $180,000, $3,700,
/// and $20,000 at 45% plus the 2% levy above $180,000, $9,400: $13,100. In
/// 2012-13, $150,000 rises to $180,000 within the 37% band: $11,100. The
/// threshold, 10% of $170,000, is $17,000, so the offset is $2,000.
#[test]
fn the_repair_levy_applies_in_a_year_of_receipt() {
    let accrual = json!([{"year": "2012-13", "arrears": "30000", "taxable_income": "150000"}]);
    let result = evaluate(&input("2014-15", "200000", "0", accrual));
    let amounts = &result["amounts"];
    assert_eq!(amounts["tax_on_arrears"], "13100.00");
    assert_eq!(amounts["notional_tax_most_recent_year"], "11100.00");
    assert_eq!(amounts["lump_sum_arrears_tax_offset"], "2000.00");
}

/// The offset is never more than the tax on the arrears, and nil when the
/// test is not met.
#[test]
fn the_offset_never_exceeds_the_tax_on_the_arrears() {
    let accrual = json!([
        {"year": "2023-24", "arrears": "6000", "taxable_income": "20000"},
        {"year": "2022-23", "arrears": "4000", "taxable_income": "90000"},
        {"year": "2014-15", "arrears": "9000"},
    ]);
    for income in (0..400_000).step_by(7_919) {
        let result = evaluate(&input("2025-26", &income.to_string(), "0", accrual.clone()));
        let amounts = &result["amounts"];
        let offset =
            Decimal::parse(amounts["lump_sum_arrears_tax_offset"].as_str().unwrap()).unwrap();
        match amounts["tax_on_arrears"].as_str() {
            Some(tax) => assert!(
                offset.le(&Decimal::parse(tax).unwrap()).unwrap(),
                "{income}"
            ),
            None => assert!(offset.is_zero(), "{income}"),
        }
    }
}

/// A year, the tax at the start of each band above the first and each band's
/// rate.
type PublishedYear = (&'static str, [(i64, i64); 3], [&'static str; 4]);

/// The ATO's "Tax rates – Australian resident" (last updated 13 August 2026),
/// read at 17:23 on 16 November 2026: for each earlier year, the tax at the
/// start of each band above the first, and each band's rate; 2014-15 to
/// 2016-17 add the 2% levy above $180,000, which the ATO's tables leave out.
#[test]
fn the_earlier_years_rates_are_the_atos() {
    let published: [PublishedYear; 12] = [
        (
            "2012-13",
            [(37000, 3572), (80000, 17547), (180000, 54547)],
            ["0.19", "0.325", "0.37", "0.45"],
        ),
        (
            "2013-14",
            [(37000, 3572), (80000, 17547), (180000, 54547)],
            ["0.19", "0.325", "0.37", "0.45"],
        ),
        (
            "2014-15",
            [(37000, 3572), (80000, 17547), (180000, 54547)],
            ["0.19", "0.325", "0.37", "0.45"],
        ),
        (
            "2015-16",
            [(37000, 3572), (80000, 17547), (180000, 54547)],
            ["0.19", "0.325", "0.37", "0.45"],
        ),
        (
            "2016-17",
            [(37000, 3572), (87000, 19822), (180000, 54232)],
            ["0.19", "0.325", "0.37", "0.45"],
        ),
        (
            "2017-18",
            [(37000, 3572), (87000, 19822), (180000, 54232)],
            ["0.19", "0.325", "0.37", "0.45"],
        ),
        (
            "2018-19",
            [(37000, 3572), (90000, 20797), (180000, 54097)],
            ["0.19", "0.325", "0.37", "0.45"],
        ),
        (
            "2019-20",
            [(37000, 3572), (90000, 20797), (180000, 54097)],
            ["0.19", "0.325", "0.37", "0.45"],
        ),
        (
            "2020-21",
            [(45000, 5092), (120000, 29467), (180000, 51667)],
            ["0.19", "0.325", "0.37", "0.45"],
        ),
        (
            "2021-22",
            [(45000, 5092), (120000, 29467), (180000, 51667)],
            ["0.19", "0.325", "0.37", "0.45"],
        ),
        (
            "2022-23",
            [(45000, 5092), (120000, 29467), (180000, 51667)],
            ["0.19", "0.325", "0.37", "0.45"],
        ),
        (
            "2023-24",
            [(45000, 5092), (120000, 29467), (180000, 51667)],
            ["0.19", "0.325", "0.37", "0.45"],
        ),
    ];
    let years: Vec<&str> = PRIOR_RESIDENT_TAX_SCALES.iter().map(|(y, _)| *y).collect();
    assert_eq!(years, published.map(|(y, _, _)| y));
    for ((year, scale), (_, bases, rates)) in PRIOR_RESIDENT_TAX_SCALES.iter().zip(published) {
        let levy = ["2014-15", "2015-16", "2016-17"].contains(year);
        let (bands, extra) = scale.split_at(4);
        assert_eq!(
            extra,
            if levy {
                &[(180000, None, "0.02")][..]
            } else {
                &[]
            },
            "{year}"
        );
        assert_eq!(bands[0].0, 18200, "{year}: the tax-free threshold");
        let band_rates: Vec<&str> = bands.iter().map(|band| band.2).collect();
        assert_eq!(band_rates, rates, "{year}");
        for pair in bands.windows(2) {
            assert_eq!(pair[0].1, Some(pair[1].0), "{year}: bands abut");
        }
        for (start, tax) in bases {
            let mut total = Decimal::from_int(0);
            for (low, high, rate) in bands {
                let top = high.map_or(start, |high| high.min(start));
                let part = Decimal::from_int((top - low).max(0));
                total = total
                    .add(&part.mul(&Decimal::parse(rate).unwrap()).unwrap())
                    .unwrap();
            }
            assert!(
                total.equals(&Decimal::from_int(tax)).unwrap(),
                "{year} at {start}: {total}"
            );
        }
    }
}

#[test]
fn refusals() {
    let recent = json!({"year": "2023-24", "arrears": "3000", "taxable_income": "30000"});
    let base = input("2025-26", "30000", "0", json!([recent]));
    let with = |key: &str, value: Value| {
        let mut facts = base.clone();
        facts[key] = value;
        refusal(&facts)
    };
    let years = |list: Value| with("accrual_years", list).0;
    assert_eq!(
        with("scope_confirmed", json!(false)).0,
        "scope.not_confirmed"
    );
    assert_eq!(with("year", json!("2012-13")).0, "period.unsupported");
    // RESIDENT_TAX_SCALES holds 2027-28, but the offset has not been checked
    // against the law from that year.
    let (code, message) = with("year", json!("2027-28"));
    assert_eq!(code, "period.unsupported");
    assert!(message.contains("Act No. 49 of 2026"), "{message}");
    assert_eq!(
        with("taxable_income", json!("1.50")).0,
        "input.amount_not_whole_dollars"
    );
    assert_eq!(
        with("net_capital_gain", json!("-1")).0,
        "input.amount_invalid"
    );
    assert_eq!(with("medicare_levy", json!(true)).0, "input.field_unknown");
    let mut missing = base.clone();
    missing.as_object_mut().unwrap().remove("accrual_years");
    assert_eq!(refusal(&missing).0, "input.missing_field");
    let invalid = "lump_sum_arrears.accrual_years_invalid";
    for list in [
        json!({"year": "2023-24", "arrears": "3000"}),
        json!([]),
        json!(["2023-24"]),
        json!([{"year": "2023-2024", "arrears": "3000", "taxable_income": "30000"}]),
        json!([{"year": "2023-25", "arrears": "3000", "taxable_income": "30000"}]),
        json!([{"arrears": "3000", "taxable_income": "30000"}]),
        json!([{"year": "2023-24", "taxable_income": "30000"}]),
        json!([{"year": "2023-24", "arrears": "0", "taxable_income": "30000"}]),
        json!([{"year": "2023-24", "arrears": 3000, "taxable_income": "30000"}]),
        json!([{"year": "2023-24", "arrears": "3000"}]),
        json!([{"year": "2023-24", "arrears": "3000", "taxable_income": "30000", "payer": "ACME"}]),
        json!([{"year": "2025-26", "arrears": "3000", "taxable_income": "30000"}]),
        json!([recent, {"year": "2023-24", "arrears": "1000", "taxable_income": "30000"}]),
        json!([recent, {"year": "2022-23", "arrears": "1000", "taxable_income": "30000",
                        "net_capital_gain": "0"}]),
        json!([recent, {"year": "2022-23", "arrears": "1000", "taxable_income": "30000"},
               {"year": "2021-22", "arrears": "1000", "taxable_income": "30000"}]),
    ] {
        assert_eq!(years(list.clone()), invalid, "{list}");
    }
    assert_eq!(
        with(
            "accrual_years",
            json!([recent, {"year": "2023-24", "arrears": "1"}])
        )
        .1,
        "accrual_years: 2023-24 is given more than once: add every payer's arrears for a year together."
    );
    assert_eq!(
        years(json!([{"year": "2023-24", "arrears": "3000.50", "taxable_income": "30000"}])),
        "input.amount_not_whole_dollars"
    );
    // A recent accrual year needs that year's rates; a distant one does not.
    let old = json!([{"year": "2011-12", "arrears": "3000", "taxable_income": "30000"}]);
    assert_eq!(
        with("accrual_years", old),
        (
            "lump_sum_arrears.recent_year_unsupported",
            "2011-12 is a recent accrual year (ITAA 1936 s 159ZR(1)), whose notional tax s 159ZRC works out at that year's rates, and this worksheet holds resident rates from 2012-13 only".to_string()
        )
    );
    let distant = json!([recent, {"year": "2022-23", "arrears": "1000", "taxable_income": "30000"},
                         {"year": "1999-00", "arrears": "1000"}]);
    let result = evaluate(&input("2025-26", "30000", "0", distant));
    assert_eq!(
        result["amounts"]["notional_tax_distant_accrual_years"],
        "190.00"
    );
}

#[test]
fn catalogue_lists_the_worksheet_and_its_example_runs() {
    let catalogue = worksheet_catalogue();
    let entry = &catalogue["lump_sum_arrears_tax_offset"];
    let years: Vec<&str> = entry["supported_periods"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap())
        .collect();
    assert_eq!(years[..3], ["2013-14", "2014-15", "2015-16"]);
    assert!(years.contains(&"2026-27"));
    let mut facts = entry["example"]["facts"].clone();
    facts.as_object_mut().unwrap().remove("kind");
    assert_eq!(facts, yan(&[0, 1, 2, 3, 4, 5]));
    let result = evaluate(&facts);
    assert_eq!(result["amounts"]["lump_sum_arrears_tax_offset"], "1688.25");
    assert!(entry["optional_inputs"].is_null());
}
