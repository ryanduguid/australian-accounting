// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `residency_tests`: whether an individual is a resident of Australia for an
//! income year from 2020-21 to 2026-27 under each of the 4 tests in
//! paragraph (a) of 'resident or resident of Australia' in ITAA 1936
//! s 6(1): resides (ordinary concepts), domicile, 183 days and the
//! Commonwealth superannuation fund test. The definition reads the same in
//! compilations 169 (1 July 2020) and 193 (1 October 2026), read on
//! 22 November 2026, with TR 2023/1. Whether the person resides here, their
//! domicile and their places of abode and intention are the operator's
//! answers, never inferred; the worksheet counts the days in Australia from
//! the dates given. Treaty tie-breakers are refused.

use serde_json::{Map, Value, json};

use super::stated;
use super::{Arg, Result, flag, refuse, result, scope};
use crate::dates;

const KIND: &str = "residency_tests";

/// The most periods in Australia the worksheet takes.
const MAX_PERIODS: usize = 400;

/// The facts as the command passes them; see the README for each.
pub struct ResidencyTestsArgs<'a> {
    pub resides: &'a Arg,
    pub domicile_in_australia: &'a Arg,
    pub permanent_place_of_abode_outside: &'a Arg,
    pub presence: &'a Arg,
    pub usual_place_of_abode_outside: &'a Arg,
    pub intends_to_take_up_residence: &'a Arg,
    pub commonwealth_super: &'a Arg,
    pub resident_elsewhere: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// The income year's first and last days (1 July to 30 June).
fn year_bounds(year: &str) -> (i64, i64) {
    let start: i64 = year[..4].parse().expect("supported year");
    let day = |y: i64, md: &str| dates::parse(&format!("{y}-{md}")).expect("valid date");
    (day(start, "07-01"), day(start + 1, "06-30"))
}

fn bad_presence<T>(why: &str) -> Result<T> {
    refuse(
        "residency_tests.presence_invalid",
        format!(
            "presence must be a list of up to {MAX_PERIODS} periods, each an object with from and to as YYYY-MM-DD dates, to not before from: {why}"
        ),
    )
}

fn period_date(fields: &Map<String, Value>, item: &str, key: &str) -> Result<i64> {
    match stated::required(fields, item, key)? {
        Value::String(text) => match dates::parse(text) {
            Some(day) => Ok(day),
            None => bad_presence(&format!("{item}: {key} is not a date")),
        },
        _ => bad_presence(&format!("{item}: {key} is not a date")),
    }
}

/// The periods given, each `(from, to)` inclusive.
fn periods(given: &Arg) -> Result<Vec<(i64, i64)>> {
    let Arg::Json(Value::Array(items)) = given else {
        return bad_presence("not a list");
    };
    if items.len() > MAX_PERIODS {
        return bad_presence("too many periods");
    }
    let mut out = Vec::with_capacity(items.len());
    for (i, item) in items.iter().enumerate() {
        let name = format!("period {}", i + 1);
        let Value::Object(fields) = item else {
            return bad_presence(&format!("{name} is not an object"));
        };
        stated::known_keys(fields, &name, "a period", &["from", "to"])?;
        let (from, to) = (
            period_date(fields, &name, "from")?,
            period_date(fields, &name, "to")?,
        );
        if to < from {
            return bad_presence(&format!("{name} ends before it starts"));
        }
        out.push((from, to));
    }
    Ok(out)
}

/// TR 2023/1 paragraphs 86 and 87: the distinct days in the income year on
/// which the person was in Australia for any part of the day.
fn days_present(spans: &[(i64, i64)], first: i64, last: i64) -> i64 {
    let mut present = vec![false; (last - first + 1) as usize];
    for &(from, to) in spans {
        for day in from.max(first)..=to.min(last) {
            present[(day - first) as usize] = true;
        }
    }
    present.iter().filter(|&&p| p).count() as i64
}

/// s 6(1)(a)(iii): the sub-subparagraph the stated membership falls in.
fn super_link(given: &Arg) -> Result<Option<&'static str>> {
    match given.text() {
        Some("none") => Ok(None),
        Some("pss_member") => Ok(Some("(A), a member of the PSS")),
        Some("css_eligible_employee") => Ok(Some("(B), an eligible employee under the CSS")),
        Some("partner_of_member") => Ok(Some(
            "(C), the spouse of a PSS member or CSS eligible employee",
        )),
        Some("child_under_16_of_member") => Ok(Some(
            "(C), a child under 16 of a PSS member or CSS eligible employee",
        )),
        _ => refuse(
            "residency_tests.super_invalid",
            "commonwealth_super must be none, pss_member, css_eligible_employee, partner_of_member or child_under_16_of_member.",
        ),
    }
}

fn said(answer: bool) -> &'static str {
    if answer {
        "Yes, as you answered"
    } else {
        "No, as you answered"
    }
}

fn yes_no(answer: bool) -> &'static str {
    if answer { "yes" } else { "no" }
}

fn met(test: bool) -> &'static str {
    if test { "met" } else { "not met" }
}

fn step(label: &str, finding: impl Into<String>) -> Value {
    json!({"step": label, "finding": finding.into()})
}

/// The stated answers, read in the order the README lists them.
struct Answers {
    resides: bool,
    domicile: bool,
    permanent_outside: bool,
    usual_outside: bool,
    intends: bool,
    super_link: Option<&'static str>,
    elsewhere: bool,
}

fn answers(args: &ResidencyTestsArgs) -> Result<Answers> {
    Ok(Answers {
        resides: flag(args.resides, "resides")?,
        domicile: flag(args.domicile_in_australia, "domicile_in_australia")?,
        permanent_outside: flag(
            args.permanent_place_of_abode_outside,
            "permanent_place_of_abode_outside",
        )?,
        usual_outside: flag(
            args.usual_place_of_abode_outside,
            "usual_place_of_abode_outside",
        )?,
        intends: flag(
            args.intends_to_take_up_residence,
            "intends_to_take_up_residence",
        )?,
        super_link: super_link(args.commonwealth_super)?,
        elsewhere: flag(args.resident_elsewhere, "resident_elsewhere")?,
    })
}

/// The 4 tests, each with its steps.
struct Tests {
    steps: Vec<Value>,
    met: Vec<&'static str>,
    figures: Map<String, Value>,
}

impl Tests {
    fn record(&mut self, name: &'static str, passed: bool) {
        self.figures.insert(name.into(), json!(passed));
        if passed {
            self.met.push(name);
        }
    }

    fn ordinary(&mut self, a: &Answers) {
        self.steps.push(step(
            "Subsection 6(1), resident (a): resides in Australia according to ordinary concepts",
            format!("{}: {}", said(a.resides), met(a.resides)),
        ));
        self.record("ordinary_concepts", a.resides);
    }

    /// s 6(1)(a)(i): domicile in Australia unless the permanent place of
    /// abode is outside Australia.
    fn domicile(&mut self, a: &Answers) {
        self.steps.push(step(
            "Subparagraph 6(1)(a)(i): domicile in Australia",
            said(a.domicile),
        ));
        if a.domicile {
            self.steps.push(step(
                "Subparagraph 6(1)(a)(i): permanent place of abode outside Australia",
                said(a.permanent_outside),
            ));
        }
        let passed = a.domicile && !a.permanent_outside;
        self.steps.push(step("Domicile test", met(passed)));
        self.record("domicile", passed);
    }

    /// s 6(1)(a)(ii): more than one-half of the income year in Australia,
    /// unless both the usual place of abode is outside Australia and there
    /// is no intention to take up residence here.
    fn days(&mut self, a: &Answers, days: i64, year_days: i64) {
        let needed = year_days / 2 + 1;
        let enough = days >= needed;
        self.steps.push(step(
            "Subparagraph 6(1)(a)(ii): days in Australia in the income year, part days counted whole (TR 2023/1 paragraphs 86 and 87)",
            format!(
                "{days} of {year_days}; more than one-half is {needed} or more: {}",
                if enough { "reached" } else { "not reached" }
            ),
        ));
        let visitor = a.usual_outside && !a.intends;
        if enough {
            self.steps.push(step(
                "Subparagraph 6(1)(a)(ii): usual place of abode outside Australia and no intention to take up residence in Australia",
                format!(
                    "Usual place of abode outside Australia: {}; intends to take up residence: {}, as you answered: {}",
                    yes_no(a.usual_outside),
                    yes_no(a.intends),
                    if visitor { "the exception applies" } else { "the exception does not apply" }
                ),
            ));
        }
        let passed = enough && !visitor;
        self.steps.push(step("183-day test", met(passed)));
        self.figures.insert("days_in_australia".into(), json!(days));
        self.figures.insert("days_needed".into(), json!(needed));
        self.record("days_183", passed);
    }

    fn commonwealth_super(&mut self, a: &Answers) {
        let finding = match a.super_link {
            Some(which) => format!("Yes, as you answered: sub-subparagraph {which}: met"),
            None => "No, as you answered: not met".into(),
        };
        self.steps.push(step(
            "Subparagraph 6(1)(a)(iii): Commonwealth superannuation fund test",
            finding,
        ));
        self.record("commonwealth_super", a.super_link.is_some());
    }
}

pub fn residency_tests(args: &ResidencyTestsArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let a = answers(args)?;
    let spans = periods(args.presence)?;
    let (first, last) = year_bounds(&year);
    let year_days = last - first + 1;
    let days = days_present(&spans, first, last);

    let mut tests = Tests {
        steps: Vec::new(),
        met: Vec::new(),
        figures: Map::new(),
    };
    tests.ordinary(&a);
    tests.domicile(&a);
    tests.days(&a, days, year_days);
    tests.commonwealth_super(&a);
    let resident = !tests.met.is_empty();
    if resident && a.elsewhere {
        return refuse(
            "residency_tests.treaty_tie_breaker",
            "You are a resident under subsection 6(1) and also a resident of another country under its law: a tax treaty's tie-breaker rules may allocate your residence for the treaty's purposes (TR 2023/1 paragraph 108). This worksheet does not apply treaties.",
        );
    }

    let mut out = result(KIND, &year, Vec::new(), Vec::new())?;
    out["resident"] = json!(resident);
    out["met_by"] = json!(tests.met);
    out["income_year"] = json!({
        "first_day": dates::format(first),
        "last_day": dates::format(last),
        "days": year_days,
    });
    out["tests"] = Value::Object(tests.figures);
    out["steps"] = json!(tests.steps);
    Ok(out)
}
