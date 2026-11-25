// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `help_to_buy` and `help_to_buy_review`: the Commonwealth's equity
//! contribution and share under a Help to Buy arrangement, and the income
//! test and repayment a 5-yearly review leads to, under the Help to Buy Act
//! 2024 (compilation 1, 21 February 2025) and the Help to Buy Program
//! Directions 2025 (F2025L00682, as made, in force from 14 June 2025 with no
//! amendment), both read on 26 November 2026: ss 7, 8, 16, 17, 19, 21, 25, 34,
//! 36, 37 and 43. The 2026-27 income thresholds, indexed under s 9, are
//! Housing Australia's published amounts (firsthomebuyers.gov.au, 'Help to
//! Buy income and threshold updates 2026-27', read the same day). Housing
//! Australia's own judgements (financial capacity, s 20; the agreed
//! percentage, s 16(3); the lender's assessment, s 37) are stated facts.

use serde_json::{Value, json};

use super::stated;
use super::{Arg, Result, int, money, refuse, result, scope, whole_dollars};
use crate::dates;
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "help_to_buy";
const REVIEW: &str = "help_to_buy_review";
const LAST: &str = "2026-11-26";

/// Section 7 price caps: (code, area, cap).
const CAPS: [(&str, &str, i64); 16] = [
    (
        "nsw_capital_regional",
        "New South Wales—capital city and regional centre",
        1_300_000,
    ),
    ("nsw_other", "New South Wales—other", 800_000),
    (
        "vic_capital_regional",
        "Victoria—capital city and regional centre",
        950_000,
    ),
    ("vic_other", "Victoria—other", 650_000),
    (
        "qld_capital_regional",
        "Queensland—capital city and regional centre",
        1_000_000,
    ),
    ("qld_other", "Queensland—other", 700_000),
    ("wa_capital", "Western Australia—capital city", 850_000),
    ("wa_other", "Western Australia—other", 600_000),
    ("sa_capital", "South Australia—capital city", 900_000),
    ("sa_other", "South Australia—other", 500_000),
    ("tas_capital", "Tasmania—capital city", 700_000),
    ("tas_other", "Tasmania—other", 550_000),
    ("act", "Australian Capital Territory", 1_000_000),
    ("nt", "Northern Territory", 600_000),
    (
        "jervis_bay_norfolk",
        "Jervis Bay Territory and Norfolk Island",
        550_000,
    ),
    (
        "christmas_cocos",
        "Christmas Island and Cocos (Keeling) Islands",
        400_000,
    ),
];

/// Section 8 thresholds as indexed: (financial year, single, joint). The
/// unindexed amounts apply to every year before 2026-27 (s 9(1)).
const THRESHOLDS: [(&str, i64, i64); 2] =
    [("2025-26", 100_000, 160_000), ("2026-27", 103_000, 165_000)];

const FACTS: [(&str, &str); 8] = [
    (
        "citizen",
        "every applicant is an Australian citizen aged at least 18 (s 17(a))",
    ),
    (
        "financial_capacity",
        "Housing Australia believes the applicants could not likely buy the property without the arrangement (s 20)",
    ),
    (
        "principal_residence",
        "the property will be the applicants' principal place of residence (s 17(c), Sch 1 cl 1.3)",
    ),
    (
        "no_other_assistance",
        "no applicant receives a Commonwealth home buyer guarantee, a shared equity scheme or a State or Territory home ownership loan or guarantee (s 17(d))",
    ),
    (
        "no_property_interest",
        "no applicant holds a disqualifying property interest, or s 18 applies (s 17(e))",
    ),
    (
        "costs_covered",
        "the applicants will cover all other costs of the purchase (s 17(f)(ii))",
    ),
    (
        "sole_owners",
        "the applicants will be the only registered owners on settlement (s 17(g))",
    ),
    (
        "participating_jurisdiction",
        "the property is in a participating or cooperating State or a Territory (Act ss 10 and 15)",
    ),
];

/// The entry facts as the command passes them; see the README for each.
pub struct HelpToBuyArgs<'a> {
    pub area: &'a Arg,
    pub property: &'a Arg,
    pub purchase_price: &'a Arg,
    pub agreed_percentage: &'a Arg,
    pub deposit: &'a Arg,
    pub lender_valuation: &'a Arg,
    pub applicants: &'a Arg,
    pub taxable_income: &'a Arg,
    pub application_date: &'a Arg,
    pub facts: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// The review facts as the command passes them; see the README for each.
pub struct HelpToBuyReviewArgs<'a> {
    pub applicants: &'a Arg,
    pub taxable_income: &'a Arg,
    pub previous_taxable_income: &'a Arg,
    pub share_percentage: &'a Arg,
    pub property_value: &'a Arg,
    pub capacity_to_repay: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

fn area(given: &Arg) -> Result<&'static (&'static str, &'static str, i64)> {
    match given.text().and_then(|c| CAPS.iter().find(|a| a.0 == c)) {
        Some(found) => Ok(found),
        None => refuse(
            "help_to_buy.area_invalid",
            format!(
                "area must be one of {}.",
                CAPS.iter().map(|a| a.0).collect::<Vec<_>>().join(", ")
            ),
        ),
    }
}

/// `(single parent or joint, the label)` for the applicants.
fn applicants(given: &Arg, code: &'static str) -> Result<(bool, &'static str)> {
    match (given.text(), code) {
        (Some("single"), _) => Ok((false, "single")),
        (Some("single_parent"), _) => Ok((true, "single_parent")),
        (Some("joint"), _) => Ok((true, "joint")),
        (_, "help_to_buy") => refuse(
            "help_to_buy.applicants_invalid",
            "applicants must be single, single_parent or joint.",
        ),
        _ => refuse(
            "help_to_buy_review.applicants_invalid",
            "applicants must be single, single_parent or joint.",
        ),
    }
}

fn threshold(year: &str, joint: bool) -> i64 {
    let (_, single, both) = THRESHOLDS
        .iter()
        .find(|t| t.0 == year)
        .unwrap_or(&THRESHOLDS[0]);
    if joint { *both } else { *single }
}

/// Whole dollars from 1 to 1,000,000,000.
fn dollars(given: &Arg, name: &str, zero: bool) -> Result<Decimal> {
    let value = money(given)?;
    whole_dollars(&value, name)?;
    let least = if zero { 0 } else { 1 };
    if value.lt(&int(least))? || value.gt(&int(1_000_000_000))? {
        return refuse(
            "help_to_buy.value_invalid",
            format!("{name} must be whole dollars from {least} to 1,000,000,000."),
        );
    }
    Ok(value)
}

/// A percentage above 0 and at most 100, at most 2 decimal places.
fn percentage(given: &Arg, name: &str) -> Result<Decimal> {
    let value = money(given)?;
    if !value.gt(&int(0))? || value.gt(&int(100))? {
        return refuse(
            "help_to_buy.percentage_invalid",
            format!("{name} must be a percentage above 0 and at most 100."),
        );
    }
    Ok(value)
}

fn application_date(given: &Arg, year: &str) -> Result<String> {
    let start: i64 = year[..4].parse().unwrap_or_default();
    let (first, last) = (format!("{start}-07-01"), format!("{}-06-30", start + 1));
    match given.text().filter(|t| dates::parse(t).is_some()) {
        Some(t) if first.as_str() <= t && t <= last.as_str() && t <= LAST => Ok(t.to_owned()),
        Some(t) if first.as_str() <= t && t <= last.as_str() => refuse(
            "help_to_buy.date_uncovered",
            format!(
                "Not computed: {t} is after 26 November 2026, the day the price caps and thresholds were read."
            ),
        ),
        _ => refuse(
            "help_to_buy.date_invalid",
            format!("application_date must be a YYYY-MM-DD date in {year}."),
        ),
    }
}

fn facts(given: &Arg) -> Result<()> {
    let Arg::Json(Value::Object(fields)) = given else {
        return refuse(
            "help_to_buy.facts_invalid",
            "facts must be an object of booleans.",
        );
    };
    let keys: Vec<&str> = FACTS.iter().map(|(k, _)| *k).collect();
    stated::known_keys(fields, "facts", "facts", &keys)?;
    let mut missing = Vec::new();
    for (key, text) in FACTS {
        match fields.get(key) {
            Some(Value::Bool(true)) => {}
            None | Some(Value::Bool(false)) => missing.push(text),
            Some(_) => {
                return refuse(
                    "input.flag_invalid",
                    format!("facts: {key} must be a boolean."),
                );
            }
        }
    }
    if missing.is_empty() {
        return Ok(());
    }
    refuse(
        "help_to_buy.facts_not_stated",
        format!(
            "Not computed: the arrangement needs every eligibility fact stated; not stated: {}.",
            missing.join("; ")
        ),
    )
}

/// A percentage of `of`, in cents.
fn part(of: &Decimal, percent: &Decimal) -> Result<Decimal> {
    Ok(of
        .mul(percent)?
        .div(&int(100))?
        .quantize(-2, Rounding::HalfUp)?)
}

/// `(condition, provision, met)`.
pub(super) type Test = (&'static str, &'static str, bool);

fn property(given: &Arg) -> Result<bool> {
    match given.text() {
        Some("new") => Ok(true),
        Some("existing") => Ok(false),
        _ => refuse(
            "help_to_buy.property_invalid",
            "property must be new or existing.",
        ),
    }
}

fn entry_tests(within_cap: bool, worked: &Worked, income_ok: bool) -> Vec<Test> {
    vec![
        (
            "price_within_cap",
            "Program Directions s 21(b) and s 7: the purchase price does not exceed the area's price cap",
            within_cap,
        ),
        (
            "percentage_within_limits",
            "s 16(1): 5% to 30% of the purchase price for an existing dwelling, 5% to 40% for a new home",
            worked.within,
        ),
        (
            "deposit_with_contribution_20_percent",
            "s 16(2): the contribution and deposit together at least 20% of the lender's valuation",
            worked.lmi_free,
        ),
        (
            "deposit_at_least_2_percent",
            "s 17(f)(i): a deposit of at least 2% of the purchase price",
            worked.deposit_ok,
        ),
        (
            "income_test",
            "s 19: taxable income on the latest notice of assessment (combined for joint applicants) no more than the threshold for the financial year of application",
            income_ok,
        ),
    ]
}

/// Sets `eligible`, `conditions_not_met` and `conditions` from the tests.
pub(super) fn set_conditions(out: &mut Value, tests: &[Test]) {
    out["eligible"] = json!(tests.iter().all(|t| t.2));
    out["conditions_not_met"] = json!(
        tests
            .iter()
            .filter(|t| !t.2)
            .map(|t| t.0)
            .collect::<Vec<_>>()
    );
    out["conditions"] = json!(
        tests
            .iter()
            .map(|(c, p, m)| json!({"condition": c, "provision": p, "met": m}))
            .collect::<Vec<_>>()
    );
}

pub fn help_to_buy(args: &HelpToBuyArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let (code, name, cap) = *area(args.area)?;
    let new_home = property(args.property)?;
    let price = dollars(args.purchase_price, "purchase_price", false)?;
    let agreed = percentage(args.agreed_percentage, "agreed_percentage")?;
    let deposit = dollars(args.deposit, "deposit", true)?;
    let valuation = dollars(args.lender_valuation, "lender_valuation", false)?;
    let (joint, who) = applicants(args.applicants, KIND)?;
    let income = dollars(args.taxable_income, "taxable_income", true)?;
    let date = application_date(args.application_date, &year)?;
    facts(args.facts)?;
    let contribution = part(&price, &agreed)?;
    if deposit.add(&contribution)?.gt(&price)? {
        return refuse(
            "help_to_buy.deposit_exceeds_price",
            "deposit and the Commonwealth's contribution together cannot be more than purchase_price.",
        );
    }
    let worked = entry(
        Entry {
            price: &price,
            agreed: &agreed,
            deposit: &deposit,
            valuation: &valuation,
            contribution: &contribution,
        },
        new_home,
    )?;
    let limit = threshold(&year, joint);
    let tests = entry_tests(!price.gt(&int(cap))?, &worked, !income.gt(&int(limit))?);
    let mut out = result(
        KIND,
        &year,
        vec![
            ("price_cap", int(cap)),
            ("contribution", contribution),
            ("minimum_contribution", worked.minimum),
            ("minimum_deposit", worked.min_deposit),
            ("income_threshold", int(limit)),
        ],
        Vec::new(),
    )?;
    out["area"] = json!(name);
    out["area_code"] = json!(code);
    out["property"] = json!(if new_home { "new" } else { "existing" });
    out["applicants"] = json!(who);
    out["application_date"] = json!(date);
    out["share_percentage_at_settlement"] = json!(worked.share.to_string());
    set_conditions(&mut out, &tests);
    Ok(out)
}

struct Entry<'a> {
    price: &'a Decimal,
    agreed: &'a Decimal,
    deposit: &'a Decimal,
    valuation: &'a Decimal,
    contribution: &'a Decimal,
}

struct Worked {
    within: bool,
    lmi_free: bool,
    deposit_ok: bool,
    minimum: Decimal,
    min_deposit: Decimal,
    share: Decimal,
}

/// Sections 16, 17(f)(i) and 25(3) item 1 on the stated amounts.
fn entry(e: Entry, new_home: bool) -> Result<Worked> {
    let most = if new_home { 40 } else { 30 };
    let within = !e.agreed.lt(&int(5))? && !e.agreed.gt(&int(most))?;
    let fifth = part(e.valuation, &int(20))?;
    let minimum = super::py_max(fifth.sub(e.deposit)?, int(0))?;
    let lmi_free = !e.contribution.lt(&minimum)?;
    let min_deposit = part(e.price, &int(2))?;
    let deposit_ok = !e.deposit.lt(&min_deposit)?;
    // s 25(3) item 1: a valuation below the price lifts the share so that
    // it is worth the contribution on the settlement date.
    let share = if e.valuation.lt(e.price)? {
        e.contribution
            .mul(&int(100))?
            .div(e.valuation)?
            .quantize(-2, Rounding::HalfUp)?
    } else {
        e.agreed.clone()
    };
    Ok(Worked {
        within,
        lmi_free,
        deposit_ok,
        minimum,
        min_deposit,
        share,
    })
}

/// Nearest $1,000, refusing an amount exactly midway (the Directions do not
/// say which way it goes).
fn nearest_thousand(amount: &Decimal) -> Result<Decimal> {
    let thousands = amount.div(&int(1000))?;
    let down = thousands.quantize(0, Rounding::Down)?;
    if thousands.sub(&down)?.equals(&super::d("0.5"))? {
        return refuse(
            "help_to_buy_review.rounding_midway",
            "Not computed: 5% of property_value is exactly midway between 2 multiples of $1,000, and the Program Directions do not say which way it rounds (ss 37(5)(e) and 43(1)(a)).",
        );
    }
    Ok(thousands.quantize(0, Rounding::HalfUp)?.mul(&int(1000))?)
}

fn review_income(given: &Arg, name: &str) -> Result<Decimal> {
    let value = money(given)?;
    whole_dollars(&value, name)?;
    Ok(value)
}

fn review_share(given: &Arg) -> Result<Decimal> {
    match money(given) {
        Ok(p) if p.gt(&int(0))? && !p.gt(&int(100))? => Ok(p),
        _ => refuse(
            "help_to_buy_review.share_invalid",
            "share_percentage must be a percentage above 0 and at most 100, at most 2 decimal places.",
        ),
    }
}

fn review_value(given: &Arg) -> Result<Decimal> {
    let value = money(given)?;
    if !value.gt(&int(0))? {
        return refuse(
            "help_to_buy_review.value_invalid",
            "property_value must be above 0.",
        );
    }
    Ok(value)
}

pub fn help_to_buy_review(args: &HelpToBuyReviewArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, REVIEW)?;
    let (joint, who) = applicants(args.applicants, REVIEW)?;
    let latest = review_income(args.taxable_income, "taxable_income")?;
    let earlier = review_income(args.previous_taxable_income, "previous_taxable_income")?;
    let share = review_share(args.share_percentage)?;
    let value = review_value(args.property_value)?;
    let capacity = match args.capacity_to_repay {
        Arg::Bool(b) => *b,
        _ => return refuse("input.flag_invalid", "capacity_to_repay must be a boolean."),
    };
    let start: i64 = year[..4].parse().unwrap_or_default();
    let before = format!("{}-{:02}", start - 1, start % 100);
    let limits = [threshold(&year, joint), threshold(&before, joint)];
    let exceeded = latest.gt(&int(limits[0]))? && earlier.gt(&int(limits[1]))?;
    let share_value = part(&value, &share)?;
    let five = part(&value, &int(5))?;
    let over_five = share_value.gt(&five)?;
    let minimum = if over_five {
        nearest_thousand(&five)?
    } else {
        share_value.clone()
    };
    let required = if exceeded && capacity {
        minimum.clone()
    } else {
        int(0)
    };
    let mut out = result(
        REVIEW,
        &year,
        vec![
            ("commonwealth_share_value", share_value),
            ("income_threshold", int(limits[0])),
            ("previous_income_threshold", int(limits[1])),
            ("required_repayment", required),
            ("minimum_voluntary_repayment", minimum),
        ],
        Vec::new(),
    )?;
    out["applicants"] = json!(who);
    out["income_exceeded_both_years"] = json!(exceeded);
    out["outcome"] = json!(outcome(exceeded, capacity, over_five));
    Ok(out)
}

fn outcome(exceeded: bool, capacity: bool, over_five: bool) -> &'static str {
    match (exceeded, capacity, over_five) {
        (false, _, _) => {
            "Taxable income did not exceed the threshold in both income years (s 34): the review requires no repayment for income."
        }
        (true, true, true) => {
            "Income exceeded the threshold in both years (ss 34 and 36(1)): with the lender's assessment of capacity, s 37(5)(e) requires repaying 5% of the property's value, rounded to the nearest $1,000, within 90 days; more may be repaid, and a further assessment follows while any share remains (s 37(5)(f) to (h))."
        }
        (true, true, false) => {
            "Income exceeded the threshold in both years (ss 34 and 36(1)): with a share of 5% of the value or less and the lender's assessment of capacity, s 37(8) requires repaying the share in full within 90 days, ending the arrangement."
        }
        (true, false, _) => {
            "Income exceeded the threshold in both years (ss 34 and 36(1)), but the lender assessed no capacity to repay: no repayment is required now and Housing Australia requires a further assessment at least 12 months later (s 37(3)(c), (6) and (9))."
        }
    }
}
