// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The s 109E(6) formula's seeded properties. Seeds, run counts and input
//! domains are the Python fast-check properties'; a local SplitMix64 replaces
//! fast-check's generator, and exact rational arithmetic is the reference. The
//! s 109E(7) rollforward's properties, at the end, use the same generator.

use div7a_loan_review::api;
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One, Signed, Zero};
use serde_json::{Value, json};

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn int(&mut self, low: i64, high: i64) -> i64 {
        low + (self.next() % (high - low + 1) as u64) as i64
    }
}

/// A loan from the Python domains: principal 0.01 to 100,000,000.00 in cents,
/// rate 0.0001 to 0.2500 in ten-thousandths, and a whole-year term.
#[derive(Debug, Clone, Copy)]
struct Loan {
    cents: i64,
    rate: i64,
    term: i64,
}

fn loans(seed: u64, max_term: i64) -> Vec<Loan> {
    let mut rng = Rng(campaign_seed(seed));
    (0..200)
        .map(|_| Loan {
            cents: rng.int(1, 10_000_000_000),
            rate: rng.int(1, 2_500),
            term: rng.int(1, max_term),
        })
        .collect()
}

fn text(units: i64, places: u32) -> String {
    let scale = 10i64.pow(places);
    format!(
        "{}.{:0width$}",
        units / scale,
        units % scale,
        width = places as usize
    )
}

/// The engine's figure, in cents.
fn figure(loan: Loan, term: i64) -> BigInt {
    let result = api::evaluate(
        "minimum-yearly-repayment-amount",
        &json!({"principal": text(loan.cents, 2), "rate": text(loan.rate, 4),
                "remaining_term": term.to_string()}),
    )
    .unwrap();
    let digits: String = result
        .as_str()
        .unwrap()
        .chars()
        .filter(|c| *c != '.')
        .collect();
    digits.parse().unwrap()
}

/// A non-negative rational number.
#[derive(Clone, Debug, PartialEq)]
struct Fraction {
    num: BigInt,
    den: BigInt,
}

impl Fraction {
    fn new(num: BigInt, den: BigInt) -> Fraction {
        let g = num.gcd(&den);
        let g = if g.is_zero() { BigInt::one() } else { g };
        Fraction {
            num: num / &g,
            den: den / &g,
        }
    }

    fn int(n: i64) -> Fraction {
        Fraction::new(BigInt::from(n), BigInt::one())
    }

    fn ratio(n: i64, d: i64) -> Fraction {
        Fraction::new(BigInt::from(n), BigInt::from(d))
    }

    fn add(&self, o: &Fraction) -> Fraction {
        Fraction::new(&self.num * &o.den + &o.num * &self.den, &self.den * &o.den)
    }

    fn sub(&self, o: &Fraction) -> Fraction {
        Fraction::new(&self.num * &o.den - &o.num * &self.den, &self.den * &o.den)
    }

    fn mul(&self, o: &Fraction) -> Fraction {
        Fraction::new(&self.num * &o.num, &self.den * &o.den)
    }

    fn div(&self, o: &Fraction) -> Fraction {
        Fraction::new(&self.num * &o.den, &self.den * &o.num)
    }

    fn le(&self, o: &Fraction) -> bool {
        &self.num * &o.den <= &o.num * &self.den
    }

    fn abs(&self) -> Fraction {
        Fraction::new(self.num.abs(), self.den.abs())
    }

    fn pow(&self, n: i64) -> Fraction {
        (0..n).fold(Fraction::int(1), |acc, _| acc.mul(self))
    }
}

/// The s 109E(6) formula in exact rational arithmetic, rounded half up to cents.
fn exact(loan: Loan, term: i64) -> BigInt {
    let p = Fraction::ratio(loan.cents, 100);
    let r = Fraction::ratio(loan.rate, 10_000);
    let growth = Fraction::int(1).add(&r);
    let value = p
        .mul(&r)
        .div(&Fraction::int(1).sub(&Fraction::int(1).div(&growth).pow(term)));
    let cents = value.mul(&Fraction::int(100));
    let (whole, rest) = cents.num.div_rem(&cents.den);
    if rest * 2 >= cents.den {
        whole + 1
    } else {
        whole
    }
}

#[test]
fn formula_matches_exact_arithmetic_everywhere() {
    for loan in loans(0xD17A, 25) {
        assert_eq!(figure(loan, loan.term), exact(loan, loan.term), "{loan:?}");
    }
}

#[test]
fn paying_the_figure_every_year_clears_the_loan_to_rounding() {
    for loan in loans(0xD17B, 25) {
        let payment = Fraction::new(figure(loan, loan.term), BigInt::from(100));
        let growth = Fraction::int(1).add(&Fraction::ratio(loan.rate, 10_000));
        let mut balance = Fraction::ratio(loan.cents, 100);
        let mut tolerance = Fraction::int(0);
        for k in 0..loan.term {
            balance = balance.mul(&growth).sub(&payment);
            tolerance = tolerance.add(&growth.pow(k));
        }
        let tolerance = tolerance.mul(&Fraction::ratio(1, 200));
        assert!(balance.abs().le(&tolerance), "{loan:?}");
    }
}

#[test]
fn a_longer_term_never_raises_the_figure() {
    for loan in loans(0xD17C, 24) {
        assert!(
            figure(loan, loan.term + 1) <= figure(loan, loan.term),
            "{loan:?}"
        );
    }
}

#[test]
fn the_figure_covers_interest_and_a_straight_line_share() {
    for loan in loans(0xD17D, 25) {
        let fig = Fraction::new(figure(loan, loan.term), BigInt::from(100));
        let p = Fraction::ratio(loan.cents, 100);
        let r = Fraction::ratio(loan.rate, 10_000);
        let half_cent = Fraction::ratio(1, 200);
        assert!(p.mul(&r).sub(&half_cent).le(&fig), "{loan:?}");
        assert!(
            p.div(&Fraction::int(loan.term)).sub(&half_cent).le(&fig),
            "{loan:?}"
        );
    }
}

#[test]
fn regression_points_hold() {
    let amount = |p: &str, r: &str, t: &str| {
        api::evaluate(
            "minimum-yearly-repayment-amount",
            &json!({"principal": p, "rate": r, "remaining_term": t}),
        )
    };
    // The evaluation README's hand-worked figure and an independent engine's.
    assert_eq!(amount("100000.00", "0.0877", "5").unwrap(), "25556.00");
    assert_eq!(amount("70000", "0.0877", "6").unwrap(), "15497.53");
    // n = 1 is principal plus a year of interest; a vast term is interest only.
    assert_eq!(amount("123456.78", "0.0877", "1").unwrap(), "134283.94");
    for term in ["1E+30", "1E+999999999"] {
        assert_eq!(amount("123456.78", "0.0877", term).unwrap(), "10827.16");
    }
    assert_eq!(
        amount("100000", "0.0877", "4.5").unwrap_err(),
        "remaining_term must be a whole number of years; use statutory_remaining_term first"
    );
    // A rate too small to move 1 + rate at 60 digits leaves a nil denominator.
    assert_eq!(
        amount("100000", "1E-70", "5").unwrap_err(),
        "decimal.DivisionByZero"
    );
}

/// The 2 minimum yearly repayments worked on the ATO's 'Loans by private companies'
/// page (last updated 2 July 2026, read on 5 November 2026; a banner says it is
/// under review after the Bendel High Court decision). The ATO rounds each to the
/// closest dollar, so the engine's cents are held to that. Their rates, for income
/// years 2015 and 2016, are older than the engine's rate table, so the formula
/// takes them directly.
#[test]
fn the_ato_worked_repayments_hold() {
    let dollars = |principal: &str, rate: &str, term: &str| {
        let result = api::evaluate(
            "minimum-yearly-repayment-amount",
            &json!({"principal": principal, "rate": rate, "remaining_term": term}),
        )
        .unwrap();
        let cents: i64 = result.as_str().unwrap().replace('.', "").parse().unwrap();
        (cents + 50) / 100
    };
    // 55,000 x 0.0595 / (1 - 1.0595^-7) = 3,272.50 / 0.332742794 = 9,834.92.
    assert_eq!(dollars("55000.00", "0.0595", "7"), 9_835);
    // 50,430 x 0.0545 / (1 - 1.0545^-6) = 2,748.435 / 0.272688431 = 10,079.03.
    assert_eq!(dollars("50430.00", "0.0545", "6"), 10_079);
}

// Metamorphic relations: how the answer must move when one fact moves. The cases
// use 2024-25, whose benchmark rate is 8.77%.

/// The repayment review of one loan on complying terms, with `paid` applied.
fn reviewed(loan: Loan, paid_cents: i64) -> Value {
    api::evaluate(
        "minimum-yearly-repayment",
        &json!({"facts": {
            "year_of_income": "2024-25",
            "year_loan_made": "2023-24",
            "amalgamated_loan_unpaid_at_end_of_previous_year": text(loan.cents, 2),
            "remaining_term_years": loan.term.to_string(),
            "payments_applied_during_the_year": text(paid_cents, 2),
            "gate_result": {"verdict": "COMPLYING", "benchmark_year_used": "2023-24"},
            "loan_id": "L-1",
        }}),
    )
    .unwrap()
}

fn cents_of(result: &Value, key: &str) -> i64 {
    result[key]
        .as_str()
        .unwrap_or_else(|| panic!("{key} missing from {result}"))
        .replace('.', "")
        .parse()
        .unwrap()
}

/// What every completed review must say, whatever its facts.
fn check_sanity(result: &Value, paid_cents: i64) {
    let required = cents_of(result, "myr_required");
    let shortfall = cents_of(result, "shortfall");
    assert!(required > 0, "{result}");
    assert_eq!(cents_of(result, "payments_applied"), paid_cents, "{result}");
    assert_eq!(shortfall, (required - paid_cents).max(0), "{result}");
    if shortfall == 0 {
        assert_eq!(result["verdict"], "MYR_MET", "{result}");
        assert!(result["experimental_deemed_dividend_exposure"].is_null());
    } else {
        assert_eq!(result["verdict"], "MYR_SHORT", "{result}");
        assert_eq!(
            cents_of(result, "experimental_deemed_dividend_exposure"),
            shortfall,
            "{result}"
        );
    }
}

#[test]
fn more_repayment_never_raises_the_shortfall() {
    let mut rng = Rng(campaign_seed(0xD17E));
    for loan in loans(0xD17F, 25) {
        let required = cents_of(&reviewed(loan, 0), "myr_required");
        // Either side of the figure, and the figure itself.
        let less = rng.int(0, required);
        let more = rng.int(less, required + 100);
        let (low, high) = (reviewed(loan, less), reviewed(loan, more));
        check_sanity(&low, less);
        check_sanity(&high, more);
        assert!(
            cents_of(&high, "shortfall") <= cents_of(&low, "shortfall"),
            "{loan:?} {less} {more}"
        );
        assert_eq!(cents_of(&reviewed(loan, 0), "shortfall"), required);
        assert_eq!(reviewed(loan, required)["verdict"], "MYR_MET", "{loan:?}");
        let cent_short = reviewed(loan, required - 1);
        assert_eq!(cents_of(&cent_short, "shortfall"), 1, "{loan:?}");
    }
}

#[test]
fn a_larger_balance_or_rate_never_lowers_the_figure() {
    let mut rng = Rng(campaign_seed(0xD180));
    for loan in loans(0xD181, 25) {
        let larger = Loan {
            cents: rng.int(loan.cents, 10_000_000_000),
            ..loan
        };
        let dearer = Loan {
            rate: rng.int(loan.rate, 2_500),
            ..loan
        };
        let base = figure(loan, loan.term);
        assert!(base <= figure(larger, loan.term), "{loan:?} {larger:?}");
        assert!(base <= figure(dearer, loan.term), "{loan:?} {dearer:?}");
    }
}

#[test]
fn doubling_the_balance_doubles_the_figure_to_the_cent() {
    for loan in loans(0xD182, 25) {
        let doubled = Loan {
            cents: loan.cents * 2,
            ..loan
        };
        let gap: BigInt = figure(doubled, loan.term) - figure(loan, loan.term) * 2;
        assert!(gap.abs() <= BigInt::one(), "{loan:?}");
    }
}

#[test]
fn a_worked_shortfall_holds_through_the_full_review() {
    // Worked by hand: 100,000 x 0.0877 / (1 - 1.0877^-5) = 25,556.00, the
    // regression point above, less 20,000.00 applied.
    let loan = Loan {
        cents: 10_000_000,
        rate: 877,
        term: 5,
    };
    let result = reviewed(loan, 2_000_000);
    assert_eq!(result["myr_required"], "25556.00");
    assert_eq!(result["shortfall"], "5556.00");
    assert_eq!(result["verdict"], "MYR_SHORT");
}

// Nightly campaigns vary generated cases whilst ordinary checks retain their frozen seeds.
fn campaign_seed(default: u64) -> u64 {
    match std::env::var("LL_PROPERTY_SEED") {
        Ok(text) => {
            let seed = text.parse::<u64>().expect("LL_PROPERTY_SEED must be a u64");
            let effective = (default ^ seed).max(1);
            eprintln!("LL_PROPERTY_SEED={seed} effective={effective}");
            effective
        }
        Err(std::env::VarError::NotPresent) => default,
        Err(error) => panic!("invalid LL_PROPERTY_SEED: {error}"),
    }
}

// The s 109E(7) rollforward. Interest is rounded to cents per period, as the
// ATO calculator does, so the properties allow for that rounding where it bites.

/// Whether the year of income starting in `start` holds 29 February.
fn holds_leap_day(start: i64) -> bool {
    let end = start + 1;
    end % 4 == 0 && (end % 100 != 0 || end % 400 == 0)
}

fn days_in_year(start: i64) -> i64 {
    if holds_leap_day(start) { 366 } else { 365 }
}

/// The date `offset` days after 1 July of the year starting in `start`.
fn date_in_year(start: i64, offset: i64) -> String {
    let february = if holds_leap_day(start) { 29 } else { 28 };
    let months = [31, 31, 30, 31, 30, 31, 31, february, 31, 30, 31, 30];
    let mut day = offset;
    for (i, length) in months.iter().enumerate() {
        if day < *length {
            let month = (i + 6) % 12 + 1;
            let year = if month >= 7 { start } else { start + 1 };
            return format!("{year:04}-{month:02}-{:02}", day + 1);
        }
        day -= length;
    }
    panic!("offset {offset} is past the year");
}

/// A year, an opening balance in cents and repayments as (day offset, cents),
/// each at least `min_cents`, together no more than the opening balance.
fn rollforward_case(rng: &mut Rng, min_cents: i64) -> (i64, i64, Vec<(i64, i64)>) {
    let start = rng.int(2019, 2026);
    let days = days_in_year(start);
    let opening = rng.int(min_cents.max(1) * 6, 1_000_000_000);
    let mut left = opening;
    let mut repayments = Vec::new();
    for _ in 0..rng.int(0, 6) {
        if left < min_cents.max(1) {
            break;
        }
        let amount = rng.int(min_cents.max(1), left.min(min_cents.max(1) + opening / 3));
        left -= amount;
        repayments.push((rng.int(0, days - 1), amount));
    }
    (start, opening, repayments)
}

fn rolled(start: i64, opening: i64, repayments: &[(i64, i64)]) -> Value {
    let input = json!({
        "year_of_income": format!("{start}-{:02}", (start + 1) % 100),
        "year_loan_made": format!("{}-{:02}", start - 1, start % 100),
        "balance_at_start_of_year": text(opening, 2),
        "repayments": repayments.iter().map(|(offset, cents)| json!({
            "date": date_in_year(start, *offset), "amount": text(*cents, 2),
        })).collect::<Vec<_>>(),
    });
    div7a_loan_review::rollforward::rollforward(&input).unwrap_or_else(|e| panic!("{input}: {e}"))
}

#[test]
fn the_rollforward_keeps_its_books() {
    let mut rng = Rng(campaign_seed(0xD17C));
    for _ in 0..200 {
        let (start, opening, repayments) = rollforward_case(&mut rng, 1);
        let result = rolled(start, opening, &repayments);
        let at = format!("{start} {opening} {repayments:?}");
        let periods = result["periods"].as_array().unwrap();
        let days: i64 = periods.iter().map(|p| p["days"].as_i64().unwrap()).sum();
        assert_eq!(days, days_in_year(start), "{at}");
        assert_eq!(result["days_in_year"], days, "{at}");
        let charged: Vec<i64> = periods.iter().map(|p| cents_of(p, "interest")).collect();
        assert!(charged.iter().all(|c| *c >= 0), "{at}");
        let interest = cents_of(&result, "interest");
        assert_eq!(charged.iter().sum::<i64>(), interest, "{at}");
        let paid: i64 = repayments.iter().map(|(_, c)| c).sum();
        assert_eq!(cents_of(&result, "repayments_total"), paid, "{at}");
        assert_eq!(
            cents_of(&result, "applied_to_interest") + cents_of(&result, "applied_to_principal"),
            paid,
            "{at}"
        );
        let closing = cents_of(&result, "amalgamated_loan_unpaid_at_end_of_year");
        // s 109E(7): repayments meet the interest first, and unpaid interest is not loan.
        let expected = if paid >= interest {
            opening + interest - paid
        } else {
            opening
        };
        assert_eq!(closing, expected, "{at}");
        assert!(closing >= 0, "{at}");
    }
}

/// With no repayment the year's interest is the balance times the benchmark
/// rate, 366-day years included.
#[test]
fn a_full_year_without_repayments_earns_the_benchmark_rate() {
    let mut rng = Rng(campaign_seed(0xD17D));
    for start in 2019..=2026 {
        for _ in 0..25 {
            let opening = rng.int(1, 10_000_000_000);
            let result = rolled(start, opening, &[]);
            let rate: i64 = result["benchmark_rate"]
                .as_str()
                .unwrap()
                .strip_prefix("0.")
                .unwrap()
                .parse()
                .unwrap();
            let exact = i128::from(opening) * i128::from(rate);
            let expected = (exact + 5_000) / 10_000;
            assert_eq!(
                i128::from(cents_of(&result, "interest")),
                expected,
                "{start} {opening}"
            );
        }
    }
}

/// Moving a repayment later, without passing another, never lowers the year's
/// interest. Each repayment here is at least $1,000, so the interest it adds,
/// at least $1,000 x 4.52% / 366 = 12 cents a day, outweighs the at most 2 cents
/// that rounding the 2 periods either side of it to cents can move.
#[test]
fn a_later_repayment_never_lowers_the_interest() {
    let mut rng = Rng(campaign_seed(0xD17E));
    let mut moved = 0;
    for _ in 0..300 {
        let (start, opening, mut repayments) = rollforward_case(&mut rng, 100_000);
        if repayments.is_empty() {
            continue;
        }
        repayments.sort();
        let days = days_in_year(start);
        let which = rng.int(0, repayments.len() as i64 - 1) as usize;
        let limit = repayments.get(which + 1).map_or(days, |(next, _)| *next);
        let from = repayments[which].0;
        if from + 1 >= limit {
            continue;
        }
        let before = cents_of(&rolled(start, opening, &repayments), "interest");
        repayments[which].0 = rng.int(from + 1, limit - 1);
        let after = cents_of(&rolled(start, opening, &repayments), "interest");
        assert!(
            after >= before,
            "{start} {opening} {repayments:?}: {before} then {after}"
        );
        moved += 1;
    }
    assert!(moved > 100, "{moved}");
}
