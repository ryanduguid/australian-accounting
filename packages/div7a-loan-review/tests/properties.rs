// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The s 109E(6) formula's seeded properties. Seeds, run counts and input
//! domains are the Python fast-check properties'; a local SplitMix64 replaces
//! fast-check's generator, and exact rational arithmetic is the reference.

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
    let mut rng = Rng(seed);
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
    let mut rng = Rng(0xD17E);
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
    let mut rng = Rng(0xD180);
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
