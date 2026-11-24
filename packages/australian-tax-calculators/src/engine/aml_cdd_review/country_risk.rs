// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use serde_json::{Value, json};

use super::{Given, KIND};
use crate::decimal::{Decimal, Rounding};
use crate::engine::{Arg, Result, d, precise, rate, refuse, result};

const COUNTRY_KEYS: [&str; 4] = [
    "basel_aml_score",
    "fatf_call_for_action",
    "fatf_increased_monitoring",
    "australian_sanctioned_jurisdiction",
];

/// Strict unsigned decimal string (digits and optional single decimal point),
/// finite, with at most `max_dp` decimal places.
fn parse_plain_decimal(text: &str, max_dp: i128) -> Option<Decimal> {
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit() || b == b'.') {
        return None;
    }
    let val = Decimal::parse(text).ok()?;
    (val.is_finite() && val.exponent() >= -max_dp).then_some(val)
}

fn parse_basel_score(arg: &Arg) -> Result<Decimal> {
    let bad = || {
        refuse(
            "aml_cdd_review.basel_score_invalid",
            "basel_aml_score must be a decimal string from 0.00 to 10.00 with at most 2 decimal places.",
        )
    };
    let Some(val) = arg.text().and_then(|t| parse_plain_decimal(t, 2)) else {
        return bad();
    };
    if val.lt(&crate::engine::int(0))? || val.gt(&crate::engine::int(10))? {
        return bad();
    }
    Ok(val.quantize(-2, Rounding::HalfUp)?)
}

fn basel_band(score: &Decimal) -> Result<&'static str> {
    if score.le(&d("5.00"))? {
        Ok("low")
    } else if score.le(&d("6.00"))? {
        Ok("medium")
    } else {
        Ok("high")
    }
}

fn parse_country_flags(given: &Given<'_>) -> Result<(bool, bool, bool)> {
    Ok((
        given.need_flag("fatf_call_for_action")?,
        given.need_flag("fatf_increased_monitoring")?,
        given.need_flag("australian_sanctioned_jurisdiction")?,
    ))
}

pub(super) fn country_risk(given: &Given<'_>, period: &str) -> Result<Value> {
    given.only(&COUNTRY_KEYS)?;
    let score = parse_basel_score(given.need("basel_aml_score")?)?;
    let (call_for_action, grey_list, sanctioned) = parse_country_flags(given)?;
    let band = precise(|| basel_band(&score))?;
    let override_applied = call_for_action || grey_list || sanctioned;
    let rating = if override_applied { "high" } else { band };
    let mut provisions = Vec::new();
    if call_for_action {
        provisions.push("AML/CTF Act 2006 s 32(d): mandatory enhanced CDD applies only if a covered individual is physically present in, or a covered body corporate or legal arrangement was formed in, this FATF Call-for-Action jurisdiction");
        provisions.push("AML/CTF Rules 2025 s 6-21: if s 32(d) applies, establish and hold source of wealth and source of funds where relevant to the nature of the customer's ML/TF risk");
    }
    let rates = vec![
        rate("basel_aml_score", &score),
        rate("basel_low_max", "5.00"),
        rate("basel_medium_max", "6.00"),
        rate("basel_high_max", "10.00"),
    ];
    let mut out = precise(|| result(KIND, period, vec![], rates))?;
    out["question"] = json!("country_risk");
    out["basel_band"] = json!(band);
    out["override_applied"] = json!(override_applied);
    out["country_risk_rating"] = json!(rating);
    out["fatf_call_for_action_ecdd_applies_if_covered_connection"] = json!(call_for_action);
    out["statutory_provisions"] = json!(provisions);
    out["guidance_authority"] = json!(
        "AUSTRAC, Step 2: Identify and assess your risks (Basel AML Index 0-5 low, 5.01-6 medium, 6.01-10.00 high; FATF grey/blacklist and Australian sanctions high-risk override)"
    );
    Ok(out)
}
