// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `first_home_owner_grant`: the first home owner grant in one of the 8
//! states and territories for a new home bought or built under a contract
//! made from 1 July 2025 to 22 November 2026, as the site's grant page
//! (ryanduguid.github.io `assets/first-home-owner-grant.mjs`, 22 November
//! 2026) works it, with each First Home Owner Grant Act read on 23 November
//! 2026: Vic version 053 ss 13 and 18; Qld as at 22 October 2026 ss 5, 15 and
//! 20; WA as at 22 October 2026 ss 14AC, 14AD and 19; SA version 4.12.2025 ss 7 and
//! 18; Tas as at 9 October 2026 ss 18, 18L and 18M; NT as in force at 1 July
//! 2026 ss 13, 18 and 56; ACT republication 37 s 18. The NSW Act could not be
//! read that day, so NSW rests on Revenue NSW's page. Where an Act differs
//! from the page, the Act is followed: an NT established home bought from
//! 1 July to 30 September 2025, and a Victorian, Western Australian or NT home
//! bought for less than the grant. The eligibility facts are the operator's
//! answers: the worksheet does not test them.

use serde_json::{Value, json};

use super::stated;
use super::{Arg, Result, flag, int, money, refuse, result, scope, whole_dollars};
use crate::dates;
use crate::decimal::Decimal;

const KIND: &str = "first_home_owner_grant";
const FIRST: &str = "2025-07-01";
const LAST: &str = "2026-11-22";
const NEW_ONLY: &str = "the grant is for a new home only";
const LESSER: &str = "the grant cannot be more than the consideration";

/// The inputs taken only in some jurisdictions: Western Australia's
/// location, and the value Queensland and Western Australia test against
/// their caps where it is not the consideration.
pub const FHOG_OPTIONAL: [&str; 2] = ["north_of_26th", "cap_value"];

/// The 4 facts every grant needs, then the facts some transactions need.
const FACTS: [(&str, &str); 4] = [
    ("person", "every applicant is a person aged 18 or over"),
    (
        "citizen",
        "an applicant is an Australian citizen or permanent resident",
    ),
    (
        "first_home",
        "no applicant or spouse is excluded by the jurisdiction's prior grant and ownership rules",
    ),
    ("residence", "the residence requirement will be met"),
];
const CASE_FACTS: [(&str, &str); 3] = [
    (
        "building_contract",
        "the building contract price is at least the grant",
    ),
    (
        "nt_no_replacement",
        "the contract replaces none signed before 1 October 2024",
    ),
    (
        "tas_conditions",
        "the transaction completes within 24 months or a period the Commissioner extends, and replaces no transaction for the same property, between or benefiting substantially the same parties, entered into before 1 July 2025 (for a 2025-26 contract) or 1 July 2026 (for a 2026-27 contract)",
    ),
];

/// The facts as the command passes them; see the README for each.
pub struct FirstHomeOwnerGrantArgs<'a> {
    pub jurisdiction: &'a Arg,
    pub contract_date: &'a Arg,
    pub home: &'a Arg,
    pub total_value: &'a Arg,
    pub facts: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
    pub optional: [Option<&'a Arg>; 2],
}

/// One amount for contracts made from `from` until the next rule's `from`. `bought`,
/// `built` and `north` are the highest eligible whole-dollar value; `None`
/// is no cap. `established` is the amount for an established home.
struct Rule {
    from: &'static str,
    amount: i64,
    established: i64,
    bought: Option<i64>,
    built: Option<i64>,
    north: Option<i64>,
    authority: &'static str,
}

struct Scheme {
    code: &'static str,
    name: &'static str,
    residence: &'static str,
    /// Whether the source read gives the lesser of the consideration and
    /// the amount.
    lesser: bool,
    rules: &'static [Rule],
}

const fn rule(
    from: &'static str,
    amount: i64,
    (bought, built, north): (Option<i64>, Option<i64>, Option<i64>),
    authority: &'static str,
) -> Rule {
    Rule {
        from,
        amount,
        established: 0,
        bought,
        built,
        north,
        authority,
    }
}

static SCHEMES: [Scheme; 8] = [
    Scheme {
        code: "nsw",
        name: "New South Wales",
        residence: "each applicant occupies the home as their principal place of residence for at least 12 continuous months, starting within 12 months of the purchase or construction",
        lesser: false,
        rules: &[rule(
            "2016-01-01",
            10_000,
            (Some(600_000), Some(750_000), None),
            "Revenue NSW, First Home Owner (New Homes) Grant: $10,000; a price not over $600,000 for a home bought, and land, building contract and variations not over $750,000 for a home built",
        )],
    },
    Scheme {
        code: "vic",
        name: "Victoria",
        residence: "at least one applicant occupies the home as their principal place of residence for at least 12 months, starting within 12 months of settlement or of the build finishing",
        lesser: true,
        rules: &[rule(
            "2013-07-01",
            10_000,
            (Some(750_000), Some(750_000), None),
            "First Home Owner Grant and Home Buyer Schemes Act 2000 (Vic) s 18(1A): the lesser of the consideration and $10,000; s 13(1A): consideration not over $750,000",
        )],
    },
    Scheme {
        code: "qld",
        name: "Queensland",
        residence: "each applicant occupies the home as their principal place of residence for a continuous period of at least 6 months, starting within 1 year after completion of the transaction (s 15)",
        lesser: true,
        rules: &[rule(
            "2023-11-20",
            30_000,
            (Some(749_999), Some(749_999), None),
            "First Home Owner Grant and Other Home Owner Grants Act 2000 (Qld) s 20(1): the lesser of the consideration and $30,000; s 5(6) to (8): not eligible at $750,000 or more",
        )],
    },
    Scheme {
        code: "wa",
        name: "Western Australia",
        residence: "each applicant occupies the home as their principal place of residence for a continuous period of at least 6 months, starting within 12 months of completion of the transaction",
        lesser: true,
        rules: &[
            rule(
                "2000-07-01",
                10_000,
                (Some(750_000), Some(750_000), Some(1_000_000)),
                "First Home Owner Grant Act 2000 (WA) s 19(2CA): the lesser of $10,000 and the consideration; s 14AD: $750,000 south and $1,000,000 north of the 26th parallel",
            ),
            rule(
                "2026-05-07",
                10_000,
                (Some(800_000), Some(800_000), Some(1_000_000)),
                "First Home Owner Grant Act 2000 (WA) s 19(2CA): the lesser of $10,000 and the consideration; DTF fact sheet as at 28 July 2026: $800,000 south of the 26th parallel from 7 May 2026, $1,000,000 north (s 14AD)",
            ),
        ],
    },
    Scheme {
        code: "sa",
        name: "South Australia",
        residence: "all applicants occupy the home as their principal place of residence for a continuous period of at least 6 months, starting within 12 months after completion of the transaction",
        lesser: true,
        rules: &[rule(
            "2024-06-06",
            15_000,
            (None, None, None),
            "First Home and Housing Construction Grants Act 2000 (SA) s 18(1): the lesser of the consideration and $15,000; s 7(1a): no value cap from 6 June 2024",
        )],
    },
    Scheme {
        code: "tas",
        name: "Tasmania",
        residence: "each applicant occupies the home as their principal place of residence for a continuous period of at least 6 months, starting within 12 months of completion of the transaction",
        lesser: true,
        rules: &[
            rule(
                "2025-07-01",
                30_000,
                (None, None, None),
                "First Home Owner Grant Act 2000 (Tas) s 18(1) and (2)(al): the lesser of the consideration and $30,000 for a transaction satisfying s 18L; s 7 sets no value cap",
            ),
            rule(
                "2026-07-01",
                20_000,
                (None, None, None),
                "First Home Owner Grant Act 2000 (Tas) s 18(1) and (2)(am): the lesser of the consideration and $20,000 for a transaction satisfying s 18M; s 7 sets no value cap",
            ),
        ],
    },
    Scheme {
        code: "act",
        name: "Australian Capital Territory",
        residence: "none, as no grant is paid",
        lesser: true,
        rules: &[rule(
            "2019-07-01",
            0,
            (None, None, None),
            "First Home Owner Grant Act 2000 (ACT) s 18: the amount the Minister determines; ACT Revenue Office: no grant for a commencement date from 1 July 2019",
        )],
    },
    Scheme {
        code: "nt",
        name: "Northern Territory",
        residence: "each applicant occupies the home as their principal place of residence for a continuous period of at least 12 months, starting within 12 months after completion of the transaction (s 12(2))",
        lesser: true,
        rules: &[
            Rule {
                established: 10_000,
                ..rule(
                    "2024-10-01",
                    50_000,
                    (None, None, None),
                    "First Home Owner Grant Act 2000 (NT) s 18(1) and (3D): the lesser of the consideration and $50,000 for a new home or $10,000 for an established one (s 13(1)(a)(iii)); no value cap (s 13AA)",
                )
            },
            rule(
                "2025-10-01",
                50_000,
                (None, None, None),
                "First Home Owner Grant Act 2000 (NT) s 18(1) and (3E): the lesser of the consideration and $50,000, the HomeGrown Territory Grant, for a new home (s 13(1)(a)(iv)); no value cap (s 13AA)",
            ),
        ],
    },
];

/// `n` dollars with thousands separators.
fn whole(n: i64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    format!("${out}")
}

/// 1 July 2025 as the page writes it.
fn long_date(iso: &str) -> String {
    const MONTHS: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let (y, m, d) = dates::to_civil(dates::parse(iso).unwrap_or_default());
    format!("{d} {} {y}", MONTHS[usize::try_from(m - 1).unwrap_or(0)])
}

fn scheme(given: &Arg) -> Result<&'static Scheme> {
    let found = given
        .text()
        .and_then(|code| SCHEMES.iter().find(|s| s.code == code));
    match found {
        Some(s) => Ok(s),
        None => refuse(
            "first_home_owner_grant.jurisdiction_invalid",
            "jurisdiction must be nsw, vic, qld, wa, sa, tas, act or nt.",
        ),
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Home {
    Bought,
    Built,
    Established,
}

fn home(given: &Arg) -> Result<Home> {
    match given.text() {
        Some("bought") => Ok(Home::Bought),
        Some("built") => Ok(Home::Built),
        Some("established") => Ok(Home::Established),
        _ => refuse(
            "first_home_owner_grant.home_invalid",
            "home must be bought, built or established.",
        ),
    }
}

/// The contract date, within `year` and the dates the rules were read for.
fn contract_date(given: &Arg, year: &str) -> Result<String> {
    let start: i64 = year[..4].parse().unwrap_or_default();
    let within = |date: &str| {
        date >= format!("{start}-07-01").as_str() && date <= format!("{}-06-30", start + 1).as_str()
    };
    match given.text() {
        Some(date) if dates::parse(date).is_some() && within(date) => {
            if (FIRST..=LAST).contains(&date) {
                Ok(date.to_owned())
            } else {
                refuse(
                    "first_home_owner_grant.date_uncovered",
                    format!(
                        "Not computed: choose a contract date from {} to {}, the day the grant rules were read.",
                        long_date(FIRST),
                        long_date(LAST)
                    ),
                )
            }
        }
        _ => refuse(
            "first_home_owner_grant.date_invalid",
            format!("contract_date must be a YYYY-MM-DD date in {year}."),
        ),
    }
}

/// The facts stated true; a fact left out is not stated.
fn facts(given: &Arg) -> Result<Vec<&'static str>> {
    let Arg::Json(Value::Object(fields)) = given else {
        return refuse(
            "first_home_owner_grant.facts_invalid",
            "facts must be an object of booleans.",
        );
    };
    let keys: Vec<&str> = FACTS.iter().chain(&CASE_FACTS).map(|(k, _)| *k).collect();
    stated::known_keys(fields, "facts", "facts", &keys)?;
    let mut stated = Vec::new();
    for key in FACTS.iter().chain(&CASE_FACTS).map(|(k, _)| *k) {
        match fields.get(key) {
            None | Some(Value::Bool(false)) => {}
            Some(Value::Bool(true)) => stated.push(key),
            Some(_) => {
                return refuse(
                    "input.flag_invalid",
                    format!("facts: {key} must be a boolean."),
                );
            }
        }
    }
    let missing: Vec<&str> = FACTS
        .iter()
        .filter(|(k, _)| !stated.contains(k))
        .map(|(_, text)| *text)
        .collect();
    if !missing.is_empty() {
        return refuse(
            "first_home_owner_grant.facts_not_stated",
            format!(
                "Not computed: the grant needs every eligibility fact stated; not stated: {}.",
                missing.join("; ")
            ),
        );
    }
    Ok(stated)
}

fn value(given: &Arg, name: &str) -> Result<Decimal> {
    let dollars = money(given)?;
    whole_dollars(&dollars, name)?;
    if dollars.lt(&int(1))? || dollars.gt(&int(1_000_000_000))? {
        return refuse(
            "first_home_owner_grant.value_invalid",
            format!("{name} must be whole dollars from 1 to 1,000,000,000."),
        );
    }
    Ok(dollars)
}

/// The value Queensland (s 5(7) and (8)) and Western Australia (s 14AC)
/// test against the cap where it differs from the consideration: taken
/// there and refused elsewhere.
fn cap_value(s: &Scheme, given: Option<&Arg>) -> Result<Option<Decimal>> {
    match given {
        Some(given) if matches!(s.code, "qld" | "wa") => value(given, "cap_value").map(Some),
        Some(_) => refuse(
            "input.field_unknown",
            "cap_value is an input only for Queensland and Western Australia",
        ),
        None => Ok(None),
    }
}

/// Western Australia's location: required there and refused elsewhere.
fn north(s: &Scheme, given: Option<&Arg>) -> Result<Option<bool>> {
    match (s.code == "wa", given) {
        (true, Some(given)) => flag(given, "north_of_26th").map(Some),
        (true, None) => refuse(
            "first_home_owner_grant.location_needed",
            "Not computed: choose where the Western Australian home is, south or north of the 26th parallel (north_of_26th).",
        ),
        (false, Some(_)) => refuse(
            "input.field_unknown",
            "north_of_26th is an input only for Western Australia",
        ),
        (false, None) => Ok(None),
    }
}

/// The facts the transaction needs beyond the 4.
fn case_facts(s: &Scheme, house: Home, stated: &[&str]) -> Result<()> {
    let unstated: Vec<&str> = CASE_FACTS
        .iter()
        .filter(|(key, _)| {
            !stated.contains(key)
                && match *key {
                    "building_contract" => house == Home::Built,
                    "nt_no_replacement" => s.code == "nt",
                    _ => s.code == "tas",
                }
        })
        .map(|(_, text)| *text)
        .collect();
    if unstated.is_empty() {
        return Ok(());
    }
    refuse(
        "first_home_owner_grant.fact_needed",
        format!(
            "Not computed: the {} grant for this home needs another fact stated: {}.",
            s.name,
            unstated.join("; ")
        ),
    )
}

/// The grant, its cap and the reason for anything less than the amount.
struct Grant {
    grant: Decimal,
    cap: String,
    reason: String,
}

fn grant(grant: Decimal, cap: &str, reason: &str) -> Grant {
    Grant {
        grant,
        cap: cap.to_owned(),
        reason: reason.to_owned(),
    }
}

pub fn first_home_owner_grant(args: &FirstHomeOwnerGrantArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let s = scheme(args.jurisdiction)?;
    let house = home(args.home)?;
    let date = contract_date(args.contract_date, &year)?;
    let stated = facts(args.facts)?;
    let dollars = value(args.total_value, "total_value")?;
    let north = north(s, args.optional[0])?;
    let tested = tested(house, &dollars, cap_value(s, args.optional[1])?)?;
    // Every scheme has a rule from 1 July 2025 or earlier.
    let r = s
        .rules
        .iter()
        .rfind(|r| r.from <= date.as_str())
        .unwrap_or(&s.rules[0]);
    let worked = work(s, r, house, (&dollars, &tested), north, &stated)?;
    let mut out = result(KIND, &year, vec![("grant", worked.grant)], Vec::new())?;
    out["jurisdiction"] = json!(s.name);
    out["contract_date"] = json!(date);
    out["cap"] = json!(worked.cap);
    out["reason"] = json!(worked.reason);
    out["residence_requirement"] = json!(s.residence);
    out["rule"] = json!(r.authority);
    Ok(out)
}

/// The value tested against the cap: for a home bought, the greater of the
/// consideration and `cap_value` (Qld s 5(6) and (7), WA s 14AC(2)); for a
/// home built, `cap_value` in place of the land and contract total (Qld
/// s 5(8), WA s 14AC(3)); otherwise `total_value`.
fn tested(house: Home, dollars: &Decimal, cap: Option<Decimal>) -> Result<Decimal> {
    Ok(match cap {
        Some(cap) if house == Home::Built || cap.gt(dollars)? => cap,
        _ => dollars.clone(),
    })
}

/// The pair is the consideration, which limits the grant, and the value
/// tested against the cap.
fn work(
    s: &Scheme,
    r: &Rule,
    house: Home,
    (dollars, tested): (&Decimal, &Decimal),
    north: Option<bool>,
    stated: &[&str],
) -> Result<Grant> {
    if r.amount == 0 {
        let why = format!("the {} has paid none since {}", s.name, long_date(r.from));
        return Ok(grant(int(0), "none", &why));
    }
    let amount = if house == Home::Established {
        r.established
    } else {
        r.amount
    };
    if amount == 0 {
        return Ok(grant(int(0), "none", NEW_ONLY));
    }
    case_facts(s, house, stated)?;
    let limit = match (north, house) {
        (Some(true), _) => r.north,
        (_, Home::Built) => r.built,
        _ => r.bought,
    };
    let cap = match limit {
        None => "none".to_owned(),
        Some(_) if s.code == "qld" => "under $750,000".to_owned(),
        Some(limit) => format!("up to {}", whole(limit)),
    };
    if let Some(limit) = limit
        && tested.gt(&int(limit))?
    {
        return over_cap(s, house, &cap);
    }
    below_amount(s, house, dollars, amount, &cap)
}

fn over_cap(s: &Scheme, house: Home, cap: &str) -> Result<Grant> {
    if s.code == "vic" && house == Home::Built {
        return refuse(
            "first_home_owner_grant.value_basis",
            "Not computed: the Victorian cap applies to the consideration for the building work (First Home Owner Grant and Home Buyer Schemes Act 2000 (Vic) s 13(1A) and (7)(b)), not the land and building total entered.",
        );
    }
    let why = if s.code == "qld" {
        "the value is $750,000 or more".to_owned()
    } else {
        format!("the value is over the {} cap", s.name)
    };
    Ok(grant(int(0), cap, &why))
}

fn below_amount(
    s: &Scheme,
    house: Home,
    dollars: &Decimal,
    amount: i64,
    cap: &str,
) -> Result<Grant> {
    if !dollars.lt(&int(amount))? {
        return Ok(grant(int(amount), cap, ""));
    }
    if house == Home::Built {
        return refuse(
            "first_home_owner_grant.below_grant",
            format!(
                "Not computed: a total value below the {} grant cannot hold a building contract price of at least the grant.",
                whole(amount)
            ),
        );
    }
    if !s.lesser {
        return refuse(
            "first_home_owner_grant.below_grant",
            format!(
                "Not computed: a value below the {} grant; the {} rule for that case was not confirmed.",
                whole(amount),
                s.name
            ),
        );
    }
    Ok(grant(dollars.clone(), cap, LESSER))
}
