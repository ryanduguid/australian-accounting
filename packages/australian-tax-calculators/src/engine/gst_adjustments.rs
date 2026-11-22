// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! GST adjustments and tests beside `gst` and `gst-margin-scheme`: a change in
//! creditable purpose (GST Act Division 129), annual apportionment of
//! creditable purpose (Division 131) and the financial acquisitions threshold
//! (Division 189). The citations are to the A New Tax System (Goods and
//! Services Tax) Act 1999 compilation 96 (C2026C00081, from 1 January 2026),
//! whose provisions here read the same as compilation 82 (in force on
//! 1 July 2020), and to GSTR 2000/24 and GSTR 2003/9, all read on
//! 22 November 2026.

use serde_json::{Value, json};

use super::{Arg, Result, finite_between, refuse};
use crate::decimal::Decimal;

mod annual_apportionment;
mod creditable_purpose;
mod financial_acquisitions;

pub use annual_apportionment::{AnnualApportionmentArgs, gst_annual_apportionment};
pub use creditable_purpose::{CreditablePurposeArgs, gst_creditable_purpose_change};
pub use financial_acquisitions::{FinancialAcquisitionsArgs, gst_financial_acquisitions_threshold};

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
fn condition(id: &str, authority: &str, met: bool) -> Value {
    json!({"condition": id, "authority": authority, "met": met})
}

/// The ids of the rows not met.
fn failing(rows: &[Value]) -> Vec<Value> {
    rows.iter()
        .filter(|row| row["met"] == json!(false))
        .map(|row| row["condition"].clone())
        .collect()
}
