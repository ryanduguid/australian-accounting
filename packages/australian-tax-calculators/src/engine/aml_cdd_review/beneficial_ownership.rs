// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use num_bigint::BigUint;
use serde_json::{Value, json};

use super::{Given, KIND, choice};
use crate::engine::{Arg, Result, precise, rate, refuse, result};

const MAX_CHAINS: usize = 16;
const MAX_LAYERS: usize = 8;
const SCALE_EXPONENT: u32 = 46; // 6 * MAX_LAYERS - 2

const BO_BODY_KEYS: [&str; 7] = [
    "entity_kind",
    "customer_exemption",
    "ownership_chains",
    "board_control",
    "financial_operating_policy_control",
    "voting_power_over_50",
    "issued_share_capital_over_50",
];
const BO_NON_BODY_KEYS: [&str; 5] = [
    "entity_kind",
    "customer_exemption",
    "ownership_chains",
    "board_control",
    "financial_operating_policy_control",
];

/// Parse an exact percentage string (> 0 to 100, at most 4 decimal places) as
/// a scaled integer in ten-thousandths of 1% (1..=1_000_000).
fn parse_scaled_layer(item: &Value) -> Result<u64> {
    let bad = || {
        refuse(
            "aml_cdd_review.chains_invalid",
            "ownership_chains must be a list of up to 16 chains, each a list of 1 to 8 percentage strings above 0 to 100 with at most 4 decimal places.",
        )
    };
    let Some(s) = item.as_str() else {
        return bad();
    };
    let (whole_s, frac_s) = match s.split_once('.') {
        Some((w, f)) if !w.is_empty() && !f.is_empty() && f.len() <= 4 => (w, f),
        Some(_) => return bad(),
        None if !s.is_empty() => (s, ""),
        None => return bad(),
    };
    if !whole_s.bytes().all(|b| b.is_ascii_digit()) || !frac_s.bytes().all(|b| b.is_ascii_digit()) {
        return bad();
    }
    let Some(whole) = whole_s.parse::<u64>().ok().filter(|&w| w <= 100) else {
        return bad();
    };
    let frac = if frac_s.is_empty() {
        0
    } else {
        frac_s.parse::<u64>().unwrap() * 10u64.pow((4 - frac_s.len()) as u32)
    };
    let scaled = whole * 10_000 + frac;
    if !(1..=1_000_000).contains(&scaled) {
        return bad();
    }
    Ok(scaled)
}

/// Exact combined ownership numerator over `10^46`, with zero intermediate rounding.
fn compute_effective_ownership(arg: &Arg) -> Result<BigUint> {
    let bad = || {
        refuse(
            "aml_cdd_review.chains_invalid",
            "ownership_chains must be a list of up to 16 chains, each a list of 1 to 8 percentage strings above 0 to 100 with at most 4 decimal places.",
        )
    };
    let Arg::Json(Value::Array(chains)) = arg else {
        return bad();
    };
    if chains.len() > MAX_CHAINS {
        return bad();
    }
    let ten = BigUint::from(10u8);
    let mut total = BigUint::ZERO;
    for chain in chains {
        let Some(layers) = chain.as_array() else {
            return bad();
        };
        if layers.is_empty() || layers.len() > MAX_LAYERS {
            return bad();
        }
        let mut prod = BigUint::from(1u8);
        for layer in layers {
            prod *= BigUint::from(parse_scaled_layer(layer)?);
        }
        let lift = ten.pow(6 * (MAX_LAYERS - layers.len()) as u32);
        total += prod * lift;
    }
    let hundred_scaled = BigUint::from(100u8) * ten.pow(SCALE_EXPONENT);
    if total > hundred_scaled {
        return refuse(
            "aml_cdd_review.ownership_above_100",
            "Combined effective ownership across ownership_chains exceeds 100%.",
        );
    }
    Ok(total)
}

/// Format an exact integer scaled by `10^46` as a decimal string with at least
/// 2 decimal places and no superfluous trailing zeroes beyond the second place.
fn format_scaled_percent(num: &BigUint) -> String {
    let digits = num.to_string();
    let scale = SCALE_EXPONENT as usize;
    let (whole, frac) = if digits.len() <= scale {
        (
            "0".to_string(),
            format!("{}{digits}", "0".repeat(scale - digits.len())),
        )
    } else {
        let (w, f) = digits.split_at(digits.len() - scale);
        (w.to_string(), f.to_string())
    };
    let trimmed = frac.trim_end_matches('0');
    match trimmed.len() {
        0 => format!("{whole}.00"),
        1 => format!("{whole}.{trimmed}0"),
        _ => format!("{whole}.{trimmed}"),
    }
}

fn control_grounds(given: &Given<'_>, is_body: bool) -> Result<Vec<&'static str>> {
    let mut grounds = Vec::new();
    if is_body {
        if given.need_flag("voting_power_over_50")? {
            grounds.push(
                "AML/CTF Act 2006 s 11(1)(a): capacity to cast or control more than 50% of votes",
            );
        }
        if given.need_flag("issued_share_capital_over_50")? {
            grounds.push("AML/CTF Act 2006 s 11(1)(b): holds more than 50% of qualifying issued share capital");
        }
    }
    if given.need_flag("board_control")? {
        grounds.push(if is_body {
            "AML/CTF Act 2006 s 11(1)(c): capacity to control composition of board or governing body"
        } else {
            "AML/CTF Act 2006 s 11(2)(a): capacity to control composition of board or governing body"
        });
    }
    if given.need_flag("financial_operating_policy_control")? {
        grounds.push(if is_body {
            "AML/CTF Act 2006 s 11(1)(d): capacity to determine financial and operating policy decisions"
        } else {
            "AML/CTF Act 2006 s 11(2)(b): capacity to determine financial and operating policy decisions"
        });
    }
    Ok(grounds)
}

fn parse_bo_header(given: &Given<'_>) -> Result<(&'static str, &'static str)> {
    given.only(&BO_BODY_KEYS)?;
    let Some(entity_kind) = choice(
        given.need("entity_kind")?,
        &["body_corporate", "non_body_corporate"],
    ) else {
        return refuse(
            "aml_cdd_review.entity_kind_invalid",
            "entity_kind must be body_corporate or non_body_corporate.",
        );
    };
    if entity_kind == "non_body_corporate" {
        given.only(&BO_NON_BODY_KEYS)?;
    }
    let Some(exemption) = choice(
        given.need("customer_exemption")?,
        &[
            "none",
            "listed_public_company",
            "government_body",
            "controlled_by_listed_or_government",
            "owned_through_listed_or_government_only",
        ],
    ) else {
        return refuse(
            "aml_cdd_review.exemption_invalid",
            "customer_exemption must be none, listed_public_company, government_body, controlled_by_listed_or_government or owned_through_listed_or_government_only.",
        );
    };
    if exemption == "listed_public_company" && entity_kind != "body_corporate" {
        return refuse(
            "aml_cdd_review.exemption_inconsistent",
            "listed_public_company exemption applies only when entity_kind is body_corporate.",
        );
    }
    Ok((entity_kind, exemption))
}

fn exemption_authority(exemption: &str) -> Option<&'static str> {
    match exemption {
        "listed_public_company" => Some(
            "AML/CTF Rules 2025 s 6-7(1): listed public company subject to public disclosure requirements",
        ),
        "government_body" => Some("AML/CTF Rules 2025 s 6-7(1A): government body"),
        "controlled_by_listed_or_government" => Some(
            "AML/CTF Rules 2025 s 6-7(1B): customer controlled by a qualifying listed public company or government body",
        ),
        "owned_through_listed_or_government_only" => Some(
            "AML/CTF Rules 2025 s 6-7(1C): individual is a beneficial owner solely through a qualifying listed public company or government body owning part of the customer",
        ),
        _ => None,
    }
}

pub(super) fn beneficial_ownership(given: &Given<'_>, period: &str) -> Result<Value> {
    let (entity_kind, exemption) = parse_bo_header(given)?;
    let is_body = entity_kind == "body_corporate";
    let effective = compute_effective_ownership(given.need("ownership_chains")?)?;
    let grounds = control_grounds(given, is_body)?;
    let threshold = BigUint::from(25u8) * BigUint::from(10u8).pow(SCALE_EXPONENT);
    let owns_25 = effective >= threshold;
    let controls = !grounds.is_empty();
    let is_bo = owns_25 || controls;
    let deemed = exemption != "none";
    let rates = vec![
        rate("beneficial_ownership_threshold_percent", "25.00"),
        rate(
            "effective_ownership_percent",
            format_scaled_percent(&effective),
        ),
    ];
    let mut out = precise(|| result(KIND, period, vec![], rates))?;
    out["question"] = json!("beneficial_ownership");
    out["entity_kind"] = json!(entity_kind);
    out["owns_25_or_more"] = json!(owns_25);
    out["controls_customer"] = json!(controls);
    out["control_grounds"] = json!(grounds);
    out["is_beneficial_owner"] = json!(is_bo);
    out["beneficial_owner_cdd_deemed_established"] = json!(deemed);
    out["exemption_authority"] = json!(exemption_authority(exemption));
    out["identification_required"] = json!(!deemed && is_bo);
    out["ceo_fallback"] = json!(
        "AML/CTF Rules 2025 s 6-8(1)-(3): where a body corporate, partnership or unincorporated association has no beneficial owners or none can be established after all reasonable steps, establish the identity of the chief executive officer (or equivalent)."
    );
    Ok(out)
}
