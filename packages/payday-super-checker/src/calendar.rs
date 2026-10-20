//! National business-day calendar for SGAA 1992 s 6(1).
//!
//! "business day means a day other than: (a) a Saturday or a Sunday; or
//! (b) a day which is a public holiday for the whole of: (i) any State; or
//! (ii) the Australian Capital Territory; or (iii) the Northern Territory."
//!
//! One national calendar for every employer, regardless of location. The
//! holiday table ships in data/business_days.json, compiled into the binary.
//! Part-day holidays are treated as business days; regional holidays never
//! appear in the table.

use std::collections::BTreeMap;

use crate::csv_io;
use crate::py::Date;
use crate::pytext;
use crate::raw::Raw;
use crate::{Failure, Result, repr_list};

pub const BUNDLED: &str = include_str!("../data/business_days.json");
const BUNDLED_PATH: &str = "data/business_days.json";

pub const JURISDICTIONS: [&str; 9] = ["ACT", "NSW", "NT", "QLD", "SA", "TAS", "VIC", "WA", "ALL"];

#[derive(Clone, Debug, PartialEq)]
pub struct Holiday {
    pub day: Date,
    pub name: String,
    pub jurisdictions: Vec<String>,
    pub provisional: bool,
}

fn calendar_error(message: String) -> Failure {
    Failure::Value(message)
}

/// `verified_until` is the last day the bundled table was checked against
/// official jurisdiction sources; `coverage_until` is the last day the table
/// is complete to, which an override raises only by declaring it.
#[derive(Clone, Debug, PartialEq)]
pub struct BusinessCalendar {
    holidays: BTreeMap<Date, Holiday>,
    pub verified_from: Date,
    pub verified_until: Date,
    pub coverage_until: Date,
}

impl BusinessCalendar {
    pub fn new(
        holidays: Vec<Holiday>,
        verified_from: Date,
        verified_until: Date,
        coverage_until: Option<Date>,
    ) -> BusinessCalendar {
        let mut table = BTreeMap::new();
        for h in holidays {
            table.insert(h.day, h);
        }
        let declared = coverage_until.unwrap_or(verified_until);
        BusinessCalendar {
            holidays: table,
            verified_from,
            verified_until,
            coverage_until: verified_until.max(declared),
        }
    }

    pub fn holidays(&self) -> impl Iterator<Item = &Holiday> {
        self.holidays.values()
    }

    pub fn is_business_day(&self, d: Date) -> bool {
        if d.weekday() >= 5 {
            return false;
        }
        // An unconfirmed date must not extend a statutory deadline.
        self.holidays.get(&d).is_none_or(|h| h.provisional)
    }

    /// The n-th business day after d; d itself is never counted.
    pub fn add_business_days(&self, d: Date, n: i64) -> Result<Date> {
        if n < 0 {
            return Err(calendar_error("n must be >= 0".into()));
        }
        let mut cur = d;
        let mut counted = 0;
        while counted < n {
            cur = cur.add_days(1)?;
            if self.is_business_day(cur) {
                counted += 1;
            }
        }
        Ok(cur)
    }

    /// Unconfirmed holidays in [start, end].
    pub fn provisional_hits(&self, start: Date, end: Date) -> Vec<String> {
        self.holidays
            .values()
            .filter(|h| start <= h.day && h.day <= end && h.provisional)
            .map(|h| format!("{} {}", h.day.isoformat(), h.name))
            .collect()
    }

    pub fn check_horizon(&self, d: Date) -> Option<String> {
        (d > self.coverage_until).then(|| {
            format!(
                "{} is beyond the calendar's coverage ({}, the last day the holiday table \
                 is complete to). Holidays after that date may be missing, so the \
                 real deadline can only be later than the one shown",
                d.isoformat(),
                self.coverage_until.isoformat()
            )
        })
    }
}

/// `date.fromisoformat(raw)` for a str that must also be in canonical form.
fn canonical_date(raw: &Raw) -> Option<Date> {
    let Raw::Str(text) = raw else { return None };
    let day = Date::fromisoformat(text)?;
    (day.isoformat() == *text).then_some(day)
}

fn parse_entry(e: &Raw, at: &str) -> Result<Holiday> {
    let Raw::Dict(_) = e else {
        return Err(calendar_error(format!(
            "{at} must be an object with 'date' and 'name'"
        )));
    };
    for key in ["date", "name", "jurisdictions"] {
        if e.get(key).is_none() {
            return Err(calendar_error(format!("{at} is missing '{key}'")));
        }
    }
    let raw_date = e.get("date").expect("checked");
    let Some(day) = canonical_date(raw_date) else {
        return Err(calendar_error(format!(
            "{at} has date {}; write it as YYYY-MM-DD",
            raw_date.py_repr()
        )));
    };
    let juris = e.get("jurisdictions").expect("checked");
    let Raw::List(items) = juris else {
        return Err(calendar_error(format!(
            "{at} has jurisdictions {}; write it as a list of \
             codes, e.g. [\"NSW\", \"VIC\"] or [\"ALL\"]",
            juris.py_repr()
        )));
    };
    let mut codes = Vec::new();
    for item in items {
        let Raw::Str(code) = item else {
            return Err(calendar_error(format!(
                "{at} has a jurisdiction that is not a string: {}",
                juris.py_repr()
            )));
        };
        codes.push(code.clone());
    }
    let unique: std::collections::BTreeSet<&String> = codes.iter().collect();
    if codes.is_empty()
        || unique.len() != codes.len()
        || !codes.iter().all(|c| JURISDICTIONS.contains(&c.as_str()))
    {
        return Err(calendar_error(format!(
            "{at} has invalid or duplicate jurisdictions {}; use one or more of {}",
            juris.py_repr(),
            repr_list(&crate::sorted_strings(JURISDICTIONS))
        )));
    }
    let name = e.get("name").expect("checked");
    let Raw::Str(name) = name else {
        return Err(calendar_error(format!("{at} has an invalid holiday name")));
    };
    if pytext::strip(name).is_empty() || pytext::code_points(name).any(|cp| cp < 32) {
        return Err(calendar_error(format!("{at} has an invalid holiday name")));
    }
    let provisional = match e.get("provisional") {
        None => false,
        Some(Raw::Bool(b)) => *b,
        Some(_) => {
            return Err(calendar_error(format!(
                "{at} provisional must be true or false"
            )));
        }
    };
    Ok(Holiday {
        day,
        name: pytext::strip(name).to_string(),
        jurisdictions: codes,
        provisional,
    })
}

fn calendar_date(raw: &Raw, key: &str, at: &str) -> Result<Date> {
    canonical_date(raw).ok_or_else(|| {
        calendar_error(format!(
            "{at} has {key} {}; write it as YYYY-MM-DD",
            raw.py_repr()
        ))
    })
}

/// The bundled table: its holidays in file order and its verified span.
fn bundled() -> Result<(Vec<Holiday>, Date, Date)> {
    let doc = Raw::from_json(
        &crate::pyjson::loads(&crate::py::universal_newlines(BUNDLED))
            .map_err(|e| calendar_error(format!("{BUNDLED_PATH} is not valid JSON: {e}")))?,
    );
    let Some(Raw::List(entries)) = doc.get("non_business_days") else {
        return Err(calendar_error(format!(
            "{BUNDLED_PATH}: 'non_business_days' must be a list of holiday entries"
        )));
    };
    let mut holidays: Vec<Holiday> = Vec::new();
    for (n, entry) in entries.iter().enumerate() {
        let holiday = parse_entry(entry, &format!("bundled calendar entry {}", n + 1))?;
        if holidays.iter().any(|h| h.day == holiday.day) {
            return Err(calendar_error(format!(
                "bundled calendar contains duplicate date {}",
                holiday.day.isoformat()
            )));
        }
        holidays.push(holiday);
    }
    let from = calendar_date(
        doc.get("verified_from").unwrap_or(&Raw::None),
        "verified_from",
        BUNDLED_PATH,
    )?;
    let until = calendar_date(
        doc.get("verified_until").unwrap_or(&Raw::None),
        "verified_until",
        BUNDLED_PATH,
    )?;
    Ok((holidays, from, until))
}

/// Insert or replace a holiday by day, keeping a replaced day's position.
fn put(holidays: &mut Vec<Holiday>, holiday: Holiday) {
    match holidays.iter_mut().find(|h| h.day == holiday.day) {
        Some(slot) => *slot = holiday,
        None => holidays.push(holiday),
    }
}

/// Load the bundled table, optionally patched by an override file:
/// `{"add": [...], "remove": ["2026-09-25"], "verified_until": "2029-12-31"}`.
pub fn load_calendar(override_path: Option<&str>) -> Result<BusinessCalendar> {
    let (mut holidays, verified_from, verified_until) = bundled()?;
    let mut declared_until = None;
    if let Some(path) = override_path {
        let doc = csv_io::load_json(path, |e| format!("{path} is not valid JSON: {e}"))?;
        apply_override(&mut holidays, &mut declared_until, &doc, path)?;
    }
    Ok(BusinessCalendar::new(
        holidays,
        verified_from,
        verified_until,
        declared_until,
    ))
}

/// Apply a parsed override document named `path` in messages.
pub fn apply_override(
    holidays: &mut Vec<Holiday>,
    declared_until: &mut Option<Date>,
    doc: &Raw,
    path: &str,
) -> Result<()> {
    let Raw::Dict(entries) = doc else {
        return Err(calendar_error(format!(
            "{path} must be an object with 'add' and 'remove' lists"
        )));
    };
    let unknown: Vec<&str> = entries
        .iter()
        .map(|(k, _)| k.as_str())
        .filter(|k| !["add", "remove", "verified_until"].contains(k))
        .collect();
    if !unknown.is_empty() {
        return Err(calendar_error(format!(
            "override file has unknown keys: {}",
            repr_list(&crate::sorted_strings(unknown))
        )));
    }
    if let Some(raw) = doc.get("verified_until") {
        *declared_until = Some(calendar_date(raw, "verified_until", path)?);
    }
    let empty = Raw::List(Vec::new());
    let add = doc.get("add").unwrap_or(&empty);
    let remove = doc.get("remove").unwrap_or(&empty);
    let Raw::List(add) = add else {
        return Err(calendar_error(format!(
            "{path}: 'add' must be a list of entries"
        )));
    };
    let Raw::List(remove) = remove else {
        return Err(calendar_error(format!(
            "{path}: 'remove' must be a list of dates"
        )));
    };
    let mut added: Vec<Date> = Vec::new();
    for (n, e) in add.iter().enumerate() {
        let h = parse_entry(e, &format!("{path} add entry {}", n + 1))?;
        if added.contains(&h.day) {
            return Err(calendar_error(format!(
                "override adds duplicate date {}",
                h.day.isoformat()
            )));
        }
        added.push(h.day);
        put(holidays, h);
    }
    for (n, iso) in remove.iter().enumerate() {
        let Some(day) = canonical_date(iso) else {
            return Err(calendar_error(format!(
                "{path} remove entry {} is {}; write it as YYYY-MM-DD",
                n + 1,
                iso.py_repr()
            )));
        };
        let Some(at) = holidays.iter().position(|h| h.day == day) else {
            return Err(calendar_error(format!(
                "override removes {}, which is not in the table",
                iso.py_str()
            )));
        };
        holidays.remove(at);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bundled_table_loads() {
        let cal = load_calendar(None).unwrap();
        assert_eq!(cal.verified_until.isoformat(), "2027-08-31");
        assert!(cal.holidays().count() > 10);
    }

    fn day(y: i64, m: i64, d: i64) -> Date {
        Date::new(y, m, d).unwrap()
    }

    #[test]
    fn only_whole_of_state_holidays_and_weekends_stop_the_clock() {
        let cal = load_calendar(None).unwrap();
        // Weekends; the Ekka (Brisbane only); WA Day, NT Picnic Day and the
        // AFL grand final eve holiday; WA's King's Birthday and Melbourne
        // Cup Day, which parts of their States replace.
        for (date, business) in [
            (day(2026, 8, 8), false),
            (day(2026, 8, 9), false),
            (day(2026, 8, 10), true),
            (day(2026, 8, 12), true),
            (day(2027, 6, 7), false),
            (day(2026, 8, 3), false),
            (day(2026, 9, 25), false),
            (day(2026, 9, 28), true),
            (day(2026, 11, 3), true),
        ] {
            assert_eq!(cal.is_business_day(date), business, "{}", date.isoformat());
        }
    }

    #[test]
    fn no_bank_holiday_reaches_the_shipped_table() {
        let table = crate::pyjson::loads(BUNDLED).unwrap();
        let table = crate::raw::Raw::from_json(&table);
        let Some(crate::raw::Raw::List(entries)) = table.get("non_business_days") else {
            panic!("the table lists its non-business days");
        };
        for entry in entries {
            let name = entry.get("name").unwrap().py_str().to_lowercase();
            assert!(!name.contains("bank holiday"), "{name}");
        }
    }

    #[test]
    fn the_horizon_warns_past_the_calendar_coverage() {
        let cal = load_calendar(None).unwrap();
        assert_eq!(cal.check_horizon(day(2027, 1, 1)), None);
        assert_eq!(cal.check_horizon(day(2027, 8, 31)), None);
        for past in [day(2027, 9, 1), day(2029, 1, 1)] {
            assert!(
                cal.check_horizon(past)
                    .unwrap()
                    .contains("beyond the calendar's coverage")
            );
        }
    }

    #[test]
    fn provisional_hits_flag_grand_final_day() {
        let cal = load_calendar(None).unwrap();
        let hits = cal.provisional_hits(day(2027, 9, 1), day(2027, 10, 1));
        assert!(hits.iter().any(|h| h.contains("Grand Final")));
        assert!(
            cal.provisional_hits(day(2026, 7, 1), day(2026, 7, 31))
                .is_empty()
        );
    }
}
