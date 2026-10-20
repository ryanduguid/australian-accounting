//! The year of income, written the way the Act's readers write it: 2026-27.
//! A year starts on 1 July; a substituted accounting period is outside v1.

use crate::pytext;
use crate::raw::Raw;

// Typo guards, not statutory limits; the frozen rate table is the real gate.
pub const EARLIEST: i64 = 1900;
pub const LATEST: i64 = 2100;

/// A year of income identified by the calendar year it starts in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct YearOfIncome {
    pub start_year: i64,
}

impl YearOfIncome {
    pub fn new(start_year: i64) -> Result<YearOfIncome, String> {
        if !(EARLIEST..=LATEST).contains(&start_year) {
            return Err(format!(
                "year of income {start_year}-.. is outside {EARLIEST}-{LATEST}; \
                 this looks like a typo rather than a year"
            ));
        }
        Ok(YearOfIncome { start_year })
    }

    pub fn label(&self) -> String {
        format!("{}-{:02}", self.start_year, (self.start_year + 1) % 100)
    }

    /// 1 July, in ISO form: s 109N(2) takes the rate last published before it.
    pub fn starts_on(&self) -> String {
        format!("{:04}-07-01", self.start_year)
    }
}

/// The value of a run of Unicode decimal digits, as `int()` reads `\d` matches.
fn digits_value(cps: &[u32]) -> Option<i64> {
    cps.iter().try_fold(0i64, |acc, cp| {
        Some(acc * 10 + i64::from(pytext::decimal_value(*cp)?))
    })
}

/// `parse_year(raw, where)`: '2026-27', with the second half checked against the first.
pub fn parse_year(raw: &Raw, where_: &str) -> Result<YearOfIncome, String> {
    let shown = raw.py_str();
    let text = pytext::strip(&shown);
    let cps: Vec<u32> = pytext::code_points(text).collect();
    let matched = cps.len() == 7 && cps[4] == '-' as u32;
    let start = matched.then(|| digits_value(&cps[..4])).flatten();
    let second = matched.then(|| digits_value(&cps[5..])).flatten();
    let (Some(start), Some(_)) = (start, second) else {
        return Err(format!(
            "{where_} {} must be written like 2026-27",
            raw.py_repr()
        ));
    };
    let expected = format!("{:02}", (start + 1) % 100);
    let written: String = text.chars().skip(5).collect();
    if expected != written {
        return Err(format!(
            "{where_} {} is not a year of income: {start} is followed by {expected}, not {written}",
            raw.py_repr()
        ));
    }
    YearOfIncome::new(start)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_and_refusals_match_python() {
        let year = |t: &str| parse_year(&Raw::text(t), "year of income");
        assert_eq!(year(" 2026-27 ").unwrap().label(), "2026-27");
        assert_eq!(year("1999-00").unwrap().label(), "1999-00");
        assert_eq!(
            year("2026-28").unwrap_err(),
            "year of income '2026-28' is not a year of income: 2026 is followed by 27, not 28"
        );
        assert_eq!(
            year("2026").unwrap_err(),
            "year of income '2026' must be written like 2026-27"
        );
        assert_eq!(
            year("\u{ff12}\u{ff10}\u{ff12}\u{ff16}-27").unwrap().label(),
            "2026-27"
        );
        assert_eq!(
            year("1800-01").unwrap_err(),
            "year of income 1800-.. is outside 1900-2100; this looks like a typo rather than a year"
        );
    }

    /// Every character Python's `str.isspace` accepts trims around a year or an
    /// unknown token and is refused inside one; the workbook's year helper
    /// relies on the same set.
    #[test]
    fn python_whitespace_trims_around_a_year_but_not_inside_it() {
        use crate::facts::{UNKNOWN_TOKENS, optional_year_of_income};
        let spaces: Vec<char> = crate::tables::PY_SPACE
            .iter()
            .flat_map(|&(low, high)| (low..=high).filter_map(char::from_u32))
            .collect();
        assert_eq!(spaces.len(), 29);
        let text = |s: String| Raw::Str(s);
        for c in spaces {
            let padded = parse_year(&text(format!("{c}2023-24{c}")), "year").unwrap();
            assert_eq!(padded.label(), "2023-24");
            for token in UNKNOWN_TOKENS {
                let raw = text(format!("{c}{token}{c}"));
                assert_eq!(optional_year_of_income(Some(&raw), "year"), Ok(None));
            }
            for bad in [
                format!("20{c}23-24"),
                format!("2023-{c}24"),
                format!("un{c}known"),
                format!("n{c}/a"),
            ] {
                assert!(optional_year_of_income(Some(&text(bad)), "year").is_err());
            }
        }
        for c in ['\u{200b}', '\u{feff}'] {
            assert!(parse_year(&text(format!("{c}2023-24{c}")), "year").is_err());
        }
    }
}
