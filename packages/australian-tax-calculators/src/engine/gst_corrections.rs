// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! GST corrections and credits pending a tax invoice: whether an error in an
//! earlier tax period may be corrected in a later GST return under the A New
//! Tax System (Goods and Services Tax) (Correcting GST Errors) Determination
//! 2023 (F2023L01284) ss 6 and 7, and when an input tax credit waits for a tax
//! invoice (GST Act s 29-10(3), s 29-80 and GST Regulations reg 29-80.01) and
//! lapses (Division 93). The citations are to the GST Act compilation 96
//! (C2026C00081), the GST Regulations 2019 compilation 5 (F2025C01015) and the
//! Taxation Administration Act 1953 Sch 1 s 155-35, all read on 24 November
//! 2026, on facts the caller states.

use serde_json::{Value, json};

use super::gst_adjustments::{condition, failing};
use super::{Arg, Result, d, date_arg, flag, money, refuse, result, scope};
use crate::dates;
use crate::decimal::Decimal;

mod credit_time_limit;
pub use credit_time_limit::{CreditTimeLimitArgs, gst_credit_time_limit};

const KIND: &str = "gst_error_correction";

/// The facts as the command passes them; see the README for each.
pub struct ErrorCorrectionArgs<'a> {
    pub error_kind: &'a Arg,
    pub earlier_period_start: &'a Arg,
    pub earlier_return_due: &'a Arg,
    pub lodged: &'a Arg,
    pub current_gst_turnover: &'a Arg,
    pub net_debit_errors: &'a Arg,
    pub within_period_of_review: &'a Arg,
    pub compliance_activity: &'a Arg,
    pub commissioner_permits: &'a Arg,
    pub already_corrected: &'a Arg,
    pub reckless_or_intentional: &'a Arg,
    pub credit_ceased_under_division_93: &'a Arg,
    pub unrefundable_under_section_105_65: &'a Arg,
    pub year: &'a Arg,
    pub scope_confirmed: &'a Arg,
}

/// Whether `day` falls in `year`, a financial year such as 2025-26.
fn in_year(day: i64, year: &str) -> bool {
    let start: i64 = year[..4].parse().expect("a supported year");
    let from = dates::parse(&format!("{start}-07-01")).expect("a date");
    let to = dates::parse(&format!("{}-06-30", start + 1)).expect("a date");
    (from..=to).contains(&day)
}

/// F2023L01284 s 7 table: by current GST turnover, the months after the due
/// date of the earlier return and the debit error value limit.
fn debit_limits(turnover: &Decimal) -> Result<(i64, &'static str)> {
    const BANDS: [(&str, i64, &str); 4] = [
        ("20000000", 18, "12500"),
        ("100000000", 12, "25000"),
        ("500000000.00", 12, "50000"),
        ("1000000000", 12, "100000"),
    ];
    for (below, months, limit) in BANDS {
        if turnover.lt(&d(below))? {
            return Ok((months, limit));
        }
    }
    Ok((12, "560000"))
}

const RELATES: &str = "F2023L01284 s 4 'error' and s 6(a): a mistake in an earlier tax period's net amount about GST, an input tax credit or an adjustment, not a credit that ceased under Division 93 (s 4(a)) or an amount the Commissioner need not refund under TAA 1953 Sch 1 s 105-65 (s 4(b))";
const STARTED: &str = "F2023L01284 s 6(b): the earlier tax period started on or after 1 July 2012";
const REVIEW: &str = "F2023L01284 s 6(c): the later GST return is lodged within the period of review for the earlier assessment (TAA 1953 Sch 1 s 155-35(2): 4 years starting the day after notice of it)";
const COMPLIANCE: &str = "F2023L01284 s 6(d): at lodgment the error is not about a matter, or a period, you were notified is subject to a compliance activity, unless the Commissioner has notified you in writing that it can be corrected";
const ONCE: &str =
    "F2023L01284 s 6(e): you have not corrected the error, to any extent, in another tax period";
const NOT_RECKLESS: &str = "F2023L01284 s 7(a): a debit error not the result of recklessness as to, or intentional disregard of, a GST law";
const TIME: &str = "F2023L01284 s 7(b): the debit error is corrected in a GST return lodged within the debit error time limit for your current GST turnover (18 months after the earlier return's due date under $20 million, 12 months otherwise)";
const VALUE: &str = "F2023L01284 s 7(c): the net sum of the debit errors is less than the debit error value limit for your current GST turnover";

struct DebitFacts {
    time_limit: i64,
    value_limit: Decimal,
    net_debit_errors: Decimal,
    reckless: bool,
}

/// The facts' dates and, for a debit error, the s 7 limits, checked.
struct Facts {
    debit: Option<DebitFacts>,
    started: i64,
    lodged: i64,
}

fn facts(args: &ErrorCorrectionArgs, year: &str) -> Result<Facts> {
    let is_debit = match args.error_kind.text() {
        Some("debit") => true,
        Some("credit") => false,
        _ => {
            return refuse(
                "gst_error_correction.fact_invalid",
                "error_kind must be one of credit, debit.",
            );
        }
    };
    let started = date_arg(args.earlier_period_start, "earlier_period_start")?;
    let due = date_arg(args.earlier_return_due, "earlier_return_due")?;
    let lodged = date_arg(args.lodged, "lodged")?;
    if !in_year(lodged, year) {
        return refuse(
            "gst_error_correction.lodged_outside_year",
            format!("lodged must fall in {year}."),
        );
    }
    if due < started || lodged < due {
        return refuse(
            "gst_error_correction.dates_invalid",
            "earlier_return_due must not be before earlier_period_start, nor lodged before earlier_return_due.",
        );
    }
    let debit = if is_debit {
        let (months, limit) = debit_limits(&money(args.current_gst_turnover)?)?;
        Some(DebitFacts {
            time_limit: dates::add_months(due, months),
            value_limit: d(limit),
            net_debit_errors: money(args.net_debit_errors)?,
            reckless: flag(args.reckless_or_intentional, "reckless_or_intentional")?,
        })
    } else {
        None
    };
    Ok(Facts {
        debit,
        started,
        lodged,
    })
}

/// The conditions of ss 6 and 7; s 7's only for a debit error.
fn conditions(args: &ErrorCorrectionArgs, f: &Facts) -> Result<Vec<Value>> {
    let notified = flag(args.compliance_activity, "compliance_activity")?;
    let permits = flag(args.commissioner_permits, "commissioner_permits")?;
    let division_93 = flag(
        args.credit_ceased_under_division_93,
        "credit_ceased_under_division_93",
    )?;
    let section_105_65 = flag(
        args.unrefundable_under_section_105_65,
        "unrefundable_under_section_105_65",
    )?;
    let mut rows = vec![
        condition("relates_to_gst", RELATES, !division_93 && !section_105_65),
        condition(
            "period_started_from_1_july_2012",
            STARTED,
            f.started >= dates::parse("2012-07-01").expect("a date"),
        ),
        condition(
            "within_period_of_review",
            REVIEW,
            flag(args.within_period_of_review, "within_period_of_review")?,
        ),
        condition("no_compliance_activity", COMPLIANCE, !notified || permits),
        condition(
            "not_already_corrected",
            ONCE,
            !flag(args.already_corrected, "already_corrected")?,
        ),
    ];
    if let Some(ref df) = f.debit {
        rows.push(condition("not_reckless", NOT_RECKLESS, !df.reckless));
        rows.push(condition(
            "within_time_limit",
            TIME,
            f.lodged <= df.time_limit,
        ));
        rows.push(condition(
            "below_value_limit",
            VALUE,
            df.net_debit_errors.lt(&df.value_limit)?,
        ));
    }
    Ok(rows)
}

/// F2023L01284 ss 6 and 7: whether the error may be corrected in the later
/// GST return, with each condition.
pub fn gst_error_correction(args: &ErrorCorrectionArgs) -> Result<Value> {
    let year = scope(args.scope_confirmed, args.year, KIND)?;
    let f = facts(args, &year)?;
    let rows = conditions(args, &f)?;
    let failing = failing(&rows);
    let mut out = result(KIND, &year, vec![], vec![])?;
    out["may_correct"] = json!(failing.is_empty());
    out["failing_conditions"] = Value::Array(failing);
    out["conditions"] = Value::Array(rows);
    if let Some(ref df) = f.debit {
        out["debit_error_time_limit"] = json!(dates::format(df.time_limit));
        out["debit_error_value_limit"] = json!(df.value_limit.to_string());
    }
    out["note"] = json!(
        "F2023L01284 s 8 provides only that an earlier-period error cannot be corrected by requesting an amendment of an assessment for a later tax period; this worksheet does not amend or revise an earlier assessment or activity statement."
    );
    Ok(out)
}
