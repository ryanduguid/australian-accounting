//! Seeded properties that hold for every input the worksheets accept. The
//! seeds, run counts, explicit examples and input domains are the Python
//! fast-check properties'; the generator is a local SplitMix64.
use australian_tax_calculators::api;
use australian_tax_calculators::decimal::Decimal;
use australian_tax_calculators::metadata::{
    PAYG_WITHHOLDING_COEFFICIENTS, RESIDENT_TAX_SCALES, supported_periods,
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
    api::evaluate(command, &input).unwrap_or_else(|e| panic!("{command} {input}: {e}"))
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

fn withheld(earnings: &str, period: &str, scale: i64, year: &str) -> Decimal {
    let result = run(
        "payg-withholding",
        json!({"earnings": earnings, "pay_period": period, "scale": scale,
               "year": year, "scope_confirmed": true}),
    );
    amount(&result, "withholding")
}

fn payg_scales() -> Vec<(&'static str, i64)> {
    PAYG_WITHHOLDING_COEFFICIENTS
        .iter()
        .flat_map(|(year, scales)| scales.iter().map(move |(scale, _)| (*year, *scale)))
        .collect()
}

#[test]
fn payg_withholding_never_falls_as_earnings_rise() {
    let check = |(year, scale): (&str, i64), period: &str, low: String, high: String| {
        let (low, high) = sorted_dec(low, high);
        let (a, b) = (
            withheld(&low, period, scale, year),
            withheld(&high, period, scale, year),
        );
        assert!(
            le(&n(0), &a) && le(&a, &b),
            "{year} {scale} {period} {low} {high}"
        );
    };
    check(
        ("2026-27", 2),
        "monthly",
        "3000.32".into(),
        "3000.33".into(),
    );
    let scales = payg_scales();
    let mut rng = Rng(0x9A76);
    for _ in 0..RUNS {
        let year_scale = *rng.pick(&scales);
        let period = *rng.pick(&["weekly", "fortnightly", "monthly"]);
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
                    let amount = withheld(&dollars.to_string(), period, *scale, year);
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
    for (year, lower, upper) in [("2024-25", 27222, 34027), ("2025-26", 28011, 35013)] {
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

#[test]
fn study_loan_components_agree_with_the_combined_coefficients_to_a_dollar() {
    // The ATO says the two methods can differ slightly through rounding and accepts
    // either. Every whole-dollar weekly amount to $4,000 stays within $1.
    for (scale, _, rows) in common::COMBINED {
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
                       "scale": scale, "year": "2026-27", "scope_confirmed": true}),
            );
            let gap = amount(&result, "withholding").sub(&expected).unwrap();
            assert!(le(&n(-1), &gap) && le(&gap, &n(1)), "{scale} {dollars}");
            saw_zero |= gap.is_zero();
        }
        assert!(saw_zero, "{scale}");
    }
}

/// Every car result reconciles to the cent: the applied and excess
/// contribution make up the contribution, the applied part and the value
/// before exemption make up the gross value, and an exempt car is nil.
fn check_car_reconciles(result: &Value, contribution: &str, exempt: bool) {
    let gross = amount(result, "gross_taxable_value");
    let applied = amount(result, "employee_contribution_applied");
    let before = amount(result, "taxable_value_before_exemption");
    let total = applied
        .add(&amount(result, "excess_employee_contribution"))
        .unwrap();
    assert!(eq(&total, &d(contribution)), "{result}");
    assert!(eq(&applied.add(&before).unwrap(), &gross), "{result}");
    assert!(le(&n(0), &before), "{result}");
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
        // The base never exceeds cost plus accessories, nor the gross value 20% of it.
        let base = amount(&result, "base_value");
        assert!(
            le(&base, &d(&cost).add(&d(&accessories)).unwrap()),
            "{result}"
        );
        let ceiling = base.mul(&d("0.20")).unwrap().add(&d("0.005")).unwrap();
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
                   "days_held": rng.int(0, 365), "business_use": business,
                   "employee_contribution": contribution, "electric_car_exempt": exempt,
                   "year_ended": year, "scope_confirmed": true}),
        );
        check_car_reconciles(&result, &contribution, exempt);
        let parts = ["deemed_depreciation", "deemed_interest"]
            .iter()
            .fold(d(&running).add(&d(&lease)).unwrap(), |sum, key| {
                sum.add(&amount(&result, key)).unwrap()
            });
        assert!(eq(&parts, &amount(&result, "operating_costs")), "{result}");
        let costs = amount(&result, "operating_costs").add(&d("0.005")).unwrap();
        assert!(
            le(&amount(&result, "gross_taxable_value"), &costs),
            "{result}"
        );
    }
}
