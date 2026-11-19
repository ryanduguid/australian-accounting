// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! `dasp_tax`: the tax on a departing Australia superannuation payment, by
//! component, at the ordinary or the working holiday maker rates. The
//! citations are to the Superannuation (Departing Australia Superannuation
//! Payments Tax) Act 2007 ss 4 to 6 (compilation 5, from the Federal Register
//! of Legislation) and the Income Tax Assessment Act 1997 ss 301-170 and
//! 301-175 (from the ATO's legal database), all read on 19 November 2026.

use serde_json::{Value, json};

use super::{
    Arg, Result, Step, cents, d, derived_result, flag, int, money, rate, refuse, scope, step,
};
use crate::decimal::Decimal;

const KIND: &str = "dasp_tax";

/// The facts as the command passes them; see the README for each.
pub struct DaspTaxArgs<'a> {
    pub tax_free_component: &'a Arg,
    pub taxed_element: &'a Arg,
    pub untaxed_element: &'a Arg,
    pub working_holiday_maker_contributions: &'a Arg,
    pub unclaimed_money_rollover: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

const PAYMENT: &str = "ITAA 1997 s 301-170: the superannuation lump sum paid as a departing Australia superannuation payment, its tax free component and the elements taxed and untaxed in the fund of its taxable component";
const TAX_FREE: &str = "DASP Tax Act s 5(1)(a): nil for the tax free component";
const TAXED: &str = "DASP Tax Act s 5(1)(b): 35% of the element taxed in the fund";
const UNTAXED: &str = "DASP Tax Act s 5(1)(c): 45% of the element untaxed in the fund";
const TAXED_WHM: &str = "DASP Tax Act s 5(3): paid on or after 1 July 2017 and including amounts attributable to contributions made whilst a working holiday maker, so 65% of the element taxed in the fund in place of s 5(1)(b)'s 35%";
const UNTAXED_WHM: &str = "DASP Tax Act s 5(3): paid on or after 1 July 2017 and including amounts attributable to contributions made whilst a working holiday maker, so 65% of the element untaxed in the fund in place of s 5(1)(c)'s 45%";
const TOTAL: &str = "DASP Tax Act s 4 and ITAA 1997 s 301-175(2): the tax on the payment, the sum of the tax on each component; the Act states no rounding, so this worksheet rounds each to the cent (0.5 cents up) and the tax and the net payment add back to the payment";
const NET: &str = "ITAA 1997 s 301-175(1): the payment is not assessable income and not exempt income, so the amount left after the tax is what the person receives, with nothing to include in a tax return";

/// The tax on each component's step and the rates it was worked at.
type Components = ([Step; 3], Vec<(String, String)>);

/// The tax on each component, at the ordinary or the working holiday maker
/// rates, with those rates.
fn component_steps(
    tax_free: &Decimal,
    taxed: &Decimal,
    untaxed: &Decimal,
    whm: bool,
) -> Result<Components> {
    let (taxed_rate, untaxed_rate, taxed_rule, untaxed_rule) = if whm {
        ("0.65", "0.65", TAXED_WHM, UNTAXED_WHM)
    } else {
        ("0.35", "0.45", TAXED, UNTAXED)
    };
    let free_tax = step(
        "tax_free_component_tax",
        TAX_FREE,
        "tax_free_component x tax_free_component_rate",
        &[tax_free, &int(0)],
        tax_free.mul(&int(0))?,
    );
    let taxed_tax = step(
        "taxed_element_tax",
        taxed_rule,
        "cents(taxed_element x taxed_element_rate)",
        &[taxed, &d(taxed_rate)],
        cents(taxed.mul(&d(taxed_rate))?)?,
    );
    let untaxed_tax = step(
        "untaxed_element_tax",
        untaxed_rule,
        "cents(untaxed_element x untaxed_element_rate)",
        &[untaxed, &d(untaxed_rate)],
        cents(untaxed.mul(&d(untaxed_rate))?)?,
    );
    let rates = vec![
        rate("tax_free_component_rate", "0"),
        rate("taxed_element_rate", taxed_rate),
        rate("untaxed_element_rate", untaxed_rate),
    ];
    Ok(([free_tax, taxed_tax, untaxed_tax], rates))
}

pub fn dasp_tax(args: &DaspTaxArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let tax_free = money(args.tax_free_component)?;
    let taxed = money(args.taxed_element)?;
    let untaxed = money(args.untaxed_element)?;
    let key = "working_holiday_maker_contributions";
    let whm = flag(args.working_holiday_maker_contributions, key)?;
    let rollover = flag(args.unclaimed_money_rollover, "unclaimed_money_rollover")?;
    if rollover && !untaxed.is_zero() {
        return refuse(
            "dasp_tax.unclaimed_money_rollover",
            "The element untaxed in the fund of a roll-over paid under s 20H(2), (2AA) or (2A) of the Superannuation (Unclaimed Money and Lost Members) Act 1999 bears nil on any excess untaxed roll-over amount (DASP Tax Act s 5(2)), which this command does not take.",
        );
    }
    let payment = tax_free.add(&taxed)?.add(&untaxed)?;
    if payment.is_zero() {
        return refuse(
            "dasp_tax.payment_nil",
            "A departing Australia superannuation payment of nil has no tax; give its components.",
        );
    }
    let payment = step(
        "payment",
        PAYMENT,
        "tax_free_component + taxed_element + untaxed_element",
        &[&tax_free, &taxed, &untaxed],
        payment,
    );
    let ([free_tax, taxed_tax, untaxed_tax], rates) =
        component_steps(&tax_free, &taxed, &untaxed, whm)?;
    let total = free_tax
        .value
        .add(&taxed_tax.value)?
        .add(&untaxed_tax.value)?;
    let total = step(
        "dasp_tax",
        TOTAL,
        "tax_free_component_tax + taxed_element_tax + untaxed_element_tax",
        &[&free_tax.value, &taxed_tax.value, &untaxed_tax.value],
        total,
    );
    let net = step(
        "net_payment",
        NET,
        "payment - dasp_tax",
        &[&payment.value, &total.value],
        payment.value.sub(&total.value)?,
    );
    let steps = vec![payment, free_tax, taxed_tax, untaxed_tax, total, net];
    let mut out = derived_result(KIND, &year, steps, rates)?;
    out["rate_basis"] = json!(if whm {
        "working_holiday_maker"
    } else {
        "ordinary"
    });
    Ok(out)
}
