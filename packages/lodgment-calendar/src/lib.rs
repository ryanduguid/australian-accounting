//! Local preparation aid for the recorded 2026-27 ATO lodgment program.
//! Missing facts, ambiguous rules and incomplete calendars remain UNKNOWN.
pub mod api;
pub mod cli;
pub mod holidays;
mod returns;
mod statements;

use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::Value;
use std::sync::OnceLock;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const CHECKED: &str = "2026-10-28";

/// What the program records about its period and sources, for a freshness report.
pub fn provenance() -> Value {
    serde_json::json!({"program_year":"2026-27","activity_year":"2026-27","return_year":"2025-26",
        "covers_from":"2026-07-01","covers_to":"2027-06-30","source_checked":CHECKED,
        "sources":data()["sources"],"holiday_provision":data()["holiday_provision"]})
}

fn data() -> &'static Value {
    static DATA: OnceLock<Value> = OnceLock::new();
    DATA.get_or_init(|| {
        serde_json::from_str(include_str!("../data/program-2026-27.json"))
            .expect("compiled program data")
    })
}

#[derive(Clone, Deserialize)]
#[serde(untagged)]
enum Payment {
    Mode(String),
    Separate(Box<Rule>),
}

#[derive(Clone, Deserialize)]
struct Rule {
    id: String,
    summary: String,
    source: String,
    nominal: Option<NaiveDate>,
    unresolved: Option<String>,
    payment: Payment,
}

impl Rule {
    fn new(id: &str, summary: impl Into<String>, source: &str, nominal: Option<NaiveDate>) -> Self {
        Self {
            id: id.into(),
            summary: summary.into(),
            source: source.into(),
            nominal,
            unresolved: None,
            payment: Payment::Mode("with_lodgment".into()),
        }
    }
    fn unresolved(mut self, reason: &str) -> Self {
        self.unresolved = Some(reason.into());
        self
    }
}

struct Ask<'a> {
    facts: &'a Value,
    missing: Vec<String>,
}
impl<'a> Ask<'a> {
    fn new(facts: &'a Value) -> Self {
        Self {
            facts,
            missing: Vec::new(),
        }
    }
    fn get(&mut self, name: &str) -> Option<bool> {
        let value = self.facts[name].as_bool();
        if value.is_none() && !self.missing.iter().any(|n| n == name) {
            self.missing.push(name.into());
        }
        value
    }
    fn yes(&mut self, name: &str) -> bool {
        self.get(name) == Some(true)
    }
}

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).expect("recorded date")
}
