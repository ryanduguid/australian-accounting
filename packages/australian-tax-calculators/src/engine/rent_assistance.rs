// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `rent_assistance`: one fortnight's rent assistance under Social Security
//! Act 1991 Part 3.7 (compilation 234 of 20 September 2026, read on 26
//! November 2026) for a person without dependent children whose payment is
//! worked out under Pension Rate Calculator A (age pension, disability support
//! pension at 21 or over, carer payment; s 1070L) or Benefit Rate Calculator B
//! (jobseeker payment; s 1070Q): rate A, (fortnightly rent - rent threshold)
//! x 3/4, or x 3/8 for a couple both receiving it (column 3 of the tables,
//! read from the WMF images of the compilation's Word download), up to the
//! maximum rate for the family situation, two-thirds of it for a single
//! person sharing accommodation. The
//! indexed thresholds and maximum rates are the Department of Social Services'
//! Social Security Guide 5.1.7.10 (as at 20 September 2026, read through
//! Firecrawl on 26 November 2026) and Services Australia's 'A guide to
//! Australian Government payments' for 1 July to 19 September 2026 and 20
//! September to 31 December 2026, for each day from 1 July 2026 to 19 March
//! 2027.

use serde_json::{Value, json};

use super::stated;
use super::{Arg, Result, Step, derived_result, int, money, precise, rate, refuse, scope, step};
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "rent_assistance";

const FACTS: [(&str, &str); 4] = [
    (
        "rate_calculator",
        "the person's payment is worked out under Pension Rate Calculator A (age pension, disability support pension at 21 or over, carer payment) or Benefit Rate Calculator B (jobseeker payment), and the person meets that payment's specific requirement (ss 1070D and 1070H)",
    ),
    (
        "pays_private_rent",
        "the person pays or is liable to pay rent, other than Government rent, for premises in Australia, and is neither an aged care resident nor an ineligible homeowner (s 1070C)",
    ),
    (
        "no_dependent_children",
        "the person has no dependent child: rent assistance for a family with children is paid with family tax benefit Part A (A New Tax System (Family Assistance) Act 1999 Schedule 1 clauses 38B to 38H)",
    ),
    (
        "partner_not_veteran",
        "no partner receives a service pension, income support supplement or a veteran payment (s 1070L items 4 to 6)",
    ),
];

/// The thresholds then the maximum rates per fortnight, each `[single,
/// couple]`, from each date to the next, the last to [`UNTIL`].
type Indexed = [[&'static str; 2]; 2];
const RATES: [(&str, Indexed); 2] = [
    ("2026-07-01", [["154.80", "250.80"], ["219.40", "206.80"]]),
    ("2026-09-20", [["157.80", "255.80"], ["223.80", "211.00"]]),
];
const UNTIL: &str = "2027-03-19";

/// Each family situation: (name, threshold and maximum `[single, couple]`
/// index, share of the amount, the table item in s 1070L and s 1070Q).
#[derive(Clone, Copy, PartialEq)]
enum Share {
    Whole,
    Sharer,
    Half,
}
const SITUATIONS: [(&str, usize, usize, Share, &str); 8] = [
    (
        "single",
        0,
        0,
        Share::Whole,
        "s 1070L item 1, s 1070Q item 1",
    ),
    (
        "single_sharer",
        0,
        0,
        Share::Sharer,
        "s 1070L(2) and s 1070Q(2): two-thirds of rate B, to the nearest cent, 0.5 cent up",
    ),
    (
        "couple",
        1,
        1,
        Share::Whole,
        "s 1070L item 2, s 1070Q item 2",
    ),
    (
        "couple_both_receive",
        1,
        1,
        Share::Half,
        "s 1070L item 3, s 1070Q item 3: rate A x 3/8, half rate B",
    ),
    (
        "illness_separated",
        0,
        0,
        Share::Whole,
        "s 1070L item 7, s 1070Q item 4",
    ),
    (
        "respite_care",
        0,
        0,
        Share::Whole,
        "s 1070L item 8, s 1070Q item 5",
    ),
    (
        "temporarily_separated",
        0,
        1,
        Share::Whole,
        "s 1070L item 9, s 1070Q item 6",
    ),
    (
        "partner_in_gaol",
        0,
        0,
        Share::Whole,
        "s 1070L item 10, s 1070Q item 7",
    ),
];

/// The facts as the command passes them; see the README for each.
pub struct RentAssistanceArgs<'a> {
    pub date: &'a Arg,
    pub situation: &'a Arg,
    pub fortnightly_rent: &'a Arg,
    pub facts: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

fn period(given: &Arg) -> Result<&'static Indexed> {
    match super::dated(given, &RATES, UNTIL) {
        Some(rates) => Ok(rates),
        None => refuse(
            "rent_assistance.date_uncovered",
            "Not computed: date must be a YYYY-MM-DD day from 1 July 2026 to 19 March 2027, the days the indexed rates were read for.",
        ),
    }
}

fn facts(given: &Arg) -> Result<()> {
    let Arg::Json(Value::Object(fields)) = given else {
        return refuse(
            "rent_assistance.facts_invalid",
            "facts must be an object of booleans.",
        );
    };
    match stated::unstated(fields, &FACTS, FACTS.len())? {
        None => Ok(()),
        Some(missing) => refuse(
            "rent_assistance.facts_not_stated",
            format!(
                "Not computed: the worksheet covers only the stated case; not stated: {missing}."
            ),
        ),
    }
}

pub fn rent_assistance(args: &RentAssistanceArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let p = period(args.date)?;
    let Some(&(name, at, top, share, item)) = SITUATIONS
        .iter()
        .find(|s| Some(s.0) == args.situation.text())
    else {
        return refuse(
            "rent_assistance.situation_invalid",
            "situation must be single, single_sharer, couple, couple_both_receive, illness_separated, respite_care, temporarily_separated or partner_in_gaol.",
        );
    };
    let rent = money(args.fortnightly_rent)?;
    facts(args.facts)?;
    let d = super::d;
    let threshold = d(p[0][at]);
    let rate_b = d(p[1][top]);
    let steps = precise(|| working(share, [&rent, &threshold, &rate_b]))?;
    let mut out = derived_result(
        KIND,
        &year,
        steps,
        vec![rate("rent_threshold", &threshold), rate("rate_b", &rate_b)],
    )?;
    out["date"] = json!(args.date.text());
    out["situation"] = json!(name);
    out["table_item"] = json!(item);
    out["rate_basis"] = json!(
        "per fortnight; under Pension Rate Calculator A the amount x 26 is added to the maximum basic rate a year (s 1070A(b))"
    );
    Ok(out)
}

const AUTH: [&str; 3] = [
    "SSA s 1070L(2) and s 1070Q(2) column 3, s 1070T and s 1070U: rate A is (fortnightly rent - rent threshold) x 3/4, x 3/8 for a couple both receiving it",
    "SSA s 1070L(2) and s 1070Q(2): the maximum rent assistance rate is rate B for the family situation",
    "SSA s 1070L(2) and s 1070Q(2): the rate of rent assistance is rate A, but only up to the maximum rent assistance rate",
];

fn working(share: Share, [rent, threshold, rate_b]: [&Decimal; 3]) -> Result<Vec<Step>> {
    let mut rate_a = super::py_max(rent.sub(threshold)?, int(0))?.mul(&super::d("0.75"))?;
    let (maximum, max_formula, a_formula) = match share {
        Share::Whole => (
            rate_b.clone(),
            "rate_b",
            "max(fortnightly_rent - rent_threshold, 0) x 0.75",
        ),
        Share::Sharer => (
            rate_b
                .mul(&int(2))?
                .div(&int(3))?
                .quantize(-2, Rounding::HalfUp)?,
            "cents(rate_b x 2 / 3)",
            "max(fortnightly_rent - rent_threshold, 0) x 0.75",
        ),
        Share::Half => {
            rate_a = rate_a.div(&int(2))?;
            (
                rate_b.div(&int(2))?,
                "rate_b / 2",
                "max(fortnightly_rent - rent_threshold, 0) x 0.75 / 2",
            )
        }
    };
    let paid = super::py_min(rate_a.clone(), maximum.clone())?;
    Ok(vec![
        step(
            "rate_a",
            AUTH[0],
            a_formula,
            &[rent, threshold],
            rate_a.clone(),
        ),
        step(
            "maximum_rate",
            AUTH[1],
            max_formula,
            &[rate_b],
            maximum.clone(),
        ),
        step(
            "rent_assistance",
            AUTH[2],
            "min(rate_a, maximum_rate)",
            &[&rate_a, &maximum],
            paid,
        ),
    ])
}
