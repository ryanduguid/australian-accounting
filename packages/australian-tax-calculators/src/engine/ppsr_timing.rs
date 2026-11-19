// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `ppsr_timing`: what the Personal Property Securities Register's law decides
//! without a search. A registration's latest end time and its fee (PPSA s 153
//! item 5 and subss (2) and (3); Personal Property Securities (Fees)
//! Determination 2018 ss 4 and 5), a purchase money security interest's
//! registration deadline (PPSA s 62(2) and (3)), and the time by which a
//! company grantor's security interest must be registered to escape vesting
//! (Corporations Act 2001 s 588FL). Business days are counted under PPSA s 10
//! and Corporations Act s 9 over the public holidays the caller states. Times
//! are ACT legal time (PPSA s 294). Read on 20 November 2026: the PPSA,
//! compilation 22 (14 October 2024); the PPS Regulations 2010, compilation 6
//! (1 July 2022), which prescribe no day for the s 10 definition; the Fees
//! Determination 2018 as made (in force from 1 August 2018, unamended); the
//! Corporations Act, compilation 149 (19 September 2026).

use std::collections::BTreeSet;

use serde_json::{Value, json};

use super::{Arg, Result, d, flag, rate, refuse, result, scope};
use crate::dates;

const KIND: &str = "ppsr_timing";
const DAY: i64 = 86_400;
const QUESTIONS: [&str; 3] = ["registration", "pmsi", "company_grantor"];

/// The first day whose s 588FL this command has not read: the Corporations
/// Act's compilation 149 is in force until 30 November 2026, and later
/// compilations were unregistered on 20 November 2026.
const CORPORATIONS_ACT_READ_TO: &str = "2026-12-01";

/// An instant in seconds from 1970-01-01 00:00, ACT legal time.
type Instant = i64;

fn end_of_day(day: i64) -> Instant {
    (day + 1) * DAY - 1
}

fn show(at: Instant) -> String {
    let day = at.div_euclid(DAY);
    if at == end_of_day(day) {
        return format!("{} end of day", dates::format(day));
    }
    let minutes = at.rem_euclid(DAY) / 60;
    format!(
        "{}T{:02}:{:02}",
        dates::format(day),
        minutes / 60,
        minutes % 60
    )
}

/// A strict `YYYY-MM-DDTHH:MM` time, ACT legal time.
fn time(arg: &Arg, key: &str) -> Result<Instant> {
    let parsed = arg.text().and_then(|t| {
        let (day, clock) = t.split_once('T')?;
        let (h, m) = clock.split_once(':')?;
        let digits = |s: &str| s.len() == 2 && s.bytes().all(|b| b.is_ascii_digit());
        let (h, m): (i64, i64) = (
            digits(h).then(|| h.parse().ok())??,
            digits(m).then(|| m.parse().ok())??,
        );
        (h < 24 && m < 60).then_some(dates::parse(day)? * DAY + h * 3600 + m * 60)
    });
    match parsed {
        Some(at) => Ok(at),
        None => refuse(
            "ppsr_timing.time_invalid",
            format!("{key} must be a time YYYY-MM-DDTHH:MM, ACT legal time (PPSA s 294)."),
        ),
    }
}

/// A list of `YYYY-MM-DD` days.
fn days(arg: &Arg, key: &str) -> Result<BTreeSet<i64>> {
    let bad = || {
        refuse(
            "ppsr_timing.days_invalid",
            format!("{key} must be a list of dates YYYY-MM-DD."),
        )
    };
    let Arg::Json(Value::Array(items)) = arg else {
        return bad();
    };
    let mut out = BTreeSet::new();
    for item in items {
        match item.as_str().and_then(dates::parse) {
            Some(day) => out.insert(day),
            None => return bad(),
        };
    }
    Ok(out)
}

/// Monday is 0: 1970-01-01 was a Thursday.
fn weekend(day: i64) -> bool {
    (day + 3).rem_euclid(7) >= 5
}

/// The inputs given beside the positional ones, by name.
struct Given<'a> {
    question: &'a str,
    named: &'a [(String, Arg)],
}

impl<'a> Given<'a> {
    fn get(&self, key: &str) -> Option<&'a Arg> {
        self.named.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    fn need(&self, key: &str) -> Result<&'a Arg> {
        match self.get(key) {
            Some(arg) => Ok(arg),
            None => refuse(
                "input.missing_field",
                format!("{key} is required for question {}.", self.question),
            ),
        }
    }

    /// Refuse a field `question` does not take.
    fn only(&self, takes: &[&str]) -> Result<()> {
        match self
            .named
            .iter()
            .find(|(k, _)| !takes.contains(&k.as_str()))
        {
            Some((key, _)) => refuse(
                "input.field_unknown",
                format!("{key} is not an input of question {}.", self.question),
            ),
            None => Ok(()),
        }
    }
}

/// The `n`th business day after `from`, where `excluded` decides the days
/// that are not business days. Before counting, every 25 December and
/// 1 January in the window that falls on a weekday must be one `listed` holds
/// or one `excluded` already excludes, so a holiday list that forgot them is
/// refused rather than counted.
fn business_day_after(
    from: i64,
    n: usize,
    listed: &BTreeSet<i64>,
    key: &str,
    excluded: impl Fn(i64) -> bool,
) -> Result<i64> {
    let mut day = from;
    let mut counted = 0;
    while counted < n {
        day += 1;
        let (_, month, date) = dates::to_civil(day);
        let named = (month, date) == (12, 25) || (month, date) == (1, 1);
        if named && !weekend(day) && !listed.contains(&day) {
            return refuse(
                "ppsr_timing.holidays_incomplete",
                format!(
                    "{} falls in the count on a weekday and is not in {key}: list every public holiday the count spans.",
                    dates::format(day)
                ),
            );
        }
        if !excluded(day) {
            counted += 1;
        }
    }
    Ok(day)
}

/// PPSA s 10 'business day': not a Saturday or Sunday, not a public holiday
/// for the whole of any State, the ACT or the NT, not a day between Christmas
/// Day and New Year's Day, and not a day the Registrar suspended the register
/// (the caller lists the holidays and suspensions together); the regulations
/// prescribe no other day.
fn ppsa_business_day_after(from: i64, n: usize, listed: &BTreeSet<i64>) -> Result<i64> {
    business_day_after(from, n, listed, "excluded_days", |day| {
        let (_, month, date) = dates::to_civil(day);
        weekend(day) || listed.contains(&day) || (month == 12 && date > 25)
    })
}

/// Corporations Act s 9 'business day': not a Saturday, Sunday, or public or
/// bank holiday in the place concerned.
fn corporations_business_day_after(from: i64, n: usize, listed: &BTreeSet<i64>) -> Result<i64> {
    business_day_after(from, n, listed, "place_holidays", |day| {
        weekend(day) || listed.contains(&day)
    })
}

/// The day `years` after `day`, refused when that date does not exist.
fn years_after(day: i64, years: i64) -> Result<i64> {
    let (y, m, dd) = dates::to_civil(day);
    match dates::parse(&format!("{:04}-{m:02}-{dd:02}", y + years)) {
        Some(later) => Ok(later),
        None => refuse(
            "ppsr_timing.date_unsettled",
            format!(
                "{} has no anniversary {years} years on; the Act gives no rule for it.",
                dates::format(day)
            ),
        ),
    }
}

/// The same time 6 months before `at`, refused when that day does not exist.
fn six_months_before(at: Instant) -> Result<Instant> {
    let day = at.div_euclid(DAY);
    let earlier = dates::add_months(day, -6);
    if dates::add_months(earlier, 6) != day || dates::to_civil(earlier).2 != dates::to_civil(day).2
    {
        return refuse(
            "ppsr_timing.date_unsettled",
            format!(
                "{} has no same day 6 months earlier; this command does not settle the month rule for it.",
                dates::format(day)
            ),
        );
    }
    Ok(earlier * DAY + at.rem_euclid(DAY))
}

/// The income year a day falls in, as `YYYY-YY`.
fn income_year(day: i64) -> String {
    let (y, m, _) = dates::to_civil(day);
    let start = if m >= 7 { y } else { y - 1 };
    format!("{start}-{:02}", (start + 1) % 100)
}

fn in_year(at: Instant, period: &str, key: &str) -> Result<()> {
    let year = income_year(at.div_euclid(DAY));
    if year == period {
        return Ok(());
    }
    refuse(
        "ppsr_timing.date_outside_year",
        format!("{key} falls in {year}, not the year given, {period}."),
    )
}

const FEES: &str = "Personal Property Securities (Fees) Determination 2018 s 4 table items 1 to 3 (registering a financing statement under PPSA s 150(1)): $115.00 with no end time, $6.00 with an end time up to the end of the day 7 years after the registration time, $25.00 with an end time after that and up to the end of the day 25 years after; not subject to GST (the Registrar's fees page)";
/// Whether the collateral takes item 5(b)'s 7 years: consumer property, or
/// property described by a serial number.
fn short_collateral(given: &Given) -> Result<bool> {
    let consumer = match given.need("collateral")?.text() {
        Some("consumer") => true,
        Some("commercial") => false,
        _ => {
            return refuse(
                "ppsr_timing.fact_invalid",
                "collateral must be consumer or commercial (PPSA s 153 item 4(a)).",
            );
        }
    };
    Ok(consumer || flag(given.need("serial_numbered")?, "serial_numbered")?)
}

/// The end of the requested end date, if one is given.
fn requested_end(given: &Given) -> Result<Option<Instant>> {
    match given.get("end_date") {
        None => Ok(None),
        Some(arg) => match arg.text().and_then(dates::parse) {
            Some(day) => Ok(Some(end_of_day(day))),
            None => refuse("input.date_invalid", "end_date must be YYYY-MM-DD."),
        },
    }
}

const SHORT_END: &str = "PPSA s 153 item 5(b)(i): consumer property, or property described by a serial number, has an end time no later than the end of the day 7 years after the registration time";
const LONG_END: &str = "PPSA s 153 item 5(a): other collateral has no stated end time, or one no later than the end of the day 25 years after the registration time";
const DEFAULT_NOTE: &str = "PPSA s 153(2) and (3): an end time later than the default time, earlier than the registration time, or missing where item 5(b) requires one does not comply, and the financing statement is taken to provide for the default time as its end time.";

fn registration(given: &Given, period: &str) -> Result<Value> {
    given.only(&[
        "registration_time",
        "collateral",
        "serial_numbered",
        "end_date",
    ])?;
    let at = time(given.need("registration_time")?, "registration_time")?;
    in_year(at, period, "registration_time")?;
    let short = short_collateral(given)?;
    let requested = requested_end(given)?;
    let day = at.div_euclid(DAY);
    let seven = end_of_day(years_after(day, 7)?);
    let (latest, item) = if short {
        (seven, SHORT_END)
    } else {
        (end_of_day(years_after(day, 25)?), LONG_END)
    };
    // s 153(2) and (3): an end time that does not comply, as one after the
    // default time or before the registration time does not, is taken to be
    // the default time; so is a missing end time where item 5(b) requires one.
    let (end, applied) = match requested {
        Some(end) if end >= at && end <= latest => (Some(end), "requested"),
        Some(_) => (Some(latest), "default_time"),
        None if short => (Some(latest), "default_time"),
        None => (None, "no_end_time"),
    };
    let fee = match end {
        None => "115.00",
        Some(end) if end <= seven => "6.00",
        Some(_) => "25.00",
    };
    let amounts = vec![("registration_fee", d(fee))];
    let mut out = result(KIND, period, amounts, fee_rates())?;
    out["question"] = json!("registration");
    out["registration_time"] = json!(show(at));
    out["latest_end_time"] = json!(show(latest));
    out["end_time"] = json!(end.map(show));
    out["end_time_basis"] = json!(applied);
    out["end_time_rule"] = json!(item);
    if applied == "default_time" {
        out["end_time_note"] = json!(DEFAULT_NOTE);
    }
    out["fee_rule"] = json!(FEES);
    Ok(out)
}

/// The s 62 deadline, its rule, and whether a registration at the deadline
/// itself is in time (inventory: at the time; other property: before the end).
fn pmsi_deadline(
    given: &Given,
    inventory: bool,
    trigger: Instant,
) -> Result<(Instant, String, bool)> {
    let event = if flag(given.need("goods")?, "goods")? {
        "the grantor, or another person at its request, obtains possession"
    } else {
        "the interest attaches"
    };
    if inventory {
        let rule = format!(
            "PPSA s 62(2)(b): inventory; perfected by registration at the time {event}, so registered no later than that time"
        );
        return Ok((trigger, rule, true));
    }
    let listed = days(given.need("excluded_days")?, "excluded_days")?;
    let day = ppsa_business_day_after(trigger.div_euclid(DAY), 15, &listed)?;
    let rule = format!(
        "PPSA s 62(3)(b): not inventory; perfected by registration before the end of 15 business days (s 10) after the day {event}"
    );
    Ok((end_of_day(day), rule, false))
}

const SUPER_PRIORITY: &str = "PPSA s 62(2)(c) and (3)(c): the registration must also state, under s 153 item 7, that the interest is a purchase money security interest; priority over a non-purchase money interest of the same grantor in the same collateral then follows, subject to s 57 and, for inventory, ss 64 and 71";

fn pmsi(given: &Given, period: &str) -> Result<Value> {
    let inventory = flag(given.need("inventory")?, "inventory")?;
    let mut takes = vec!["inventory", "goods", "trigger_time", "registration_time"];
    if !inventory {
        takes.push("excluded_days");
    }
    if given.get("registration_time").is_some() {
        takes.push("pmsi_stated");
    }
    given.only(&takes)?;
    let trigger = time(given.need("trigger_time")?, "trigger_time")?;
    in_year(trigger, period, "trigger_time")?;
    let (deadline, rule, inclusive) = pmsi_deadline(given, inventory, trigger)?;
    let mut out = result(KIND, period, vec![], fee_rates())?;
    out["question"] = json!("pmsi");
    out["trigger_time"] = json!(show(trigger));
    out["register_by"] = json!(show(deadline));
    out["rule"] = json!(rule);
    if let Some(arg) = given.get("registration_time") {
        let at = time(arg, "registration_time")?;
        let stated = flag(given.need("pmsi_stated")?, "pmsi_stated")?;
        let in_time = if inclusive {
            at <= deadline
        } else {
            at < deadline
        };
        out["registration_time"] = json!(show(at));
        out["registered_in_time"] = json!(in_time);
        out["pmsi_stated"] = json!(stated);
        out["super_priority"] = json!(in_time && stated);
        out["super_priority_rule"] = json!(SUPER_PRIORITY);
    }
    Ok(out)
}

fn optional_time(given: &Given, key: &str) -> Result<Option<Instant>> {
    given.get(key).map(|arg| time(arg, key)).transpose()
}

/// The s 588FL(2)(b) times: 6 months before the critical time, and the
/// latest of it, the 20 day time capped at the critical time, and a court's.
fn latest_time(
    critical: Instant,
    court: Option<Instant>,
    safe_by: Instant,
) -> Result<(Instant, Instant)> {
    let unread = dates::parse(CORPORATIONS_ACT_READ_TO).expect("date") * DAY;
    if critical >= unread {
        return refuse(
            "ppsr_timing.law_unread",
            "critical_time is on or after 1 December 2026, when a later compilation of the Corporations Act starts, which this command has not read.",
        );
    }
    let months = six_months_before(critical)?;
    let latest = [Some(months), Some(safe_by.min(critical)), court]
        .into_iter()
        .flatten()
        .max()
        .expect("a time");
    Ok((months, latest))
}

const NOT_PERFECTED: &str = "not_perfected_at_critical_time: registered after the critical time, so not perfected by registration then (s 588FL(2)(a)(ii)) unless the interest arose after it; s 588FL's note points to PPSA ss 267 and 267A for an interest unperfected at the critical time";
const VESTS: &str = "vests: the registration time is after the latest time, so the interest vests in the company (s 588FL(4)) if, at the critical time, it was enforceable against third parties and perfected by registration only (s 588FL(2)(a)), unless s 588FN applies";
const DOES_NOT_VEST: &str =
    "does_not_vest: the registration time is not after the latest time (s 588FL(2)(b))";
const TWENTY_DAYS: &str = "Corporations Act s 588FL(2)(b)(ii): the end of 20 business days (s 9, in the place concerned) after the security agreement came into force; a registration no later than that time is never after the latest of the s 588FL(2)(b) times";
const LATEST_RULE: &str = "Corporations Act s 588FL(2)(b): the latest of 6 months before the critical time, the end of 20 business days after the agreement came into force or the critical time if earlier, and a later time the Court orders under s 588FL(2)(b)(iv) and s 588FM";

/// Whether a registration at `at` vests, and why.
fn vesting(at: Instant, critical: Instant, latest: Instant) -> (Value, &'static str) {
    if at > critical {
        (Value::Null, NOT_PERFECTED)
    } else if at > latest {
        (json!(true), VESTS)
    } else {
        (json!(false), DOES_NOT_VEST)
    }
}

/// The agreement's in-force time, refusing a foreign-law agreement.
fn agreement_time(given: &Given, period: &str) -> Result<Instant> {
    let key = "agreement_in_force_time";
    let agreed = time(given.need(key)?, key)?;
    in_year(agreed, period, key)?;
    let key = "foreign_law_agreement";
    if flag(given.need(key)?, key)? {
        return refuse(
            "ppsr_timing.foreign_law_unsupported",
            "A security agreement that came into force under a foreign law takes s 588FL(2)(b)(iii)'s 56 day time, which turns on when the interest first became enforceable in Australia; this command does not compute it.",
        );
    }
    Ok(agreed)
}

fn company_grantor(given: &Given, period: &str) -> Result<Value> {
    given.only(&[
        "agreement_in_force_time",
        "place_holidays",
        "foreign_law_agreement",
        "registration_time",
        "critical_time",
        "court_ordered_time",
    ])?;
    let agreed = agreement_time(given, period)?;
    let listed = days(given.need("place_holidays")?, "place_holidays")?;
    let day = corporations_business_day_after(agreed.div_euclid(DAY), 20, &listed)?;
    let safe_by = end_of_day(day);
    let [critical, court, registered] =
        ["critical_time", "court_ordered_time", "registration_time"]
            .map(|key| optional_time(given, key));
    let [critical, court, registered] = [critical?, court?, registered?];
    let mut out = result(KIND, period, vec![], fee_rates())?;
    out["question"] = json!("company_grantor");
    out["agreement_in_force_time"] = json!(show(agreed));
    out["register_by"] = json!(show(safe_by));
    out["rule"] = json!(TWENTY_DAYS);
    if let Some(at) = registered {
        out["registration_time"] = json!(show(at));
    }
    let Some(critical) = critical else {
        if court.is_some() {
            return refuse(
                "input.field_unknown",
                "court_ordered_time is an input only with critical_time.",
            );
        }
        return Ok(out);
    };
    let (months, latest) = latest_time(critical, court, safe_by)?;
    out["critical_time"] = json!(show(critical));
    out["six_months_before"] = json!(show(months));
    out["latest_time"] = json!(show(latest));
    out["latest_time_rule"] = json!(LATEST_RULE);
    if let Some(at) = registered {
        let (vests, why) = vesting(at, critical, latest);
        out["vests"] = vests;
        out["vesting"] = json!(why.split(':').next().expect("a verdict"));
        out["vesting_reason"] = json!(why);
    }
    Ok(out)
}

/// The published fee schedule, so every answer carries it.
fn fee_rates() -> Vec<(String, String)> {
    vec![
        rate("registration_no_end_time", "115.00"),
        rate("registration_up_to_7_years", "6.00"),
        rate("registration_over_7_to_25_years", "25.00"),
        rate("search_online", "2.00"),
        rate("search_contact_centre", "7.00"),
        rate("maintenance_of_registration", "4.00"),
        rate("registrar_report_per_hour", "300.00"),
    ]
}

pub fn ppsr_timing(
    question: &Arg,
    year: &Arg,
    scope_confirmed: &Arg,
    named: &[(String, Arg)],
) -> Result<Value> {
    let period = scope(scope_confirmed, year, KIND)?;
    let question = match question.text() {
        Some(q) if QUESTIONS.contains(&q) => q,
        _ => {
            return refuse(
                "ppsr_timing.fact_invalid",
                format!("question must be one of {}.", QUESTIONS.join(", ")),
            );
        }
    };
    let given = Given { question, named };
    match question {
        "registration" => registration(&given, &period),
        "pmsi" => pmsi(&given, &period),
        _ => company_grantor(&given, &period),
    }
}
