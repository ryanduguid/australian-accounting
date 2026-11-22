// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `gst_bad_debt`: one bad debt adjustment under GST Act Division 21 for a
//! supplier (ss 21-5 and 21-10) or a recipient (ss 21-15 and 21-20), reduced
//! for a partly taxable supply (s 136-5) or a partly creditable acquisition
//! (s 136-10(1)), with GSTR 2000/2's examples.

use serde_json::{Value, json};

use super::super::{
    Arg, Result, Step, d, date_arg, derived_result, flag, money, precise, refuse, scope, step,
};
use super::{condition, failing, fraction, product};
use crate::dates;
use crate::decimal::Decimal;

const KIND: &str = "gst_bad_debt";

/// The facts as the command passes them; see the README for each.
pub struct BadDebtArgs<'a> {
    pub party: &'a Arg,
    pub event: &'a Arg,
    pub amount: &'a Arg,
    pub taxable_or_creditable: &'a Arg,
    pub cash_basis: &'a Arg,
    pub special_rule: &'a Arg,
    pub attributed: Option<&'a Arg>,
    pub overdue_from: Option<&'a Arg>,
    pub as_at: Option<&'a Arg>,
    pub prior_adjustment: Option<&'a Arg>,
    pub taxable_proportion: Option<&'a Arg>,
    pub creditable_purpose: Option<&'a Arg>,
    pub consideration_extent: Option<&'a Arg>,
    pub reduced_credit_acquisition: Option<&'a Arg>,
    pub annual_apportionment: Option<&'a Arg>,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// One of the 4 Division 21 adjustments.
struct Case {
    supplier: bool,
    recovered: bool,
    event: &'static str,
}

impl Case {
    fn increasing(&self) -> bool {
        self.supplier == self.recovered
    }

    fn section(&self) -> &'static str {
        match (self.supplier, self.recovered) {
            (true, false) => {
                "GST Act s 21-5(1): 1/11 of the amount written off, or overdue for 12 months or more"
            }
            (true, true) => "GST Act s 21-10: 1/11 of the amount recovered",
            (false, false) => {
                "GST Act s 21-15(1): 1/11 of the amount the supplier writes off, or overdue for 12 months or more"
            }
            (false, true) => "GST Act s 21-20: 1/11 of the amount paid to the supplier",
        }
    }
}

/// Party and event, each one of its allowed values.
fn case(args: &BadDebtArgs) -> Result<Case> {
    let supplier = match args.party.text() {
        Some("supplier") => true,
        Some("recipient") => false,
        _ => {
            return refuse(
                "gst_bad_debt.fact_invalid",
                "party must be one of supplier, recipient.",
            );
        }
    };
    let event = match args.event.text() {
        Some("written_off") => "written_off",
        Some("overdue_12_months") => "overdue_12_months",
        Some("recovered") => "recovered",
        _ => {
            return refuse(
                "gst_bad_debt.fact_invalid",
                "event must be one of written_off, overdue_12_months, recovered.",
            );
        }
    };
    Ok(Case {
        supplier,
        recovered: event == "recovered",
        event,
    })
}

/// An optional input: refused when given but not used, or missing when needed.
fn field<'a>(
    value: Option<&'a Arg>,
    name: &str,
    used: bool,
    needed: bool,
) -> Result<Option<&'a Arg>> {
    match (value, used, needed) {
        (Some(_), false, _) => refuse(
            "input.field_unknown",
            format!("{name} is not an input for this party and event."),
        ),
        (None, _, true) => refuse(
            "input.missing_field",
            format!("{name} is required for this party and event."),
        ),
        _ => Ok(value),
    }
}

/// The special rules that take a debt outside this command, by `special_rule`.
const SPECIAL_RULES: [(&str, &str); 6] = [
    ("none", ""),
    (
        "taxable_at_less_than_one_eleventh",
        "The supply or acquisition was taxable or creditable at less than 1/11 of the price, such as under the margin scheme: Subdivision 136-B works the adjustment (ss 136-30 to 136-45).",
    ),
    (
        "gambling_supply",
        "A bad debt from a gambling supply: Division 126 deals with it, not Division 21 (GSTR 2000/2 para 7).",
    ),
    (
        "debt_assigned",
        "The debt was assigned: Division 21 cannot operate, and an assignee has no adjustment (GSTR 2000/2 para 47).",
    ),
    (
        "transition_act_s15g",
        "A decreasing adjustment under s 15G of the GST Transition Act, which displaces s 21-5 (GSTR 2000/2 para 8A).",
    ),
    (
        "incapacitated_entity_representative",
        "A representative of an incapacitated entity: Division 58 has special rules for bad debt adjustments (s 21-99).",
    ),
];

/// A recipient's reduced credit acquisition or Division 131 credit.
fn recipient_excluded(args: &BadDebtArgs, recipient: bool) -> Result<()> {
    let stated = |value, name| -> Result<bool> {
        let given = field(value, name, recipient, recipient)?;
        Ok(given.map(|v| flag(v, name)).transpose()? == Some(true))
    };
    if stated(
        args.reduced_credit_acquisition,
        "reduced_credit_acquisition",
    )? {
        return refuse(
            "gst_bad_debt.reduced_credit_acquisition",
            "A reduced credit acquisition: s 136-10(2) reduces the adjustment by the percentage credit reduction of Division 70, which this command does not work.",
        );
    }
    if stated(args.annual_apportionment, "annual_apportionment")? {
        return refuse(
            "gst_bad_debt.annual_apportionment",
            "The credit was worked out under Division 131: s 136-10(3) turns on whether the adjustment is attributable to a tax period no later than the s 131-55 adjustment's, which this command does not work.",
        );
    }
    Ok(())
}

/// The cases this command does not work.
fn excluded(args: &BadDebtArgs, case: &Case) -> Result<()> {
    let rule = SPECIAL_RULES
        .iter()
        .find(|(name, _)| args.special_rule.text() == Some(name));
    match rule {
        None => {
            return refuse(
                "gst_bad_debt.fact_invalid",
                "special_rule must be one of none, taxable_at_less_than_one_eleventh, gambling_supply, debt_assigned, transition_act_s15g, incapacitated_entity_representative.",
            );
        }
        Some((name, why)) if *name != "none" => {
            return refuse("gst_bad_debt.special_rule", *why);
        }
        _ => {}
    }
    if case.recovered && flag(args.cash_basis, "cash_basis")? {
        return refuse(
            "gst_bad_debt.accounting_basis_changed",
            "A recovery or payment on a cash basis: the earlier adjustment needed a non-cash basis (ss 21-5(2) and 21-15(2)), so the basis has changed, which Division 159 governs.",
        );
    }
    recipient_excluded(args, !case.supplier)
}

/// Whether the debt is written off, or overdue for 12 months or more on `as_at`.
fn written_off_or_overdue(args: &BadDebtArgs, case: &Case) -> Result<bool> {
    let overdue = case.event == "overdue_12_months";
    let from = field(args.overdue_from, "overdue_from", overdue, overdue)?;
    let as_at = field(args.as_at, "as_at", overdue, overdue)?;
    match (from, as_at) {
        (Some(from), Some(as_at)) => {
            let (from, as_at) = (date_arg(from, "overdue_from")?, date_arg(as_at, "as_at")?);
            if as_at < from {
                return refuse(
                    "gst_bad_debt.dates_invalid",
                    "as_at cannot be before overdue_from.",
                );
            }
            Ok(as_at >= dates::add_months(from, 12))
        }
        _ => Ok(true),
    }
}

const QUALIFIES_SUPPLIER: &str = "GST Act s 21-5(1)(a): you made a taxable supply";
const QUALIFIES_RECIPIENT: &str =
    "GST Act s 21-15(1)(a): you made a creditable acquisition for consideration";
const ATTRIBUTED_SUPPLIER: &str = "GSTR 2000/2 para 15: the GST on the supply was attributed to an earlier tax period, or is attributed to the one in which the debt is written off";
const ATTRIBUTED_RECIPIENT: &str = "GSTR 2000/2 para 21: the input tax credit was attributed to an earlier tax period, or is attributed to the one in which you become aware the debt is written off";
const WRITTEN_OFF: &str = "GST Act ss 21-5(1)(c) and 21-15(1)(c): the debt is written off as bad, or has been overdue for 12 months or more (s 195-1 'overdue')";
const NOT_CASH: &str = "GST Act ss 21-5(2) and 21-15(2): you do not account on a cash basis";
const PRIOR: &str = "GST Act ss 21-10(a) and 21-20(a): you had the earlier adjustment under s 21-5 or s 21-15 for the debt";

/// The rows for a write-off or an overdue debt.
fn initial_rows(args: &BadDebtArgs, case: &Case) -> Result<Vec<Value>> {
    let initial = !case.recovered;
    let attributed = field(args.attributed, "attributed", initial, initial)?;
    let Some(attributed) = attributed else {
        return Ok(vec![]);
    };
    let rule = if case.supplier {
        ATTRIBUTED_SUPPLIER
    } else {
        ATTRIBUTED_RECIPIENT
    };
    Ok(vec![
        condition("attributed", rule, flag(attributed, "attributed")?),
        condition(
            "written_off_or_overdue_12_months",
            WRITTEN_OFF,
            written_off_or_overdue(args, case)?,
        ),
        condition(
            "not_cash_basis",
            NOT_CASH,
            !flag(args.cash_basis, "cash_basis")?,
        ),
    ])
}

/// The conditions for the case, each met or not.
fn conditions(args: &BadDebtArgs, case: &Case) -> Result<Vec<Value>> {
    let qualifies = if case.supplier {
        QUALIFIES_SUPPLIER
    } else {
        QUALIFIES_RECIPIENT
    };
    let prior = field(
        args.prior_adjustment,
        "prior_adjustment",
        case.recovered,
        case.recovered,
    )?;
    let mut rows = vec![condition(
        "taxable_or_creditable",
        qualifies,
        flag(args.taxable_or_creditable, "taxable_or_creditable")?,
    )];
    if let Some(prior) = prior {
        rows.push(condition(
            "prior_adjustment",
            PRIOR,
            flag(prior, "prior_adjustment")?,
        ));
    }
    rows.extend(initial_rows(args, case)?);
    Ok(rows)
}

/// The Subdivision 136-A proportion or extents given, none of them nil.
struct Reductions {
    proportion: Option<Decimal>,
    purpose: Option<Decimal>,
    extent: Option<Decimal>,
}

fn reductions(args: &BadDebtArgs, case: &Case) -> Result<Reductions> {
    let (supplier, recipient) = (case.supplier, !case.supplier);
    let given = |value: Option<&Arg>, name, used| -> Result<Option<Decimal>> {
        let Some(value) = field(value, name, used, false)? else {
            return Ok(None);
        };
        let fraction = fraction(value, name)?;
        if fraction.equals(&d("0"))? {
            return refuse(
                "gst_bad_debt.fact_conflict",
                format!(
                    "{name} of nil contradicts taxable_or_creditable: omit it for a wholly taxable supply or creditable acquisition."
                ),
            );
        }
        Ok(Some(fraction))
    };
    Ok(Reductions {
        proportion: given(args.taxable_proportion, "taxable_proportion", supplier)?,
        purpose: given(args.creditable_purpose, "creditable_purpose", recipient)?,
        extent: given(args.consideration_extent, "consideration_extent", recipient)?,
    })
}

const PARTLY_TAXABLE: &str = "GST Act s 136-5: the full adjustment times the taxable proportion";
const PARTLY_CREDITABLE: &str = "GST Act s 136-10(1): the full adjustment times the extent of creditable purpose times the extent of consideration";

/// The adjustment: 1/11 of the amount, reduced under Subdivision 136-A when a
/// proportion or extent is given.
fn adjustment(case: &Case, amount: &Decimal, r: &Reductions) -> Result<Vec<Step>> {
    let id = if case.increasing() {
        "increasing_adjustment"
    } else {
        "decreasing_adjustment"
    };
    let full = precise(|| Ok(amount.div(&d("11"))?))?;
    if r.proportion.is_none() && r.purpose.is_none() && r.extent.is_none() {
        return Ok(vec![step(
            id,
            case.section(),
            "amount / 11",
            &[amount],
            full,
        )]);
    }
    let first = step(
        "full_adjustment",
        case.section(),
        "amount / 11",
        &[amount],
        full.clone(),
    );
    let (rule, factors) = if case.supplier {
        (
            PARTLY_TAXABLE,
            [("taxable_proportion", r.proportion.as_ref()), ("", None)],
        )
    } else {
        (
            PARTLY_CREDITABLE,
            [
                ("creditable_purpose", r.purpose.as_ref()),
                ("consideration_extent", r.extent.as_ref()),
            ],
        )
    };
    let (formula, operands, value) = precise(|| product(("full_adjustment", &full), factors))?;
    Ok(vec![first, step(id, rule, &formula, &operands, value)])
}

/// GST Act Division 21 with Subdivision 136-A: one bad debt adjustment.
pub fn gst_bad_debt(args: &BadDebtArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let case = case(args)?;
    excluded(args, &case)?;
    let amount = money(args.amount)?;
    let r = reductions(args, &case)?;
    let rows = conditions(args, &case)?;
    let failing = failing(&rows);
    let applies = failing.is_empty();
    let steps = if applies {
        adjustment(&case, &amount, &r)?
    } else {
        vec![]
    };
    let mut out = derived_result(KIND, &year, steps, vec![])?;
    out["applies"] = json!(applies);
    out["direction"] = json!(if case.increasing() {
        "increasing"
    } else {
        "decreasing"
    });
    out["failing_conditions"] = Value::Array(failing);
    out["conditions"] = Value::Array(rows);
    out["consequences"] = json!([if applies {
        "The adjustment is made in the tax period to which it is attributable; this command does not work out that period."
    } else {
        "Division 21 gives no adjustment on these facts."
    }]);
    Ok(out)
}
