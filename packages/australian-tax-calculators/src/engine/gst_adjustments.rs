// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! GST adjustments and tests beside `gst` and `gst-margin-scheme`: a change in
//! creditable purpose (GST Act Division 129), annual apportionment of
//! creditable purpose (Division 131), the financial acquisitions threshold
//! (Division 189), the supply of a going concern (s 38-325), second-hand goods
//! (Subdivision 66-A) and bad debts (Division 21 with Subdivision 136-A). The
//! citations are to the A New Tax System (Goods and Services Tax) Act 1999
//! compilation 96 (C2026C00081, from 1 January 2026), whose provisions here
//! read the same as compilation 82 (in force on 1 July 2020), and to
//! GSTR 2000/24, GSTR 2003/9, GSTR 2002/5, GSTD 2012/6 and GSTR 2000/2, all
//! read on 22 November 2026.

use serde_json::{Value, json};

use super::{Arg, Result, finite_between, refuse};
use crate::decimal::Decimal;

mod annual_apportionment;
mod bad_debts;
mod creditable_purpose;
mod financial_acquisitions;
mod going_concern;
mod second_hand_goods;

pub use annual_apportionment::{AnnualApportionmentArgs, gst_annual_apportionment};
pub use bad_debts::{BadDebtArgs, gst_bad_debt};
pub use creditable_purpose::{CreditablePurposeArgs, gst_creditable_purpose_change};
pub use financial_acquisitions::{FinancialAcquisitionsArgs, gst_financial_acquisitions_threshold};
pub use going_concern::{GoingConcernArgs, gst_going_concern};
pub use second_hand_goods::{SecondHandGoodsArgs, gst_second_hand_goods};

/// A finite Decimal fraction from 0 to 1, as a percentage over 100.
fn fraction(value: &Arg, name: &str) -> Result<Decimal> {
    match finite_between(value, "0", "1")? {
        Some(v) => Ok(v),
        None => refuse(
            "input.fraction_invalid",
            format!("{name} must be a finite Decimal fraction from 0 to 1."),
        ),
    }
}

/// One condition row.
pub(super) fn condition(id: &str, authority: &str, met: bool) -> Value {
    json!({"condition": id, "authority": authority, "met": met})
}

/// The ids of the rows not met.
pub(super) fn failing(rows: &[Value]) -> Vec<Value> {
    rows.iter()
        .filter(|row| row["met"] == json!(false))
        .map(|row| row["condition"].clone())
        .collect()
}

/// `first` times each factor given: the formula naming them, its operands and
/// the product.
fn product<'a>(
    first: (&str, &'a Decimal),
    rest: [(&str, Option<&'a Decimal>); 2],
) -> Result<(String, Vec<&'a Decimal>, Decimal)> {
    let (mut formula, mut operands, mut value) =
        (first.0.to_string(), vec![first.1], first.1.clone());
    for (name, factor) in rest {
        if let Some(factor) = factor {
            formula = format!("{formula} x {name}");
            operands.push(factor);
            value = value.mul(factor)?;
        }
    }
    Ok((formula, operands, value))
}
