// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Calendar days for the main residence and borrowing expenses worksheets: ISO
//! dates as day numbers (days since 1970-01-01, proleptic Gregorian) and
//! calendar month steps.

/// A strict `YYYY-MM-DD` date from 1900 to 2100, as its day number.
pub fn parse(text: &str) -> Option<i64> {
    let b = text.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let num = |r: std::ops::Range<usize>| -> Option<i64> {
        let part = &text[r];
        part.bytes()
            .all(|c| c.is_ascii_digit())
            .then(|| part.parse().ok())?
    };
    let (y, m, d) = (num(0..4)?, num(5..7)?, num(8..10)?);
    let valid =
        (1900..=2100).contains(&y) && (1..=12).contains(&m) && (1..=month_days(y, m)).contains(&d);
    valid.then(|| from_civil(y, m, d))
}

/// The day number's `YYYY-MM-DD`.
pub fn format(day: i64) -> String {
    let (y, m, d) = to_civil(day);
    format!("{y:04}-{m:02}-{d:02}")
}

/// The same day of the month `months` later (or earlier), or the month's last
/// day when it is shorter: 31 August plus 6 months is 28 or 29 February.
pub fn add_months(day: i64, months: i64) -> i64 {
    let (y, m, d) = to_civil(day);
    let index = y * 12 + (m - 1) + months;
    let (y, m) = (index.div_euclid(12), index.rem_euclid(12) + 1);
    from_civil(y, m, d.min(month_days(y, m)))
}

/// The earliest day that is `months` months before `day` counting both ends
/// in the same day of the month: 1 November for 1 May and 6 months. Where that
/// day of the month does not exist, the first of the next month, so the period
/// never runs longer: 1 March for 31 August, not 28 February.
pub fn months_before(day: i64, months: i64) -> i64 {
    let start = add_months(day, -months);
    if add_months(start, months) == day {
        start
    } else {
        start + 1
    }
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn month_days(y: i64, m: i64) -> i64 {
    match m {
        2 if is_leap(y) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

// Howard Hinnant's days_from_civil and civil_from_days.
fn from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The year, month and day of a day number.
pub fn to_civil(day: i64) -> (i64, i64, i64) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_every_day_in_range() {
        let (first, last) = (parse("1900-01-01").unwrap(), parse("2100-12-31").unwrap());
        assert_eq!(first, -25_567);
        for day in first..=last {
            assert_eq!(parse(&format(day)), Some(day));
        }
    }

    #[test]
    fn refuses_malformed_dates() {
        for bad in [
            "2026-02-29",
            "2026-13-01",
            "2026-00-10",
            "2026-1-01",
            "26-01-01",
            "2026/01/01",
            "1899-12-31",
            "2101-01-01",
            "+026-01-01",
            "2026-01-0a",
        ] {
            assert_eq!(parse(bad), None, "{bad}");
        }
        assert!(parse("2024-02-29").is_some());
    }

    #[test]
    fn month_steps_clamp_to_the_last_day() {
        let step = |t: &str, n| format(add_months(parse(t).unwrap(), n));
        assert_eq!(step("2016-07-01", 72), "2022-07-01");
        assert_eq!(step("2026-05-01", -6), "2025-11-01");
        assert_eq!(step("2025-08-31", 6), "2026-02-28");
        assert_eq!(step("2024-02-29", 12), "2025-02-28");
        assert_eq!(step("2026-03-31", -1), "2026-02-28");
        let before = |t: &str, n| format(months_before(parse(t).unwrap(), n));
        assert_eq!(before("2026-05-01", 6), "2025-11-01");
        assert_eq!(before("2026-08-31", 6), "2026-03-01");
        assert_eq!(before("2026-02-28", 12), "2025-02-28");
    }
}
