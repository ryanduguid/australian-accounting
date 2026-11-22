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
    pub taxable_at_less_than_one_eleventh: &'a Arg,
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

/// The cases this command does not work.
fn excluded(args: &BadDebtArgs, case: &Case) -> Result<()> {
    if flag(
        args.taxable_at_less_than_one_eleventh,
        "taxable_at_less_than_one_eleventh",
    )? {
        return refuse(
            "gst_bad_debt.less_than_one_eleventh",
            "A supply or acquisition taxable or creditable at less than 1/11 of the price, such as under the margin scheme: Subdivision 136-B works the adjustment instead (ss 136-30 to 136-45), which this command does not work.",
        );
    }
    let recipient = !case.supplier;
    let reduced = field(
        args.reduced_credit_acquisition,
        "reduced_credit_acquisition",
        recipient,
        recipient,
    )?;
    if reduced
        .map(|v| flag(v, "reduced_credit_acquisition"))
        .transpose()?
        == Some(true)
    {
        return refuse(
            "gst_bad_debt.reduced_credit_acquisition",
            "A reduced credit acquisition: s 136-10(2) reduces the adjustment by the percentage credit reduction of Division 70, which this command does not work.",
        );
    }
    let annual = field(
        args.annual_apportionment,
        "annual_apportionment",
        recipient,
        recipient,
    )?;
    if annual
        .map(|v| flag(v, "annual_apportionment"))
        .transpose()?
        == Some(true)
    {
        return refuse(
            "gst_bad_debt.annual_apportionment",
            "The credit was worked out under Division 131: s 136-10(3) turns on whether the adjustment is attributable to a tax period no later than the s 131-55 adjustment's, which this command does not work.",
        );
    }
    Ok(())
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
const WRITTEN_OFF: &str = "GST Act ss 21-5(1)(c) and 21-15(1)(c): the debt is written off as bad, or has been overdue for 12 months or more (s 195-1 'overdue')";
const NOT_CASH: &str = "GST Act ss 21-5(2) and 21-15(2): you do not account on a cash basis";
const PRIOR: &str = "GST Act ss 21-10(a) and 21-20(a): you had the earlier adjustment under s 21-5 or s 21-15 for the debt";

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
    match prior {
        Some(prior) => rows.push(condition(
            "prior_adjustment",
            PRIOR,
            flag(prior, "prior_adjustment")?,
        )),
        None => {
            rows.push(condition(
                "written_off_or_overdue_12_months",
                WRITTEN_OFF,
                written_off_or_overdue(args, case)?,
            ));
            rows.push(condition(
                "not_cash_basis",
                NOT_CASH,
                !flag(args.cash_basis, "cash_basis")?,
            ));
        }
    }
    Ok(rows)
}

const PARTLY_TAXABLE: &str = "GST Act s 136-5: the full adjustment times the taxable proportion";
const PARTLY_CREDITABLE: &str = "GST Act s 136-10(1): the full adjustment times the extent of creditable purpose times the extent of consideration";

/// The adjustment: 1/11 of the amount, reduced under Subdivision 136-A when a
/// proportion or extent is given.
fn adjustment(args: &BadDebtArgs, case: &Case, amount: &Decimal) -> Result<Vec<Step>> {
    let (supplier, recipient) = (case.supplier, !case.supplier);
    let given = |value: Option<&Arg>, name, used| -> Result<Option<Decimal>> {
        field(value, name, used, false)?
            .map(|v| fraction(v, name))
            .transpose()
    };
    let proportion = given(args.taxable_proportion, "taxable_proportion", supplier)?;
    let purpose = given(args.creditable_purpose, "creditable_purpose", recipient)?;
    let extent = given(args.consideration_extent, "consideration_extent", recipient)?;
    let id = if case.increasing() {
        "increasing_adjustment"
    } else {
        "decreasing_adjustment"
    };
    let full = precise(|| Ok(amount.div(&d("11"))?))?;
    let reduced = proportion.is_some() || purpose.is_some() || extent.is_some();
    if !reduced {
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
    let (rule, factors) = if supplier {
        (
            PARTLY_TAXABLE,
            [("taxable_proportion", proportion.as_ref()), ("", None)],
        )
    } else {
        (
            PARTLY_CREDITABLE,
            [
                ("creditable_purpose", purpose.as_ref()),
                ("consideration_extent", extent.as_ref()),
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
    let rows = conditions(args, &case)?;
    let failing = failing(&rows);
    let applies = failing.is_empty();
    let steps = if applies {
        adjustment(args, &case, &amount)?
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
