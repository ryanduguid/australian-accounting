//! Seeded date-boundary and exact-money properties. Seeds, run counts,
//! examples and input domains are the Python fast-check properties'; a local
//! SplitMix64 replaces fast-check's generator.

use payday_super_checker::calendar::{BusinessCalendar, load_calendar};
use payday_super_checker::decimal::Decimal;
use payday_super_checker::py::Date;
use payday_super_checker::sgc::{exposure_range, scenario, uplift_scenarios};

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
    let mut rng = Rng(0xDA7E);
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
    let mut rng = Rng(0x5AC);
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
