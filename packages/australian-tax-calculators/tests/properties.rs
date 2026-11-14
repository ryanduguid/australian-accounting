// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Seeded properties that hold for every input the worksheets accept. The
//! seeds, run counts, explicit examples and input domains are the Python
//! fast-check properties'; the generator is a local SplitMix64.
use australian_tax_calculators::api;
use australian_tax_calculators::decimal::Decimal;
use australian_tax_calculators::metadata::{
    PAYG_WITHHOLDING_COEFFICIENTS, RESIDENT_TAX_SCALES, STUDY_LOAN_WITHHOLDING_FROM,
    supported_periods,
};
use serde_json::{Value, json};

mod common;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `low..=high`.
    fn int(&mut self, low: i64, high: i64) -> i64 {
        low + (self.next() % (high - low + 1) as u64) as i64
    }

    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.int(0, items.len() as i64 - 1) as usize]
    }

    fn bool(&mut self) -> bool {
        self.next() & 1 == 1
    }

    /// Two-decimal money up to `max_cents`, biased to the ends of the range.
    fn cents(&mut self, max_cents: i64) -> String {
        let cents = match self.int(0, 9) {
            0 => self.int(0, 100),
            1 => max_cents - self.int(0, 100),
            _ => self.int(0, max_cents),
        };
        format!("{}.{:02}", cents / 100, cents % 100)
    }

    fn money(&mut self) -> String {
        self.cents(10_000_000_000)
    }

    fn pay(&mut self) -> String {
        self.cents(10_000_000)
    }

    fn dollars(&mut self) -> i64 {
        match self.int(0, 9) {
            0 => self.int(0, 100),
            _ => self.int(0, 2_000_000),
        }
    }
}

fn d(text: &str) -> Decimal {
    Decimal::parse(text).unwrap()
}

fn n(value: i64) -> Decimal {
    Decimal::from_int(value)
}

fn run(command: &str, input: Value) -> Value {
    let result =
        api::evaluate(command, &input).unwrap_or_else(|e| panic!("{command} {input}: {e}"));
    check_sanity(command, &result);
    result
}

/// What every worksheet result must say, whatever its inputs: it names its
/// period, scope and sources, and every amount is whole cents and not negative.
fn check_sanity(command: &str, result: &Value) {
    assert_eq!(result["ok"], json!(true), "{command} {result}");
    for key in ["period", "scope"] {
        let text = result[key].as_str().unwrap_or_default();
        assert!(!text.is_empty(), "{command} has no {key}: {result}");
    }
    let sources = result["sources"].as_array();
    assert!(
        sources.is_some_and(|s| !s.is_empty()),
        "{command} cites nothing: {result}"
    );
    for (name, value) in result["amounts"].as_object().unwrap() {
        if value.is_null() {
            continue;
        }
        let text = value
            .as_str()
            .unwrap_or_else(|| panic!("{command} {name} is {value}"));
        let (whole, cents) = text
            .split_once('.')
            .unwrap_or_else(|| panic!("{command} {name} is {text}"));
        assert!(
            cents.len() == 2
                && whole
                    .chars()
                    .chain(cents.chars())
                    .all(|c| c.is_ascii_digit()),
            "{command} {name} is {text}"
        );
    }
}

fn amount(result: &Value, key: &str) -> Decimal {
    d(result["amounts"][key].as_str().unwrap())
}

fn le(a: &Decimal, b: &Decimal) -> bool {
    a.le(b).unwrap()
}

fn eq(a: &Decimal, b: &Decimal) -> bool {
    a.equals(b).unwrap()
}

fn sorted<T: Ord>(a: T, b: T) -> (T, T) {
    if b < a { (b, a) } else { (a, b) }
}

fn sorted_dec(a: String, b: String) -> (String, String) {
    if d(&b).lt(&d(&a)).unwrap() {
        (b, a)
    } else {
        (a, b)
    }
}

const RUNS: usize = 100;

fn check_gst(amount_text: &str, inclusive: bool) {
    let result = run(
        "gst",
        json!({"amount": amount_text, "gst_inclusive": inclusive,
               "scope_confirmed": true, "year": "2025-26"}),
    );
    let (gst, exclusive, total) = (
        amount(&result, "gst"),
        amount(&result, "exclusive"),
        amount(&result, "inclusive"),
    );
    assert!(eq(&gst.add(&exclusive).unwrap(), &total), "{amount_text}");
    let given = if inclusive { &total } else { &exclusive };
    assert!(eq(given, &d(amount_text)), "{amount_text}");
    let share = if inclusive {
        total.div(&n(11)).unwrap()
    } else {
        exclusive.div(&n(10)).unwrap()
    };
    let gap = gst.sub(&share).unwrap();
    assert!(
        le(&gap, &d("0.005")) && le(&d("-0.005"), &gap),
        "{amount_text}"
    );
}

#[test]
fn gst_and_the_exclusive_amount_add_to_the_inclusive_amount_to_the_cent() {
    check_gst("0.06", true);
    check_gst("0.05", false);
    let mut rng = Rng(0x6571);
    for _ in 0..RUNS {
        let (text, inclusive) = (rng.money(), rng.bool());
        check_gst(&text, inclusive);
    }
}

fn tax(income: i64, year: &str) -> Decimal {
    let result = run(
        "resident-tax",
        json!({"taxable_income": income.to_string(), "year": year, "scope_confirmed": true}),
    );
    amount(&result, "basic_income_tax")
}

fn check_resident_tax(year: &str, low: i64, high: i64) {
    let (low, high) = sorted(low, high);
    let scale = RESIDENT_TAX_SCALES
        .iter()
        .find(|(y, _)| *y == year)
        .unwrap()
        .1;
    let mut top_rate = n(0);
    for (_, _, rate) in scale {
        if top_rate.lt(&d(rate)).unwrap() {
            top_rate = d(rate);
        }
    }
    let (t_low, t_high) = (tax(low, year), tax(high, year));
    assert!(le(&n(0), &t_low) && le(&t_low, &t_high));
    assert!(
        le(&t_high, &n(high).mul(&top_rate).unwrap()),
        "{year} {high}"
    );
    let step = |x: i64| tax(x + 1, year).sub(&tax(x, year)).unwrap();
    assert!(le(&step(low), &step(high)), "{year} {low} {high}");
    if low <= scale[0].0 {
        assert!(t_low.is_zero(), "{year} {low}");
    }
}

#[test]
fn resident_tax_never_falls_and_its_marginal_rate_never_falls() {
    check_resident_tax("2026-27", 18200, 18201);
    check_resident_tax("2026-27", 45000, 45001);
    let years = supported_periods("resident_tax");
    let mut rng = Rng(0x7A11);
    for _ in 0..RUNS {
        let year = *rng.pick(&years);
        let (low, high) = (rng.dollars(), rng.dollars());
        check_resident_tax(year, low, high);
    }
}

/// `foreign` is scale 4's residency, `None` on the other scales.
fn withheld(
    earnings: &str,
    period: &str,
    scale: i64,
    year: &str,
    foreign: Option<bool>,
) -> Decimal {
    let mut input = json!({"earnings": earnings, "pay_period": period, "scale": scale,
                           "year": year, "scope_confirmed": true});
    if let Some(foreign) = foreign {
        input["foreign_resident"] = json!(foreign);
    }
    amount(&run("payg-withholding", input), "withholding")
}

/// Every year's scales, with scale 4 for a resident and a foreign resident.
fn payg_scales() -> Vec<(&'static str, i64, Option<bool>)> {
    PAYG_WITHHOLDING_COEFFICIENTS
        .iter()
        .flat_map(|(year, scales)| {
            let no_tfn = [false, true].map(|foreign| (*year, 4, Some(foreign)));
            scales
                .iter()
                .map(move |(scale, _)| (*year, *scale, None))
                .chain(no_tfn)
        })
        .collect()
}

#[test]
fn payg_withholding_never_falls_as_earnings_rise() {
    let check = |(year, scale, foreign): (&str, i64, Option<bool>),
                 period: &str,
                 low: String,
                 high: String| {
        let (low, high) = sorted_dec(low, high);
        let (a, b) = (
            withheld(&low, period, scale, year, foreign),
            withheld(&high, period, scale, year, foreign),
        );
        assert!(
            le(&n(0), &a) && le(&a, &b),
            "{year} {scale} {foreign:?} {period} {low} {high}"
        );
    };
    check(
        ("2026-27", 2, None),
        "monthly",
        "3000.32".into(),
        "3000.33".into(),
    );
    check(
        ("2026-27", 4, Some(false)),
        "weekly",
        "2.99".into(),
        "3.00".into(),
    );
    let scales = payg_scales();
    let mut rng = Rng(0x9A76);
    for _ in 0..RUNS {
        let year_scale = *rng.pick(&scales);
        let period = *rng.pick(&["weekly", "fortnightly", "monthly", "quarterly"]);
        let (low, high) = (rng.pay(), rng.pay());
        check(year_scale, period, low, high);
    }
}

#[test]
fn payg_withholding_never_falls_across_any_bracket_boundary() {
    // Schedule 1 works on whole-dollar weekly earnings. Whole-dollar steps of a
    // weekly, fortnightly or monthly payment move those by at most $1, so this
    // sweep reaches every weekly amount, and every bracket edge, of each scale.
    for (year, scales) in PAYG_WITHHOLDING_COEFFICIENTS {
        for (scale, rows) in *scales {
            let last_limit = rows.iter().filter_map(|r| r.0).max().unwrap();
            for (period, weeks) in [("weekly", 1), ("fortnightly", 2), ("monthly", 5)] {
                let mut previous = n(0);
                for dollars in 0..(last_limit + 50) * weeks {
                    let amount = withheld(&dollars.to_string(), period, *scale, year, None);
                    assert!(le(&previous, &amount), "{year} {scale} {period} {dollars}");
                    previous = amount;
                }
            }
        }
    }
}

fn check_study_loan(year: &str, low: i64, high: i64) {
    let (low, high) = sorted(low, high);
    let repayment = |income: i64| {
        let result = run(
            "study-loan-repayment",
            json!({"repayment_income": income.to_string(), "year": year, "scope_confirmed": true}),
        );
        amount(&result, "compulsory_repayment")
    };
    let (a, b) = (repayment(low), repayment(high));
    assert!(le(&n(0), &a) && le(&a, &b), "{year} {low} {high}");
    assert!(le(&b, &n(high).div(&n(10)).unwrap()), "{year} {high}");
}

#[test]
fn study_loan_repayment_never_falls_and_never_exceeds_a_tenth() {
    check_study_loan("2026-27", 69528, 69529);
    check_study_loan("2026-27", 186050, 186051);
    let years = supported_periods("study_loan_repayment");
    let mut rng = Rng(0x5714);
    for _ in 0..RUNS {
        let year = *rng.pick(&years);
        let (low, high) = (rng.dollars(), rng.dollars());
        check_study_loan(year, low, high);
    }
}

fn check_pension(year: &str, low: String, high: String, younger: i64, older: i64, days: i64) {
    let (low, high) = sorted_dec(low, high);
    let (younger, older) = sorted(younger, older);
    let minimum = |balance: &str, age: i64| {
        let result = run(
            "pension-minimum",
            json!({"account_balance": balance, "age": age, "days": days,
                   "year": year, "scope_confirmed": true}),
        );
        amount(&result, "minimum_payment")
    };
    let (a, b, c) = (
        minimum(&low, younger),
        minimum(&high, younger),
        minimum(&high, older),
    );
    assert!(
        le(&a, &b) && le(&b, &c),
        "{year} {low} {high} {younger} {older} {days}"
    );
    assert!(c.rem(&n(10)).unwrap().is_zero(), "{c}");
}

#[test]
fn pension_minimum_never_falls_with_balance_or_age() {
    check_pension(
        "2025-26",
        "100000.00".into(),
        "100000.00".into(),
        64,
        65,
        365,
    );
    let years = supported_periods("pension_minimum");
    let mut rng = Rng(0x9E45);
    for _ in 0..RUNS {
        let year = *rng.pick(&years);
        let (low, high) = (rng.money(), rng.money());
        let (younger, older, days) = (rng.int(0, 150), rng.int(0, 150), rng.int(1, 365));
        check_pension(year, low, high, younger, older, days);
    }
}

fn check_capital_losses(other: &str, discount: &str, current: &str, prior: &str, extra: &str) {
    let gains = |prior: &str| {
        run(
            "capital-gains",
            json!({"other_gains": other, "discount_gains": discount,
                   "current_losses": current, "prior_losses": prior,
                   "scope_confirmed": true, "year": "2025-26"}),
        )
    };
    let more_prior = d(prior).add(&d(extra)).unwrap().to_string();
    let (first, more) = (gains(prior), gains(&more_prior));
    let losses = d(current).add(&d(prior)).unwrap();
    let used = amount(&first, "losses_used");
    let gross = d(other).add(&d(discount)).unwrap();
    assert!(eq(
        &used.add(&amount(&first, "losses_remaining")).unwrap(),
        &losses
    ));
    assert!(le(&used, &gross));
    let (net_first, net_more) = (
        amount(&first, "net_capital_gain"),
        amount(&more, "net_capital_gain"),
    );
    assert!(le(&net_more, &net_first) && le(&net_first, &gross));
}

#[test]
fn capital_losses_are_all_accounted_for_and_more_losses_never_raise_the_gain() {
    check_capital_losses("100.00", "100.00", "150.00", "0.00", "0.01");
    let mut rng = Rng(0xC671);
    for _ in 0..RUNS {
        let v: Vec<String> = (0..5).map(|_| rng.money()).collect();
        check_capital_losses(&v[0], &v[1], &v[2], &v[3], &v[4]);
    }
}

#[test]
fn cap_room_and_excess_reconcile_to_the_cap_available() {
    let years = supported_periods("contribution_caps");
    let mut rng = Rng(0xCA95);
    for _ in 0..RUNS {
        let year = *rng.pick(&years);
        let (balance, concessional, unused, non_concessional) =
            (rng.money(), rng.money(), rng.money(), rng.money());
        let result = run(
            "contribution-caps",
            json!({"total_super_balance": balance, "concessional_contributions": concessional,
                   "unused_concessional_cap": unused,
                   "non_concessional_contributions": non_concessional,
                   "under_75_in_year": rng.bool(), "year": year, "scope_confirmed": true}),
        );
        for (kind, contributed) in [
            ("concessional", &concessional),
            ("non_concessional", &non_concessional),
        ] {
            let remaining = amount(&result, &format!("{kind}_remaining"));
            let excess = amount(&result, &format!("excess_{kind}"));
            let available = amount(&result, &format!("{kind}_available"));
            assert!(eq(
                &remaining.sub(&excess).unwrap(),
                &available.sub(&d(contributed)).unwrap()
            ));
            assert!(remaining.is_zero() || excess.is_zero(), "{result}");
        }
    }
}

#[test]
fn carry_forward_applies_only_the_unused_cap_the_excess_needs() {
    // s 291-20(3)-(4): 2025-26 cap $30,000 and $15,000 unused.
    for (balance, concessional, applied, excess) in [
        ("450000", "20000", "0.00", "0.00"),
        ("450000", "30000", "0.00", "0.00"),
        ("450000", "40000", "10000.00", "0.00"),
        ("450000", "50000", "15000.00", "5000.00"),
        ("499999.99", "40000", "10000.00", "0.00"),
        ("500000", "40000", "0.00", "10000.00"),
    ] {
        let result = run(
            "contribution-caps",
            json!({"total_super_balance": balance, "concessional_contributions": concessional,
                   "unused_concessional_cap": "15000", "non_concessional_contributions": "0",
                   "under_75_in_year": true, "year": "2025-26", "scope_confirmed": true}),
        );
        assert_eq!(
            result["amounts"]["carry_forward_applied"], applied,
            "{result}"
        );
        assert_eq!(result["amounts"]["excess_concessional"], excess, "{result}");
    }
}

// Whole-range sweeps from the Python tests. The frozen corpus keeps only their
// first 60 calls, so each sweep runs here in full.

#[test]
fn division_293_tax_never_exceeds_15_percent_of_contributions() {
    for income in (200_000..=300_000).step_by(2500) {
        for contributions in ["0", "1000", "25000", "60000"] {
            let result = run(
                "division-293",
                json!({"division_293_income": income.to_string(),
                       "low_tax_contributions": contributions,
                       "year": "2025-26", "scope_confirmed": true}),
            );
            let tax = amount(&result, "division_293_tax");
            let limit = d(contributions).mul(&d("0.15")).unwrap();
            assert!(
                le(&n(0), &tax) && le(&tax, &limit),
                "{income} {contributions}"
            );
        }
    }
}

#[test]
fn low_income_tax_offset_never_rises_with_income() {
    let mut previous = n(700);
    for income in (37_000..=67_000).step_by(7) {
        let result = run(
            "low-income-tax-offset",
            json!({"taxable_income": income.to_string(), "year": "2025-26",
                   "scope_confirmed": true}),
        );
        let offset = amount(&result, "low_income_tax_offset");
        assert!(le(&n(0), &offset) && le(&offset, &previous), "{income}");
        previous = offset;
    }
}

#[test]
fn medicare_levy_never_falls_as_income_rises() {
    for (year, lower, upper) in [
        ("2024-25", 27222, 34027),
        ("2025-26", 28011, 35013),
        ("2026-27", 28011, 35013),
    ] {
        let mut previous = n(0);
        for income in lower - 2..upper + 3 {
            let result = run(
                "medicare-levy",
                json!({"taxable_income": income.to_string(),
                       "section_9a_adjustment_applies": false,
                       "year": year, "scope_confirmed": true}),
            );
            let levy = amount(&result, "medicare_levy");
            let limit = n(income).mul(&d("0.02")).unwrap();
            assert!(le(&previous, &levy) && le(&levy, &limit), "{year} {income}");
            previous = levy;
        }
    }
}

/// Medicare Levy Act s 9(1): whatever the income and family facts, another
/// exemption day never raises the levy, and a full day never leaves more than
/// a half one.
#[test]
fn medicare_levy_never_rises_with_exemption_days() {
    let mut rng = Rng(0x9E1D);
    for _ in 0..400 {
        let year = *rng.pick(&["2024-25", "2025-26", "2026-27"]);
        let mut facts = json!({"taxable_income": rng.int(0, 150_000).to_string(),
                               "section_9a_adjustment_applies": false,
                               "sapto_entitled": rng.bool(),
                               "year": year, "scope_confirmed": true});
        if rng.bool() {
            facts["spouse_taxable_income"] = json!(rng.int(0, 90_000).to_string());
            facts["spouse_sapto_entitled"] = json!(rng.bool());
            facts["dependent_children"] = json!(rng.int(0, 4));
        }
        let levy = |full: i64, half: i64| {
            let mut input = facts.clone();
            input["medicare_levy_exemption_full_days"] = json!(full);
            input["medicare_levy_exemption_half_days"] = json!(half);
            amount(&run("medicare-levy", input), "medicare_levy")
        };
        let full = rng.int(0, 364);
        let half = rng.int(0, 364 - full);
        let (now, more_full, more_half) =
            (levy(full, half), levy(full + 1, half), levy(full, half + 1));
        assert!(
            le(&more_half, &now) && le(&more_full, &more_half),
            "{facts} {full} {half}"
        );
    }
}

/// Medicare levy surcharge: with income for surcharge purposes, and so the
/// tier, held, the surcharge never falls as taxable income, reportable fringe
/// benefits or the days without cover rise.
#[test]
fn medicare_levy_surcharge_never_falls_as_its_base_or_days_rise() {
    let mut rng = Rng(0x3B5C);
    for _ in 0..400 {
        let year = *rng.pick(&["2024-25", "2025-26", "2026-27"]);
        let income = rng.int(0, 400_000);
        let mut facts = json!({"income_for_surcharge_purposes": income.to_string(),
                               "year": year, "scope_confirmed": true});
        if rng.bool() {
            facts["spouse_income_for_surcharge_purposes"] = json!(rng.int(0, 300_000).to_string());
            facts["dependent_children"] = json!(rng.int(0, 5));
        }
        let surcharge = |taxable: i64, fringe: i64, days: i64| {
            let mut input = facts.clone();
            input["taxable_income"] = json!(taxable.to_string());
            input["reportable_fringe_benefits"] = json!(fringe.to_string());
            input["days_without_cover"] = json!(days);
            amount(
                &run("medicare-levy-surcharge", input),
                "medicare_levy_surcharge",
            )
        };
        let (taxable, fringe, days) = (rng.int(0, income), rng.int(0, 50_000), rng.int(0, 364));
        let now = surcharge(taxable, fringe, days);
        for later in [
            surcharge(taxable + 1, fringe, days),
            surcharge(taxable, fringe + 1, days),
            surcharge(taxable, fringe, days + 1),
        ] {
            assert!(le(&now, &later), "{facts} {taxable} {fringe} {days}");
        }
    }
}

#[test]
fn study_loan_components_agree_with_the_combined_coefficients_to_a_dollar() {
    // The ATO says the two methods can differ slightly through rounding and accepts
    // either. Every whole-dollar weekly amount to $4,000 stays within $1.
    for (year, scale, _, rows) in common::COMBINED {
        let (_, first_pay_date) = STUDY_LOAN_WITHHOLDING_FROM
            .iter()
            .find(|(y, _)| *y == year)
            .unwrap();
        let mut saw_zero = false;
        for dollars in 0..=4000 {
            let x = n(dollars).add(&d("0.99")).unwrap();
            let (_, a, b) = rows
                .iter()
                .find(|(top, _, _)| top.is_none_or(|top| x.lt(&n(top)).unwrap()))
                .unwrap();
            let formula = d(a).mul(&x).unwrap().sub(&d(b)).unwrap();
            let rounded = formula
                .quantize(0, australian_tax_calculators::decimal::Rounding::HalfUp)
                .unwrap();
            let expected = if rounded.lt(&n(0)).unwrap() {
                n(0)
            } else {
                rounded
            };
            let result = run(
                "study-loan-withholding",
                json!({"earnings": dollars.to_string(), "pay_period": "weekly",
                       "scale": scale, "year": year, "scope_confirmed": true,
                       "pay_date": first_pay_date}),
            );
            let gap = amount(&result, "withholding").sub(&expected).unwrap();
            assert!(
                le(&n(-1), &gap) && le(&gap, &n(1)),
                "{year} {scale} {dollars}"
            );
            saw_zero |= gap.is_zero();
        }
        assert!(saw_zero, "{year} {scale}");
    }
}

#[test]
fn study_loan_withholding_never_falls_as_weekly_earnings_rise() {
    // Every whole-dollar weekly amount to $4,000 passes every Schedule 1 and
    // Schedule 8 bracket edge of each year's tables.
    for (year, first_pay_date) in STUDY_LOAN_WITHHOLDING_FROM {
        for scale in [1, 2, 3, 5, 6] {
            let mut previous = n(0);
            for dollars in 0..=4000 {
                let result = run(
                    "study-loan-withholding",
                    json!({"earnings": dollars.to_string(), "pay_period": "weekly",
                           "scale": scale, "year": year, "scope_confirmed": true,
                           "pay_date": first_pay_date}),
                );
                let withheld = amount(&result, "withholding");
                assert!(le(&previous, &withheld), "{year} {scale} {dollars}");
                previous = withheld;
            }
        }
    }
}

#[test]
fn quarterly_study_loan_withholding_is_13_weekly_amounts() {
    for (year, first_pay_date) in STUDY_LOAN_WITHHOLDING_FROM {
        for scale in [1, 2, 3, 5, 6] {
            for dollars in (0..=4000).step_by(7) {
                let pay = |earnings: i64, period| {
                    let input = json!({"earnings": earnings.to_string(), "pay_period": period,
                                       "scale": scale, "year": year, "scope_confirmed": true,
                                       "pay_date": first_pay_date});
                    amount(&run("study-loan-withholding", input), "withholding")
                };
                let weekly = pay(dollars, "weekly").mul(&n(13)).unwrap();
                let gap = pay(dollars * 13, "quarterly").sub(&weekly).unwrap();
                assert!(gap.is_zero(), "{year} {scale} {dollars}");
            }
        }
    }
}

/// Every car result reconciles to the cent: the applied and excess
/// contribution make up the contribution, the applied part and the value
/// before exemption make up the gross value, that value is also the reportable
/// taxable value, and an exempt car is nil.
fn check_car_reconciles(result: &Value, contribution: &str, exempt: bool) {
    let gross = amount(result, "gross_taxable_value");
    let applied = amount(result, "employee_contribution_applied");
    let before = amount(result, "taxable_value_before_exemption");
    let total = applied
        .add(&amount(result, "excess_employee_contribution"))
        .unwrap();
    assert!(eq(&total, &d(contribution)), "{result}");
    // Under s 10(5) the operating cost value gives way to a lower statutory value.
    let own = match result["amounts"].get("operating_cost_taxable_value") {
        Some(_) => {
            let own = amount(result, "operating_cost_taxable_value");
            let statutory = amount(result, "statutory_formula_taxable_value");
            let lower = if le(&own, &statutory) {
                &own
            } else {
                &statutory
            };
            assert!(eq(&before, lower), "{result}");
            own
        }
        None => before.clone(),
    };
    assert!(eq(&applied.add(&own).unwrap(), &gross), "{result}");
    assert!(le(&n(0), &before), "{result}");
    // The reportable taxable value ignores the exemption.
    assert!(
        eq(&amount(result, "reportable_taxable_value"), &before),
        "{result}"
    );
    let expected = if exempt { n(0) } else { before };
    assert!(eq(&amount(result, "taxable_value"), &expected), "{result}");
}

#[test]
fn car_worksheets_reconcile_and_stay_within_their_bases() {
    let mut rng = Rng(0xCA45);
    for _ in 0..RUNS {
        let year = rng.int(2026, 2027);
        let (contribution, exempt) = (rng.pay(), rng.bool());
        let (cost, accessories) = (rng.pay(), rng.pay());
        let (days, reduce) = (rng.int(0, 365), rng.bool());
        let result = run(
            "fbt-car-statutory",
            json!({"cost_price": cost, "later_accessories": accessories,
                   "one_third_reduction": reduce, "days_available": days,
                   "employee_contribution": contribution, "electric_car_exempt": exempt,
                   "year_ended": year, "scope_confirmed": true}),
        );
        check_car_reconciles(&result, &contribution, exempt);
        // The base never exceeds cost plus accessories, nor the gross value 20% of
        // it, allowing a cent for the unrounded base behind the displayed one.
        let base = amount(&result, "base_value");
        assert!(
            le(&base, &d(&cost).add(&d(&accessories)).unwrap()),
            "{result}"
        );
        let ceiling = base.mul(&d("0.20")).unwrap().add(&d("0.01")).unwrap();
        assert!(
            le(&amount(&result, "gross_taxable_value"), &ceiling),
            "{result}"
        );

        let owned = rng.bool();
        let (running, value) = (rng.pay(), rng.pay());
        let lease = if owned { "0.00".to_string() } else { rng.pay() };
        let value = if owned { value } else { "0.00".to_string() };
        let business = format!("0.{:04}", rng.int(0, 9999));
        let result = run(
            "fbt-car-operating-cost",
            json!({"running_costs": running, "lease_charges": lease, "depreciated_value": value,
                   "effective_life_years": "8",
                   "days_held": rng.int(0, 365), "business_use": business,
                   "employee_contribution": contribution,
                   "statutory_formula_taxable_value": rng.pay(), "electric_car_exempt": exempt,
                   "year_ended": year, "scope_confirmed": true}),
        );
        check_car_reconciles(&result, &contribution, exempt);
        // The displayed parts add to the operating costs within a cent, since the
        // deemed amounts are rounded only for display.
        let parts = ["deemed_depreciation", "deemed_interest"]
            .iter()
            .fold(d(&running).add(&d(&lease)).unwrap(), |sum, key| {
                sum.add(&amount(&result, key)).unwrap()
            });
        let gap = parts.sub(&amount(&result, "operating_costs")).unwrap();
        assert!(le(&d("-0.01"), &gap) && le(&gap, &d("0.01")), "{result}");
        let costs = amount(&result, "operating_costs").add(&d("0.005")).unwrap();
        assert!(
            le(&amount(&result, "gross_taxable_value"), &costs),
            "{result}"
        );
    }
}

/// Random alternating lived and absent runs, income use on some days and
/// choice periods after some lived runs. Inputs the worksheet refuses as out
/// of scope are skipped; the market value basis is retried when it applies.
#[test]
fn main_residence_shares_reconcile_and_stay_within_the_gain() {
    let mut rng = Rng(20_261_101);
    let mut checked = 0;
    for _ in 0..300 {
        let start = 10_000 + rng.int(0, 3000); // days from 1970: 1997 to 2005
        let mut day = start;
        let (mut lived, mut income, mut chosen) = (vec![], vec![], vec![]);
        let date = |d: i64| {
            let (y, m, dd) = civil(d);
            format!("{y:04}-{m:02}-{dd:02}")
        };
        let mut was_lived = false;
        for _ in 0..rng.int(1, 6) {
            let len = rng.int(1, 3000);
            let (a, b) = (day, day + len - 1);
            let is_lived = !was_lived && rng.bool();
            if is_lived {
                lived.push(json!({"start": date(a), "end": date(b)}));
                if rng.int(0, 3) == 0 {
                    income.push(json!({"start": date(a), "end": date(b), "floor_area": "0.25"}));
                }
            } else {
                if was_lived && rng.bool() {
                    chosen.push(json!({"start": date(a), "end": date(b)}));
                }
                if rng.bool() {
                    income.push(json!({"start": date(a), "end": date(b), "floor_area": "1"}));
                }
            }
            was_lived = is_lived;
            day = b + 1;
        }
        let end = day.max(20_271) - 1; // 2025-07-01 or later
        let gain = rng.money();
        let mut input = json!({
            "capital_gain": gain, "capital_loss": "0.00", "market_value_basis": false,
            "ownership_start": date(start), "ownership_end": date(end),
            "lived_periods": lived, "income_use_periods": income, "absence_choice_periods": chosen,
            "new_home_acquired": if rng.bool() { json!(date(end - rng.int(0, 400))) } else { json!(null) },
            "year": "2025-26", "scope_confirmed": true,
        });
        let mut outcome = api::evaluate("main-residence", &input);
        if matches!(&outcome, Err(m) if m.contains("market value on")) {
            input["market_value_basis"] = json!(true);
            outcome = api::evaluate("main-residence", &input);
        }
        let Ok(r) = outcome else { continue };
        let (taxable, exempt) = (
            amount(&r, "assessable_capital_gain"),
            amount(&r, "exempt_capital_gain"),
        );
        assert!(eq(&taxable.add(&exempt).unwrap(), &d(&gain)), "{input}");
        assert!(le(&n(0), &taxable) && le(&taxable, &d(&gain)), "{input}");
        let days = d(r["rates"]["ownership_period_days"].as_str().unwrap());
        let weighted = d(r["rates"]["non_main_residence_days"].as_str().unwrap());
        assert!(le(&n(0), &weighted) && le(&weighted, &days), "{input}");
        checked += 1;
    }
    assert!(checked > 150, "only {checked} accepted inputs");
}

/// Loans whose whole period falls in 2024-25 to 2026-27, ended by the contract
/// or by repayment: each year's remaining expenditure is the last year's less
/// its maximum deduction (s 25-25(4) step 1), the maximums add up to the
/// expenditure, and no deduction exceeds its maximum.
#[test]
fn borrowing_expense_maximums_carry_forward_and_add_up() {
    let mut rng = Rng(20_261_104);
    let years = ["2024-25", "2025-26", "2026-27"];
    let year_of = |day: i64| (day >= 20_270) as usize + (day >= 20_635) as usize;
    let date = |d: i64| {
        let (y, m, dd) = civil(d);
        format!("{y:04}-{m:02}-{dd:02}")
    };
    for _ in 0..300 {
        let start = rng.int(19_905, 20_999); // 2024-07-01 to 2027-06-30
        let end = rng.int(start, 20_999);
        let expenditure = format!("{}.{:02}", rng.int(500, 100_000_000), rng.int(0, 99));
        let repaid = rng.bool();
        let share = *rng.pick(&["1", "0.5", "0.25"]);
        let input = |year: &str| {
            json!({
                "borrowing_expenses": expenditure, "other_borrowing_expenses_in_year": "0.00",
                "income_use": share,
                "loan_start": date(start),
                "loan_term_end": if repaid { json!(null) } else { json!(date(end)) },
                "repaid_on": if repaid { json!(date(end)) } else { json!(null) },
                "section_26_50_applies": false, "year": year, "scope_confirmed": true,
            })
        };
        let (mut carried, mut total) = (d(&expenditure), n(0));
        for year in &years[year_of(start)..=year_of(end)] {
            let facts = input(year);
            let r = run("borrowing-expenses", facts.clone());
            let maximum = amount(&r, "maximum_deduction");
            assert!(
                eq(&amount(&r, "remaining_expenditure"), &carried),
                "{facts}"
            );
            assert!(le(&amount(&r, "deduction"), &maximum), "{facts}");
            carried = amount(&r, "remaining_expenditure_after_year");
            total = total.add(&maximum).unwrap();
        }
        assert!(
            eq(&total, &d(&expenditure)) && eq(&carried, &n(0)),
            "{expenditure}"
        );
    }
}

// Metamorphic relations: how the answer must move when one fact moves.

fn decline(cost: &str, life: &str, days: i64, method: &str) -> Value {
    run(
        "depreciation",
        json!({"cost": cost, "effective_life": life, "days": days, "taxable_use": "0.75",
               "method": method, "scope_confirmed": true, "year": "2025-26"}),
    )
}

#[test]
fn holding_an_asset_longer_never_lowers_its_decline_and_its_value_is_conserved() {
    let mut rng = Rng(0xDEC1);
    for _ in 0..RUNS {
        let cost = rng.cents(100_000_000);
        let life = rng.int(1, 40).to_string();
        let fewer = rng.int(0, 365);
        let more = rng.int(fewer, 365);
        for method in ["prime_cost", "diminishing_value"] {
            let low = decline(&cost, &life, fewer, method);
            let high = decline(&cost, &life, more, method);
            assert!(le(
                &amount(&low, "decline_in_value"),
                &amount(&high, "decline_in_value")
            ));
            for result in [&low, &high] {
                let taken = amount(result, "decline_in_value");
                let kept = amount(result, "closing_adjustable_value");
                assert!(le(&taken, &d(&cost)), "{cost} {result}");
                assert!(le(&amount(result, "deduction"), &taken), "{cost} {result}");
                let gap = taken.add(&kept).unwrap().sub(&d(&cost)).unwrap();
                assert!(
                    le(&gap, &d("0.01")) && le(&d("-0.01"), &gap),
                    "{cost} {result}"
                );
            }
        }
        let prime = decline(&cost, &life, more, "prime_cost");
        let diminishing = decline(&cost, &life, more, "diminishing_value");
        assert!(le(
            &amount(&prime, "decline_in_value"),
            &amount(&diminishing, "decline_in_value")
        ));
    }
}

/// Rolling an asset forward, each closing value opening the next year: every
/// year's decline and closing value add back to its opening value to the cent,
/// so the declines never sum to more than cost. $10,001 over 8 years by prime
/// cost declines $1,250.125 a year, a half cent each time.
#[test]
fn rolling_an_asset_forward_never_writes_off_more_than_its_cost() {
    let mut rng = Rng(0xDEC2);
    let mut assets = vec![("10001.00".to_string(), "8".to_string(), "prime_cost", true)];
    for _ in 0..RUNS {
        let method = *rng.pick(&["prime_cost", "diminishing_value"]);
        assets.push((
            rng.cents(100_000_000),
            rng.int(1, 20).to_string(),
            method,
            rng.bool(),
        ));
    }
    for (cost, life, method, whole_years) in assets {
        let mut opening: Option<String> = None;
        let mut total = n(0);
        for _ in 0..25 {
            let days = if whole_years { 365 } else { rng.int(0, 365) };
            let mut facts = json!({"cost": cost, "effective_life": life, "days": days,
                "taxable_use": "1", "method": method, "scope_confirmed": true, "year": "2026-27"});
            if let Some(value) = &opening {
                facts["opening_adjustable_value"] = json!(value);
            }
            let result = run("depreciation", facts);
            let taken = amount(&result, "decline_in_value");
            let kept = amount(&result, "closing_adjustable_value");
            let start = d(opening.as_deref().unwrap_or(&cost));
            assert!(
                eq(&taken.add(&kept).unwrap(), &start),
                "{cost} {life} {result}"
            );
            total = total.add(&taken).unwrap();
            assert!(le(&total, &d(&cost)), "{cost} {life} {result}");
            opening = Some(
                result["amounts"]["closing_adjustable_value"]
                    .as_str()
                    .unwrap()
                    .into(),
            );
        }
    }
}

fn quarter(earnings: &str, paid: &str) -> Value {
    run(
        "quarterly-sg",
        json!({"ordinary_time_earnings": earnings, "qualifying_contributions": paid,
               "year": "2025-26", "quarter": 1, "scope_confirmed": true}),
    )
}

#[test]
fn paying_more_super_never_raises_the_amount_still_owing() {
    let mut rng = Rng(0xDEC2);
    for _ in 0..RUNS {
        let earnings = rng.cents(10_000_000);
        let (paid_less, paid_more) = sorted_dec(rng.cents(2_000_000), rng.cents(2_000_000));
        let low = quarter(&earnings, &paid_less);
        let high = quarter(&earnings, &paid_more);
        let still_low = amount(&low, "additional_contribution");
        let still_high = amount(&high, "additional_contribution");
        assert!(le(&still_high, &still_low), "{earnings} {low} {high}");
        assert!(le(&still_low, &amount(&low, "minimum_sg")), "{low}");
        assert!(eq(
            &amount(&quarter(&earnings, "0.00"), "additional_contribution"),
            &amount(&low, "minimum_sg")
        ));
    }
}

#[test]
fn more_ordinary_earnings_never_lower_the_minimum_and_the_base_caps_it() {
    let mut rng = Rng(0xDEC3);
    for _ in 0..RUNS {
        let (less, more) = sorted_dec(rng.cents(10_000_000), rng.cents(10_000_000));
        let low = amount(&quarter(&less, "0.00"), "minimum_sg");
        let high = amount(&quarter(&more, "0.00"), "minimum_sg");
        assert!(le(&low, &high), "{less} {more}");
        assert!(le(&high, &d("7500.00")), "{more}");
    }
}

#[test]
fn a_worked_quarter_and_asset_hold() {
    // Worked by hand: 12% of 10,000.00 is 1,200.00, less 200.00 paid. The base
    // caps 100,000.00 at 62,500.00, and 12% of that is 7,500.00.
    let result = quarter("10000.00", "200.00");
    assert_eq!(result["amounts"]["minimum_sg"], "1200.00");
    assert_eq!(result["amounts"]["additional_contribution"], "1000.00");
    let capped = quarter("100000.00", "0.00");
    assert_eq!(capped["amounts"]["earnings_used"], "62500.00");
    assert_eq!(capped["amounts"]["minimum_sg"], "7500.00");
    // 10,000.00 over 5 years for a full year: prime cost 2,000.00, diminishing
    // value twice that, and 75% of each is the deduction.
    let prime = decline("10000.00", "5", 365, "prime_cost");
    assert_eq!(prime["amounts"]["decline_in_value"], "2000.00");
    assert_eq!(prime["amounts"]["deduction"], "1500.00");
    assert_eq!(prime["amounts"]["closing_adjustable_value"], "8000.00");
    let diminishing = decline("10000.00", "5", 365, "diminishing_value");
    assert_eq!(diminishing["amounts"]["decline_in_value"], "4000.00");
}

/// Division 296 tax never falls as the balance rises with the same earnings,
/// never exceeds 25% of them (15% plus 10%, each of at most all the earnings),
/// and is nil when the earnings are nil or less.
#[test]
fn division_296_tax_rises_with_the_balance_and_stays_within_a_quarter() {
    let mut rng = Rng(20_261_113);
    let tax = |balance: &str, earnings: &str| {
        let r = run(
            "division-296",
            json!({
                "total_super_balance_end_of_year": balance, "total_super_earnings": earnings,
                "child_recipient": false, "structured_settlement_contribution": false,
                "died_in_year": false, "year": "2026-27", "scope_confirmed": true,
            }),
        );
        amount(&r, "division_296_tax")
    };
    for _ in 0..RUNS {
        let (low, high) = sorted_dec(rng.money(), rng.money());
        let earnings = rng.cents(1_000_000_000);
        let higher = tax(&high, &earnings);
        assert!(
            le(&tax(&low, &earnings), &higher),
            "{low} {high} {earnings}"
        );
        let quarter = d(&earnings).mul(&d("0.25")).unwrap();
        assert!(
            le(&higher, &quarter.add(&d("0.005")).unwrap()),
            "{high} {earnings}"
        );
        assert!(eq(&tax(&high, &format!("-{earnings}")), &n(0)), "{high}");
    }
}

/// A termination payment's parts add back to what was paid, each part stays
/// within its cap and the 2 within the ETP cap, the withholding lies between
/// 17% and 47% of the taxable components but for a dollar of rounding, and
/// more other taxable payments never lower it.
#[test]
fn termination_payment_parts_add_up_and_withholding_rises_with_other_income() {
    let mut rng = Rng(20_261_114);
    let add = |a: &Decimal, b: &Decimal| a.add(b).unwrap();
    for _ in 0..RUNS {
        let (redundancy, excluded) = (rng.cents(50_000_000), rng.cents(40_000_000));
        let mut facts = json!({
            "redundancy_payment": redundancy, "completed_years_of_service": rng.int(0, 45),
            "excluded_payment": excluded, "non_excluded_payment": rng.cents(40_000_000),
            "other_taxable_payments": rng.cents(30_000_000),
            "date_of_birth": *rng.pick(&["1955-03-01", "1967-06-30", "1967-07-01", "1990-01-15"]),
            "year": "2026-27", "scope_confirmed": true,
        });
        let r = run("termination-payment", facts.clone());
        let a = |key: &str| amount(&r, key);
        assert!(
            eq(
                &add(
                    &a("tax_free_redundancy_amount"),
                    &a("excluded_taxable_component")
                ),
                &add(&d(&redundancy), &d(&excluded))
            ),
            "{facts}"
        );
        for part in ["excluded", "non_excluded"] {
            let split = add(
                &a(&format!("{part}_within_cap")),
                &a(&format!("{part}_above_cap")),
            );
            assert!(
                eq(&split, &a(&format!("{part}_taxable_component"))),
                "{facts}"
            );
        }
        let cap = a("non_excluded_cap");
        assert!(le(&a("non_excluded_within_cap"), &cap), "{facts}");
        assert!(le(&cap, &a("remaining_etp_cap")) && le(&cap, &a("whole_of_income_cap")));
        let within = add(&a("excluded_within_cap"), &a("non_excluded_within_cap"));
        assert!(le(&within, &n(270_000)), "{facts}");
        let taxable = add(
            &a("excluded_taxable_component"),
            &a("non_excluded_taxable_component"),
        );
        let withheld = a("total_withholding");
        let lowest = taxable.mul(&d("0.17")).unwrap().sub(&n(1)).unwrap();
        let highest = add(&taxable.mul(&d("0.47")).unwrap(), &n(1));
        assert!(le(&lowest, &withheld) && le(&withheld, &highest), "{facts}");
        let more = add(
            &d(facts["other_taxable_payments"].as_str().unwrap()),
            &d(&rng.cents(10_000_000)),
        );
        facts["other_taxable_payments"] = json!(more.to_string());
        let after = run("termination-payment", facts.clone());
        assert!(
            le(&withheld, &amount(&after, "total_withholding")),
            "{facts}"
        );
    }
}

/// Schedule 5 Methods A and B(ii): the withholding from an additional payment
/// is never negative or more than 47% of it, and the total is that plus the
/// withholding from the normal earnings.
#[test]
fn schedule_5_withholding_stays_within_the_47_percent_limit() {
    let mut rng = Rng(20_261_114);
    for _ in 0..RUNS {
        let pay = *rng.pick(&["weekly", "fortnightly", "monthly"]);
        let scale = *rng.pick(&[1, 2, 3, 5, 6]);
        let earnings = rng.pay();
        let additional = rng.cents(100_000_000);
        let common = json!({
            "earnings": earnings, "additional_payment": additional, "pay_period": pay,
            "scale": scale, "study_loan": rng.bool(), "year": "2026-27", "scope_confirmed": true,
        });
        let most = match pay {
            "weekly" => 52,
            "fortnightly" => 26,
            _ => 12,
        };
        let mut method_a = common.clone();
        method_a["pay_periods"] = json!(rng.int(2, most));
        let mut method_b_ii = common;
        method_b_ii["year_to_date_earnings"] = json!(rng.money());
        method_b_ii["pay_periods_to_date"] = json!(rng.int(1, most));
        method_b_ii["earlier_additional_payments"] = json!(rng.pay());
        method_b_ii["earlier_additional_withholding"] = json!(rng.pay());
        let limit = d(&additional).mul(&d("0.47")).unwrap();
        for (command, facts) in [
            ("schedule-5-method-a", method_a),
            ("schedule-5-method-b-ii", method_b_ii),
        ] {
            let result = run(command, facts.clone());
            let extra = amount(&result, "withholding_on_additional_payment");
            assert!(le(&extra, &limit), "{command} {facts}");
            let total = amount(&result, "withholding_on_earnings")
                .add(&extra)
                .unwrap();
            assert!(
                eq(&amount(&result, "withholding"), &total),
                "{command} {facts}"
            );
        }
    }
}

/// Schedule 15: a registered employer's withholding from a working holiday
/// maker who gave a TFN never falls as the payment rises, nor as the payments
/// before it rise for a payment over $19, and never exceeds 45% of the
/// payment. Up to $19, column 5's ignored cents can withhold a dollar less than
/// column 4's rounding ($2: $1 at 37%, $0 at 45%).
#[test]
fn working_holiday_maker_withholding_rises_and_stays_within_45_percent() {
    let mut rng = Rng(20_261_114);
    let withholding = |payment: &str, before: i64| {
        let r = run(
            "working-holiday-maker-withholding",
            json!({
                "payment": payment, "payments_before": format!("{before}.00"),
                "employer_registered": true, "tfn_quoted": true,
                "pay_period": "fortnightly", "year": "2026-27", "scope_confirmed": true,
            }),
        );
        amount(&r, "withholding")
    };
    for _ in 0..RUNS {
        let (low, high) = sorted_dec(rng.pay(), rng.pay());
        let (fewer, more) = sorted(rng.dollars(), rng.dollars());
        let top = withholding(&high, more);
        assert!(le(&withholding(&low, more), &top), "{low} {high} {more}");
        assert!(
            le(&d(&high), &n(19)) || le(&withholding(&high, fewer), &top),
            "{high} {fewer} {more}"
        );
        let limit = d(&high).mul(&d("0.45")).unwrap().add(&d("0.5")).unwrap();
        assert!(le(&top, &limit), "{high} {more}");
    }
}

/// Schedule 9: for each status, withholding never falls as whole-dollar weekly
/// earnings rise past the last coefficient step ($3,653), and a full Medicare
/// levy exemption never withholds more than a half one, nor a half one more
/// than none.
#[test]
fn seniors_pensioners_withholding_rises_and_exemptions_never_raise_it() {
    for year in ["2024-25", "2025-26", "2026-27"] {
        for status in ["single", "illness_separated", "couple"] {
            let mut previous = [n(0), n(0), n(0)];
            for dollars in 0..3_700 {
                let withholding = ["full", "half", "none"].map(|exemption| {
                    let r = run(
                        "seniors-pensioners-withholding",
                        json!({"earnings": dollars.to_string(), "pay_period": "weekly",
                               "status": status, "medicare_levy_exemption": exemption,
                               "year": year, "scope_confirmed": true}),
                    );
                    amount(&r, "withholding")
                });
                let [full, half, none] = &withholding;
                assert!(
                    le(full, half) && le(half, none),
                    "{year} {status} {dollars}"
                );
                for (before, now) in previous.iter().zip(&withholding) {
                    assert!(le(before, now), "{year} {status} {dollars}");
                }
                previous = withholding;
            }
        }
    }
}

/// The seniors and pensioners tax offset never rises as rebate income rises,
/// for each status, with the spouse's rebate income held fixed and no
/// transfer, starting from the base rebate amount and reaching nil.
#[test]
fn seniors_pensioners_tax_offset_never_rises_with_rebate_income() {
    for year in ["2024-25", "2025-26", "2026-27"] {
        for (status, base) in [
            ("single", 2230),
            ("illness_separated", 2040),
            ("couple", 1602),
        ] {
            let mut previous = n(base);
            for income in (20_000..62_000).step_by(7) {
                let mut facts = json!({"rebate_income": income.to_string(), "status": status,
                                       "year": year, "scope_confirmed": true});
                if status != "single" {
                    facts["spouse_rebate_income"] = json!("40000");
                }
                let result = run("seniors-pensioners-tax-offset", facts);
                let offset = amount(&result, "seniors_pensioners_tax_offset");
                assert!(le(&offset, &previous), "{year} {status} {income}");
                previous = offset;
            }
            assert!(eq(&previous, &n(0)), "{year} {status} reaches nil");
        }
    }
}

/// Howard Hinnant's civil_from_days, for generating dates.
fn civil(day: i64) -> (i64, i64, i64) {
    let z = day + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}
