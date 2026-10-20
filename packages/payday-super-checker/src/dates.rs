//! `datetime` behaviour the engine depends on: date arithmetic with Python's
//! `OverflowError`, `datetime.strptime` through the regexes `_strptime`
//! compiles, and `datetime.fromisoformat` for the ISO shapes the reader
//! admits.

use std::cell::RefCell;
use std::collections::HashMap;

use crate::py::{Date, MAX_DATE};
use crate::pyre::Regex;
use crate::pytext;
use crate::strptime_table::{FULL_MONTHS, MONTHS, PATTERNS};

/// `date + timedelta(days=n)` past `date.min` or `date.max` raises
/// `OverflowError("date value out of range")`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Overflow;

impl Date {
    /// `self + timedelta(days=days)`.
    pub fn add_days(self, days: i64) -> Result<Date, Overflow> {
        Date::from_ordinal(self.ordinal() + days).ok_or(Overflow)
    }

    /// `(self - other).days`.
    pub fn days_since(self, other: Date) -> i64 {
        self.ordinal() - other.ordinal()
    }

    /// `date.weekday()`: Monday is 0.
    pub fn weekday(self) -> i64 {
        (self.ordinal() + 6) % 7
    }

    /// `date.replace(year=year)`, or None where that day does not exist.
    pub fn with_year(self, year: i64) -> Option<Date> {
        Date::new(year, i64::from(self.month), i64::from(self.day)).ok()
    }

    pub fn is_max(self) -> bool {
        self == MAX_DATE
    }
}

thread_local! {
    static COMPILED: RefCell<HashMap<&'static str, std::rc::Rc<Regex>>> = RefCell::new(HashMap::new());
}

fn regex_for(format: &str) -> Option<std::rc::Rc<Regex>> {
    let (key, pattern) = PATTERNS.iter().find(|(f, _)| *f == format)?;
    Some(COMPILED.with(|cache| {
        cache
            .borrow_mut()
            .entry(key)
            .or_insert_with(|| std::rc::Rc::new(Regex::ignore_case(pattern)))
            .clone()
    }))
}

/// `int(text)` for a run of Unicode decimal digits, with any leading spaces.
fn int(text: &str) -> Option<i64> {
    let mut value: i64 = 0;
    let mut seen = false;
    for cp in pytext::code_points(text) {
        if cp == 0x20 && !seen {
            continue;
        }
        value = value.checked_mul(10)? + i64::from(pytext::decimal_value(cp)?);
        seen = true;
    }
    seen.then_some(value)
}

/// `datetime.strptime(text, format).date()`, or None where it raises
/// `ValueError`. `format` must be one of the generated table's formats.
pub fn strptime_date(text: &str, format: &str) -> Option<Date> {
    let regex = regex_for(format).unwrap_or_else(|| panic!("no strptime pattern for {format}"));
    let found = regex.captures(text)?;
    // "unconverted data remains": `match` stops at its first success.
    if found.end != pytext::code_points(text).count() {
        return None;
    }
    let year = if let Some(y) = found.name("Y") {
        int(&y)?
    } else {
        let y = int(&found.name("y")?)?;
        if y <= 68 { y + 2000 } else { y + 1900 }
    };
    let month = if let Some(m) = found.name("m") {
        int(&m)?
    } else {
        // `a_month.index(found.lower())`: only an ASCII spelling is in the list.
        let (text, names) = match found.name("b") {
            Some(b) => (b, MONTHS),
            None => (found.name("B")?, FULL_MONTHS),
        };
        if !text.is_ascii() {
            return None;
        }
        let lowered = text.to_ascii_lowercase();
        names.iter().position(|n| *n == lowered)? as i64 + 1
    };
    let day = int(&found.name("d")?)?;
    if let Some(second) = found.name("S")
        && int(&second)? > 59
    {
        return None;
    }
    Date::new(year, month, day).ok()
}

fn ascii_number(text: &[u8]) -> Option<i64> {
    if text.is_empty() || !text.iter().all(u8::is_ascii_digit) {
        return None;
    }
    std::str::from_utf8(text).ok()?.parse().ok()
}

/// `datetime.fromisoformat(text).date()` for a text in the reader's ISO
/// shape: `YYYY-MM-DD`, optionally followed by `T` or a space and `HH:MM`,
/// `HH:MM:SS` or `HH:MM:SS.ffffff`. Only ASCII digits parse, and 24:00
/// is midnight at the start of the next day.
pub fn fromisoformat_date(text: &str) -> Option<Date> {
    let b = text.as_bytes();
    if b.len() < 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let date = Date::new(
        ascii_number(&b[0..4])?,
        ascii_number(&b[5..7])?,
        ascii_number(&b[8..10])?,
    )
    .ok()?;
    if b.len() == 10 {
        return Some(date);
    }
    let time = &b[11..];
    let (hour, minute, second, micro) = match time.len() {
        5 if time[2] == b':' => (ascii_number(&time[0..2])?, ascii_number(&time[3..5])?, 0, 0),
        8 if time[2] == b':' && time[5] == b':' => (
            ascii_number(&time[0..2])?,
            ascii_number(&time[3..5])?,
            ascii_number(&time[6..8])?,
            0,
        ),
        15 if time[2] == b':' && time[5] == b':' && time[8] == b'.' => (
            ascii_number(&time[0..2])?,
            ascii_number(&time[3..5])?,
            ascii_number(&time[6..8])?,
            ascii_number(&time[9..15])?,
        ),
        _ => return None,
    };
    if hour == 24 && minute == 0 && second == 0 && micro == 0 {
        return date.add_days(1).ok();
    }
    (hour <= 23 && minute <= 59 && second <= 59).then_some(date)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn iso(d: Option<Date>) -> Option<String> {
        d.map(|d| d.isoformat())
    }

    #[test]
    fn strptime_follows_python() {
        assert_eq!(
            iso(strptime_date("01/07/68", "%d/%m/%y")).as_deref(),
            Some("2068-07-01")
        );
        assert_eq!(
            iso(strptime_date("01/07/69", "%d/%m/%y")).as_deref(),
            Some("1969-07-01")
        );
        assert_eq!(strptime_date("1 \u{17F}ep 2026", "%d %b %Y"), None);
        assert_eq!(
            iso(strptime_date("1 SEP 2026", "%d %b %Y")).as_deref(),
            Some("2026-09-01")
        );
        assert_eq!(
            iso(strptime_date(
                "01/07/\u{662}\u{660}\u{662}\u{666}",
                "%d/%m/%Y"
            ))
            .as_deref(),
            Some("2026-07-01")
        );
        assert_eq!(
            strptime_date("01/07/2026 12:00:60", "%d/%m/%Y %H:%M:%S"),
            None
        );
        assert_eq!(
            strptime_date("01/07/2026 13:00 PM", "%d/%m/%Y %I:%M %p"),
            None
        );
        assert_eq!(
            iso(strptime_date("01/07/2026\t\t12:00", "%d/%m/%Y %H:%M")).as_deref(),
            Some("2026-07-01")
        );
        assert_eq!(strptime_date("01/07/2026 12:00 ", "%d/%m/%Y %H:%M"), None);
        assert_eq!(strptime_date("29/02/2026", "%d/%m/%Y"), None);
        assert_eq!(
            iso(strptime_date("0001-01-01", "%Y-%m-%d")).as_deref(),
            Some("0001-01-01")
        );
        assert_eq!(strptime_date("0000-01-01", "%Y-%m-%d"), None);
    }

    #[test]
    fn fromisoformat_follows_python() {
        assert_eq!(
            iso(fromisoformat_date("2026-07-31T24:00")).as_deref(),
            Some("2026-08-01")
        );
        assert_eq!(fromisoformat_date("9999-12-31T24:00"), None);
        assert_eq!(fromisoformat_date("2026-07-31T23:59:60"), None);
        assert_eq!(fromisoformat_date("2026-07-31T24:01"), None);
        assert_eq!(
            iso(fromisoformat_date("2026-07-01 12:00:00.123456")).as_deref(),
            Some("2026-07-01")
        );
    }
}
