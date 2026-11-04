//! `datetime.strptime(text, "%d %B %Y").date()` in the C locale, as CPython
//! 3.15's `_strptime` reads it: the regular expression it builds for the
//! format, matched case-insensitively from the start, then the month looked
//! up by `str.lower()` and the date checked by `datetime.date`.
use std::sync::LazyLock;

use crate::py::Date;
use crate::pyvalue::repr_points;
use crate::sre::{self, Pattern};

const FORMAT: &str = "%d %B %Y";

const MONTHS: [&str; 12] = [
    "january",
    "february",
    "march",
    "april",
    "may",
    "june",
    "july",
    "august",
    "september",
    "october",
    "november",
    "december",
];

/// `_TimeRE_cache.compile("%d %B %Y")`: `%B` lists the month names longest first.
static REGEX: LazyLock<Pattern> = LazyLock::new(|| {
    sre::compile_str(
        r"(?P<d>3[0-1]|[1-2]\d|0[1-9]|[1-9]| [1-9])\s+(?P<B>september|february|november|december|january|october|august|march|april|june|july|may)\s+(?P<Y>\d\d\d\d)",
        sre::IGNORECASE,
    )
});

fn points(text: &str) -> Vec<u32> {
    text.chars().map(u32::from).collect()
}

/// `int()` of the digits a group matched: any Unicode decimal digits, with
/// the one leading space `%d` allows.
fn int(digits: &[u32]) -> i64 {
    digits
        .iter()
        .filter_map(|&cp| char::from_u32(cp).and_then(crate::py::decimal_value))
        .fold(0, |n, d| n * 10 + i64::from(d))
}

/// The date, or the `ValueError` message as code points.
pub fn date(text: &[u32]) -> Result<Date, Vec<u32>> {
    let Some(found) = REGEX.match_start(text) else {
        return Err(points(&format!(
            "time data {} does not match format {}",
            repr_points(text),
            crate::py::repr_str(FORMAT)
        )));
    };
    if found.end() != text.len() {
        let mut message = points("unconverted data remains: ");
        message.extend_from_slice(&text[found.end()..]);
        return Err(message);
    }
    let group = |i: usize| {
        let (start, end) = found.span(i).unwrap_or((0, 0));
        &text[start..end]
    };
    let day = int(group(1));
    let name = sre::str_lower(group(2));
    let month = MONTHS
        .iter()
        .position(|m| m.chars().map(u32::from).eq(name.iter().copied()))
        .ok_or_else(|| points("list.index(x): x not in list"))?;
    let year = int(group(3));
    Date::new(year, month as i64 + 1, day).map_err(|e| points(&e))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(text: &str) -> Result<String, String> {
        date(&points(text))
            .map(|d| d.isoformat())
            .map_err(|e| e.into_iter().filter_map(char::from_u32).collect())
    }

    #[test]
    fn reads_dates_as_cpython_does() {
        assert_eq!(run("1 July 2026").unwrap(), "2026-07-01");
        assert_eq!(run("01 JULY 2026").unwrap(), "2026-07-01");
        assert_eq!(
            run("1 August").unwrap_err(),
            "time data '1 August' does not match format '%d %B %Y'"
        );
        assert_eq!(
            run("31 February 2026").unwrap_err(),
            "day 31 must be in range 1..28 for month 2 in year 2026"
        );
        assert_eq!(
            run("1 Aprİl 2026").unwrap_err(),
            "list.index(x): x not in list"
        );
        assert_eq!(
            run("1 Auguſt 2026").unwrap_err(),
            "list.index(x): x not in list"
        );
        assert_eq!(
            run("1 August 2026x").unwrap_err(),
            "unconverted data remains: x"
        );
        assert_eq!(
            run("1 August 0000").unwrap_err(),
            "year must be in 1..9999, not 0"
        );
    }
}
