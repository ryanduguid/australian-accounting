// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `thin-capitalisation`: the debt deductions a company that is a general
//! class investor has disallowed under ITAA 1997 Subdivision 820-AA, by the
//! fixed ratio test (ss 820-50(1)(a), 820-51(1) and 820-52), the group ratio
//! test (ss 820-50(1)(b), 820-51(2) and 820-53 to 820-55) or the third party
//! debt test (ss 820-50(1)(c) and 820-427A), each disallowed deduction's share
//! (s 820-50(2)) and the special deduction for earlier fixed ratio test
//! disallowed amounts (ss 820-56 to 820-59). Exemptions: ss 820-35, 820-37 and
//! 820-39. Read on 26 November 2026 in ITAA 1997 compilation 268
//! (C2026C00471) with the application provisions, Treasury Laws Amendment
//! (Making Multinationals Pay Their Fair Share—Integrity and Transparency) Act
//! 2024 Schedule 2 items 144 and 146: income years starting on or after
//! 1 July 2023, and the older law for Australian plantation forestry entities.

mod carry;

use serde_json::{Map, Value, json};

use super::{
    Arg, Result, Step, d, derived_result, flag, money, precise, rate, refuse, scope, signed_money,
    step,
};
use crate::decimal::{Calc, Decimal, Rounding};

const KIND: &str = "thin_capitalisation";
const MOST_DEDUCTIONS: usize = 200;

/// The worksheet's inputs, as the caller gave them; `given` holds the
/// optional inputs present, by name.
pub struct ThinCapitalisationArgs<'a> {
    pub test: &'a Arg,
    pub entity_class: &'a Arg,
    pub plantation_forestry_entity: &'a Arg,
    pub special_purpose_entity: &'a Arg,
    pub outward_investor_only: &'a Arg,
    pub associate_inclusive_debt_deductions: &'a Arg,
    pub debt_deductions: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
    pub given: &'a [(&'static str, Arg)],
}

impl ThinCapitalisationArgs<'_> {
    fn get(&self, key: &str) -> Option<&Arg> {
        self.given.iter().find(|(k, _)| *k == key).map(|(_, v)| v)
    }
}

/// The lines of tax EBITDA (s 820-52(1)) after taxable income, in the order
/// the working takes them: subtracted (s 820-52(2), (3) and (10)), then added
/// after net debt deductions (s 820-52(1)(c) and (ca)).
const SUBTRACTED: [&str; 3] = [
    "franking_credits",
    "associate_dividends",
    "rd_notional_deductions",
];
const ADDED: [&str; 4] = [
    "division_40_43_deductions",
    "forestry_deductions",
    "section_70_120_deductions",
    "excess_tax_ebitda_amount",
];

/// The inputs that may be negative.
const SIGNED: [&str; 4] = [
    "taxable_income",
    "gr_group_net_third_party_interest_expense",
    "gr_group_net_profit",
    "gr_group_adjusted_net_third_party_interest_expense",
];

const GR_GROUP: [&str; 5] = [
    "gr_group_net_third_party_interest_expense",
    "gr_group_net_profit",
    "gr_group_adjusted_net_third_party_interest_expense",
    "gr_group_depreciation_and_amortisation",
    "gr_group_negative_member_ebitda",
];

/// One debt deduction for the year.
struct Deduction {
    label: String,
    amount: Decimal,
    third_party: Option<bool>,
}

#[derive(Clone, Copy, PartialEq)]
enum Test {
    Fixed,
    Group,
    ThirdParty,
}

fn fact<T>(why: impl Into<String>) -> Result<T> {
    refuse("thin_capitalisation.fact_invalid", why)
}

fn not_used<T>(key: &str, why: &str) -> Result<T> {
    refuse(
        "thin_capitalisation.input_not_used",
        format!("{key} is not used here: {why}."),
    )
}

fn test(arg: &Arg) -> Result<Test> {
    match arg.text() {
        Some("fixed_ratio") => Ok(Test::Fixed),
        Some("group_ratio") => Ok(Test::Group),
        Some("third_party_debt") => Ok(Test::ThirdParty),
        _ => fact("test must be fixed_ratio, group_ratio or third_party_debt."),
    }
}

/// `true` for a general class investor, `false` for an entity Division 820
/// does not cover; the other classes keep tests this worksheet does not build.
fn general_class(arg: &Arg) -> Result<bool> {
    match arg.text() {
        Some("general_class_investor") => Ok(true),
        Some("not_covered") => Ok(false),
        Some(
            "outward_investing_financial_entity"
            | "inward_investing_financial_entity"
            | "outward_investing_adi"
            | "inward_investing_adi",
        ) => refuse(
            "thin_capitalisation.entity_class_not_built",
            "A financial entity (non-ADI) is tested under Subdivision 820-B or 820-C, and an ADI under 820-D or 820-E, by tests this worksheet does not apply: it covers a general class investor (s 820-46(2)) only.",
        ),
        _ => fact(
            "entity_class must be general_class_investor, not_covered, outward_investing_financial_entity, inward_investing_financial_entity, outward_investing_adi or inward_investing_adi.",
        ),
    }
}

fn text_amount(v: Option<&Value>) -> Option<Decimal> {
    let parsed = Arg::Decimal(Decimal::parse_input(v?.as_str()?).ok()?);
    money(&parsed).ok()
}

fn deduction(v: &Value, n: usize) -> Result<Deduction> {
    let wrong = |why: &str| {
        refuse(
            "thin_capitalisation.debt_deductions_invalid",
            format!("debt deduction {n}: {why}"),
        )
    };
    let Some(o) = v.as_object() else {
        return wrong("must be an object.");
    };
    if !o
        .keys()
        .all(|k| matches!(k.as_str(), "label" | "amount" | "third_party"))
    {
        return wrong("takes label, amount and third_party only.");
    }
    let label = o
        .get("label")
        .and_then(Value::as_str)
        .filter(|l| !l.trim().is_empty());
    let third_party = match o.get("third_party") {
        None => Some(None),
        Some(Value::Bool(b)) => Some(Some(*b)),
        Some(_) => None,
    };
    match (label, text_amount(o.get("amount")), third_party) {
        (Some(l), Some(amount), Some(third_party)) => Ok(Deduction {
            label: l.to_string(),
            amount,
            third_party,
        }),
        _ => wrong(
            "needs a label, an amount in AUD from 0 and, optionally, third_party true or false.",
        ),
    }
}

fn deductions(given: &Arg, t: Test) -> Result<Vec<Deduction>> {
    let items = match given {
        Arg::Json(Value::Array(items)) if (1..=MOST_DEDUCTIONS).contains(&items.len()) => items,
        _ => {
            return refuse(
                "thin_capitalisation.debt_deductions_invalid",
                format!("debt_deductions must be a list of 1 to {MOST_DEDUCTIONS} objects."),
            );
        }
    };
    let all = items
        .iter()
        .enumerate()
        .map(|(i, v)| deduction(v, i + 1))
        .collect::<Result<Vec<_>>>()?;
    let marked = all.iter().filter(|x| x.third_party.is_some()).count();
    match t {
        Test::ThirdParty if marked < all.len() => refuse(
            "thin_capitalisation.input_missing",
            "Under the third party debt test every debt deduction needs third_party: whether it is attributable to a debt interest that satisfies the third party debt conditions (s 820-427A(1) to (3)).",
        ),
        Test::Fixed | Test::Group if marked > 0 => not_used(
            "third_party",
            "it matters only under the third party debt test (s 820-427A)",
        ),
        _ => Ok(all),
    }
}

const NIL_CARRY: (&str, &str) = (
    "frt_disallowed_amounts",
    "a choice under s 820-46(3) or (4) makes every earlier fixed ratio test disallowed amount nil (s 820-58)",
);

/// The inputs a test does not use are refused rather than ignored.
fn check_unused(a: &ThinCapitalisationArgs, t: Test) -> Result<()> {
    let group = "the GR group figures belong to the group ratio test (s 820-53)";
    let third = "the third party debt test compares debt deductions with the third party earnings limit alone (s 820-50(1)(c))";
    let unused: Vec<(&str, &str)> = match t {
        Test::Fixed => GR_GROUP.iter().map(|k| (*k, group)).collect(),
        Test::Group => vec![
            (
                "excess_tax_ebitda_amount",
                "s 820-60 adds it only where no choice is made (fixed ratio test)",
            ),
            NIL_CARRY,
        ],
        Test::ThirdParty => ["taxable_income", "interest_income"]
            .into_iter()
            .chain(SUBTRACTED)
            .chain(ADDED)
            .chain(GR_GROUP)
            .map(|k| (k, third))
            .chain([NIL_CARRY])
            .collect(),
    };
    match unused.iter().find(|(k, _)| a.get(k).is_some()) {
        Some((k, why)) => not_used(k, why),
        None => Ok(()),
    }
}

/// An optional amount, read as a money amount or, where it may be negative,
/// a signed one; `None` when absent.
fn optional_amount(a: &ThinCapitalisationArgs, key: &str) -> Result<Option<Decimal>> {
    a.get(key)
        .map(|v| {
            if SIGNED.contains(&key) {
                signed_money(v)
            } else {
                money(v)
            }
        })
        .transpose()
}

fn required(a: &ThinCapitalisationArgs, key: &str, why: &str) -> Result<Decimal> {
    match optional_amount(a, key)? {
        Some(v) => Ok(v),
        None => refuse(
            "thin_capitalisation.input_missing",
            format!("{key} is needed: {why}."),
        ),
    }
}

/// Which provision, if any, keeps Subdivision 820-AA from disallowing
/// anything, with the ratio s 820-37 compares when it is worked out.
fn exemption(
    a: &ThinCapitalisationArgs,
    general: bool,
    own: &Decimal,
) -> Result<(Option<&'static str>, Option<Decimal>)> {
    let associates = money(a.associate_inclusive_debt_deductions)?;
    if associates.lt(own)? {
        return fact(
            "associate_inclusive_debt_deductions includes the entity's own debt deductions, so cannot be less than their total.",
        );
    }
    let special = flag(a.special_purpose_entity, "special_purpose_entity")?;
    let outward = flag(a.outward_investor_only, "outward_investor_only")?;
    let ratio = assets_ratio(a, outward)?;
    let meets = match &ratio {
        Some(r) => !r.lt(&d("0.9"))?,
        None => false,
    };
    let reason = if !general {
        Some(
            "s 820-46(2): not a general class investor, and no other Subdivision of Division 820 covers the entity",
        )
    } else if special {
        Some("s 820-39(1): a special purpose entity meeting s 820-39(3) throughout the year")
    } else if !associates.gt(&d("2000000"))? {
        Some(
            "s 820-35: debt deductions of the entity and its associate entities of $2 million or less",
        )
    } else if meets {
        Some(
            "s 820-37(1): average Australian assets at least 90% of average total assets, associates' included",
        )
    } else {
        None
    };
    Ok((reason, ratio))
}

/// Section 820-37's ratio, for an entity stated to be an outward investor
/// only (s 820-37(1)(a)(ii) and (b)).
fn assets_ratio(a: &ThinCapitalisationArgs, outward: bool) -> Result<Option<Decimal>> {
    let aus = optional_amount(a, "average_australian_assets")?;
    let all = optional_amount(a, "average_total_assets")?;
    match (outward, aus, all) {
        (false, None, None) => Ok(None),
        (false, _, _) => not_used(
            "average_australian_assets and average_total_assets",
            "s 820-37 applies only to an entity that would be an outward investing financial entity (non-ADI) and not an inward investing one (outward_investor_only)",
        ),
        (true, Some(aus), Some(all)) if all.gt(&d("0"))? && !all.lt(&aus)? => {
            Ok(Some(precise(|| Ok(aus.div(&all)?))?))
        }
        (true, Some(_), Some(_)) => fact(
            "average_total_assets must be above nil and not less than average_australian_assets.",
        ),
        (true, _, _) => refuse(
            "thin_capitalisation.input_missing",
            "average_australian_assets and average_total_assets are needed for an outward investor (s 820-37(1)(c)).",
        ),
    }
}

fn total<'a>(values: impl Iterator<Item = &'a Decimal>) -> Calc<Decimal> {
    let mut sum = d("0");
    for v in values {
        sum = sum.add(v)?;
    }
    Ok(sum)
}

/// A step adding `terms`, written as numbers; `0` when there are none.
fn sum_step(id: &'static str, authority: &'static str, terms: &[&Decimal]) -> Calc<Step> {
    let value = total(terms.iter().copied())?;
    let formula = if terms.is_empty() {
        "0".to_string()
    } else {
        terms
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" + ")
    };
    Ok(step(id, authority, &formula, &[], value))
}

fn max0(v: Decimal) -> Calc<Decimal> {
    Ok(if v.gt(&d("0"))? { v } else { d("0") })
}

const DD: &str = "s 820-40 and s 820-50(3)(a): the year's debt deductions, after any disallowed under Subdivision 820-EAA (s 820-31)";
const NDD: &str = "s 820-50(3): debt deductions less interest and amounts in the nature of interest included in assessable income; may be negative";
const EBITDA: &str = "s 820-52(1), (1A), (2), (3) and (10): taxable income (losses of earlier years deducted, a tax loss negative) less franking credits, associate dividends and R&D notional deductions, plus net debt deductions, Division 40 and 43, forestry and s 70-120 deductions and any excess tax EBITDA amount (s 820-60), nil if negative";
const DISALLOWED: &str = "s 820-50(1): the amount by which net debt deductions (fixed ratio and group ratio tests) or debt deductions (third party debt test) exceed the earnings limit";
const ALLOWED: &str = "s 820-50: debt deductions less the total disallowed amount";

/// The year's debt deductions and, given interest income, net debt deductions.
fn opening(items: &[Deduction], interest: Option<&Decimal>) -> Calc<Vec<Step>> {
    let amounts: Vec<&Decimal> = items.iter().map(|x| &x.amount).collect();
    let dd = sum_step("debt_deductions", DD, &amounts)?;
    let Some(i) = interest else {
        return Ok(vec![dd]);
    };
    let ndd = dd.value.sub(i)?;
    let net = step(
        "net_debt_deductions",
        NDD,
        "debt_deductions - interest_income",
        &[&dd.value, i],
        ndd,
    );
    Ok(vec![dd, net])
}

/// Tax EBITDA (s 820-52(1)) from the stated lines present: taxable income,
/// the subtracted lines, net debt deductions, then the added lines.
fn tax_ebitda(taxable: &Decimal, lines: &[(&'static str, Decimal)], ndd: &Decimal) -> Calc<Step> {
    let mut formula = String::from("max(taxable_income");
    let mut operands = vec![taxable.clone()];
    let mut value = taxable.clone();
    for (name, v) in lines.iter().filter(|(n, _)| SUBTRACTED.contains(n)) {
        formula.push_str(&format!(" - {name}"));
        operands.push(v.clone());
        value = value.sub(v)?;
    }
    formula.push_str(" + net_debt_deductions");
    operands.push(ndd.clone());
    value = value.add(ndd)?;
    for (name, v) in lines.iter().filter(|(n, _)| ADDED.contains(n)) {
        formula.push_str(&format!(" + {name}"));
        operands.push(v.clone());
        value = value.add(v)?;
    }
    formula.push_str(", 0)");
    let refs: Vec<&Decimal> = operands.iter().collect();
    Ok(step("tax_ebitda", EBITDA, &formula, &refs, max0(value)?))
}

/// Taxable income and the stated tax EBITDA lines present.
fn ebitda_lines(a: &ThinCapitalisationArgs) -> Result<(Decimal, Vec<(&'static str, Decimal)>)> {
    let taxable = required(
        a,
        "taxable_income",
        "tax EBITDA starts from it (s 820-52(1)(a))",
    )?;
    let mut lines = Vec::new();
    for name in SUBTRACTED.into_iter().chain(ADDED) {
        let v = if name == "division_40_43_deductions" {
            Some(required(
                a,
                name,
                "tax EBITDA adds it (s 820-52(1)(c)(ii)); state 0.00 if none",
            )?)
        } else {
            optional_amount(a, name)?
        };
        if let Some(v) = v {
            lines.push((name, v));
        }
    }
    Ok((taxable, lines))
}

fn interest(a: &ThinCapitalisationArgs) -> Result<Decimal> {
    required(
        a,
        "interest_income",
        "net debt deductions subtract it (s 820-50(3)(b)); state 0.00 if none",
    )
}

/// The disallowed amount's share of each debt deduction (s 820-50(2)), in
/// cents rounded cumulatively so the shares add to the total.
fn shares(items: &[Deduction], disallowed: &Decimal, dd: &Decimal) -> Calc<Value> {
    let whole = disallowed.quantize(-2, Rounding::HalfUp)?;
    let (mut running, mut shown) = (d("0"), d("0.00"));
    let mut rows = Vec::with_capacity(items.len());
    for x in items {
        let share = if dd.is_zero() {
            d("0.00")
        } else {
            running = running.add(&whole.mul(&x.amount)?.div(dd)?)?;
            let now = running.quantize(-2, Rounding::HalfUp)?;
            let share = now.sub(&shown)?;
            shown = now;
            share
        };
        let amount = x.amount.quantize(-2, Rounding::HalfUp)?;
        let mut row = Map::new();
        row.insert("label".into(), json!(x.label));
        row.insert("amount".into(), json!(amount.to_string()));
        if let Some(t) = x.third_party {
            row.insert("third_party".into(), json!(t));
        }
        row.insert("disallowed".into(), json!(share.to_string()));
        row.insert("allowed".into(), json!(amount.sub(&share)?.to_string()));
        rows.push(Value::Object(row));
    }
    Ok(Value::Array(rows))
}

fn value_of<'a>(steps: &'a [Step], id: &str) -> &'a Decimal {
    &steps
        .iter()
        .find(|s| s.id == id)
        .expect("an earlier step")
        .value
}

/// The disallowed and allowed steps after an earnings limit.
fn closing(steps: &mut Vec<Step>, limit_id: &'static str, against: &'static str) -> Calc<()> {
    let base = value_of(steps, against).clone();
    let limit = value_of(steps, limit_id).clone();
    let dd = value_of(steps, "debt_deductions").clone();
    let disallowed = max0(base.sub(&limit)?)?;
    let formula = format!("max({against} - {limit_id}, 0)");
    steps.push(step(
        "total_disallowed_amount",
        DISALLOWED,
        &formula,
        &[&base, &limit],
        disallowed.clone(),
    ));
    steps.push(step(
        "allowed_debt_deductions",
        ALLOWED,
        "debt_deductions - total_disallowed_amount",
        &[&dd, &disallowed],
        dd.sub(&disallowed)?,
    ));
    Ok(())
}

struct Worked {
    steps: Vec<Step>,
    rates: Vec<(String, String)>,
    carried: Option<Value>,
}

fn fixed_ratio(a: &ThinCapitalisationArgs, items: &[Deduction], year: &str) -> Result<Worked> {
    let interest = interest(a)?;
    let (taxable, lines) = ebitda_lines(a)?;
    let carried = carry::read(a.get("frt_disallowed_amounts"), year)?;
    precise(|| {
        let mut steps = opening(items, Some(&interest))?;
        let ebitda = tax_ebitda(&taxable, &lines, &steps[1].value)?;
        let limit = d("0.3").mul(&ebitda.value)?;
        let limit = step(
            "fixed_ratio_earnings_limit",
            "s 820-51(1): 30% of tax EBITDA",
            "fixed_ratio x tax_ebitda",
            &[&d("0.3"), &ebitda.value],
            limit,
        );
        steps.extend([ebitda, limit]);
        closing(
            &mut steps,
            "fixed_ratio_earnings_limit",
            "net_debt_deductions",
        )?;
        let (special, rows) = carry::apply(&steps, &carried)?;
        steps.push(special);
        Ok(Worked {
            steps,
            rates: vec![rate("fixed_ratio", "0.3")],
            carried: Some(rows),
        })
    })
}

/// The GR group's EBITDA (s 820-55(2) and (3)) and the group ratio test.
fn group_ratio(a: &ThinCapitalisationArgs, items: &[Deduction]) -> Result<Worked> {
    let interest = interest(a)?;
    let (taxable, lines) = ebitda_lines(a)?;
    let why = "the group ratio test needs the GR group's figures (ss 820-53 to 820-55)";
    let expense = required(a, GR_GROUP[0], why)?;
    let profit = required(a, GR_GROUP[1], why)?;
    let adjusted = required(a, GR_GROUP[2], why)?;
    let da = required(a, GR_GROUP[3], why)?;
    let negatives = negative_members(a.get(GR_GROUP[4]))?;
    if expense.lt(&d("0"))? {
        return refuse(
            "thin_capitalisation.group_ratio_negative",
            "A GR group net third party interest expense below nil gives a negative group ratio and earnings limit; whether s 820-50(1)(b) then disallows more than the net debt deductions is not settled by the sources read, so the worksheet does not decide it.",
        );
    }
    let mut formula = String::from(
        "gr_group_net_profit + gr_group_adjusted_net_third_party_interest_expense + gr_group_depreciation_and_amortisation",
    );
    let mut group = precise(|| Ok(profit.add(&adjusted)?.add(&da)?))?;
    for n in &negatives {
        formula.push_str(&format!(" - {n}"));
        group = precise(|| Ok(group.sub(n)?))?;
    }
    if !group.gt(&d("0"))? {
        return refuse(
            "thin_capitalisation.gr_group_ebitda_not_positive",
            "The GR group EBITDA is nil or less, so the group ratio test cannot be chosen (s 820-46(3)(b)).",
        );
    }
    precise(|| {
        let mut steps = opening(items, Some(&interest))?;
        let ebitda = tax_ebitda(&taxable, &lines, &steps[1].value)?;
        let gr = step(
            "gr_group_ebitda",
            "s 820-55(2) and (3): the GR group's net profit before tax, adjusted net third party interest expense and depreciation and amortisation, each member's negative entity EBITDA disregarded",
            &formula,
            &[&profit, &adjusted, &da],
            group.clone(),
        );
        let ratio = expense.div(&group)?;
        let limit = step(
            "group_ratio_earnings_limit",
            "s 820-51(2): the group ratio (s 820-53(1): GR group net third party interest expense over GR group EBITDA) times tax EBITDA",
            "group_ratio x tax_ebitda",
            &[&ratio, &ebitda.value],
            ratio.mul(&ebitda.value)?,
        );
        steps.extend([ebitda, gr, limit]);
        closing(
            &mut steps,
            "group_ratio_earnings_limit",
            "net_debt_deductions",
        )?;
        Ok(Worked {
            steps,
            rates: vec![rate("group_ratio", ratio)],
            carried: None,
        })
    })
}

/// The members' negative entity EBITDAs that s 820-55(3) disregards, each
/// below nil.
fn negative_members(given: Option<&Arg>) -> Result<Vec<Decimal>> {
    let read: Option<Vec<Decimal>> = match given {
        None => Some(vec![]),
        Some(Arg::Json(Value::Array(items))) if items.len() <= MOST_DEDUCTIONS => items
            .iter()
            .map(|v| {
                let parsed = Arg::Decimal(Decimal::parse_input(v.as_str()?).ok()?);
                signed_money(&parsed)
                    .ok()
                    .filter(|x| x.lt(&d("0")).unwrap_or(false))
            })
            .collect(),
        Some(_) => None,
    };
    read.map_or_else(
        || {
            fact(format!(
                "gr_group_negative_member_ebitda must be a list of at most {MOST_DEDUCTIONS} decimal strings below nil: each GR group member's entity EBITDA that is less than zero (s 820-55(3))."
            ))
        },
        Ok,
    )
}

fn third_party_debt(items: &[Deduction]) -> Result<Worked> {
    precise(|| {
        let mut steps = opening(items, None)?;
        let qualifying: Vec<&Decimal> = items
            .iter()
            .filter(|x| x.third_party == Some(true))
            .map(|x| &x.amount)
            .collect();
        steps.push(sum_step(
            "third_party_earnings_limit",
            "s 820-427A(1) and (2): the debt deductions attributable to debt interests satisfying the third party debt conditions",
            &qualifying,
        )?);
        closing(&mut steps, "third_party_earnings_limit", "debt_deductions")?;
        Ok(Worked {
            steps,
            rates: vec![],
            carried: None,
        })
    })
}

fn exempt(items: &[Deduction], why: &'static str) -> Result<Worked> {
    precise(|| {
        let mut steps = opening(items, None)?;
        let dd = steps[0].value.clone();
        steps.push(step("total_disallowed_amount", why, "0", &[], d("0")));
        steps.push(step(
            "allowed_debt_deductions",
            ALLOWED,
            "debt_deductions - total_disallowed_amount",
            &[&dd, &d("0")],
            dd,
        ));
        Ok(Worked {
            steps,
            rates: vec![
                rate("de_minimis_threshold", "2000000"),
                rate("assets_threshold", "0.9"),
            ],
            carried: None,
        })
    })
}

/// Subdivision 820-AA for one company and one income year at stated figures.
pub fn thin_capitalisation(a: &ThinCapitalisationArgs) -> Result<Value> {
    let year = scope(a.scope_confirmed, a.year, KIND)?;
    let t = test(a.test)?;
    let general = general_class(a.entity_class)?;
    if flag(a.plantation_forestry_entity, "plantation_forestry_entity")? {
        return refuse(
            "thin_capitalisation.plantation_forestry",
            "An Australian plantation forestry entity stays under Division 820 as it stood before the 2024 amendments (Treasury Laws Amendment (Making Multinationals Pay Their Fair Share—Integrity and Transparency) Act 2024 Schedule 2 item 146), which this worksheet does not apply.",
        );
    }
    let items = deductions(a.debt_deductions, t)?;
    check_unused(a, t)?;
    let own = precise(|| Ok(total(items.iter().map(|x| &x.amount))?))?;
    let (reason, ratio) = exemption(a, general, &own)?;
    let worked = match (reason, t) {
        (Some(_), _) if a.get("frt_disallowed_amounts").is_some() => {
            return refuse(
                "thin_capitalisation.carry_forward_unsettled",
                "Whether s 820-56 allows a deduction for earlier fixed ratio test disallowed amounts in a year Subdivision 820-AA disallows nothing is not settled by the sources read, so the worksheet does not decide it.",
            );
        }
        (Some(why), _) => exempt(&items, why)?,
        (None, Test::Fixed) => fixed_ratio(a, &items, &year)?,
        (None, Test::Group) => group_ratio(a, &items)?,
        (None, Test::ThirdParty) => third_party_debt(&items)?,
    };
    let disallowed = value_of(&worked.steps, "total_disallowed_amount").clone();
    let mut out = derived_result(KIND, &year, worked.steps, worked.rates)?;
    out["test"] = json!(a.test.text());
    out["exemption"] = json!(reason);
    if let Some(r) = ratio {
        out["australian_assets_ratio"] = json!(r.to_string());
    }
    out["debt_deductions"] = precise(|| Ok(shares(&items, &disallowed, &own)?))?;
    if let Some(rows) = worked.carried {
        out["frt_disallowed_amounts"] = rows;
    }
    Ok(out)
}
