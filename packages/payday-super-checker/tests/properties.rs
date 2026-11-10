// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Seeded date-boundary and exact-money properties. Seeds, run counts,
//! examples and input domains are the Python fast-check properties'; a local
//! SplitMix64 replaces fast-check's generator.

use payday_super_checker::api;
use payday_super_checker::assess::VERDICTS;
use payday_super_checker::calendar::{BusinessCalendar, load_calendar};
use payday_super_checker::decimal::Decimal;
use payday_super_checker::py::Date;
use payday_super_checker::sgc::{exposure_range, scenario, uplift_scenarios};
use serde_json::{Value, json};

// The verdicts that carry exposure figures, spelled here so the engine's list
// cannot move them.
const EXPOSED: [&str; 2] = ["LATE", "UNPAID"];

const RUNS: usize = 100;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

fn date(text: &str) -> Date {
    Date::fromisoformat(text).unwrap()
}

fn dec(text: &str) -> Decimal {
    Decimal::parse(text).unwrap()
}

fn check_business_days(cal: &BusinessCalendar, start: Date, count: i64) {
    let result = cal.add_business_days(start, count).unwrap();
    if count == 0 {
        assert_eq!(result, start);
        return;
    }
    assert!(result > start);
    assert!(cal.is_business_day(result));
    let elapsed = result.days_since(start);
    let business = |offset: i64| cal.is_business_day(start.add_days(offset).unwrap());
    let counted = (1..=elapsed).filter(|o| business(*o)).count() as i64;
    let before = (1..elapsed).filter(|o| business(*o)).count() as i64;
    assert_eq!(counted, count, "{} + {count}", start.isoformat());
    assert_eq!(before, count - 1);
}

#[test]
fn business_day_addition_counts_only_days_after_the_start() {
    let cal = load_calendar(None).unwrap();
    for (start, count) in [("2026-12-24", 7), ("2027-03-25", 7), ("2026-08-01", 0)] {
        check_business_days(&cal, date(start), count);
    }
    let mut rng = Rng(campaign_seed(0xDA7E));
    let first = date("2026-07-01");
    let span = date("2027-07-01").days_since(first) as u64 + 1;
    for _ in 0..RUNS {
        let start = first.add_days(rng.below(span) as i64).unwrap();
        let count = rng.below(41) as i64;
        check_business_days(&cal, start, count);
    }
}

fn check_exposure(final_shortfall: &Decimal, nec: &Decimal) {
    let base = final_shortfall.add(nec).unwrap();
    let hundred = dec("100");
    let scenarios = uplift_scenarios(final_shortfall, nec).unwrap();
    for (history, row) in [
        (
            "clean_history",
            [
                ("vds_within_30d", "0"),
                ("vds_31_60d", "5"),
                ("vds_61_120d", "10"),
                ("vds_after_120d", "25"),
                ("no_vds", "40"),
            ],
        ),
        (
            "prior_history",
            [
                ("vds_within_30d", "20"),
                ("vds_31_60d", "25"),
                ("vds_61_120d", "30"),
                ("vds_after_120d", "45"),
                ("no_vds", "60"),
            ],
        ),
    ] {
        for (name, pct) in row {
            let want = base.mul(&dec(pct)).unwrap().div(&hundred).unwrap();
            assert!(
                scenario(&scenarios, history, name).equals(&want).unwrap(),
                "{history} {name}"
            );
        }
    }
    let (low, high) = exposure_range(final_shortfall, nec).unwrap();
    assert!(low.equals(&base).unwrap());
    assert!(high.equals(&base.mul(&dec("1.60")).unwrap()).unwrap());
}

#[test]
fn exposure_scenarios_conserve_the_shortfall_plus_nec_base() {
    for (shortfall, nec) in [("0", "0"), ("1000", "100")] {
        check_exposure(&dec(shortfall), &dec(nec));
    }
    let mut rng = Rng(campaign_seed(0x5AC));
    // 0.00 to 1,000,000,000.00, in cents.
    let money = |rng: &mut Rng| {
        Decimal::from_int(rng.below(100_000_000_001) as i64)
            .div(&dec("100"))
            .unwrap()
            .quantize(-2, payday_super_checker::decimal::Rounding::HalfEven)
            .unwrap()
    };
    for _ in 0..RUNS {
        let shortfall = money(&mut rng);
        let nec = money(&mut rng);
        check_exposure(&shortfall, &nec);
    }
}

// Metamorphic relations: how the answer must move when one fact moves. Every
// date stays inside the bundled calendar, so no line is left undecided by a
// table running out, and after the 28 July 2026 transition.

/// One contribution's facts, in cents and dates.
#[derive(Debug, Clone)]
struct Case {
    qe_day: Date,
    cents: i64,
    first_to_fund: bool,
}

fn case(rng: &mut Rng) -> Case {
    Case {
        qe_day: date("2026-07-29").add_days(rng.below(120) as i64).unwrap(),
        cents: 1 + rng.below(2_000_000) as i64,
        first_to_fund: rng.below(2) == 1,
    }
}

fn dollars(cents: i64) -> String {
    format!("{}.{:02}", cents / 100, cents % 100)
}

fn after(day: Date, days: u64) -> Date {
    day.add_days(days as i64).unwrap()
}

fn assessed(line: Value, as_at: Date) -> Value {
    let results = api::evaluate(
        "assess",
        &json!({"lines": [line], "as_at": as_at.isoformat(), "allow_stale_gic": true}),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let result = results[0].clone();
    check_sanity(&result);
    result
}

fn line(case: &Case) -> Value {
    json!({
        "employee_id": "E1",
        "qe_day": case.qe_day.isoformat(),
        "sg_amount": dollars(case.cents),
        "first_to_fund": case.first_to_fund,
    })
}

/// The line after the fund received `matched_cents` of it on `day`.
fn received(case: &Case, day: Date, matched_cents: i64) -> Value {
    let mut out = line(case);
    out["remitted"] = json!(case.qe_day.isoformat());
    out["remitted_amount"] = json!(dollars(matched_cents));
    out["received"] = json!(day.isoformat());
    out["matched_amount"] = json!(dollars(matched_cents));
    out
}

/// A money string in cents. Anything else fails the test.
fn cents_at(value: &Value) -> i64 {
    let text = value
        .as_str()
        .unwrap_or_else(|| panic!("{value} is not a money string"));
    let (whole, part) = text.split_once('.').unwrap_or((text, "00"));
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    assert!(
        digits(whole) && digits(part) && part.len() <= 2,
        "{text} is not a money string"
    );
    format!("{whole}{part:0<2}").parse().unwrap()
}

/// A money field that must be present, and may be null where nothing is owed.
fn optional_cents(result: &Value, key: &str) -> i64 {
    let value = result
        .get(key)
        .unwrap_or_else(|| panic!("no {key} in {result}"));
    if value.is_null() { 0 } else { cents_at(value) }
}

#[test]
fn a_money_string_reads_to_the_cent() {
    for (text, cents) in [("120.00", 12_000), ("0.5", 50), ("7", 700)] {
        assert_eq!(cents_at(&json!(text)), cents, "{text}");
    }
}

#[test]
#[should_panic(expected = "1..2 is not a money string")]
fn a_doubled_decimal_point_is_not_a_money_string() {
    cents_at(&json!("1..2"));
}

fn figure(result: &Value, key: &str) -> i64 {
    optional_cents(&result["figures"], key)
}

/// A count that must be present, and is null on a line that was not late.
fn days_late(result: &Value) -> i64 {
    let value = result
        .get("days_late")
        .unwrap_or_else(|| panic!("no days_late in {result}"));
    if value.is_null() {
        return 0;
    }
    value
        .as_i64()
        .unwrap_or_else(|| panic!("days_late is {value}"))
}

/// What every assessed line must say, whatever its facts.
fn check_sanity(result: &Value) {
    let verdict = result["verdict"].as_str().unwrap();
    assert!(VERDICTS.contains(&verdict), "{result}");
    let sg = cents_at(&result["line"]["sg_amount"]);
    let (base, last) = (
        optional_cents(result, "base_shortfall"),
        optional_cents(result, "final_shortfall"),
    );
    assert!((0..=sg).contains(&base), "{result}");
    assert!((0..=base).contains(&last), "{result}");
    assert!(days_late(result) >= 0, "{result}");
    assert!(figure(result, "low") <= figure(result, "high"), "{result}");
    if EXPOSED.contains(&verdict) {
        assert!(result["days_late"].is_i64(), "{result}");
        for figure in [
            &result["base_shortfall"],
            &result["final_shortfall"],
            &result["figures"]["low"],
            &result["figures"]["high"],
        ] {
            assert!(figure.is_string(), "{result}");
        }
    } else {
        assert_eq!(last, 0, "{result}");
        assert_eq!(figure(result, "high"), 0, "{result}");
    }
    if verdict == "ON_TIME" {
        assert_eq!(days_late(result), 0, "{result}");
        let due = result["deadline"]["due"].as_str().unwrap();
        assert!(
            result["line"]["received"].as_str().unwrap() <= due,
            "{result}"
        );
    }
    if verdict == "LATE" {
        assert!(days_late(result) > 0, "{result}");
    }
}

#[test]
fn a_later_receipt_never_improves_timeliness() {
    let mut rng = Rng(campaign_seed(0xFACE));
    for _ in 0..RUNS {
        let case = case(&mut rng);
        let earlier = after(case.qe_day, rng.below(45));
        let later = after(earlier, rng.below(45));
        let as_at = after(later, rng.below(30));
        let first = assessed(received(&case, earlier, case.cents), as_at);
        let second = assessed(received(&case, later, case.cents), as_at);
        let rank = |result: &Value| match result["verdict"].as_str().unwrap() {
            "ON_TIME" => 0,
            "LATE" => 1,
            other => panic!("{other} from a whole receipt: {result}"),
        };
        assert!(rank(&first) <= rank(&second), "{case:?} {first} {second}");
        assert!(days_late(&first) <= days_late(&second), "{case:?}");
        assert!(figure(&first, "low") <= figure(&second, "low"), "{case:?}");
        assert!(
            figure(&first, "high") <= figure(&second, "high"),
            "{case:?}"
        );
    }
}

#[test]
fn more_of_the_amount_received_never_raises_the_shortfall() {
    let mut rng = Rng(campaign_seed(0xFACF));
    for _ in 0..RUNS {
        let case = case(&mut rng);
        let less = rng.below(case.cents as u64 + 1) as i64;
        let more = less + rng.below((case.cents - less) as u64 + 1) as i64;
        let day = after(case.qe_day, rng.below(5));
        let as_at = after(day, 60 + rng.below(30));
        let first = assessed(received(&case, day, less), as_at);
        let second = assessed(received(&case, day, more), as_at);
        assert!(
            optional_cents(&second, "final_shortfall") <= optional_cents(&first, "final_shortfall"),
            "{case:?} {first} {second}"
        );
        assert!(
            figure(&second, "high") <= figure(&first, "high"),
            "{case:?}"
        );
    }
}

#[test]
fn a_later_review_date_never_lowers_an_unpaid_exposure() {
    let mut rng = Rng(campaign_seed(0xFAD0));
    for _ in 0..RUNS {
        let case = case(&mut rng);
        let earlier = after(case.qe_day, rng.below(90));
        let later = after(earlier, rng.below(90));
        let first = assessed(line(&case), earlier);
        let second = assessed(line(&case), later);
        assert!(
            optional_cents(&first, "final_shortfall") <= optional_cents(&second, "final_shortfall"),
            "{case:?}"
        );
        assert!(
            figure(&first, "high") <= figure(&second, "high"),
            "{case:?}"
        );
        assert!(days_late(&first) <= days_late(&second), "{case:?}");
    }
}

#[test]
fn a_return_never_leaves_a_line_on_time_or_late() {
    let mut rng = Rng(campaign_seed(0xFAD1));
    for _ in 0..RUNS {
        let case = case(&mut rng);
        let day = after(case.qe_day, rng.below(60));
        let as_at = after(day, rng.below(30));
        let mut flagged = received(&case, day, case.cents);
        flagged["returned_by_fund"] = json!(true);
        let result = assessed(flagged, as_at);
        assert_eq!(result["verdict"], "UNKNOWN", "{case:?} {result}");
    }
}

#[test]
fn a_first_contribution_never_shortens_the_deadline() {
    let mut rng = Rng(campaign_seed(0xFAD2));
    for _ in 0..RUNS {
        let mut case = case(&mut rng);
        let due = |case: &Case| {
            let deadline = api::evaluate("compute-due", &json!({"line": line(case)})).unwrap();
            deadline["due"].as_str().unwrap().to_string()
        };
        case.first_to_fund = false;
        let usual = due(&case);
        case.first_to_fund = true;
        assert!(usual <= due(&case), "{case:?}");
        let mut next_day = case.clone();
        next_day.qe_day = after(case.qe_day, 1);
        assert!(due(&case) <= due(&next_day), "{case:?}");
    }
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
