// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `home_guarantee`: whether a loan meets the price cap, loan-to-value and
//! borrower tests for a First Home Guarantee or Family Home Guarantee under
//! the Housing Australia Investment Mandate Direction 2018 Part 5, as
//! compiled on 7 February 2026 (compilation 20, the text from 1 October 2025)
//! and 18 July 2026 (compilation 22, the text from 1 July 2026), read on
//! 26 November 2026: ss 4A, 29C, 29D and 29F, with the application rules in
//! ss 44 and 46 (a guarantee is issued on the day Housing Australia approves
//! it). The capital city and regional centre the property is in, and the
//! borrower's eligibility, are stated facts.

use serde_json::{Value, json};

use super::help_to_buy::Test;
use super::stated;
use super::{Arg, Result, int, money, refuse, result, scope};
use crate::dates;
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "home_guarantee";
const FIRST: &str = "2025-10-01";
const SPLIT: &str = "2026-07-01";
const LAST: &str = "2026-11-26";

/// Section 29F(1) price caps: (code, area, cap from 1 October 2025, cap from
/// 1 July 2026), 0 where the area is not an item on that day.
const CAPS: [(&str, &str, i64, i64); 18] = [
    (
        "nsw_capital_regional",
        "New South Wales—capital city and regional centre",
        1_500_000,
        1_500_000,
    ),
    ("nsw_other", "New South Wales—other", 800_000, 800_000),
    (
        "vic_capital_regional",
        "Victoria—capital city and regional centre",
        950_000,
        950_000,
    ),
    ("vic_other", "Victoria—other", 650_000, 650_000),
    (
        "qld_capital_regional",
        "Queensland—capital city and regional centre",
        1_000_000,
        1_000_000,
    ),
    ("qld_other", "Queensland—other", 700_000, 700_000),
    (
        "wa_capital",
        "Western Australia—capital city",
        850_000,
        850_000,
    ),
    ("wa_other", "Western Australia—other", 600_000, 600_000),
    (
        "sa_capital",
        "South Australia—capital city",
        900_000,
        900_000,
    ),
    ("sa_other", "South Australia—other", 500_000, 500_000),
    ("tas_capital", "Tasmania—capital city", 700_000, 700_000),
    ("tas_other", "Tasmania—other", 550_000, 550_000),
    ("act", "Australian Capital Territory", 1_000_000, 1_000_000),
    ("nt", "Northern Territory", 600_000, 0),
    ("nt_capital", "Northern Territory—capital city", 0, 750_000),
    ("nt_other", "Northern Territory—other", 0, 600_000),
    (
        "jervis_bay_norfolk",
        "Jervis Bay Territory and Norfolk Island",
        550_000,
        550_000,
    ),
    (
        "christmas_cocos",
        "Christmas Island and Cocos (Keeling) Islands",
        400_000,
        400_000,
    ),
];

const FIRST_FACTS: [(&str, &str); 2] = [
    (
        "no_property_10_years",
        "no borrower has held a freehold interest in real property in Australia, or a relevant interest in land, in the last 10 years (s 29D(1)(a))",
    ),
    (
        "adult_citizen_or_resident",
        "each borrower is at least 18 and an Australian citizen or permanent resident (s 29D(1)(b))",
    ),
];
const FAMILY_FACTS: [(&str, &str); 4] = [
    (
        "single_with_dependant",
        "the borrower is single and a parent or legal guardian of at least one dependent child (s 29D(3)(d), (e), (3A)(e), (f))",
    ),
    (
        "property_interest_allowed",
        "the borrower holds no freehold interest in Australia, or will not within 4 weeks, or is buying out the property the loan is for (s 29D(3)(a), (3A)(a))",
    ),
    (
        "no_relevant_interest",
        "the borrower holds no relevant interest in land (s 29D(3)(aa), (3A)(b))",
    ),
    (
        "adult_citizen_or_resident",
        "the borrower is at least 18 and an Australian citizen or permanent resident (s 29D(3)(b), (3A)(c))",
    ),
];
const LOAN_FACTS: [(&str, &str); 4] = [
    (
        "eligible_lender",
        "the loan is made by an eligible lender (s 29C(2)(a))",
    ),
    (
        "owner_occupied",
        "the property is to be owner-occupied (s 29C(2)(h))",
    ),
    (
        "principal_and_interest",
        "the loan requires scheduled repayments of principal for its full term, apart from interest-only during construction (s 29C(2)(j), (3)(a))",
    ),
    (
        "term_30_years",
        "the loan term is no more than 30 years after any construction (s 29C(2)(k), (3)(b))",
    ),
];

/// The facts as the command passes them; see the README for each.
pub struct HomeGuaranteeArgs<'a> {
    pub stream: &'a Arg,
    pub area: &'a Arg,
    pub issue_date: &'a Arg,
    pub property_value: &'a Arg,
    pub loan_amount: &'a Arg,
    pub borrowers: &'a Arg,
    pub facts: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

fn issue_date(given: &Arg, year: &str) -> Result<String> {
    let start: i64 = year[..4].parse().unwrap_or_default();
    let (first, last) = (format!("{start}-07-01"), format!("{}-06-30", start + 1));
    match given.text().filter(|t| dates::parse(t).is_some()) {
        Some(t) if first.as_str() <= t && t <= last.as_str() => {
            if (FIRST..=LAST).contains(&t) {
                Ok(t.to_owned())
            } else {
                refuse(
                    "home_guarantee.date_uncovered",
                    "Not computed: issue_date must be from 1 October 2025, when the current caps began, to 26 November 2026, the day they were read.",
                )
            }
        }
        _ => refuse(
            "home_guarantee.date_invalid",
            format!("issue_date must be a YYYY-MM-DD date in {year}."),
        ),
    }
}

fn cap(given: &Arg, date: &str) -> Result<(&'static str, &'static str, i64)> {
    let later = date >= SPLIT;
    let found = given.text().and_then(|c| CAPS.iter().find(|a| a.0 == c));
    match found {
        Some(&(code, name, before, after)) if (if later { after } else { before }) > 0 => {
            Ok((code, name, if later { after } else { before }))
        }
        Some(_) => refuse(
            "home_guarantee.area_invalid",
            "The Northern Territory is one item (nt) for a guarantee issued before 1 July 2026 and 2 (nt_capital, nt_other) from that day (s 29F(1) items 14 and 14A).",
        ),
        None => refuse(
            "home_guarantee.area_invalid",
            format!(
                "area must be one of {}.",
                CAPS.iter().map(|a| a.0).collect::<Vec<_>>().join(", ")
            ),
        ),
    }
}

fn needed(family: bool) -> Vec<(&'static str, &'static str)> {
    let own: &[(&str, &str)] = if family { &FAMILY_FACTS } else { &FIRST_FACTS };
    own.iter().chain(&LOAN_FACTS).copied().collect()
}

fn facts(given: &Arg, family: bool) -> Result<()> {
    let Arg::Json(Value::Object(fields)) = given else {
        return refuse(
            "home_guarantee.facts_invalid",
            "facts must be an object of booleans.",
        );
    };
    match stated::unstated(fields, &needed(family), needed(family).len())? {
        None => Ok(()),
        Some(missing) => refuse(
            "home_guarantee.facts_not_stated",
            format!(
                "Not computed: the guarantee needs every eligibility fact stated; not stated: {missing}."
            ),
        ),
    }
}

/// The loan-to-value ratio as a percentage, unrounded, and whether it is in
/// the band: above 80 and no more than 95 (98 for a Family Home Guarantee)
/// from 1 July 2026 (s 29C(2)(i) and (2B)(c)); before it the Mandate said
/// 'between', and a ratio of exactly 80 or the top is refused.
fn band(lvr: &Decimal, top: i64, later: bool) -> Result<bool> {
    let ends = lvr.equals(&int(80))? || lvr.equals(&int(top))?;
    if ends && !later {
        return refuse(
            "home_guarantee.ratio_boundary",
            format!(
                "Not computed: before 1 July 2026 s 29C(2)(i) and (2B)(c) required a ratio 'between' 80 and {top} per cent, which does not settle a ratio of exactly {lvr}."
            ),
        );
    }
    Ok(lvr.gt(&int(80))? && !lvr.gt(&int(top))?)
}

fn stream(given: &Arg) -> Result<bool> {
    match given.text() {
        Some("first_home") => Ok(false),
        Some("family_home") => Ok(true),
        _ => refuse(
            "home_guarantee.stream_invalid",
            "stream must be first_home or family_home.",
        ),
    }
}

fn borrowers(given: &Arg) -> Result<i64> {
    match given.int() {
        Some(Some(n)) if n >= 1 => Ok(n),
        _ => refuse(
            "home_guarantee.borrowers_invalid",
            "borrowers must be a whole number of at least 1.",
        ),
    }
}

pub fn home_guarantee(args: &HomeGuaranteeArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let family = stream(args.stream)?;
    let date = issue_date(args.issue_date, &year)?;
    let (code, name, limit) = cap(args.area, &date)?;
    let value = money(args.property_value)?;
    let loan = money(args.loan_amount)?;
    if !value.gt(&int(0))? {
        return refuse(
            "home_guarantee.value_invalid",
            "property_value must be above 0.",
        );
    }
    let borrowers = borrowers(args.borrowers)?;
    facts(args.facts, family)?;
    let lvr = loan.mul(&int(100))?.div(&value)?;
    let top = if family { 98 } else { 95 };
    let in_band = band(&lvr, top, date.as_str() >= SPLIT)?;
    let tests = conditions(family, borrowers, !value.gt(&int(limit))?, in_band);
    let deposit = value.sub(&loan)?;
    let out = result(
        KIND,
        &year,
        vec![("price_cap", int(limit)), ("deposit", deposit)],
        Vec::new(),
    );
    let mut out = out?;
    out["stream"] = json!(if family { "family_home" } else { "first_home" });
    out["area"] = json!(name);
    out["area_code"] = json!(code);
    out["issue_date"] = json!(date);
    out["loan_to_value_ratio"] = json!(lvr.quantize(-2, Rounding::HalfUp)?.to_string());
    super::help_to_buy::set_conditions(&mut out, &tests);
    Ok(out)
}

fn conditions(family: bool, borrowers: i64, within: bool, in_band: bool) -> Vec<Test> {
    let (borrower_rule, ratio_rule) = if family {
        (
            "s 29C(2B)(a): a single borrower",
            "s 29C(2B)(c): a loan-to-value ratio above 80% and no more than 98%",
        )
    } else {
        (
            "s 29C(2)(b): no more than 2 borrowers",
            "s 29C(2)(i): a loan-to-value ratio above 80% and no more than 95%",
        )
    };
    vec![
        (
            "price_within_cap",
            "s 29C(2)(g) and s 29F(1): the property's value does not exceed the area's price cap",
            within,
        ),
        ("loan_to_value_ratio", ratio_rule, in_band),
        (
            "borrowers",
            borrower_rule,
            borrowers <= if family { 1 } else { 2 },
        ),
    ]
}
