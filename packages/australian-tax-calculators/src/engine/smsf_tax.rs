// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `smsf_tax`: the income tax calculation statement (item 13) of a complying
//! self managed superannuation fund's annual return for 2020-21 to 2026-27,
//! with exempt current pension income, the low tax and non-arm's length
//! components, the no-TFN contributions tax and the supervisory levy.
//!
//! The steps cite ITAA 1997 Division 295 as the ATO legal database gave it on
//! 22 November 2026 (ss 295-385 to 295-390 unchanged since No 10 of 2022 and
//! No 111 of 2021; s 295-545(2A), from Act No. 52 of 2024, applies to income
//! derived from 2018-19 by Schedule 7 item 5 of that Act), the Income Tax
//! Rates Act 1986 compilation 66 (1 July 2026) ss 26 and 29, the
//! Superannuation (Self Managed Superannuation Funds) Supervisory Levy
//! Imposition Regulations 2018 s 6, and the ATO's 'Instructions to complete
//! your SMSF annual return 2026' (last updated 30 May 2026). Every judgement
//! (the actuary's proportion, which income is from segregated assets or is
//! non-arm's length, deductions, offsets and credits) is the operator's.

use serde_json::{Map, Value, json};

use super::income_tests::{Values, named, total, value};
use super::{
    Arg, Result, Step, d, derived_result, finite_between, flag, int, money, precise, py_max,
    py_min, rate, refuse, scope, step_by_name, whole_dollars,
};
use crate::decimal::{Decimal, Rounding};

const KIND: &str = "smsf_tax";

/// Income Tax Rates Act 1986 s 26(1)(a) and (b).
const LOW_TAX_RATE: &str = "0.15";
const NON_ARMS_LENGTH_RATE: &str = "0.45";
/// Rates Act s 29(2): the 45% top rate of Schedule 7 Part I for 2020-21 to
/// 2026-27, plus 2%, less the 15% of s 26(1)(a).
const NO_TFN_RATE: &str = "0.32";
/// Supervisory Levy Imposition Regulations 2018 s 6: 2018-19 and each later year.
const LEVY: &str = "259";

/// Section B and C amounts, in whole dollars ('Don't show cents').
const WHOLE_DOLLARS: [&str; 8] = [
    "assessable_contributions",
    "no_tfn_contributions",
    "net_non_arms_length_income",
    "segregated_pension_income",
    "unsegregated_income",
    "segregated_non_current_income",
    "deductions",
    "contribution_deductions",
];

/// Section D amounts and the general expense amount, to the cent.
const CENTS: [&str; 7] = [
    "general_expense_non_arms_length_income",
    "non_refundable_offsets",
    "carry_forward_offsets",
    "refundable_offsets",
    "eligible_credits",
    "payg_instalments",
    "interest_charge",
];

/// Each part, the total it is part of, and why.
const PARTS_OF: [(&str, &str, &str); 3] = [
    (
        "no_tfn_contributions",
        "assessable_contributions",
        "no-TFN contributions income is an amount Subdivision 295-C includes (s 295-610(1))",
    ),
    (
        "contribution_deductions",
        "deductions",
        "they are the deductions attributable to the contributions (s 295-545(2A)(b)(ii))",
    ),
    (
        "contribution_deductions",
        "assessable_contributions",
        "the command does not take deductions attributable to contributions above the contributions",
    ),
];

/// (id, authority, formula, return label) for each amount, in order.
const STEPS: [(&str, &str, &str, &str); 20] = [
    (
        "gross_income",
        "ITAA 1997 s 295-10(1) step 2 and SAR 2026 label W: assessable contributions (Subdivision 295-C), net non-arm's length income (label U) and the fund's other ordinary and statutory income",
        "assessable_contributions + net_non_arms_length_income + segregated_pension_income + unsegregated_income + segregated_non_current_income",
        "W",
    ),
    (
        "exempt_current_pension_income",
        "ITAA 1997 s 295-385(1) (income from segregated current pension assets) and s 295-390(1) and (3) (the actuary's proportion of other income, not non-arm's length income, contributions or income from segregated non-current assets, s 295-390(2)); SAR 2026 label Y, in whole dollars, the proportion's product rounded down",
        "segregated_pension_income + floor(unsegregated_income x exempt_proportion)",
        "Y",
    ),
    (
        "total_assessable_income",
        "SAR 2026 label V: gross income less exempt current pension income",
        "gross_income - exempt_current_pension_income",
        "V",
    ),
    (
        "taxable_income",
        "ITAA 1997 s 4-15 and SAR 2026 labels O and A: total assessable income less deductions; nil for a loss",
        "max(0, total_assessable_income - deductions)",
        "A",
    ),
    (
        "loss",
        "SAR 2026 label O: the loss, which Section E carries forward (Division 36, not worked out here)",
        "max(0, deductions - total_assessable_income)",
        "",
    ),
    (
        "non_arms_length_component",
        "ITAA 1997 s 295-545(2A): the lesser of net non-arm's length income under s 295-550(1), (2), (4) and (5) plus that under s 295-550(8) and (9), and taxable income less assessable contributions plus the deductions attributable to them (not below nil)",
        "min(net_non_arms_length_income + general_expense_non_arms_length_income, max(0, taxable_income - assessable_contributions + contribution_deductions))",
        "",
    ),
    (
        "low_tax_component",
        "ITAA 1997 s 295-545(3): the rest of taxable income",
        "taxable_income - non_arms_length_component",
        "",
    ),
    (
        "tax_on_taxable_income",
        "Income Tax Rates Act 1986 s 26(1): 15% of the low tax component and 45% of the non-arm's length component",
        "low_tax_component x low_tax_rate + non_arms_length_component x non_arms_length_rate",
        "T1",
    ),
    (
        "no_tfn_contributions_tax",
        "ITAA 1997 ss 295-605 and 295-610 and Income Tax Rates Act 1986 s 29(2): the 45% top rate plus 2% less 15%",
        "no_tfn_contributions x no_tfn_rate",
        "J",
    ),
    (
        "gross_tax",
        "ITAA 1997 s 295-10(1) steps 1 and 5; SAR 2026 label B",
        "tax_on_taxable_income + no_tfn_contributions_tax",
        "B",
    ),
    (
        "subtotal_1",
        "ITAA 1997 s 295-10(1) step 6; SAR 2026 label T2: gross tax less non-refundable non-carry forward tax offsets (label C), not below nil",
        "max(0, gross_tax - non_refundable_offsets)",
        "T2",
    ),
    (
        "non_refundable_offsets_lost",
        "SAR 2026 label T2: the non-refundable non-carry forward tax offsets above gross tax are lost",
        "max(0, non_refundable_offsets - gross_tax)",
        "",
    ),
    (
        "subtotal_2",
        "SAR 2026 label T3: subtotal 1 less non-refundable carry forward tax offsets (label D), not below nil",
        "max(0, subtotal_1 - carry_forward_offsets)",
        "T3",
    ),
    (
        "carry_forward_offsets_unused",
        "SAR 2026 label T3: the carry forward tax offsets above subtotal 1, which Division 65 may carry forward",
        "max(0, carry_forward_offsets - subtotal_1)",
        "",
    ),
    (
        "tax_payable",
        "SAR 2026 label T5: subtotal 2 less refundable tax offsets (label E), not below nil",
        "max(0, subtotal_2 - refundable_offsets)",
        "T5",
    ),
    (
        "tax_offset_refunds",
        "SAR 2026 label I: the refundable tax offsets above subtotal 2",
        "max(0, refundable_offsets - subtotal_2)",
        "I",
    ),
    (
        "supervisory_levy",
        "Superannuation (Self Managed Superannuation Funds) Supervisory Levy Imposition Regulations 2018 s 6; SAR 2026 label L: the levy for the next year, paid with this return",
        "levy",
        "L",
    ),
    (
        "levy_adjustment_wound_up",
        "SAR 2026 label M and table 9: an existing fund wound up in the year pays no levy for the next year",
        "levy",
        "M",
    ),
    (
        "levy_adjustment_new_fund",
        "SAR 2026 label N and table 9: a newly registered fund not wound up in the year also pays the levy for this year",
        "levy",
        "N",
    ),
    (
        "amount_due_or_refundable",
        "SAR 2026 label S: tax payable, the s 102AAM interest charge (G), the levy (L) and the new fund adjustment (N), less eligible credits (H), tax offset refunds (I), PAYG instalments (K) and the wound up adjustment (M); negative is a refund",
        "tax_payable + interest_charge + supervisory_levy + levy_adjustment_new_fund - eligible_credits - tax_offset_refunds - payg_instalments - levy_adjustment_wound_up",
        "S",
    ),
];

/// `max(0, a - b)` for the 2 names in `formula`.
fn excess(values: &[(&str, Decimal)], formula: &str) -> Result<Decimal> {
    let inner = formula.trim_start_matches("max(0, ").trim_end_matches(')');
    Ok(py_max(int(0), total(values, inner)?)?)
}

/// Step `id`'s amount from the inputs and the earlier steps.
fn amount(id: &str, formula: &str, values: &[(&str, Decimal)]) -> Result<Decimal> {
    let v = |name: &str| value(values, name).clone();
    Ok(match id {
        "exempt_current_pension_income" => v("unsegregated_income")
            .mul(&v("exempt_proportion"))?
            .quantize(0, Rounding::Down)?
            .add(&v("segregated_pension_income"))?,
        "non_arms_length_component" => py_min(
            v("net_non_arms_length_income").add(&v("general_expense_non_arms_length_income"))?,
            excess(
                values,
                "max(0, taxable_income - assessable_contributions + contribution_deductions)",
            )?,
        )?,
        "tax_on_taxable_income" => v("low_tax_component")
            .mul(&v("low_tax_rate"))?
            .add(&v("non_arms_length_component").mul(&v("non_arms_length_rate"))?)?,
        "no_tfn_contributions_tax" => v("no_tfn_contributions").mul(&v("no_tfn_rate"))?,
        _ if formula == "0" => int(0),
        _ if formula.starts_with("max(0, ") => excess(values, formula)?,
        _ => total(values, formula)?,
    })
}

const NON_COMPLYING: &str = "A non-complying fund is taxed at 45% on all its taxable income (Income Tax Rates Act 1986 s 26(2)), takes its franking credits as a non-refundable offset and may have assessable income from a change of status (ss 295-320 and 295-325); this worksheet takes a complying fund only.";

const FOREIGN: &str = "A fund that is not an Australian superannuation fund works out its taxable income as if its trustee were not an Australian resident (ITAA 1997 s 295-10(1) step 3(b)); this worksheet takes an Australian superannuation fund only.";

const DISREGARDED: &str = "Disregarded small fund assets are not segregated current pension assets (ITAA 1997 ss 295-385(7) and 295-387): give their income as unsegregated_income, with the actuary's proportion.";

/// The fund's status: complying and Australian, or refused.
fn check_fund(inputs: &[(&str, &Arg)]) -> Result<()> {
    if !flag(named(inputs, "complying_fund"), "complying_fund")? {
        return refuse("smsf_tax.non_complying", NON_COMPLYING);
    }
    let australian = named(inputs, "australian_superannuation_fund");
    if !flag(australian, "australian_superannuation_fund")? {
        return refuse("smsf_tax.foreign_fund", FOREIGN);
    }
    Ok(())
}

/// The inputs as values, each checked, with the rates and levy flags.
fn values(inputs: &[(&str, &Arg)]) -> Result<Values> {
    let mut values = Vec::new();
    for key in WHOLE_DOLLARS {
        let amount = money(named(inputs, key))?;
        whole_dollars(&amount, key)?;
        values.push((key, amount));
    }
    for key in CENTS {
        values.push((key, money(named(inputs, key))?));
    }
    let Some(proportion) = finite_between(named(inputs, "exempt_proportion"), "0", "1")? else {
        return refuse(
            "smsf_tax.proportion_invalid",
            "exempt_proportion must be the actuary's proportion as a decimal from 0 to 1 (ITAA 1997 s 295-390(3)).",
        );
    };
    values.push(("exempt_proportion", proportion));
    for (part, whole, why) in PARTS_OF {
        if value(&values, part).gt(value(&values, whole))? {
            return refuse(
                "smsf_tax.part_exceeds_total",
                format!("{part} exceeds {whole}: {why}."),
            );
        }
    }
    let disregarded = flag(
        named(inputs, "disregarded_small_fund_assets"),
        "disregarded_small_fund_assets",
    )?;
    if disregarded && !value(&values, "segregated_pension_income").is_zero() {
        return refuse("smsf_tax.disregarded_small_fund_assets", DISREGARDED);
    }
    values.extend(levy_flags(inputs)?);
    Ok(values)
}

/// The rates, the levy and whether table 9 puts it at M or N, as 1 or 0.
fn levy_flags(inputs: &[(&str, &Arg)]) -> Result<Values> {
    let new = flag(named(inputs, "newly_registered"), "newly_registered")?;
    let wound_up = flag(named(inputs, "wound_up"), "wound_up")?;
    let one_if = |b: bool| int(i64::from(b));
    Ok(vec![
        ("low_tax_rate", d(LOW_TAX_RATE)),
        ("non_arms_length_rate", d(NON_ARMS_LENGTH_RATE)),
        ("no_tfn_rate", d(NO_TFN_RATE)),
        ("levy", d(LEVY)),
        ("wound_up_existing_fund", one_if(wound_up && !new)),
        ("new_fund_not_wound_up", one_if(new && !wound_up)),
    ])
}

/// Table 9: the levy at M for an existing fund wound up in the year and at N
/// for a new fund not wound up; nil otherwise.
fn levy_formula(id: &str, formula: &'static str, values: &[(&str, Decimal)]) -> &'static str {
    let off = |flag: &str| value(values, flag).is_zero();
    match id {
        "levy_adjustment_wound_up" if off("wound_up_existing_fund") => "0",
        "levy_adjustment_new_fund" if off("new_fund_not_wound_up") => "0",
        _ => formula,
    }
}

/// Every step, each worked from the inputs and the steps before it.
fn working(values: &mut Values) -> Result<Vec<Step>> {
    let mut steps = Vec::new();
    for (id, authority, formula, _) in STEPS {
        let formula = levy_formula(id, formula, values);
        let result = amount(id, formula, values)?;
        steps.push(step_by_name(id, authority, formula, values, result.clone()));
        values.push((id, result));
    }
    Ok(steps)
}

/// The fund's tax, levy and amount due or refundable for one income year,
/// from `inputs` by name.
pub fn smsf_tax(inputs: &[(&str, &Arg)]) -> Result<Value> {
    let year = scope(
        named(inputs, "scope_confirmed"),
        named(inputs, "year"),
        KIND,
    )?;
    check_fund(inputs)?;
    let mut values = values(inputs)?;
    let steps = precise(|| working(&mut values))?;
    let rates = vec![
        rate("low_tax_rate", LOW_TAX_RATE),
        rate("non_arms_length_rate", NON_ARMS_LENGTH_RATE),
        rate("no_tfn_rate", NO_TFN_RATE),
        rate("levy", LEVY),
    ];
    let mut out = derived_result(KIND, &year, steps, rates)?;
    out["return_labels"] = Value::Object(
        STEPS
            .iter()
            .filter(|(.., label)| !label.is_empty())
            .map(|(id, .., label)| ((*id).to_owned(), json!(label)))
            .collect::<Map<_, _>>(),
    );
    Ok(out)
}
