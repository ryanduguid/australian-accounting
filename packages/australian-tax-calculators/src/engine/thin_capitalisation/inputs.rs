// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The thin capitalisation worksheet's inputs: the test, the entity class,
//! the debt deductions, the inputs a test does not use and the exemptions
//! (ss 820-35, 820-37, 820-39 and 820-46).

use serde_json::Value;

use super::{MOST_DEDUCTIONS, ThinCapitalisationArgs};
use crate::decimal::Decimal;
use crate::engine::{Arg, Result, d, flag, money, precise, refuse, signed_money};

/// The lines of tax EBITDA (s 820-52(1)) after taxable income, in the order
/// the working takes them: subtracted (s 820-52(2), (3) and (10)), then added
/// after net debt deductions (s 820-52(1)(c) and (ca)).
pub(super) const SUBTRACTED: [&str; 3] = [
    "franking_credits",
    "associate_dividends",
    "rd_notional_deductions",
];
pub(super) const ADDED: [&str; 4] = [
    "division_40_43_deductions",
    "forestry_deductions",
    "section_70_120_deductions",
    "excess_tax_ebitda_amount",
];

/// The inputs that may be negative.
pub(super) const SIGNED: [&str; 7] = [
    "average_worldwide_equity",
    "statement_worldwide_equity",
    "statement_worldwide_assets",
    "taxable_income",
    "gr_group_net_third_party_interest_expense",
    "gr_group_net_profit",
    "gr_group_adjusted_net_third_party_interest_expense",
];

pub(super) const GR_GROUP: [&str; 5] = [
    "gr_group_net_third_party_interest_expense",
    "gr_group_net_profit",
    "gr_group_adjusted_net_third_party_interest_expense",
    "gr_group_depreciation_and_amortisation",
    "gr_group_negative_member_ebitda",
];

/// One debt deduction for the year.
pub(super) struct Deduction {
    pub(super) label: String,
    pub(super) amount: Decimal,
    pub(super) third_party: Option<bool>,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Test {
    Fixed,
    Group,
    ThirdParty,
    MaxDebt,
    MinCapital,
}

/// The entity's class for the year (s 820-46(2) and ss 820-85(2), 820-185(2),
/// 820-300(2) and 820-395(2)), as the caller states it.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Class {
    General,
    NotCovered,
    OutwardFinancial,
    InwardFinancial,
    OutwardAdi,
    InwardAdi,
}

impl Class {
    pub(super) fn financial(self) -> bool {
        matches!(self, Class::OutwardFinancial | Class::InwardFinancial)
    }

    pub(super) fn adi(self) -> bool {
        matches!(self, Class::OutwardAdi | Class::InwardAdi)
    }
}

pub(super) fn fact<T>(why: impl Into<String>) -> Result<T> {
    refuse("thin_capitalisation.fact_invalid", why)
}

pub(super) fn not_used<T>(key: &str, why: &str) -> Result<T> {
    refuse(
        "thin_capitalisation.input_not_used",
        format!("{key} is not used here: {why}."),
    )
}

pub(super) fn test(arg: &Arg) -> Result<Test> {
    match arg.text() {
        Some("fixed_ratio") => Ok(Test::Fixed),
        Some("group_ratio") => Ok(Test::Group),
        Some("third_party_debt") => Ok(Test::ThirdParty),
        Some("maximum_allowable_debt") => Ok(Test::MaxDebt),
        Some("minimum_capital_amount") => Ok(Test::MinCapital),
        _ => fact(
            "test must be fixed_ratio, group_ratio, third_party_debt, maximum_allowable_debt or minimum_capital_amount.",
        ),
    }
}

pub(super) fn class(arg: &Arg) -> Result<Class> {
    match arg.text() {
        Some("general_class_investor") => Ok(Class::General),
        Some("not_covered") => Ok(Class::NotCovered),
        Some("outward_investing_financial_entity") => Ok(Class::OutwardFinancial),
        Some("inward_investing_financial_entity") => Ok(Class::InwardFinancial),
        Some("outward_investing_adi") => Ok(Class::OutwardAdi),
        Some("inward_investing_adi") => Ok(Class::InwardAdi),
        _ => fact(
            "entity_class must be general_class_investor, not_covered, outward_investing_financial_entity, inward_investing_financial_entity, outward_investing_adi or inward_investing_adi.",
        ),
    }
}

/// The tests each class has: Subdivision 820-AA's 3 for a general class
/// investor (and an entity no Subdivision covers), the maximum allowable debt
/// or the third party debt choice for a financial entity (ss 820-85(2C) and
/// 820-185(2C)), and the minimum capital amount for an ADI.
pub(super) fn check_test(t: Test, c: Class) -> Result<()> {
    let fits = match c {
        Class::General | Class::NotCovered => {
            matches!(t, Test::Fixed | Test::Group | Test::ThirdParty)
        }
        Class::OutwardFinancial | Class::InwardFinancial => {
            matches!(t, Test::MaxDebt | Test::ThirdParty)
        }
        Class::OutwardAdi | Class::InwardAdi => t == Test::MinCapital,
    };
    match (fits, c) {
        (true, _) => Ok(()),
        (false, Class::General | Class::NotCovered) => {
            fact("test must be fixed_ratio, group_ratio or third_party_debt.")
        }
        (false, Class::OutwardFinancial | Class::InwardFinancial) => fact(
            "A financial entity (non-ADI) is tested by its maximum allowable debt (s 820-90 or 820-190) or, by a choice under s 820-85(2C) or 820-185(2C), the third party debt test: test must be maximum_allowable_debt or third_party_debt.",
        ),
        (false, _) => fact(
            "An ADI is tested by its minimum capital amount (s 820-305 or 820-400): test must be minimum_capital_amount.",
        ),
    }
}

pub(super) fn text_amount(v: Option<&Value>) -> Option<Decimal> {
    let parsed = Arg::Decimal(Decimal::parse_input(v?.as_str()?).ok()?);
    money(&parsed).ok()
}

pub(super) fn deduction(v: &Value, n: usize) -> Result<Deduction> {
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

pub(super) fn deductions(given: &Arg, t: Test) -> Result<Vec<Deduction>> {
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
        Test::Fixed | Test::Group | Test::MaxDebt | Test::MinCapital if marked > 0 => not_used(
            "third_party",
            "it matters only under the third party debt test (s 820-427A)",
        ),
        _ => Ok(all),
    }
}

pub(super) const NIL_CARRY: (&str, &str) = (
    "frt_disallowed_amounts",
    "a choice under s 820-46(3) or (4) makes every earlier fixed ratio test disallowed amount nil (s 820-58)",
);

/// The inputs a test does not use are refused rather than ignored.
pub(super) fn check_unused(a: &ThinCapitalisationArgs, t: Test) -> Result<()> {
    let group = "the GR group figures belong to the group ratio test (s 820-53)";
    let third = "the third party debt test compares debt deductions with the third party earnings limit alone (s 820-50(1)(c))";
    let older = "it belongs to Subdivisions 820-B to 820-E, which do not apply to a general class investor (ss 820-85(2A), 820-185(2A), 820-300(2A) and 820-395(2A))";
    let aa =
        "it belongs to Subdivision 820-AA, which tests only a general class investor (s 820-46)";
    let unused: Vec<(&str, &str)> = match t {
        Test::Fixed => GR_GROUP.iter().map(|k| (*k, group)).collect(),
        Test::Group => vec![
            (
                "excess_tax_ebitda_amount",
                "s 820-60 adds it only where no choice is made (fixed ratio test)",
            ),
            NIL_CARRY,
        ],
        Test::ThirdParty => ["taxable_income", "interest_and_equivalent_income"]
            .into_iter()
            .chain(SUBTRACTED)
            .chain(ADDED)
            .chain(GR_GROUP)
            .map(|k| (k, third))
            .chain([NIL_CARRY])
            .collect(),
        Test::MaxDebt | Test::MinCapital => vec![],
    };
    let unused = unused.into_iter().chain(match t {
        Test::Fixed | Test::Group => OLDER.iter().map(|k| (*k, older)).collect::<Vec<_>>(),
        _ => AA.iter().map(|k| (*k, aa)).collect(),
    });
    match unused.into_iter().find(|(k, _)| a.get(k).is_some()) {
        Some((k, why)) => not_used(k, why),
        None => Ok(()),
    }
}

/// Subdivision 820-AA's inputs other than s 820-37's.
const AA: [&str; 15] = [
    "interest_and_equivalent_income",
    "taxable_income",
    "franking_credits",
    "associate_dividends",
    "rd_notional_deductions",
    "division_40_43_deductions",
    "forestry_deductions",
    "section_70_120_deductions",
    "excess_tax_ebitda_amount",
    "frt_disallowed_amounts",
    "gr_group_net_third_party_interest_expense",
    "gr_group_net_profit",
    "gr_group_adjusted_net_third_party_interest_expense",
    "gr_group_depreciation_and_amortisation",
    "gr_group_negative_member_ebitda",
];

/// The inputs of Subdivisions 820-B to 820-E.
pub(super) const OLDER: [&str; 33] = [
    "whole_year",
    "inward_investment_vehicle",
    "foreign_controlled_australian_entity",
    "average_debt_capital",
    "average_associate_entity_debt",
    "average_pe_associate_entity_debt",
    "average_controlled_foreign_entity_debt",
    "average_borrowed_securities_amount",
    "average_cost_free_debt_capital",
    "average_assets",
    "average_excluded_equity_interests",
    "average_associate_entity_equity",
    "average_controlled_foreign_entity_equity",
    "average_non_debt_liabilities",
    "average_zero_capital_amount",
    "average_associate_entity_excess_amount",
    "average_on_lent_amount",
    "average_worldwide_debt",
    "average_worldwide_equity",
    "statement_worldwide_debt",
    "statement_worldwide_equity",
    "statement_worldwide_assets",
    "statement_average_australian_assets",
    "audited_consolidated_statements",
    "average_adi_equity_capital",
    "average_interest_free_pe_loans",
    "average_risk_weighted_assets",
    "average_tier_1_prudential_capital_deductions",
    "worldwide_capital_risk_weighted_assets",
    "worldwide_capital_tier_1_prudential_capital_deductions",
    "group_average_tier_1_capital",
    "group_average_risk_weighted_assets",
    "arm_length_capital_amount",
];

/// An optional amount, read as a money amount or, where it may be negative,
/// a signed one; `None` when absent.
pub(super) fn optional_amount(a: &ThinCapitalisationArgs, key: &str) -> Result<Option<Decimal>> {
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

pub(super) fn required(a: &ThinCapitalisationArgs, key: &str, why: &str) -> Result<Decimal> {
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
pub(super) fn exemption(
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
pub(super) fn assets_ratio(a: &ThinCapitalisationArgs, outward: bool) -> Result<Option<Decimal>> {
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

/// The GR group's figures and its EBITDA (s 820-55(2) and (3)).
pub(super) struct GrGroup {
    pub(super) expense: Decimal,
    pub(super) profit: Decimal,
    pub(super) adjusted: Decimal,
    pub(super) da: Decimal,
    pub(super) formula: String,
    pub(super) ebitda: Decimal,
}

pub(super) fn gr_group(a: &ThinCapitalisationArgs) -> Result<GrGroup> {
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
    let mut ebitda = precise(|| Ok(profit.add(&adjusted)?.add(&da)?))?;
    for n in &negatives {
        formula.push_str(&format!(" - {n}"));
        ebitda = precise(|| Ok(ebitda.sub(n)?))?;
    }
    if !ebitda.gt(&d("0"))? {
        return refuse(
            "thin_capitalisation.gr_group_ebitda_not_positive",
            "The GR group EBITDA is nil or less, so the group ratio test cannot be chosen (s 820-46(3)(b)).",
        );
    }
    Ok(GrGroup {
        expense,
        profit,
        adjusted,
        da,
        formula,
        ebitda,
    })
}

/// The members' negative entity EBITDAs that s 820-55(3) disregards, each
/// below nil.
pub(super) fn negative_members(given: Option<&Arg>) -> Result<Vec<Decimal>> {
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
