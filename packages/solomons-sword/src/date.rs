//! A calendar date in Python's `date.isoformat()` form. Years 1 to 9999 are
//! always four digits, so the text orders the same way the dates do.

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date(String);

fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        _ => 0,
    }
}

impl Date {
    /// `datetime.date(year, month, day)`, with its range message.
    pub fn new(year: i64, month: u32, day: u32) -> Result<Date, String> {
        if !(1..=9999).contains(&year) {
            return Err(format!("year must be in 1..9999, not {year}"));
        }
        if day == 0 || day > days_in_month(year, month) {
            return Err("day is out of range for month".into());
        }
        Ok(Date(format!("{year:04}-{month:02}-{day:02}")))
    }

    /// A `YYYY-MM-DD` date.
    pub fn parse(text: &str) -> Option<Date> {
        let b = text.as_bytes();
        let shaped = b.len() == 10
            && b[4] == b'-'
            && b[7] == b'-'
            && b.iter()
                .enumerate()
                .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit());
        if !shaped {
            return None;
        }
        let year = text[..4].parse().ok()?;
        let month = text[5..7].parse().ok()?;
        let day = text[8..].parse().ok()?;
        Date::new(year, month, day).ok()
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
