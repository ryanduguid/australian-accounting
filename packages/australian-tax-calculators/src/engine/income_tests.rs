// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `income_tests`: the income measures that the Medicare levy surcharge,
//! Division 293 tax, the seniors and pensioners tax offset, study and training
//! loan repayments, the dependant offsets and the super co-contribution are
//! tested against, each worked out from taxable income and the amounts its
//! definition adds back, in a step that cites the definition.
//!
//! The steps cite ITAA 1997 compilation 268 (1 October 2026), ITAA 1936
//! compilation 193 (1 October 2026), HESA compilation 105 (17 October 2026),
//! the A New Tax System (Family Assistance) Act 1999 compilation 124
//! (1 July 2026), the FBTAA compilation of 1 July 2026, SISR compilation 159
//! (1 July 2026) and the Superannuation (Government Co-contribution for Low
//! Income Earners) Act 2003 compilation 28 (1 July 2018), all read on
//! 16 November 2026. The definitions read the same in the ITAA 1997, ITAA 1936
//! and Family Assistance Act compilations of 1 July 2024, apart from the list
//! of tax free pensions and benefits, which the operator classifies.

use serde_json::{Map, Value, json};

use super::{
    Arg, Result, Step, d, derived_result, int, money, precise, py_max, rate, refuse, scope,
    step_by_name, whole_dollars,
};
use crate::decimal::{Decimal, Rounding};
use crate::texts;

const KIND: &str = "income_tests";

/// The amounts the measures are built from, in the command's order.
const COMPONENTS: [&str; 16] = [
    "taxable_income",
    "assessable_fhss_released_amount",
    "family_trust_distribution_amounts",
    "reportable_fringe_benefits_section_57a",
    "reportable_fringe_benefits_other",
    "reportable_employer_super_contributions",
    "deductible_personal_super_contributions",
    "excess_concessional_contributions",
    "net_financial_investment_loss",
    "net_rental_property_loss",
    "tax_free_government_pensions",
    "target_foreign_income",
    "exempt_foreign_employment_income",
    "child_support_paid",
    "assessable_income",
    "business_deductions",
];

/// 1 less the 47% FBT rate for the FBT years ending 31 March 2025 to 2027 (the
/// ATO's 'Fringe benefits tax - rates and thresholds', last updated 20 May
/// 2026): a section 57A employer's reportable amount is its individual
/// quasi-fringe benefits amount grossed up at 1 / (1 - rate) (FBTAA s 135Q(4)).
pub(super) const SECTION_57A_FACTOR: &str = "0.53";

/// The measures, each a step below, whose unused components the result lists.
const MEASURES: [&str; 6] = [
    "income_for_surcharge_purposes",
    "division_293_income",
    "rebate_income",
    "repayment_income",
    "adjusted_taxable_income",
    "co_contribution_total_income",
];

/// The amounts: (id, authority, formula), in the order they are worked out.
const STEPS: [(&str, &str, &str); 12] = [
    (
        "taxable_income_without_fhss",
        "ITAA 1997 s 995-1 'income for surcharge purposes' (a), ITAA 1936 s 6(1) 'rebate income' (a), HESA s 154-5(1)(a) and Family Assistance Act Sch 3 cl 2(1)(a): taxable income disregarding the assessable FHSS released amount (ITAA 1997 s 313-20), nil where that leaves no taxable income (s 4-15(1))",
        "max(0, taxable_income - assessable_fhss_released_amount)",
    ),
    (
        "reportable_fringe_benefits_total",
        "FBTAA s 135N: the sum of the reportable fringe benefits amounts (ss 135P and 135Q), from section 57A employers and others",
        "reportable_fringe_benefits_section_57a + reportable_fringe_benefits_other",
    ),
    (
        "adjusted_fringe_benefits_total",
        "Family Assistance Act Sch 3 cl 4 (ITAA 1936 s 6(1)): a section 57A employer's amounts as individual quasi-fringe benefits amounts, before FBTAA s 135Q(4) grossed them up at 1 / (1 - 47%), so x 0.53 rounded down to the dollar as the ATO's rebate income and adjusted taxable income worksheets do, and other employers' amounts in full",
        "floor(reportable_fringe_benefits_section_57a x section_57a_factor) + reportable_fringe_benefits_other",
    ),
    (
        "total_net_investment_loss",
        "ITAA 1997 s 995-1 'total net investment loss' (a) and (b)",
        "net_financial_investment_loss + net_rental_property_loss",
    ),
    (
        "reportable_superannuation_contributions",
        "ITAA 1997 s 995-1 'reportable superannuation contributions': reportable employer superannuation contributions and Subdivision 290-C deductions, reduced, but not below zero, by excess concessional contributions",
        "max(0, reportable_employer_super_contributions + deductible_personal_super_contributions - excess_concessional_contributions)",
    ),
    (
        "income_for_surcharge_purposes",
        "ITAA 1997 s 995-1 'income for surcharge purposes' (a) to (d), (a) disregarding ITAA 1936 Sch 2F s 271-105(1), which kept the family trust distribution amounts out of assessable income; the s 301-20(3) amount taken off is nil from 2024-25, when no one under 60 has reached preservation age (SISR reg 6.01(2))",
        "taxable_income_without_fhss + family_trust_distribution_amounts + reportable_fringe_benefits_total + reportable_superannuation_contributions + total_net_investment_loss",
    ),
    (
        "division_293_income",
        "ITAA 1997 s 293-20(1)(a): income for surcharge purposes disregarding reportable superannuation contributions",
        "income_for_surcharge_purposes - reportable_superannuation_contributions",
    ),
    (
        "rebate_income",
        "ITAA 1936 s 6(1) 'rebate income' (a) to (d)",
        "taxable_income_without_fhss + reportable_superannuation_contributions + total_net_investment_loss + adjusted_fringe_benefits_total",
    ),
    (
        "repayment_income",
        "HESA s 154-5(1)(a) to (e); exempt foreign employment income is net of the losses and outgoings incurred in deriving it (s 154-5(4) and (5))",
        "taxable_income_without_fhss + total_net_investment_loss + reportable_fringe_benefits_total + exempt_foreign_employment_income + reportable_superannuation_contributions",
    ),
    (
        "adjusted_taxable_income",
        "ITAA 1997 s 995-1 'adjusted taxable income for offsets', ITAA 1936 s 6(1) 'adjusted taxable income for rebates' and Family Assistance Act Sch 3 cl 2(1), disregarding cl 3 and 3A: less deductible child maintenance expenditure (cl 8)",
        "taxable_income_without_fhss + adjusted_fringe_benefits_total + target_foreign_income + total_net_investment_loss + tax_free_government_pensions + reportable_superannuation_contributions - child_support_paid",
    ),
    (
        "co_contribution_employer_contributions",
        "Superannuation (Government Co-contribution for Low Income Earners) Act 2003 s 8(1)(c) and (1A): reportable employer superannuation contributions reduced, but not below zero, by the excess concessional contributions in assessable income (ITAA 1997 s 291-15(a))",
        "max(0, reportable_employer_super_contributions - excess_concessional_contributions)",
    ),
    (
        "co_contribution_total_income",
        "Superannuation (Government Co-contribution for Low Income Earners) Act 2003 s 8(1) and (2): assessable income disregarding the assessable FHSS released amount, the reportable fringe benefits total and those contributions, less business deductions",
        "assessable_income - assessable_fhss_released_amount + reportable_fringe_benefits_total + co_contribution_employer_contributions - business_deductions",
    ),
];

/// Pairs of components where the first is part of the second, and why.
const PARTS_OF: [(&str, &str, &str); 4] = [
    (
        "taxable_income",
        "assessable_income",
        "taxable income is assessable income less deductions (ITAA 1997 s 4-15(1))",
    ),
    (
        "assessable_fhss_released_amount",
        "assessable_income",
        "the assessable FHSS released amount is included in assessable income (ITAA 1997 s 313-20)",
    ),
    (
        "excess_concessional_contributions",
        "assessable_income",
        "excess concessional contributions are included in assessable income (ITAA 1997 s 291-15(a))",
    ),
    (
        "exempt_foreign_employment_income",
        "target_foreign_income",
        "target foreign income includes it (Family Assistance Act Sch 3 cl 5(1)(b))",
    ),
];

/// Only a full-year resident's measures are worked out.
fn check_residency(residency: &Arg) -> Result<()> {
    let reason = match residency.text() {
        Some("full_year_resident") => return Ok(()),
        Some("foreign_resident") => {
            "a foreign resident pays no Medicare levy or surcharge (ITAA 1936 s 251S(1)(a)), and a foreign resident's study loan levy is worked on assessed worldwide income (HESA ss 154-16 and 154-17), which no worksheet here works out"
        }
        Some("part_year_resident" | "working_holiday_maker") => {
            "the measures were checked for a person resident for the whole year only"
        }
        _ => {
            return refuse(
                "input.residency_invalid",
                "residency must be full_year_resident, part_year_resident, working_holiday_maker or foreign_resident",
            );
        }
    };
    refuse(
        "income_tests.residency_unsupported",
        format!("income-tests works out the measures for a full-year resident only: {reason}"),
    )
}

/// The value `values` holds for `name`.
pub(super) fn value<'a>(values: &'a [(&str, Decimal)], name: &str) -> &'a Decimal {
    &values
        .iter()
        .find(|(n, _)| *n == name)
        .unwrap_or_else(|| panic!("a value for {name}"))
        .1
}

/// The value of a formula of names joined by `+` and `-`, left to right.
fn total(values: &[(&str, Decimal)], formula: &str) -> Result<Decimal> {
    let mut out = int(0);
    let mut minus = false;
    for token in formula.split(' ') {
        match token {
            "+" => minus = false,
            "-" => minus = true,
            name => {
                let v = value(values, name);
                out = if minus { out.sub(v)? } else { out.add(v)? };
            }
        }
    }
    Ok(out)
}

/// Step `id`'s unrounded amount from the inputs and the earlier steps.
fn amount(id: &str, formula: &str, values: &[(&str, Decimal)]) -> Result<Decimal> {
    let v = |name: &str| value(values, name).clone();
    Ok(match id {
        "taxable_income_without_fhss" => py_max(
            int(0),
            v("taxable_income").sub(&v("assessable_fhss_released_amount"))?,
        )?,
        "adjusted_fringe_benefits_total" => v("reportable_fringe_benefits_section_57a")
            .mul(&v("section_57a_factor"))?
            .quantize(0, Rounding::Down)?
            .add(&v("reportable_fringe_benefits_other"))?,
        "reportable_superannuation_contributions" => py_max(
            int(0),
            v("reportable_employer_super_contributions")
                .add(&v("deductible_personal_super_contributions"))?
                .sub(&v("excess_concessional_contributions"))?,
        )?,
        "co_contribution_employer_contributions" => py_max(
            int(0),
            v("reportable_employer_super_contributions")
                .sub(&v("excess_concessional_contributions"))?,
        )?,
        // The rest are sums and differences of named values.
        _ => total(values, formula)?,
    })
}

/// Every step, each worked from the inputs and the steps before it.
fn working(values: &mut Vec<(&'static str, Decimal)>) -> Result<Vec<Step>> {
    let mut steps = Vec::new();
    for (id, authority, formula) in STEPS {
        let result = amount(id, formula, values)?;
        steps.push(step_by_name(id, authority, formula, values, result.clone()));
        values.push((id, result));
    }
    Ok(steps)
}

/// The components a step uses, directly or through the steps it names.
fn components_used(id: &str, used: &mut Vec<&'static str>) {
    let (_, _, formula) = STEPS.iter().find(|(s, ..)| *s == id).expect("a step");
    let names = formula.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'));
    for name in names {
        if let Some(component) = COMPONENTS.iter().find(|c| **c == name) {
            used.push(component);
        } else if STEPS.iter().any(|(s, ..)| *s == name) {
            components_used(name, used);
        }
    }
}

/// For each measure, the components given that it does not use.
fn not_used() -> Map<String, Value> {
    MEASURES
        .iter()
        .map(|measure| {
            let mut used = Vec::new();
            components_used(measure, &mut used);
            let unused: Vec<&str> = COMPONENTS
                .into_iter()
                .filter(|c| !used.contains(c))
                .collect();
            (measure.to_string(), json!(unused))
        })
        .collect()
}

/// Each component and step by name, with its value.
pub(super) type Values = Vec<(&'static str, Decimal)>;

/// The input `key` of `inputs`, which the command always passes.
pub(super) fn named<'a>(inputs: &[(&str, &'a Arg)], key: &str) -> &'a Arg {
    inputs
        .iter()
        .find(|(k, _)| *k == key)
        .unwrap_or_else(|| panic!("{key} is an input"))
        .1
}

/// The components and every measure's step for one full-year resident, from
/// `inputs`: each of [`COMPONENTS`] and `residency`, by name. The government
/// super amounts take their income measures from here, for their own years.
pub(super) fn measures(inputs: &[(&str, &Arg)]) -> Result<(Values, Vec<Step>)> {
    let get = |key: &str| named(inputs, key);
    check_residency(get("residency"))?;
    let mut values = Vec::new();
    for key in COMPONENTS {
        values.push((key, money(get(key))?));
    }
    whole_dollars(value(&values, "taxable_income"), "taxable income")?;
    for (part, whole, why) in PARTS_OF {
        if value(&values, part).gt(value(&values, whole))? {
            return refuse(
                "income_tests.components_inconsistent",
                format!("{part} exceeds {whole}: {why}."),
            );
        }
    }
    values.push(("section_57a_factor", d(SECTION_57A_FACTOR)));
    let steps = precise(|| working(&mut values))?;
    Ok((values, steps))
}

/// The income measures for one full-year resident and income year, from
/// `inputs`: each of [`COMPONENTS`] and `residency`, `year` and
/// `scope_confirmed`, by name.
pub fn income_tests(inputs: &[(&str, &Arg)]) -> Result<Value> {
    let year = scope(
        named(inputs, "scope_confirmed"),
        named(inputs, "year"),
        KIND,
    )?;
    let (values, steps) = measures(inputs)?;
    let rates = vec![rate("section_57a_factor", SECTION_57A_FACTOR)];
    let mut out = derived_result(KIND, &year, steps, rates)?;
    out["not_used"] = Value::Object(not_used());
    let exempt_foreign = value(&values, "exempt_foreign_employment_income");
    if !exempt_foreign.is_zero() && !value(&values, "taxable_income").is_zero() {
        let warnings = out["warnings"].as_array_mut().expect("warnings");
        warnings.push(json!(texts::INCOME_TESTS_EXEMPT_FOREIGN_NOTE));
    }
    Ok(out)
}
